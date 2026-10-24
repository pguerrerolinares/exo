#!/usr/bin/env python3
"""Criba de la campana M: orden de corridas y seleccion de supervivientes.

  criba.py orden <a0|ar> <candidatas.txt> <salida>
  criba.py selecciona <a0|ar> <K_ROOT> <candidatas.txt>
  criba.py principal <sondas.txt> <neutras.txt> <salida>
"""
import random
import sys
from pathlib import Path

SEMILLA = 20261024
MAX_SONDAS = 40
BRAZOS_SONDA = ("a0", "ar", "a2", "n")
BRAZOS_NEUTRA = ("a0", "a2", "n")


def leer(path):
    return [l.strip() for l in Path(path).read_text().splitlines() if l.strip()]


def escribir(path, corridas):
    random.Random(SEMILLA).shuffle(corridas)
    Path(path).write_text("".join(f"{t}\t{b}\t{r}\n" for t, b, r in corridas))


def rcs(k_root, tarea, brazo):
    """rc de las 2 replicas, o None si falta alguna."""
    out = []
    for rep in (1, 2):
        f = Path(k_root) / "corridas-criba" / tarea / f"{brazo}-r{rep}" / "check.rc"
        if not f.is_file():
            return None
        out.append(int(f.read_text().strip()))
    return out


def orden(paso, candidatas, salida):
    escribir(salida, [(t, paso, r) for t in leer(candidatas) for r in (1, 2)])


def selecciona(paso, k_root, candidatas):
    for t in leer(candidatas):
        v = rcs(k_root, t, paso)
        if v is None:
            print(f"incompleta {t}", file=sys.stderr)
        elif (paso == "a0" and all(rc != 0 for rc in v)) or (paso == "ar" and any(rc == 0 for rc in v)):
            print(t)


def principal(sondas, neutras, salida):
    s = leer(sondas)
    if len(s) > MAX_SONDAS:
        barajadas = list(s)
        random.Random(SEMILLA).shuffle(barajadas)
        s = barajadas[:MAX_SONDAS]
    corr = [(t, b, r) for t in s for b in BRAZOS_SONDA for r in (1, 2, 3)]
    corr += [(t, b, r) for t in leer(neutras) for b in BRAZOS_NEUTRA for r in (1, 2)]
    escribir(salida, corr)


if __name__ == "__main__":
    cmd, args = (sys.argv[1], sys.argv[2:]) if len(sys.argv) > 1 else ("", [])
    if cmd in ("orden", "selecciona") and len(args) == 3 and args[0] in ("a0", "ar"):
        {"orden": orden, "selecciona": selecciona}[cmd](*args)
    elif cmd == "principal" and len(args) == 3:
        principal(*args)
    else:
        sys.exit(__doc__)
