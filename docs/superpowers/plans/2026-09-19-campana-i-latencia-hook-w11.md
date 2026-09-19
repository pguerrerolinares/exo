# Campaña I — Latencia del hook en W11: menos spawns, misma salida

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no lo
> edita). La Task 7 es **PAUL-STEP**: ningún ejecutor automático la corre — la
> fábrica prepara el comando y el fichero de resultados, y la campaña se
> considera "implementación completa" tras la Task 6, con la Task 8 (sync de
> backlog) esperando a que Paul entregue el resultado de la Task 7.

**Goal:** que el hook de cada prompt (`recall-inject.sh`) y de cada arranque
de sesión (`exo-recall.sh`) paguen el motor (el binario `exo`, ≈1,1 s en W11)
y poco más — no ≈1,2 s adicionales de spawns de Git Bash — sin cambiar un
solo byte del bloque que se inyecta en el contexto del agente.

**Architecture:** ocho tareas. La Task 1 instrumenta `recall-inject.sh` con
`hook_ms` (reloj de pared del hook entero, vía `$EPOCHREALTIME`, sin spawn) y
hace que `recall-latencia.sh` decida la reapertura del daemon con ese número
en vez de con el tiempo interno del engine — es la enmienda de la decisión
#12 de Paul al pre-registro de la campaña A. Las Tasks 2 y 3 son el cuerpo de
la campaña: reescriben el parseo de `recall-inject.sh` y `exo-recall.sh`
respectivamente para sustituir invocaciones externas (`sed`, `tr`, jq
redundante) por expansión de parámetros de bash puro, con un golden de
equivalencia byte a byte capturado ANTES de tocar cada script. La Task 4 hace
lo mismo en los tres guards de `PreToolUse:Bash` con un pre-filtro bash que
evita invocar `jq` cuando el comando no contiene "git". La Task 5 (opcional,
con criterio numérico) funde esos tres guards en un solo script si el ahorro
de la Task 4 no basta. Las Tasks 6 y 7 leen el instrumento que ya existe
(`evals/recall-coste/harness/bench.sh` en Linux, el bloque manual del
pre-registro de A en W11) — la 6 la corre la fábrica, la 7 la corre Paul en
su máquina. La Task 8 cierra sincronizando `docs/backlog.md` con los commits
reales y los números medidos.

**Tech Stack:** bash puro (≥3.2, con fallback expreso donde bash ≥5 no está
garantizado) + `jq` 1.7. Sin Rust: ninguna task de esta campaña toca
`engine/src`. Sin Go, sin Python. Harness de medición ya existente:
`hyperfine` (Linux, vía `evals/recall-coste/harness/bench.sh`).

## Verificación de la propuesta contra el código (2026-09-19, este worktree)

La propuesta de origen
(`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §«I —
Latencia del hook en W11», esbozo de 6 tasks) se releyó línea a línea contra
`HEAD` (`5efe812`). Correcciones encontradas:

- **Las líneas de backlog citadas están desfasadas** (el propio encargo lo
  anticipaba). `backlog:568-594` → el ítem real («El coste del hook completo
  en Windows no está medido») vive hoy en `docs/backlog.md:736-761`.
  `backlog:968-1000` → el ítem real («Proceso residente para el coste fijo
  del recall por prompt») vive hoy en `docs/backlog.md:1237-1266`. Ambos
  localizados por título, no por línea, como pedía el encargo.
- **`hooks.json:14-28` es casi exacto**: el bloque completo del matcher
  `"Bash"` de `plugins/exo/hooks/hooks.json` es `:13-29` (`{` de apertura en
  13, `"matcher": "Bash"` en 14, el array `"hooks"` con los tres comandos en
  15-28, cierres en 28-29). La cita de la propuesta señalaba bien el array de
  hooks; se usa `:13-29` en este plan para referirse al bloque entero.
- **«14 jq en `recall-inject.sh`» es un artefacto de conteo, no el número
  real de invocaciones.** `grep -c "jq " plugins/exo/scripts/recall-inject.sh`
  da en efecto 14 hoy, pero esa cifra incluye 7 líneas de **comentario** que
  mencionan "jq" en prosa. Las invocaciones reales de `jq` (pipe o
  sustitución de comando) son **7**: líneas 31 (`PROMPT`), 192 (chequeo de
  envelope), 202 (`META`), 241 (`EXO_KB_NAME`), 337 (`BLOQUE`), 355
  (`PERMALINKS`), 364 (`JSON_OUT`). El ítem de backlog en `:736-761` da un
  número más preciso — «7 `jq`, 2 `sed`, 4 `tr` (re-contado el 2026-09-11)»
  — pero **también está desfasado**: verificado hoy, `recall-inject.sh` tiene
  7 `jq` + 2 `sed` (líneas 70 y 76, dentro de `norm_token()`) + **5** `tr`
  (líneas 57, 75, 159, 181, 250 — no 4). El quinto `tr` (`AVISOS_ERR`, línea
  159) lo añadió el commit `8f41e99` («campaña A, H2/H3», 2026-09-13), **dos
  días después** del recuento de backlog citado. Total real: **14
  invocaciones externas de `jq`/`sed`/`tr`** en el fichero — que
  coincidencialmente iguala al número que la propuesta atribuía solo a
  `jq`, pero por una razón distinta a la que la propuesta daba.
- **El hallazgo que ni la propuesta ni el backlog capturan**: `norm_token()`
  (invocada dentro del bucle léxico `gate_skip()`, una vez por cada token del
  prompt hasta que aparece la primera palabra sustantiva) gasta **2 `sed` +
  1 `tr` POR TOKEN**, no una vez por invocación del hook. Medido con
  `strace -f -c -e trace=execve` sobre un stub y el prompt
  «como funciona el trinquete...» (el mismo usado en el pre-registro de A):
  **31 `execve` reales** para un prompt donde el gate rompe al segundo token
  (`como` es stopword, `funciona` no) — de esos, 6 son sed/tr del gate. Con
  un prompt donde las primeras N palabras son todas stopwords, el
  multiplicador crece con N. Esto es más relevante para el "menos spawns"
  del Goal que el recuento estático de jq: la Task 2 lo trata como la
  prioridad, no como una nota al margen.
- **`ENGINE_MIN`/`_engine-version.sh` (campaña H) ya están en el árbol**
  (PR #24, merged): `plugins/exo/ENGINE_MIN` existe (`0.1.0`) y
  `exo-recall.sh` ya consulta `ENGINE_MIN` (líneas 37-45, 66-118). **Pero la
  Task 2 de H (el guard `engine-stale` en `recall-inject.sh`, descrita en el
  plan `2026-09-15-campana-h-fail-closed.md`) NO se ejecutó**: verificado hoy,
  `recall-inject.sh` no tiene ninguna referencia a `ENGINE_MIN` ni a
  `_engine-version.sh`. La propuesta asumía «H añade 20 líneas a
  `exo-recall.sh` y `recall-inject.sh`»; en el árbol real solo
  `exo-recall.sh` las lleva. Esto no cambia nada de esta campaña (I no toca
  el guard de versión), pero corrige una premisa del orden H→I que la
  propuesta daba por completa en los dos ficheros.
- **`exo-recall.sh` tiene 5 invocaciones reales de `jq`** (líneas 80, 123,
  124, 133, 151), no un número que la propuesta o el backlog dieran. Dos de
  ellas (123, 124: `SOURCE` y `SID` del mismo `$INPUT`) son fundibles en una
  sola — **con separador `\x1f`, no `@tsv`/tab** (ver Task 3: `read` con
  `IFS=tab` desalinea campos cuando el primero está vacío, que es
  precisamente el caso normal de `source` en un prompt cualquiera).
- **Corrección de la review adversarial (verificada con ejecución real):
  el gate reescrito de la Task 2 NO reproduce byte a byte el comportamiento
  del `sed` original bajo cualquier locale.** Bajo `LC_ALL=es_ES.utf8`,
  `sed 's/[^a-z0-9/.-]//g'` (el filtro final de `norm_token()` original)
  **conserva** caracteres fuera del alfabeto español que la reescritura
  bash SIEMPRE filtra (`ç`, `ß`, `ï`, `ö` sobreviven al `sed` viejo bajo ese
  locale y no sobreviven a la versión nueva; medido con los dos código lado
  a lado). El propio `sed` original YA era inconsistente entre locales antes
  de esta campaña (bajo `LC_ALL=C` los filtraba igual que la versión nueva);
  la reescritura de la Task 2 declara el comportamiento bajo `C`
  (equivalente en cualquier locale, porque son sustituciones LITERALES de
  bytes) como el **normativo**, y esto es una mejora deliberada — gate
  determinista independientemente de en qué máquina/locale corra Claude
  Code — no un efecto colateral sin querer. El alfabeto español completo
  (Á É Í Ó Ú Ü Ñ + minúsculas) sí es byte-a-byte idéntico en cualquier
  locale, en las dos versiones: es el caso que de verdad importa para un
  gate en castellano.

## Decisión de Paul firmada HOY (2026-09-19) — Global Constraints

- **#12 — la métrica de reapertura del daemon es `hook_ms` de reloj, umbral
  1.500 ms p95 por SO.** Esto **enmienda** el pre-registro de la campaña A
  (`docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`
  §«Criterio de reapertura del proceso residente (H3)»), que fijaba el
  criterio sobre `elapsed_ms + refresh_ms` (tiempo **interno** del engine,
  reportado por el propio `exo recall`). Motivo medido en
  `evals/recall-coste/results/w11-2026-09-15.txt`: en W11 el hook completo da
  p50 2.312 ms / p95 2.568 ms, mientras que `elapsed_ms+refresh_ms` ronda
  993-1.003 ms en las tres muestras registradas — con la métrica vieja, el
  umbral de 1.500 ms **nunca se dispara en W11** aunque cada prompt cueste
  el doble. El umbral (1.500 ms), el porcentaje de timeouts (2%) y el mínimo
  de disparos (200) **no cambian** — la enmienda es de qué mide el reloj, no
  de dónde está la barrera. Consecuencia operativa (Task 1): `hook_ms` no es
  un dato que se reporta al lado — `recall-latencia.sh` pasa a decidir por
  él. La enmienda se documenta como anexo fechado en el propio pre-registro
  de A (Task 1, Step 6), **antes** de que la Task 6/7 miren ninguna ventana
  real, tal y como exige el contrato de pre-registro de ese fichero.

## Restricciones duras (todas las tasks las heredan)

- **Salida byte-idéntica del bloque inyectado.** `recall-inject.sh` produce
  el texto que se inyecta como `additionalContext` de `UserPromptSubmit`;
  `exo-recall.sh`, el de `SessionStart`. Ninguna task que reescriba su
  parseo puede cambiar un solo byte de ese texto para la misma entrada.
  Cada task que toca ese parseo (2, 3, 4) captura un **golden** — la salida
  completa del hook sobre un puñado de fixtures ya existentes en la suite
  correspondiente — **como PRIMER paso de la task, antes de tocar el
  script**, y termina comparando byte a byte contra ese golden. Los goldens
  quedan commiteados como test de regresión permanente, no como andamiaje
  desechable.
- **bash ≥ 3.2 en macOS.** `$EPOCHREALTIME` es bash ≥5; el `/bin/bash` de
  macOS (usado de verdad en el job `plugin-tests` de CI con
  `runs-on: macos-latest`, confirmado por el propio pin de shellcheck del
  repo y por comentarios existentes — `_timeout.sh:6`, `a1-gate.sh:33`,
  ambos citan la corrida de CI `34720014952` en `macos-latest`) es 3.2 y
  **no la tiene**. La campaña E ya pagó un bug de ese mismo bash
  (`${arr[@]}` tratado como variable no definida bajo `set -u` cuando el
  array está vacío — `subagent-inject.sh:42`); esta campaña no lo repite:
  toda comprobación de versión usa `${BASH_VERSINFO[0]}` (array SIEMPRE no
  vacío, seguro bajo `set -u` en cualquier bash) y **ningún** camino de
  código llama a `${var,,}` (case-fold, bash ≥4, inexistente en 3.2) ni a
  `${arr[@]}` sin el guard `${arr[@]+"${arr[@]}"}`. Donde `$EPOCHREALTIME`
  no está disponible, `hook_ms` queda **vacío/`NA`** — nunca se sustituye
  por `date +%s%N` ni ningún otro spawn: eso sería exactamente el coste que
  esta campaña existe para quitar.
- **Todo snippet bash nuevo o reescrito se prueba en bash 3.2 REAL, no solo
  se razona.** Comando exacto, el mismo que usó la review adversarial de
  esta campaña: `docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w
  bash:3.2 ./<test>.sh` (ajustar el volumen montado si el test vive en otra
  ruta). La imagen `bash:3.2` es Alpine sin `jq` ni `git` — cualquier test
  que los necesite (todos salvo `test-hook-ms.sh`) los instala primero
  dentro del contenedor: `docker run --rm -v "$PWD/plugins/exo/scripts":/w
  -w /w bash:3.2 sh -c "apk add --no-cache jq git >/dev/null 2>&1 &&
  ./<test>.sh"`. Cada task que toca bash (1, 2, 3, 4) incluye este comando
  explícito en su paso de verificación — no es opcional ni "si hay tiempo":
  la primera versión de `_hook-ms.sh` de este mismo plan pasaba su propio
  test en bash ≥5 y fallaba 6 de 8 casos en bash 3.2 real, precisamente
  porque nadie lo había ejecutado ahí antes de darlo por bueno.
  **Excepción conocida y PRE-EXISTENTE, no introducida por esta campaña**:
  en `bash:3.2` (Alpine/busybox) los dos casos `P4` de
  `test-recall-inject.sh` (`timeout propio corta` / `loguea timeout-guard`)
  fallan — verificado que fallan IGUAL sobre el `recall-inject.sh` de
  ANTES de esta campaña, sin tocar nada: `con_timeout` (`_timeout.sh`)
  interactúa mal con el `timeout` de busybox en este contenedor concreto,
  algo distinto del macOS real (que sí tiene GNU `timeout` o cae al
  fallback de `perl` de `_timeout.sh`). Si esos dos casos son los ÚNICOS que
  fallan en el contenedor, no es un rojo de esta campaña — no se investiga
  ni se arregla aquí (fuera de alcance: `_timeout.sh` no lo toca ninguna
  task de este plan).
- **Todo shellcheck nuevo se corre con la versión 0.11.0** (la que usa
  `scripts/test-shellcheck.sh`, pineada por sha256 en `ci.yml`), no con la
  que traiga el sistema — versiones distintas de shellcheck difieren en qué
  reglas activan por defecto. Si no está instalada:
  `curl -fsSL -o /tmp/sc.tar.xz https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz && tar -xJf /tmp/sc.tar.xz -C /tmp` y usar `/tmp/shellcheck-v0.11.0/shellcheck`.
- **Nada escribe en `~/.local/bin`** (entorno vivo de Paul). Ninguna task
  de esta campaña instala, copia ni sobrescribe binarios; todo el trabajo es
  edición de scripts versionados en el repo y lectura de resultados de
  benchmark hacia `evals/recall-coste/results/`.
- **Rama actual: `plan-campanas-i-l-j`. No se cambia de rama. No se
  commitea nada de esta sesión de planificación** (esto lo dicta el encargo
  de escribir el plan, no el plan en sí — cuando `exo:orchestrate` ejecute
  este documento, sí commitea, task a task).
- **Git**: `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Sin `push`. Convención de mensajes de esta rama:
  `feat(i, <área>): …` / `fix(i, <área>): …` / `test(i, <área>): …` /
  `docs(i, <área>): …`.
- **`.gitattributes`: `* text=auto eol=lf`.** No normalizar finales de línea
  a mano.

## Tabla de ficheros tocados por task (colisión con la campaña L en paralelo)

La fábrica corre esta campaña (I) en paralelo con la campaña L, que toca
`scripts/test-*.sh` (los gates de CI en la **raíz** del repo — un directorio
distinto de `plugins/exo/scripts/`), `engine/src/buscador.rs` y
`docs/backlog.md`.

| Task | Ficheros (Create/Modify) | Zona / colisión con L |
|---|---|---|
| 1 | Modify: `plugins/exo/scripts/recall-inject.sh`, `plugins/exo/scripts/recall-latencia.sh`, `plugins/exo/scripts/test-recall-inject.sh`, `plugins/exo/scripts/test-recall-latencia.sh`, `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md` (anexo). Create: `plugins/exo/scripts/_hook-ms.sh`, `plugins/exo/scripts/test-hook-ms.sh` | `plugins/exo/scripts/**` — L no toca este directorio. Sin colisión. |
| 2 | Modify: `plugins/exo/scripts/recall-inject.sh`. Create: `plugins/exo/scripts/test-recall-inject-golden.sh` + `plugins/exo/scripts/testdata/golden-recall-inject/*.txt` | ídem, sin colisión |
| 3 | Modify: `plugins/exo/scripts/exo-recall.sh`. Create: `plugins/exo/scripts/test-exo-recall-golden.sh` + `plugins/exo/scripts/testdata/golden-exo-recall/*.txt` | ídem, sin colisión |
| 4 | Modify: `plugins/exo/scripts/{git-c-bash,git-add-all-guard,verify-before-commit}.sh` y sus `test-*.sh` | ídem, sin colisión |
| 5 (opcional) | Create: `plugins/exo/scripts/bash-guards.sh`, `plugins/exo/scripts/test-bash-guards.sh`. Modify: `plugins/exo/hooks/hooks.json` | `hooks.json` lo **valida** (no lo toca) el gate `scripts/test-hooks-json.sh` de la campaña F, ya mergeada. L no lo toca. Si en el momento de ejecutar la Task 5 L sigue en curso, re-verificar `git -C . log -1 --format=%H -- scripts/test-hooks-json.sh` no haya cambiado desde el inicio de la fábrica antes de tocar `hooks.json`. |
| 6 | Create: `evals/recall-coste/results/campana-i-<fecha>/**` (vía `bench.sh`) | `evals/**` — L no lo toca |
| 7 (PAUL-STEP) | Create (Paul, a mano): `evals/recall-coste/results/w11-<fecha>-campana-i.txt` | ídem |
| 8 | Modify: `docs/backlog.md` (re-anclaje por texto exacto de los dos ítems citados arriba, nunca por número de línea) | Zona **compartida** con L — toda edición usa `Edit` con `old_string` copiado literal del contenido actual, nunca offsets de línea. |

**Ninguna task de esta campaña toca `engine/src/**` ni `scripts/test-*.sh`**
(los de la raíz, gestionados por L) — declarado explícitamente porque L sí
toca `engine/src/buscador.rs` y ese fichero queda fuera del radar de I por
completo.

---

### Task 1: `hook_ms` de reloj — instrumentación + `recall-latencia.sh` decide por `hook_ms`

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`bash plugins/exo/scripts/test-hook-ms.sh`,
`bash plugins/exo/scripts/test-recall-latencia.sh` y
`bash plugins/exo/scripts/test-recall-inject.sh` verdes.

**Evidencia:** hoy `recall-inject.sh` no mide su propio reloj de pared en
ningún punto; `recall-latencia.sh:31-33` calcula el p95 sobre
`elapsed_ms + refresh_ms` (campos que reporta el propio `exo recall --json`,
tiempo interno del engine). Con esa métrica, W11 nunca dispara la
reapertura del daemon (ver decisión #12 arriba).

**Files:**
- Create: `plugins/exo/scripts/_hook-ms.sh`
- Create: `plugins/exo/scripts/test-hook-ms.sh`
- Modify: `plugins/exo/scripts/recall-inject.sh`
- Modify: `plugins/exo/scripts/recall-latencia.sh`
- Modify: `plugins/exo/scripts/test-recall-inject.sh`
- Modify: `plugins/exo/scripts/test-recall-latencia.sh`
- Modify: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`

**Interfaces:**
- Produces: `hook_ms_soportado` (función bash, exit 0 si `bash ≥5`),
  `hook_ms_calcula INICIO FIN` (función bash **pura** — sin guard de
  versión, testeable en CUALQUIER bash incluido 3.2 — setea la global
  `HOOK_MS` con el entero de milisegundos entre INICIO y FIN) y
  `hook_ms_de INICIO [FIN]` (envoltorio que aplica el guard de
  `hook_ms_soportado` y delega en `hook_ms_calcula`; `HOOK_MS` queda vacío
  si no hay soporte). Ninguna imprime por stdout, ninguna hace fork. Las
  consumen `recall-inject.sh` (Task 1 misma) y, potencialmente, cualquier
  hook futuro que quiera el mismo reloj sin spawn.
- Produces: el campo `hook_ms=<entero>|NA` en el payload del evento
  `recall-inject-emitted` del log de reflejos, **entre** `refresh_ms=` y
  `permalinks=` (nunca al final: `_reflex-log.sh` trunca el payload a 2.000
  chars y `permalinks` es el único campo que puede crecer sin cota — un
  campo nuevo después de él podría perderse en el corte).

**Fixes de la review adversarial (2026-09-19, verificados con ejecución
real):**
- `hook_ms_de` hacía `hook_ms_soportado || return 0` **antes** de la
  aritmética, así que un test que la ejercita directamente en bash 3.2 (la
  máquina donde el guard SÍ debe frenar) nunca corre la aritmética que
  quiere probar — 6 de los 8 casos de `test-hook-ms.sh` fallaban en
  `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh`. Fix:
  separar la aritmética pura (`hook_ms_calcula`, sin guard) del envoltorio
  con guard (`hook_ms_de`) — el test llama a la pura.
- `$EPOCHREALTIME` usa **coma** decimal bajo un locale con `LC_NUMERIC` que
  la use (verificado en la máquina de Paul: `LC_NUMERIC=es_ES.utf8` da
  `1789804285,193241`, y Git Bash en Windows hereda el locale regional de
  Windows). Sin normalizar, `${ini%.*}` no encuentra ningún punto que
  cortar y devuelve la cadena ENTERA con la coma dentro; la aritmética
  `10#$s0` sobre eso da un número absurdo sin ni siquiera fallar (medido:
  `HOOK_MS` del orden de 8×10⁸). Fix: `hook_ms_calcula` normaliza
  `ini="${ini//,/.}"; fin="${fin//,/.}"` como primera línea, y
  `recall-inject.sh` fuerza `export LC_NUMERIC=C` (Step 4) — el mismo patrón
  que ya usa `recall-latencia.sh:13`, cinturón y tirantes.

- [ ] **Step 1: Test que falla — `test-hook-ms.sh`**

Crea `plugins/exo/scripts/test-hook-ms.sh`:

```bash
#!/usr/bin/env bash
# Test standalone para _hook-ms.sh: `hook_ms_calcula` es PURA (sin guard de
# versión) a propósito, para poder probar la aritmética en CUALQUIER bash,
# incluido 3.2 — el guard (`hook_ms_soportado`/`hook_ms_de`) se prueba aparte,
# sin mezclar los dos motivos de fallo posibles en el mismo caso.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

. "$SCRIPT_DIR/_hook-ms.sh"

verifica() {  # $1=inicio $2=fin $3=esperado_ms
  hook_ms_calcula "$1" "$2"
  if [ "$HOOK_MS" = "$3" ]; then
    pass "hook_ms_calcula $1 $2 -> $HOOK_MS"
  else
    fail "hook_ms_calcula $1 $2 -> $3" "obtuve '$HOOK_MS'"
  fi
}

verifica "100.900000" "101.100000" 200
verifica "100.500000" "101.100000" 600
# Componente con cero a la izquierda: interpretado como octal sin el `10#`
# de _hook-ms.sh, "007811" revienta la aritmética ("value too great for base").
verifica "1758300000.007811" "1758300000.500000" 492
# Cruce de segundo con el microsegundo del fin MENOR que el del inicio: el
# bug que tenía la primera versión de esta función (dividir el delta de
# microsegundos por separado del de segundos perdía precisión al cruzar el
# borde). 11.000100 - 10.999900 = 200 microsegundos = 0 ms enteros.
verifica "10.999900" "11.000100" 0
verifica "5.000000" "5.000000" 0
verifica "5.000000" "8.000000" 3000
# Coma decimal: $EPOCHREALTIME bajo un locale con LC_NUMERIC que la use (p.
# ej. es_ES.utf8, verificado real en la máquina de Paul y en la de Git Bash
# en Windows, que hereda el locale regional). Sin normalizar la coma, esto
# devolvía un HOOK_MS del orden de 8x10^8 sin ni siquiera fallar.
verifica "10,999900" "11,000100" 0

if hook_ms_soportado; then
  pass "hook_ms_soportado: verdadero en bash ${BASH_VERSINFO[0]} (>=5 en esta máquina)"
else
  # Esta rama solo se ejerce de verdad en CI macOS (bash 3.2 real) o en
  # `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh` (Global
  # Constraints). Aquí documentamos el contrato sin fingir la versión:
  # BASH_VERSINFO es de solo lectura, no se puede simular.
  pass "hook_ms_soportado: falso (bash <5 real de esta máquina)"
fi

# hook_ms_de aplica el guard ENCIMA de la aritmética pura: sin soporte,
# HOOK_MS queda vacío SIN que la aritmética llegue a correr (a diferencia de
# la primera versión de este helper, donde el guard vivía dentro de la misma
# función que la aritmética y un test que llamaba a esa función directamente
# en bash <5 nunca ejercía la cuenta). Esto se prueba de verdad —no
# simulado— en `docker run --rm -v "$PWD":/w -w /w bash:3.2 ./test-hook-ms.sh`.
hook_ms_de "100.0" "101.0"
if hook_ms_soportado; then
  [ "$HOOK_MS" = "1000" ] && pass "hook_ms_de con soporte: delega en hook_ms_calcula" \
    || fail "hook_ms_de con soporte: delega en hook_ms_calcula" "obtuve '$HOOK_MS'"
else
  [ -z "$HOOK_MS" ] && pass "hook_ms_de sin soporte (bash <5 real): HOOK_MS vacío, sin correr la aritmética" \
    || fail "hook_ms_de sin soporte: HOOK_MS vacío" "obtuve '$HOOK_MS'"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

Marca ejecutable y corre:

```bash
chmod +x plugins/exo/scripts/test-hook-ms.sh
bash plugins/exo/scripts/test-hook-ms.sh
```

Expected: falla al cargar — `_hook-ms.sh: No such file or directory` (el
fichero que se sourcea todavía no existe: es el rojo de esta tarea).

Run adicional (obligatorio para esta task — verifica bash 3.2 real, no
simulado; Global Constraints):

```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 ./test-hook-ms.sh
```

Expected en este punto: falla igual, mismo motivo (`_hook-ms.sh` no existe
todavía).

- [ ] **Step 2: Implementación — `_hook-ms.sh`**

Crea `plugins/exo/scripts/_hook-ms.sh`:

```bash
#!/usr/bin/env bash
# Helper COMPARTIDO: hook_ms de reloj de pared para hooks bash (campaña I,
# decisión #12 de Paul, 2026-09-19). Sin spawn: usa $EPOCHREALTIME (bash >=5)
# y aritmética entera de bash. En bash <5 (macOS con /bin/bash 3.2 -- la
# campaña E ya pagó un bug de ese mismo bash con `${arr[@]}` tratado como
# variable no definida bajo `set -u` cuando el array está vacío,
# subagent-inject.sh:42) $EPOCHREALTIME no existe: se degrada a HOOK_MS
# vacío. NUNCA se llama a `date` como sustituto -- ese spawn es exactamente
# lo que esta campaña existe para quitar.
#
# Dos funciones, no una (review adversarial 2026-09-19): `hook_ms_calcula`
# es PURA -- sin mirar la versión de bash -- para que se pueda testear la
# aritmética en CUALQUIER bash, incluido 3.2. `hook_ms_de` es la que se usa
# de verdad en un hook: aplica el guard y delega. Con las dos fundidas en
# una sola función (como la primera versión de este helper), un test que
# llama a esa función directamente en bash <5 nunca llega a ejercer la
# aritmética -- el guard corta antes.
#
# Uso:
#   . "$SCRIPT_DIR/_hook-ms.sh"
#   HOOK_START=""; hook_ms_soportado && HOOK_START="$EPOCHREALTIME"
#   ...
#   hook_ms_de "$HOOK_START"   # setea HOOK_MS (global); vacío si no hay soporte

hook_ms_soportado() { [ "${BASH_VERSINFO[0]:-0}" -ge 5 ]; }

# hook_ms_calcula INICIO FIN -> setea HOOK_MS (global, entero de
# milisegundos). PURA: no mira `hook_ms_soportado`, así que es testeable en
# cualquier bash. INICIO/FIN en formato de $EPOCHREALTIME
# ("SEGUNDOS.MICROS", 6 dígitos de fracción, separador PUNTO en el caso
# normal).
#
# Todo se convierte a microsegundos ANTES de dividir por 1.000: dividir el
# delta de segundos y el de microsegundos por separado y sumar los dos
# resultados pierde precisión cuando el microsegundo del FIN es menor que el
# del INICIO (cruce de segundo) -- medido con "10.999900" -> "11.000100"
# (200 microsegundos reales): esa fórmula daba 1 ms en vez de 0.
#
# Normaliza coma a punto ANTES de partir por `.`: bajo un locale con
# LC_NUMERIC que use coma decimal (es_ES.utf8, verificado real en la máquina
# de Paul -- y Git Bash en Windows hereda el locale regional de Windows),
# $EPOCHREALTIME imprime "1789804285,193241". Sin esto, `${ini%.*}` no
# encuentra ningún punto, devuelve la cadena ENTERA con la coma dentro, y la
# aritmética de abajo da un número absurdo SIN fallar (medido: del orden de
# 8x10^8) -- el recall-inject.sh que llama a esto TAMBIÉN fuerza
# `export LC_NUMERIC=C` (mismo patrón que `recall-latencia.sh:13`), así que
# esto es cinturón y tirantes, no la única defensa.
#
# `10#` fuerza base 10 en la aritmética: un componente con cero a la
# izquierda ("007811") lo interpretaría bash como octal y "008" o "009"
# revientan con "value too great for base 8".
hook_ms_calcula() {
  local ini="$1" fin="$2"
  ini="${ini//,/.}"; fin="${fin//,/.}"
  local s0="${ini%.*}" u0="${ini#*.}" s1="${fin%.*}" u1="${fin#*.}"
  # shellcheck disable=SC2034 # HOOK_MS es la salida global: la lee el llamador
  HOOK_MS=$(( ((10#$s1 - 10#$s0) * 1000000 + (10#$u1 - 10#$u0)) / 1000 ))
}

# hook_ms_de INICIO [FIN] -> guard + hook_ms_calcula. FIN por defecto: ahora.
# HOOK_MS queda vacío (cadena "") si `hook_ms_soportado` es falso -- la
# aritmética de hook_ms_calcula NUNCA llega a correr en ese caso.
hook_ms_de() {
  # shellcheck disable=SC2034 # HOOK_MS es la salida global: la lee el llamador
  HOOK_MS=""
  hook_ms_soportado || return 0
  hook_ms_calcula "$1" "${2:-$EPOCHREALTIME}"
}
```

Marca ejecutable: `chmod +x plugins/exo/scripts/_hook-ms.sh`.

Run adicional: `/tmp/shellcheck-v0.11.0/shellcheck -x plugins/exo/scripts/_hook-ms.sh`
(o `shellcheck` a secas si ya está instalado — versión 0.11.0, la que usa
`scripts/test-shellcheck.sh`). Expected: sin avisos.

- [ ] **Step 3: Verlo verde**

Run: `bash plugins/exo/scripts/test-hook-ms.sh`
Expected: `9 passed, 0 failed` (7 `verifica` + el chequeo de
`hook_ms_soportado` + el de `hook_ms_de`).

Run adicional (bash 3.2 real):

```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 ./test-hook-ms.sh
```

Expected: `9 passed, 0 failed` — en ESTA máquina `hook_ms_soportado` da
falso y la rama sin-soporte de `hook_ms_de` es la que corre de verdad (no
la simulada).

- [ ] **Step 4: Instrumentar `recall-inject.sh`**

`Edit` sobre `plugins/exo/scripts/recall-inject.sh`:

old_string:
```bash
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi
```

new_string:
```bash
set -uo pipefail
# $EPOCHREALTIME (usado más abajo) imprime con COMA decimal bajo un locale
# cuyo LC_NUMERIC la use (es_ES.utf8, verificado real en la máquina de
# Paul; Git Bash en Windows hereda el locale regional de Windows) --
# forzarlo a C es el mismo patrón que ya usa `recall-latencia.sh:13`.
# `_hook-ms.sh` normaliza coma->punto también por su cuenta (cinturón y
# tirantes), pero fijar el locale aquí es más barato que confiar solo en esa
# normalización defensiva.
export LC_NUMERIC=C
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# hook_ms (campaña I, decisión #12): reloj de pared del hook entero, medido
# desde AQUÍ (antes de leer stdin, para que ese spawn de `cat` también
# cuente) hasta el evento `emitted`. La única parte que no se puede medir es
# el `dirname`/`cd`/`pwd` de la línea de arriba, necesarios para localizar
# este mismo helper -- unos pocos ms de suelo de proceso, no el shell que
# esta campaña mide.
. "$SCRIPT_DIR/_hook-ms.sh" 2>/dev/null
HOOK_START=""
hook_ms_soportado 2>/dev/null && HOOK_START="$EPOCHREALTIME"

if [ -t 0 ]; then INPUT=""; else INPUT="$(cat)"; fi
```

Y en el punto donde se loguea el evento `emitted` (al final del script):

old_string:
```bash
log_ri "emitted" "n_hits=$N bytes=$BYTES elapsed_ms=${ELAPSED_MS:-?} refresh_ms=${REFRESH_MS:-?} permalinks=$PERMALINKS"
```

new_string:
```bash
hook_ms_de "$HOOK_START"
log_ri "emitted" "n_hits=$N bytes=$BYTES elapsed_ms=${ELAPSED_MS:-?} refresh_ms=${REFRESH_MS:-?} hook_ms=${HOOK_MS:-NA} permalinks=$PERMALINKS"
```

`log_ri()` (la función que llama a `reflex_log`) **no se toca**: el cambio
vive entero en la línea que arma el payload del evento `emitted`. Ningún
otro `log_ri "degraded" ...` lleva `hook_ms` — esta campaña solo instrumenta
el camino que `recall-latencia.sh` consume (`recall-inject-emitted`).

- [ ] **Step 5: Actualizar el test existente que asume el formato viejo del payload**

`test-recall-inject.sh:557` comprueba el orden literal de campos del
payload `emitted`, que este Step cambia. `Edit`:

old_string:
```bash
if contains "$PL_AV" "elapsed_ms=987 refresh_ms=12 permalinks="; then pass "H3: emitted lleva elapsed_ms y refresh_ms antes de permalinks"
else fail "H3: emitted lleva elapsed_ms y refresh_ms antes de permalinks" "payload='$PL_AV'"; fi
```

new_string:
```bash
if contains "$PL_AV" "elapsed_ms=987 refresh_ms=12 hook_ms=" \
   && printf '%s' "$PL_AV" | grep -qE 'hook_ms=(NA|[0-9]+) permalinks='; then
  pass "H3/I1: emitted lleva elapsed_ms, refresh_ms y hook_ms (NA o entero) antes de permalinks"
else
  fail "H3/I1: emitted lleva elapsed_ms, refresh_ms y hook_ms antes de permalinks" "payload='$PL_AV'"
fi
```

- [ ] **Step 6: Verlo verde**

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: `0 failed` (mismo conteo de PASS que antes de este Step, más
ninguno nuevo perdido — el bloque inyectado no cambió, solo el payload del
log).

Run adicional (bash 3.2 real, Global Constraints):
```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 \
  sh -c "apk add --no-cache jq >/dev/null 2>&1 && ./test-recall-inject.sh"
```
Expected: `0 failed`, **salvo** los dos casos `P4` (`timeout propio corta` /
`loguea timeout-guard`), que fallan en este contenedor concreto por una
razón PRE-EXISTENTE y ajena a esta task (ver Global Constraints —
`con_timeout`/busybox, no algo que este Step toque). Si fallan MÁS de esos
dos casos, sí es un rojo real de este Step.

- [ ] **Step 7: `recall-latencia.sh` decide por `hook_ms`**

`Edit` sobre `plugins/exo/scripts/recall-latencia.sh`:

old_string:
```bash
#!/usr/bin/env bash
# Criterio de reapertura del proceso residente (H3, campaña A). READ-ONLY.
#
# La spec 2026-08-22-m6-06-recall-punto-de-uso-design.md §2.2 aceptó «el segundo
# por turno» con reapertura «si duele tras semanas de uso», sin mecanismo que
# detectara el dolor. Esto es ese mecanismo: lee los eventos de recall-inject.sh
# del log de ESTA máquina (W11 y Linux se evalúan por separado).
#
# Uso: recall-latencia.sh [DESDE] [HASTA]   (YYYY-MM-DD, inclusivas; por defecto todo)
# Umbrales HARDCODEADOS adrede: son pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md §«Criterio de reapertura».)
```

new_string:
```bash
#!/usr/bin/env bash
# Criterio de reapertura del proceso residente (H3, campaña A). READ-ONLY.
#
# La spec 2026-08-22-m6-06-recall-punto-de-uso-design.md §2.2 aceptó «el segundo
# por turno» con reapertura «si duele tras semanas de uso», sin mecanismo que
# detectara el dolor. Esto es ese mecanismo: lee los eventos de recall-inject.sh
# del log de ESTA máquina (W11 y Linux se evalúan por separado).
#
# Uso: recall-latencia.sh [DESDE] [HASTA]   (YYYY-MM-DD, inclusivas; por defecto todo)
# Umbrales HARDCODEADOS adrede: son pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md §«Criterio de reapertura».)
#
# ENMIENDA (2026-09-19, decisión de Paul #12, campaña I): la métrica de p95
# pasa de `elapsed_ms + refresh_ms` (tiempo INTERNO del engine) a `hook_ms`
# (reloj de PARED del hook `recall-inject.sh` entero). En W11 el shell
# alrededor del binario cuesta tanto como el binario mismo
# (evals/recall-coste/results/w11-2026-09-15.txt: hook p50 2.312 ms frente a
# elapsed_ms+refresh_ms ~993-1.003 ms) -- con la métrica vieja este criterio
# nunca dispara en W11 aunque cada prompt cueste el doble. Umbral (1.500 ms),
# porcentaje de timeouts (2%) y mínimo de disparos (200) NO cambian.
```

Y en el cuerpo del `jq`:

old_string:
```bash
  [ $ev[] | select(.reflex == "recall-inject-emitted") | (.payload // "")
      | {e: campo("elapsed_ms"), r: campo("refresh_ms")} | select(.e != null)
      | .e + (.r // 0) ] | sort as $ms
```

new_string:
```bash
  [ $ev[] | select(.reflex == "recall-inject-emitted") | (.payload // "")
      | campo("hook_ms") | select(. != null) ] | sort as $ms
```

`campo($k)` ya extrae `hook_ms=<entero>` con la misma regex genérica que
usaba para `elapsed_ms`/`refresh_ms` (no matchea `hook_ms=NA`, que carece de
dígitos — exactamente el comportamiento deseado: las líneas `NA` cuentan
igual que las líneas viejas sin el campo, es decir, no aportan al p95).

- [ ] **Step 8: Actualizar `test-recall-latencia.sh` a payloads con `hook_ms`**

El generador de fixtures `emite()` sintetiza el payload con
`elapsed_ms=%d refresh_ms=%d`; ahora también necesita `hook_ms=%d` para que
los tests ejerzan el campo real que consume el script. `Edit`:

old_string:
```bash
# emite <n> <elapsed_ms> <refresh_ms> <session> <fecha>
emite() {
  awk -v n="$1" -v e="$2" -v r="$3" -v s="$4" -v d="$5" 'BEGIN {
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"%s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 elapsed_ms=%d refresh_ms=%d permalinks=kb/a,kb/b\"}\n", d, s, e, r
  }'
}
```

new_string:
```bash
# emite <n> <base_ms> <refresh_ms> <session> <fecha>
# `hook_ms` del payload sintético = base_ms + refresh_ms (fix de la review
# adversarial 2026-09-19): las CINCO llamadas a `emite` de este fichero ya
# existían antes de la campaña I con estos mismos números pensados como
# "elapsed_ms + refresh_ms suman la latencia total del caso" (ver los
# comentarios de cada caso, p. ej. "p95 = 910" = 900+10). Si `hook_ms`
# fuera solo `base_ms` sin sumar `refresh_ms`, el caso 1 daría p95=900 en vez
# de los 910 que su propia aserción espera — desalineación real, detectada
# corriendo el test, no solo leyéndolo. Sumar aquí preserva las CINCO
# aserciones existentes sin tocarlas: no es una fórmula real de producción
# (hook_ms de verdad es un número medido, no una suma), es una elección de
# este fixture sintético para no reescribir comentarios y aserciones que ya
# estaban bien.
emite() {
  awk -v n="$1" -v b="$2" -v r="$3" -v s="$4" -v d="$5" 'BEGIN {
    for (i = 0; i < n; i++)
      printf "{\"ts\":\"%sT10:00:00Z\",\"reflex\":\"recall-inject-emitted\",\"session_id\":\"%s\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"\",\"payload\":\"n_hits=3 bytes=900 elapsed_ms=999 refresh_ms=%d hook_ms=%d permalinks=kb/a,kb/b\"}\n", d, s, r, (b+r)
  }'
}
```

Y los cinco usos de `emite` (casos 1, 2, 3, 5, 6) mantienen la misma forma
posicional `emite <n> <X> <Y> <session> <fecha>` y los mismos valores
numéricos — ningún `Edit` adicional hace falta en los cinco `{ emite ... }`
existentes; `hook_ms` sale de `X+Y` automáticamente.

El caso 6 («Payloads sin tiempos... no rompen ni cuentan») usa un payload
escrito a mano sin `hook_ms` — ese SÍ se mantiene igual (es justo el caso
que prueba la compatibilidad con logs viejos, y el nuevo `emite()` mete
`hook_ms` en todas sus líneas, así que la línea a mano sigue siendo la única
sin el campo).

- [ ] **Step 9: Verlo verde**

Run: `bash plugins/exo/scripts/test-recall-latencia.sh`
Expected: `7 passed, 0 failed` (los 7 casos existentes, ahora ejercitando
`hook_ms` en vez de `elapsed_ms+refresh_ms`).

Run adicional (bash 3.2 real):
```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 \
  sh -c "apk add --no-cache jq >/dev/null 2>&1 && ./test-recall-latencia.sh"
```
Expected: `7 passed, 0 failed` igual (`recall-latencia.sh` no usa
`$EPOCHREALTIME`, solo lee el campo `hook_ms` ya escrito por otro proceso —
no depende de la versión de bash que lo ejecuta).

- [ ] **Step 10: Anexo fechado al pre-registro de A**

`Edit` sobre
`docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`:

old_string:
```
Los valores 1.500 ms, 2% y 200 son una **propuesta** que queda pendiente de
Paul (D4 del plan). Si los cambia al gatear, se sustituyen aquí y en las tres
constantes de la Task 13 antes de ejecutarla. La ventana se evalúa **por
máquina**: el log vive en el `$HOME` de cada una, y W11 no se mezcla con Linux.
```

new_string:
```
Los valores 1.500 ms, 2% y 200 son una **propuesta** que queda pendiente de
Paul (D4 del plan). Si los cambia al gatear, se sustituyen aquí y en las tres
constantes de la Task 13 antes de ejecutarla. La ventana se evalúa **por
máquina**: el log vive en el `$HOME` de cada una, y W11 no se mezcla con Linux.

**Enmienda (2026-09-19, decisión de Paul #12, campaña I):** el criterio de
p95 pasa de `elapsed_ms + refresh_ms` (tiempo interno del engine, el que
reporta el propio `exo recall`) a **`hook_ms`** (reloj de pared del hook
`recall-inject.sh` entero, medido con `$EPOCHREALTIME`, sin spawn). Motivo:
en W11 el shell alrededor del binario cuesta tanto como el binario mismo
(`evals/recall-coste/results/w11-2026-09-15.txt`: hook p50 2.312 ms frente a
`elapsed_ms+refresh_ms` ~993-1.003 ms en las tres muestras registradas) —
con la métrica vieja, el criterio de 1.500 ms nunca se dispara en W11 aunque
cada prompt cueste el doble. El umbral (1.500 ms), el porcentaje de
timeouts (2%) y el mínimo de disparos (200) **no cambian**: la enmienda es
solo de qué mide el reloj, no de dónde está la barrera. Implementado en
`docs/superpowers/plans/2026-09-19-campana-i-latencia-hook-w11.md` (Task 1).
No se reescribe el texto de arriba: esto es un anexo fechado, como pide el
propio contrato de pre-registro de la cabecera de este fichero.
```

- [ ] **Step 11: Commit**

```bash
git add plugins/exo/scripts/_hook-ms.sh plugins/exo/scripts/test-hook-ms.sh \
        plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/recall-latencia.sh \
        plugins/exo/scripts/test-recall-inject.sh plugins/exo/scripts/test-recall-latencia.sh \
        docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md
git commit -m "feat(i, hook-ms): reloj de pared del hook sin spawn -- recall-latencia.sh decide por hook_ms (decision #12), enmienda al pre-registro de A"
```

---

### Task 2: `recall-inject.sh` — golden + fusión de spawns (gate sin `sed`/`tr`, `META` fundido con el chequeo de envelope)

**Lane:** mecánica. **Depende de:** Task 1 (mismo fichero — se aplica
después para no pisar el mismo rango de líneas dos veces en paralelo).
**Oráculo:** `bash plugins/exo/scripts/test-recall-inject.sh` y
`bash plugins/exo/scripts/test-recall-inject-golden.sh` verdes, con el
golden capturado ANTES de tocar el script.

**Evidencia:** ver §«Verificación de la propuesta» arriba. Objetivo
concreto: de **7 `jq` + 2 `sed` + 5 `tr`** (14 spawns estáticos, más 3
spawns adicionales — 2 `sed` + 1 `tr` — por cada token del prompt que
`gate_skip()` normaliza antes de encontrar el primero no-stopword) a **6
`jq` + 0 `sed` + 0 `tr`**, sin ningún spawn dentro del bucle léxico.

**Files:**
- Modify: `plugins/exo/scripts/recall-inject.sh`
- Modify: `plugins/exo/scripts/test-recall-inject.sh` (Step 2 bis: nuevo caso
  de consistencia de locale)
- Create: `plugins/exo/scripts/test-recall-inject-golden.sh`
- Create: `plugins/exo/scripts/testdata/golden-recall-inject/*.txt` (6
  ficheros, uno por escenario — generados por el Step 1, no escritos a mano)

**Interfaces:**
- Consumes: nada nuevo de otras tasks.
- Produces: nada que otra task consuma — el gate y la composición del
  bloque son internos a este script.

- [ ] **Step 1: Golden ANTES de tocar el script**

Crea `plugins/exo/scripts/test-recall-inject-golden.sh` — reutiliza
literalmente 6 fixtures ya existentes en `test-recall-inject.sh`
(`CUATRO`, `GORDO`, `RECORTE`, `TITREP`, `UNICO`, `AVISA`), que entre las
seis ejercitan: dedup de core-index, cap de bytes exacto, recorte de
snippet, omisión de título repetido, un único hit (cabecera sin raíz común),
y avisos del engine (`H2`).

```bash
#!/usr/bin/env bash
# Golden de equivalencia para recall-inject.sh (campaña I, Task 2): congela
# la salida COMPLETA (el bloque inyectado, byte a byte) de seis escenarios ya
# existentes en test-recall-inject.sh, para que ninguna task que reescriba el
# parseo (jq/sed/tr -> bash) cambie ni un byte de lo que se inyecta.
#
# RECAPTURA=1 (re)escribe los goldens. Sin ella, compara y FALLA si faltan o
# difieren -- nunca aprueba en silencio un golden ausente.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/recall-inject.sh"
GOLD_DIR="${SCRIPT_DIR}/testdata/golden-recall-inject"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"
export EXO_INJECT_CAP="${EXO_INJECT_CAP:-1024}"
export EXO_KB_NAME="kb-demo"
FAKE_DB="$TMP/index.db"; : > "$FAKE_DB"; export EXO_INDEX="$FAKE_DB"

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

run_hook() {  # $1=prompt $2=exo_bin
  printf '%s' "$1" | jq -Rs '{prompt:., session_id:"golden-sess"}' \
    | EXO_BIN="$2" "$HOOK" 2>/dev/null
}

# --- Fixtures: copiadas literalmente de test-recall-inject.sh (misma fuente) ---
CUATRO="$TMP/exo-cuatro"
cat > "$CUATRO" <<'PYEOF'
#!/usr/bin/env bash
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":1400,"mode":"consulta","notes":[
{"permalink":"kb-demo/core/core-index","path":"/kb/core/core-index.md","score":0.6,"snippet":"mapa de memoria","tier":null,"title":"core-index"},
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"},
{"permalink":"kb-demo/projects/kbx","path":"/kb/projects/kbx.md","score":0.47,"snippet":"destilado de kbx","tier":null,"title":"kbx"},
{"permalink":"kb-demo/log/exo-bitacora","path":"/kb/log/exo-bitacora.md","score":0.44,"snippet":"bitacora de exo","tier":null,"title":"exo-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
PYEOF
chmod +x "$CUATRO"

GORDO="$TMP/exo-gordo"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/n"+($i|tostring)),
  path:("/kb/log/nota-larga-numero-"+($i|tostring)+".md"),
  score:0.5, tier:null,
  title:("nota larga numero "+($i|tostring)),
  snippet:(("palabra "*25)+"fin")}]}}' > "$TMP/gordo.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/gordo.json" > "$GORDO"
chmod +x "$GORDO"

RECORTE="$TMP/exo-recorte"
jq -n '{data:{notes:[range(1;4) as $i | {
  permalink:("kb-demo/log/r"+($i|tostring)),
  path:("/kb/log/recorte-"+($i|tostring)+".md"), score:0.5, tier:null,
  title:("recorte "+($i|tostring)),
  snippet:(("análisis técnico — decisión sellada según medición práctica; "*8))}]}}' \
  > "$TMP/recorte.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/recorte.json" > "$RECORTE"
chmod +x "$RECORTE"

TITREP="$TMP/exo-titrep"
jq -n '{data:{notes:[
 {permalink:"kb-demo/log/kbx-bitacora",path:"/kb/log/kbx-bitacora.md",score:0.5,tier:null,
  title:"kbx-bitacora",snippet:"# kbx-bitacora  cuerpo real de la bitacora"},
 {permalink:"kb-demo/log/otra",path:"/kb/log/otra.md",score:0.4,tier:null,
  title:"Un título que sí aporta",snippet:"cuerpo de la otra"}]}}' > "$TMP/titrep.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/titrep.json" > "$TITREP"
chmod +x "$TITREP"

UNICO="$TMP/exo-unico"
jq -n '{data:{notes:[{permalink:"kb-demo/log/solo",path:"/kb/log/solo.md",score:0.5,
  tier:null,title:"nota solitaria",snippet:"cuerpo de la unica nota"}]}}' > "$TMP/unico.json"
printf '#!/usr/bin/env bash\ncat "%s"\n' "$TMP/unico.json" > "$UNICO"
chmod +x "$UNICO"

AVISA="$TMP/exo-avisa"
cat > "$AVISA" <<'EOF'
#!/usr/bin/env bash
echo "aviso: arm vector INERTE: 0 vectores para 9 trozos." >&2
cat <<'JSON'
{"command":"recall","data":{"cap_bytes":4000,"mode":"consulta","elapsed_s":0.9876,"refresh_s":0.0123,"warnings":["arm vector INERTE: 0 vectores para 9 trozos."],"notes":[
{"permalink":"kb-demo/log/kbx-bitacora","path":"/kb/log/kbx-bitacora.md","score":0.5,"snippet":"bitacora de kbx","tier":null,"title":"kbx-bitacora"}
],"query":"kbx","truncated":false},"schema_version":2}
JSON
EOF
chmod +x "$AVISA"

# --- Comparación: cada escenario emite EXACTAMENTE el stdout completo del hook ---
# `tr -d '\r'` en AMBOS lados (review adversarial 2026-09-19): jq en
# Windows/Git Bash emite CRLF (mismo hallazgo ya documentado en
# `scripts/test-hooks-json.sh:14-16` — "jq en Windows/Git Bash emite CRLF:
# cada salida que se compara o se lee línea a línea pasa por tr -d '\r'").
# Sin esto, el golden capturado en Linux (sin \r) nunca compararía en verde
# contra la salida real de windows-latest (con \r) aunque el CONTENIDO sea
# idéntico -- un falso rojo que no dice nada sobre el bloque inyectado.
comprueba() {  # $1=nombre $2=prompt $3=bin
  local nombre="$1" out golden="$GOLD_DIR/$1.txt"
  out="$(run_hook "$2" "$3" | tr -d '\r')"
  if [ "${RECAPTURA:-0}" = "1" ]; then
    mkdir -p "$GOLD_DIR"
    printf '%s' "$out" > "$golden"
    pass "golden: $nombre capturado en $golden"
    return
  fi
  if [ ! -f "$golden" ]; then
    fail "golden: $nombre" "no existe $golden — corre con RECAPTURA=1 antes de comparar"
    return
  fi
  if [ "$out" = "$(tr -d '\r' < "$golden")" ]; then
    pass "golden: $nombre sin cambios"
  else
    fail "golden: $nombre" "difiere de $golden"
  fi
}

comprueba cuatro-hits "kbx trinquete" "$CUATRO"
comprueba gordo-cap-exacto "kbx trinquete" "$GORDO"
comprueba recorte-snippet "kbx trinquete" "$RECORTE"
comprueba titulo-repetido "kbx trinquete" "$TITREP"
comprueba unico-hit "kbx trinquete" "$UNICO"
comprueba con-avisos "kbx trinquete" "$AVISA"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

Marca ejecutable y **captura el golden sobre el script SIN TOCAR**:

```bash
chmod +x plugins/exo/scripts/test-recall-inject-golden.sh
RECAPTURA=1 bash plugins/exo/scripts/test-recall-inject-golden.sh
```

Expected: `6 passed, 0 failed`, y
`plugins/exo/scripts/testdata/golden-recall-inject/` con 6 ficheros `.txt`
nuevos.

Run de verificación (sin `RECAPTURA`, debe comparar en verde contra lo que
se acaba de capturar):

```bash
bash plugins/exo/scripts/test-recall-inject-golden.sh
```

Expected: `6 passed, 0 failed`.

- [ ] **Step 2: Reescribir el gate — `norm_token()` sin `sed`/`tr`**

`Edit` sobre `plugins/exo/scripts/recall-inject.sh`:

old_string:
```bash
STOP=" $(printf '%s' "$STOP" | tr '\n' ' ') "
```

new_string:
```bash
# Sustituye `tr '\n' ' '` por expansión de parámetros bash pura (campaña I):
# CERO spawns, funciona en cualquier bash >=3.2 (ANSI-C quoting `$'\n'` es
# anterior a esa versión).
STOP=" ${STOP//$'\n'/ } "
```

old_string:
```bash
norm_token() {
  local t
  # shellcheck disable=SC2018,SC2019 # los acentos ya los pliega el sed de arriba; tr solo ve ASCII
  t="$(printf '%s' "$1" | sed \
        -e 's/Á/A/g' -e 's/É/E/g' -e 's/Í/I/g' -e 's/Ó/O/g' -e 's/Ú/U/g' \
        -e 's/Ü/U/g' -e 's/Ñ/N/g' \
        -e 's/á/a/g' -e 's/é/e/g' -e 's/í/i/g' -e 's/ó/o/g' -e 's/ú/u/g' \
        -e 's/ü/u/g' -e 's/ñ/n/g' \
        | tr 'A-Z' 'a-z' 2>/dev/null)" || t="$1"
  printf '%s' "$t" | sed 's/[^a-z0-9/.-]//g' 2>/dev/null || true
}
```

new_string:
```bash
# Sustituye 2 `sed` + 1 `tr` POR TOKEN por expansión de parámetros bash pura
# (campaña I): CERO spawns, sin subshell (setea la global TOKEN_NORM en vez
# de imprimir — `n="$(norm_token "$tok")"` habría forkeado igual que un
# spawn externo en Git Bash/MSYS2, donde fork() está emulado y no es barato).
# Verificado equivalente byte a byte contra la versión sed/tr sobre 24 tokens
# del alfabeto ESPAÑOL (acentos, mayúsculas, glob, LC_ALL=C incluido) — que
# es el alfabeto que un gate en castellano necesita cubrir.
#
# NO es equivalente byte a byte para caracteres FUERA de ese alfabeto bajo un
# locale no-C (medido: `ç`/`ß`/`ï`/`ö` sobreviven al `sed` final original
# bajo `LC_ALL=es_ES.utf8` -- el propio filtro de caracteres de sed se vuelve
# locale-aware ahí -- y esta reescritura los filtra SIEMPRE, igual que el
# `sed` original bajo `LC_ALL=C`). Es una divergencia deliberada, no un bug
# sin ver: el comportamiento nuevo (equivalente a C en cualquier locale,
# porque son sustituciones LITERALES de bytes) es el que se declara
# NORMATIVO a partir de esta campaña — un gate cuyo criterio de disparo
# cambiara según el locale regional de la máquina de Paul sería peor que uno
# determinista que no cubre acentos franceses/alemanes. Ver Step 2 bis.
#
# El ORDEN importa: primero se pliegan los acentos (deja solo ASCII), LUEGO
# se pasa a minúsculas con sustituciones LITERALES letra a letra — nunca
# `${t,,}` (ese operador es bash >=4, inexistente en el /bin/bash 3.2 de
# macOS; y aunque existiera, bajo locale C/POSIX foldea mal el multibyte,
# que es justo el bug medido que obligó al `sed` original: "SÍ" -> "s").
# Con el acento ya plegado a ASCII antes de este punto, la sustitución
# LITERAL A->a es un match de bytes, no una operación de locale: funciona
# igual bajo cualquier locale y cualquier versión de bash.
norm_token() {
  local t="$1"
  t="${t//Á/A}"; t="${t//É/E}"; t="${t//Í/I}"; t="${t//Ó/O}"; t="${t//Ú/U}"
  t="${t//Ü/U}"; t="${t//Ñ/N}"
  t="${t//á/a}"; t="${t//é/e}"; t="${t//í/i}"; t="${t//ó/o}"; t="${t//ú/u}"
  t="${t//ü/u}"; t="${t//ñ/n}"
  t="${t//A/a}"; t="${t//B/b}"; t="${t//C/c}"; t="${t//D/d}"; t="${t//E/e}"
  t="${t//F/f}"; t="${t//G/g}"; t="${t//H/h}"; t="${t//I/i}"; t="${t//J/j}"
  t="${t//K/k}"; t="${t//L/l}"; t="${t//M/m}"; t="${t//N/n}"; t="${t//O/o}"
  t="${t//P/p}"; t="${t//Q/q}"; t="${t//R/r}"; t="${t//S/s}"; t="${t//T/t}"
  t="${t//U/u}"; t="${t//V/v}"; t="${t//W/w}"; t="${t//X/x}"; t="${t//Y/y}"
  t="${t//Z/z}"
  TOKEN_NORM="${t//[^a-z0-9\/.-]/}"
}
```

old_string:
```bash
  local tok n hay=0
  for tok in $p; do
    n="$(norm_token "$tok")"
    [ -n "$n" ] || continue
```

new_string:
```bash
  local tok n hay=0
  for tok in $p; do
    norm_token "$tok"
    n="$TOKEN_NORM"
    [ -n "$n" ] || continue
```

- [ ] **Step 2 bis: caso de consistencia de locale en `test-recall-inject.sh`**

Añade a `plugins/exo/scripts/test-recall-inject.sh`, justo después del caso
`F2` existente (`"SÍ, DALE" ... bajo LC_ALL=C`, línea ~118-124 de este
worktree):

```bash
# F2b (campaña I): el gate se comporta IGUAL bajo un locale con coma decimal
# y colación no-C (es_ES.utf8) que bajo C -- declarado normativo en Step 2
# (ver el comentario de norm_token en recall-inject.sh: el sed viejo SÍ
# divergía aquí, esta reescritura no). Se salta si la máquina no tiene el
# locale instalado, en vez de fallar por un motivo ajeno al gate.
if locale -a 2>/dev/null | grep -qi '^es_ES\.utf8$'; then
  : > "$REFLEX_LOG_FILE"
  printf '%s' "SÍ, DALE" | jq -Rs '{prompt:., session_id:"test-sess"}' \
    | LC_ALL=es_ES.utf8 EXO_BIN="$NO_BIN" "$HOOK" >/dev/null 2>&1
  if ! grep -q 'no-engine' "$REFLEX_LOG_FILE" 2>/dev/null; then
    pass "F2b: 'SÍ, DALE' calla también bajo LC_ALL=es_ES.utf8"
  else
    fail "F2b: 'SÍ, DALE' calla también bajo LC_ALL=es_ES.utf8" "la normalización depende del locale"
  fi
else
  pass "F2b: SKIP (es_ES.utf8 no instalado en esta máquina)"
fi
```

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: `F2b` en verde (o `SKIP` si el locale no está instalado — ninguna
de las dos cuenta como fallo).

- [ ] **Step 3: Fundir el chequeo de envelope con la extracción de `META`**

old_string:
```bash
if ! printf '%s' "$SALIDA" | jq -e 'has("data") and (.data | has("notes"))' >/dev/null 2>&1; then
  log_ri "degraded" "reason=error err=envelope-ilegible"
  exit 0
fi

# Metadatos del envelope en UNA pasada de jq (H2/H3): cada spawn cuesta decenas
# de ms en Git Bash. Si el engine recortó su propia respuesta (cap de fetch), el
# hit de repuesto puede haber desaparecido: no degrada nada, pero deja rastro.
# `@tsv` con los avisos AL FINAL, porque `read` colapsa un campo vacío en medio
# (el tab es whitespace de IFS).
META="$(printf '%s' "$SALIDA" | jq -r '[ (.data.truncated // false | tostring),
    ((.data.elapsed_s // 0) * 1000 | floor | tostring),
    ((.data.refresh_s // 0) * 1000 | floor | tostring),
    ((.data.warnings // []) | join(" | ")) ] | @tsv' 2>/dev/null)" || META=""
TRUNCADO=""; ELAPSED_MS=""; REFRESH_MS=""; AVISOS=""
[ -n "$META" ] && IFS=$'\t' read -r TRUNCADO ELAPSED_MS REFRESH_MS AVISOS <<< "$META"
```

new_string:
```bash
# Envelope + metadatos en UNA SOLA pasada de jq (campaña I; antes eran dos:
# un `jq -e` solo para validar la forma y un `jq -r` separado para extraer
# los campos). Si el envelope no tiene `data.notes`, jq emite el centinela
# "envelope-ilegible" (sin tabs, indistinguible de un fallo de jq — ambos
# caen al mismo `case` de abajo). `@tsv` con los avisos AL FINAL, porque
# `read` colapsa un campo vacío en medio (el tab es whitespace de IFS).
META="$(printf '%s' "$SALIDA" | jq -r '
  if (has("data") and (.data | has("notes"))) then
    [ (.data.truncated // false | tostring),
      ((.data.elapsed_s // 0) * 1000 | floor | tostring),
      ((.data.refresh_s // 0) * 1000 | floor | tostring),
      ((.data.warnings // []) | join(" | ")) ] | @tsv
  else
    "envelope-ilegible"
  end' 2>/dev/null)" || META=""

case "$META" in
  *$'\t'*) ;;
  *) log_ri "degraded" "reason=error err=envelope-ilegible"; exit 0 ;;
esac
TRUNCADO=""; ELAPSED_MS=""; REFRESH_MS=""; AVISOS=""
IFS=$'\t' read -r TRUNCADO ELAPSED_MS REFRESH_MS AVISOS <<< "$META"
```

- [ ] **Step 4: Verlo verde — suite completa + golden**

Run:
```bash
bash plugins/exo/scripts/test-recall-inject.sh
```
Expected: `0 failed` (mismo comportamiento observable de siempre — gate,
degradaciones, composición del bloque — sin ninguna aserción rota).

Run:
```bash
bash plugins/exo/scripts/test-recall-inject-golden.sh
```
Expected: `6 passed, 0 failed` — **sin** `RECAPTURA=1`. Si algo difiere, el
`fail` dice qué escenario y contra qué fichero: NO se regenera el golden
para hacerlo pasar sin entender por qué cambió.

Run adicional (bash 3.2 real, Global Constraints):
```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 \
  sh -c "apk add --no-cache jq >/dev/null 2>&1 && ./test-recall-inject.sh && ./test-recall-inject-golden.sh"
```
Expected: `test-recall-inject-golden.sh` en `6 passed, 0 failed`.
`test-recall-inject.sh` en `0 failed` **salvo** los dos casos `P4`
documentados en Global Constraints (pre-existentes, ajenos a esta task —
si fallan MÁS de esos dos, sí es un rojo real).

Run adicional (verificación del ahorro de spawns, informativo — no es gate,
pero documenta el resultado en el mensaje de commit):
```bash
EXO_BIN=/tmp/exo-stub-count EXO_INDEX=/tmp/index-count.db \
  strace -f -c -e trace=execve -o /tmp/strace-after.log \
  bash plugins/exo/scripts/recall-inject.sh < /tmp/prompt-count.json > /dev/null 2>/dev/null
tail -5 /tmp/strace-after.log
```
(Requiere el stub y el prompt de fixture del propio Step de verificación —
si no existen en la máquina del ejecutor, se recrean con el mismo prompt de
prueba del pre-registro de A, `"como funciona el trinquete de techos"`, y
cualquier binario `exo` de stub que responda a `config`/`recall`.) Expected:
menos `execve` que en una corrida equivalente antes de esta task (la del
§«Verificación de la propuesta» arriba medía 31 antes de tocar nada).

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/recall-inject.sh \
        plugins/exo/scripts/test-recall-inject.sh \
        plugins/exo/scripts/test-recall-inject-golden.sh \
        plugins/exo/scripts/testdata/golden-recall-inject
git commit -m "fix(i, recall-inject): gate sin sed/tr y META fundido con el chequeo de envelope -- golden de equivalencia byte a byte, verificado en bash 3.2 real"
```

---

### Task 3: `exo-recall.sh` — golden + fusión de `SOURCE`/`SID`

**Lane:** mecánica. **Depende de:** nada (fichero distinto de Task 1/2).
**Oráculo:** `bash plugins/exo/scripts/test-exo-recall.sh` y
`bash plugins/exo/scripts/test-exo-recall-golden.sh` verdes.

**Evidencia:** `exo-recall.sh` tiene 5 invocaciones reales de `jq` (líneas
80, 123, 124, 133, 151 de este worktree). Dos de ellas (123, 124) extraen
`SOURCE` y `SID` del MISMO `$INPUT` en dos pasadas separadas — fundibles en
una con `@tsv`, el mismo patrón que ya usa `META` en `recall-inject.sh`.
Este hook corre una vez por sesión (no por prompt), así que el ahorro
absoluto es menor que en Task 2, pero el patrón de reescritura es el mismo y
la campaña lo cubre por completitud del Goal («el hook de cada prompt Y de
cada arranque»).

**Files:**
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Modify: `plugins/exo/scripts/test-exo-recall.sh` (Step 1 bis: caso de
  aislamiento del separador de campos)
- Create: `plugins/exo/scripts/test-exo-recall-golden.sh`
- Create: `plugins/exo/scripts/testdata/golden-exo-recall/*.txt` (2 ficheros)

**Fix de la review adversarial (2026-09-19, verificado con ejecución
real):** la fusión de `SOURCE`/`SID` NO puede usar `@tsv` (tab) como
separador con `IFS=$'\t' read`. `read` trata el tab como whitespace de IFS
incluso cuando es el ÚNICO carácter de `IFS`: un campo VACÍO antes de un tab
se COLAPSA (no se respeta como delimitador), así que con `source` ausente
(el caso normal — la inmensa mayoría de prompts no llevan `source`), el
valor de `session_id` se cuela en `SOURCE` y `SID` queda vacío. Verificado:
`IFS=$'\t' read -r A B <<< "$(printf '%s\t%s' '' 'sess-x')"` da `A=sess-x
B=` (mal); con `\x1f` (unit separator, NO es un carácter de whitespace de
IFS) da `A= B=sess-x` (bien). En el USO actual de `exo-recall.sh` este bug
queda enmascarado — el guard de reafirmación exige `SOURCE == "compact"` Y
`SID` no vacío A LA VEZ, y el mismo desplazamiento que ensucia `SOURCE`
también vacía `SID`, así que el resultado visible no cambia para ningún
prompt real — pero es un bug real de la técnica, no una casualidad segura:
se corrige con `\x1f` y se prueba en aislado (Step 1 bis), no se confía en
que el enmascaramiento actual siga siendo cierto si `exo-recall.sh` cambia
de uso en el futuro.

**Interfaces:** ninguna consumida ni producida hacia otras tasks.

- [ ] **Step 1: Golden ANTES de tocar el script**

Crea `plugins/exo/scripts/test-exo-recall-golden.sh` — reutiliza el stub
`STUB_FELIZ` (camino feliz) y el fixture de reafirmación tras `compact` de
`test-exo-recall.sh` (líneas 25-39: es EXACTAMENTE el código que la Task 3
toca — la extracción de `SOURCE`/`SID`).

```bash
#!/usr/bin/env bash
# Golden de equivalencia para exo-recall.sh (campaña I, Task 3).
# RECAPTURA=1 (re)escribe los goldens; sin ella, compara y FALLA si faltan o
# difieren.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/exo-recall.sh"
GOLD_DIR="${SCRIPT_DIR}/testdata/golden-exo-recall"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME/.claude"
LOGC="$HOME/.claude/reflex-log.jsonl"
export REFLEX_LOG_FILE="$LOGC"

PASS=0; FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

run_hook() {  # $1=input_json  resto=env
  local input="$1"; shift
  printf '%s' "$input" | env "$@" "$HOOK" 2>/dev/null
}

# `tr -d '\r'` en AMBOS lados (review adversarial 2026-09-19): jq en
# Windows/Git Bash emite CRLF (mismo hallazgo que `scripts/test-hooks-json.sh:14-16`).
# Sin esto, un golden capturado en Linux nunca compararía en verde contra la
# salida real de windows-latest aunque el contenido sea idéntico.
guarda_o_compara() {  # $1=nombre $2=salida
  local nombre="$1" out golden="$GOLD_DIR/$1.txt"
  out="$(printf '%s' "$2" | tr -d '\r')"
  if [ "${RECAPTURA:-0}" = "1" ]; then
    mkdir -p "$GOLD_DIR"
    printf '%s' "$out" > "$golden"
    pass "golden: $nombre capturado en $golden"
    return
  fi
  if [ ! -f "$golden" ]; then
    fail "golden: $nombre" "no existe $golden — corre con RECAPTURA=1 antes de comparar"
    return
  fi
  if [ "$out" = "$(tr -d '\r' < "$golden")" ]; then
    pass "golden: $nombre sin cambios"
  else
    fail "golden: $nombre" "difiere de $golden"
  fi
}

# --- Escenario 1: camino feliz (STUB_FELIZ de test-exo-recall.sh) ---
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  --version) echo "exo 9.0.0" ;;
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"
touch "$TMP/index-feliz.db"
: > "$LOGC"
OUT1="$(run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db" ENGINE_MIN=0.1.0)"
guarda_o_compara camino-feliz "$OUT1"

# --- Escenario 2: reafirmación tras compact (ejercita SOURCE/SID, lo que ---
# --- toca esta task) — mismo fixture de log que test-exo-recall.sh:25-39 ---
{
  printf '{"ts":"t","reflex":"git-c","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"viejo"}\n'
  awk 'BEGIN { for (i = 0; i < 2498; i++) printf "{\"ts\":\"t\",\"reflex\":\"zero-residuo\",\"session_id\":\"otra-%d\",\"agent_id\":\"\",\"agent_type\":\"\",\"tool\":\"Bash\",\"payload\":\"x\"}\n", i }'
  printf '{"ts":"t","reflex":"verify-before-commit","session_id":"sess-x","agent_id":"","agent_type":"","tool":"Bash","payload":"nuevo"}\n'
} > "$LOGC"
OUT2="$(run_hook '{"session_id":"sess-x","source":"compact"}' EXO_BIN="$TMP/no-existe")"
guarda_o_compara reafirma-compact "$OUT2"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

Marca ejecutable y captura:

```bash
chmod +x plugins/exo/scripts/test-exo-recall-golden.sh
RECAPTURA=1 bash plugins/exo/scripts/test-exo-recall-golden.sh
```

Expected: `2 passed, 0 failed`, con
`plugins/exo/scripts/testdata/golden-exo-recall/{camino-feliz,reafirma-compact}.txt`
creados.

Run de verificación: `bash plugins/exo/scripts/test-exo-recall-golden.sh`
Expected: `2 passed, 0 failed`.

- [ ] **Step 1 bis: test que falla — el separador de campos debe ser `\x1f`, no tab**

Añade a `plugins/exo/scripts/test-exo-recall.sh`, después de los helpers
`ultimo_evento`/`ultimo_payload` (línea ~42 de este worktree) y antes del
caso `no-engine`:

```bash
# ------------------- Campaña I: SOURCE/SID no se desalinean -----------------
# `read` con IFS=tab trata el tab como whitespace de IFS y COLAPSA un campo
# vacío inicial en vez de respetarlo como delimitador -- con `source`
# ausente (el caso normal), el valor de session_id se cuela en SOURCE y SID
# queda vacío. Se prueba en AISLADO (no a través del hook completo): el uso
# real de estas dos variables en exo-recall.sh exige SOURCE=="compact" Y SID
# no vacío A LA VEZ, y el mismo desplazamiento que ensucia una las dos
# también vacía la otra -- el bug queda enmascarado en el comportamiento
# visible del hook para CUALQUIER prompt real, así que probarlo a través del
# hook nunca lo detectaría. Esto prueba la TÉCNICA (jq + read), no el hook.
JSON_SIN_SOURCE='{"session_id":"sess-solo-id"}'

# Contraprueba: la forma naif (@tsv + IFS=tab) SÍ desalinea -- documenta por
# qué esta task no la usa. Si esta contraprueba deja de fallar, revisar el
# razonamiento de este fix antes de tocar nada más.
SOURCE_TAB=""; SID_TAB=""
IFS=$'\t' read -r SOURCE_TAB SID_TAB <<< "$(printf '%s' "$JSON_SIN_SOURCE" | jq -r '[(.source // ""), (.session_id // "")] | @tsv')"
if [ "$SOURCE_TAB" = "sess-solo-id" ] && [ "$SID_TAB" = "" ]; then
  pass "contraprueba: @tsv + IFS=tab SÍ desalinea (por eso este hook usa \\x1f, no @tsv)"
else
  fail "contraprueba: @tsv + IFS=tab debería desalinear (si no, revisar el razonamiento de este fix)" \
    "SOURCE_TAB='$SOURCE_TAB' SID_TAB='$SID_TAB'"
fi

# El caso real: con \x1f (unit separator, no es whitespace de IFS) no desalinea.
SOURCE_X1F=""; SID_X1F=""
IFS=$'\x1f' read -r SOURCE_X1F SID_X1F <<< "$(printf '%s' "$JSON_SIN_SOURCE" | jq -r '[(.source // ""), (.session_id // "")] | join("\u001f")')"
if [ "$SOURCE_X1F" = "" ] && [ "$SID_X1F" = "sess-solo-id" ]; then
  pass "campaña I: separador \\x1f no desalinea SOURCE/SID con source ausente"
else
  fail "campaña I: separador \\x1f no desalinea SOURCE/SID con source ausente" \
    "SOURCE_X1F='$SOURCE_X1F' SID_X1F='$SID_X1F'"
fi
```

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: los dos casos nuevos en verde YA en este punto — este Step no
toca `exo-recall.sh` todavía, solo prueba la técnica aislada que el Step 2
va a instalar. Es "rojo antes del fix" en el sentido de que, si alguien
escribiera el Step 2 con `@tsv`/tab (el error real que cometió la primera
versión de este plan), la contraprueba de arriba seguiría en verde pero el
caso `\x1f` real fallaría — este test existe precisamente para que ESE error
no pueda colarse sin que algo se ponga en rojo.

- [ ] **Step 2: Fundir `SOURCE`/`SID` en una pasada de jq**

`Edit` sobre `plugins/exo/scripts/exo-recall.sh`:

old_string:
```bash
SOURCE="$(printf '%s' "$INPUT" | jq -r '.source // empty' 2>/dev/null)" || SOURCE=""
SID="$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)" || SID=""
```

new_string:
```bash
# SOURCE y SID en una sola pasada de jq (campaña I; antes eran dos jq sobre
# el mismo $INPUT). Separador `\x1f` (unit separator), NO `@tsv`/tab: `read`
# trata el tab como whitespace de IFS y colapsa un campo vacío inicial (el
# caso normal de `source`, ausente en casi todo prompt) -- verificado que
# `@tsv` + `IFS=tab` desalinea SOURCE/SID en ese caso, `\x1f` no.
SOURCE=""; SID=""
IFS=$'\x1f' read -r SOURCE SID <<< "$(printf '%s' "$INPUT" | jq -r '[(.source // ""), (.session_id // "")] | join("\u001f")' 2>/dev/null)"
```

- [ ] **Step 3: Verlo verde — suite completa + golden**

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: `0 failed`.

Run: `bash plugins/exo/scripts/test-exo-recall-golden.sh`
Expected: `2 passed, 0 failed`, sin `RECAPTURA=1`.

Run adicional (bash 3.2 real, Global Constraints):
```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 \
  sh -c "apk add --no-cache jq >/dev/null 2>&1 && ./test-exo-recall.sh && ./test-exo-recall-golden.sh"
```
Expected: `0 failed` en las dos suites.

- [ ] **Step 4: Commit**

```bash
git add plugins/exo/scripts/exo-recall.sh \
        plugins/exo/scripts/test-exo-recall.sh \
        plugins/exo/scripts/test-exo-recall-golden.sh \
        plugins/exo/scripts/testdata/golden-exo-recall
git commit -m "fix(i, exo-recall): funde SOURCE/SID con separador \\x1f (no @tsv/tab -- desalinea con source vacio) -- golden de equivalencia"
```

---

### Task 4: `PreToolUse:Bash` — pre-filtro bash antes de `jq` en los tres guards

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`bash plugins/exo/scripts/test-git-c-bash.sh`,
`bash plugins/exo/scripts/test-git-add-all-guard.sh` y
`bash plugins/exo/scripts/test-verify-before-commit.sh` verdes, con el caso
nuevo de cada suite visto en rojo antes del cambio.

**Evidencia:** `git-c-bash.sh`, `git-add-all-guard.sh` y
`verify-before-commit.sh` corren en **cada** `Bash` tool call (matcher
`"Bash"` de `hooks.json:13-29`) y cada uno hace `INPUT="$(cat)"` seguido de
`CMD="$(... | jq -r '.tool_input.command // empty')"` — un spawn de `jq`
POR GUARD, incluso cuando el comando no tiene absolutamente nada que ver
con git (`ls`, `npm test`, `cat foo.md`...). Los tres patrones de detección
(`REWRITE_RE`/warn de `git-c-bash.sh`, `PATRON` de `git-add-all-guard.sh`,
`PATRON` de `verify-before-commit.sh`) exigen todos la palabra literal
`git`, así que si `"git"` no aparece en ningún sitio del JSON crudo de
entrada, ninguno de los tres puede disparar — el spawn de `jq` es
descartable sin perder ni un caso.

**Files:**
- Modify: `plugins/exo/scripts/git-c-bash.sh`
- Modify: `plugins/exo/scripts/git-add-all-guard.sh`
- Modify: `plugins/exo/scripts/verify-before-commit.sh`
- Test: `plugins/exo/scripts/test-git-c-bash.sh`
- Test: `plugins/exo/scripts/test-git-add-all-guard.sh`
- Test: `plugins/exo/scripts/test-verify-before-commit.sh`

**Interfaces:** ninguna hacia otras tasks (Task 5, si se ejecuta, sustituye
estos tres ficheros por uno, pero consume el MISMO patrón de pre-filtro,
documentado aquí).

Nota sobre por qué esta task no lleva golden de bloque: los tres guards no
componen un bloque de texto libre como `recall-inject.sh`/`exo-recall.sh` —
cada caso posible ya tiene una aserción exacta de exit-code/stdout en su
suite (`assert_rewrite`, `assert_silent`, casos numerados de
`verify-before-commit`). Esas suites completas, corriendo en verde
antes y después sin tocar ni una aserción existente, SON el equivalente al
golden byte a byte para este tipo de script — más un test nuevo por fichero
que prueba en rojo→verde el ahorro de spawn en sí.

- [ ] **Step 1: Test que falla — `git-c-bash.sh`**

Añade al final de `plugins/exo/scripts/test-git-c-bash.sh`, antes del bloque
`echo ""` / `TOTAL=$((PASS+FAIL))` de cierre:

```bash
# --- Campaña I: sin "git" en el JSON crudo, el pre-filtro evita el spawn de jq ---
{
  POISON_DIR="$(mktemp -d)"
  JQ_MARK="$(mktemp -u)"
  cat > "$POISON_DIR/jq" <<EOF
#!/usr/bin/env bash
touch "$JQ_MARK"
exit 1
EOF
  chmod +x "$POISON_DIR/jq"
  rm -f "$JQ_MARK"
  PAYLOAD_SIN_GIT='{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"ls -la /tmp","description":"desc-original","timeout":5000},"hook_event_name":"PreToolUse"}'
  OUTPUT="$(printf '%s' "$PAYLOAD_SIN_GIT" | PATH="$POISON_DIR:$PATH" bash "$HOOK" 2>/dev/null)"
  EC=$?
  if [ -z "$OUTPUT" ] && [ "$EC" -eq 0 ] && [ ! -f "$JQ_MARK" ]; then
    printf '[PASS] campaña I: comando sin "git" no invoca jq (marca ausente)\n'; PASS=$((PASS+1))
  else
    MARCA="ausente"; [ -f "$JQ_MARK" ] && MARCA="presente"
    printf '[FAIL] campaña I: comando sin "git" no invoca jq — ec=%d out=%s marca=%s\n' "$EC" "$OUTPUT" "$MARCA"
    FAIL=$((FAIL+1))
  fi
  rm -rf "$POISON_DIR"
}
```

Run: `bash plugins/exo/scripts/test-git-c-bash.sh`
Expected: `[FAIL] campaña I: comando sin "git" no invoca jq — ... marca=presente`
(hoy el script SIEMPRE llama a `jq` para extraer `tool_input.command`, sin
mirar antes si "git" aparece).

Repite el MISMO bloque (adaptado al nombre del `HOOK` de cada fichero) en
`test-git-add-all-guard.sh` y `test-verify-before-commit.sh`:

Para `test-git-add-all-guard.sh`, el payload de prueba es
`'{"session_id":"test-sid","tool_name":"Bash","tool_input":{"command":"ls -la /tmp"},"hook_event_name":"PreToolUse"}'`
(sin `description`, que es el formato real de `make_payload` en ese
fichero).

Para `test-verify-before-commit.sh`, igual, con el mismo payload mínimo sin
`description`.

Run de los tres: mismo resultado — `[FAIL] ... marca=presente` en los tres.

- [ ] **Step 2: Implementación — pre-filtro idéntico en los tres scripts**

`Edit` sobre `plugins/exo/scripts/git-c-bash.sh`:

old_string:
```bash
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# ---- Rama 1: REWRITE silencioso (HECHO parseado, alta confianza) ----
```

new_string:
```bash
set -uo pipefail

INPUT="$(cat)"

# Pre-filtro bash puro (campaña I): los tres patrones de detección de este
# reflejo exigen "git" literal en tool_input.command. Si "git" no aparece EN
# NINGÚN SITIO del JSON crudo de entrada (que incluye tool_input.command
# como substring textual), el comando no puede contener ninguno de esos
# patrones -- se ahorra el spawn de jq que solo serviría para descartarlo.
# Si "git" aparece en OTRO campo del JSON (cwd, description...) el filtro
# simplemente no descarta y se sigue el camino de siempre: nunca produce un
# falso NEGATIVO de disparo, como mucho pierde una oportunidad de ahorro.
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# ---- Rama 1: REWRITE silencioso (HECHO parseado, alta confianza) ----
```

`Edit` sobre `plugins/exo/scripts/git-add-all-guard.sh`:

old_string:
```bash
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
```

new_string:
```bash
set -uo pipefail

INPUT="$(cat)"

# Pre-filtro bash puro (campaña I): el PATRON de este reflejo exige "git"
# literal. Mismo contrato que git-c-bash.sh: sin "git" en el JSON crudo, se
# ahorra el spawn de jq sin poder perder ningún disparo real.
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# Patron: git add seguido de -A, --all, o . (con espacio o fin de string tras el argumento).
```

`Edit` sobre `plugins/exo/scripts/verify-before-commit.sh`:

old_string:
```bash
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# 1. Solo actuar en git commit.
```

new_string:
```bash
set -uo pipefail

INPUT="$(cat)"

# Pre-filtro bash puro (campaña I): el PATRON de este reflejo exige "git
# commit" literal. Mismo contrato que git-c-bash.sh: sin "git" en el JSON
# crudo, se ahorra el spawn de jq sin poder perder ningún disparo real.
case "$INPUT" in
  *git*) : ;;
  *) exit 0 ;;
esac

command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# 1. Solo actuar en git commit.
```

- [ ] **Step 3: Verlo verde**

Run las tres suites:
```bash
bash plugins/exo/scripts/test-git-c-bash.sh
bash plugins/exo/scripts/test-git-add-all-guard.sh
bash plugins/exo/scripts/test-verify-before-commit.sh
```
Expected: las tres en `0` fallos, INCLUIDO el caso nuevo de la campaña I
(marca ausente ahora sí, porque el `exit 0` del pre-filtro corta antes de
que `command -v jq`/la extracción lleguen a ejecutarse).

Run adicional (bash 3.2 real, Global Constraints — `git` también hace
falta aquí, `verify-before-commit.sh` lo invoca de verdad):
```bash
docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2 sh -c \
  "apk add --no-cache jq git >/dev/null 2>&1 && \
   ./test-git-c-bash.sh && ./test-git-add-all-guard.sh && ./test-verify-before-commit.sh"
```
Expected: las tres en `0` fallos.

- [ ] **Step 4: Commit**

```bash
git add plugins/exo/scripts/git-c-bash.sh plugins/exo/scripts/git-add-all-guard.sh \
        plugins/exo/scripts/verify-before-commit.sh \
        plugins/exo/scripts/test-git-c-bash.sh plugins/exo/scripts/test-git-add-all-guard.sh \
        plugins/exo/scripts/test-verify-before-commit.sh
git commit -m "fix(i, pretooluse-bash): pre-filtro bash antes de jq en los tres guards de Bash -- sin 'git' en el JSON crudo, cero spawns"
```

---

### Task 5 (OPCIONAL): fundir los tres guards de `PreToolUse:Bash` en uno solo

**Lane:** mecánica. **Depende de:** Task 4. **Criterio numérico de
ejecución** (decide la Task 6/7, no se ejecuta a ciegas):

> Se ejecuta la Task 5 **si y solo si**, tras la Task 4, medir el triple
> `PreToolUse:Bash` completo (los tres scripts, uno detrás de otro, sobre un
> `tool_input.command` que SÍ contiene "git" — el caso donde el pre-filtro
> de la Task 4 ya no ahorra nada) sigue costando **> 200 ms** de reloj de
> pared en W11. El número no es arbitrario: es el que el propio ítem de
> backlog `docs/backlog.md:736-761` fija como umbral de fusión («Si el
> `PreToolUse:Bash` triple supera ~200 ms, fusionar los tres scripts en
> uno»). Si el triple mide ≤200 ms tras la Task 4, esta task se documenta
> como «no ejecutada — criterio numérico no alcanzado» en la Task 8 y no se
> toca ni `hooks.json` ni los tres scripts.

Comando de medición (para decidir, se corre en la Task 6/7 junto al resto):

```bash
PAYLOAD='{"session_id":"bench","tool_name":"Bash","tool_input":{"command":"git status"},"hook_event_name":"PreToolUse"}'
TOTAL_NS_INICIO=$(date +%s%N)
for h in git-c-bash.sh git-add-all-guard.sh verify-before-commit.sh; do
  printf '%s' "$PAYLOAD" | bash "plugins/exo/scripts/$h" > /dev/null 2>&1
done
TOTAL_NS_FIN=$(date +%s%N)
echo "PreToolUse:Bash triple (git status): $(( (TOTAL_NS_FIN - TOTAL_NS_INICIO) / 1000000 )) ms"
```

**Si el criterio se cumple**, esta es la implementación (Files/Steps
completos, para no dejar la task a medias si se activa):

**Files:**
- Create: `plugins/exo/scripts/bash-guards.sh` (funde la lógica de los tres,
  UN SOLO `cat`/pre-filtro/`jq` de extracción de `tool_input.command`,
  seguido de las tres ramas de detección en secuencia — cada una conserva su
  propio `PATRON`/`REWRITE_RE` y su propio evento de log, solo se comparte
  el parseo de entrada).
- Create: `plugins/exo/scripts/test-bash-guards.sh` (fusiona los casos de
  las tres suites existentes contra el script único).
- Modify: `plugins/exo/hooks/hooks.json` (el matcher `"Bash"` pasa de tres
  `"command"` a uno).
- No se borran `git-c-bash.sh`, `git-add-all-guard.sh`,
  `verify-before-commit.sh` ni sus tests en esta task — quedar huérfanos de
  `hooks.json` pero seguir corriendo en `scripts/test-plugin.sh` es
  aceptable como red de regresión hasta un commit posterior de limpieza,
  fuera de esta campaña (evita que esta task, ya opcional, crezca a "borrar
  código en uso por otra campaña que pueda estar en curso").

**Interfaces:**
- Produces: `plugins/exo/scripts/bash-guards.sh`, invocado una vez por
  `Bash` tool call en vez de tres.

Dado que su ejecución depende de un número que solo se conoce en la Task
6/7, esta task se deja **especificada pero no debe ejecutarse antes de
tener esa medición** — el orquestador la salta con el motivo «criterio
numérico no alcanzado (X ms ≤ 200 ms)» si corresponde, y ese motivo es lo
que la Task 8 sincroniza al backlog.

---

### Task 6: Leer el instrumento en Linux (fábrica)

**Lane:** medición. **Depende de:** Tasks 1-4 (y 5, si se ejecutó)
mergeadas. **Oráculo:** ninguno propio — esta task LEE el harness que la
campaña A ya construyó (`evals/recall-coste/harness/bench.sh`,
`compara.sh`), no escribe uno nuevo (directiva «construir antes que medir»
del §5 de la propuesta madre).

**Evidencia:** `evals/recall-coste/results/despues/` (campaña A, Task 14,
2026-09-13, commit `41e01bf`) ya tiene un `resumen.tsv` con los escenarios
`s6-hook-entero` y `s7-config-jq` para `N ∈ {174, 1000, 5000}`, medidos
sobre el `recall-inject.sh` de justo ANTES de esta campaña — YA incluye la
instrumentación de H2/H3 (`elapsed_ms`/`refresh_ms`/avisos, commit `8f41e99`)
que Task 1/2 de esta campaña modifican, así que es la referencia correcta
para atribuir la ganancia A LA CAMPAÑA I específicamente. `baseline/` es
ANTERIOR también a esa instrumentación de H2/H3 (campaña A, línea de base
`3c1918f`): comparar contra `baseline` mezclaría en el mismo número lo que
ganó A (menos, en este caso: H2/H3 AÑADIÓ un `tr` más) con lo que gana I,
y el commit/backlog de esta campaña citaría una cifra que no es
exclusivamente suya. Ningún commit de esta campaña toca `engine/src`, así
que los escenarios que solo ejercen el binario (`s1`-`s5`, `s8`-`s10`) no
deberían cambiar frente a `despues` — el único movimiento esperado es en
`s6`.

**Files:**
- Create: `evals/recall-coste/results/campana-i-<fecha>/**` (vía `bench.sh`,
  no a mano)

- [ ] **Step 1: Correr el bench (puede tardar — compila release y genera
  KB sintética para tres tamaños)**

```bash
ETIQUETA="campana-i-$(date +%F)"
evals/recall-coste/harness/bench.sh "$ETIQUETA"
```

Expected: exit 0, y `evals/recall-coste/results/$ETIQUETA/resumen.tsv`
creado con las mismas filas que `despues/resumen.tsv`.

- [ ] **Step 2: Comparar contra `despues` (campaña A, NO `baseline`)**

`despues` es la referencia correcta para aislar la ganancia de ESTA
campaña (ver «Evidencia» arriba) — `baseline` mezclaría también el cambio
de H2/H3 de la propia campaña A.

```bash
evals/recall-coste/harness/compara.sh despues "$ETIQUETA" \
  | tee "evals/recall-coste/results/${ETIQUETA}/comparacion-vs-despues.md"
```

Expected: la tabla `| id | p50 antes | p50 después | ... |` muestra
`s6-hook-entero-n174`/`n1000`/`n5000` con `p50 después` **menor** que `p50
antes`, y `s7-config-jq-n174` sin cambio material (esta campaña no toca la
extracción de `EXO_KB_NAME` en el camino de `s7`, que usa `exo config --json
| jq` directo, sin pasar por `recall-inject.sh`). `C-noregresión` da `PASA`
(ninguna otra fila se movió, porque el binario no cambió).

- [ ] **Step 3: Medir el triple `PreToolUse:Bash` (criterio de la Task 5)**

```bash
PAYLOAD='{"session_id":"bench","tool_name":"Bash","tool_input":{"command":"git status"},"hook_event_name":"PreToolUse"}'
TOTAL_NS_INICIO=$(date +%s%N)
for h in git-c-bash.sh git-add-all-guard.sh verify-before-commit.sh; do
  printf '%s' "$PAYLOAD" | bash "plugins/exo/scripts/$h" > /dev/null 2>&1
done
TOTAL_NS_FIN=$(date +%s%N)
echo "PreToolUse:Bash triple (git status), Linux: $(( (TOTAL_NS_FIN - TOTAL_NS_INICIO) / 1000000 )) ms" \
  | tee -a "evals/recall-coste/results/${ETIQUETA}/comparacion-vs-despues.md"
```

Este número es de Linux — el criterio de la Task 5 (§Task 5) se decide con
el de W11 (Task 7), no con este. Se registra igual porque es gratis
obtenerlo aquí y da una cota inferior (Linux nunca es más lento que W11 en
spawns de shell).

- [ ] **Step 4: Commit**

```bash
git add "evals/recall-coste/results/${ETIQUETA}"
git commit -m "docs(i, medicion): lee el instrumento de A en Linux tras la campaña I -- comparado contra 'despues' (no 'baseline') para atribuir la ganancia a I; s6 baja, s7 y el resto sin regresión"
```

---

### Task 7 (PAUL-STEP): Leer el instrumento en W11

> **Esta task la ejecuta Paul en su máquina Windows. Ningún ejecutor
> automático la corre.** La fábrica prepara el comando exacto y el nombre
> del fichero de resultados; Paul lo pega en su Git Bash, guarda la salida,
> y se la entrega a la sesión para que la Task 8 la cite.

**Lane:** medición (Paul). **Depende de:** Tasks 1-4 (y 5, si se ejecutó)
mergeadas e instaladas en la máquina de Paul (plugin actualizado). **Sin
oráculo automático** — es una lectura manual, igual que la Task 15 del
pre-registro de A (`docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`
§«W11»), de la que este Step reutiliza el bloque probado, con dos añadidos:
el `hook_ms` de Task 1 (que ahora sí viaja en el log) y la medición del
triple `PreToolUse:Bash`.

**Instrucciones para Paul** (copiar y pegar tal cual en Git Bash, con el
plugin de esta campaña ya instalado — `~/.claude/plugins/cache/exo/exo/<version>`
apuntando al árbol con las Tasks 1-4 mergeadas):

```bash
TMP="$(mktemp -d)"; cp ~/.exo/index.db "$TMP/index.db"
jq -n '{prompt:"como funciona el trinquete de techos", session_id:"bench-w11-campana-i"}' > "$TMP/prompt.json"
H="${EXO_PLUGIN_SCRIPTS:?exporta EXO_PLUGIN_SCRIPTS con la ruta a scripts/ del plugin instalado (NO plugins/exo/scripts -- en la caché instalada es <installPath>/scripts, sin el prefijo plugins/exo; lo aprendió la Task 15 de A a la mala)}"

FECHA="$(date +%F)"
SALIDA="evals/recall-coste/results/w11-${FECHA}-campana-i.txt"

{
  echo "exo_version: $(exo --version 2>/dev/null)"
  echo "fecha: $(date -Iseconds)"
  echo "uname: $(uname -a)"

  for i in $(seq 20); do
    s=$(date +%s%N)
    EXO_INDEX="$TMP/index.db" REFLEX_LOG_FILE="$TMP/log.jsonl" "$H/recall-inject.sh" < "$TMP/prompt.json" > /dev/null
    e=$(date +%s%N); echo $(( (e - s) / 1000000 ))
  done | sort -n | awk '{a[NR]=$1} END{print "hook_wall_p50_ms", a[int((NR-1)*0.5)+1], "hook_wall_p95_ms", a[int((NR-1)*0.95)+1]}'

  echo "--- payloads emitidos (con hook_ms, campaña I) ---"
  jq -r 'select(.reflex=="recall-inject-emitted") | .payload' "$TMP/log.jsonl" | tail -3

  echo "--- veredicto de recall-latencia.sh sobre ESTA ventana (decide por hook_ms, decisión #12) ---"
  REFLEX_LOG_FILE="$TMP/log.jsonl" bash "$H/recall-latencia.sh"

  echo "--- PreToolUse:Bash triple (git status), W11 ---"
  PAYLOAD='{"session_id":"bench","tool_name":"Bash","tool_input":{"command":"git status"},"hook_event_name":"PreToolUse"}'
  TOTAL_NS_INICIO=$(date +%s%N)
  for h in git-c-bash.sh git-add-all-guard.sh verify-before-commit.sh; do
    printf '%s' "$PAYLOAD" | bash "$H/$h" > /dev/null 2>&1
  done
  TOTAL_NS_FIN=$(date +%s%N)
  echo "PreToolUse_Bash_triple_ms $(( (TOTAL_NS_FIN - TOTAL_NS_INICIO) / 1000000 ))"
} | tee "$SALIDA"

rm -rf "$TMP"
echo "Guardado en: $SALIDA -- pégalo/commitéalo y avisa a la sesión con el contenido."
```

**Qué mirar en el resultado** (para que Paul sepa si algo salió mal antes de
avisar): `hook_wall_p95_ms` debería bajar sustancialmente frente al p95
2.568 ms de `evals/recall-coste/results/w11-2026-09-15.txt` (la campaña
existe para eso). El veredicto de `recall-latencia.sh` con solo 20 disparos
sintéticos dirá `INSUFICIENTE (menos de 200 disparos)` — eso es CORRECTO y
esperado (el mínimo de 200 disparos de la decisión #12 no lo cubre una
corrida manual de 20; el veredicto real se lee semanas después, sobre el
`~/.claude/reflex-log.jsonl` de producción). Lo que importa de esta corrida
es `hook_wall_p95_ms` y `PreToolUse_Bash_triple_ms`, no el veredicto del
script.

**Al recibir el fichero:** este resultado alimenta la Task 8 (cifras
citadas en el backlog) y decide el criterio numérico de la Task 5
(§Task 5 — el número relevante es `PreToolUse_Bash_triple_ms` de ESTE
fichero, no el de Linux de la Task 6).

- [ ] **PAUL-STEP: correr el bloque de arriba y entregar
  `evals/recall-coste/results/w11-<fecha>-campana-i.txt`**

---

### Task 8: Sync de `docs/backlog.md`

**Lane:** documental. **Depende de:** Tasks 1-4 (y 5 si aplica), 6 y **7**
(necesita el número de W11 para cerrar el ítem con evidencia real — no se
cierra con solo el número de Linux). **Oráculo:** ningún ítem se cierra sin
cita a un commit real de esta campaña.

**Files:**
- Modify: `docs/backlog.md` (dos ítems existentes, localizados por texto —
  las líneas ya están desfasadas hoy mismo respecto a la propuesta madre,
  así que ninguna edición de esta task usa números de línea).

Antes de editar, releer el estado actual de los dos ítems con
`grep -n "El coste del hook completo en Windows\|Proceso residente para el coste fijo del recall" docs/backlog.md`
para confirmar que el texto de `old_string` sigue siendo literal (si la
campaña L, en paralelo, ya editó otra parte del fichero, esto NO afecta a
estos dos ítems — pero si alguien más los tocó entretanto, hay que releer y
adaptar el `old_string` a lo que haya de verdad, nunca forzar con
`overwrite_unread`).

- [ ] **Step 1: Cerrar «El coste del hook completo en Windows no está medido»**

`Edit` sobre `docs/backlog.md`, con `old_string` el bloque que termina en
(usar el texto real del fichero en el momento de ejecutar, verificado en
este plan contra `docs/backlog.md:736-761` de `HEAD` `5efe812`):

```
  **(W11, 2026-09-15, Task 15 de A):** hook `recall-inject.sh` entero p50 =
  2312 ms, p95 = 2568 ms; config+jq p50 = 68 ms (C-H10 CERRADA, sin fusión de
  llamadas). **En W11 el shell sí pesa:** `exo recall` en caliente ≈1,1 s y
  el ≈1,2 s restante son ≈20 spawns de Git Bash a 25-60 ms cada uno (`jq -n
  1` ≈55 ms, `exo --version` ≈60 ms). Queda sin medir el `PreToolUse:Bash`
  triple. Evidencia: `evals/recall-coste/results/w11-2026-09-15.txt`.
```

new_string (añade el cierre, sin borrar el histórico de arriba):

```
  **(W11, 2026-09-15, Task 15 de A):** hook `recall-inject.sh` entero p50 =
  2312 ms, p95 = 2568 ms; config+jq p50 = 68 ms (C-H10 CERRADA, sin fusión de
  llamadas). **En W11 el shell sí pesa:** `exo recall` en caliente ≈1,1 s y
  el ≈1,2 s restante son ≈20 spawns de Git Bash a 25-60 ms cada uno (`jq -n
  1` ≈55 ms, `exo --version` ≈60 ms). Queda sin medir el `PreToolUse:Bash`
  triple. Evidencia: `evals/recall-coste/results/w11-2026-09-15.txt`.
  **(campaña I, 2026-09-19, CERRADO):** `recall-inject.sh` pasó de 7 `jq` +
  2 `sed` + 5 `tr` (más 2 `sed` + 1 `tr` POR TOKEN dentro del gate léxico) a
  6 `jq` + 0 `sed` + 0 `tr`, todos fuera del bucle léxico. Los tres
  `PreToolUse:Bash` (`git-c-bash.sh`, `git-add-all-guard.sh`,
  `verify-before-commit.sh`) ganaron un pre-filtro bash que evita el spawn
  de `jq` cuando el comando no contiene "git". Medido: W11 hook p95 pasó de
  2.568 ms a [RELLENAR con `hook_wall_p95_ms` del fichero
  `evals/recall-coste/results/w11-<fecha>-campana-i.txt` de la Task 7];
  Linux (`evals/recall-coste/harness/compara.sh despues campana-i-<fecha>` —
  contra `despues`, no `baseline`, para atribuir la ganancia a esta campaña)
  sin regresión en ningún escenario ajeno a `s6`. `PreToolUse:Bash` triple en
  W11: [RELLENAR con `PreToolUse_Bash_triple_ms`] — [si >200 ms: "por encima
  del umbral de fusión del propio ítem; ver Task 5, ejecutada en el commit
  [SHA]" / si ≤200 ms: "por debajo del umbral de fusión (~200 ms); Task 5 no
  se ejecutó"]. Bloque inyectado verificado byte-idéntico antes/después
  (goldens en `plugins/exo/scripts/testdata/golden-{recall-inject,exo-recall}/`).
  Commits: [listar los SHA reales de las Tasks 1-4/5 de este plan, con
  `git log --oneline` sobre la rama mergeada].
```

- [ ] **Step 2: Cerrar el sub-ítem W11 de «Proceso residente para el coste
  fijo del recall por prompt»**

`Edit` sobre `docs/backlog.md`, con `old_string` el párrafo (verificado
contra `docs/backlog.md:1237-1266` de `HEAD` `5efe812`):

```
  **(W11, 2026-09-15) El instrumento de reapertura no ve la mitad del coste
  en Windows.** `recall-latencia.sh` suma `elapsed_ms + refresh_ms` del
  payload, es decir, el tiempo **interno** del engine: en W11 son ≈1,0 s de
  un hook que tarda ≈2,3 s de reloj (p95 2568 ms). Con el umbral de 1.500 ms
  no dispararía nunca en W11, aunque allí cada prompt paga más que en Linux.
  **Acción:** que `recall-inject.sh` registre también la duración de reloj
  del hook (p. ej. `hook_ms` desde `$EPOCHREALTIME`, sin spawn) y que el
  criterio la use. Cambiar el umbral o la métrica es tocar el pre-registro de
  A: hay que decidirlo antes de mirar los datos de la ventana. Evidencia:
  `evals/recall-coste/results/w11-2026-09-15.txt`.
```

new_string:

```
  **(W11, 2026-09-15) El instrumento de reapertura no ve la mitad del coste
  en Windows.** `recall-latencia.sh` suma `elapsed_ms + refresh_ms` del
  payload, es decir, el tiempo **interno** del engine: en W11 son ≈1,0 s de
  un hook que tarda ≈2,3 s de reloj (p95 2568 ms). Con el umbral de 1.500 ms
  no dispararía nunca en W11, aunque allí cada prompt paga más que en Linux.
  **Acción:** que `recall-inject.sh` registre también la duración de reloj
  del hook (p. ej. `hook_ms` desde `$EPOCHREALTIME`, sin spawn) y que el
  criterio la use. Cambiar el umbral o la métrica es tocar el pre-registro de
  A: hay que decidirlo antes de mirar los datos de la ventana. Evidencia:
  `evals/recall-coste/results/w11-2026-09-15.txt`.
  **(campaña I, 2026-09-19, CERRADO):** decisión de Paul #12 —
  `recall-latencia.sh` decide por `hook_ms` (reloj de pared, medido en
  `recall-inject.sh` sin spawn, `NA` en bash <5). Enmienda fechada en
  `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`
  §«Criterio de reapertura». Umbral, porcentaje de timeouts y mínimo de
  disparos sin cambios (1.500 ms / 2% / 200). El instrumento ya ve el coste
  real en W11 desde el commit [SHA de la Task 1].
```

- [ ] **Step 3: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(i, backlog): cierra los dos items de la latencia del hook en W11 con las cifras y commits reales de la campaña"
```

---

## Self-review del skill (cobertura, placeholders, firmas)

- **Cobertura**: los 6 puntos del esbozo original de la propuesta
  (instrumentar, `recall-inject.sh`, `exo-recall.sh`, `PreToolUse:Bash`,
  fusión opcional, medición) tienen task propia (1-4, 5 opcional, 6-7). Se
  añadieron la Task 8 (sync backlog, pedida explícitamente) y el anexo al
  pre-registro de A dentro de la Task 1 (pedido explícitamente por la
  decisión #12).
- **Placeholders**: los únicos `[RELLENAR ...]` y `[SHA ...]` del documento
  están en la Task 8, Steps 1-2 — son literalmente los números y hashes que
  no pueden existir hasta que las Tasks 1-7 se hayan ejecutado y commiteado;
  no son placeholders de pereza, son huecos que la propia task instruye
  rellenar con el comando exacto (`git log --oneline`, leer el fichero de
  resultados) para hacerlo. Todo el resto del plan lleva código completo,
  comandos exactos y output esperado.
- **Firmas/tipos consistentes entre tasks**: `HOOK_MS` (global, entero o
  vacío) y `hook_ms_calcula`/`hook_ms_de`/`hook_ms_soportado` se definen en
  Task 1 y no los reconsume ninguna otra task de este plan (por diseño: el
  ahorro de Task 2 es independiente del cableado de Task 1, solo comparten
  fichero). `TOKEN_NORM` (global de Task 2) no colisiona con nada usado en
  Task 1 ni en el resto del fichero. `SOURCE`/`SID` de Task 3 mantienen los
  mismos nombres que tenían antes del cambio — ningún consumidor externo a
  `exo-recall.sh` los usa.

## Ronda de review adversarial (2026-09-19, aplicada tras la primera versión)

Ocho hallazgos, todos verificados con ejecución real (no solo razonados)
antes de aplicar el fix — `docker run bash:3.2`, shellcheck 0.11.0 real
(descargado y pineado igual que `scripts/test-shellcheck.sh`), y pruebas
directas de `read`/`$EPOCHREALTIME`/`sed` bajo `LC_ALL=es_ES.utf8`:

1. **[Bloq]** `hook_ms_de` mezclaba guard + aritmética; un test que la
   llamaba en bash 3.2 nunca ejercía la cuenta. Fix: `hook_ms_calcula`
   (pura) separada de `hook_ms_de` (guard). `${BASH_VERSINFO[0]}` en vez de
   `$BASH_VERSINFO` a secas (SC2128); `# shellcheck disable=SC2034` donde
   `HOOK_MS` se asigna (la lee el llamador, no este fichero).
2. **[Bloq]** `$EPOCHREALTIME` usa coma decimal bajo un locale con
   `LC_NUMERIC` que la use — verificado real en la máquina de Paul y
   relevante porque Git Bash en Windows hereda el locale regional. Fix:
   normalización de coma en `hook_ms_calcula` + `export LC_NUMERIC=C` en
   `recall-inject.sh` (mismo patrón que `recall-latencia.sh:13`) + caso de
   test con coma.
3. **[Imp]** El fixture `emite()` de `test-recall-latencia.sh` rompía la
   aserción `p95_ms=910` del caso 1 al pasar de `elapsed_ms+refresh_ms` a
   `hook_ms=h` a secas. Fix: `hook_ms = base_ms + refresh_ms` en el
   fixture, preservando las cinco aserciones existentes sin tocarlas.
4. **[Imp]** `IFS=$'\t' read` desalinea SOURCE/SID cuando `source` está
   vacío (tab es whitespace de IFS, colapsa el campo vacío) — verificado
   con ejecución real. Fix: separador `\x1f` + test de aislamiento con
   contraprueba (Step 1 bis de Task 3).
5. **[Imp]** Los goldens comparaban sin normalizar CRLF, que `jq` emite en
   Windows/Git Bash (hallazgo ya conocido del repo,
   `scripts/test-hooks-json.sh:14-16`). Fix: `tr -d '\r'` en ambos lados de
   cada comparación (Tasks 2 y 3).
6. **[Imp]** `compara.sh baseline` mezclaría en el mismo número la ganancia
   de la campaña A (que ya tocó `recall-inject.sh` en H2/H3) con la de esta
   campaña. Fix: Task 6 compara contra `despues`, no `baseline`; Task 8
   actualizada para citar la comparación correcta.
7. **[Menor]** La equivalencia "byte a byte sobre 24 tokens" no se sostiene
   bajo `LC_ALL=es_ES.utf8` para caracteres fuera del alfabeto español
   (`ç`/`ß`/`ï`/`ö`) — verificado con ejecución real lado a lado. Fix:
   afirmación corregida, comportamiento nuevo declarado normativo (Task 2),
   caso `F2b` con `LC_ALL=es_ES.UTF-8` y `SKIP` si el locale no está
   instalado.
8. Añadido a Global Constraints: todo snippet bash nuevo se prueba en
   `docker run --rm -v "$PWD/plugins/exo/scripts":/w -w /w bash:3.2
   ./<test>.sh` (con `apk add --no-cache jq [git]` cuando el test los
   necesite), con el comando exacto repetido en el paso de verificación de
   cada task que toca bash (1, 2, 3, 4).
