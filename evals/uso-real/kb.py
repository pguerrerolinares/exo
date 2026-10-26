"""La KB tal como estaba en la fecha de una sesión (git de la KB, solo lectura)."""
import json
import os
import subprocess
from datetime import datetime, timezone


def _git(kb, *args):
    r = subprocess.run(["git", "-C", kb, "-c", "core.quotepath=off", *args],
                       capture_output=True, text=True, encoding="utf-8")
    return r.stdout if r.returncode == 0 else None


def _epoch(ts):
    d = datetime.fromisoformat(ts.replace("Z", "+00:00"))
    if d.tzinfo is None:
        d = d.replace(tzinfo=timezone.utc)
    return d.timestamp()


def commit_antes(kb, ts):
    """Último commit con fecha de commit estrictamente anterior a `ts` (ISO, Z u offset)."""
    limite = _epoch(ts)
    salida = _git(kb, "log", "--format=%H %ct", "HEAD")
    mejor = None
    for linea in (salida or "").splitlines():
        h, ct = linea.split()
        if int(ct) < limite and (mejor is None or int(ct) > mejor[1]):
            mejor = (h, int(ct))
    return mejor[0] if mejor else None


def nota_head(cwd, kb):
    binario = os.environ.get("EXO_BIN", "exo")
    try:
        r = subprocess.run([binario, "rules", "--json", "--kb", kb, "--cwd", cwd],
                           capture_output=True, text=True, timeout=60)
        if r.returncode != 0:
            return None
        return json.loads(r.stdout)["data"].get("note") or None
    except (OSError, subprocess.SubprocessError, ValueError, KeyError, TypeError):
        return None


def ruta_en(kb, ruta_head, commit):
    """Nombre de `ruta_head` en `commit`, siguiendo renombres hacia atrás."""
    log = _git(kb, "log", "--follow", "--name-only", "--format=%x01%H", "--", ruta_head)
    ancestros = _git(kb, "rev-list", commit)
    if log is None or ancestros is None:
        return None
    ancestros = set(ancestros.split())
    for bloque in log.split("\x01")[1:]:
        h, *nombres = [x for x in bloque.splitlines() if x]
        if h in ancestros and nombres:
            return nombres[0]
    return None


def _show(kb, commit, ruta):
    return _git(kb, "show", f"{commit}:{ruta}")


def contexto(kb, commit, nota):
    texto = _show(kb, commit, nota) if nota else None
    ls = _git(kb, "ls-tree", "-r", "--name-only", commit, "core/") or ""
    core = {}
    for ruta in ls.splitlines():
        if ruta.endswith(".md") and ruta.count("/") == 1:
            t = _show(kb, commit, ruta)
            if t is not None:
                core[ruta] = t
    return {"nota": texto, "reglas_globales": core.get("core/reglas-duras.md"),
            "core": core, "nota_ausente": texto is None}
