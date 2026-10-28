#!/usr/bin/env python3
"""Instrumento del kill 3 de delegar-decision (spec 2026-10-28 §6.3, R27; issue #217).

`lote`: prefiltro léxico sobre la base sellada y sobre la ventana (sesiones desde el primer evento del hook, con su
propio `--hasta`) + 40 no-hits de la ventana, barajado con semilla fija. Escribe a la cache `etiquetar-kill3.jsonl`
(ciego al periodo: sin fecha, repo ni periodo; mismo formato que `muestra.py`) y `kill3-clave.json` (privada: periodo,
sesión y repo de cada ítem). Se niega si la clave ya existe salvo --forzar: re-sortear tras etiquetar invalida el lote.
`resultado`: lee la clave y las etiquetas (si|no) y calcula tasas por sesión, RR y veredicto. Kill si RR >= 0,80.
Toda vía de «faltan datos» es `incompleto`, nunca `no_kill`.
Códigos de salida: 0 kill/no_kill, 1 entrada inválida, 2 el sello de la base no casa, 3 incompleto."""
import argparse, collections, hashlib, json, os, random, re, sys

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import muestra, transcripts
from sellar import sha_sesiones

PREFILTRO = re.compile(r"consultor|subagente|agente|deleg|lanza|despach|executor|ejecutor|fork|paralel")
SEMILLA = 20261028
VENTANA_DESDE = "2026-10-28T21:34:00Z"  # primer evento `delegar-decision` del reflex-log
N_NOHITS = 40
MIN_SESIONES_VENTANA = 30
UMBRAL_RR = 0.80
TRIPWIRE = 2
ETIQUETAS = ("si", "no")
ITEMS = "etiquetar-kill3.jsonl"
CLAVE = "kill3-clave.json"


def prefiltro(texto):
    return PREFILTRO.search(muestra._normaliza(texto)) is not None


def _repo(cwd):
    return hashlib.sha256((cwd or "").encode()).hexdigest()[:8]


def _mensajes(sesiones, periodo):
    return [{**m, "periodo": periodo, "sesion": s["id"], "repo": _repo(s["cwd"])}
            for s in sesiones for m in sorted(s["mensajes"], key=lambda m: m["ts"] or "")]


def construye(base, ventana, semilla=SEMILLA, n_nohits=N_NOHITS):
    """(items ciegos barajados, clave). base y ventana son listas de sesiones de transcripts.poblacion."""
    mb, mv = _mensajes(base, "base"), _mensajes(ventana, "ventana")
    ids = [m["id"] for m in mb + mv]
    if len(set(ids)) != len(ids):
        raise ValueError("ids de mensaje repetidos entre sesiones o periodos")
    hits = [m for m in mb + mv if prefiltro(m["texto"])]
    rng = random.Random(semilla)
    nohits = rng.sample(sorted((m for m in mv if not prefiltro(m["texto"])), key=lambda m: m["id"]),
                        min(n_nohits, sum(not prefiltro(m["texto"]) for m in mv)))
    sel = hits + nohits
    nohit_ids = {m["id"] for m in nohits}
    rng.shuffle(sel)
    clave = {
        "items": {m["id"]: {"periodo": m["periodo"], "sesion": m["sesion"], "repo": m["repo"],
                            "nohit": m["id"] in nohit_ids} for m in sel},
        "sesiones": {p: len(s) for p, s in (("base", base), ("ventana", ventana))},
        "mensajes": {"base": len(mb), "ventana": len(mv)},
        "sesiones_por_repo": {p: dict(collections.Counter(_repo(s["cwd"]) for s in ss))
                              for p, ss in (("base", base), ("ventana", ventana))},
        "n_nohits_pedidos": n_nohits,
        "semilla": semilla,
    }
    return [muestra._item(m) for m in sel], clave


def _tasa(si, n):
    return si / n if n else None


def calcula(clave, etiquetas):
    """Resultado a partir de la clave y {id: 'si'|'no'}. Lanza ValueError si hay etiquetas ajenas o inválidas."""
    items = clave["items"]
    ajenas = sorted(set(etiquetas) - set(items))
    if ajenas:
        raise ValueError(f"{len(ajenas)} etiquetas de ítems que no están en el lote, p. ej. {ajenas[0]}")
    malas = sorted(i for i, e in etiquetas.items() if e not in ETIQUETAS)
    if malas:
        raise ValueError(f"{len(malas)} etiquetas que no son si|no, p. ej. {malas[0]}")
    sin = [i for i in items if i not in etiquetas]
    si = collections.Counter()
    por_repo = collections.defaultdict(collections.Counter)
    for i, e in etiquetas.items():
        it = items[i]
        if e == "si" and not it["nohit"]:
            si[it["periodo"]] += 1
            por_repo[it["repo"]][it["periodo"]] += 1
    nohits_si = sum(e == "si" for i, e in etiquetas.items() if items[i]["nohit"])
    ns, nm = clave["sesiones"], clave["mensajes"]
    tasa = {p: _tasa(si[p], ns[p]) for p in ("base", "ventana")}
    rr = tasa["ventana"] / tasa["base"] if tasa["base"] else None
    n_nohits = sum(it["nohit"] for it in items.values())
    motivos = []
    if sin:
        motivos.append(f"{len(sin)} ítems sin etiquetar")
    if ns["base"] <= 0:
        motivos.append("la base no tiene sesiones")
    if ns["ventana"] < MIN_SESIONES_VENTANA:
        motivos.append(f"la ventana tiene {ns['ventana']} sesiones (< {MIN_SESIONES_VENTANA}): sigue abierta")
    if rr is None:
        motivos.append("RR indefinido (tasa base 0 o sin sesiones)")
    if n_nohits < clave["n_nohits_pedidos"]:
        motivos.append(f"solo {n_nohits} no-hits de {clave['n_nohits_pedidos']}: el tripwire pierde potencia")
    if nohits_si >= TRIPWIRE:
        motivos.append(f"tripwire: {nohits_si} delegaciones entre los no-hits; volver al censo completo de la ventana")
    veredicto = "incompleto" if motivos else ("kill" if rr >= UMBRAL_RR else "no_kill")
    repos = sorted(set(clave["sesiones_por_repo"]["base"]) | set(clave["sesiones_por_repo"]["ventana"]))
    return {
        "veredicto": veredicto,
        "motivos_incompleto": motivos,
        "umbral_rr": UMBRAL_RR,
        "rr": rr,
        "sesiones": ns,
        "si": {p: si[p] for p in ("base", "ventana")},
        "tasa_por_sesion": tasa,
        "por_mensaje_humano": {"mensajes": nm, "tasa": {p: _tasa(si[p], nm[p]) for p in ("base", "ventana")}},
        "por_repo": {r: {"sesiones": {p: clave["sesiones_por_repo"][p].get(r, 0) for p in ("base", "ventana")},
                         "si": {p: por_repo[r][p] for p in ("base", "ventana")}} for r in repos},
        "tripwire": {"nohits": n_nohits, "delegaciones": nohits_si, "umbral": TRIPWIRE, "salta": nohits_si >= TRIPWIRE},
        "n_sin_etiquetar": len(sin),
    }


def _jsonl(path):
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


def _lote(a):
    destino = os.path.join(a.cache, CLAVE)
    if os.path.exists(destino) and not a.forzar:
        print(f"kill3: {destino} ya existe; --forzar para rehacer el lote (invalida las etiquetas)", file=sys.stderr)
        return 1
    with open(a.sello) as f:
        sello = json.load(f)
    if a.ventana_desde < sello["hasta"]:
        print("kill3: la ventana empieza antes del cierre de la base; los periodos se solaparían", file=sys.stderr)
        return 1
    base, _ = transcripts.poblacion(a.proyectos, sello.get("desde", "2026-10-06"), sello["hasta"])
    if sha_sesiones(base) != sello["sesiones_sha256"] or len(base) != sello["n_sesiones"]:
        print("kill3: la población base no casa con el sello", file=sys.stderr)
        return 2
    ventana, _ = transcripts.poblacion(a.proyectos, a.ventana_desde, a.hasta)
    try:
        items, clave = construye(base, ventana, a.semilla)
    except ValueError as e:
        print(f"kill3: {e}", file=sys.stderr)
        return 1
    clave["ventana"] = {"desde": a.ventana_desde, "hasta": a.hasta, "sesiones_sha256": sha_sesiones(ventana)}
    os.makedirs(a.cache, exist_ok=True)
    with open(os.path.join(a.cache, ITEMS), "w") as f:
        for it in items:
            f.write(json.dumps(it, ensure_ascii=False) + "\n")
    with open(destino, "w") as f:
        json.dump(clave, f, indent=2)
        f.write("\n")
    n = collections.Counter(c["periodo"] for c in clave["items"].values() if not c["nohit"])
    print(f"{len(items)} ítems en {os.path.join(a.cache, ITEMS)} (hits base {n['base']}, hits ventana {n['ventana']}, "
          f"no-hits {len(items) - n['base'] - n['ventana']}); sesiones {clave['sesiones']}")
    return 0


def _resultado(a):
    try:
        with open(a.clave) as f:
            clave = json.load(f)
        etiquetas = {e["id"]: e.get("etiqueta") for e in _jsonl(a.etiquetas)}
        # null = sin rellenar: cuenta como pendiente, no como etiqueta inválida.
        r = calcula(clave, {i: e for i, e in etiquetas.items() if e is not None})
    except (OSError, ValueError, KeyError) as e:
        print(f"kill3: {e}", file=sys.stderr)
        return 1
    r["ventana"] = clave.get("ventana")
    with open(a.salida, "w") as f:
        json.dump(r, f, indent=2)
        f.write("\n")
    print(f"{r['veredicto']} (RR {r['rr']}, umbral {UMBRAL_RR})" + "".join(f"\n  - {m}" for m in r["motivos_incompleto"]))
    return 3 if r["veredicto"] == "incompleto" else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("lote")
    p.add_argument("--proyectos", required=True)
    p.add_argument("--sello", required=True, help="sello de la base (sello.json)")
    p.add_argument("--hasta", required=True, help="cierre de la ventana (ISO 8601 con Z); sin default: un 'ahora' movería la población")
    p.add_argument("--ventana-desde", default=VENTANA_DESDE)
    p.add_argument("--cache", default=os.path.expanduser("~/.cache/exo-uso-real"))
    p.add_argument("--semilla", type=int, default=SEMILLA)
    p.add_argument("--forzar", action="store_true")
    r = sub.add_parser("resultado")
    r.add_argument("--clave", required=True)
    r.add_argument("--etiquetas", required=True, help="etiquetar-kill3.jsonl con \"etiqueta\": si|no")
    r.add_argument("--salida", required=True)
    a = ap.parse_args(argv)
    return _lote(a) if a.cmd == "lote" else _resultado(a)


if __name__ == "__main__":
    sys.exit(main())
