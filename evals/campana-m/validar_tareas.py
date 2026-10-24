#!/usr/bin/env python3
"""Valida las tareas candidatas de un directorio gold/s1 (contrato: pool/brief-redaccion.md).

Uso: validar_tareas.py <dir gold/s1>  ->  `<id>\tok` o `<id>\tKO\t<motivo>`; sale 1 si alguna es KO.
Workdirs de prueba: `pruebas/<cumple|viola>/` si existe (si no, el repo base) y
transcript `pruebas/<cumple|viola>.jsonl` (si no, vacío).
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

PROHIBIDAS = re.compile(r"\b(KB|memoria|nota|doctrina|exo)\b", re.IGNORECASE)
RCS_VALIDOS = (0, 1, 2)


def _check(d, workdir, transcript, commit):
    r = subprocess.run(["bash", str(d / "check.sh"), str(workdir), str(transcript), commit],
                       capture_output=True, timeout=120)
    return r.returncode


def _base(d, tarea, tmp):
    base = tmp / "base"
    if tarea.get("setup"):
        subprocess.run(["bash", str(d / "setup.sh"), str(base)], check=True, capture_output=True)
    else:
        repo = Path(tarea.get("repo") or "").expanduser()
        if not repo.is_dir():
            return None
        subprocess.run(["git", "clone", "-q", "--no-hardlinks", str(repo), str(base)],
                       check=True, capture_output=True)
        if tarea.get("commit"):
            subprocess.run(["git", "-C", str(base), "checkout", "-q", tarea["commit"]],
                           check=True, capture_output=True)
    return base


def valida(d: Path) -> str | None:
    d = Path(d)
    try:
        t = json.loads((d / "tarea.json").read_text())
    except (OSError, ValueError) as e:
        return f"tarea.json ilegible: {e}"
    if t.get("no_convertible"):
        return None if t.get("motivo") else "falta motivo"
    if not t.get("prompt"):
        return "falta prompt"
    if PROHIBIDAS.search(t["prompt"]):
        return "prompt menciona la memoria"
    check = t.get("check") or {}
    if check.get("tipo") == "rubrica":
        n = len(check.get("items") or [])
        return None if 1 <= n <= 3 else "rubrica: items debe tener de 1 a 3"
    if not t.get("sanity"):
        return "falta sanity"
    if not (d / "check.sh").is_file():
        return "falta check.sh"

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        try:
            base = _base(d, t, tmp)
        except subprocess.CalledProcessError as e:
            return f"no se pudo preparar el repo: {(e.stderr or b'').decode()[:200]}"
        if base is None:
            return "falta repo"
        head = subprocess.run(["git", "-C", str(base), "rev-parse", "HEAD"],
                              capture_output=True, text=True).stdout.strip()
        vacio = tmp / "vacio.jsonl"
        vacio.write_text("")

        rc = _check(d, base, vacio, head)
        if rc not in RCS_VALIDOS:
            return f"vacio rc={rc}"
        for nombre, esperado in (("cumple", 0), ("viola", 1)):
            wd = tmp / nombre
            origen = d / "pruebas" / nombre
            shutil.copytree(origen if origen.is_dir() else base, wd, symlinks=True)
            tr = d / "pruebas" / f"{nombre}.jsonl"
            rc = _check(d, wd, tr if tr.is_file() else vacio, head)
            if rc != esperado:
                return f"sanity {nombre}={rc}"
    return None


def main(argv):
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    ko = nc = 0
    for d in sorted(p for p in Path(argv[1]).iterdir() if (p / "tarea.json").is_file()):
        m = valida(d)
        print(f"{d.name}\tok" if m is None else f"{d.name}\tKO\t{m}")
        ko += m is not None
        try:
            nc += bool(json.loads((d / "tarea.json").read_text()).get("no_convertible"))
        except ValueError:
            pass
    print(f"no_convertibles: {nc}", file=sys.stderr)
    return 1 if ko else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
