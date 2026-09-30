#!/usr/bin/env python3
"""Campaña K, errata E5 punto 3: valida candidatas de S2 en el orden de la semilla.

Una candidata es válida si los módulos de test que toca su commit C
  - FALLAN en el padre P con los ficheros de test de C encima, y
  - PASAN en C.
Se para al llegar a --objetivo válidas. Usa los clones de <base>/<repo> (se
dejan en su HEAD de origen al terminar) y los venvs <base>/venv-<repo>.

Uso: validar_s2.py <base> <orden-s2.txt> <salida.jsonl> [--objetivo 20]
"""
import json, os, re, subprocess, sys, time

base, orden, salida = sys.argv[1:4]
objetivo = int(sys.argv[sys.argv.index("--objetivo") + 1]) if "--objetivo" in sys.argv else 20
MOD_TEST = re.compile(r"(^|/)(test_[^/]*|tests)\.py$")
ENV_EXTRA = {"django-oscar": {"DATABASE_ENGINE": "django.db.backends.sqlite3", "DATABASE_NAME": ":memory:"}}


def git(repo, *a, check=True):
    return subprocess.run(["git", "-C", repo, *a], capture_output=True, text=True, check=check).stdout.strip()


def corre(repo, nombre, ficheros):
    py = os.path.join(base, f"venv-{nombre}", "bin", "python")
    if nombre == "wagtail":
        labels = [f[:-3].replace("/", ".") for f in ficheros]
        cmd = [py, "runtests.py", *labels, "--parallel", "1"]
    else:
        cmd = [py, "-m", "pytest", "-q", "-p", "no:cacheprovider", *ficheros]
    env = {**os.environ, **ENV_EXTRA.get(nombre, {})}
    t0 = time.time()
    try:
        r = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=900, env=env)
        return r.returncode, round(time.time() - t0), (r.stdout + r.stderr)[-400:]
    except subprocess.TimeoutExpired:
        return "timeout", 900, ""


validas = 0
cabezas = {}
with open(orden) as f, open(salida, "w") as out:
    for linea in f:
        cand = linea.strip()
        if not cand:
            continue
        nombre, corto = cand.split("@")
        repo = os.path.join(base, nombre)
        cabezas.setdefault(nombre, git(repo, "rev-parse", "HEAD"))
        c = git(repo, "rev-parse", corto)
        p = git(repo, "rev-parse", c + "^")
        cambiados = git(repo, "show", "--name-only", "--format=", c).splitlines()
        ficheros = [x for x in cambiados if MOD_TEST.search(x)]
        fila = {"cand": cand, "commit": c, "padre": p, "tests": ficheros}
        if not ficheros:
            fila.update(valida=False, motivo="sin módulo de test")
        else:
            git(repo, "checkout", "-q", "-f", p)
            git(repo, "clean", "-qfd")
            git(repo, "checkout", c, "--", *[x for x in cambiados if "test" in x])
            rc_pre, s_pre, log_pre = corre(repo, nombre, ficheros)
            git(repo, "checkout", "-q", "-f", c)
            git(repo, "clean", "-qfd")
            rc_post, s_post, log_post = corre(repo, nombre, ficheros)
            ok = rc_pre not in (0, "timeout") and rc_post == 0
            fila.update(valida=ok, rc_padre=rc_pre, rc_commit=rc_post, seg=[s_pre, s_post],
                        motivo="" if ok else ("pasa en el padre" if rc_pre == 0 else "falla en el commit" if rc_post != 0 else "timeout"),
                        cola_commit=log_post if rc_post != 0 else "")
        validas += fila["valida"]
        out.write(json.dumps(fila, ensure_ascii=False) + "\n"); out.flush()
        print(f"{cand}: {'VÁLIDA' if fila['valida'] else 'no — ' + fila['motivo']} ({validas}/{objetivo})", flush=True)
        if validas >= objetivo:
            break

for nombre, h in cabezas.items():
    git(os.path.join(base, nombre), "checkout", "-q", "-f", h)
print(f"válidas: {validas}")
