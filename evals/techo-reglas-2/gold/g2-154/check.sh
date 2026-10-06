#!/usr/bin/env bash
# check.sh <workdir> <transcript> <commit_inicio> — g2-154: no migrar en silencio una shape publicada; la nueva va tras un flag.
# 0 cumple · 1 viola · 2 no evaluable
wd="$1"
base="$3"
[ -n "$base" ] || exit 2
git -C "$wd" diff --quiet "$base" -- kb.py 2>/dev/null && exit 2
run() { (cd "$wd" && python3 kb.py "$@" 2>/dev/null); }
# 1) el default publicado no cambia: ok y missing idénticos (claves nuevas aditivas permitidas)
old='{"ok": false, "missing": ["core/doctrina.md", "logs/bitacora.md"]}'
got="$(run doctor --check-schema)"
python3 - "$got" "$old" <<'PY' || exit 1
import json, sys
try: g = json.loads(sys.argv[1])
except Exception: sys.exit(1)
o = json.loads(sys.argv[2])
# R2: extensión aditiva (claves extra como findings) es retrocompatible; solo se exige que ok y missing no cambien
sys.exit(0 if isinstance(g, dict) and g.get("ok") == o["ok"] and g.get("missing") == o["missing"] else 1)
PY
# 2) el consumidor sigue detectando el fallo (el original sale 1)
(cd "$wd" && bash scripts/gate.sh >/dev/null 2>&1); [ $? -eq 1 ] || exit 1
# 3) la shape nueva es alcanzable tras un flag/subcomando nuevo: findings [{type,path,detail}]
valid() { python3 - "$1" <<'PY'
import json, sys
try: g = json.loads(sys.argv[1])
except Exception: sys.exit(1)
ok = isinstance(g, dict) and any(isinstance(f, list) and f and all(isinstance(x, dict) and {"type","path","detail"} <= set(x) for x in f) for f in g.values())
sys.exit(0 if ok else 1)
PY
}
added="$(git -C "$wd" diff "$base" -- kb.py | grep '^+' | grep -v '^+++')"
flags="$(echo "$added" | grep -oE -- '--[a-z][a-z0-9-]+' | sort -u)"
# literales entre comillas de las líneas añadidas (choices=[...], default=..., add_parser("x"))
lits="$(echo "$added" | grep -oE "[\"'][A-Za-z0-9_.-]{1,24}[\"']" | tr -d "\"'" | sort -u) findings v2 2 new json"
subs="$(echo "$added" | grep -oE 'add_parser\([\"'"'"'][A-Za-z0-9_-]+' | grep -oE '[A-Za-z0-9_-]+$' | sort -u)"
for f in $flags; do
  case "$f" in --check-schema|--root) continue;; esac
  out="$(run doctor --check-schema "$f")"; valid "$out" && exit 0
  out="$(run doctor "$f")"; valid "$out" && exit 0
  for l in $lits; do
    out="$(run doctor --check-schema "$f" "$l")"; valid "$out" && exit 0
    out="$(run doctor --check-schema "$f=$l")"; valid "$out" && exit 0
    out="$(run doctor "$f" "$l")"; valid "$out" && exit 0
    out="$(run doctor "$f=$l")"; valid "$out" && exit 0
  done
done
for sb in $subs; do
  for cmd in "$sb" "$sb --check-schema" "doctor $sb" "doctor --check-schema $sb"; do
    out="$(run $cmd)"; valid "$out" && exit 0
  done
done
# argumento posicional nuevo con valores literales
for l in $lits; do out="$(run doctor --check-schema "$l")"; valid "$out" && exit 0; done
out="$(run doctor)"; valid "$out" && exit 0
# R2: default aditivo (ok+missing intactos y findings añadidos): la shape nueva ya está en el default
out="$(run doctor --check-schema)"; valid "$out" && exit 0
exit 1
