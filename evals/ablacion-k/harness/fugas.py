#!/usr/bin/env python3
"""fugas.py <dir de corrida> <brazo> — detector de fuga de brazo (errata E2).

Detecta por canal y por ruta, no por texto:
  - hooks que el brazo no cablea (eventos system hook_*);
  - un tool_use que invoca el binario exo en A0/A1;
  - recall-inject-emitted en el log del brazo en A0/A1/A2;
  - un tool_use cuya entrada toca la KB de producción o ~/.exo (todos los brazos),
    o el snapshot de la KB (A0).
Imprime {"fuga": bool, "motivos": [...]} y sale con 1 si hay fuga.
"""
import json, os, re, sys

d, brazo = sys.argv[1], sys.argv[2]
K_ROOT = os.environ.get("K_ROOT", os.path.expanduser("~/.cache/exo-ablacion-k"))
SNAP = os.path.join(K_ROOT, "prep", "kb")
PROD = ["/home/paul/Documentos/proyectos/wisdom-paul", "/home/paul/.exo", "~/.exo"]
EXO = re.compile(r"(^|[\s;&|(`$])(\S*/)?exo\s+(search|recall|targets|index|rebuild|write|config|budget|lint|ratchet|rotate|doctor|stale|init)\b")
HOOKS = {"a0": set(), "a1": {"SessionStart"}, "a2": {"SessionStart"}, "a3": {"SessionStart", "UserPromptSubmit"}}

motivos = []
with open(os.path.join(d, "transcript.jsonl"), encoding="utf-8") as f:
    for linea in f:
        try:
            e = json.loads(linea)
        except ValueError:
            continue
        if e.get("type") == "system" and str(e.get("subtype", "")).startswith("hook"):
            ev = e.get("hook_event") or e.get("hook_event_name") or str(e.get("hook_name", "")).split(":")[0]
            if ev and ev not in HOOKS[brazo]:
                motivos.append(f"hook no cableado: {ev}")
        if e.get("type") != "assistant":
            continue
        for b in (e.get("message") or {}).get("content") or []:
            if b.get("type") != "tool_use":
                continue
            entrada = json.dumps(b.get("input"), ensure_ascii=False)
            if brazo in ("a0", "a1") and b.get("name") == "Bash" and EXO.search((b.get("input") or {}).get("command", "")):
                motivos.append("invoca exo")
            if any(p in entrada for p in PROD):
                motivos.append("toca la KB de producción o ~/.exo")
            if brazo == "a0" and SNAP in entrada:
                motivos.append("toca el snapshot de la KB")

log = os.path.join(d, "reflex.jsonl")
if brazo in ("a0", "a1", "a2") and os.path.exists(log):
    if any('"recall-inject-emitted"' in l for l in open(log, encoding="utf-8")):
        motivos.append("recall-inject emitió")

motivos = sorted(set(motivos))
print(json.dumps({"fuga": bool(motivos), "motivos": motivos}, ensure_ascii=False))
sys.exit(1 if motivos else 0)
