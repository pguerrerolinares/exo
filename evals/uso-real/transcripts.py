"""Lectura de transcripts de Claude Code: población de sesiones y mensajes humanos."""
import datetime
import glob
import json
import os

PREFIJOS = ("<command-", "<local-command", "<task-notification>", "<bash-stdout>",
            "Another Claude session", "[Request interrupted", "This session is being continued")
PROMPT_SOURCES = {"typed", "queued"}
MAX_PREVIO = 6000


def texto(d):
    c = (d.get("message") or {}).get("content")
    if isinstance(c, str):
        return c
    if isinstance(c, list):
        return " ".join(b.get("text") or "" for b in c
                        if isinstance(b, dict) and b.get("type") == "text")
    return ""


def es_mensaje_humano(d):
    if d.get("type") != "user":
        return False
    if d.get("isMeta") or d.get("isCompactSummary") or "toolUseResult" in d:
        return False
    if "origin" in d and (d["origin"] or {}).get("kind") != "human":
        return False
    if "promptSource" in d and d["promptSource"] not in PROMPT_SOURCES:
        return False
    t = texto(d).lstrip()
    return bool(t) and not t.startswith(PREFIJOS)


def _lineas(path):
    # El jsonl de una sesión abierta puede acabar en una línea a medio escribir.
    with open(path, encoding="utf-8", errors="replace") as f:
        for linea in f:
            try:
                d = json.loads(linea)
            except ValueError:
                continue
            if isinstance(d, dict):
                yield d


def _reglas(attachment):
    sp = attachment.get("systemPrompt")
    if isinstance(sp, list):
        sp = "\n".join(x for x in sp if isinstance(x, str))
    if not isinstance(sp, str):
        return None
    bloque = []
    dentro = False
    for l in sp.split("\n"):
        if l.startswith("## Reglas duras"):
            dentro = True
        elif dentro and l.startswith("## "):
            break
        if dentro:
            bloque.append(l)
    return "\n".join(bloque).rstrip() if bloque else None


def lee_sesion(path):
    """Devuelve la sesión o None si el fichero no tiene ninguna línea JSON válida."""
    s = {"id": os.path.basename(path)[:-len(".jsonl")] if path.endswith(".jsonl") else os.path.basename(path),
         "repo_dir": os.path.basename(os.path.dirname(path)),
         "cwd": None, "entrypoint": None, "inicio": None, "mensajes": [], "reglas_canal": None}
    previo = ""
    visto = False
    snapshot = False
    for d in _lineas(path):
        visto = True
        s["cwd"] = s["cwd"] or d.get("cwd")
        s["entrypoint"] = s["entrypoint"] or d.get("entrypoint")
        s["inicio"] = s["inicio"] or d.get("timestamp")
        a = d.get("attachment")
        if d.get("type") == "attachment" and isinstance(a, dict) and a.get("type") == "prompt_snapshot" and not snapshot:
            snapshot = True
            s["reglas_canal"] = _reglas(a)
        elif d.get("type") == "assistant":
            t = texto(d)
            if t:
                previo = t
        elif es_mensaje_humano(d):
            s["mensajes"].append({"id": d.get("uuid"), "ts": d.get("timestamp"),
                                  "texto": texto(d), "previo": previo[-MAX_PREVIO:]})
    return s if visto else None


def _epoch(ts):
    d = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    if d.tzinfo is None:
        d = d.replace(tzinfo=datetime.timezone.utc)
    return d.timestamp()


def cortes(sello, desde="2026-10-06"):
    """(desde, hasta) de la población: los del sello si hay ruta de sello, si no (desde, None)."""
    if not sello:
        return desde, None
    with open(sello) as f:
        s = json.load(f)
    return s.get("desde", desde), s.get("hasta")


def poblacion(proyectos, desde="2026-10-06", hasta=None):
    """Sesiones `cli` con inicio >= desde y >= 1 mensaje humano, ordenadas por id.

    `inicio` se compara como cadena ISO: `desde` sin hora ("2026-10-06") queda
    por debajo de "2026-10-06T00:00:00Z", así que vale.

    Con `hasta` la población se cierra: solo sesiones con inicio < hasta y, en cada una, mensajes con ts < hasta
    (sin ts, fuera). Una sesión que se queda sin mensajes cuenta como vacía. Se compara en epoch: los ts pueden
    venir con offset.
    """
    corte = datetime.datetime.fromisoformat(desde).replace(tzinfo=datetime.timezone.utc).timestamp()
    cuentas = {"sdk": 0, "vacias": 0, "reserva": 0}
    aceptadas = []
    # Un solo nivel: `*/<sesion>.jsonl` deja fuera `subagents/`.
    for path in sorted(glob.glob(os.path.join(proyectos, "*", "*.jsonl"))):
        s = lee_sesion(path)
        if s is None or not s["inicio"]:
            continue
        cli = s["entrypoint"] == "cli"
        if s["inicio"] < desde:
            if cli and os.path.getmtime(path) >= corte:
                cuentas["reserva"] += 1
            continue
        if not cli:
            if (s["entrypoint"] or "").startswith("sdk"):
                cuentas["sdk"] += 1
            continue
        if hasta:
            tope = _epoch(hasta)
            if _epoch(s["inicio"]) >= tope:
                continue
            s["mensajes"] = [m for m in s["mensajes"] if m["ts"] and _epoch(m["ts"]) < tope]
        if not s["mensajes"]:
            cuentas["vacias"] += 1
            continue
        aceptadas.append(s)
    aceptadas.sort(key=lambda s: s["id"])
    return aceptadas, cuentas
