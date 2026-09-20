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

Enmienda 2026-09-20 (c), segunda ronda de review adversarial — (b) seguía
ciego en cinco caminos más, y la cota real de fuga no eran céntimos sino EL
TOPE ENTERO: `--out` solo contiene las filas que pasaron `parsea()`, así que
todo lo facturado que no llegó a fila (reintentos fallidos, timeouts,
crashes a medio flush) era invisible para `reconstruye_gasto()`. Cambio de
diseño que cierra los cinco hallazgos a la vez: un **diario de gasto
append-only**, `<out>.gasto.jsonl`, con una línea por llamada FACTURADA,
escrita ANTES de validar la respuesta (`{id, intento, prompt_tokens,
completion_tokens, usd, ts, precio_entrada, precio_salida,
motivo_desconocida}`). El gasto acumulado se reconstruye SIEMPRE del
diario (`reconstruye_gasto()`), nunca de `--out`.

Enmienda 2026-09-20 (d), tercera ronda de review adversarial — (c) resolvió
que el diario fuera la fuente de verdad, pero dejó dos huecos igual de
graves: nada obligaba a que el diario y `--out` fueran EL MISMO trabajo
(C1), y el diario seguía escribiéndose DESPUÉS de que la llamada volviera,
no antes de lanzarla (C2, el hueco real de (c) no era el mismo que el de
(b): (b) cubría excepciones capturadas dentro del proceso vivo, no la
muerte del proceso a medio HTTP).
- **C1 — diario y `--out` deben ser VERIFICABLEMENTE coherentes o el
  pipeline no arranca:** antes, `gasto` se reconstruía del diario y `hechos`
  (ids a saltar) de `--out` por caminos totalmente independientes — sin
  checksum, sin job-id, sin recuento cruzado, solo el nombre de fichero
  posicional `<out>.gasto.jsonl`. Un `--out` con filas válidas sin su
  diario (borrado, vacío, `--out` reanudado con typo/timestamp regenerado,
  o un diario de OTRO trabajo pegado en esa ruta) arrancaba en `$0.0000`
  sin un solo aviso: el tope dejaba de ser un tope acumulado y pasaba a ser
  "$10 por cada vez que se pierde la traza", sin techo garantizado. Ahora
  `procesa_lote` verifica ANTES de tocar la red:
    1. cada id con fila válida en `--out` tiene una entrada `"tipo": "ok"`
       correspondiente en el diario (`_lee_diario_meta`) — si falta alguna,
       aborta con `SystemExit` listando exactamente qué ids faltan;
    2. el diario tiene contenido pero `--out` no existe — aborta (el `--out`
       se borró, o el diario no es de este `--out`);
    3. la cabecera del diario (`"tipo": "header"`, escrita una sola vez al
       crear el diario, con un `job_id` y la identidad de fichero
       `(st_dev, st_ino)` de `--out` en ese momento) no coincide con la
       identidad actual de `--out` — aborta (diario de otro trabajo pegado
       en esta ruta, aunque los ids coincidan por casualidad).
  Si alguna verificación falla, el pipeline NO arranca salvo que el
  operador pase `--acepto-riesgo-gasto-no-verificable` (nombre largo y
  explícito a propósito: no se activa sin querer) — y en ese caso queda un
  rastro `"tipo": "override_trazabilidad"` en el propio diario con el
  detalle de qué se aceptó y el gasto heredado en ese momento, además de un
  aviso por stderr. Límite conocido: sin cabecera (diario legado de antes
  de (d)) la verificación 3 no puede aplicarse — solo protege diarios
  creados desde esta enmienda en adelante.
- **C2 — hueco entre "la llamada sale" y "se escribe el diario":** antes,
  la línea se escribía DESPUÉS de que `urlopen()` retornara; un `kill -9`
  real entre el envío y esa escritura dejaba el diario sin ninguna traza de
  una llamada ya facturada (~$0,02-0,03 por evento con los precios de
  kimi-k3, verificado matando un proceso de verdad). Ahora cada intento
  escribe una marca `"tipo": "en_vuelo"` en el diario ANTES de llamar a la
  API, con `flush()` + `os.fsync()` para sobrevivir a un `kill -9`; al
  resolverse (con coste real, como desconocida, o como usage inválido) se
  escribe una segunda línea que la resuelve por `(id, intento)`. Al
  reconstruir, una marca en vuelo que quedó sin resolver (proceso muerto
  entre el fsync de la marca y la resolución) cuenta como DESCONOCIDA —
  nunca como cero. Residuo conocido, documentado aquí porque cerrarlo del
  todo es imposible: `fsync()` es una garantía del sistema operativo, no
  del hardware — en un disco con caché de escritura sin protección de
  batería, o ciertos discos virtualizados sin passthrough correcto de
  flush, un `fsync()` que retorna éxito puede no sobrevivir a un corte de
  corriente real (no a un `kill -9`, que SÍ está cubierto y verificado). Es
  un riesgo de la capa de almacenamiento, no de este código, y es
  estrictamente mejor que el estado anterior (sin marca alguna). Además,
  si el proceso muere en la ventana microscópica entre "se escribe y
  fsyncea la marca en vuelo" y "se dispara la llamada real", la marca queda
  sin resolver y se cuenta como desconocida aunque no se facturara nada:
  un falso positivo (el breaker se para de más), nunca un falso negativo
  (nunca deja de contar un gasto real) — la asimetría es intencional.
- **Important — el diario distingue el MOTIVO del `usd: null`:** antes,
  `USAGE_INVALIDO` (bug de schema: la API respondió pero `usage` no es
  utilizable, C3) y una desconocida real (F1: la respuesta se perdió por
  timeout/URLError, nunca llegó) quedaban indistinguibles (`usd: null` en
  ambos casos). Consecuencia: tras una parada por `USAGE_INVALIDO` ya
  resuelta por el operador, reanudar consumía para siempre 1 unidad de
  `--tolerancia-desconocidas` por esa línea vieja, aunque no tuviera nada
  que ver con una llamada perdida. Ahora cada línea del diario lleva
  `"tipo"` (`ok` | `desconocida` | `usage_invalido` | `en_vuelo` | `header`
  | `override_trazabilidad`) y solo `"tipo": "desconocida"` consume la
  tolerancia al reconstruir; `usage_invalido` heredado se cuenta aparte
  (informativo) y nunca dispara `parada_por_gasto_desconocido` por sí solo.
- **Minor — `flock` y NFS:** el código ya degrada explícitamente si
  `fcntl` no existe (F2), pero eso no cubre el caso donde `fcntl` SÍ existe
  y el lock se toma "con éxito" sobre un filesystem donde `flock()` no es
  fiable (NFS, sobre todo NFSv3 o exports mal configurados: el lock puede
  no ser visible entre distintos clientes). Esto es una propiedad del
  filesystem, no detectable desde Python sin dependencias extra — se deja
  documentado aquí y en el `--help ` en vez de fingir una garantía que el
  código no puede verificar.
- **F1 — timeout/URLError con la API ya facturada:** un intento que SALE
  hacia la API pero cuya respuesta se pierde (`URLError`, `TimeoutError`,
  JSON de la API no parseable) es una llamada de facturación DESCONOCIDA:
  se registra en el diario con `usd: null` (nunca se inventan tokens ni se
  asume coste cero) vía el callback `contabiliza_desconocida` de `kimi()`.
  Política (criterio de Paul, documentada también en `--help` de
  `--tolerancia-desconocidas`): por defecto (tolerancia 0) la PRIMERA
  llamada perdida para el pipeline en seco (`parada_por_gasto_desconocido`,
  código de salida 5) — un breaker que no sabe cuánto lleva gastado no
  puede seguir gastando. El operador puede subir `--tolerancia-desconocidas`
  para asumir el riesgo explícitamente; se acumula entre reanudaciones (lo
  heredado del diario cuenta antes de lanzar la siguiente llamada, igual
  que el tope de gasto).
- **F2 — sin lock, dos procesos con el mismo `--out` casi doblan el
  gasto:** `procesa_lote` toma un lock exclusivo (`fcntl.flock`, stdlib)
  sobre `<out>.gasto.jsonl.lock` durante toda la corrida; un segundo
  proceso con el mismo `--out` falla rápido con `SystemExit` en vez de
  correr en paralelo. Solo POSIX (Linux/macOS): sin `fcntl` disponible se
  avisa explícitamente por stderr de que el lock NO está activo, en vez de
  fingir que lo está.
- **F3 — `NaN` apaga el freno de forma permanente:** `_numerico()` exige
  `math.isfinite()` además de ser `int`/`float` — `NaN`/`Infinity` son
  `float` mas no finitos, y `json.loads` acepta esos literales. Antes,
  `gasto` podía quedar en `NaN` y `gasto > tope` pasaba a ser SIEMPRE
  `False` (también tras reanudar). `costo()` también rechaza tokens
  negativos.
- **F4 — `--out`/diario con la última línea truncada** (crash a medio
  flush) da un `SystemExit` explícito (`_lee_jsonl_robusto`), no un
  `json.JSONDecodeError` crudo.
- **F5 — precios distintos entre run y resume no se detectan:** cada línea
  del diario persiste el `usd` YA CALCULADO con el precio vigente en ese
  momento (no solo los tokens), así que un resume con precios distintos
  (el proyecto tiene dos tarifas reales de Moonshot, k3 y k2.6) nunca
  recalcula gasto viejo con precio de hoy. `reconstruye_gasto()` además
  avisa por stderr si los precios del run actual difieren de los últimos
  registrados en el diario (aviso informativo; el gasto heredado no se
  toca).

Uso: juez.py paquetes --candidatos C --snap KB --out P.jsonl
     juez.py modelos --env-keys F
     juez.py kimi --paquetes P.jsonl --env-keys F --model M --out R.jsonl
         --precio-entrada 3.00 --precio-salida 15.00 [--tope-usd 10] [--max N]
         [--tolerancia-desconocidas 0] [--acepto-riesgo-gasto-no-verificable]
     juez.py valida --respuestas R.jsonl --candidatos C
"""
import argparse
import json
import math
import os
import sys
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path

try:
    import fcntl
except ImportError:  # no-POSIX (p.ej. Windows): degrada con aviso, sin fingir lock (F2).
    fcntl = None

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
    """int/float finito -- NaN e Infinity son `float` pero no pasan (F3)."""
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def costo(usage, precio_entrada, precio_salida):
    """(usd, None) si `usage` es utilizable, o (None, error) si no. Un
    `usage` ausente, incompleto, no numérico/finito (NaN, Infinity -- F3) o
    con tokens negativos es SIEMPRE un error explícito (C3) — nunca se
    estima ni se trata como coste cero."""
    if not isinstance(usage, dict):
        return None, f"usage ausente o no es un objeto: {usage!r}"
    pt, ct = usage.get("prompt_tokens"), usage.get("completion_tokens")
    if not _numerico(pt) or not _numerico(ct):
        return None, f"usage incompleto o no numérico/finito: prompt_tokens={pt!r} completion_tokens={ct!r}"
    if pt < 0 or ct < 0:
        return None, f"usage con tokens negativos: prompt_tokens={pt!r} completion_tokens={ct!r}"
    return pt / 1_000_000 * precio_entrada + ct / 1_000_000 * precio_salida, None


def _lee_jsonl_robusto(ruta, contexto):
    """Generador (n, dict) por línea no vacía de `ruta`. Una línea que no
    parsea como JSON -p.ej. la última, truncada por un crash a medio flush-
    aborta con SystemExit explícito en vez de dejar pasar un
    json.JSONDecodeError crudo (F4)."""
    with open(ruta, encoding="utf-8") as fh:
        for n, l in enumerate(fh, start=1):
            if not l.strip():
                continue
            try:
                yield n, json.loads(l)
            except json.JSONDecodeError as e:
                raise SystemExit(
                    f"{contexto}: línea {n} de {ruta} no parsea como JSON ({e}) -probable crash "
                    "a medio flush-. Corrige o trunca el fichero a la última línea completa antes "
                    "de continuar (no se ignora en silencio)."
                )


def ids_ya_escritos(ruta_out):
    """ids con juicio ya válido en `ruta_out` (para saltarlos al reanudar).
    Robusto a una última línea truncada (F4). Ya NO es la fuente del gasto
    heredado -eso es `reconstruye_gasto()` sobre el diario-: `ruta_out` solo
    tiene las filas que pasaron `parsea()`, y por eso era ciego a lo
    facturado que no llegó a fila (enmienda (c))."""
    ids = set()
    if not Path(ruta_out).exists():
        return ids
    for _, fila in _lee_jsonl_robusto(ruta_out, "resume --out"):
        ids.add(fila["id"])
    return ids


def reconstruye_gasto(ruta_diario, precio_entrada, precio_salida):
    """Reanudación (C1/(c)), reescrita para la enmienda (d): recorre el
    diario append-only `ruta_diario` (`<out>.gasto.jsonl`) y devuelve
    (gasto_heredado, [prompt_tok, completion_tok], desconocidas_heredadas,
    usage_invalidas_heredadas). El diario es la fuente de verdad del gasto,
    NO `--out` -ver `_lee_diario_meta()` para la verificación de coherencia
    entre ambos, que corre ANTES de esto en `procesa_lote`-.
    Cada línea lleva `"tipo"` desde (d): `header` y `override_trazabilidad`
    se ignoran para el cómputo; `en_vuelo` (marca escrita ANTES de lanzar la
    llamada, C2) se guarda como pendiente hasta encontrar su línea de
    resolución por `(id, intento)` -si el diario se acaba con marcas sin
    resolver (proceso muerto entre facturar y resolver), cada una cuenta
    como DESCONOCIDA, nunca como cero-. `ok` y `usage_invalido` resuelven
    la marca en vuelo correspondiente; solo `desconocida` (F1: respuesta
    perdida, nunca llegó) consume la tolerancia -`usage_invalido` (C3: la
    respuesta llegó pero el `usage` no es utilizable, un bug de schema) se
    cuenta aparte en `usage_invalidas_heredadas`, informativo, para no
    consumir para siempre `--tolerancia-desconocidas` por un motivo que no
    tiene nada que ver con una llamada perdida (hallazgo Important de (d)).
    Diarios legados de antes de (d) no tienen `"tipo"`: por compatibilidad
    conservadora, una línea sin `"tipo"` y `usd: null` cuenta como
    `desconocida` (el comportamiento de antes de (d), que ya era seguro).
    Avisa por stderr si los precios del run actual difieren de los últimos
    registrados en el diario (F5) -el `usd` ya persistido por llamada no se
    recalcula con los precios de hoy."""
    gasto, tok, desconocidas, usage_invalidas = 0.0, [0, 0], 0, 0
    if not Path(ruta_diario).exists():
        return gasto, tok, desconocidas, usage_invalidas
    precios_vistos = None
    en_vuelo_pendientes = {}
    for n, fila in _lee_jsonl_robusto(ruta_diario, "diario de gasto"):
        tipo = fila.get("tipo")
        if tipo in ("header", "override_trazabilidad"):
            continue
        clave = (fila.get("id"), fila.get("intento"))
        if tipo == "en_vuelo":
            en_vuelo_pendientes[clave] = n
            continue
        en_vuelo_pendientes.pop(clave, None)  # esta línea resuelve la marca en vuelo, si había
        if "precio_entrada" in fila and "precio_salida" in fila:
            precios_vistos = (fila["precio_entrada"], fila["precio_salida"])
        usd = fila.get("usd")
        if usd is None:
            if tipo == "usage_invalido":
                usage_invalidas += 1
            else:  # tipo == "desconocida", o diario legado sin "tipo" (compat conservadora)
                desconocidas += 1
            continue
        if not _numerico(usd) or usd < 0:
            raise SystemExit(
                f"diario de gasto corrupto: línea {n} (id={fila.get('id')}) de {ruta_diario} "
                f"tiene usd={usd!r} (no numérico/finito o negativo). El breaker no puede heredar "
                "un gasto que no sabe leer; corrige o descarta esa línea antes de reanudar (no "
                "se estima ni se asume cero)."
            )
        gasto += usd
        tok[0] += fila.get("prompt_tokens") or 0
        tok[1] += fila.get("completion_tokens") or 0
    # marca(s) en vuelo sin resolver: el proceso murió entre facturar y escribir la resolución
    # (C2) -- se cuentan como DESCONOCIDAS, nunca como cero.
    desconocidas += len(en_vuelo_pendientes)
    if precios_vistos is not None and precios_vistos != (precio_entrada, precio_salida):
        print(
            f"AVISO: los precios del run actual (entrada=${precio_entrada}/salida=${precio_salida}) "
            f"difieren de los últimos registrados en {ruta_diario} (entrada=${precios_vistos[0]}/"
            f"salida=${precios_vistos[1]}). El gasto heredado usa el usd YA PERSISTIDO por llamada "
            "-no se recalcula con el precio de hoy-; esto es solo un aviso por si el operador se "
            "equivocó de tarifa (el proyecto usa tarifas reales distintas de Moonshot, k3 y k2.6).",
            file=sys.stderr,
        )
    return gasto, tok, desconocidas, usage_invalidas


def _lee_diario_meta(ruta_diario):
    """Metadatos del diario para la verificación de coherencia diario/`--out`
    (C1, enmienda (d)): la cabecera (`"tipo": "header"`, `{job_id, out_dev,
    out_ino, out_path, ts}`, escrita una sola vez al crear el diario -- o
    `None` si el diario no existe o es legado sin cabecera) y el conjunto de
    ids con al menos una llamada resuelta con éxito (`"tipo": "ok"`) -- las
    únicas que pueden corresponder a una fila válida en `--out`, porque
    `parsea()` solo se intenta después de que `contabiliza()` confirme un
    coste conocido en ese mismo intento."""
    header = None
    ids_ok = set()
    if not Path(ruta_diario).exists():
        return header, ids_ok
    for _, fila in _lee_jsonl_robusto(ruta_diario, "diario de gasto (metadatos)"):
        if fila.get("tipo") == "header":
            if header is None:
                header = fila
            continue
        if fila.get("tipo") == "ok":
            ids_ok.add(fila.get("id"))
    return header, ids_ok


def kimi(paq, key, model, reintentos=3, http=_http, contabiliza=None,
         contabiliza_desconocida=None, tope_alcanzado=None, marca_en_vuelo=None):
    """Intenta hasta `reintentos` veces. Cada intento que SÍ recibe
    respuesta de la API es una llamada FACTURADA y se pasa a
    `contabiliza(usage, intento)` -- se valide o no después con `parsea()`
    (C2): un permalink alucinado que pasa el json_schema pero falla la
    validación de negocio se pagó igual. Si `contabiliza` devuelve error
    (usage inutilizable, C3), `kimi()` para en seco de inmediato.
    Un intento que SALE hacia la API pero cuya respuesta se pierde
    (`URLError`, `TimeoutError`, JSON de la API no parseable) es una llamada
    de facturación DESCONOCIDA (F1): nunca se inventan tokens ni se asume
    coste cero, se reporta vía `contabiliza_desconocida(intento, motivo)`.
    Si esa devuelve error (tolerancia de desconocidas agotada, ver
    `procesa_lote`), `kimi()` para en seco igual que con `USAGE_INVALIDO`.
    Antes de cada intento -incluidos los reintentos- se consulta
    `tope_alcanzado()`; si ya se rebasó, NO se lanza el intento (C2).
    `marca_en_vuelo(intento)` (enmienda (d), C2) se llama JUSTO ANTES de
    `http()`, para que un `kill -9` real entre el envío y el retorno deje
    una marca ya fsyncada en el diario -sin ella, ese hueco no dejaba
    ninguna traza-."""
    ultimo = None
    for i in range(reintentos):
        if tope_alcanzado is not None and tope_alcanzado():
            return None, "tope superado: reintento no lanzado"
        if marca_en_vuelo is not None:
            marca_en_vuelo(i + 1)
        try:
            r = http(f"{BASE}/chat/completions", key, cuerpo_peticion(paq, model))
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
            motivo = f"{type(e).__name__}: {str(e)[:120]}"
            ultimo = motivo
            if contabiliza_desconocida is not None:
                _, err_desc = contabiliza_desconocida(i + 1, motivo)
                if err_desc is not None:
                    return None, f"GASTO_DESCONOCIDO: {err_desc}"
            time.sleep(2 ** i)
            continue
        uso = r.get("usage")
        if contabiliza is not None:
            _, err_costo = contabiliza(uso, i + 1)
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
                  tope_usd=TOPE_USD_DEFAULT, maximo=0, http=_http, reintentos=3,
                  tolerancia_desconocidas=0, acepto_riesgo_gasto=False):
    """Corre `kimi()` sobre cada paquete de `ruta_paquetes`, escribiendo en
    `out` (reanudable: salta los ids ya presentes en `out`, `ids_ya_escritos`
    -F4-). Mantiene un diario append-only `<out>.gasto.jsonl` (enmienda (c)):
    una línea por llamada FACTURADA -coste conocido o DESCONOCIDA (F1)-, con
    una marca `en_vuelo` fsyncada ANTES de lanzar la llamada y resuelta
    después (enmienda (d), C2). El gasto acumulado se reconstruye SIEMPRE de
    ese diario (`reconstruye_gasto`), nunca de `--out` -que solo tiene las
    filas que pasaron `parsea()`, y por eso era ciego a reintentos fallidos,
    timeouts y crashes a medio flush-.
    Antes de tocar la red, verifica que el diario y `--out` sean
    VERIFICABLEMENTE coherentes (C1, enmienda (d)): cada id con fila válida
    en `--out` debe tener una entrada `"tipo": "ok"` en el diario, el diario
    no puede tener contenido si `--out` no existe, y si el diario tiene
    cabecera (`job_id` + identidad de fichero de `--out`) esa identidad debe
    coincidir con la de `--out` actual. Si algo no cuadra, aborta con
    `SystemExit` -salvo que `acepto_riesgo_gasto` sea `True` (bandera
    explícita, deja rastro `"tipo": "override_trazabilidad"` en el diario).
    Toma un lock exclusivo (F2) sobre `<out>.gasto.jsonl.lock` durante toda
    la corrida: un segundo proceso con el mismo `--out` falla rápido con
    `SystemExit` en vez de doblar el gasto corriendo en paralelo (nota: esto
    depende de que `flock()` sea fiable en el filesystem -no lo es siempre
    sobre NFS-, no solo de que `fcntl` exista).
    Para en seco -sin lanzar la siguiente llamada, ni dentro de los
    reintentos de un mismo paquete (C2)- en cuanto el acumulado SUPERA
    `tope_usd` (`parada_por_tope`). Un `usage` inutilizable en una respuesta
    nueva para en seco de inmediato (`parada_por_usage_invalido`, C3, código
    4). Más de `tolerancia_desconocidas` llamadas de facturación desconocida
    -heredadas del diario o nuevas de esta corrida; `usage_invalido` NO
    cuenta para esto, ver `reconstruye_gasto`- para en seco de inmediato
    (`parada_por_gasto_desconocido`, F1, código 5); default 0: la primera ya
    para. Devuelve un resumen con `desconocidas` (recuento total, heredadas
    + nuevas) siempre presente."""
    ruta_gasto = f"{out}.gasto.jsonl"
    ruta_lock = f"{ruta_gasto}.lock"

    lock_fh = open(ruta_lock, "w")
    if fcntl is not None:
        try:
            fcntl.flock(lock_fh, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            lock_fh.close()
            raise SystemExit(
                f"otro proceso ya tiene el lock de {ruta_gasto} (mismo --out): dos procesos "
                "concurrentes sobre el mismo --out casi doblan el gasto (F2). Espera a que "
                "termine el otro proceso, o usa un --out distinto."
            )
    else:
        print("AVISO: plataforma sin fcntl (no-POSIX, p.ej. Windows): el lock de concurrencia "
              "NO está activo; no lances dos procesos con el mismo --out a la vez.", file=sys.stderr)

    try:
        # --- C1 (enmienda (d)): diario y --out deben ser verificablemente coherentes. ---
        hechos = ids_ya_escritos(out)
        header, ids_ok_diario = _lee_diario_meta(ruta_gasto)
        faltan = hechos - ids_ok_diario

        existe_out = Path(out).exists()
        identidad_out = None
        if existe_out:
            st_out_previo = Path(out).stat()
            identidad_out = (st_out_previo.st_dev, st_out_previo.st_ino)

        diario_path = Path(ruta_gasto)
        diario_no_vacio = diario_path.exists() and diario_path.stat().st_size > 0
        necesita_header = not diario_no_vacio

        incoherencias = []
        if faltan:
            incoherencias.append(
                f"{len(faltan)} id(s) con fila válida en {out} sin entrada 'tipo':'ok' "
                f"correspondiente en el diario {ruta_gasto}: {sorted(faltan)}. Ese gasto quedaría "
                "invisible para el breaker (nunca $0 en silencio)."
            )
        if diario_no_vacio and not existe_out:
            incoherencias.append(
                f"el diario {ruta_gasto} ya tiene contenido pero {out} no existe: o se borró el "
                "--out original, o este diario no corresponde a este --out."
            )
        if header is not None and identidad_out is not None:
            header_id = (header.get("out_dev"), header.get("out_ino"))
            if header_id != identidad_out:
                incoherencias.append(
                    f"la cabecera del diario {ruta_gasto} (job_id={header.get('job_id')!r}) está "
                    f"ligada a un --out con identidad de fichero {header_id}, distinta de la de "
                    f"{out} actual ({identidad_out}): probable diario de otro trabajo en esta ruta "
                    "(aunque los ids coincidan)."
                )

        if incoherencias and not acepto_riesgo_gasto:
            raise SystemExit(
                "diario de gasto incoherente con --out; el breaker no puede reconstruir el gasto "
                "de forma fiable y NO arranca (si arrancara igual, el tope '$X' pasaría a ser "
                "'$X por cada vez que se pierde la traza', sin techo garantizado):\n- "
                + "\n- ".join(incoherencias)
                + "\nOpciones: (1) reconstruye el diario a mano añadiendo, por cada id listado "
                'arriba, una línea {"tipo": "ok", "id": "<id>", "usd": <coste real>} con su coste '
                "real; o (2) si aceptas explícitamente arrancar sin poder verificar esa traza (el "
                "gasto heredado será el que el diario permita reconstruir, potencialmente $0), "
                "vuelve a correr con --acepto-riesgo-gasto-no-verificable -queda registrado en el "
                "propio diario, con el detalle de qué se aceptó-."
            )

        gasto, tok, desconocidas, usage_invalidas_heredadas = reconstruye_gasto(
            ruta_gasto, precio_entrada, precio_salida)
        if gasto or desconocidas or usage_invalidas_heredadas:
            print(f"gasto heredado del diario {ruta_gasto} (reanudación): ${gasto:.4f} "
                  f"({desconocidas} llamadas de facturación desconocida heredadas, "
                  f"{usage_invalidas_heredadas} de usage inválido heredadas -no consumen "
                  f"tolerancia-, {len(hechos)} filas ya escritas en {out})", file=sys.stderr)

        errores, n = [], 0
        parado_en = parado_por_usage = parado_por_desconocidas = None
        paq_actual = None

        def tope_alcanzado():
            return gasto > tope_usd

        def desconocidas_superadas():
            return desconocidas > tolerancia_desconocidas

        def _diario(entrada):
            # flush + fsync en cada línea (C2, enmienda (d)): una marca en_vuelo -o cualquier
            # otra línea- debe sobrevivir a un kill -9 real, no solo a una excepción capturada.
            fh_diario.write(json.dumps(entrada, ensure_ascii=False) + "\n")
            fh_diario.flush()
            os.fsync(fh_diario.fileno())

        def marca_en_vuelo(intento):
            _diario({"tipo": "en_vuelo", "id": paq_actual["id"], "intento": intento, "ts": time.time()})

        def contabiliza(usage, intento):
            nonlocal gasto
            c, err = costo(usage, precio_entrada, precio_salida)
            _diario({
                "tipo": "usage_invalido" if err is not None else "ok",
                "id": paq_actual["id"], "intento": intento, "ts": time.time(),
                "prompt_tokens": usage.get("prompt_tokens") if isinstance(usage, dict) else None,
                "completion_tokens": usage.get("completion_tokens") if isinstance(usage, dict) else None,
                "usd": None if err is not None else c,
                "precio_entrada": precio_entrada, "precio_salida": precio_salida,
                "motivo_desconocida": err,
            })
            if err is not None:
                return None, err
            gasto += c
            tok[0] += usage["prompt_tokens"]
            tok[1] += usage["completion_tokens"]
            return c, None

        def contabiliza_desconocida(intento, motivo):
            nonlocal desconocidas
            _diario({
                "tipo": "desconocida",
                "id": paq_actual["id"], "intento": intento, "ts": time.time(),
                "prompt_tokens": None, "completion_tokens": None, "usd": None,
                "precio_entrada": precio_entrada, "precio_salida": precio_salida,
                "motivo_desconocida": motivo,
            })
            desconocidas += 1
            if desconocidas_superadas():
                return None, (
                    f"{desconocidas} llamadas de facturación desconocida (tolerancia="
                    f"{tolerancia_desconocidas}); el breaker no sabe cuánto lleva gastado y no "
                    "puede seguir (usa --tolerancia-desconocidas para asumir el riesgo "
                    "explícitamente)."
                )
            return True, None

        with open(out, "a", encoding="utf-8") as fh, \
             open(ruta_gasto, "a", encoding="utf-8") as fh_diario, \
             open(ruta_paquetes, encoding="utf-8") as fpaq:
            if necesita_header:
                st_out = os.fstat(fh.fileno())
                _diario({
                    "tipo": "header", "job_id": uuid.uuid4().hex,
                    "out_dev": st_out.st_dev, "out_ino": st_out.st_ino,
                    "out_path": str(Path(out).resolve()), "ts": time.time(),
                })
            if incoherencias and acepto_riesgo_gasto:
                print(
                    "AVISO: --acepto-riesgo-gasto-no-verificable activo; se arranca pese a "
                    "incoherencia(s) diario/--out: " + " | ".join(incoherencias)
                    + f" -- gasto heredado reconstruido del diario: ${gasto:.4f}.",
                    file=sys.stderr,
                )
                _diario({
                    "tipo": "override_trazabilidad", "ts": time.time(),
                    "detalle": incoherencias, "gasto_heredado_tras_override": round(gasto, 4),
                })
            for l in fpaq:
                if not l.strip():
                    continue
                paq = json.loads(l)
                if paq["id"] in hechos or (maximo and n >= maximo):
                    continue
                paq_actual = paq
                if tope_alcanzado():
                    parado_en = paq["id"]
                    print(f"TOPE SUPERADO: ${gasto:.4f} > ${tope_usd} antes de procesar id={parado_en}; "
                          "parada en seco. Reanudable por id (vuelve a correr con el mismo --out).",
                          file=sys.stderr)
                    break
                if desconocidas_superadas():
                    parado_por_desconocidas = paq["id"]
                    print(f"GASTO DESCONOCIDO: {desconocidas} llamadas de facturación desconocida "
                          f"(tolerancia={tolerancia_desconocidas}) antes de procesar id="
                          f"{parado_por_desconocidas}; parada en seco. Reanudable por id.", file=sys.stderr)
                    break
                d, err = kimi(paq, key, model, reintentos=reintentos, http=http, contabiliza=contabiliza,
                              contabiliza_desconocida=contabiliza_desconocida, tope_alcanzado=tope_alcanzado,
                              marca_en_vuelo=marca_en_vuelo)
                n += 1
                if d is None:
                    if err is not None and err.startswith("USAGE_INVALIDO"):
                        parado_por_usage = paq["id"]
                        print(f"USAGE INUTILIZABLE: {err} en id={parado_por_usage}; parada en seco "
                              "-un breaker que no sabe lo que gastó no puede seguir gastando-. "
                              "Reanudable por id (vuelve a correr con el mismo --out).", file=sys.stderr)
                        break
                    if err is not None and err.startswith("GASTO_DESCONOCIDO"):
                        parado_por_desconocidas = paq["id"]
                        print(f"GASTO DESCONOCIDO: {err} en id={parado_por_desconocidas}; parada en "
                              "seco. Reanudable por id.", file=sys.stderr)
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
                     "gasto_usd": round(gasto, 4), "tope_usd": tope_usd, "desconocidas": desconocidas}
        if parado_en is not None:
            resultado["parada_por_tope"] = parado_en
        if parado_por_usage is not None:
            resultado["parada_por_usage_invalido"] = parado_por_usage
        if parado_por_desconocidas is not None:
            resultado["parada_por_gasto_desconocido"] = parado_por_desconocidas
        return resultado
    finally:
        if fcntl is not None:
            try:
                fcntl.flock(lock_fh, fcntl.LOCK_UN)
            except OSError:
                pass
        lock_fh.close()


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
                           f"plan §Global Constraints). Reanudable: el gasto heredado se reconstruye "
                           f"del diario append-only <out>.gasto.jsonl -no de --out- antes de seguir.")
    a_k.add_argument("--tolerancia-desconocidas", type=int, default=0, dest="tolerancia_desconocidas",
                      help="cuántas llamadas de facturación DESCONOCIDA tolerar antes de parar en seco "
                           "-la API se llamó y pudo facturar, pero la respuesta se perdió por timeout/"
                           "URLError/JSON inválido, así que no hay usage que contar; nunca se inventan "
                           "tokens ni se asume coste cero-. Default 0: la PRIMERA llamada perdida para "
                           "el pipeline (código de salida 5) -un breaker que no sabe cuánto lleva "
                           "gastado no puede seguir gastando-. Sube este número solo si asumes "
                           "explícitamente el riesgo de gasto no contabilizado hasta ese número de "
                           "llamadas perdidas. Se acumula entre reanudaciones: lo heredado del diario "
                           "cuenta antes de lanzar la siguiente llamada, igual que el tope de gasto. "
                           "Una llamada 'usage_invalido' (C3: la API respondió pero el usage no es "
                           "utilizable) NO consume esta tolerancia -es un motivo distinto, ver "
                           "--acepto-riesgo-gasto-no-verificable para el otro tipo de incoherencia-.")
    a_k.add_argument("--acepto-riesgo-gasto-no-verificable", action="store_true",
                      dest="acepto_riesgo_gasto",
                      help="BANDERA DE ALTO RIESGO, nombre largo a propósito para que no se active "
                           "sin querer. El pipeline verifica que el diario <out>.gasto.jsonl y --out "
                           "sean coherentes antes de arrancar (cada id con fila válida en --out tiene "
                           "su entrada 'ok' en el diario, y la identidad de --out coincide con la que "
                           "el diario tiene registrada en su cabecera); si no lo son, aborta en vez de "
                           "arrancar el gasto heredado en $0 en silencio. Esta bandera salta esa "
                           "verificación y arranca igual -el gasto heredado será el que el diario "
                           "permita reconstruir, potencialmente $0 aunque --out ya tenga filas con "
                           "coste real-. Queda registrado en el propio diario ('tipo': "
                           "'override_trazabilidad', con el detalle de qué se aceptó) y se avisa por "
                           "stderr. Úsala solo si entiendes y aceptas que el tope deja de ser una cota "
                           "fiable de lo gastado en total.")
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
                                  tope_usd=a.tope_usd, maximo=a.max,
                                  tolerancia_desconocidas=a.tolerancia_desconocidas,
                                  acepto_riesgo_gasto=a.acepto_riesgo_gasto)
        print(json.dumps(resultado, ensure_ascii=False))
        if "parada_por_usage_invalido" in resultado:
            sys.exit(4)
        if "parada_por_gasto_desconocido" in resultado:
            sys.exit(5)
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
