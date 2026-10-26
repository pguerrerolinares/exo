"""Juez aislado por `claude -p` (spec 2026-10-26 §4.2): llamada, parseo de lotes y cache por id.

La invocación es la de `evals/rubrica-reglas/etiquetar.py:llama`, con el modelo como parámetro."""
import hashlib, json, os, re, subprocess, tempfile


class FalloProceso(Exception):
    pass


def sha256_fichero(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def comprueba_sello(sello, actual):
    """Claves de `actual` que no casan con el sello; `a.b` entra en el dict anidado `a`."""
    def valor(k):
        v = sello
        for parte in k.split("."):
            v = v.get(parte) if isinstance(v, dict) else None
        return v
    return [k for k, v in actual.items() if valor(k) != v]


def claude_bin():
    return os.environ.get("CLAUDE_BIN", "claude")


def claude_version():
    return subprocess.run([claude_bin(), "--version"], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                          timeout=60, check=True).stdout.strip()


def parsea_envelope(stdout):
    """(result, coste). Envelope inválido o con is_error -> FalloProceso."""
    try:
        env = json.loads(stdout)
        coste = float(env.get("total_cost_usd") or 0)
        res, error = env["result"], env.get("is_error")
    except (ValueError, TypeError, AttributeError, KeyError):
        raise FalloProceso("la salida de claude no es el envelope JSON con result")
    if error or not isinstance(res, str):
        raise FalloProceso("el envelope de claude trae is_error o un result que no es texto")
    return res, coste


def llama(entrada, modelo, por_llamada):
    # El prompt va el último: --tools es variádico y se lo tragaría.
    argv = [claude_bin(), "-p", "--model", modelo, "--setting-sources", "", "--strict-mcp-config",
            "--settings", '{"autoMemoryEnabled": false}', "--tools", "", "--no-session-persistence",
            "--output-format", "json", "--max-budget-usd", por_llamada, entrada]
    with tempfile.TemporaryDirectory() as cwd:
        try:
            p = subprocess.run(argv, cwd=cwd, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=600)
        except subprocess.TimeoutExpired:
            raise FalloProceso("claude agotó el timeout")
    if p.returncode != 0:
        raise FalloProceso(f"claude salió con rc {p.returncode}")
    return parsea_envelope(p.stdout)


def parsea_lote(res, ids, valida):
    """{id: objeto validado} solo para los ids pedidos con salida válida; lo demás no aparece."""
    if not isinstance(res, str):
        return {}
    t = res.strip()
    m = re.search(r"```(?:json)?\s*(.*?)```", t, re.S)
    if m:
        t = m.group(1).strip()
    try:
        arr = json.loads(t)
    except ValueError:
        return {}
    if not isinstance(arr, list):
        return {}
    pedidos, out = set(ids), {}
    for o in arr:
        v = valida(o) if isinstance(o, dict) else None
        if v is not None and v["id"] in pedidos:
            out[v["id"]] = v
    return out


class Cache:
    """jsonl append por id; la última escritura gana."""

    def __init__(self, path):
        self.path, self.d = path, {}
        if os.path.exists(path):
            with open(path) as f:
                for l in f:
                    try:
                        o = json.loads(l)
                        self.d[o["id"]] = o
                    except (ValueError, KeyError, TypeError):
                        continue

    def get(self, i):
        return self.d.get(i)

    def put(self, i, obj):
        self.d[i] = obj
        os.makedirs(os.path.dirname(os.path.abspath(self.path)), exist_ok=True)
        with open(self.path, "a") as f:
            f.write(json.dumps(obj, ensure_ascii=False) + "\n")
