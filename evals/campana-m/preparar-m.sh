#!/usr/bin/env bash
# preparar-m.sh — prep de la campaña M: snapshot de K (80cba57) + índice en cuda + MEMORY.md de N.
# Exige K_ROOT explícito: preparar.sh hace `rm -rf "$K_ROOT/prep"` y el default de K es el prep de K.
set -euo pipefail
H="$(cd "$(dirname "$0")" && pwd)"
PREPARAR="${PREPARAR:-$H/../ablacion-k/harness/preparar.sh}"
KB_COMMIT=80cba57878d37583be1f2654ecb7d96c51dad8ab

[ -n "${K_ROOT:-}" ] || { echo "preparar-m: K_ROOT debe venir explícito (p. ej. ~/.cache/exo-campana-m)" >&2; exit 2; }
case "$(realpath -m "$K_ROOT")" in
  "$(realpath -m "$HOME/.cache/exo-ablacion-k")"*) echo "preparar-m: K_ROOT apunta a los datos de K; preparar.sh los borraría" >&2; exit 2;;
esac
export K_ROOT; P="$K_ROOT/prep"

"$PREPARAR" "$KB_COMMIT"

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
