#!/usr/bin/env python3
"""Juez de relevancia para el gold J (borrador §3): dos jueces ciegos e
independientes, fable (subagente de la fábrica) y Kimi (Moonshot, otra
familia de modelo). Tres funciones puras y dos con red:
  paquete(fila, rutas)  -> lo que ve un juez: query + candidatos (permalink,
                           título, primeros MAX_CHARS caracteres del cuerpo).
                           Nunca etiquetas, rankings ni el otro juez.
  parsea(resp, cands)   -> valida el JSON del juez contra SCHEMA (expected ∈
                           candidatos ∪ {null}; acceptable ⊆ candidatos, ≤2).
  kimi(paq, key, model) -> POST {BASE}/chat/completions (OpenAI-compatible,
                           response_format json_schema estricto, urllib).
  modelos(env_keys)     -> GET {BASE}/models para fijar el modelo en §10.
  procesa_lote(...)     -> corre kimi() sobre un fichero de paquetes,
                           reanudable por id, con circuit breaker de gasto
                           real (enmienda 2026-09-20, ver más abajo).
La key se lee en runtime de --env-keys (variable kimi_api_key, fichero
gitignored de otro repo) y NUNCA se imprime ni se escribe. Envío de datos a
Moonshot autorizado por Paul el 2026-09-19 (propuesta.md §7).

Circuit breaker de gasto (enmienda 2026-09-20, doctrina «Fallo silencioso —
el instrumento que no grita», ley 2: un tope que nadie compara con el gasto
real no es un freno, es decoración). `procesa_lote` acumula el coste real de
cada llamada a partir de los tokens que devuelve la propia respuesta de la
API (campo `usage.prompt_tokens` / `usage.completion_tokens`; si la API
reportara además tokens cacheados —Moonshot ofrece caché de entrada más
barata, $0,30/M hit, para el contexto de 1.048.576 tokens de kimi-k3— ya
vienen incluidos dentro de `prompt_tokens` por el propio contrato de la API:
se cuentan del lado de la entrada tal cual, sin lógica de precio de caché
aparte) multiplicado por el precio por millón de tokens que el operador pasa
por CLI. Los precios NO tienen default: `--precio-entrada`/`--precio-salida`
son obligatorios en el subcomando `kimi`; si faltan, argparse aborta antes
de la primera llamada. Precios de kimi-k3 verificados contra
platform.kimi.ai/docs/pricing/chat el 2026-09-20: $3,00 / millón de tokens
de entrada, $15,00 / millón de salida (kimi-k2.6: $0,95 / $4,00 — Paul
eligió k3 el 2026-09-19 por calidad de juez, ver plan §Global Constraints).
En cuanto el gasto acumulado supera `--tope-usd` (default 10, el tope
autorizado por el plan), `procesa_lote` para en seco: NO lanza la siguiente
llamada, y el CLI sale con código 3 (distinto de 0) informando el gasto
acumulado y el id en el que se paró; el pipeline es reanudable por id
(vuelve a correrse con el mismo --out y salta los ids ya escritos).

Uso: juez.py paquetes --candidatos C --snap KB --out P.jsonl
     juez.py modelos --env-keys F
     juez.py kimi --paquetes P.jsonl --env-keys F --model M --out R.jsonl
         --precio-entrada 3.00 --precio-salida 15.00 [--tope-usd 10] [--max N]
     juez.py valida --respuestas R.jsonl --candidatos C
"""
import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

BASE = "https://api.moonshot.ai/v1"
MAX_CHARS = 4000
TOPE_USD_DEFAULT = 10.0
SISTEMA = (
    "Eres un juez de relevancia para un buscador de notas personales en castellano. "
    "Recibes UNA consulta y hasta cinco notas candidatas (permalink, título y el inicio del cuerpo). "
    "Decide qué nota querría recuperar un agente de programación que lanzó esa consulta SOLA, sin más contexto: "
    "expected = el permalink de la mejor nota, o null si ninguna responde a la consulta (una consulta operativa "
    "tipo 'mergea y sigue', o un tema que ninguna candidata cubre, es null). acceptable = hasta dos permalinks más "
    "que servirían razonablemente igual (canon frente a bitácora: consulta genérica/estado → canon; consulta "
    "histórica/fechada → bitácora), o lista vacía. Si la consulta pide un hecho concreto y la nota que lo contiene "
    "es una rotación archivada, elige la archivada. No premies que la nota repita literalmente las palabras de la "
    "consulta: premia que la nota RESPONDA. Responde solo con el JSON pedido."
)
SCHEMA = {
    "type": "object",
    "properties": {
        "expected": {"type": ["string", "null"]},
        "acceptable": {"type": "array", "items": {"type": "string"}, "maxItems": 2},
        "razon": {"type": "string"},
    },
    "required": ["expected", "acceptable", "razon"],
    "additionalProperties": False,
}


def rutas_por_permalink(snap):
    out = {}
    raiz = Path(snap)
    for p in raiz.rglob("*.md"):
        if any(x.startswith(".") for x in p.relative_to(raiz).parts):
            continue
        lineas = p.read_text(encoding="utf-8", errors="ignore").splitlines()
        if not lineas or lineas[0].strip() != "---":
            continue
        for l in lineas[1:]:
            if l.strip() == "---":
                break
            if l.startswith("permalink:"):
                out[l.split(":", 1)[1].strip().strip("'\"")] = str(p)
                break
    return out


def nota(permalink, rutas):
    ruta = rutas.get(permalink)
    if ruta is None:
        return {"permalink": permalink, "titulo": "", "cuerpo": "(nota no encontrada en el snapshot)"}
    texto = Path(ruta).read_text(encoding="utf-8", errors="ignore")
    titulo, cuerpo = "", texto
    if texto.startswith("---"):
        partes = texto.split("---", 2)
        if len(partes) == 3:
            for l in partes[1].splitlines():
                if l.startswith("title:"):
                    titulo = l.split(":", 1)[1].strip().strip("'\"")
            cuerpo = partes[2]
    return {"permalink": permalink, "titulo": titulo, "cuerpo": cuerpo.strip()[:MAX_CHARS]}


def paquete(fila, rutas):
    cands = [nota(p, rutas) for p in fila["candidatos"][:5]]
    texto = [f"CONSULTA: {fila['query']}", "", f"CANDIDATAS ({len(cands)}):"]
    for i, c in enumerate(cands, start=1):
        texto += ["", f"[{i}] permalink: {c['permalink']}", f"título: {c['titulo']}", "cuerpo:", c["cuerpo"]]
    if not cands:
        texto.append("(sin candidatas: expected debe ser null)")
    return {"id": fila["id"], "source": fila["source"], "candidatos": [c["permalink"] for c in cands], "texto": "\n".join(texto)}


def parsea(respuesta, candidatos):
    """(dict válido, None) o (None, error)."""
    try:
        d = json.loads(respuesta) if isinstance(respuesta, str) else respuesta
    except json.JSONDecodeError as e:
        return None, f"json: {e}"
    if not isinstance(d, dict) or set(d) != {"expected", "acceptable", "razon"}:
        return None, "claves"
    exp, acc = d["expected"], d["acceptable"]
    if exp is not None and exp not in candidatos:
        return None, f"expected fuera de candidatos: {exp}"
    if not isinstance(acc, list) or len(acc) > 2 or len(set(acc)) != len(acc) or any(a not in candidatos or a == exp for a in acc):
        return None, "acceptable inválido"
    if exp is None and acc:
        return None, "null con acceptable"
    return {"expected": exp, "acceptable": list(acc), "razon": str(d["razon"])[:500]}, None


def _key(env_keys):
    for l in Path(env_keys).read_text(encoding="utf-8").splitlines():
        if l.startswith("kimi_api_key="):
            return l.split("=", 1)[1].strip().strip("'\"")
    raise SystemExit("kimi_api_key no encontrada en --env-keys")


def _http(url, key, cuerpo=None, timeout=120):
    data = None if cuerpo is None else json.dumps(cuerpo).encode("utf-8")
    req = urllib.request.Request(url, data=data, method="POST" if data else "GET",
                                 headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode("utf-8"))


def modelos(env_keys):
    return sorted(m["id"] for m in _http(f"{BASE}/models", _key(env_keys), timeout=30)["data"])


def cuerpo_peticion(paq, model):
    return {
        "model": model,
        "temperature": 0,
        "messages": [{"role": "system", "content": SISTEMA}, {"role": "user", "content": paq["texto"]}],
        "response_format": {"type": "json_schema", "json_schema": {"name": "juicio", "schema": SCHEMA, "strict": True}},
    }


def kimi(paq, key, model, reintentos=3, http=_http):
    ultimo = None
    for i in range(reintentos):
        try:
            r = http(f"{BASE}/chat/completions", key, cuerpo_peticion(paq, model))
            d, err = parsea(r["choices"][0]["message"]["content"], paq["candidatos"])
            if d is not None:
                uso = r.get("usage", {})
                return {"id": paq["id"], **d, "usage": {k: uso.get(k) for k in ("prompt_tokens", "completion_tokens")}}, None
            ultimo = err
        except (urllib.error.URLError, KeyError, TimeoutError, json.JSONDecodeError) as e:
            ultimo = f"{type(e).__name__}: {str(e)[:120]}"
        time.sleep(2 ** i)
    return None, ultimo


def procesa_lote(ruta_paquetes, out, key, model, precio_entrada, precio_salida,
                  tope_usd=TOPE_USD_DEFAULT, maximo=0, http=_http):
    """Corre `kimi()` sobre cada paquete de `ruta_paquetes`, escribiendo en
    `out` (reanudable: salta los ids ya presentes en `out`). Acumula el
    gasto real en USD (tokens de la respuesta × precio por parámetro) y para
    en seco -sin lanzar la siguiente llamada- en cuanto el acumulado supera
    `tope_usd`. Devuelve un resumen; la clave `parada_por_tope` (id en el
    que se paró) solo aparece si el breaker disparó."""
    if Path(out).exists():
        with open(out, encoding="utf-8") as fh0:
            hechos = {json.loads(l)["id"] for l in fh0 if l.strip()}
    else:
        hechos = set()
    errores, n, tok, gasto, parado_en = [], 0, [0, 0], 0.0, None
    with open(out, "a", encoding="utf-8") as fh, open(ruta_paquetes, encoding="utf-8") as fpaq:
        for l in fpaq:
            if not l.strip():
                continue
            paq = json.loads(l)
            if paq["id"] in hechos or (maximo and n >= maximo):
                continue
            d, err = kimi(paq, key, model, http=http)
            n += 1
            if d is None:
                errores.append({"id": paq["id"], "error": err})
                continue
            pt = d["usage"]["prompt_tokens"] or 0
            ct = d["usage"]["completion_tokens"] or 0
            tok[0] += pt
            tok[1] += ct
            gasto += pt / 1_000_000 * precio_entrada + ct / 1_000_000 * precio_salida
            fh.write(json.dumps(d, ensure_ascii=False) + "\n")
            fh.flush()
            print(f"[{n}] {paq['id']} · gasto acumulado: ${gasto:.4f}", file=sys.stderr)
            if gasto > tope_usd:
                parado_en = paq["id"]
                print(f"TOPE SUPERADO: ${gasto:.4f} > ${tope_usd} en id={parado_en}; "
                      "parada en seco, no se lanza la siguiente llamada. Reanudable por id "
                      "(vuelve a correr con el mismo --out).", file=sys.stderr)
                break
    resultado = {"llamadas": n, "ok": n - len(errores), "errores": errores,
                 "prompt_tokens": tok[0], "completion_tokens": tok[1], "model": model,
                 "gasto_usd": round(gasto, 4), "tope_usd": tope_usd}
    if parado_en is not None:
        resultado["parada_por_tope"] = parado_en
    return resultado


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a_p = sub.add_parser("paquetes")
    a_p.add_argument("--candidatos", required=True)
    a_p.add_argument("--snap", required=True)
    a_p.add_argument("--out", required=True)
    a_m = sub.add_parser("modelos")
    a_m.add_argument("--env-keys", required=True)
    a_k = sub.add_parser("kimi")
    a_k.add_argument("--paquetes", required=True)
    a_k.add_argument("--env-keys", required=True)
    a_k.add_argument("--model", required=True)
    a_k.add_argument("--out", required=True)
    a_k.add_argument("--max", type=int, default=0)
    a_k.add_argument("--precio-entrada", type=float, required=True,
                      help="USD por millón de tokens de ENTRADA. Obligatorio, sin default: "
                           "kimi-k3 = 3.00 (verificado platform.kimi.ai/docs/pricing/chat, 2026-09-20).")
    a_k.add_argument("--precio-salida", type=float, required=True,
                      help="USD por millón de tokens de SALIDA. Obligatorio, sin default: "
                           "kimi-k3 = 15.00 (verificado platform.kimi.ai/docs/pricing/chat, 2026-09-20).")
    a_k.add_argument("--tope-usd", type=float, default=TOPE_USD_DEFAULT,
                      help=f"tope de gasto acumulado en USD; al superarlo, para en seco antes de la "
                           f"siguiente llamada (default {TOPE_USD_DEFAULT}, plan §Global Constraints).")
    a_v = sub.add_parser("valida")
    a_v.add_argument("--respuestas", required=True)
    a_v.add_argument("--candidatos", required=True)
    a = ap.parse_args()
    if a.cmd == "paquetes":
        rutas = rutas_por_permalink(a.snap)
        n = 0
        with open(a.out, "w", encoding="utf-8") as fh:
            for l in open(a.candidatos, encoding="utf-8"):
                if l.strip():
                    fh.write(json.dumps(paquete(json.loads(l), rutas), ensure_ascii=False) + "\n")
                    n += 1
        print(json.dumps({"paquetes": n}))
    elif a.cmd == "modelos":
        print(json.dumps(modelos(a.env_keys)))
    elif a.cmd == "kimi":
        key = _key(a.env_keys)
        resultado = procesa_lote(a.paquetes, a.out, key, a.model, a.precio_entrada, a.precio_salida,
                                  tope_usd=a.tope_usd, maximo=a.max)
        print(json.dumps(resultado, ensure_ascii=False))
        sys.exit(3 if "parada_por_tope" in resultado else (1 if resultado["errores"] else 0))
    else:
        cands = {json.loads(l)["id"]: json.loads(l)["candidatos"] for l in open(a.candidatos, encoding="utf-8") if l.strip()}
        filas = [json.loads(l) for l in open(a.respuestas, encoding="utf-8") if l.strip()]
        malas = []
        for r in filas:
            _, err = parsea({k: r.get(k) for k in ("expected", "acceptable", "razon")}, cands.get(r["id"], []))
            if err:
                malas.append(f"{r['id']}: {err}")
        for m in malas:
            print(m, file=sys.stderr)
        print(json.dumps({"respuestas": len(filas), "invalidas": len(malas)}))
        sys.exit(1 if malas else 0)


if __name__ == "__main__":
    main()
