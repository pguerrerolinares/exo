#!/usr/bin/env python3
"""Etiquetador sellado de la rúbrica (spec 2026-10-26, §5.2): una llamada `claude -p` aislada por regla.

Salida pública (--salida): solo id, choque, coste y cita_recortada. Justificación y ruta van a --cache (repo público).
Códigos de salida: 0 termina, 1 entrada inválida, 2 el sello no casa, 3 tope de coste alcanzado,
4 fallo de proceso de claude (rc != 0, timeout o salida que no es el envelope JSON)."""
import argparse, glob, hashlib, importlib.util, json, os, re, subprocess, sys, tempfile

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import rubrica

_s = importlib.util.spec_from_file_location("siembra_reglas", os.path.join(H, "..", "..", "scripts", "siembra-reglas.py"))
sr = importlib.util.module_from_spec(_s); _s.loader.exec_module(sr)

PROMPT = os.path.join(H, "etiquetador.prompt.md")
MODELO = "claude-sonnet-5-5"


def sha256_fichero(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def claude_bin():
    return os.environ.get("CLAUDE_BIN", "claude")


def claude_version():
    return subprocess.run([claude_bin(), "--version"], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                          timeout=60, check=True).stdout.strip()


def construye_entrada(regla, nota_recortada, prompt):
    bloque = f"id: {regla['id']}\n{regla['regla']}"
    valores = {"{REGLA}": bloque, "{NOTA}": nota_recortada}
    return re.sub(r"\{REGLA\}|\{NOTA\}", lambda m: valores[m[0]], prompt)


class FalloProceso(Exception):
    pass


def parsea_salida(stdout):
    """(etiqueta | None, coste). Etiqueta inválida -> None; envelope inválido -> FalloProceso."""
    try:
        env = json.loads(stdout)
        coste = float(env.get("total_cost_usd") or 0)
        res, error = env["result"], env.get("is_error")
    except (ValueError, TypeError, AttributeError, KeyError):
        raise FalloProceso("la salida de claude no es el envelope JSON con result")
    if error or not isinstance(res, str):
        raise FalloProceso("el envelope de claude trae is_error o un result que no es texto")
    try:
        return rubrica.valida_etiqueta(json.loads(res.strip())), coste
    except ValueError:
        return None, coste


def llama(entrada, por_llamada):
    # El prompt va el último: --tools es variádico y se lo tragaría.
    argv = [claude_bin(), "-p", "--model", MODELO, "--setting-sources", "", "--strict-mcp-config",
            "--settings", '{"autoMemoryEnabled": false}', "--tools", "", "--no-session-persistence",
            "--output-format", "json", "--max-budget-usd", por_llamada, entrada]
    with tempfile.TemporaryDirectory() as cwd:
        try:
            p = subprocess.run(argv, cwd=cwd, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=600)
        except subprocess.TimeoutExpired:
            raise FalloProceso("claude agotó el timeout")
    if p.returncode != 0:
        raise FalloProceso(f"claude salió con rc {p.returncode}")
    return parsea_salida(p.stdout)


def lee_nota(kb, commit, nota):
    p = subprocess.run(["git", "-C", kb, "show", f"{commit}:projects/{nota}"], capture_output=True, text=True)
    if p.returncode != 0:
        sys.exit(f"etiquetar: no puedo leer projects/{nota} en {commit}: {p.stderr.strip()}")
    return p.stdout


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for a in ("pool", "kb", "commit", "ids", "salida", "cache"):
        ap.add_argument("--" + a, required=True)
    ap.add_argument("--sello")
    ap.add_argument("--tope-usd", type=float, default=5)
    ap.add_argument("--por-llamada-usd", default="0.10")
    a = ap.parse_args()

    if a.sello:
        sello = json.loads(sr.leer(a.sello))
        actual = {"prompt_sha256": sha256_fichero(PROMPT), "claude_version": claude_version(), "kb_commit": a.commit}
        malos = [k for k, v in actual.items() if sello.get(k) != v]
        if malos:
            print(f"etiquetar: el sello no casa en {malos}", file=sys.stderr)
            return 2

    ids = [l.strip() for l in sr.leer(a.ids).splitlines() if l.strip()]
    pool = {}
    for f in sorted(glob.glob(os.path.join(a.pool, "reglas-g*.jsonl"))):
        for l in sr.leer(f).splitlines():
            if l.strip():
                r = json.loads(l); pool[r["id"]] = r
    for i in ids:
        r = pool.get(i)
        if r is None:
            sys.exit(f"etiquetar: {i} no está en el pool")
        if sr.REPO.get(r["proyecto"], (None, None))[0] is None:
            sys.exit(f"etiquetar: {i} es del proyecto {r['proyecto']!r}, sin repo en REPO; no se manda al modelo")

    prompt = sr.leer(PROMPT)
    os.makedirs(a.cache, exist_ok=True)
    parcial, total = a.salida + ".parcial", 0.0
    if os.path.exists(a.salida):
        os.remove(a.salida)
    with open(parcial, "w") as pub, open(os.path.join(a.cache, "etiquetas.jsonl"), "a") as priv:
        for i in ids:
            r = pool[i]
            nota, encontrada = rubrica.recorta_nota(lee_nota(a.kb, a.commit, sr.REPO[r["proyecto"]][1]), r["cita"])
            entrada = construye_entrada(r, nota, prompt)
            et = None
            for _ in range(2):
                try:
                    et, coste = llama(entrada, a.por_llamada_usd)
                except FalloProceso as e:
                    print(f"etiquetar: {e}; queda {parcial}", file=sys.stderr)
                    return 4
                total += coste
                if et is not None and et["id"] != i:
                    et = None
                if et is not None or total >= a.tope_usd:
                    break
            if et is None:
                pub.write(json.dumps({"id": i, "sin_etiqueta": True}) + "\n")
            else:
                et["cita_recortada"] = encontrada
                priv.write(json.dumps(et, ensure_ascii=False) + "\n")
                pub.write(json.dumps({"id": i, "choque": et["choque"], "coste": et["coste"],
                                      "cita_recortada": encontrada}) + "\n")
            pub.flush(); priv.flush()
            if total >= a.tope_usd:
                print(f"etiquetar: tope de {a.tope_usd} USD alcanzado ({total:.2f}); queda {parcial}", file=sys.stderr)
                return 3
    os.replace(parcial, a.salida)
    return 0


if __name__ == "__main__":
    sys.exit(main())
