#!/usr/bin/env python3
"""fugas.py <dir de corrida> <brazo> — detector de fuga de brazo (errata E2).

Detecta por canal y por ruta, no por texto:
  - hooks que el brazo no cablea (eventos system hook_*);
  - un tool_use que invoca el binario exo en A0/A1;
  - recall-inject-emitted en el log del brazo en A0/A1/A2;
  - un tool_use cuya entrada toca la KB de producción o ~/.exo (todos los brazos),
    o el snapshot de la KB (A0/N);
  - escrituras (Write/Edit o Bash) en el snapshot y en <dir de corrida>/memoria/ (todos los brazos).
Imprime {"fuga": bool, "motivos": [...]} y sale con 1 si hay fuga.
"""
import json, os, re, sys

d, brazo = sys.argv[1], sys.argv[2]
K_ROOT = os.environ.get("K_ROOT", os.path.expanduser("~/.cache/exo-ablacion-k"))
SNAP = os.path.join(K_ROOT, "prep", "kb")
PROD = ["/home/paul/Documentos/proyectos/wisdom-paul", "/home/paul/.exo", "~/.exo"]
EXO = re.compile(r"(^|[\s;&|(`$])(\S*/)?exo\s+(search|recall|targets|index|rebuild|write|config|budget|lint|ratchet|rotate|doctor|stale|init|rules)\b")
STUB = "exo: orden no encontrada"
# Escritura por Bash en el snapshot: redirección, tee, sed -i, cp/mv/install con destino en él, rm/touch/truncate.
SNAPRE = r"(?:\S*prep/kb\S*)"
ESCRIBE_BASH = re.compile("|".join([
    r">>?\s*" + SNAPRE,
    r"\btee\b[^|;&]*" + SNAPRE,
    r"\bsed\b[^|;&]*\s-\w*i\w*\b[^|;&]*" + SNAPRE,
    r"\b(?:cp|mv|install)\b[^|;&]*" + SNAPRE + r"\s*(?:$|[;&|])",
    r"\b(?:rm|touch|truncate)\b[^|;&]*" + SNAPRE,
]))
ESCRIBE_TOOLS = ("Write", "Edit", "MultiEdit", "NotebookEdit")
MEMORIA = os.path.join(os.path.abspath(d), "memoria") + os.sep
HOOKS = {"n": set(), "a0": set(), "arp": set(), "ars": set(), "ar": {"SessionStart"}, "a1": {"SessionStart"}, "a2": {"SessionStart"}, "a3": {"SessionStart", "UserPromptSubmit"}}

motivos, avisos = [], []
eventos = []
with open(os.path.join(d, "transcript.jsonl"), encoding="utf-8") as f:
    for linea in f:
        try:
            eventos.append(json.loads(linea))
        except ValueError:
            pass
# tool_use_id -> texto del tool_result, para saber si un intento de exo lo bloqueó el stub
resultados = {}
for e in eventos:
    if e.get("type") == "user":
        for b in (e.get("message") or {}).get("content") or []:
            if isinstance(b, dict) and b.get("type") == "tool_result":
                c = b.get("content")
                resultados[b.get("tool_use_id")] = c if isinstance(c, str) else json.dumps(c, ensure_ascii=False)
for e in eventos:
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
        if brazo in ("n", "a0", "a1", "ar", "arp", "ars") and b.get("name") == "Bash" and EXO.search((b.get("input") or {}).get("command", "")):
            # Ampliación de E2: un intento que el stub bloquea no pasa información; se avisa, no es fuga.
            if STUB in resultados.get(b.get("id"), ""):
                avisos.append("intento de exo bloqueado por el stub")
            else:
                motivos.append("invoca exo sin bloqueo del stub")
        if any(p in entrada for p in PROD):
            motivos.append("toca la KB de producción o ~/.exo")
        if brazo in ("n", "a0", "ar", "arp", "ars") and SNAP in entrada:
            motivos.append("toca el snapshot de la KB")
        inp = b.get("input") or {}
        if b.get("name") in ESCRIBE_TOOLS:
            ruta = os.path.abspath(inp.get("file_path") or inp.get("notebook_path") or "")
            if ruta.startswith(SNAP + os.sep):
                motivos.append("escribe en el snapshot de la KB")
            if ruta.startswith(MEMORIA):
                motivos.append("escribe en la memoria nativa")
        elif b.get("name") == "Bash" and ESCRIBE_BASH.search(inp.get("command", "")):
            motivos.append("escribe en el snapshot de la KB")

log = os.path.join(d, "reflex.jsonl")
if brazo in ("a0", "a1", "a2", "ar", "arp", "ars") and os.path.exists(log):
    if any('"recall-inject-emitted"' in l for l in open(log, encoding="utf-8")):
        motivos.append("recall-inject emitió")

motivos = sorted(set(motivos))
print(json.dumps({"fuga": bool(motivos), "motivos": motivos, "avisos": sorted(set(avisos))}, ensure_ascii=False))
sys.exit(1 if motivos else 0)
