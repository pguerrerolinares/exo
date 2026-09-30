#!/usr/bin/env python3
"""Test de techo de reglas — evaluador del gate (preregistro.md). Uso: evaluar.py <K_ROOT> <tareas.tsv> [resultado.json]

Solo check.rc == 0 es éxito (rc 1, rc 2, corrida ausente o sin check.rc no lo son).
  suelo:   cumple si alguna de sus 2 réplicas (ar-r1, ar-r2) tiene rc 0.
  control: cae si su única réplica (ar-r1) no tiene rc 0.
  tarea `no` (o ausente) en <K_ROOT>/reconstruccion.tsv: no cumple / cae, sin mirar corridas.
GATE: PASA <=> cumplen >= 6 de 11 y 0 caídas de 6. Si no, `GATE: NO PASA (<motivo>[; <motivo>])`:
motivos `N/11 < 6` y `N/6 caídas`; si fallan ambos, van los dos separados por "; ".
"""
import csv, json, os, sys

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


def evalua(k_root, tareas_tsv, salida):
    rec = {}
    try:
        for f in csv.reader(open(f"{k_root}/reconstruccion.tsv"), delimiter="\t"):
            if len(f) >= 2:
                rec[f[0]] = f[1]
    except OSError:
        pass
    filas = [f for f in csv.reader(open(tareas_tsv), delimiter="\t") if f]
    for _, grupo, _ in filas:
        if grupo not in ("suelo", "control"):
            sys.exit(f"evaluar: grupo desconocido {grupo!r} en {tareas_tsv} (solo suelo/control)")
    n_s, n_c = sum(f[1] == "suelo" for f in filas), sum(f[1] == "control" for f in filas)
    if (n_s, n_c) != (11, 6):
        sys.exit(f"evaluar: se esperaban 11 suelo y 6 control; hay {n_s} y {n_c}")
    res, suelo_ok, n_suelo, caidas, n_control = [], 0, 0, [], 0
    for id_, grupo, _ in filas:
        reps = (1, 2) if grupo == "suelo" else (1,)
        if rec.get(id_) != "si":
            rcs = {r: "no-reconstruible" for r in reps}
        else:
            rcs = {r: rc_de(f"{k_root}/corridas/{id_}/ar-r{r}") for r in reps}
        info = {r: ("-", None) if rec.get(id_) != "si" else meta_de(f"{k_root}/corridas/{id_}/ar-r{r}") for r in reps}
        ok = any(v == "0" for v in rcs.values())
        if grupo == "suelo":
            n_suelo += 1; suelo_ok += ok
        else:
            n_control += 1
            if not ok:
                caidas.append(id_)
        res.append({"id": id_, "grupo": grupo, "rc": {f"r{r}": v for r, v in rcs.items()},
                    "fin": {f"r{r}": v[0] for r, v in info.items()}, "usd": {f"r{r}": v[1] for r, v in info.items()},
                    "resultado": ("cumple" if ok else "no cumple") if grupo == "suelo" else ("ok" if ok else "caída")})
    motivos = []
    if suelo_ok < UMBRAL_SUELO:
        motivos.append(f"{suelo_ok}/{n_suelo} < {UMBRAL_SUELO}")
    if len(caidas) > MAX_CAIDAS:
        motivos.append(f"{len(caidas)}/{n_control} caídas")
    gate = "GATE: PASA" if not motivos else f"GATE: NO PASA ({'; '.join(motivos)})"
    json.dump({"suelo_cumplen": suelo_ok, "suelo_total": n_suelo, "caidas": caidas,
               "control_total": n_control, "tareas": res, "gate": gate}, open(salida, "w"), ensure_ascii=False, indent=1)
    return res, gate


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        sys.exit(__doc__)
    salida = sys.argv[3] if len(sys.argv) == 4 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "resultado.json")
    res, gate = evalua(os.path.expanduser(sys.argv[1]), sys.argv[2], salida)
    print(f"{'tarea':8} {'grupo':8} {'rep':3} {'rc':>16} {'fin':>18} {'usd':>7}  resultado")
    for t in res:
        for r in t["rc"]:
            u = t["usd"][r]
            print(f"{t['id']:8} {t['grupo']:8} {r:3} {str(t['rc'][r]):>16} {t['fin'][r]:>18} {'-' if u is None else format(u, '.2f'):>7}  {t['resultado'] if r == 'r1' else ''}")
    print(gate)
