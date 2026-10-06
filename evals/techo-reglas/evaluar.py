#!/usr/bin/env python3
"""Test de techo de reglas — evaluador del gate (preregistro.md). Uso: evaluar.py <K_ROOT> <tareas.tsv> [resultado.json] [--brazo B] [--base A --margen M] [--esperado S,C]
Por defecto --brazo ar, sin base, --esperado 11,6: el gate del v1. Réplicas por tarea: 4.ª columna opcional de
tareas.tsv `brazo:k,...` (def. ar:2 suelo / ar:1 control). Con --base: PASA <=> cumple(B) >= 6, cumple(B)-cumple(A) >= M
y 0 caídas (solo sobre B); motivo extra `margen X−Y=D < M`.

Solo check.rc == 0 es éxito (rc 1, rc 2, corrida ausente o sin check.rc no lo son).
  suelo:   cumple si alguna de sus 2 réplicas (ar-r1, ar-r2) tiene rc 0.
  control: cae si ninguna de sus réplicas (v1: solo ar-r1; v2: 2/2) tiene rc 0.
  tarea `no` (o ausente) en <K_ROOT>/reconstruccion.tsv: no cumple / cae, sin mirar corridas.
GATE: PASA <=> cumplen >= 6 de 11 y 0 caídas de 6. Si no, `GATE: NO PASA (<motivo>[; <motivo>])`:
motivos `N/11 < 6` y `N/6 caídas`; si fallan ambos, van los dos separados por "; ".
"""
import argparse, csv, json, os, sys

UMBRAL_SUELO, MAX_CAIDAS = 6, 0


def rc_de(d):
    try:
        return open(f"{d}/check.rc").read().strip()
    except OSError:
        return None


def meta_de(d):
    try:
        m = json.load(open(f"{d}/meta.json"))
        return m.get("fin") or m.get("error") or "?", m.get("usd")
    except (OSError, ValueError):
        return "sin meta", None


def replicas(f, brazo):
    """Réplicas de `brazo` para la fila f (id, grupo, regla[, brazos]); () si el brazo no corre en esa tarea."""
    spec = f[3] if len(f) > 3 and f[3] else ("ar:2" if f[1] == "suelo" else "ar:1")
    for par in spec.split(","):
        b, _, k = par.partition(":")
        if b == brazo:
            return tuple(range(1, int(k) + 1))
    return ()


def evalua_brazo(k_root, filas, rec, brazo):
    res = []
    for f in filas:
        id_, grupo = f[0], f[1]
        reps = replicas(f, brazo)
        rec_ok = rec.get(id_) == "si"
        d = lambda r: f"{k_root}/corridas/{id_}/{brazo}-r{r}"
        rcs = {r: rc_de(d(r)) if rec_ok else "no-reconstruible" for r in reps}
        info = {r: meta_de(d(r)) if rec_ok else ("-", None) for r in reps}
        ok = any(v == "0" for v in rcs.values())
        res.append({"id": id_, "grupo": grupo, "rc": {f"r{r}": v for r, v in rcs.items()},
                    "fin": {f"r{r}": v[0] for r, v in info.items()}, "usd": {f"r{r}": v[1] for r, v in info.items()},
                    "ok": ok, "resultado": ("cumple" if ok else "no cumple") if grupo == "suelo" else ("ok" if ok else "caída")})
    return res


def evalua(k_root, tareas_tsv, salida, brazo="ar", base=None, margen=3, esperado=(11, 6)):
    rec = {}
    try:
        for f in csv.reader(open(f"{k_root}/reconstruccion.tsv"), delimiter="\t"):
            if len(f) >= 2:
                rec[f[0]] = f[1]
    except OSError as e:
        sys.exit(f"evaluar: no se puede leer {k_root}/reconstruccion.tsv ({e}); sin él todo sería 'no reconstruible'")
    filas = [f for f in csv.reader(open(tareas_tsv), delimiter="\t") if f]
    for f in filas:
        if f[1] not in ("suelo", "control"):
            sys.exit(f"evaluar: grupo desconocido {f[1]!r} en {tareas_tsv} (solo suelo/control)")
    n_s, n_c = sum(f[1] == "suelo" for f in filas), sum(f[1] == "control" for f in filas)
    if (n_s, n_c) != tuple(esperado):
        sys.exit(f"evaluar: se esperaban {esperado[0]} suelo y {esperado[1]} control; hay {n_s} y {n_c}")
    for f in filas:
        if not replicas(f, brazo):
            sys.exit(f"evaluar: {f[0]} no tiene réplicas del brazo {brazo!r} en {tareas_tsv}")
    res = evalua_brazo(k_root, filas, rec, brazo)
    suelo_ok = sum(t["ok"] for t in res if t["grupo"] == "suelo")
    caidas = [t["id"] for t in res if t["grupo"] == "control" and not t["ok"]]
    motivos = []
    if suelo_ok < UMBRAL_SUELO:
        motivos.append(f"{suelo_ok}/{n_s} < {UMBRAL_SUELO}")
    res_base = None
    if base:
        res_base = evalua_brazo(k_root, [f for f in filas if f[1] == "suelo"], rec, base)
        base_ok = sum(t["ok"] for t in res_base)
        if suelo_ok - base_ok < margen:
            motivos.append(f"margen {suelo_ok}\u2212{base_ok}={suelo_ok - base_ok} < {margen}")
    if len(caidas) > MAX_CAIDAS:
        motivos.append(f"{len(caidas)}/{n_c} caídas")
    gate = "GATE: PASA" if not motivos else f"GATE: NO PASA ({'; '.join(motivos)})"
    out = {"suelo_cumplen": suelo_ok, "suelo_total": n_s, "caidas": caidas, "control_total": n_c,
           "tareas": [{k: v for k, v in t.items() if k != "ok"} for t in res], "gate": gate}
    if res_base is not None:
        out.update(brazo=brazo, base=base, base_cumplen=base_ok, margen=margen,
                   base_tareas=[{k: v for k, v in t.items() if k != "ok"} for t in res_base])
    json.dump(out, open(salida, "w"), ensure_ascii=False, indent=1)
    return res, gate, res_base


def tabla(res):
    for t in res:
        for r in t["rc"]:
            u = t["usd"][r]
            print(f"{t['id']:8} {t['grupo']:8} {r:3} {str(t['rc'][r]):>16} {t['fin'][r]:>18} {'-' if u is None else format(u, '.2f'):>7}  {t['resultado'] if r == 'r1' else ''}")


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("k_root"); ap.add_argument("tareas"); ap.add_argument("salida", nargs="?")
    ap.add_argument("--brazo", default="ar"); ap.add_argument("--base"); ap.add_argument("--margen", type=int, default=3)
    ap.add_argument("--esperado", default="11,6")
    a = ap.parse_args()
    try:
        esperado = tuple(int(x) for x in a.esperado.split(","))
        assert len(esperado) == 2
    except (ValueError, AssertionError):
        sys.exit("evaluar: --esperado debe ser S,C (dos enteros)")
    salida = a.salida or os.path.join(os.path.dirname(os.path.abspath(__file__)), "resultado.json")
    res, gate, res_base = evalua(os.path.expanduser(a.k_root), a.tareas, salida, a.brazo, a.base, a.margen, esperado)
    print(f"{'tarea':8} {'grupo':8} {'rep':3} {'rc':>16} {'fin':>18} {'usd':>7}  resultado")
    tabla(res)
    if res_base is not None:
        print(f"-- base {a.base}")
        tabla(res_base)
    print(gate)
