#!/usr/bin/env python3
"""Convierte un snapshot de la KB en la auto memory de Claude Code (brazo N).

Uso: gen_memoria.py <kb> <salida>
Escribe <salida>/MEMORY.md (solo tier core y stable) y copia todas las notas
.md como topic files. Sale 1 si el índice pasa los límites de carga de la
plataforma (200 líneas / 25 KB), en vez de dejar que se trunque en silencio.
"""
import re
import shutil
import sys
from pathlib import Path

MAX_LINEAS = 200
MAX_BYTES = 25000
TIERS_INDICE = {"core", "stable"}


def parsear(texto):
    """Devuelve (frontmatter: dict, cuerpo: str). Frontmatter clave: valor plano."""
    fm = {}
    if texto.startswith("---\n"):
        fin = texto.find("\n---", 4)
        if fin != -1:
            for l in texto[4:fin].splitlines():
                k, sep, v = l.partition(":")
                if sep:
                    fm[k.strip()] = v.strip().strip("\"'")
            resto = texto[fin + 4:]
            return fm, resto.split("\n", 1)[1] if "\n" in resto else ""
    return fm, texto


def primera_frase(cuerpo):
    _, cuerpo = parsear(cuerpo)
    parrafo = []
    for l in cuerpo.splitlines():
        s = l.strip()
        if not s or s.startswith("#"):
            if parrafo:
                break
            continue
        parrafo.append(s)
    txt = " ".join(parrafo)
    m = re.match(r"(.+?[.!?])(\s|$)", txt)
    return m.group(1) if m else txt


def linea_indice(titulo, ruta, cuerpo, description):
    destino = f"<{ruta}>" if " " in ruta else ruta
    resumen = description or primera_frase(cuerpo)
    base = f"- [{titulo}]({destino})"
    return f"{base} — {resumen}" if resumen else base


def notas(kb):
    return sorted(p for p in kb.rglob("*.md") if ".git" not in p.relative_to(kb).parts)


def main(kb, salida):
    kb, salida = Path(kb), Path(salida)
    todas = notas(kb)
    lineas = []
    for p in todas:
        rel = p.relative_to(kb).as_posix()
        texto = p.read_text(encoding="utf-8")
        fm, cuerpo = parsear(texto)
        if fm.get("tier") in TIERS_INDICE:
            lineas.append(linea_indice(fm.get("title") or p.stem, rel, cuerpo, fm.get("description")))
    indice = "".join(l + "\n" for l in lineas)
    n_bytes = len(indice.encode("utf-8"))
    if len(lineas) > MAX_LINEAS or n_bytes > MAX_BYTES:
        print(f"lineas={len(lineas)} bytes={n_bytes} notas={len(todas)} FUERA DE LIMITE "
              f"(max {MAX_LINEAS} lineas, {MAX_BYTES} bytes)", file=sys.stderr)
        return 1
    salida.mkdir(parents=True, exist_ok=True)
    for p in todas:
        dst = salida / p.relative_to(kb)
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, dst)
    (salida / "MEMORY.md").write_text(indice, encoding="utf-8")
    print(f"lineas={len(lineas)} bytes={n_bytes} notas={len(todas)}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("uso: gen_memoria.py <kb> <salida>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(*sys.argv[1:]))
