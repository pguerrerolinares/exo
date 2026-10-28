#!/usr/bin/env python3
"""Detector del gate delegar-decision: cierre del padre con propuesta + permiso sin delegar.

Reimplementa `es_mensaje_humano` y `_epoch` de evals/uso-real/transcripts.py sin importarlo: el
`filtro_sha256` del sello cubre el original y este módulo corre en el hook, sin evals/ a mano.
"""
import datetime
import json
import re
import sys

PERM = re.compile(r"(?i)¿\s*(te parece|te vale|te cuadra|lo lanzo|los lanzo|lanzo|lo hago|los hago|hago|procedo|sigo|arranco|empiezo|aplico|abro|quieres que|lo copio|los copio|arreglo|preparo|escribo|implemento|lo implemento|lo aplico|vamos|seguimos|arrancamos|lo dejo|commiteo|lo commiteo|lo subo|le doy|doy)\b")
PROP = re.compile(r"(?i)\b(mi recomendaci[oó]n|te propongo|lo que propongo|propongo|recomiendo|yo har[ií]a|har[ií]a)\b")
EXCLUSION = re.compile(r"(?i)(?<!\w)(?:push|prs?|issues?|merge|deploy|commit|/document|borrar)\b")
PREGUNTA = re.compile(r"¿[^?]*\?")
ETIQUETA = re.compile(r"\[reserva:\s*(?:externo|irreversible|re-scope|hecho-de-Paul|pedido)\s*\]")

PREFIJOS = ("<command-", "<local-command", "<task-notification>", "<bash-stdout>",
            "Another Claude session", "[Request interrupted", "This session is being continued")
PROMPT_SOURCES = {"typed", "queued"}
TAIL = 700


def _texto(d):
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
    t = _texto(d).lstrip()
    return bool(t) and not t.startswith(PREFIJOS)


def _epoch(ts):
    d = datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    if d.tzinfo is None:
        d = d.replace(tzinfo=datetime.timezone.utc)
    return d.timestamp()


def _solo_tool_result(d):
    c = (d.get("message") or {}).get("content")
    return isinstance(c, list) and c and all(isinstance(x, dict) and x.get("type") == "tool_result" for x in c)


def segmentar(lineas, tope=None):
    """Segmentos de assistant (sin sidechain) cerrados por cada input; el último queda abierto (clave None).

    Clave = uuid del mensaje humano que cierra el segmento, "NOHUMANO" si lo cierra un input no humano
    que abre turno (hand-back de subagente, task-notification).
    """
    cur, out = [], []
    for d in lineas:
        if d.get("isSidechain"):
            continue
        ts = d.get("timestamp")
        if tope is not None and ts and _epoch(ts) >= tope:
            break
        if d.get("type") == "assistant":
            c = (d.get("message") or {}).get("content")
            if isinstance(c, list):
                cur.extend(b for b in c if isinstance(b, dict))
        elif es_mensaje_humano(d):
            out.append((d.get("uuid"), cur))
            cur = []
        elif d.get("type") == "user" and "toolUseResult" not in d and not _solo_tool_result(d) and (
                not d.get("isMeta") or d.get("promptSource") == "system" or (d.get("origin") or {}).get("kind")):
            out.append(("NOHUMANO", cur))
            cur = []
    out.append((None, cur))
    return out


def _ambito_exclusion(tail):
    preguntas = PREGUNTA.findall(tail)
    if preguntas:
        return preguntas[-1]
    lineas = tail.rstrip().splitlines()
    return lineas[-1] if lineas else ""


def evalua(bloques):
    tools = [b for b in bloques if b.get("type") == "tool_use"]
    textos = [b.get("text") or "" for b in bloques if b.get("type") == "text"]
    last = textos[-1] if textos else ""
    tail = last[-TAIL:]
    r = {"dispara": False, "kind": None, "patron": None, "excluido": None, "tail": tail}
    if any(b.get("name") in ("Agent", "Task") for b in tools):
        r["excluido"] = "agent"
        return r
    ask = any(b.get("name") == "AskUserQuestion" for b in tools)
    if PERM.search(tail):
        r["patron"] = "P2"
    elif PROP.search(last) and ("?" in tail or ask):
        r["patron"] = "P3"
    if r["patron"]:
        if ETIQUETA.match(last.lstrip()):
            r["excluido"] = "etiqueta"
        elif EXCLUSION.search(_ambito_exclusion(tail)):
            r["excluido"] = "reserva"
        else:
            r["dispara"], r["kind"] = True, "disparo"
    elif "?" in tail or ask:
        r["kind"] = "p1_log"
    elif textos and not tools:
        r["kind"] = "banda"
    return r


def _cli(ruta):
    with open(ruta, encoding="utf-8", errors="replace") as f:
        lineas = []
        for ln in f:
            try:
                d = json.loads(ln)
            except ValueError:
                continue
            if isinstance(d, dict):
                lineas.append(d)
    try:
        segs = segmentar(lineas)
        uuid = next((k for k, _ in reversed(segs) if k not in (None, "NOHUMANO")), None)
        r = evalua(segs[-1][1])
        # R15: el informativo de R12 (>10 % con término de reserva fuera de la pregunta) se mide con esto.
        fuera = bool(r["dispara"] and EXCLUSION.search(r["tail"]))
        return {"uuid": uuid, "dispara": r["dispara"], "kind": r["kind"], "patron": r["patron"],
                "tail": r["tail"], "reserva_fuera": fuera}
    except Exception:  # R16: la excepción del detector es abstención (kind=error), nunca un fallo del hook
        return {"uuid": None, "dispara": False, "kind": "error", "patron": None, "tail": "",
                "reserva_fuera": False}


if __name__ == "__main__":
    try:
        res = _cli(sys.argv[1])
    except (OSError, IndexError):
        sys.exit(2)
    print(json.dumps(res, ensure_ascii=False))
