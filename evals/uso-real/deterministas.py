"""Violaciones mecánicas de reglas globales en comandos Bash. Regex de shell, no parser."""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import transcripts

REGLAS = {
    "cd-git": re.compile(r"(?:^|[;&|(]\s*)cd\s[^;&|\n]*(?:&&|;)\s*git(?![\w-])", re.M),
    "git-add-A": re.compile(r"\bgit\s+add\s(?:\S+\s+)*?(?:-A|--all)(?![\w-])"),
    "git-stash": re.compile(r"\bgit\s+stash(?![\w-])(?!\s+(?:list|show)(?![\w-]))"),
    "git-checkout-fichero": re.compile(r"\bgit\s+checkout\s+--(?:\s|$)"),
}


def violaciones(comando):
    return [id_ for id_, rx in REGLAS.items() if rx.search(comando)]


def _antes(ev, tope):
    try:
        return transcripts._epoch(ev["timestamp"]) < tope
    except (KeyError, AttributeError, ValueError):
        return False


def comandos_bash(path, hasta=None):
    """Comandos Bash; con `hasta` (ISO) solo los de eventos con timestamp < hasta (sin timestamp, fuera)."""
    tope = transcripts._epoch(hasta) if hasta else None
    out = []
    with open(path, encoding="utf-8") as f:
        for linea in f:
            try:
                ev = json.loads(linea)
            except ValueError:
                continue
            if ev.get("type") != "assistant" or (tope is not None and not _antes(ev, tope)):
                continue
            contenido = (ev.get("message") or {}).get("content")
            if not isinstance(contenido, list):
                continue
            for b in contenido:
                if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") == "Bash":
                    cmd = (b.get("input") or {}).get("command")
                    if isinstance(cmd, str):
                        out.append(cmd)
    return out


def cuenta(ficheros, hasta=None):
    cmds = [c for p in ficheros for c in comandos_bash(p, hasta)]
    por_regla = {}
    for c in cmds:
        for id_ in violaciones(c):
            por_regla[id_] = por_regla.get(id_, 0) + 1
    n = len(cmds)
    por_100 = {id_: 100 * k / n for id_, k in por_regla.items()} if n else {}
    return {"llamadas_bash": n, "por_regla": por_regla, "por_100": por_100}
