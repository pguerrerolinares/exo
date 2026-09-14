#!/usr/bin/env python3
"""Pool de queries candidatas del held-out C y muestreo estratificado con
semilla (pre-registro §7). Lee SOLO logs locales de ~/.claude; escribe SOLO en
--out-dir (privado). No inventa queries: si una cuota no cabe, sale con 1.

Uso: pool.py --in-sample $PRIV/in-sample-55.jsonl --out-dir $PRIV \
             --cuota prompt=N1 --cuota agent-search=N2
"""
import argparse
import glob
import json
import os
import random
import shlex
import sys
import unicodedata
from datetime import datetime, timedelta
from pathlib import Path

HOME = Path.home()
RETRIEVAL_LOG = HOME / ".claude" / "reflex-retrieval-log.jsonl"
REFLEX_LOG = HOME / ".claude" / "reflex-log.jsonl"
PROYECTOS = HOME / ".claude" / "projects"
# Ventana por fuente (pre-registro §7, D1): fin común, inicio distinto porque
# `agent-search` arranca al día siguiente del sellado del sweep M2-07
# (ee839ac, 2026-07-18) y `prompt` arranca cuando se fijó D1/D2 de la
# campaña C. Ambos límites son exclusivos.
VENTANA_FIN = "2026-09-13T00:00:00Z"  # sesiones de la campaña C ya vieron las 55
VENTANAS = {
    "prompt": ("2026-08-22T00:00:00Z", VENTANA_FIN),
    "agent-search": ("2026-07-19T00:00:00Z", VENTANA_FIN),
}
MAX_CHARS = 1500
SEMILLA = 20260913
FLAGS_CON_VALOR = {"--db", "--kb", "--limit", "--limite", "--type", "--min-similarity",
                   "--min-similitud", "--bonus", "--fts-scale", "--escala-fts", "--cap-bytes"}
SEPARADORES = {"|", "&&", ";", "||"}


def normaliza(q):
    q = unicodedata.normalize("NFD", q.lower())
    return " ".join("".join(ch for ch in q if unicodedata.category(ch) != "Mn").split())


def jaccard(a, b):
    ta, tb = set(a.split()), set(b.split())
    return len(ta & tb) / len(ta | tb) if ta | tb else 0.0


def query_de_comando(cmd):
    try:
        toks = shlex.split(cmd)
    except ValueError:
        return None
    for i, t in enumerate(toks[:-1]):
        if os.path.basename(t) in ("exo", "kbx") and toks[i + 1] in ("search", "targets"):
            posicionales, j = [], i + 2
            while j < len(toks) and toks[j] not in SEPARADORES:
                if toks[j] in FLAGS_CON_VALOR:
                    j += 2
                    continue
                if not toks[j].startswith("-"):
                    posicionales.append(toks[j])
                j += 1
            return posicionales[-1] if posicionales else None
    return None


def _ts(s):
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def _transcript(sid):
    rutas = glob.glob(str(PROYECTOS / "*" / f"{sid}.jsonl"))
    return Path(rutas[0]) if rutas else None


def pool_search_notes():
    for linea in RETRIEVAL_LOG.read_text(encoding="utf-8").splitlines():
        e = json.loads(linea)
        if e.get("tool") == "mcp__basic-memory__search_notes":
            yield {"query": e.get("target", ""), "source": "agent-search", "session_id": e.get("session_id", ""), "ts": e.get("ts", "")}


def pool_comandos():
    for t in PROYECTOS.glob("*/*.jsonl"):
        with t.open(encoding="utf-8", errors="ignore") as fh:
            for linea in fh:
                if "search" not in linea and "targets" not in linea:
                    continue
                try:
                    e = json.loads(linea)
                except json.JSONDecodeError:
                    continue
                contenido = (e.get("message") or {}).get("content")
                if not isinstance(contenido, list):
                    continue
                for c in contenido:
                    if not isinstance(c, dict) or c.get("type") != "tool_use" or c.get("name") != "Bash":
                        continue
                    cmd = (c.get("input") or {}).get("command", "")
                    if any(x in cmd for x in ("/tmp/", "target/", "evals/", "cargo ")):
                        continue
                    q = query_de_comando(cmd)
                    if q:
                        yield {"query": q, "source": "agent-search", "session_id": e.get("sessionId", ""), "ts": e.get("timestamp", "")}


def mas_cercano(ts_evento, mensajes):
    """El mensaje de `mensajes` (dicts con clave "ts" = datetime) con menor
    |Δt| respecto a `ts_evento`. Empate ⇒ el anterior al evento (ts menor).
    None si `mensajes` está vacío."""
    mejor, mejor_delta = None, None
    for m in mensajes:
        delta = abs((m["ts"] - ts_evento).total_seconds())
        if mejor is None or delta < mejor_delta or (delta == mejor_delta and m["ts"] < mejor["ts"]):
            mejor, mejor_delta = m, delta
    return mejor


def pool_prompts():
    vistos = set()  # (session_id, ts del mensaje): un mismo mensaje no se emite dos veces
    for linea in REFLEX_LOG.read_text(encoding="utf-8").splitlines():
        e = json.loads(linea)
        if e.get("reflex") not in ("recall-inject-emitted", "recall-inject-degraded"):
            continue
        t = _transcript(e.get("session_id", ""))
        if not t or not e.get("ts"):
            continue
        ts_evento = _ts(e["ts"])
        ini, fin = ts_evento - timedelta(seconds=60), ts_evento + timedelta(seconds=2)
        mensajes = []
        with t.open(encoding="utf-8", errors="ignore") as fh:
            for l in fh:
                if '"user"' not in l:
                    continue
                u = json.loads(l)
                c = (u.get("message") or {}).get("content")
                if u.get("type") != "user" or u.get("isSidechain") or not isinstance(c, str) or c.lstrip().startswith("<"):
                    continue
                ts_msg = _ts(u["timestamp"])
                if ini <= ts_msg <= fin:
                    mensajes.append({"query": c, "ts": ts_msg, "ts_str": u["timestamp"]})
        mejor = mas_cercano(ts_evento, mensajes)
        if mejor is None:
            continue
        clave = (e["session_id"], mejor["ts_str"])
        if clave in vistos:
            continue
        vistos.add(clave)
        yield {"query": mejor["query"], "source": "prompt", "session_id": e["session_id"], "ts": mejor["ts_str"]}


def filtra(candidatas, in55):
    desc = {"vacia": 0, "guion": 0, "larga": 0, "fuera-de-ventana": 0, "dup-55": 0, "dup-pool": 0}
    vistos, pool = set(), []
    for c in candidatas:
        q = c["query"].strip()
        n = normaliza(q)
        ini, fin = VENTANAS.get(c["source"], (None, VENTANA_FIN))
        if not n:
            desc["vacia"] += 1
        elif q.startswith("-"):
            desc["guion"] += 1
        elif len(q) > MAX_CHARS:
            desc["larga"] += 1
        elif ini is None or not (_ts(ini) < _ts(c["ts"]) < _ts(fin)):
            desc["fuera-de-ventana"] += 1
        elif any(n == m or jaccard(n, m) >= 0.8 for m in in55):
            desc["dup-55"] += 1
        elif n in vistos:
            desc["dup-pool"] += 1
        else:
            vistos.add(n)
            pool.append({**c, "query": q})
    return pool, desc


class PoolInsuficiente(Exception):
    """Una fuente no tiene candidatas suficientes para cubrir su cuota."""

    def __init__(self, fuente, disponibles, n):
        self.fuente, self.disponibles, self.n = fuente, disponibles, n
        super().__init__(f"pool insuficiente para {fuente}: {disponibles} < {n}")


def muestrea(pool, cuotas):
    """Muestreo estratificado determinista: un `random.Random` por estrato,
    derivado de la semilla y del nombre de la fuente (`SEMILLA:fuente`), para
    que el orden de las cuotas en la CLI no cambie el barajado de ningún
    estrato ni entre versiones de Python (la semilla es un `str`, no depende
    de PYTHONHASHSEED). `cuotas` es una lista de (fuente, n); se conserva el
    estrato completo barajado (nota de diseño de la Task 2: la Task 3 etiqueta
    en ese orden hasta cubrir la cuota)."""
    muestra = []
    for fuente, n in cuotas:
        cands = sorted((c for c in pool if c["source"] == fuente), key=lambda c: (c["ts"], c["query"]))
        rng = random.Random(f"{SEMILLA}:{fuente}")
        rng.shuffle(cands)
        if len(cands) < n:
            raise PoolInsuficiente(fuente, len(cands), n)
        muestra.extend(cands)
    return muestra


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in-sample", required=True)
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--cuota", action="append", required=True, help="source=n (prompt|agent-search)")
    a = ap.parse_args()
    in55 = [normaliza(json.loads(l)["query"]) for l in open(a.in_sample, encoding="utf-8") if l.strip()]
    pool, desc = filtra([*pool_search_notes(), *pool_comandos(), *pool_prompts()], in55)
    cuotas = [(c.split("=")[0], int(c.split("=")[1])) for c in a.cuota]
    try:
        muestra = muestrea(pool, cuotas)
    except PoolInsuficiente as exc:
        print(json.dumps({"pool": {exc.fuente: exc.disponibles}, "descartes": desc}), file=sys.stderr)
        sys.exit(f"pool insuficiente para {exc.fuente}: {exc.disponibles} < {exc.n} — PENDIENTE-PAUL, no se inventan queries")
    out = Path(a.out_dir)
    with open(out / "pool.jsonl", "w", encoding="utf-8") as fh:
        for c in pool:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    with open(out / "muestra.jsonl", "w", encoding="utf-8") as fh:
        for k, c in enumerate(muestra, start=1):
            fh.write(json.dumps({"id": f"c{k:03d}", **c}, ensure_ascii=False) + "\n")
    print(json.dumps({"pool": {s: sum(c["source"] == s for c in pool) for s in ("prompt", "agent-search")},
                      "muestra": len(muestra), "descartes": desc}, ensure_ascii=False))


if __name__ == "__main__":
    main()
