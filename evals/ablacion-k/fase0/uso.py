#!/usr/bin/env python3
"""Campaña K, fase 0 (§3 del pre-registro): tasa de uso de lo que inyecta recall-inject.

U      = fracción de eventos `recall-inject-emitted` con >=1 permalink inyectado «tocado».
U_base = lo mismo sustituyendo los inyectados por notas aleatorias del mismo tier.
Lift   = U - U_base, IC95 por bootstrap sobre sesiones.

Tocado (§3), en la misma sesión y con timestamp posterior al evento:
  (a) tool_use Read/Edit sobre el fichero de la nota;
  (b) tool_result de Grep/Bash que contiene su ruta o su permalink;
  (c) texto del asistente que cita su permalink o su título exacto.

Sale en stdout un resumen agregado sin permalinks ni texto de prompts (repo
público). El detalle por evento va a --detalle (fuera del repo).
"""
import argparse, glob, json, os, random, re, sqlite3, sys
from collections import defaultdict

SEMILLA = 20260923
EXO_CMD = re.compile(r"\bexo\b[^|;&\n]*?\b(search|recall|targets)\b")
CONGELACION = "2026-09-28T21:38:55Z"  # commit f324818 (2026-09-28T23:38:55+02:00)


def carga_notas(db, kb):
    c = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    notas = {}
    for perm, ruta, titulo in c.execute("select permalink, ruta, titulo from notas"):
        tier = "sin-tier"
        try:
            with open(os.path.join(kb, ruta), encoding="utf-8") as f:
                cab = f.read(2000)
            m = re.search(r"^tier:\s*(\S+)", cab.split("\n---", 1)[0] if cab.startswith("---") else "", re.M)
            if m:
                tier = m.group(1)
        except OSError:
            tier = "sin-fichero"
        notas[perm] = {"ruta": ruta, "titulo": titulo, "tier": tier}
    return notas


def carga_eventos(log):
    evs = []
    with open(log, encoding="utf-8") as f:
        for linea in f:
            try:
                e = json.loads(linea)
            except ValueError:
                continue
            if e.get("reflex") != "recall-inject-emitted" or e.get("ts", "") >= CONGELACION:
                continue
            m = re.search(r"permalinks=(\S*)", e.get("payload", ""))
            perms = [p for p in (m.group(1).split(",") if m else []) if p]
            if perms:
                evs.append({"ts": e["ts"], "sid": e.get("session_id", ""), "perms": perms})
    evs.sort(key=lambda e: (e["ts"], e["sid"]))
    return evs


def ts_norm(t):
    # transcripts: 2026-09-13T07:56:46.487Z · log: 2026-09-13T07:56:46Z
    return t[:19] + "Z" if t else ""


def trozos_de(ficheros):
    """Por entrada del transcript: (ts, tipo, texto) con tipo en {ruta, result, texto}."""
    out = []
    for fp in ficheros:
        origen = {}  # tool_use_id -> "exo" | "otro"
        with open(fp, encoding="utf-8") as f:
            for linea in f:
                try:
                    e = json.loads(linea)
                except ValueError:
                    continue
                ts = ts_norm(e.get("timestamp", ""))
                msg = e.get("message") or {}
                cont = msg.get("content")
                if e.get("type") == "assistant" and isinstance(cont, list):
                    for b in cont:
                        if b.get("type") == "text":
                            out.append((ts, "texto", b.get("text", "")))
                        elif b.get("type") == "tool_use" and b.get("name") in ("Read", "Edit", "Write", "MultiEdit"):
                            out.append((ts, "ruta", str((b.get("input") or {}).get("file_path", ""))))
                        if b.get("type") == "tool_use":
                            cmd = str((b.get("input") or {}).get("command", ""))
                            origen[b.get("id")] = "exo" if EXO_CMD.search(cmd) else "otro"
                        if b.get("type") == "tool_use" and b.get("name") not in ("Read", "Edit", "Write", "MultiEdit"):
                            out.append((ts, "uso:" + str(b.get("name")), json.dumps(b.get("input"), ensure_ascii=False)))
                elif e.get("type") == "user" and isinstance(cont, list):
                    for b in cont:
                        if b.get("type") == "tool_result":
                            c = b.get("content")
                            txt = c if isinstance(c, str) else json.dumps(c, ensure_ascii=False)
                            out.append((ts, "result:" + origen.get(b.get("tool_use_id"), "otro"), txt))
    return out


def tocados(perms, trozos, ts_ev, notas, kb, id_a_tool):
    """Devuelve {criterio: bool} para el conjunto de permalinks."""
    r = {"a": False, "b_exo": False, "b_otro": False, "c": False, "a_bash": False}
    for p in perms:
        n = notas.get(p)
        ruta_abs = os.path.join(kb, n["ruta"]) if n else None
        # Guarda de validación (fase0/informe.md §2): una ruta sin directorio
        # (README.md, AGENTS.md) o un título de una palabra solo cuentan en
        # forma absoluta / como permalink; si no, casan con cualquier texto.
        ruta_rel = n["ruta"] if n and "/" in n["ruta"] else None
        titulo = n["titulo"] if n and " " in n["titulo"].strip() else None
        def en(txt):
            return p in txt or (ruta_abs and ruta_abs in txt) or (ruta_rel and ruta_rel in txt)
        for ts, tipo, txt in trozos:
            if ts <= ts_ev:
                continue
            if tipo == "ruta" and ruta_abs and txt == ruta_abs:
                r["a"] = True
            elif tipo.startswith("result:") and en(txt):
                r["b_" + tipo.split(":")[1]] = True
            elif tipo == "texto" and (p in txt or (titulo and titulo in txt)):
                r["c"] = True
            elif tipo == "uso:Bash" and en(txt):
                r["a_bash"] = True  # sensibilidad, fuera de la definición de §3
    r["b"] = r["b_exo"] or r["b_otro"]
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", default=os.path.expanduser("~/.claude/reflex-log.jsonl"))
    ap.add_argument("--db", default=os.path.expanduser("~/.exo/index.db"))
    ap.add_argument("--kb", default="/home/paul/Documentos/proyectos/wisdom-paul")
    ap.add_argument("--proyectos", default=os.path.expanduser("~/.claude/projects"))
    ap.add_argument("--detalle", required=True)
    ap.add_argument("--reps", type=int, default=10000)
    a = ap.parse_args()

    notas = carga_notas(a.db, a.kb)
    por_tier = defaultdict(list)
    for p, n in notas.items():
        por_tier[n["tier"]].append(p)
    for t in por_tier:
        por_tier[t].sort()

    evs = carga_eventos(a.log)
    rng = random.Random(SEMILLA)
    cache = {}
    filas, sin_transcript, no_mapeados, total_perms = [], 0, 0, 0
    for ev in evs:
        sid = ev["sid"]
        if sid not in cache:
            principal = glob.glob(os.path.join(a.proyectos, "*", sid + ".jsonl"))
            subs = glob.glob(os.path.join(a.proyectos, "*", sid, "subagents", "*.jsonl"))
            cache[sid] = (trozos_de(principal), trozos_de(subs)) if principal else None
        # el muestreo de la base se hace siempre, para que la secuencia del RNG
        # no dependa de qué transcripts existen
        base = []
        for p in ev["perms"]:
            tier = notas[p]["tier"] if p in notas else None
            pool = [q for q in por_tier.get(tier, []) if q not in ev["perms"] and q not in base] if tier else []
            base.append(rng.choice(pool) if pool else None)
        if cache[sid] is None:
            sin_transcript += 1
            continue
        total_perms += len(ev["perms"])
        no_mapeados += sum(p not in notas for p in ev["perms"])
        principal, subs = cache[sid]
        base = [q for q in base if q]
        fila = {"sid": sid, "ts": ev["ts"], "n": len(ev["perms"]), "n_base": len(base)}
        for nombre, trozos in (("main", principal), ("todo", principal + subs)):
            fila[nombre] = tocados(ev["perms"], trozos, ev["ts"], notas, a.kb, None)
            fila[nombre + "_base"] = tocados(base, trozos, ev["ts"], notas, a.kb, None)
        filas.append(fila)

    with open(a.detalle, "w", encoding="utf-8") as f:
        for fila in filas:
            f.write(json.dumps(fila, ensure_ascii=False) + "\n")

    def u(fs, clave, crits=("a", "b", "c")):
        return sum(any(f[clave][c] for c in crits) for f in fs) / len(fs) if fs else float("nan")

    sesiones = sorted({f["sid"] for f in filas})
    por_ses = defaultdict(list)
    for f in filas:
        por_ses[f["sid"]].append(f)

    def ic_lift(ambito, crits):
        r = random.Random(SEMILLA)
        lifts = []
        for _ in range(a.reps):
            muestra = [x for s in (r.choice(sesiones) for _ in sesiones) for x in por_ses[s]]
            lifts.append(u(muestra, ambito, crits) - u(muestra, ambito + "_base", crits))
        lifts.sort()
        return lifts[int(0.025 * a.reps)], lifts[int(0.975 * a.reps) - 1]

    print(f"eventos antes de la congelación: {len(evs)} · con transcript: {len(filas)} · "
          f"sin transcript: {sin_transcript} · sesiones: {len(sesiones)}")
    print(f"permalinks inyectados: {total_perms} · no mapeados al índice actual: {no_mapeados}")
    print(f"notas por tier en el índice: " + ", ".join(f"{t}={len(v)}" for t, v in sorted(por_tier.items())))
    print()
    print("| ámbito | criterios | U | U_base | lift | IC95 lift (bootstrap por sesión) |")
    print("|---|---|---|---|---|---|")
    for ambito in ("main", "todo"):
        for crits, etiqueta in ((("a", "b", "c"), "a+b+c (§3)"), (("a",), "solo a"), (("b",), "solo b"),
                                (("b_exo",), "b: salida de exo"), (("b_otro",), "b: otra salida"),
                                (("c",), "solo c"), (("a", "b_otro", "c"), "a+b+c sin salidas de exo"),
                                (("a", "b", "c", "a_bash"), "a+b+c+bash (sensib.)")):
            lo, hi = ic_lift(ambito, crits)
            U, B = u(filas, ambito, crits), u(filas, ambito + "_base", crits)
            print(f"| {ambito} | {etiqueta} | {U:.3f} | {B:.3f} | {U - B:+.3f} | [{lo:+.3f}, {hi:+.3f}] |")


if __name__ == "__main__":
    sys.exit(main())
