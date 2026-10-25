#!/usr/bin/env python3
"""evaluar3.py <K_ROOT> <tareas.tsv> [resultado.json] — adjudicación de techo-reglas-3 (preregistro.md).

Cuenta por brazo (arp1, arp10, arp20) las tareas del suelo que cumplen (check.rc == 0 en >=1 de sus 2
réplicas; ausente, sin check.rc o no reconstruible = no cumple), con evalua_brazo de techo-reglas/evaluar.py.
  Réplica:  arp1 >= 5/10; si no, el canal no replica y NO se adjudica CAP (se queda el desplegado, 10).
  CAP:      arp20 >= 5 -> 10 · arp20 == 4 -> 7 · arp20 <= 3 -> 5.
  §7.3:     arp10 <= 3 -> se cumple la condición «canal + dilución» que invalida A (se para y se reporta).
Una réplica ausente en cualquier brazo invalida la tanda (exit 1), como el breaker.
"""
import csv, json, os, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "techo-reglas"))
from evaluar import evalua_brazo  # noqa: E402

BRAZOS = ("arp1", "arp10", "arp20")
REPLICA_MIN = 5


def cap_de(n20):
    return 10 if n20 >= 5 else 7 if n20 == 4 else 5


def adjudica(k_root, tareas_tsv):
    rec = {f[0]: f[1] for f in csv.reader(open(f"{k_root}/reconstruccion.tsv"), delimiter="\t") if len(f) >= 2}
    filas = [f for f in csv.reader(open(tareas_tsv), delimiter="\t") if f]
    if len(filas) != 10 or any(f[1] != "suelo" for f in filas):
        sys.exit("evaluar3: se esperaban exactamente 10 tareas del suelo")
    res = {b: evalua_brazo(k_root, filas, rec, b) for b in BRAZOS}
    for b, ts in res.items():
        for t in ts:
            if any(v is None for v in t["rc"].values()):
                sys.exit(f"evaluar3: {b} incompleto: {t['id']} sin check.rc — tanda inválida")
    n = {b: sum(t["ok"] for t in ts) for b, ts in res.items()}
    replica = n["arp1"] >= REPLICA_MIN
    out = {"cumplen": n, "replica": replica,
           "cap": cap_de(n["arp20"]) if replica else None,
           "invalida_A_7_3": n["arp10"] <= 3,
           "tareas": {b: [{k: v for k, v in t.items() if k != "ok"} for t in ts] for b, ts in res.items()}}
    lineas = [f"{b}: {n[b]}/10" for b in BRAZOS]
    lineas.append(f"CAP: {out['cap']}" if replica else f"CAP: sin adjudicar (arp1 {n['arp1']}/10 < {REPLICA_MIN}: el canal no replica)")
    lineas.append("§7.3: SE CUMPLE (arp10 <= 3/10): A queda invalidado" if out["invalida_A_7_3"] else "§7.3: no se cumple")
    return out, lineas


if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    out, lineas = adjudica(os.path.expanduser(sys.argv[1]), sys.argv[2])
    salida = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "resultado.json")
    json.dump(out, open(salida, "w"), ensure_ascii=False, indent=1)
    print("\n".join(lineas))
