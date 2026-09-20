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
  costo(usage, pe, ps)  -> USD de una respuesta, o error explícito si el
                           usage no es utilizable (nunca coste cero mudo).
  reconstruye_gasto(...) -> gasto/tokens heredados de un --out ya escrito,
                           para que una reanudación no "olvide" lo gastado.
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
En cuanto el gasto acumulado SUPERA (estrictamente mayor que, nunca ≥)
`--tope-usd` (default 10, el tope autorizado por el plan), `procesa_lote`
para en seco: NO lanza la siguiente llamada, y el CLI sale con código 3
(distinto de 0) informando el gasto acumulado y el id en el que se paró; el
pipeline es reanudable por id (vuelve a correrse con el mismo --out y salta
los ids ya escritos). Un gasto que iguala el tope exactamente NO dispara el
breaker (se gasta hasta el tope, inclusive); superarlo sí.

Enmienda 2026-09-20 (b), review adversarial — el breaker era ciego en tres
caminos por los que un job real lo recorre:
- **Reanudación (C1):** `--out` solo persiste las filas con juicio válido;
  al reanudar, el gasto acumulado se RECONSTRUYE sumando el `usage` de esas
  filas (nunca arranca en 0 si ya hay filas escritas) — ver
  `reconstruye_gasto()`. Se imprime el gasto heredado al arrancar.
- **Reintentos que fallan `parsea()` (C2):** el `json_schema strict` de la
  API garantiza la FORMA del JSON, no que `expected`/`acceptable` sean
  permalinks reales de `candidatos` — eso lo valida `parsea()` después. Una
  respuesta que pasa el schema pero alucina un permalink sigue siendo una
  llamada FACTURADA: su coste se contabiliza vía el callback `contabiliza`
  de `kimi()`, se haya validado o no el JSON después. El tope se comprueba
  también ANTES de cada reintento (callback `tope_alcanzado`): si ya se
  rebasó, no se lanza el siguiente intento.
- **`usage` ausente o con campos no numéricos (C3):** nunca cuenta como
  gasto $0 en silencio ni se estima — es un error explícito que para el
  pipeline en seco (`parada_por_usage_invalido` en el resultado, código de
  salida 4). Un breaker que no sabe lo que gastó no puede seguir gastando.

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


def _numerico(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool)


def costo(usage, precio_entrada, precio_salida):
    """(usd, None) si `usage` es utilizable, o (None, error) si no. Un
    `usage` ausente, incompleto o con campos no numéricos es SIEMPRE un
    error explícito (C3) — nunca se estima ni se trata como coste cero."""
    if not isinstance(usage, dict):
        return None, f"usage ausente o no es un objeto: {usage!r}"
    pt, ct = usage.get("prompt_tokens"), usage.get("completion_tokens")
    if not _numerico(pt) or not _numerico(ct):
        return None, f"usage incompleto o no numérico: prompt_tokens={pt!r} completion_tokens={ct!r}"
    return pt / 1_000_000 * precio_entrada + ct / 1_000_000 * precio_salida, None


def reconstruye_gasto(ruta_out, precio_entrada, precio_salida):
    """Reanudación (C1): recorre las filas ya escritas en `ruta_out` y
    devuelve (ids_hechos, gasto_heredado, [prompt_tok, completion_tok]). Si
    alguna fila no tiene un `usage` utilizable, aborta con SystemExit (C3):
    un breaker que no puede reconstruir lo que ya gastó no puede seguir
    gastando; no hay ficción de coste cero para filas viejas."""
    ids, gasto, tok = set(), 0.0, [0, 0]
    if not Path(ruta_out).exists():
        return ids, gasto, tok
    with open(ruta_out, encoding="utf-8") as fh:
        for n, l in enumerate(fh, start=1):
            if not l.strip():
                continue
            fila = json.loads(l)
            ids.add(fila["id"])
            c, err = costo(fila.get("usage"), precio_entrada, precio_salida)
            if err is not None:
                raise SystemExit(
                    f"gasto no reconstruible al reanudar: fila {n} (id={fila.get('id')}) de "
                    f"{ruta_out} tiene usage inutilizable ({err}). El breaker no puede heredar "
                    "un gasto que no sabe calcular; corrige o descarta esa fila antes de "
                    "reanudar (no se estima ni se asume cero)."
                )
            gasto += c
            tok[0] += fila["usage"]["prompt_tokens"]
            tok[1] += fila["usage"]["completion_tokens"]
    return ids, gasto, tok


def kimi(paq, key, model, reintentos=3, http=_http, contabiliza=None, tope_alcanzado=None):
    """Intenta hasta `reintentos` veces. Cada intento que SÍ recibe
    respuesta de la API es una llamada FACTURADA y se pasa a
    `contabiliza(usage)` -- se valide o no después con `parsea()` (C2): un
    permalink alucinado que pasa el json_schema pero falla la validación de
    negocio se pagó igual. Si `contabiliza` devuelve error (usage
    inutilizable, C3), `kimi()` para en seco de inmediato. Antes de cada
    intento -incluidos los reintentos- se consulta `tope_alcanzado()`; si ya
    se rebasó, NO se lanza el intento (C2)."""
    ultimo = None
    for i in range(reintentos):
        if tope_alcanzado is not None and tope_alcanzado():
            return None, "tope superado: reintento no lanzado"
        try:
            r = http(f"{BASE}/chat/completions", key, cuerpo_peticion(paq, model))
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
            ultimo = f"{type(e).__name__}: {str(e)[:120]}"
            time.sleep(2 ** i)
            continue
        uso = r.get("usage")
        if contabiliza is not None:
            _, err_costo = contabiliza(uso)
            if err_costo is not None:
                return None, f"USAGE_INVALIDO: {err_costo}"
        try:
            contenido = r["choices"][0]["message"]["content"]
        except (KeyError, IndexError) as e:
            ultimo = f"{type(e).__name__}: {str(e)[:120]}"
            time.sleep(2 ** i)
            continue
        d, err = parsea(contenido, paq["candidatos"])
        if d is not None:
            return {"id": paq["id"], **d, "usage": {k: uso.get(k) for k in ("prompt_tokens", "completion_tokens")}}, None
        ultimo = err
        time.sleep(2 ** i)
    return None, ultimo


def procesa_lote(ruta_paquetes, out, key, model, precio_entrada, precio_salida,
                  tope_usd=TOPE_USD_DEFAULT, maximo=0, http=_http):
    """Corre `kimi()` sobre cada paquete de `ruta_paquetes`, escribiendo en
    `out` (reanudable: salta los ids ya presentes en `out`). Al reanudar,
    hereda el gasto ya persistido en `out` (`reconstruye_gasto`, C1); nunca
    arranca en $0 si ya hay filas escritas. Acumula el coste real de CADA
    llamada facturada -pase o no `parsea()` después- y para en seco -sin
    lanzar la siguiente llamada, ni dentro de los reintentos de un mismo
    paquete (C2)- en cuanto el acumulado SUPERA `tope_usd`. Un `usage`
    inutilizable en una respuesta nueva para el pipeline en seco de
    inmediato (`parada_por_usage_invalido`, C3), sin estimar nada. Devuelve
    un resumen; `parada_por_tope` (id en el que se paró) solo aparece si el
    breaker disparó por tope, y `parada_por_usage_invalido` solo si paró por
    usage inutilizable."""
    hechos, gasto, tok = reconstruye_gasto(out, precio_entrada, precio_salida)
    if gasto:
        print(f"gasto heredado de {out} (reanudación): ${gasto:.4f} "
              f"({len(hechos)} filas ya escritas)", file=sys.stderr)

    errores, n, parado_en, parado_por_usage = [], 0, None, None

    def tope_alcanzado():
        return gasto > tope_usd

    def contabiliza(usage):
        nonlocal gasto
        c, err = costo(usage, precio_entrada, precio_salida)
        if err is not None:
            return None, err
        gasto += c
        tok[0] += usage["prompt_tokens"]
        tok[1] += usage["completion_tokens"]
        return c, None

    with open(out, "a", encoding="utf-8") as fh, open(ruta_paquetes, encoding="utf-8") as fpaq:
        for l in fpaq:
            if not l.strip():
                continue
            paq = json.loads(l)
            if paq["id"] in hechos or (maximo and n >= maximo):
                continue
            if tope_alcanzado():
                parado_en = paq["id"]
                print(f"TOPE SUPERADO: ${gasto:.4f} > ${tope_usd} antes de procesar id={parado_en}; "
                      "parada en seco. Reanudable por id (vuelve a correr con el mismo --out).",
                      file=sys.stderr)
                break
            d, err = kimi(paq, key, model, http=http, contabiliza=contabiliza, tope_alcanzado=tope_alcanzado)
            n += 1
            if d is None:
                if err is not None and err.startswith("USAGE_INVALIDO"):
                    parado_por_usage = paq["id"]
                    print(f"USAGE INUTILIZABLE: {err} en id={parado_por_usage}; parada en seco "
                          "-un breaker que no sabe lo que gastó no puede seguir gastando-. "
                          "Reanudable por id (vuelve a correr con el mismo --out).", file=sys.stderr)
                    break
                errores.append({"id": paq["id"], "error": err})
                if tope_alcanzado():
                    parado_en = paq["id"]
                    print(f"TOPE SUPERADO: ${gasto:.4f} > ${tope_usd} en id={parado_en} "
                          "(alcanzado durante los reintentos); parada en seco.", file=sys.stderr)
                    break
                continue
            fh.write(json.dumps(d, ensure_ascii=False) + "\n")
            fh.flush()
            print(f"[{n}] {paq['id']} · gasto acumulado: ${gasto:.4f}", file=sys.stderr)
            if tope_alcanzado():
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
    if parado_por_usage is not None:
        resultado["parada_por_usage_invalido"] = parado_por_usage
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
                      help=f"tope de gasto acumulado en USD. Criterio: SUPERAR (>), nunca alcanzar "
                           f"(>=) -un gasto que iguala el tope exactamente no para, se gasta hasta el "
                           f"tope inclusive-. Al superarlo para en seco antes de la siguiente llamada, "
                           f"incluso entre reintentos de un mismo paquete (default {TOPE_USD_DEFAULT}, "
                           f"plan §Global Constraints). Reanudable: si --out ya tiene filas, el gasto "
                           f"heredado se suma antes de seguir.")
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
        if "parada_por_usage_invalido" in resultado:
            sys.exit(4)
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
