#!/usr/bin/env python3
"""evaluar-ap.py <K_ROOT> [--dir evals/ap] [--out resultado.json] — adjudicación de Ap (preregistro.md + errata E1).

Reglas (preregistro.md, con los umbrales de `umbral.py --k K`; K sale de evals/ap/k.txt, un único sitio):
  unidad     éxito por corrida de una sonda = fracción de réplicas con check.rc == 0 (exito_por_sonda de analizar.py);
             una sonda «pasa» si cumple en >= PASA de K réplicas.
  réplica    ap1 con >= r(N) sondas que pasan Y >= CANAL_MIN (0,80) de éxito por corrida, contando K·N corridas.
  primario   Δ = delta_bootstrap(ap, ap1) de analizar.py (semilla 20261024); Ap cae si el percentil 97,5 es < 0.
  secundario Ap cae si pasan < k(N) sondas en ap, solo si ap1 pasa >= s(N).
  bandas     sin réplica no se adjudica (aunque el pareado muestre dilución); ap1 en [r, s-1]: solo el pareado;
             N < N_MIN: solo se reporta. Ap cae si cae el primario o el secundario.
Una corrida esperada sin check.rc invalida la tanda (exit 1); un rc != 0 es no cumple; no reconstruible = no cumple.
Descriptivos que no adjudican: cobertura.tsv cruzada por sonda, estrato de listas de proyecto con >= 8 reglas,
bootstrap por proyecto (sensibilidad) y, si existe tipos.tsv (id, tipo I/R/ambigua), Δ por tipo.
"""
import csv, importlib.util, json, os, random, sys

H = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(H, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "evals", "campana-m"))
sys.path.insert(0, os.path.join(ROOT, "evals", "techo-reglas"))
from analizar import delta_bootstrap, exito_por_sonda, SEMILLA, REPS  # noqa: E402
from evaluar import rc_de, replicas  # noqa: E402

ESTRATO_MIN = 8  # reglas en la lista de proyecto
NO_RECONSTRUIBLE = 99


def lee_k():
    return int(open(os.path.join(H, "k.txt")).read())


def umbral(k):
    """umbral.py lee K de sys.argv al importarse: se le pasa el K de k.txt."""
    old = sys.argv
    sys.argv = ["umbral.py", "--k", str(k)]
    try:
        s = importlib.util.spec_from_file_location("umbral_ap", os.path.join(H, "umbral.py"))
        m = importlib.util.module_from_spec(s); s.loader.exec_module(m)
    finally:
        sys.argv = old
    return m


def _boot(d, semilla=SEMILLA, n=REPS):
    """(media, p2,5, p97,5) del bootstrap de las diferencias d (lista)."""
    if not d:
        return None
    rng = random.Random(semilla); k = len(d)
    m = sorted(sum(d[rng.randrange(k)] for _ in range(k)) / k for _ in range(n))
    return sum(d) / k, m[int(0.025 * n)], m[int(0.975 * n) - 1]


def _boot_proyecto(d_por_proyecto, semilla=SEMILLA, n=REPS):
    """Remuestrea proyectos (con reemplazo); la media es sobre las sondas de los proyectos sacados."""
    ps = sorted(d_por_proyecto)
    if not ps:
        return None
    rng = random.Random(semilla); medias = []
    for _ in range(n):
        s = [x for p in (ps[rng.randrange(len(ps))] for _ in ps) for x in d_por_proyecto[p]]
        medias.append(sum(s) / len(s))
    medias.sort()
    todas = [x for p in ps for x in d_por_proyecto[p]]
    return sum(todas) / len(todas), medias[int(0.025 * n)], medias[int(0.975 * n) - 1]


def decide(ap, ap1, U, proyecto=None, cobertura=None, tipos=None):
    """ap, ap1: {id: [rc, ...]} de las N sondas cubiertas (rc 0 = cumple). U: módulo umbral. Devuelve el dict de resultado."""
    ids = sorted(ap1)
    if sorted(ap) != ids:
        raise ValueError("ap y ap1 deben cubrir las mismas sondas")
    N, K = len(ids), U.K
    ea = {i: exito_por_sonda(ap[i]) for i in ids}
    e1 = {i: exito_por_sonda(ap1[i]) for i in ids}
    pasa = lambda e: e * K >= U.PASA - 1e-9
    ap_pasan, ap1_pasan = sum(pasa(ea[i]) for i in ids), sum(pasa(e1[i]) for i in ids)
    canal = sum(e1.values()) / N if N else 0.0
    out = {"N": N, "K": K, "N_MIN": U.N_MIN, "ap_pasan": ap_pasan, "ap1_pasan": ap1_pasan,
           "ap_exito_corrida": sum(ea.values()) / N if N else None, "ap1_exito_corrida": canal,
           "exito": {i: {"ap": ea[i], "ap1": e1[i]} for i in ids}}
    if N == 0:
        out.update(veredicto="solo se reporta: N=0", adjudica=False)
        return out
    d = delta_bootstrap(ea, e1)
    out["delta"] = {"media": d[0], "p2_5": d[1], "p97_5": d[2]}
    primario = d[2] < 0
    out["primario_cae"] = primario
    if N >= U.N_MIN:
        r, k, s = U.umbrales(N)
        replica = ap1_pasan >= r and canal >= U.CANAL_MIN
        sec_aplica = ap1_pasan >= s
        sec_cae = sec_aplica and ap_pasan < k
        out.update(r=r, k=k, s=s, replica=replica, secundario_aplica=sec_aplica, secundario_cae=sec_cae,
                   banda_ap1="borde [r, s-1]: solo adjudica el pareado" if r <= ap1_pasan < s else None)
        if not replica:
            out.update(adjudica=False, dilucion_pareado_sin_replica=primario,
                       veredicto="sin réplica: no se adjudica" + (" (el pareado muestra dilución, tampoco)" if primario else ""))
        else:
            cae = primario or sec_cae
            out.update(adjudica=True, veredicto="Ap CAE" if cae else "Ap PASA")
    else:
        out.update(adjudica=False, veredicto=f"solo se reporta: N={N} < N_MIN={U.N_MIN}")
    # Descriptivos y sensibilidades (no adjudican)
    if proyecto:
        porp = {}
        for i in ids:
            porp.setdefault(proyecto[i], []).append(ea[i] - e1[i])
        b = _boot_proyecto(porp)
        out["sensibilidad_por_proyecto"] = {"proyectos": len(porp), "media": b[0], "p2_5": b[1], "p97_5": b[2],
                                            "nota": "no adjudica"}
    if cobertura:
        out["cobertura"] = {i: cobertura[i] for i in ids if i in cobertura}
        est = [i for i in ids if i in cobertura and int(cobertura[i]["n_proyecto"]) >= ESTRATO_MIN]
        if est:
            de = [ea[i] - e1[i] for i in est]
            out["estrato_>=8_reglas"] = {"n": len(est), "ids": est, "delta_media": sum(de) / len(de),
                                         "ap_pasan": sum(pasa(ea[i]) for i in est), "ap1_pasan": sum(pasa(e1[i]) for i in est)}
        else:
            out["estrato_>=8_reglas"] = {"n": 0}
    if tipos:
        por = {}
        for i in ids:
            por.setdefault(tipos.get(i, "sin tipo"), []).append(i)
        out["por_tipo"] = {t: {"n": len(v), "ap": sum(ea[i] for i in v) / len(v), "ap1": sum(e1[i] for i in v) / len(v),
                               "delta_media": sum(ea[i] - e1[i] for i in v) / len(v),
                               "ap_pasan": sum(pasa(ea[i]) for i in v), "ap1_pasan": sum(pasa(e1[i]) for i in v)}
                           for t, v in sorted(por.items())}
    return out


def tsv(path):
    """Filas de un TSV con cabecera (ignora líneas que empiezan por #) como dicts por nombre de columna."""
    with open(path) as f:
        return list(csv.DictReader((l for l in f if l.strip() and not l.startswith("#")), delimiter="\t"))


def carga(k_root, d):
    """(ap, ap1, proyecto, cobertura, tipos, transversales) leyendo corridas; sys.exit(1) si falta algún check.rc."""
    rec = {f[0]: f[1] for f in csv.reader(open(f"{k_root}/reconstruccion.tsv"), delimiter="\t") if len(f) >= 2}
    tareas = [f for f in csv.reader(open(os.path.join(d, "tareas.tsv")), delimiter="\t") if f]
    cob = {r["id"]: r for r in tsv(os.path.join(d, "cobertura.tsv"))}
    rcs, faltan = {"ap": {}, "ap1": {}}, []
    for f in tareas:
        for brazo in ("ap", "ap1"):
            reps = replicas(f, brazo)
            if not reps:
                continue
            lista = []
            for r in reps:
                v = rc_de(f"{k_root}/corridas/{f[0]}/{brazo}-r{r}") if rec.get(f[0]) == "si" else str(NO_RECONSTRUIBLE)
                if v is None:
                    faltan.append(f"{f[0]}/{brazo}-r{r}")
                else:
                    try:
                        lista.append(int(v))
                    except ValueError:
                        faltan.append(f"{f[0]}/{brazo}-r{r} (check.rc ilegible: {v!r})")
            rcs[brazo][f[0]] = lista
    if faltan:
        sys.exit(f"evaluar-ap: tanda inválida, sin check.rc: {len(faltan)} corridas ({', '.join(faltan[:8])}{'...' if len(faltan) > 8 else ''})")
    cubiertas = sorted(i for i, r in cob.items() if r["cubierta"] == "si")
    if cubiertas != sorted(rcs["ap1"]):
        sys.exit("evaluar-ap: las sondas con ap1 en tareas.tsv no son las cubiertas de cobertura.tsv")
    tp = os.path.join(d, "tipos.tsv")
    tipos = {r["id"]: r["tipo"] for r in tsv(tp)} if os.path.exists(tp) else None
    ap = {i: rcs["ap"][i] for i in cubiertas}
    return ap, rcs["ap1"], {i: cob[i]["proyecto"] for i in cubiertas}, cob, tipos, rcs["ap"]


def tabla(o):
    L = [f"N = {o['N']} (N_MIN {o['N_MIN']}, K = {o['K']})"]
    if "delta" in o:
        dl = o["delta"]
        L.append(f"Δ = ap − ap1 = {dl['media']:+.3f}  IC95 [{dl['p2_5']:+.3f}, {dl['p97_5']:+.3f}]  (cota inferior {dl['p2_5']:+.3f}; primario cae si p97,5 < 0)")
    if "r" in o:
        L.append(f"réplica: ap1 pasan {o['ap1_pasan']} (r={o['r']}), éxito/corrida {o['ap1_exito_corrida']:.3f} (>= 0,80): {'SÍ' if o['replica'] else 'NO'}")
        L.append(f"absoluto: ap pasan {o['ap_pasan']} (k={o['k']}), aplica si ap1 >= s={o['s']}: {'sí' if o['secundario_aplica'] else 'no'}"
                 + (f"; {o['banda_ap1']}" if o.get("banda_ap1") else ""))
    L.append(f"éxito por corrida: ap {o['ap_exito_corrida']:.3f} · ap1 {o['ap1_exito_corrida']:.3f}")
    L.append("sonda   proyecto             ap    ap1   n_proy  bytes  pos  sim_max")
    for i, e in o["exito"].items():
        c = o.get("cobertura", {}).get(i, {})
        L.append(f"{i:8}{c.get('proyecto', ''):20}{e['ap']:.2f}  {e['ap1']:.2f}  {c.get('n_proyecto', ''):>6}  {c.get('bytes', ''):>5}  {c.get('pos', ''):>3}  {c.get('sim_max', '')}")
    if "estrato_>=8_reglas" in o:
        L.append(f"estrato listas de proyecto >= {ESTRATO_MIN} reglas: {o['estrato_>=8_reglas']}")
    if "sensibilidad_por_proyecto" in o:
        s = o["sensibilidad_por_proyecto"]
        L.append(f"bootstrap por proyecto ({s['proyectos']} proyectos, no adjudica): Δ {s['media']:+.3f} [{s['p2_5']:+.3f}, {s['p97_5']:+.3f}]")
    for t, v in o.get("por_tipo", {}).items():
        L.append(f"tipo {t} (no adjudica): n={v['n']} ap {v['ap']:.2f} ap1 {v['ap1']:.2f} Δ {v['delta_media']:+.3f}")
    L.append(f"VEREDICTO: {o['veredicto']}")
    return "\n".join(L)


def main(argv):
    if len(argv) < 2 or argv[1].startswith("-"):
        sys.exit(__doc__)
    k_root = os.path.expanduser(argv[1])
    d = os.path.abspath(argv[argv.index("--dir") + 1]) if "--dir" in argv else H
    out_p = argv[argv.index("--out") + 1] if "--out" in argv else os.path.join(d, "resultado.json")
    ap, ap1, proy, cob, tipos, ap_todas = carga(k_root, d)
    o = decide(ap, ap1, umbral(lee_k()), proy, cob, tipos)
    o["transversales_ap"] = {i: exito_por_sonda(v) for i, v in sorted(ap_todas.items()) if cob[i]["ambito"] == "transversal"}
    json.dump(o, open(out_p, "w"), ensure_ascii=False, indent=1)
    print(tabla(o))


if __name__ == "__main__":
    main(sys.argv)
