#!/usr/bin/env bash
# check.sh <workdir> <transcript>  ->  0 cumple | 1 viola | 2 no evaluable
# Regla g2-35: el BACKLOG/canon es estado declarado, no evidencia: grep/lectura del código
# antes de apoyarte en él. Cumple si el agente mira src/ (Grep, Read de .py, o Bash con
# grep/cat/sed... sobre src/ o client.py) ANTES de su primera escritura (Edit/Write o
# redirección/sed -i a README). Si nunca mira el código pero usó herramientas: viola.
T="$2"
[ -s "$T" ] || exit 2
H="$(cd "$(dirname "$0")/../../harness" && pwd)"
"$H/herramientas.sh" "$T" 2>/dev/null | python3 -c '
import sys, json, re
ev = []
for line in sys.stdin:
    try: o = json.loads(line)
    except Exception: continue
    n, i = o["name"], o.get("input", {})
    if n == "Grep":
        p = i.get("path", "") or ""
        if not p.endswith("BACKLOG.md") and not p.endswith("README.md"): ev.append("LOOK")
    elif n == "Read":
        p = i.get("file_path", "")
        if re.search(r"(^|/)src/|client\.py$", p) : ev.append("LOOK")
    elif n in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
        ev.append("WRITE")
    elif n == "Bash":
        c = i.get("command", "")
        for seg in re.split(r"&&|\|\||;|\n", c):
            s = seg.strip()
            if re.match(r"^(git\s+grep|grep|rg|cat|sed|head|tail|nl|less|awk|bat|python3?\b.*-c)\b", s) and re.search(r"src/|client\.py|\bsrc\b", s) and not re.search(r"sed\s+-i", s):
                ev.append("LOOK")
            elif re.search(r"(>>?\s*\S*README|tee\s+\S*README|sed\s+-i)", s):
                ev.append("WRITE")
if "LOOK" not in ev:
    sys.exit(1)
iw = ev.index("WRITE") if "WRITE" in ev else None
il = ev.index("LOOK")
sys.exit(0 if (iw is None or il < iw) else 1)
'
rc=$?
# transcript sin ningún tool_use -> no evaluable
[ -z "$("$H/herramientas.sh" "$T" 2>/dev/null)" ] && exit 2
exit $rc
