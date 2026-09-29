#!/usr/bin/env bash
# preparar.sh <commit de wisdom-paul> — Task 3 de la campaña K.
# Deja en $K_ROOT/prep/ todo lo que comparten las corridas de la fase 1:
#   kb/            snapshot de la KB en <commit> (clon sin remote)
#   index.db       índice de exo construido desde cero sobre el snapshot
#   config.toml    config de exo que apunta al snapshot (EXO_CONFIG)
#   claude-md.md   CLAUDE.md global SIN la sección «## Memoria de sesiones» (errata E1)
#   a1-inicio.json salida de SessionStart para A1: el core-index del snapshot con la
#                  instrucción de búsqueda cambiada a grep (§4)
#   stub/exo       binario falso para A0/A1 (sale 127)
#   manifiesto.txt shas de todo lo anterior
set -euo pipefail
commit=$1
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
KB_ORIG=/home/paul/Documentos/proyectos/wisdom-paul
PLUG="$H/../../../plugins/exo/scripts"
rm -rf "$P"; mkdir -p "$P/stub"

git clone -q --no-hardlinks "$KB_ORIG" "$P/kb"
git -C "$P/kb" checkout -q "$commit"
git -C "$P/kb" remote remove origin
cat > "$P/config.toml" <<EOF
schema_version = 1
[kb]
path = "$P/kb"
name = "wisdom-paul"
[index]
db = "$P/index.db"
[embeddings]
model = "jinaai/jina-embeddings-v2-base-es"
dims = 768
min_similarity = 0.35
EOF
export EXO_CONFIG="$P/config.toml" EXO_KB="$P/kb" EXO_DB="$P/index.db" EXO_INDEX="$P/index.db"
exo rebuild > "$P/rebuild.log" 2>&1

# E1: CLAUDE.md sin la sección de memoria (hasta la siguiente cabecera ## o el final).
awk '/^## Memoria de sesiones/{skip=1; next} /^## /{skip=0} !skip' "$HOME/.claude/CLAUDE.md" > "$P/claude-md.md"
# E1 (ampliación): la cabecera que apunta a la KB («fuente de verdad… en la KB… búscala»).
sed -i '/^> .*wisdom-paul.*exo/d' "$P/claude-md.md"
grep -qiE 'Memoria de sesiones|wisdom-paul|\bexo\b' "$P/claude-md.md" && { echo "E1: quedan referencias a la KB o a exo" >&2; exit 1; }

# A1: el mismo bloque de arranque que A2/A3, con la búsqueda por grep.
REFLEX_LOG_FILE=/dev/null "$PLUG/exo-recall.sh" < /dev/null > "$P/a2-inicio.json"
python3 - "$P" <<'PY'
import json, re, sys
p = sys.argv[1]
d = json.load(open(f"{p}/a2-inicio.json"))
ctx = d["hookSpecificOutput"]["additionalContext"]
nuevo, n = re.subn(r"\(exo search --type hybrid, exo targets\)", f"(grep -ril / Read sobre {p}/kb)", ctx)
if n != 1:
    sys.exit(f"A1: se esperaba 1 instrucción de búsqueda y hay {n}")
d["hookSpecificOutput"]["additionalContext"] = nuevo
json.dump(d, open(f"{p}/a1-inicio.json", "w"), ensure_ascii=False)
PY

printf '#!/bin/sh\necho "exo: orden no encontrada" >&2\nexit 127\n' > "$P/stub/exo"; chmod +x "$P/stub/exo"

{ echo "kb_commit $commit"; echo "exo $(exo --version)"
  for f in claude-md.md a1-inicio.json a2-inicio.json config.toml; do echo "$f $(sha256sum < "$P/$f" | cut -c1-16)"; done
  echo "notas $(find "$P/kb" -name '*.md' -not -path '*/.git/*' | wc -l)"
} > "$P/manifiesto.txt"
cat "$P/manifiesto.txt"
