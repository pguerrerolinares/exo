#!/usr/bin/env python3
"""Criba de la campana M: orden de corridas y seleccion de supervivientes.

  criba.py orden <a0|ar> <candidatas.txt> <salida>
  criba.py selecciona <a0|ar> <K_ROOT> <candidatas.txt>
  criba.py principal <sondas.txt> <neutras.txt> <salida> [--pool <dir con reglas-g*.jsonl> --kb <prep/kb>]
    escribe junto a <salida> un sondas.tsv (id<TAB>tier_origen) con las sondas elegidas
"""
import json
import random
import re
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


def tiers_origen(pool, kb, ids):
    """tier: del frontmatter de la nota que cita cada regla del pool; `desconocido` (con aviso) si falta."""
    nota = {}
    for f in sorted(Path(pool).glob("reglas-g*.jsonl")) if pool else ():
        for l in f.read_text().splitlines():
            if l.strip():
                r = json.loads(l)
                nota[r["id"]] = r.get("nota")
    out = {}
    for i in ids:
        tier = None
        ruta = Path(kb) / nota[i] if kb and nota.get(i) else None
        if ruta and ruta.is_file():
            fm = re.match(r"---\n(.*?)\n---", ruta.read_text(), re.S)
            m = fm and re.search(r"^tier:\s*(\S+)", fm.group(1), re.M)
            tier = m.group(1) if m else None
        if tier is None:
            print(f"tier desconocido {i} (nota {nota.get(i)})", file=sys.stderr)
        out[i] = tier or "desconocido"
    return out


def principal(sondas, neutras, salida, pool=None, kb=None):
    s = leer(sondas)
    if len(s) > MAX_SONDAS:
        barajadas = list(s)
        random.Random(SEMILLA).shuffle(barajadas)
        s = barajadas[:MAX_SONDAS]
    tiers = tiers_origen(pool, kb, s)
    Path(salida).with_name("sondas.tsv").write_text("".join(f"{t}\t{tiers[t]}\n" for t in s))
    corr = [(t, b, r) for t in s for b in BRAZOS_SONDA for r in (1, 2, 3)]
    corr += [(t, b, r) for t in leer(neutras) for b in BRAZOS_NEUTRA for r in (1, 2)]
    escribir(salida, corr)


if __name__ == "__main__":
    cmd, args = (sys.argv[1], sys.argv[2:]) if len(sys.argv) > 1 else ("", [])
    if cmd in ("orden", "selecciona") and len(args) == 3 and args[0] in ("a0", "ar"):
        {"orden": orden, "selecciona": selecciona}[cmd](*args)
    elif cmd == "principal" and len(args) in (3, 5, 7):
        opc = args[3:]
        kw = {opc[i][2:]: opc[i + 1] for i in range(0, len(opc), 2) if opc[i] in ("--pool", "--kb")}
        if len(kw) * 2 != len(opc):
            sys.exit(__doc__)
        principal(*args[:3], **kw)
    else:
        sys.exit(__doc__)
