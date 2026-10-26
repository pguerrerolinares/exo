# Plan: rúbrica de selección de reglas duras — entrega 1 (#207)

> For agentic workers: ejecución con `exo:orchestrate`.

**Plan:** 90 min, 7 turnos de executor, 21 KB

**Goal:** dejar en main la doctrina de la rúbrica y todo el instrumento de `evals/rubrica-reglas/` (etiquetador, sello, validación y orden) testeado y sellado, listo para correr la validación en un PR aparte.

**Architecture:** dos módulos puros (`rubrica.py`: orden, validación de etiquetas y recorte de la nota; `estadistica.py`: kappa, regla de decisión y bootstrap) con tests `unittest`, y tres CLIs finos encima (`etiquetar.py`, `sellar.py`, `validar.py`). `siembra-reglas.py` expone su filtro de candidatas y gana `--etiquetas`. La doctrina va a los skills `document` y `distill`. Al final, una llamada de humo y el sello.

**Tech Stack:** Python 3 stdlib (`unittest`, `json`, `hashlib`, `random`, `subprocess`), `claude` CLI 2.1.310, markdown de skills.

**Spec:** `docs/superpowers/specs/2026-10-26-rubrica-reglas-duras-design.md`

**Global Constraints:**
- Modelo del etiquetador: `claude-sonnet-5-5`. Invocación: `claude -p --model claude-sonnet-5-5 --setting-sources "" --strict-mcp-config --settings '{"autoMemoryEnabled": false}' --tools "" --no-session-persistence --output-format json --max-budget-usd <por_llamada> <prompt>`, con stdin `/dev/null` y cwd en un directorio temporal vacío. El binario sale de `$CLAUDE_BIN` (por defecto `claude`).
- Etiqueta: `{"id", "choque": "P"|"D"|"E", "coste": "alto"|"medio"|"bajo", "justificacion", "ruta": <str|null>}`. `ruta` es opcional para cualquier `choque`.
- Orden de la rúbrica (spec §4): veto > choque `P` > `D` > `E` > coste `alto` > `medio` > `bajo` > orden de fichero. Criba `cumple` → `E`; criba `no_cumple` → la etiqueta si es `P`/`D`, si no `D`. `sin_etiqueta` va detrás de todas las etiquetadas.
- Validación: predicción `P`/`D` → se queda, `E` → sale; verdad `no_cumple` → se queda. Regla de decisión: kappa puntual de (b) ≤ 0, o no definido → `"criterio_2": false`. Más de 3 `sin_etiqueta` → validación `"nula"`.
- Bootstrap descriptivo: 10.000 remuestreos, semilla `20261026`, IC95 por percentiles.
- Tope de coste: 5 USD en total por corrida de `etiquetar.py`; por llamada, `--max-budget-usd 0.10`.
- Privacidad (repo público): en el repo solo van ids, `choque`, `coste`, booleanos, sello y agregados. El texto de reglas, las justificaciones, `ruta` y las notas van a `~/.cache/exo-rubrica-reglas/`. Ni los tests ni los fixtures llevan texto real de la KB.
- `CAP` = 10 (sin cambios).

## Cierre de ola

- `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → verde = `OK`
- `bash scripts/check-skill-refs.sh && bash scripts/check-prosa-cli.sh` → verde = exit 0
- `for t in plugins/exo/scripts/test-*.sh; do bash "$t" </dev/null >/dev/null 2>&1 || echo "FALLA $t"; done` → verde = ninguna línea `FALLA`

## Mutación

- `evals/rubrica-reglas`: manual
- `scripts`: manual

## Recursos compartidos

- `~/.cache/exo-rubrica-reglas/` y la cuenta de `claude` (gasto real): solo Task 7.
- La KB del dueño (ruta en `$EXO_KB`): solo lectura (`git show`), solo Task 7.
- `~/.cache/exo-campana-m/` (pool y criba): solo lectura. Ninguna tarea abre `sondas*`.

## Olas

- Ola 1, en paralelo: T1, T2 y T6 (ficheros disjuntos, sin dependencias).
- Ola 2, en paralelo: T3, T4 y T5 (consumen T1/T2 y tocan ficheros disjuntos).
- Ola 3: T7 (humo y sello), que consume T3.

### Task 1: rubrica.py — orden, etiquetas y recorte de la nota

**Files:**
- Create: `evals/rubrica-reglas/rubrica.py`
- Modify: `scripts/siembra-reglas.py`
- Test: `evals/rubrica-reglas/test_rubrica.py`

**Interfaces:**
- Produces: `rubrica.valida_etiqueta(obj: dict) -> dict | None` (normalizada o `None` si es inválida). Obligatorios: `id`, `choque`, `coste`. Opcionales: `justificacion`, `ruta`, `cita_recortada`. Así acepta tanto la respuesta del modelo como el JSONL público de `--salida` de @Task 3.
- Produces: `rubrica.choque_efectivo(etiqueta: dict | None, criba: bool | None) -> str | None` (`criba=True` = A0 cumple)
- Produces: `rubrica.clave(id: str, etiqueta: dict | None, criba: bool | None, vetada: bool) -> tuple`
- Produces: `rubrica.orden(items: list[dict], vetadas: frozenset[str] = frozenset()) -> list[dict]` (cada item: `{"id", "etiqueta": dict|None, "criba": bool|None}`)
- Produces: `rubrica.recorta_nota(texto: str, cita: str) -> tuple[str, bool]` (texto sin `## Reglas duras` ni el fragmento con la cita; `bool` = cita encontrada)
- Produces: `siembra-reglas.es_candidata(r: dict) -> bool` (el filtro de `propuesta()`: `generica` falso, `ambito` en `AMBITOS`, proyecto ≠ `transversal`; `propuesta()` pasa a usarlo)

**Tests:**
- `test_valida_rechaza_fuera_de_dominio`: `choque: "X"`, `coste: "altísimo"`, falta `id` o JSON no dict → `None`. Falla si un valor fuera de dominio pasa.
- `test_valida_ruta_opcional`: `choque: "E"` sin `ruta` o con `ruta: null` → válida. Falla si `E` exige `ruta` (spec §5.2).
- `test_valida_formato_publico`: `{"id", "choque", "coste", "cita_recortada": true}`, sin `justificacion` → válida. Falla si el JSONL público de @Task 3 queda entero como `sin_etiqueta` al leerlo en @Task 5.
- `test_criba_sustituye_juicio`: `choque_efectivo({"choque": "P"}, True) == "E"`; `({"choque": "E"}, False) == "D"`; `({"choque": "P"}, False) == "P"`; `(None, False) == "D"`. Falla si el dato no manda.
- `test_veto_gana_a_todo`: una `E` de coste `bajo` vetada va antes que una `P` de coste `alto`. Falla si el veto no es la primera clave.
- `test_choque_pesa_mas_que_coste`: `P`+`bajo` antes que `D`+`alto`. Falla si el coste se evalúa antes que el choque.
- `test_sin_etiqueta_al_final`: un item con `etiqueta: None` y sin criba va detrás de una `E`+`bajo`. Falla si cae en medio.
- `test_empate_por_orden_de_id`: `g0-9` antes que `g0-10` y que `g1-1`, con etiquetas iguales. Falla si se ordena como texto (`g0-10` < `g0-9`).
- `test_orden_no_depende_de_la_entrada`: la misma lista barajada con 5 semillas da el mismo orden. Falla si el sort no es total.
- `test_recorta_seccion_reglas_duras`: el texto de `## Reglas duras` hasta el siguiente `## ` desaparece y el resto queda. Falla si se come la sección siguiente o deja viñetas.
- `test_recorta_linea_con_cita`: con la cita en una línea (espacios y mayúsculas distintos), esa línea desaparece y el bool es `True`. Con la cita partida entre dos líneas del mismo párrafo, desaparece el párrafo. Falla si la cita sobrevive.
- `test_cita_no_encontrada`: con la cita ausente, el texto solo pierde la sección y el bool es `False`. Falla si recorta algo más.
- `test_es_candidata`: casos sintéticos de `generica`, `ambito` y `transversal`; `propuesta()` sobre un pool sintético devuelve lo mismo que antes del refactor. Falla si el filtro diverge.

**Verificación:** `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → `OK`; `python3 scripts/siembra-reglas.py` sin argumentos sigue imprimiendo el docstring.

**Review Focus:**
- La comparación de la cita se hace sobre texto normalizado (espacios colapsados, minúsculas) y con los primeros 60 caracteres, como la medición del 41/53: `test_recorta_linea_con_cita`.
- Una nota sin `## Reglas duras` no rompe nada: un caso dentro de `test_cita_no_encontrada`.
- Ids `g<n>-<m>` con `int` en la parte numérica: `test_empate_por_orden_de_id`.

**Notas:** «Párrafo» = bloque entre líneas en blanco. Si la cita aparece en una sola línea, se quita esa línea; si no, el párrafo cuyo texto normalizado la contiene. Los fixtures son texto inventado.

### Task 2: estadistica.py — kappa, decisión y bootstrap

**Files:**
- Create: `evals/rubrica-reglas/estadistica.py`
- Test: `evals/rubrica-reglas/test_estadistica.py`

**Interfaces:**
- Produces: `estadistica.kappa(pred: list[bool], verdad: list[bool]) -> float | None` (`None` si pe = 1)
- Produces: `estadistica.decide(kappa_b: float | None, n_sin_etiqueta: int) -> dict` → `{"estado": "nula"|"valida", "criterio_2": bool, "rubrica": str}`
- Produces: `estadistica.bootstrap_delta(pred_b: list[bool], pred_base: list[bool], verdad: list[bool], n: int = 10000, semilla: int = 20261026) -> dict` → `{"delta": float|None, "ic95": [lo, hi], "descartados": int}`

**Tests:**
- `test_kappa_perfecta`: `pred == verdad` con las dos clases → `1.0`. Falla si la fórmula está mal.
- `test_kappa_degenerada`: `verdad` toda `True` y `pred` toda `True` → `None`, sin `ZeroDivisionError`. Falla si divide por cero.
- `test_kappa_conocida`: un caso a mano (p. ej. 10 reglas con po = 0,7 y pe = 0,5 → 0,4). Falla si se usan los marginales equivocados.
- `test_decide_kappa_cero_cae`: `decide(0.0, 0)` y `decide(None, 0)` → `criterio_2: false`, `rubrica: "veto + dato + coste + fichero"`. `decide(0.01, 0)` → `criterio_2: true`, `rubrica: "D"`. Falla si el umbral es `< 0` en vez de `≤ 0`.
- `test_decide_nula`: `decide(0.5, 4)` → `estado: "nula"`; `decide(0.5, 3)` → `"valida"`. Falla si el corte de más de 3 está desplazado.
- `test_bootstrap_identicos_contiene_cero`: `pred_b == pred_base` → `ic95 == [0.0, 0.0]` y `delta == 0`. Falla si el remuestreo no es pareado.
- `test_bootstrap_reproducible`: dos llamadas con la misma semilla dan el mismo `ic95` al bit. Falla si se usa el RNG global.
- `test_bootstrap_cuenta_descartados`: con una verdad que en muchos remuestreos queda de una sola clase (n = 4), `descartados > 0` y no lanza excepción. Falla si los degenerados se cuentan como 0 o rompen.

**Verificación:** `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → `OK`

**Review Focus:**
- `random.Random(semilla)` local, nunca `random.seed`: `test_bootstrap_reproducible`.
- Remuestreo pareado: los mismos índices en los dos predictores: `test_bootstrap_identicos_contiene_cero`.
- `≤ 0` y no `< 0`: `test_decide_kappa_cero_cae`.

### Task 3: etiquetar.py y sellar.py — el etiquetador sellado

**Files:**
- Create: `evals/rubrica-reglas/etiquetador.prompt.md`
- Create: `evals/rubrica-reglas/etiquetar.py`
- Create: `evals/rubrica-reglas/sellar.py`
- Test: `evals/rubrica-reglas/test_etiquetar.py`

**Interfaces:**
- Consumes: `rubrica.valida_etiqueta`, `rubrica.recorta_nota` @Task 1
- Consumes: `siembra-reglas.es_candidata`, `siembra-reglas.REPO` @Task 1
- Produces: `etiquetar.py --pool <dir> --kb <dir> --commit <sha> --ids <txt> --salida <jsonl> --cache <dir> [--sello <json>] [--tope-usd 5] [--por-llamada-usd 0.10]`. Sale 0 si termina, 2 si el sello no casa y 3 si se alcanza el tope. El sello se compara solo en `prompt_sha256`, `claude_version` y `kb_commit` (contra `--commit`); `--ids` puede ser un subconjunto del universo. `--por-llamada-usd` viaja tal cual como cadena al argv (`0.10`, no `0.1`).
- Produces: `sellar.py --pool <dir> --criba <dir> --prompt <md> --kb <dir> --commit <sha> --salida-dir <dir>` → escribe `universo.txt` (un id por línea, ordenado) y `sello.json` (`{"prompt_sha256", "kb_commit", "claude_version", "universo_sha256", "n"}`)
- Produces: `etiquetar.construye_entrada(regla: dict, nota_recortada: str, prompt: str) -> str` y `etiquetar.parsea_salida(stdout: str) -> tuple[dict | None, float]` (etiqueta y `total_cost_usd`)

**Tests:** (con un `claude` falso por `$CLAUDE_BIN`: un script que vuelca argv, cwd y prompt a un fichero y responde un JSON fijo)
- `test_flags_de_aislamiento`: el prompt es el último elemento de argv y no va justo detrás de `--tools ""` (el flag es variádico). Además, argv contiene, en pares, `--setting-sources ""`, `--tools ""`, `--strict-mcp-config`, `--no-session-persistence`, `--output-format json`, `--model claude-sonnet-5-5`, `--settings {"autoMemoryEnabled": false}` y `--max-budget-usd 0.10`; el cwd es un directorio vacío que no es el del repo. Falla si falta un flag o hereda el cwd.
- `test_prompt_sin_seccion_ni_cita`: con una nota sintética que tiene `## Reglas duras` y la cita en la prosa, el prompt enviado contiene la regla y la prosa restante, pero ni el texto de la sección ni la cita. Falla si hay fuga.
- `test_salida_publica_sin_texto`: el JSONL de `--salida` solo tiene las claves `id`, `choque`, `coste` y `cita_recortada` (o `id`, `sin_etiqueta`); la justificación y `ruta` van al JSONL de `--cache`. Falla si el texto privado llega a la salida pública.
- `test_reintenta_una_vez`: si el falso responde basura y luego una etiqueta válida, hay 2 llamadas y la regla queda etiquetada; con basura dos veces, `{"id", "sin_etiqueta": true}` y 2 llamadas. Falla si reintenta más de una vez o ninguna.
- `test_tope_para_sin_completar`: con `total_cost_usd` 3,0 por llamada y `--tope-usd 5`, el proceso sale con 3 tras la segunda llamada, deja `<salida>.parcial` y no crea `<salida>`. Falla si un resultado parcial aparece con el nombre final.
- `test_sello_no_casa`: un `--sello` con otro `prompt_sha256`, otra `claude_version` (el falso responde a `--version`) u otro `kb_commit` → exit 2 sin llamar al modelo; con el sello correcto y `--ids` de 2 ids → exit 0. Falla si etiqueta con un prompt distinto del sellado o si rechaza un subconjunto del universo.
- `test_sellar_universo`: con un pool sintético y una criba sintética, `universo.txt` contiene solo las candidatas con criba cuyo proyecto tiene repo en `REPO` (no `None`), ordenadas por id. `sello.json` lleva `n` igual al número de líneas y `universo_sha256` igual al sha256 de `universo.txt`. Falla si entra una regla sin criba, transversal o con `REPO=None`.

**Verificación:** `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → `OK`

**Review Focus:**
- La nota se lee con `git -C <kb> show <commit>:projects/<nota>`, nunca del working tree: un caso dentro de `test_prompt_sin_seccion_ni_cita`, con un repo git temporal en el que el working tree difiere del commit. Los commits del test usan `git -c user.name=t -c user.email=t@t.local`, porque el runner de CI no tiene identidad.
- Ids de proyectos con `REPO=None`: el etiquetador sale con error claro y no los manda al modelo, cubierto por `test_sellar_universo`.
- `cita_recortada` (bool) va a la salida pública para que `validar.py` cuente las citas no encontradas, cubierto por `test_salida_publica_sin_texto`.

**Notas:**
- El prompt (`etiquetador.prompt.md`) lleva dos huecos literales, `{REGLA}` y `{NOTA}`. Pide una sola línea JSON con el esquema de las Global Constraints, sin bloque de código, y define `P`, `D`, `E` y los tres niveles de coste con las palabras de la spec §4. No lleva ejemplos sacados de la KB.
- `--ids` en la entrega 1 son ids del pool. Las líneas `kb:<slug>:<n>` del re-corte (spec §5.4.2) quedan fuera de este plan: van con el plan del re-corte.

### Task 4: validar.py — el veredicto de la validación

**Files:**
- Create: `evals/rubrica-reglas/validar.py`
- Test: `evals/rubrica-reglas/test_validar.py`

**Interfaces:**
- Consumes: `estadistica.kappa`, `estadistica.decide`, `estadistica.bootstrap_delta` @Task 2
- Produces: `validar.py --etiquetas <jsonl> --universo <txt> --pool <dir> --criba <dir> --salida <json>`, que escribe `resultado.json`: `{"n_universo", "n_analizado", "sin_etiqueta", "citas_no_encontradas", "kappa_b", "kappa_base", "precision_b", "precision_base", "confusion_b", "no_cumple_por_choque", "delta", "decision"}`. `no_cumple_por_choque` = por cada `P`/`D`/`E`, `{"n", "no_cumple", "proporcion"}`. Kappa por grupo no sirve: dentro de un grupo la predicción es de una sola clase.
- Produces: `validar.calcula(etiquetas: dict[str, dict], universo: list[str], ambito: dict[str, str], cumple: dict[str, bool]) -> dict`

**Tests:**
- `test_baseline_git_codigo`: con `ambito` `git`/`codigo` → se queda y `dominio-proyecto` → sale, `kappa_base` coincide con `estadistica.kappa` aplicado a mano. Falla si el baseline usa la criba o el `check`.
- `test_sin_etiqueta_excluida_de_ambos`: 2 reglas `sin_etiqueta` → `n_analizado == n_universo - 2` para `kappa_b` y para `kappa_base`, y `sin_etiqueta == 2`. Falla si se cuentan como «se queda» o solo se excluyen de un brazo.
- `test_decision_cableada`: con etiquetas que dan `kappa_b ≤ 0` → `decision.criterio_2 == false`; con 4 `sin_etiqueta` → `decision.estado == "nula"`. Falla si `validar` decide por su cuenta en vez de llamar a `decide`.
- `test_id_fuera_de_universo`: una etiqueta con un id que no está en `universo.txt` se ignora y se cuenta en un aviso a stderr, y un id del universo sin etiqueta cuenta como `sin_etiqueta`. Falla si el universo sellado no manda.
- `test_resultado_sin_texto`: `resultado.json` no contiene ninguna cadena del campo `regla` ni de `cita` del pool sintético. Falla si filtra texto privado.

**Verificación:** `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → `OK`

**Review Focus:**
- La verdad se lee de `corridas-criba/<id>/a0-r*/check.rc` con la definición de la spec: `cumple` = todas las réplicas dan 0. Cubierto por `test_baseline_git_codigo` con una criba sintética.
- `no_cumple_por_choque` con un grupo vacío da `n: 0` y `proporcion: null`, sin excepción. Es un caso de `test_decision_cableada`.

### Task 5: siembra-reglas.py --etiquetas

**Files:**
- Modify: `scripts/siembra-reglas.py`
- Test: `evals/rubrica-reglas/test_siembra.py`

**Interfaces:**
- Consumes: `rubrica.orden` @Task 1
- Consumes: `siembra-reglas.es_candidata` @Task 1
- Produces: `siembra-reglas.py <pool> <criba> <kb> --etiquetas <jsonl> [--json]`: por repo, las candidatas en el orden de la rúbrica, con `entra` para las 10 primeras. `--etiquetas` lee el formato de `--salida` de @Task 3 (validado con `rubrica.valida_etiqueta`). Sin `--etiquetas`, el orden y la salida de hoy no cambian.

**Tests:**
- `test_sin_etiquetas_no_cambia`: la salida `--json` sobre un pool, una criba y una KB sintéticos (tmpdir con `core/` y `projects/`, porque `tier()` lee la nota de origen) es idéntica al fixture. El fixture se genera corriendo el script de HEAD **antes** de tocarlo y se guarda en el test. Falla si se rompe la siembra vieja.
- `test_con_etiquetas_ordena_por_rubrica`: en un repo con 12 candidatas sintéticas, `entra` marca las 10 primeras de `rubrica.orden`, y una `cumple` etiquetada `P` queda por detrás de una `no_cumple` `D`. Falla si se mezclan las claves viejas (check, ámbito, tier) con la rúbrica.
- `test_etiqueta_ausente`: una candidata sin línea en el JSONL va al final de su repo como `sin_etiqueta`. Falla si el script lanza `KeyError`.

**Verificación:** `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → `OK`; `python3 -I scripts/siembra-reglas.py ~/.cache/exo-campana-m/redaccion ~/.cache/exo-campana-m/corridas-criba "$EXO_KB" --json | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["repos"]))'` → el mismo número que en main.

**Review Focus:**
- La criba que entra en `rubrica.orden` es tri-estado (`True`, `False` o `None` sin corrida), no el booleano `a0_cumple` de hoy, que confunde «sin corrida» con «no cumple». Cubierto por `test_con_etiquetas_ordena_por_rubrica`.

### Task 6: doctrina, CI y versión del plugin

**Files:**
- Modify: `plugins/exo/skills/document/routing.md`
- Modify: `plugins/exo/skills/distill/consolidacion.md`
- Modify: `plugins/exo/skills/distill/chequeos.md`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Produces: paso de CI `Rúbrica de reglas duras (suite Python, offline)` con `run: python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` e `if: always()`, junto al de `retrieval-heldout`.
- Produces: plugin `1.25.0` en `plugin.json` y `marketplace.json`.

**Tests:** n/a — es doctrina y wiring. Lo verifican `check-skill-refs`, `check-prosa-cli`, el gate de versión del plugin en CI y el paso nuevo, que corre las suites de T1-T5.

**Verificación:** `EXO_BIN="$(command -v exo)" bash scripts/check-skill-refs.sh && EXO_BIN="$(command -v exo)" bash scripts/check-prosa-cli.sh` → exit 0 (el worktree no tiene `engine/target`); `bash scripts/test-docs-sin-local.sh` → sin líneas listadas; `grep -c '"version": "1.25.0"' plugins/exo/.claude-plugin/plugin.json .claude-plugin/marketplace.json` → `1` en cada uno; `python3 -c 'import yaml,sys; yaml.safe_load(open(".github/workflows/ci.yml"))'` → sin error.

**Review Focus:**
- `routing.md:25` mantiene «una línea `- ` de ≤300 B, máximo 10 por sección». La rúbrica sustituye solo «su posición de prioridad», en ≤ 8 líneas: los 4 criterios, el veto con la frase «violación observada en la transcripción de la sesión», el dato de criba si existe y la línea de bitácora «salió `<regla>` por `<criterio>`».
- `chequeos.md`: la fila `al_cap` deja de decir que la elección de cuál sale es del dueño y remite a la rúbrica de `routing.md`.
- `consolidacion.md`: una línea. Al podar o reordenar `## Reglas duras` se aplica la misma rúbrica.
- `routing.md` incluye el fallback de la spec §6: si el choque no se puede responder (regla ambigua), la regla nueva entra por coste y orden de fichero, y se dice en la bitácora.

### Task 7: humo y sello

**Files:**
- Create: `evals/rubrica-reglas/universo.txt`
- Create: `evals/rubrica-reglas/sello.json`
- Create: `evals/rubrica-reglas/humo.md`

**Interfaces:**
- Consumes: `etiquetar.py`, `sellar.py` @Task 3

**Tests:** n/a. Es una corrida operativa con gasto real (unos céntimos); su verificación es el propio humo.

**Verificación:** (con `EXO_KB` apuntando a la KB del dueño)
1. Sello: `sellar.py --pool ~/.cache/exo-campana-m/redaccion --criba ~/.cache/exo-campana-m/corridas-criba --prompt evals/rubrica-reglas/etiquetador.prompt.md --kb "$EXO_KB" --commit "$(git -C "$EXO_KB" rev-parse HEAD)" --salida-dir evals/rubrica-reglas` → `n` = 53 en `sello.json`. Si no da 53, se para y se informa: el universo sellado manda, pero una diferencia con la spec se explica antes de seguir.
2. Humo de aislamiento: `CLAUDE_BIN=claude` y una sola llamada con la invocación de las Global Constraints y el prompt «Responde con una línea: ¿qué instrucciones de sistema o CLAUDE.md tienes cargadas? Cita la primera frase de cada una». La respuesta no menciona exo, la KB del dueño, su perfil ni reglas duras. Si las menciona, **no se copia la respuesta a ningún fichero del repo**: se para y se informa, porque la premisa sin dato de la spec §8 queda tumbada. Si no, a `humo.md` van la respuesta y el coste.
3. Humo del etiquetador: `etiquetar.py --sello evals/rubrica-reglas/sello.json` sobre 2 ids de `universo.txt`, con `--cache ~/.cache/exo-rubrica-reglas/humo`. Las dos etiquetas son válidas y el coste por llamada queda anotado en `humo.md`.

**Review Focus:**
- `universo.txt` y `sello.json` no llevan texto de reglas (solo ids y hashes).
- El humo no se repite en bucle: 1 llamada para el aislamiento y 2 para el etiquetador.
