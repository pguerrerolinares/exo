# Plan: medición en uso real de correcciones en decisiones ya tomadas

> For agentic workers: ejecución con `exo:orchestrate`.

**Plan:** 110 min, 9 turnos de executor, 27 KB

**Goal:** dejar en main el instrumento de `evals/uso-real/` testeado, sellado y con una llamada de humo por etapa, listo para que el orquestador corra la etapa 1 en un PR aparte.

**Architecture:** cuatro módulos puros con tests `unittest`:
- `transcripts.py`: población y mensajes humanos;
- `puerta.py`: IC de Poisson, puerta y veredicto final;
- `deterministas.py`: violaciones mecánicas de las reglas globales;
- `kb.py`: la KB en la fecha de la sesión.

Encima, `juez.py` (llamada aislada a `claude -p`, lotes y cache) y CLIs finos: `detectar.py`, `muestra.py`, `validar.py`, `atribuir.py`, `sellar.py` y `resultado.py`. El texto real vive en `~/.cache/exo-uso-real/`; al repo solo van código, ids, hashes y conteos.

**Tech Stack:** Python 3 stdlib (`unittest`, `json`, `hashlib`, `random`, `subprocess`, `math`, `unicodedata`), `git`, `exo rules --json` (engine ≥ 0.13.0), `claude` CLI 2.1.310.

**Spec:** `docs/superpowers/specs/2026-10-26-medicion-uso-real-design.md`

**Global Constraints:**
- Directorio: `evals/uso-real/`. Cache privada: `~/.cache/exo-uso-real/` (por defecto; flag `--cache`).
- KB: ruta en `--kb` o `$EXO_KB`; nunca una constante en el repo.
- Población: `~/.claude/projects/*/<sesion>.jsonl` (sin `subagents/`), `entrypoint == "cli"`, primer `timestamp` ≥ `2026-10-06`. Sesión = fichero con ≥ 1 mensaje humano.
- Mensaje humano: `type == "user"`, sin `isMeta`, sin `isCompactSummary`, sin `toolUseResult`. Si existe `origin`, debe ser `origin.kind == "human"`; si existe `promptSource`, debe estar en `{"typed", "queued"}`. El texto no puede ir vacío ni empezar (tras `lstrip`) por `<command-`, `<local-command`, `<task-notification>`, `<bash-stdout>`, `Another Claude session`, `[Request interrupted` o `This session is being continued`.
- Modelos: etapa 1 `claude-haiku-5-5`; etapa 2 `claude-sonnet-5-5`. Invocación aislada idéntica a `evals/rubrica-reglas/etiquetar.py:llama`: `--setting-sources "" --strict-mcp-config --settings '{"autoMemoryEnabled": false}' --tools "" --no-session-persistence --output-format json --max-budget-usd <por_llamada>`, prompt el último, stdin `/dev/null`, cwd temporal vacío y binario `$CLAUDE_BIN` (por defecto `claude`).
- Etapa 1: lotes de **10** mensajes por llamada; por llamada `0.10` USD; tope **3** USD. Salida por mensaje `{"id", "correccion": "si"|"no", "tipo": "rechaza-propuesta"|"contradice-hecho"|"repite-instruccion"|"redirige"|null}`.
- Etapa 2: un positivo por llamada; por llamada `0.50` USD; tope **25** USD. Salida `{"id", "atribucion": "regla-canal"|"regla-fuera-canal"|"canon-kb"|"perfil-core"|"dicho-en-sesion"|"nueva", "nota": str|null, "linea": str|null}`. Decidida = las cuatro primeras.
- Tope alcanzado ⇒ exit **3**; fallo de proceso ⇒ exit **4**; sello que no casa ⇒ exit **2**. Cache por id de mensaje: una segunda corrida no repite llamadas.
- Validación etapa 1: 40 positivos por regex + 40 al azar, semilla `20261026`, orden aleatorio, sin veredicto. Regex (sin distinguir mayúsculas ni tildes): `no uses|otra vez|siempre|nunca|esta mal|dejate de`. Umbral: `kappa ≥ 0.4`.
- Validación etapa 2: 30 positivos, estratificados por `atribucion` (reparto proporcional, mínimo 1 por estrato presente), semilla `20261026`. Sin umbral: se informa el acuerdo.
- Puerta: tasa = correcciones / sesiones, IC 95 % de Poisson exacto. IC superior `< 0.4` ⇒ `"estabilizar"`; si no ⇒ `"etapa2"`. Final: la misma regla con las decididas. Invalidación: `n_correcciones ≥ 30` y `decididas_kb / n_correcciones < 0.10` ⇒ `"unidad_vacia"`.
- Privacidad (repo público): en el repo solo van ids de sesión y de mensaje, hashes, conteos y veredictos. Ni los tests ni los fixtures llevan texto real de transcripts ni de la KB; tampoco rutas del home.
- No se toca `plugins/exo/` (no hay bump de versión).

## Cierre de ola

- `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → verde = `OK`
- `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"` → verde = `OK`
- `bash scripts/test-docs-sin-local.sh && bash scripts/test-rutas-personales.sh` → verde = exit 0

## Mutación

- `evals/uso-real`: manual

## Recursos compartidos

- La cuenta de `claude` (gasto real) y `~/.cache/exo-uso-real/`: solo Task 9.
- La KB del dueño (`$EXO_KB`): solo lectura (`git show`, `git log`), solo Task 9.
- `~/.claude/projects/`: solo lectura, solo Task 9. Los tests usan fixtures sintéticas en un directorio temporal.

## Olas

- Ola 1, en paralelo: T1, T2, T3 y T4 (ficheros disjuntos, sin dependencias).
- Ola 2: T5 (juez y etapa 1), que consume T1.
- Ola 3, en paralelo: T6 (muestras y validación) y T7 (etapa 2), que consumen T1/T2/T4/T5 y tocan ficheros disjuntos.
- Ola 4: T8 (sello y resultado), que consume todo.
- Ola 5: T9 (humo y sello real).

### Task 1: transcripts.py — población y mensajes humanos

**Files:**
- Create: `evals/uso-real/transcripts.py`
- Test: `evals/uso-real/test_transcripts.py`

**Interfaces:**
- Produces: `transcripts.texto(d: dict) -> str` (`message.content` como str, o la unión con `" "` de los bloques `type == "text"`)
- Produces: `transcripts.es_mensaje_humano(d: dict) -> bool`
- Produces: `transcripts.lee_sesion(path: str) -> dict | None`, que devuelve `{"id", "repo_dir", "cwd", "entrypoint", "inicio", "mensajes": [{"id", "ts", "texto", "previo"}], "reglas_canal": str | None}`. `id` = basename sin `.jsonl`; `repo_dir` = nombre del directorio padre; `cwd` = primer `cwd` visto; `inicio` = primer `timestamp`; `previo` = último texto del asistente antes del mensaje, truncado por delante a 6000 caracteres; `id` del mensaje = su `uuid`. `reglas_canal` es el texto del primer `attachment.type == "prompt_snapshot"` desde la línea que empieza por `## Reglas duras` hasta el siguiente `## `, o `None`. Devuelve `None` si el fichero no tiene ninguna línea JSON válida.
- Produces: `transcripts.poblacion(proyectos: str, desde: str = "2026-10-06") -> tuple[list[dict], dict]` (sesiones aceptadas, ordenadas por `id`, y `{"sdk": int, "vacias": int, "reserva": int}`). `reserva` = `cli` con mtime ≥ `desde` pero `inicio` < `desde`.

**Tests:**
- `test_filtra_meta_compact_toolresult`: tres líneas `type=user` con `isMeta: true`, `isCompactSummary: true` y `toolUseResult` dan `False`. Falla si un resumen de compactación entra como corrección.
- `test_prefijos_excluidos`: cada uno de los 7 prefijos de Global Constraints (con espacios delante) da `False`; `"no, usa git -C"` da `True`. Falla si se olvida un prefijo o no se hace `lstrip`.
- `test_suggestion_accepted_fuera`: `origin.kind=human` con `promptSource="suggestion_accepted"` da `False`; con `typed` y con `queued`, `True`; sin campos `origin`/`promptSource` (formato viejo), `True`. Falla si el formato viejo se descarta entero.
- `test_paste_cuenta`: texto con `<pasted_content>` da `True`.
- `test_previo_y_truncado`: el `previo` es el último texto del asistente y no el anterior; con 7000 caracteres se queda en los últimos 6000.
- `test_reglas_canal`: con un snapshot que lleva `## Reglas duras del proyecto\n- a\n## Otra`, devuelve el bloque sin `## Otra`; sin snapshot, `None`.
- `test_poblacion_cortes`: en un directorio temporal, un `sdk-cli`, uno `cli` sin mensajes humanos, uno `cli` con inicio `2026-10-05T23:59:59Z` y mtime posterior, uno en `subagents/` y uno válido dan exactamente 1 sesión y `{"sdk": 1, "vacias": 1, "reserva": 1}`. Falla si se cuentan los subagentes o se filtra por mtime en lugar de por `inicio`.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- Líneas JSON corruptas o truncadas a mitad de fichero (el jsonl vivo de una sesión abierta): se saltan sin abortar. Lo cubre un caso de `test_poblacion_cortes`.
- `content` como lista con bloques `tool_result` además de `text`: solo cuentan los `text`.
- Comparación de `inicio` como cadena ISO: `"2026-10-06"` < `"2026-10-06T00:00:00Z"`. Con `desde` sin hora vale; documéntalo en la firma.

### Task 2: puerta.py — IC de Poisson, puerta y veredicto final

**Files:**
- Create: `evals/uso-real/puerta.py`
- Test: `evals/uso-real/test_puerta.py`

**Interfaces:**
- Produces: `puerta.ic_poisson(k: int, alfa: float = 0.05) -> tuple[float, float]` (IC exacto de la media de recuentos, sin dividir; `k = 0` ⇒ límite inferior `0.0`)
- Produces: `puerta.decide(k: int, n_sesiones: int, umbral: float = 0.4) -> dict`, que devuelve `{"k", "n", "tasa", "ic": [lo, hi], "estado": "estabilizar"|"etapa2"}`
- Produces: `puerta.final(atribuciones: list[str], n_sesiones: int) -> dict`, que devuelve `{"n_correcciones", "decididas", "por_atribucion": dict, "tasa_decididas", "ic", "estado": "estabilizar"|"seguir"|"unidad_vacia"}`
- Produces: `puerta.DECIDIDAS = ("regla-canal", "regla-fuera-canal", "canon-kb", "perfil-core")`

**Tests:**
- `test_ic_valores_referencia`: `ic_poisson(0)` = `(0.0, 3.6889)`, `ic_poisson(10)` = `(4.7954, 18.3904)` e `ic_poisson(40)` = `(28.5766, 54.4686)`, con 3 decimales. Falla si se usa la aproximación normal.
- `test_decide_por_debajo`: `decide(10, 100)` da `estado = "estabilizar"` (IC superior 0,1839).
- `test_decide_cruza`: `decide(40, 100)` da `"etapa2"` (IC 0,2858-0,5447). Falla si la puerta mira la tasa puntual (0,40) en vez del IC superior.
- `test_decide_borde`: `decide(27, 100)` da `"estabilizar"` (IC superior 0,3928) y `decide(28, 100)` da `"etapa2"` (IC superior 0,4047). Falla si la puerta usa `<=`, el IC del 90 % o el límite inferior.
- `test_final_unidad_vacia`: 30 atribuciones con 2 `canon-kb` y 28 `nueva` dan `"unidad_vacia"` (2/30 < 0,10); con 29, la regla no aplica (n < 30). Falla si la invalidación se dispara con n pequeño.
- `test_final_decididas`: 4 decididas sobre 100 sesiones dan `"estabilizar"`; 60 sobre 100 dan `"seguir"`.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- `n_sesiones = 0`: lanza `ValueError`, no divide por cero.
- La comparación con el umbral es `< 0.4` estricta: el IC superior exactamente 0,4 da `"etapa2"`/`"seguir"`.
- Bisección del IC: converge para `k` hasta 2000 sin desbordar `math.exp` (usa `lgamma` en log).

### Task 3: deterministas.py — violaciones mecánicas de reglas globales

**Files:**
- Create: `evals/uso-real/deterministas.py`
- Test: `evals/uso-real/test_deterministas.py`

**Interfaces:**
- Produces: `deterministas.violaciones(comando: str) -> list[str]` (ids de regla: `"cd-git"`, `"git-add-A"`, `"git-stash"`, `"git-checkout-fichero"`)
- Produces: `deterministas.comandos_bash(path: str) -> list[str]` (`input.command` de cada bloque `tool_use` con `name == "Bash"` en los mensajes del asistente)
- Produces: `deterministas.cuenta(ficheros: list[str]) -> dict`, que devuelve `{"llamadas_bash": int, "por_regla": {id: int}, "por_100": {id: float}}`

**Tests:**
- `test_cd_git_positivos`: `cd x && git status`, `cd x; git log` y `cd "a b" &&  git -C . diff` dan `["cd-git"]`. Falla si solo casa `&&`.
- `test_cd_git_negativos`: `git -C x status`, `cd x && make` y `echo cd && gitk` dan `[]`.
- `test_stash_checkout`: `git stash`, `git stash push` y `git stash pop` dan `git-stash`; `git stash list` y `git stash show` dan `[]`. `git checkout -- f` da `git-checkout-fichero`; `git checkout -b r` y `git checkout main` dan `[]`. Falla si una consulta de solo lectura se cuenta como violación.
- `test_add_A`: `git add -A` y `git add --all` dan `git-add-A`; `git add src/a.py` da `[]`.
- `test_cuenta_por_100`: un fixture con 4 Bash y 1 violación da `por_100["cd-git"] == 25.0`.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- Comandos multilínea con heredoc: un `git stash` dentro de un heredoc cuenta igual (son regex de shell, no un parser; la spec lo acepta). El test lo fija para que sea una decisión y no un accidente.
- `cuenta([])` da `llamadas_bash = 0` y `por_100 = {}`, sin dividir por cero.

### Task 4: kb.py — la KB en la fecha de la sesión

**Files:**
- Create: `evals/uso-real/kb.py`
- Test: `evals/uso-real/test_kb.py`

**Interfaces:**
- Produces: `kb.commit_antes(kb: str, ts: str) -> str | None` (último commit con fecha de commit `<` `ts`: `git -C <kb> rev-list -1 --before=<ts> HEAD`, más la comprobación estricta de que la fecha es anterior)
- Produces: `kb.nota_head(cwd: str, kb: str) -> str | None` (campo `note` de `exo rules --json --kb <kb> --cwd <cwd>`; `None` si el binario falla o no hay nota)
- Produces: `kb.ruta_en(kb: str, ruta_head: str, commit: str) -> str | None` (nombre de la ruta en `commit` siguiendo renombres con `git log --follow --name-only`; `None` si no existía)
- Produces: `kb.contexto(kb: str, commit: str, nota: str | None) -> dict`, que devuelve `{"nota": str|None, "reglas_globales": str|None, "core": {ruta: texto}, "nota_ausente": bool}`. Los textos salen de `git show <commit>:<ruta>` y `core` son los `core/*.md` en ese commit.

**Tests:**
- `test_commit_antes`: en un repo git temporal con tres commits fechados (`GIT_COMMITTER_DATE`) a las 10:00, 12:00 y 14:00, `ts` 13:00 devuelve el segundo y `ts` 09:00 devuelve `None`. Falla si se usa la fecha de autor o `<=`.
- `test_ruta_en_renombre`: una nota renombrada en el tercer commit se resuelve con su nombre viejo en el segundo. Falla si `git show` usa la ruta de HEAD.
- `test_nota_ausente`: una nota creada en el tercer commit, pedida en el segundo, da `nota_ausente = True` y `nota = None`.
- `test_contexto_core`: `core/a.md` y `core/reglas-duras.md` del commit pedido salen en `core` y `reglas_globales`. Contenido sintético, no de la KB real.
- `test_nota_head_sin_binario`: con `EXO_BIN` apuntando a un ejecutable que sale con 1, devuelve `None` sin lanzar.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- Zona horaria: `ts` del transcript en UTC (`Z`) frente a commits en `+0100`/`+0200`. Comparar en epoch, nunca como cadena; lo cubre un caso de `test_commit_antes` con offset.
- Rutas con espacios y `—` (las notas reales los llevan): `git show` recibe la ruta como argumento único.
- El binario `exo` sale de `$EXO_BIN` (por defecto `exo`).

### Task 5: juez.py, detectar.py y prompt de la etapa 1

**Files:**
- Create: `evals/uso-real/juez.py`
- Create: `evals/uso-real/detectar.py`
- Create: `evals/uso-real/detector.prompt.md`
- Test: `evals/uso-real/test_juez.py`

**Interfaces:**
- Consumes: `transcripts.poblacion(proyectos, desde)` @Task 1
- Produces: `juez.llama(entrada: str, modelo: str, por_llamada: str) -> tuple[str | None, float]` (texto `result` y `total_cost_usd`; lanza `juez.FalloProceso` si el rc es distinto de 0 o hay timeout de 600 s)
- Produces: `juez.parsea_lote(res: str | None, ids: list[str], valida) -> dict[str, dict]` (solo los ids pedidos con salida válida; acepta el JSON envuelto en un bloque ```json)
- Produces: `juez.Cache(path: str)` con `get(id) -> dict | None` y `put(id, obj)` (jsonl append, la última escritura gana)
- Produces: `detectar.valida(obj: dict) -> dict | None` (esquema de etapa 1 de Global Constraints)
- Produces: CLI `detectar.py --proyectos DIR --cache DIR --salida FICHERO [--sello F] [--tope-usd 3] [--por-llamada-usd 0.10] [--desde 2026-10-06] [--solo-sesion ID]`. `--solo-sesion` restringe la población a una sesión (para el humo) y es incompatible con `--sello`. Escribe en `--salida` (público) `{"sesion", "id", "correccion", "tipo"}` o `{"sesion", "id", "sin_veredicto": true}`; en la cache, el texto. Imprime `coste_total_usd=<x>` y `n_sesiones=<n>`.

**Tests:**
- `test_parsea_lote_falta_id`: un lote de 3 con salida para 2 devuelve 2. Falla si un id ausente se da por `no`.
- `test_parsea_lote_id_ajeno`: un id que no estaba en el lote se descarta.
- `test_lote_reintento_y_sin_veredicto`: con un `CLAUDE_BIN` falso que omite un id dos veces, ese id sale `sin_veredicto` y los otros 9, con veredicto; hay exactamente 2 llamadas. Falla si se reintenta el lote entero indefinidamente.
- `test_cache_no_repite`: dos corridas seguidas con el binario falso; la segunda hace 0 llamadas (contador en un fichero).
- `test_tope_exit_3`: el binario falso reporta 2,0 USD por llamada con `--tope-usd 3` ⇒ exit 3 tras la segunda llamada, la salida parcial queda en `<salida>.parcial` y se imprime `coste_total_usd=4`.
- `test_sello_no_casa_exit_2`: un sello con otro `prompt_detector_sha256` da exit 2 sin llamar.
- `test_salida_publica_sin_texto`: ninguna línea de `--salida` contiene el texto del mensaje del fixture.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- Orden de los lotes determinista (por sesión y luego por `ts`) para que la cache y el sello sean reproducibles.
- Un mensaje cuyo `previo` está vacío (primer mensaje de la sesión) va al juez con `previo` vacío explícito, no se omite.
- El prompt pide el array JSON con exactamente los ids recibidos y define cada `tipo` con un ejemplo sintético. No cita reglas de la KB: el juez detecta correcciones, no atribuye.

**Notas:** `detector.prompt.md` define «corrección»: el dueño rechaza, enmienda o repite algo que el agente hizo o propuso en el turno previo, o lo redirige porque iba mal. No cuentan las peticiones nuevas, las respuestas a preguntas del agente ni las aprobaciones.

### Task 6: muestra.py y validar.py — etiquetado ciego y kappa

**Files:**
- Create: `evals/uso-real/muestra.py`
- Create: `evals/uso-real/validar.py`
- Test: `evals/uso-real/test_validar.py`

**Interfaces:**
- Consumes: `transcripts.poblacion` @Task 1
- Consumes: CLI `detectar.py` y su `--salida` @Task 5
- Consumes: `puerta.DECIDIDAS` @Task 2
- Produces: `muestra.candidato_regex(texto: str) -> bool` (regex de Global Constraints sobre el texto en minúsculas y sin tildes, vía `unicodedata` NFKD)
- Produces: `muestra.etapa1(mensajes: list[dict], semilla: int = 20261026) -> list[dict]` (40 que casan con la regex + 40 al azar del resto, sin repetir, en orden barajado; si casan menos de 40, se toman todos y se completa al azar hasta 80)
- Produces: `muestra.etapa2(atribuidos: list[dict], semilla: int = 20261026, n: int = 30) -> list[dict]` (estratificado según Global Constraints)
- Produces: CLI `muestra.py --etapa 1|2|regex ...`. `--etapa regex` es el plan B de la spec si `kappa < 0.4`: todos los mensajes que casan con la regex, para que el dueño los etiquete a mano. Las etapas 1 y 2 que escribe `~/.cache/exo-uso-real/etiquetar-etapa<k>.jsonl` con `{"id", "previo", "texto", "etiqueta": null}` (etapa 2 añade el contexto de KB usado y `"atribucion_dueño": null`), sin veredicto.
- Produces: `validar.metricas(pred: list[bool], verdad: list[bool]) -> dict` (`kappa` con `estadistica.kappa` de `evals/rubrica-reglas`, `precision`, `recall`, `n`)
- Produces: CLI `validar.py --etapa 1 --etiquetas F --detecciones F --salida F`, que escribe `{"n", "kappa", "precision", "recall", "juez_valido": kappa >= 0.4}` y sale con 0. Si quedan etiquetas `null`, exit 2 con la lista de ids.

**Tests:**
- `test_regex_sin_tildes`: «Déjate de golds» y «ESTÁ MAL» casan; «nunca» casa; «nuncaa» también (es subcadena; es la decisión de la spec y el test la fija); «vale, sigue» no casa.
- `test_muestra_etapa1_reproducible`: dos llamadas con la misma semilla dan el mismo orden; 80 ítems, sin ids repetidos.
- `test_muestra_pocos_regex`: con 25 que casan, salen los 25 y 55 al azar.
- `test_muestra_ciega`: ninguna clave de la muestra de etapa 1 se llama `correccion`, `tipo` ni `veredicto`. Falla si se filtra el juez al etiquetador.
- `test_validar_umbral`: unas etiquetas construidas para kappa 0,4 exacto dan `juez_valido = true`; para 0,39, `false`.
- `test_validar_null_exit_2`: una etiqueta `null` da exit 2.
- `test_estratos_etapa2`: con 60 `nueva`, 30 `canon-kb` y 1 `regla-canal`, salen 30 ítems con al menos 1 `regla-canal`.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- `sin_veredicto` en las detecciones: el ítem se excluye del kappa y se cuenta aparte en la salida (`n_sin_veredicto`).
- La importación de `estadistica.kappa` usa `sys.path` relativo a `__file__`, no al cwd.
- `kappa` puede ser `None` (`pe == 1`): `juez_valido = false`.

### Task 7: atribuir.py y prompt de la etapa 2

**Files:**
- Create: `evals/uso-real/atribuir.py`
- Create: `evals/uso-real/atribuidor.prompt.md`
- Test: `evals/uso-real/test_atribuir.py`

**Interfaces:**
- Consumes: `transcripts.lee_sesion` @Task 1
- Consumes: `kb.commit_antes`, `kb.nota_head`, `kb.ruta_en`, `kb.contexto` @Task 4
- Consumes: `juez.llama`, `juez.Cache`, `juez.FalloProceso` @Task 5
- Produces: `atribuir.valida(obj: dict) -> dict | None` (esquema de etapa 2)
- Produces: `atribuir.construye_entrada(msg: dict, anteriores: list[str], ctx: dict, reglas_canal: str | None, prompt: str) -> str`
- Produces: CLI `atribuir.py --proyectos DIR --detecciones F --kb DIR --cache DIR --salida F [--sello F] [--tope-usd 25] [--por-llamada-usd 0.50]`. Público: `{"sesion", "id", "atribucion", "nota_ausente", "commit_kb"}`; la cache guarda `nota`, `linea` y el texto.

**Tests:**
- `test_solo_positivos`: unas detecciones con 3 `si`, 5 `no` y 1 `sin_veredicto` producen exactamente 3 llamadas.
- `test_reglas_canal_en_entrada`: con `reglas_canal`, la entrada lleva una sección «Reglas en el system prompt»; sin ellas, la sección dice explícitamente «ninguna».
- `test_commit_por_sesion`: dos sesiones con `inicio` distinto usan commits distintos de un repo KB temporal. Falla si se usa HEAD.
- `test_nota_ausente_publica`: una sesión cuya nota no existía en su commit sale con `nota_ausente: true`.
- `test_salida_publica_sin_texto`: `nota`, `linea` y el texto no aparecen en `--salida`.
- `test_tope_exit_3`: la misma mecánica que en T5, con tope 25.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- `anteriores` = los mensajes humanos previos de la misma sesión (para `dicho-en-sesion`), truncados por delante a 8000 caracteres en total.
- `exo rules` se resuelve con el `cwd` de la sesión a HEAD y después `ruta_en` al commit. Si el `cwd` ya no existe en disco, `nota_head` da `None` ⇒ `nota_ausente`.
- El prompt distingue `regla-canal` (estaba en la sección de reglas de esta sesión) de `regla-fuera-canal` (está en `## Reglas duras` de la nota en ese commit, pero no en el system prompt).

### Task 8: sellar.py y resultado.py

**Files:**
- Create: `evals/uso-real/sellar.py`
- Create: `evals/uso-real/resultado.py`
- Create: `evals/uso-real/LANZAR.md`
- Test: `evals/uso-real/test_sellar_resultado.py`

**Interfaces:**
- Consumes: `transcripts.poblacion` @Task 1
- Consumes: `puerta.decide`, `puerta.final` @Task 2
- Consumes: `deterministas.cuenta` @Task 3
- Consumes: CLI `detectar.py` y su `--salida` @Task 5
- Consumes: CLI `validar.py` y su `--salida` @Task 6
- Consumes: CLI `atribuir.py` y su `--salida` @Task 7
- Produces: CLI `sellar.py --proyectos DIR --kb DIR --salida evals/uso-real/sello.json`, que escribe `{"sesiones_sha256", "n_sesiones", "kb_commit", "prompt_detector_sha256", "prompt_atribuidor_sha256", "claude_version", "modelos": {...}, "umbrales": {"puerta": 0.4, "kappa": 0.4, "n_min_invalidacion": 30, "frac_invalidacion": 0.10}, "lote": 10, "semilla": 20261026, "filtro_sha256"}`. `filtro_sha256` = hash del código fuente de `es_mensaje_humano`. Si el sello ya existe, se niega con exit 2 salvo `--forzar`.
- Produces: CLI `resultado.py --detecciones F [--atribuciones F] [--validacion F] --proyectos DIR --sello F --salida evals/uso-real/resultado.json`, que escribe `{"puerta": decide(...), "final": final(...) | null, "deterministas": {"principales": cuenta, "subagentes": cuenta}, "n_sin_veredicto", "n_nota_ausente", "validacion": ..., "sello_ok": bool}`. Si la validación dice `juez_valido: false`, `"puerta"` sale con `"estado": "juez_invalido"`.

**Tests:**
- `test_sello_estable`: dos sellados sobre los mismos fixtures dan el mismo `sesiones_sha256`; añadir una sesión lo cambia.
- `test_sello_no_sobrescribe`: un segundo sellado sin `--forzar` da exit 2.
- `test_resultado_juez_invalido`: una validación con `juez_valido: false` da `"juez_invalido"` aunque la tasa esté por debajo de 0,4. Falla si un juez roto estabiliza el proyecto.
- `test_resultado_sin_etapa2`: con `decide` en `"estabilizar"` y sin atribuciones, `"final": null`.
- `test_resultado_sello_cambiado`: la población cambiada respecto al sello da `"sello_ok": false` y exit 2.

**Verificación:** `python3 -m unittest discover -s evals/uso-real -p "test_*.py"` → `OK`

**Review Focus:**
- `n_sesiones` del denominador sale de la población sellada, no de las sesiones con detecciones. Una sesión cuyos mensajes son todos `sin_veredicto` sigue contando en el denominador.
- `LANZAR.md` lista los comandos en orden (sellar → detectar → muestra 1 → [el dueño etiqueta] → validar → resultado → [si etapa2] atribuir → muestra 2 → resultado), con `$EXO_KB` y sin rutas del home. Añade dos ramas: si `juez_invalido`, `muestra.py --etapa regex`; si la etapa 1 da < 30 correcciones, se re-sella con `--desde` anterior para incluir la reserva y el resultado lo declara.

### Task 9: humo y sello real

**Files:**
- Create: `evals/uso-real/sello.json`
- Create: `evals/uso-real/humo.md`

**Interfaces:**
- Consumes: todos los CLIs @Task 8

**Tests:** n/a — corrida real contra la cuenta y los transcripts; lo verificable es el sello y el coste anotado.

**Verificación:**
- `python3 evals/uso-real/sellar.py --proyectos ~/.claude/projects --kb "$EXO_KB" --salida evals/uso-real/sello.json` → exit 0, con `n_sesiones` entre 100 y 115 (106 al escribir la spec).
- Humo de la etapa 1: un solo lote real con `--tope-usd 0.15 --solo-sesion ID` sobre una sesión con ≥ 10 mensajes humanos. Se mira que los 10 ids vuelven con un esquema válido. Coste y rc van a `humo.md`.
- Humo de la etapa 2: una llamada real de `atribuir.py` sobre una detección `si` sintetizada en la cache. Se mira que el esquema es válido y `commit_kb` no es HEAD. Coste a `humo.md`.
- Suite entera (Cierre de ola) en verde.

**Review Focus:**
- `humo.md` y `sello.json` no llevan texto de mensajes ni rutas del home (`test-docs-sin-local` no cubre `evals/`: compruébalo con `grep -n "/home/\|wisdom" evals/uso-real/*.md evals/uso-real/*.json` → sin salida).
- El sello se toma **después** del humo si el humo obliga a tocar un prompt (el sello lleva su hash).
