#!/usr/bin/env bash
# preparar-m.sh — prep de la campaña M: snapshot de K (80cba57) + índice en cuda + MEMORY.md de N,
# las reglas del oráculo (reglas/<id>.txt) y la copia del gold heredado de K con el harness reapuntado.
# Exige K_ROOT explícito: el default de K es el prep de K. Rehacer el prep de M es intencional: preparar.sh --forzar
# mueve el anterior a prep.pisado-<fecha> en vez de borrarlo.
set -euo pipefail
H="$(cd "$(dirname "$0")" && pwd)"
PREPARAR="${PREPARAR:-$H/../ablacion-k/harness/preparar.sh}"
KB_COMMIT=80cba57878d37583be1f2654ecb7d96c51dad8ab
TARBALL="${K_TARBALL:-$HOME/.cache/exo-ablacion-k-registro.tar.gz}"
GOLD_K="${K_GOLD_ORIGEN:-$HOME/.cache/exo-ablacion-k/gold/s1}"
# Prefijo absoluto que el gold de K lleva grabado en sus check.sh (un worktree de K que ya no existe).
PREFIJO_K=/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness

[ -n "${K_ROOT:-}" ] || { echo "preparar-m: K_ROOT debe venir explícito (p. ej. ~/.cache/exo-campana-m)" >&2; exit 2; }
case "$(realpath -m "$K_ROOT")" in
  "$(realpath -m "$HOME/.cache/exo-ablacion-k")"*) echo "preparar-m: K_ROOT apunta a los datos de K; preparar.sh podría pisarlos" >&2; exit 2;;
esac
export K_ROOT; P="$K_ROOT/prep"

"$PREPARAR" --forzar "$KB_COMMIT"

export EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db"
exo rebuild > "$P/rebuild-cuda.log" 2>&1
disp=$(python3 - "$P/index.db" <<'PY'
import sqlite3, sys
r = sqlite3.connect(sys.argv[1]).execute("SELECT valor FROM meta WHERE clave = 'dispositivo_embeddings'").fetchone()
print(r[0] if r else "")
PY
) || disp=""
[ "$disp" = cuda ] || { echo "preparar-m: índice no embebido en cuda (dispositivo_embeddings='${disp}')" >&2; exit 1; }

rm -rf "$P/memoria"
python3 "$H/gen_memoria.py" "$P/kb" "$P/memoria"
echo "dispositivo_embeddings = $disp"

# Reglas del oráculo: campo `regla` byte a byte (jq -j, sin salto añadido), como evals/techo-reglas/preregistro.md.
[ -f "$TARBALL" ] || { echo "preparar-m: falta el tarball de K: $TARBALL" >&2; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
tar xzf "$TARBALL" -C "$tmp" pool/reglas-g0.jsonl pool/reglas-g1.jsonl pool/reglas-g2.jsonl
rm -rf "$K_ROOT/reglas"; mkdir -p "$K_ROOT/reglas"
for f in "$tmp"/pool/reglas-g*.jsonl; do
  while IFS= read -r id; do
    [[ "$id" =~ ^g[0-9]+-[0-9]+$ ]] || { echo "preparar-m: id de regla inválido: $id" >&2; exit 1; }
    jq -j --arg i "$id" 'select(.id == $i) | .regla' "$f" > "$K_ROOT/reglas/$id.txt"
  done < <(jq -r '.id' "$f")
done
echo "reglas: $(ls "$K_ROOT/reglas" | wc -l)"

# Gold heredado: la COPIA se reapunta al harness de este repo; el gold de K no se toca.
[ -d "$GOLD_K" ] || { echo "preparar-m: falta el gold de K: $GOLD_K" >&2; exit 1; }
HARNESS_REPO="$(cd "$H/../ablacion-k/harness" && pwd)"
# Los check.sh de g0-33, g0-122 y g2-35 resuelven `$(dirname $0)/../../harness` = gold/harness.
rm -rf "$K_ROOT/gold/s1" "$K_ROOT/gold/harness"; mkdir -p "$K_ROOT/gold"
cp -r "$GOLD_K" "$K_ROOT/gold/s1"
[ -d "$GOLD_K/../harness" ] && cp -r "$GOLD_K/../harness" "$K_ROOT/gold/harness"
chmod -R u+w "$K_ROOT/gold"   # el gold de K es de solo lectura
grep -rlF -- "$PREFIJO_K" "$K_ROOT/gold/s1" | while IFS= read -r f; do
  P_K="$PREFIJO_K" P_R="$HARNESS_REPO" perl -pi -e 's/\Q$ENV{P_K}\E/$ENV{P_R}/g' "$f"
done || true
echo "gold s1: $(ls "$K_ROOT/gold/s1" | wc -l) tareas"
