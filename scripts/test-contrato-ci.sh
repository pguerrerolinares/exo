#!/usr/bin/env bash
# Gate: test de contrato engine↔prosa de skills contra un fixture propio.
#
# plugins/exo/scripts/test-contrato-engine.sh confronta el envelope de
# `exo search --json` (que cita la prosa de los skills) con el emitido por el
# binario REAL. Sin índice ni KB se abstiene (exit 2), y en un runner limpio
# no hay ninguno de los dos: este script los monta vía `exo init` (KB semilla
# versionada con git e índice) — con EXO_CONFIG y EXO_DB apuntando a un directorio temporal, sin tocar ~/.exo.
#
# El indexado de `init` embebe con el modelo ONNX (~0,6 GB): en CI va en el job que ya
# lo cachea. En local reutiliza la caché de HF de la máquina.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

BIN="engine/target/release/exo"
[ "${OS:-}" = "Windows_NT" ] && BIN="$BIN.exe"

# Compila si hace falta. En el job `test` de CI, `cargo test --release` ya
# dejó los artefactos: esto solo enlaza el binario.
cargo build --release --locked --manifest-path engine/Cargo.toml || {
  echo "test-contrato-ci: no compila el engine" >&2; exit 1; }
[ -x "$BIN" ] || { echo "test-contrato-ci: falta $BIN tras compilar" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
# El binario es nativo: en Windows no entiende rutas `/tmp/...` de Git Bash.
NATIVO="$TMP"
command -v cygpath >/dev/null 2>&1 && NATIVO="$(cygpath -m "$TMP")"

export EXO_CONFIG="$NATIVO/config.toml"
# EXO_DB no es opcional: `exo init` graba en la config nueva
# `db = ~/.exo/index.db` sin importar dónde viva EXO_CONFIG, y sin EXO_DB
# indexa la semilla ahí — en el índice REAL de la máquina, pisándole
# meta.kb_root. Medido al escribir este script (2026-09-12).
export EXO_DB="$NATIVO/index.db"
KB="$NATIVO/kb"

# Los runners de GitHub no traen identidad git: sin ella el commit de la
# semilla falla y `init` solo avisa, dejando una KB a medias.
if [ -z "$(git config user.email 2>/dev/null)" ]; then
  export GIT_AUTHOR_NAME=ci GIT_AUTHOR_EMAIL=ci@exo.invalid
  export GIT_COMMITTER_NAME=ci GIT_COMMITTER_EMAIL=ci@exo.invalid
fi

"$BIN" init --kb "$KB" --name contrato-ci > "$TMP/init.txt" 2>&1 || {
  echo "test-contrato-ci: exo init falló" >&2; cat "$TMP/init.txt" >&2; exit 1; }

# `init` ya versiona la KB (git init + commit) e indexa sobre EXO_DB. Se exige
# el commit porque sin él `init` terminó a medias y solo avisó: el rojo debe
# decirlo aquí, no como un fallo raro del contrato más abajo.
git -C "$TMP/kb" rev-parse --verify -q HEAD >/dev/null || {
  echo "test-contrato-ci: init no dejó commit en la KB semilla" >&2; cat "$TMP/init.txt" >&2; exit 1; }
[ -f "$TMP/index.db" ] || {
  echo "test-contrato-ci: init no creó el índice en EXO_DB" >&2; cat "$TMP/init.txt" >&2; exit 1; }

# EXO_INDEX/EXO_KB explícitos: si el test los resolviera con `exo config
# --json`, leería el `db = ~/.exo/index.db` grabado por init y verificaría el
# contrato contra el índice de la máquina — un verde que no dice nada del
# fixture.
EXO_BIN="$BIN" EXO_INDEX="$EXO_DB" EXO_KB="$KB" \
  ./plugins/exo/scripts/test-contrato-engine.sh
