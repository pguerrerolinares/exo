#!/usr/bin/env bash
# restaurar-prep.sh — rehace en $K_ROOT/prep/ lo que usa el brazo arp, tras perderse el prep de K
# (el 10-24 quedó pisado por un fixture: kb con solo core/, commit «x», sin claude-md.md ni stub/).
# No llama a preparar.sh: su claude-md.md sale del CLAUDE.md global de HOY y no casaría con el pin.
#   kb/           clon de la KB en el kb_commit de K (evals/ablacion-k/congelacion.txt), sin remote
#   claude-md.md  recuperado byte a byte del sysprompt.md de techo-reglas-2 (g0-122/arp-r1) menos su
#                 framing; se exige el sha256 de evals/techo-reglas-2/pins.sha256
#   stub/exo      el mismo binario falso de preparar.sh (sale 127)
#   config.toml   el de preparar.sh; sin index.db: arp no llama a exo (stub en el PATH)
#   manifiesto.txt
# El prep previo no se borra: se mueve a prep.pisado-<fecha>. Si ya está restaurado, no toca nada.
set -euo pipefail
H="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$H/../.." && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; P="$K_ROOT/prep"
KB_ORIG="${KB_ORIG:-/home/paul/Documentos/proyectos/wisdom-paul}"
COMMIT=$(awk '$1=="kb_commit"{print $2; exit}' "$ROOT/evals/ablacion-k/congelacion.txt")
PIN=$(awk '$2=="prep/claude-md.md"{print $1}' "$ROOT/evals/techo-reglas-2/pins.sha256")
SYS="$K_ROOT/corridas/g0-122/arp-r1/sysprompt.md"
[ -n "$COMMIT" ] && [ -n "$PIN" ] || { echo "restaurar-prep: sin kb_commit o sin pin de claude-md.md" >&2; exit 2; }

integro() {
  [ "$({ sha256sum < "$P/claude-md.md"; } 2>/dev/null | cut -d" " -f1)" = "$PIN" ] && [ -x "$P/stub/exo" ] \
    && [ "$(awk '$1=="kb_commit"{print $2}' "$P/manifiesto.txt" 2>/dev/null)" = "$COMMIT" ] \
    && [ "$(git -C "$P/kb" rev-parse HEAD 2>/dev/null)" = "$COMMIT" ] && [ -z "$(git -C "$P/kb" status --porcelain 2>/dev/null)" ]
}
if integro; then echo "restaurar-prep: $P ya íntegro"; exit 0; fi

[ -s "$SYS" ] || { echo "restaurar-prep: falta $SYS (fuente de claude-md.md)" >&2; exit 2; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
python3 - "$SYS" "$ROOT/evals/techo-reglas-2/framing.txt" "$ROOT/evals/techo-reglas/reglas/g0-122.txt" > "$T/claude-md.md" <<'PY'
import sys
s, f, r = (open(p, "rb").read() for p in sys.argv[1:4])
cola = b"\n" + f.replace(b"{{REGLA}}", r)
if not s.endswith(cola):
    sys.exit("restaurar-prep: el sysprompt no acaba en el framing sellado")
sys.stdout.buffer.write(s[:-len(cola)])
PY
[ "$(sha256sum < "$T/claude-md.md" | cut -d' ' -f1)" = "$PIN" ] || { echo "restaurar-prep: claude-md.md recuperado no casa con el pin" >&2; exit 1; }

[ -e "$P" ] && mv "$P" "$P.pisado-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$P/stub"
git clone -q --no-hardlinks "$KB_ORIG" "$P/kb"
git -C "$P/kb" checkout -q "$COMMIT"
git -C "$P/kb" remote remove origin
cp "$T/claude-md.md" "$P/claude-md.md"
printf '#!/bin/sh\necho "exo: orden no encontrada" >&2\nexit 127\n' > "$P/stub/exo"; chmod +x "$P/stub/exo"
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
{ echo "kb_commit $COMMIT"; echo "restaurado por evals/techo-reglas-3/restaurar-prep.sh (solo arp: sin index.db ni a1/a2)"
  echo "claude-md.md $(sha256sum < "$P/claude-md.md" | cut -c1-16)"
  echo "notas $(find "$P/kb" -name '*.md' -not -path '*/.git/*' | wc -l)"
} > "$P/manifiesto.txt"
integro || { echo "restaurar-prep: el prep restaurado no pasa la comprobación" >&2; exit 1; }
cat "$P/manifiesto.txt"
