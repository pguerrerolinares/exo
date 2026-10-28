# Plan: gate Stop «delegar-decision»

> For agentic workers: ejecución con `exo:orchestrate`.

**Plan:** 75 min, 5 turnos de executor, 16 KB

**Goal:** que el padre no cierre turno pidiendo visto bueno a una propuesta sin delegar: un hook Stop que bloquea esos cierres (P2∨P3, sin `Agent`/`Task` en el segmento), con exclusión de reserva, tope de 1 bloqueo por mensaje humano y telemetría para el kill; y brainstorm reescrito a «consultor primero, Paul firma».

**Architecture:** el detector es un módulo Python puro en el plugin (`delegar-decision.py`: segmentación, P2/P3, exclusión, etiqueta, banda) con modo CLI para el hook. El hook bash (`delegar-decision.sh`) hace las abstenciones, el sentinel y el log, y emite el block. El backtest de `evals/uso-real/` carga ese mismo módulo por ruta, de modo que la regresión mide el código que se despliega; los datos personales se leen de `$EXO_DELEGAR_DATOS` y nunca entran en el repo.

**Tech Stack:** bash + jq, Python 3 stdlib (`re`, `json`, `importlib.util`, `unittest`), markdown de skills.

**Spec:** `docs/superpowers/specs/2026-10-28-delegar-decision-design.md`

**Global Constraints:**
- `PERM` y `PROP`, verbatim de `~/.cache/exo-delegar/referencia/patrones.py`:7-8 (copia de `turnos-8/patrones.py`). No se amplían sin medirlo (spec §4.1).
- `tail` = los últimos 700 caracteres del último bloque `text` del segmento. P2 = `PERM.search(tail)`. P3 = `PROP.search(último_texto) and ("?" in tail or ask)`, con `ask` = el segmento contiene un `tool_use` `AskUserQuestion`. Igual que `patrones.py`.
- Condición 1: el segmento no contiene ningún `tool_use` con `name` en `("Agent", "Task")`.
- Etiqueta de escape: solo cuenta si el último texto **empieza** (tras `lstrip`) por `[reserva: X]` con X ∈ `externo|irreversible|re-scope|hecho-de-Paul|pedido`.
- Exclusión de reserva: sobre `tail`, sin distinguir mayúsculas, con `\b`. Términos: push, PR/issue, merge, deploy, commit, `/document`, borrar. Solo esos (spec §4.2).
- Texto exacto del `reason` (spec §4.1): `Cerraste con propuesta + petición de visto bueno sin delegar. Lanza ahora el consultor (skills/brainstorm/consultor-brief.md) o el ejecutor y cierra con su resultado; o, si la pregunta es de Paul, reescribe el cierre empezando por [reserva: externo|irreversible|re-scope|hecho-de-Paul|pedido].`
- Sentinel: `~/.cache/exo/delegar-decision/<session_id>`, contenido = `uuid` del último mensaje humano. Escritura best-effort.
- Telemetría: `reflex_log` de `_reflex-log.sh`. Id `delegar-decision` con payload `kind=disparo patron=P2|P3 tail=<200c>`; id `delegar-decision` con `kind=p1_log` para P1∖(P2∨P3); id `delegar-decision:banda` con `kind=banda` para cierres sin `?`, sin `Agent` y sin ningún `tool_use`. Siempre `|| true`.
- Abstención (exit 0, sin salida): `agent_id` presente, `stop_hook_active` true, falta python3 o jq, transcript ilegible, sentinel ya consumido para ese uuid.
- Puerta de reproducción (Task 1): sobre las 99 sesiones selladas, bloqueos ∈ [236, 288] (262 ±10 %); sobre los 30 etiquetados, 18 positivos retenidos y retenidos totales ∈ [20, 24] (18/22 ±10 %); los 8 turnos conocidos siguen disparando 7/8. Si no se cumple: STOP y se vuelve al orquestador.
- Privacidad (repo público): transcripts, etiquetas, `muestra.json`, `ocho.json` y textos reales viven solo en `$EXO_DELEGAR_DATOS` (por defecto `~/.cache/exo-delegar/`) y `~/.claude/projects/`. En el repo solo van fixtures sintéticos y agregados.
- Versión del plugin: 1.25.0 → 1.26.0 en `plugins/exo/.claude-plugin/plugin.json` y `.claude-plugin/marketplace.json`.

## Setup del worktree

- `test -d ~/.cache/exo-delegar/db/etiquetas && test -f ~/.cache/exo-delegar/muestra.json && test -f ~/.cache/exo-delegar/ocho.json` → exit 0. (Copiados del scratchpad de la sesión 2026-10-28; los `.py` de `referencia/` llevan rutas personales y no se copian al repo.)

## Cierre de ola

- `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → verde = `OK` (los tests con datos reales salen como `skipped` si falta `$EXO_DELEGAR_DATOS`)
- `bash scripts/check-skill-refs.sh && bash scripts/check-prosa-cli.sh` → verde = exit 0
- `f=0; for t in plugins/exo/scripts/test-*.sh scripts/test-hooks-json.sh scripts/test-versiones.sh scripts/test-exec-bit.sh scripts/test-rutas-personales.sh scripts/test-shellcheck.sh; do bash "$t" </dev/null >/dev/null 2>&1 || { echo "FALLA $t"; f=1; }; done; exit $f` → verde = ninguna línea `FALLA`

## Mutación

- `plugins/exo/scripts`: manual
- `evals/uso-real`: manual

## Recursos compartidos

- `~/.claude/projects/` y `~/.cache/exo-delegar/`: solo lectura, solo Task 1 (backtest) y los tests con datos de Task 1.
- `~/.cache/exo/delegar-decision/` (sentinel) y `~/.claude/reflex-log.jsonl`: los tests usan `HOME` en `mktemp -d` y `REFLEX_LOG_FILE` propio; nunca los reales.
- `plugins/exo/.claude-plugin/plugin.json` y `.claude-plugin/marketplace.json`: solo Task 4.

## Olas

- Ola 1, en paralelo: T1 (detector + puerta) y T3 (brainstorm). Ficheros disjuntos, sin dependencias.
- Ola 2: T2 (hook), consume T1. Solo si la puerta de T1 pasó.
- Ola 3: T4 (wiring, versión y doctrina), consume T2.
- Ola 4: T5 (prueba manual del doble bloqueo), consume T4.

### Task 1: detector y puerta de reproducción

**Files:**
- Create: `plugins/exo/scripts/delegar-decision.py`
- Create: `evals/uso-real/backtest_delegar.py`
- Test: `evals/uso-real/test_delegar_detector.py`

**Interfaces:**
- Produces: `delegar-decision.py: PERM, PROP, EXCLUSION, ETIQUETA: re.Pattern`
- Produces: `delegar-decision.py: es_mensaje_humano(d: dict) -> bool` (reimplementa la lógica de `evals/uso-real/transcripts.py:25`, sin importarlo, y sin tocar ese fichero: su `filtro_sha256` sella el original)
- Produces: `delegar-decision.py: segmentar(lineas: Iterable[dict], tope: float | None = None) -> list[tuple[str | None, list[dict]]]` (clave = `uuid` del mensaje humano que cierra el segmento, `"NOHUMANO"` si lo cierra un input no humano que abre turno, `None` para el segmento final; misma regla que `turnos()` de `referencia/backtest.py`:64-86, sidechain fuera)
- Produces: `delegar-decision.py: evalua(bloques: list[dict]) -> dict` con claves `dispara: bool`, `kind: "disparo" | "p1_log" | "banda" | None`, `patron: "P2" | "P3" | None`, `excluido: "agent" | "etiqueta" | "reserva" | None`, `tail: str`
- Produces: `backtest_delegar.py` CLI: `python3 evals/uso-real/backtest_delegar.py [--datos DIR] [--proyectos DIR] [--sello evals/uso-real/sello.json]` → JSON en stdout con `sesiones`, `bloqueos`, `por_sesion` (media, mediana, máx), `muestra` (`positivos_retenidos`, `retenidos`), `ocho` (`disparan`), `banda_base` (cierres por sesión) y `puerta: "ok" | "falla"`; exit 0 si `ok`, 3 si `falla`. El tope de 1 bloqueo por mensaje humano se aplica agrupando por clave de segmento humana (los `NOHUMANO` heredan la del último humano previo).

**Tests:**
- `test_p2_dispara`: segmento sintético sin Agent cuyo texto acaba en `¿Te parece?` da `dispara=True, patron="P2"`. Falla si PERM no se evalúa sobre `tail`.
- `test_p3_requiere_interrogacion`: texto con `Mi recomendación: X.` y sin `?` en los últimos 700 c ni AskUserQuestion da `dispara=False`; con `AskUserQuestion` en el segmento da `patron="P3"`. Falla si se pierde `ask` (paridad con `patrones.py`).
- `test_agent_en_segmento_no_dispara`: el mismo texto P2 con un `tool_use` `Agent` (y otro con `Task`) en el segmento da `excluido="agent"`. Falla si solo se mira `Agent`.
- `test_etiqueta_solo_al_inicio`: `[reserva: externo] ¿Lo subo?` da `excluido="etiqueta"`; `¿Lo subo? [reserva: externo]` dispara; `[reserva: otra]` dispara. Falla si la etiqueta cuenta en cualquier posición o acepta valores fuera de la lista.
- `test_exclusion_con_limite_de_palabra`: `¿hago push y abro el PR?` da `excluido="reserva"`; `¿Te parece el subordinado?` y `¿Te vale el commitment?` disparan. Falla si la regex no lleva `\b`.
- `test_kinds_informativos`: `¿Seguro?` sin P2/P3 da `dispara=False, kind="p1_log"`; texto sin `?` y sin ningún `tool_use` da `kind="banda"`. Falla si se mezclan con `disparo`.
- `test_segmentacion_paridad`: sobre líneas sintéticas (humano, assistant, hand-back con `isMeta` y `origin.kind="peer"`, task-notification con `promptSource="system"`, tool_result, sidechain), `delegar-decision.es_mensaje_humano` coincide con `transcripts.es_mensaje_humano` línea a línea y `segmentar` da las claves esperadas. Falla si un hand-back no corta segmento.
- `test_puerta_datos_reales` (skip con aviso si falta `$EXO_DELEGAR_DATOS` o `~/.claude/projects`): el CLI devuelve `puerta="ok"` con los rangos de Global Constraints, y la decisión de `evalua` coincide con la de `referencia/patrones.py` (P4) sobre los 30 ítems de `muestra.json` y los 8 de `ocho.json` antes de exclusión. Falla si la paridad o la puerta se rompen.

**Verificación:** `python3 evals/uso-real/backtest_delegar.py` → JSON con `"puerta": "ok"` y exit 0; suite de Cierre de ola verde.

**Review Focus:**
- Ids de `muestra.json`: `<uuid humano>` o `<sesión>#<índice de segmento>` para segmentos `NOHUMANO`/final (ver `referencia/muestra.py`:33-37); el mapeo a segmentos tiene que reproducirlo o la muestra no casa (`test_puerta_datos_reales`).
- `tope` de `sello.json` (`T.cortes`) se respeta: sin él entran sesiones posteriores al sello y `sesiones` ≠ 99.
- `¿`/`ó`/`í` multibyte bajo `(?i)` y `\b` en `re` (Python): tests con acentos en `test_exclusion_con_limite_de_palabra`.
- Etiquetas del dueño `delegar`/`consultor` = positivo; `?` y `mia` = negativo (spec §2).

**Notas:** si la puerta falla, el executor solo puede corregir errores de implementación (segmentación, mapeo de ids, `\b`); no añade términos a `EXCLUSION` ni toca PERM/PROP. Reporta las cifras obtenidas y para. *Decisión del planner.*

### Task 2: hook Stop delegar-decision.sh

**Files:**
- Create: `plugins/exo/scripts/delegar-decision.sh`
- Modify: `plugins/exo/scripts/delegar-decision.py`
- Test: `plugins/exo/scripts/test-delegar-decision.sh`
- Create: `plugins/exo/scripts/testdata/delegar-decision/p2.jsonl`
- Create: `plugins/exo/scripts/testdata/delegar-decision/agent.jsonl`
- Create: `plugins/exo/scripts/testdata/delegar-decision/reserva.jsonl`
- Create: `plugins/exo/scripts/testdata/delegar-decision/banda.jsonl`

**Interfaces:**
- Consumes: `evalua(bloques) -> dict` @Task 1
- Consumes: `segmentar(lineas) -> list[tuple]` @Task 1
- Produces: `python3 -I delegar-decision.py <transcript_path>` → una línea JSON `{"uuid": str|null, "dispara": bool, "kind": str|null, "patron": str|null, "tail": str}` sobre el último segmento (`uuid` = último mensaje humano del transcript); exit 2 si el transcript no se puede leer.
- Produces: `delegar-decision.sh` (stdin = JSON de Stop) → `{"decision":"block","reason":<texto de Global Constraints>}` o nada.

**Tests:**
- `bloquea: p2 sin agent`: `p2.jsonl` da `decision=block` y `reason` idéntico al literal. Falla si el texto se desvía.
- `calla: agent en el segmento`, `calla: reserva al inicio`: `agent.jsonl` y `reserva.jsonl` dan salida vacía. Falla si el hook no delega la decisión al helper.
- `calla: abstenciones`: con `agent_id`, con `stop_hook_active:true`, con `transcript_path` inexistente, con `PATH` sin python3 y sin jq: salida vacía y exit 0. Falla si alguna abstención bloquea o sale con ≠0.
- `tope: un bloqueo por mensaje humano`: dos invocaciones seguidas con el mismo transcript y `session_id` → la primera bloquea, la segunda calla; tras añadir un mensaje humano nuevo con otro P2, vuelve a bloquear. Falla si el sentinel no se lee o no se escribe.
- `sentinel no escribible no rompe`: `~/.cache/exo` como fichero (no directorio) → sigue bloqueando la primera vez y sale 0. Falla si el fallo de escritura aborta.
- `log: kinds`: tras `p2.jsonl` hay una línea `reflex=delegar-decision` con `kind=disparo` en el payload; tras `banda.jsonl`, una `reflex=delegar-decision:banda`; con `REFLEX_LOG_FILE` en un directorio inexistente el hook sigue saliendo 0. Falla si la telemetría rompe el hook.

**Verificación:** `bash plugins/exo/scripts/test-delegar-decision.sh` → todas `[PASS]`, 0 `[FAIL]`; suite de Cierre de ola verde.

**Review Focus:**
- `uuid` del sentinel = último mensaje humano, no el último `NOHUMANO`: un hand-back posterior no reabre el tope (`tope: un bloqueo por mensaje humano`).
- `session_id` con caracteres raros en la ruta del sentinel: sanear a `[A-Za-z0-9_-]` (test con `session_id="../x"`).
- Detección de `agent_id` como en `sdd-wake-guard.sh` (regex sobre el INPUT crudo antes de jq).
- Fixtures sintéticos sin texto real ni rutas personales (`scripts/test-rutas-personales.sh`).

### Task 3: brainstorm «consultor primero, Paul firma»

**Files:**
- Modify: `plugins/exo/skills/brainstorm/SKILL.md`
- Modify: `plugins/exo/skills/brainstorm/consultor-brief.md`
- Modify: `plugins/exo/skills/plan/SKILL.md`

**Interfaces:**
- Produces: n/a (doctrina)

**Tests:**
- n/a — cambio de prosa en skills; lo cubren `scripts/check-skill-refs.sh` y `scripts/check-prosa-cli.sh` del Cierre de ola.

**Verificación:** `grep -n "pregunta cuál prefiere\|no tiene criterio" plugins/exo/skills/brainstorm/SKILL.md plugins/exo/skills/brainstorm/consultor-brief.md plugins/exo/skills/plan/SKILL.md` → sin resultados; suite de Cierre de ola verde.

**Review Focus:**
- La viñeta «Enfoques» (SKILL.md:71-77) despacha siempre un consultor con `consultor-brief.md` antes del mensaje; el mensaje lleva su veredicto, el trade-off de las demás vías y la condición de invalidación; Paul firma o corrige, no elige de un menú.
- La viñeta «Si el usuario no tiene criterio…» (:78-81) se borra y su contenido pasa a la anterior.
- Las preguntas de intención (el qué) siguen siendo de Paul y se marcan `[reserva: hecho-de-Paul]` si casan el detector.
- `consultor-brief.md`:3-4 («que no tiene criterio») y `plan/SKILL.md`:89 («pregunta de diseño sin criterio») se alinean con la regla nueva.

### Task 4: wiring, versión y doctrina

**Files:**
- Modify: `plugins/exo/hooks/hooks.json`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `README.md`
- Modify: `plugins/exo/README.md`
- Modify: `docs/arquitectura.md`

**Interfaces:**
- Consumes: `delegar-decision.sh` @Task 2

**Tests:**
- `scripts/test-hooks-json.sh`: `Stop` contiene `"${CLAUDE_PLUGIN_ROOT}"/scripts/delegar-decision.sh` y el script existe. Falla si la ruta no resuelve.
- `scripts/test-versiones.sh` y `scripts/plugin-bump-gate.sh`: 1.26.0 en ambos manifiestos. Falla si solo sube uno.
- `scripts/test-exec-bit.sh`: `delegar-decision.sh` y `delegar-decision.py` en 100755. Falla si el hook muere con `Permission denied`.

**Verificación:** `BASE=main bash scripts/plugin-bump-gate.sh` → exit 0; suite de Cierre de ola verde.

**Review Focus:**
- Entrada nueva en el array `Stop` existente; sin orden impuesto (corren en paralelo, spec §4.1).
- Las tres docs que listan `sdd-wake-guard` añaden `delegar-decision` con una línea: qué bloquea, la etiqueta `[reserva: …]` y el tope.
- Ningún texto de doctrina sigue anunciando «pregunta cuál prefiere».

### Task 5: prueba manual del doble bloqueo

**Files:**
- Create: `docs/superpowers/plans/2026-10-28-delegar-decision-notas.md`

**Interfaces:**
- Consumes: hooks registrados @Task 4

**Tests:**
- n/a — comportamiento de plataforma no documentado (spec §5, F4); se observa a mano.

**Verificación:** en una sesión con el plugin a 1.26.0, se fuerza a la vez el bloqueo de `delegar-decision` (cierre P2 sin Agent) y el de `document-remind.sh` o `sdd-wake-guard.sh`; se apunta en el fichero de notas qué `reason` recibe el agente (uno, ambos concatenados o el primero) y si cada petición se puede satisfacer en la continuación.

**Review Focus:**
- Si solo llega un `reason`, apuntarlo como riesgo para el kill (bloqueos perdidos) sin cambiar código en este plan.
