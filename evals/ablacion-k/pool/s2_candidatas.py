#!/usr/bin/env python3
"""Campaña K, errata E5: candidatas de S2 desde django-oscar y wagtail.
Uso: s2_candidatas.py <dir con los clones> > candidatas-s2.json"""
import json, re, subprocess, sys

def g(repo, *a):
    return subprocess.run(["git", "-C", repo, *a], capture_output=True, text=True).stdout

MSG = re.compile(r"fix|bug|add|support|allow|handle|prevent|feat|correct", re.I)
NO = re.compile(r"^(docs?|chore|release|bump|merge|revert)|translation|version", re.I)
TEST = re.compile(r"(^|/)tests?(/|_)|test_[^/]*\.py$|_tests?\.py$")
base = sys.argv[1]
out = []
for repo in ("django-oscar", "wagtail"):
    ruta = f"{base}/{repo}"
    for line in g(ruta, "log", "--no-merges", "--since=2025-09-29", "--format=%H\t%s").splitlines():
        h, s = line.split("\t", 1)
        if NO.search(s) or not MSG.search(s):
            continue
        tests = src = nsrc = otros = 0
        for x in g(ruta, "show", "--numstat", "--format=", h).splitlines():
            a, d, p = x.split("\t", 2)
            if a == "-":
                otros += 1
            elif TEST.search(p):
                tests += 1
            elif p.endswith(".py") and "/migrations/" not in p:
                src += int(a) + int(d); nsrc += 1
            else:
                otros += 1
        if tests >= 1 and 1 <= nsrc <= 3 and src <= 60 and otros == 0:
            out.append({"repo": repo, "commit": h, "asunto": s, "lineas_src": src, "ficheros_src": nsrc})
json.dump(out, sys.stdout, ensure_ascii=False, indent=0)
