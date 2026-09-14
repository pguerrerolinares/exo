# Campaña E — Hooks que no mienten + deuda de coste trivial

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no lo
> edita). Gates adjudicados por un consultor Fable delegado (régimen firmado,
> `.superpowers/fabrica/config.md` §«Ejecución de gates»).

**Goal:** que los eventos que los hooks de exo loguean cuenten lo que de
verdad hicieron (no más, no menos), y que el CI cace barato lo que hoy cuesta
horas de runner o queda mudo cuando algo rompe — sin tocar ni un byte del
motor de retrieval ni del cutover kbx→exo (eso es la campaña D, en paralelo).

**Architecture:** nueve tareas independientes entre sí (salvo la última, que
las cierra todas), agrupables en tres bloques sin dependencias cruzadas: (1)
**hooks honestos** — un evento `inject-empty` quiere que `subagent-inject.sh`
dejar de afirmar que inyectó contenido cuando solo compuso la cabecera
(Task 1), tests que faltaban para `exo-recall.sh` (Task 2), y `exo search`
imprimiendo `no results` en vez de una terminal en blanco (Task 3); (2)
**engine** — el mensaje de la guarda «una DB sirve a una KB» deja de
recomendar un flag que `exo init` no tiene (Task 4) y `engine/rust-toolchain.toml`
fija el canal (Task 5); (3) **CI de coste trivial** — `test-hermetico.sh` gana
`--locked`, log completo y `upload-artifact`, rehaciendo su ciclo rojo-verde
por ser gate ya demostrado falsable (Task 6), `release.yml` valida versiones
ANTES de los tres builds de una hora (Task 7), y un gate nuevo caza rutas de
una máquina concreta en el bash versionado (Task 8). Task 9 sincroniza
`docs/backlog.md`: cierra con evidencia lo que las ocho tareas anteriores
resuelven y caduca con cita los tres ítems de Alta que el diagnóstico del
consultor encontró parcial o totalmente superados.

**Tech Stack:** Rust 2024 (crate `exo` en `engine/`, MSRV 1.95) · `clap`
4.6.2 derive · `tempfile` 3.14 (dev) · bash (Git Bash en Windows) + `jq` ·
GitHub Actions (`.github/workflows/ci.yml`, `release.yml`) · Python 3 solo
como herramienta de verificación del ejecutor (parseo de YAML para
comprobar el grafo de `needs:`; nada de Python entra al repo).

## Correcciones sobre el dictamen del consultor (verificado hoy contra el árbol real)

El dictamen (`/home/paul/.claude/jobs/05ee55a9/tmp/dictamen-consultor-d-e.md`)
se re-verificó línea a línea contra `campana-d-e` en `b2020f7`. Los números de
`docs/backlog.md` citados abajo son los que tiene **ese commit ahora mismo**
(se leyó el fichero de verdad, no se recalculó un offset): no hace falta
re-localizar nada, pero cualquier commit posterior a `b2020f7` que toque
`docs/backlog.md` sí los mueve — ancla por texto si ejecutas esto más tarde.

- **`:298-350` (título dictamen «truncado del bloque de arranque») —
  confirmado, con una precisión**: el dictamen cita `main.rs:917-919`;
  la línea exacta del `eprintln!` del aviso es `919`, el `if` que lo guarda
  es `916`. `exo-recall.sh:89-91` (no 88-90: el `if grep -q 'truncado'...`
  empieza en `89`) ya loguea `recall-fallback reason=truncated`. Acción (a)
  de este ítem está HECHA por `2234887` (recall) y `48083e5` (el fusionado a
  plugin único que trajo `exo-recall.sh` en su forma actual). Su párrafo
  «Cruce (2026-09-09)» proponía unificar esto con el ítem de `inject-emitted`
  vacío en **un solo campo de envelope compartido**: esta campaña NO adopta
  esa unificación (exigiría rediseñar el JSON que sale de `compose-inject.sh`
  hacia `subagent-inject.sh`, superficie que el dictamen pide tratar con
  cautela) — se cierra cada blind spot con su propio evento ad-hoc, más
  simple y sin tocar ningún esquema. Task 9 lo anota así, sin fingir la
  unificación.
- **`:282-286` no es un ítem propio**: es un sub-párrafo dentro del ítem de
  Alta `:243-297` («"exo genérico" sigue siendo el plugin de Paul»).
  Confirmado en código: `git grep -l kb-demo -- plugins/exo/` da **9**
  ficheros hoy (no los 8 que cita el propio backlog en su última medición —
  el noveno es `README.md`, que no estaba cuando se escribió esa línea), y
  **ninguno es lógica de runtime** — `exo-recall.sh:63` y
  `recall-inject.sh:230` son comentarios que EXPLICAN que el nombre ya sale
  de `exo config --json` (`exo-recall.sh:70`, `recall-inject.sh:236`); el
  resto son `kb-precommit.sh` (mención de fixture), `test-recall-inject.sh`
  (fixtures deliberadas, `EXO_KB_NAME="kb-demo"` puesto por el test),
  `test-contrato-engine.sh:22` (comentario), `README.md`, y
  `distill/SKILL.md`/`chequeos.md`/`recon-first/SKILL.md` (nombran la KB real
  del propio Paul por diseño de esas skills). Task 9 caduca SOLO la frase
  real del backlog, «`kb-demo` en 8 ficheros, tres de ellos de producción»
  (`docs/backlog.md:285`, no «dos hooks de producción» — esa es una
  redacción anterior, ya superada dentro del propio backlog) — el resto del
  ítem (`Paul` en 5 ficheros, `kbx` como dependencia de `distill`) sigue
  abierto, fuera del alcance de E.
- **`:373-383` («`exo-recall.sh` no tiene suite») pasa a cerrarse entero**,
  no solo a caducar: la Task 2 de este plan cubre exactamente su Acción
  (guards `no-engine`/`no-config`, su orden, camino feliz).
- **Hallazgo nuevo, no citado por el dictamen**: el gate de rutas personales
  de la Task 8, con el patrón que Paul ya fijó (`/home/<user>`, no nombres),
  da un **falso positivo** en `plugins/exo/scripts/test-contrato-engine.sh:45`
  — el comentario dice `` `/c/Users/...` ``, que contiene la subcadena
  `/Users/...` aunque no sea una ruta personal real. Se reescribe en la misma
  Task 8 (es el único cambio adicional fuera de la lista original, y es
  consecuencia directa de implementar el ítem #8, no dispersión).
- **Hallazgo nuevo**: arreglar `test-git-c-bash.sh:75-76` sustituyendo
  `/home/paul/...` por `/home/<algo-genérico>/...` NO basta — el gate mira la
  FORMA de la ruta (`/home/<cualquier-cosa>`), no si el nombre es real. La
  Task 8 mueve el fixture fuera de `/home` por completo (`/opt/proyectos/...`).
- **`inject-empty`, verificado el consumidor** (`grep -rn inject-emitted`):
  `plugins/exo/scripts/a1-gate.sh` cuenta agentes distintos con
  `inject-emitted`/`inject-failed`/`inject-skipped-depth`/`inject-abstained`
  (`emitidos`, `sesiones`, `denom_u2`, líneas 80-92 y 245-252). Ninguna de
  esas queries menciona un evento nuevo, así que añadir `inject-empty` como
  evento **adicional** (sin dejar de loguear `inject-emitted` cuando el
  compositor sí produjo JSON válido) no cambia ni un contador de `a1-gate.sh`.
  Es la razón de diseño de la Task 1: aditivo, no sustitutivo.
- **Zona de `main.rs` que toca D**: `enum Comando` (líneas `39-85`) y los dos
  `match` exhaustivos que lo despliegan (`433-443` extracción de `.json`,
  `486-497` despacho) — D añadirá variantes `Rotate`/`Stale` ahí y tendrá que
  tocar los tres sitios (Rust exige exhaustividad). **E no toca ninguno de
  los tres**: su único cambio en `main.rs` es el cuerpo de `busca_cmd`
  (`932-980`), función ya existente, sin tocar su firma ni el enum. Ver
  «Dependencias con D» más abajo.

## Global Constraints

Todas verificadas hoy contra `b2020f7` (`campana-d-e`).

- **El crate vive en `engine/`, no en la raíz.** No hay workspace. Todo
  `cargo` con cwd `engine/`.
- **MSRV `rust-version = "1.95"`** (`engine/Cargo.toml:8`), comprobada por el
  job `msrv` de `ci.yml` con `dtolnay/rust-toolchain@1.95.0` +
  `cargo check --all-targets --locked` (`ci.yml:76-91`). Los jobs `lint` y
  `test` usan `dtolnay/rust-toolchain@stable` (`ci.yml:30`, `:103`).
- **Envelope v2, verbatim de `engine/src/envelope.rs:9-16`:** «Emite
  `{"schema_version":2,"command":<command>,"data":<data>}` como una única
  línea JSON, newline-terminada, a **stdout**. […] Lo que SIEMPRE va a stderr,
  con o sin `--json`, son los avisos y el progreso […]. Los consumidores
  gatean por exit code, jamás por campos de `data`.» **Esta campaña no toca
  ninguna clave de `data` ni el schema**: la Task 3 solo añade una línea
  `println!("no results")` en la rama `else` (sin `--json`) de `busca_cmd`
  — la rama `if args.json` queda byte-idéntica.
- **Códigos de salida (`engine/src/main.rs:387-412`):** `3` si el error es un
  `Rechazo`/`GateFallido`; `1` cualquier otro error; `0` éxito; `2` error de
  uso de clap. Esta campaña no cambia ningún exit code existente (`exo search`
  sin resultados sigue siendo `0`, igual que `exo targets` con
  `no candidates`).
- **`engine/scripts/test-hermetico.sh` es gate demostrado falsable
  (2026-08-27, re-demostrado en campaña B).** Task 6 lo modifica — es la
  única excepción a «no tocar un gate ya demostrado» de este plan, y por eso
  rehace su ciclo rojo-verde explícitamente (Step 3 de esa tarea).
- **Tests de CLI:** `std::process::Command` con `env!("CARGO_BIN_EXE_exo")`,
  `tempfile`, `serde_json`. `[dev-dependencies]` = solo `tempfile`; no se
  añaden crates nuevos.
- **Tests del plugin:** `scripts/test-plugin.sh` descubre por glob
  `plugins/exo/scripts/test-*.sh` (excluye `test-contrato-engine.sh`) y los
  invoca directamente (`"./$t"`) en ubuntu/windows/macos — todo bash nuevo
  bajo `plugins/` portable a macOS y Git Bash (sin `timeout`, `date -d`,
  `stat -c`, `touch -d` a pelo).
- **`.gitattributes`: `* text=auto eol=lf`.** Nada de normalizar finales de
  línea a mano.
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push ni de tocar GitHub (línea roja del
  config de fábrica).
- **`$TMPDIR`** en los comandos de este plan es un directorio temporal fuera
  del repo: `export TMPDIR="${TMPDIR:-$(mktemp -d)}"` al abrir la sesión del
  ejecutor. Nada de lo que va ahí se commitea.
- **Fuera de alcance, declarado:** todo lo de la campaña D (rotate/stale,
  paridad Go, `doctor.rs`, `kb-precommit.sh`, `distill/**`,
  `docs/arquitectura.md`, `README.md`); el check de desfase binario↔plugin en
  `exo doctor` (fuera por decisión de Paul); `walk_kb` vs
  `walk_kb_excluyendo` (fuera por decisión de Paul); `buscador.rs`/`trozos.rs`;
  el proceso residente; la evicción de la KB; renombrar `docs/superpowers/`;
  detección de nombres propios en el gate de rutas personales (solo rutas,
  decisión de Paul); `validate-hooks.js`/schema de `hooks.json` (sub-propuesta
  hermana del ítem de rutas personales, no pedida por el dictamen para E);
  detección de errores de COMPILACIÓN en `test-hermetico.sh` (seguirá sin
  casar ningún patrón de grep — el log completo + artifact de Task 6 lo hace
  visible en el artifact, pero no lo resalta; el backlog lo anota como
  parcial).

## Dependencias con D y conflictos de fichero

**Sin bloqueo real**: E y D no comparten ninguna precondición de arranque —
pueden despacharse en paralelo en la misma fábrica. El único fichero que
ambas tocan es `main.rs`, en zonas disjuntas, y `docs/backlog.md`, donde el
orden de merge importa.

| Fichero | D | E (tarea) | Consecuencia |
|---|---|---|---|
| `engine/src/main.rs` | añade variantes `Rotate`/`Stale` a `enum Comando` (`39-85`) y sus dos brazos de `match` (`433-443`, `486-497`); módulos nuevos | Task 3 reescribe el cuerpo de `busca_cmd` (`932-980`), función ya existente | Zonas disjuntas del mismo fichero — sin conflicto de líneas. **Orden de merge: E primero.** Si D mergea primero, la Task 3 de E debe re-anclar su `old_string` de Edit contra el `main.rs` post-D (el cuerpo de `busca_cmd` no se mueve, pero el número de línea absoluto sí) |
| `docs/backlog.md` | cierra `:1405-1413` (cutover), `:1381-1389` (paridad Go), abre/cierra ítems de rotate/stale | Task 9 caduca 3 ítems de Alta y cierra 8 líneas de items (Medio/Alto) | **E va primero** (orden pedido por Paul). D debe rebasar su edición del backlog sobre la versión que deja la Task 9 de E — releer el fichero antes de escribir, no asumir los números de línea de este plan |
| `docs/arquitectura.md`, `README.md` | los toca | E no los toca | Sin conflicto |
| `engine/src/doctor.rs`, `plugins/exo/scripts/kb-precommit.sh`, `plugins/exo/skills/distill/**` | los toca | E no los toca | Sin conflicto |
| `.github/workflows/ci.yml` | no lo toca | Tasks 6 y 8 (jobs distintos: `test` y `lint`) | Ambas de E, mismo fichero — **secuenciales entre sí** (no hace falta gate extra, pero el orquestador no las despache en paralelo) |
| `.github/workflows/release.yml` | no lo toca | Task 7 | Sin conflicto con D |

**Recomendación de orden de merge: E primero, D se rebasa.** Motivo (dado por
Paul): D toca superficie más grande y con más riesgo de tener que adjudicar
divergencias (paridad con Go); que D absorba el rebase es más barato que al
revés.

## Ramas sugeridas

Nueve tareas pequeñas, ocho independientes entre sí y ninguna con una
decisión de alcance que adjudicar (todos los oráculos están escritos en este
plan) — lane **mecánica** para las nueve. Una sola rama basta; no hay
justificación para partir en varias como hizo B (que sí tenía D1-D5
pendientes de Paul).

| Rama | Tareas | Orden sugerido |
|---|---|---|
| `e-hooks-honestos-y-ci` | 1-9 | 4, 5 (engine, sin overlap) · 1, 2 (hooks) · 6, 8 (mismo fichero `ci.yml`, secuenciales) · 3, 7 (sueltas) · 9 (última, cierra todo) |

---

### Task 1: `inject-empty` — el evento no afirma un efecto que no ocurrió

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Oráculo:**
`bash plugins/exo/scripts/test-subagent-inject.sh` y
`bash plugins/exo/scripts/test-compose-inject.sh`, verdes los dos, con el
caso nuevo del Step 1 visto en rojo antes del Step 2.

**Evidencia:** `docs/backlog.md:352-371`. Medido hoy: perfil `reducido`
(`exo:executor`) con `$KB` no resoluble (`exo config --json` falla o no está
en PATH) compone `estado()` → `rutas()` → `[ -n "$KB" ] && [ -d "$KB" ] ||
return 0` → nada. `compone_contenido()` solo imprime la cabecera
`"=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="`
(71 bytes con salto de línea, medido con `printf '%s\n' "..." | wc -c`).
`subagent-inject.sh:41` loguea `inject-emitted` igual, con `bytes=71`
enterrado en el payload — el nombre del evento afirma un efecto (inyectó
contexto) que no ocurrió.

**Diseño (justificación por escrito, ya que el dictamen pedía elegir):**
evento **aditivo** `inject-empty`, logueado ADEMÁS de `inject-emitted` (nunca
en su lugar) cuando el bloque compuesto no supera el tamaño de su propia
cabecera. Se descarta el «campo de envelope compartido» que proponía el
cruce del ítem hermano (ver «Correcciones sobre el dictamen»): cambiaría el
JSON que `subagent-inject.sh` entrega a Claude Code
(`hookSpecificOutput.additionalContext`), superficie que el dictamen pide
tratar con cautela, a cambio de nada que el diseño aditivo no consiga ya.
`a1-gate.sh` no lee `inject-empty`, así que sus contadores
(`emitidos`/`sesiones`/`denom_u2`) no cambian.

**Files:**
- Modify: `plugins/exo/scripts/compose-inject.sh`
- Modify: `plugins/exo/scripts/subagent-inject.sh`
- Test: `plugins/exo/scripts/test-subagent-inject.sh`

**Interfaces:**
- Consumes: `_reflex-log.sh::reflex_log(reflex, input, payload)` (sin cambios
  de firma).
- Produces: nuevo evento `inject-empty` en `reflex-log.jsonl`, mismo formato
  que `inject-emitted` (`reflex`, `session_id`, `agent_id`, `agent_type`,
  `payload="type=$TYPE perfil=$PERFIL bytes=$bytes"`).

- [ ] **Step 1: Test que falla**

Añade al final de `plugins/exo/scripts/test-subagent-inject.sh`, antes del
bloque final `echo ""` / `TOTAL=...` (es decir, entre el Caso 6 y ese
bloque):

```bash
# =========================================================================
# Caso 7: perfil `reducido` (agent_type exo:executor) SIN KB resoluble ⇒
# el bloque compuesto es solo la cabecera (71B) — inject-emitted se loguea
# igual (contrato: el hook nunca deja de responder), pero además debe
# quedar una línea inject-empty distinguiendo "compuse solo cabecera" de
# "compuse contenido real" (docs/backlog.md: "inject-emitted se emite
# aunque no se inyecte nada").
# =========================================================================
{
  LOG7="$TMP/log7.jsonl"
  : > "$LOG7"
  FAKEBIN7="$TMP/fakebin7"
  mkdir -p "$FAKEBIN7"
  # `exo` deliberadamente roto: fuerza a compose-inject.sh a resolver KB=""
  # por la vía real (exo config --json falla), sin depender de si esta
  # máquina tiene exo instalado de verdad.
  cat > "$FAKEBIN7/exo" <<'EOF'
#!/usr/bin/env bash
exit 1
EOF
  chmod +x "$FAKEBIN7/exo"
  PAYLOAD7='{"session_id":"test-sid","agent_id":"aE1","agent_type":"exo:executor","hook_event_name":"SubagentStart","cwd":"/tmp"}'
  OUT7="$(printf '%s' "$PAYLOAD7" | REFLEX_LOG_FILE="$LOG7" REFLEX_PROJECTS_DIR="$NO_PROJECTS" EXO_KB= REFLEX_CANARY_FILE="$TMP/no-canary" PATH="$FAKEBIN7:$PATH" "$ADAPTER")"
  EC7=$?
  CTX7="$(printf '%s' "$OUT7" | jq -r '.hookSpecificOutput.additionalContext // empty' 2>/dev/null)"
  EVENTOS7="$(jq -r '.reflex' "$LOG7" 2>/dev/null | tr '\n' ',')"
  if [ $EC7 -eq 0 ] && [ "$CTX7" = "=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ===" ] \
     && printf '%s' "$EVENTOS7" | grep -q 'inject-emitted' \
     && printf '%s' "$EVENTOS7" | grep -q 'inject-empty'; then
    pass "caso7: perfil reducido sin KB ⇒ solo cabecera ⇒ inject-emitted Y inject-empty"
  else
    fail "caso7: perfil reducido sin KB ⇒ solo cabecera ⇒ inject-emitted Y inject-empty" \
      "ec=$EC7 ctx='$CTX7' eventos=$EVENTOS7"
  fi
}
```

Run: `bash plugins/exo/scripts/test-subagent-inject.sh`
Expected: `[FAIL] caso7: ... — eventos=inject-emitted,` (solo `inject-emitted`,
sin `inject-empty` — el evento nuevo no existe todavía). El resto de casos
(1-6) siguen en verde.

- [ ] **Step 2: Implementación — `compose-inject.sh`**

`Edit` sobre `plugins/exo/scripts/compose-inject.sh`:

old_string:
```bash
compone_contenido() {
  echo "=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="
  case "$PERFIL" in
```

new_string:
```bash
# Cabecera compartida por dos usos: se IMPRIME en compone_contenido() (abajo)
# y se usa para medir si el bloque final no trajo más que ella — literal
# único, para que las dos lecturas nunca diverjan (el bug "brancNotas" de
# I4, arriba, fue justo dos copias del mismo texto separándose).
CABECERA="=== Contexto inyectado (reflex, PARCIAL — no sustituye tu brief) ==="
compone_contenido() {
  echo "$CABECERA"
  case "$PERFIL" in
```

old_string:
```bash
LINEAS_TOTAL="$(wc -l < "$CONTENT_FULL")"
LINEAS_SALIDA="$(wc -l < "$CONTENT_CUT")"
if [ "$LINEAS_SALIDA" -lt "$LINEAS_TOTAL" ]; then
  CORTADAS=$((LINEAS_TOTAL - LINEAS_SALIDA))
  INPUT_JSON="$(jq -cn --arg t "$TYPE" '{agent_type:$t}' 2>/dev/null)" || INPUT_JSON='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "inject-truncated" "$INPUT_JSON" "lines_cut=$CORTADAS budget=$BUDGET" || true
fi
cat "$CONTENT_CUT"
```

new_string:
```bash
LINEAS_TOTAL="$(wc -l < "$CONTENT_FULL")"
LINEAS_SALIDA="$(wc -l < "$CONTENT_CUT")"
if [ "$LINEAS_SALIDA" -lt "$LINEAS_TOTAL" ]; then
  CORTADAS=$((LINEAS_TOTAL - LINEAS_SALIDA))
  INPUT_JSON="$(jq -cn --arg t "$TYPE" '{agent_type:$t}' 2>/dev/null)" || INPUT_JSON='{}'
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && \
    reflex_log "inject-truncated" "$INPUT_JSON" "lines_cut=$CORTADAS budget=$BUDGET" || true
fi
# F3.2 (docs/backlog.md: "inject-emitted se emite aunque no se inyecte
# nada"): si el bloque final no supera el tamaño de su propia cabecera, el
# perfil no compuso NADA sustantivo (caso medido: `reducido` con KB no
# resoluble). Un aviso por stderr, no un evento logueado aquí: el evento
# vive en subagent-inject.sh (que es quien sabe agent_type/agent_id/session_id
# de verdad — este script no los tiene, solo TYPE) para no duplicar el join.
CABECERA_BYTES="$(printf '%s\n' "$CABECERA" | wc -c)"
CONTENIDO_BYTES="$(wc -c < "$CONTENT_CUT")"
if [ "$CONTENIDO_BYTES" -le "$CABECERA_BYTES" ]; then
  echo "sin-contenido" >&2
fi
cat "$CONTENT_CUT"
```

- [ ] **Step 3: Implementación — `subagent-inject.sh`**

`Edit` sobre `plugins/exo/scripts/subagent-inject.sh`:

old_string:
```bash
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
JSON=""
if BLOQUE="$("$SCRIPT_DIR/compose-inject.sh" --type "$TYPE" "${KB_ARGS[@]}" 2>/dev/null)" && [ -n "$BLOQUE" ]; then
  JSON="$(printf '%s' "$BLOQUE" | jq -Rs '{hookSpecificOutput:{hookEventName:"SubagentStart", additionalContext:.}}' 2>/dev/null)" || JSON=""
fi
if [ -n "$JSON" ]; then
  bytes="$(printf '%s' "$BLOQUE" | wc -c)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-emitted" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-failed" "$INPUT" "type=$TYPE perfil=$PERFIL" || true
fi
exit 0
```

new_string:
```bash
KB_ARGS=()
[ -n "${REFLEX_INJECT_KB:-}" ] && KB_ARGS=(--kb "$REFLEX_INJECT_KB")
JSON=""
COMPOSE_ERR="$(mktemp)"
if BLOQUE="$("$SCRIPT_DIR/compose-inject.sh" --type "$TYPE" "${KB_ARGS[@]}" 2>"$COMPOSE_ERR")" && [ -n "$BLOQUE" ]; then
  JSON="$(printf '%s' "$BLOQUE" | jq -Rs '{hookSpecificOutput:{hookEventName:"SubagentStart", additionalContext:.}}' 2>/dev/null)" || JSON=""
fi
if [ -n "$JSON" ]; then
  bytes="$(printf '%s' "$BLOQUE" | wc -c)"
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-emitted" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  # F3.2: compose-inject.sh avisa "sin-contenido" por stderr cuando el
  # bloque no supera el tamaño de su propia cabecera — el caso medido de
  # `reducido` sin KB resoluble. inject-emitted YA se logueó arriba (el
  # contrato "el hook siempre entrega algo" no cambia); esto es aditivo.
  if grep -q 'sin-contenido' "$COMPOSE_ERR" 2>/dev/null; then
    . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-empty" "$INPUT" "type=$TYPE perfil=$PERFIL bytes=$bytes" || true
  fi
  printf '%s' "$JSON"
else
  . "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && reflex_log "inject-failed" "$INPUT" "type=$TYPE perfil=$PERFIL" || true
fi
rm -f "$COMPOSE_ERR"
exit 0
```

- [ ] **Step 4: Verlo verde**

Run: `bash plugins/exo/scripts/test-subagent-inject.sh`
Expected: `=== Resultado: 7/7 pasaron ===`, incluido `[PASS] caso7: ...`.

Run: `bash plugins/exo/scripts/test-compose-inject.sh`
Expected: todos los casos existentes siguen en verde (el cambio en
`compose-inject.sh` es aditivo por stderr; ningún test de ese fichero
comprueba stderr hoy, así que ninguno debería verse afectado — confírmalo
leyendo el resultado, no lo asumas).

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/scripts/compose-inject.sh plugins/exo/scripts/subagent-inject.sh plugins/exo/scripts/test-subagent-inject.sh
git commit -m "feat(e, inject-empty): distingue 'compuse cabecera sola' de 'inyecté contenido' sin tocar inject-emitted"
```

---

### Task 2: `test-exo-recall.sh` — guards `no-engine`/`no-config`, su orden, camino feliz

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Oráculo:**
`bash plugins/exo/scripts/test-exo-recall.sh` verde, con el fichero pasando
de 1 a 5 casos.

**Evidencia:** `docs/backlog.md:373-383`. `exo-recall.sh` existe desde la
ola 1A y su suite (`test-exo-recall.sh`) solo cubre H5 (la reafirmación tras
compactar) — 42 líneas, un solo test. Nada ejercita las tres ramas del
`if`/`elif`/`else` de `exo-recall.sh:57-102` (`no-engine`, `no-index`,
`no-config`), su orden relativo, ni el camino feliz completo.

**No hay código de producción que cambiar**: `exo-recall.sh` ya implementa
el `elif` que evita el doble-log histórico (Task 8 de la ola 1B, ya
mergeada). Esta tarea es pura cobertura.

**Files:**
- Test: `plugins/exo/scripts/test-exo-recall.sh`

**Interfaces:**
- Consumes: `exo-recall.sh` tal cual (variables de entorno `EXO_BIN`,
  `EXO_INDEX`, `EXO_KB_NAME`, `EXO_RECALL_NOTA`; función `run_hook` ya
  definida en el fichero).
- Produces: nada nuevo — solo tests.

- [ ] **Step 1: Test que falla (los 4 casos nuevos)**

Inserta en `plugins/exo/scripts/test-exo-recall.sh`, justo antes de la línea
final `printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"`:

```bash
# ------------------- helpers para leer el log de fallback -----------------
ultimo_evento() { tail -1 "$LOGC" 2>/dev/null | jq -r '.reflex // empty' 2>/dev/null; }
ultimo_payload() { tail -1 "$LOGC" 2>/dev/null | jq -r '.payload // empty' 2>/dev/null; }

# ------------------- no-engine: EXO_BIN no ejecutable ----------------------
: > "$LOGC"
run_hook '{"session_id":"sess-ne"}' EXO_BIN="$TMP/no-existe-bin"
EV_NE="$(ultimo_evento)"; PL_NE="$(ultimo_payload)"
if [ "$EV_NE" = "recall-fallback" ] && contains "$PL_NE" "reason=no-engine"; then
  pass "no-engine: EXO_BIN no ejecutable ⇒ recall-fallback reason=no-engine"
else
  fail "no-engine: EXO_BIN no ejecutable ⇒ recall-fallback reason=no-engine" \
    "evento=$EV_NE payload=$PL_NE"
fi

# --- orden: sin engine, UNA sola causa (el bug histórico: no-config Y ------
# no-engine para la misma ausencia de binario, Task 8 de la ola 1B) ---------
LINEAS_NE="$(wc -l < "$LOGC" | tr -d ' ')"
if [ "$LINEAS_NE" -eq 1 ] && ! grep -q 'reason=no-config' "$LOGC"; then
  pass "orden: sin engine, se loguea SOLO no-engine (nunca no-config además)"
else
  fail "orden: sin engine, se loguea SOLO no-engine (nunca no-config además)" \
    "lineas=$LINEAS_NE log=$(cat "$LOGC")"
fi

# ------------------- no-index: EXO_BIN ejecutable, EXO_INDEX ausente ------
STUB_OK="$TMP/exo-stub-ok"
cat > "$STUB_OK" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$STUB_OK"

: > "$LOGC"
run_hook '{"session_id":"sess-ni"}' EXO_BIN="$STUB_OK" EXO_INDEX="$TMP/no-existe.db"
EV_NI="$(ultimo_evento)"; PL_NI="$(ultimo_payload)"
if [ "$EV_NI" = "recall-fallback" ] && contains "$PL_NI" "reason=no-index"; then
  pass "no-index: EXO_INDEX ausente ⇒ recall-fallback reason=no-index"
else
  fail "no-index: EXO_INDEX ausente ⇒ recall-fallback reason=no-index" \
    "evento=$EV_NI payload=$PL_NI"
fi

# ------------------- no-config: exo config --json no resuelve nombre ------
: > "$LOGC"
touch "$TMP/index-vacio.db"
run_hook '{"session_id":"sess-nc"}' EXO_BIN="$STUB_OK" EXO_INDEX="$TMP/index-vacio.db"
if jq -e 'select(.reflex=="recall-fallback" and (.payload|test("reason=no-config")))' "$LOGC" >/dev/null 2>&1; then
  pass "no-config: EXO_BIN sin subcomando config ⇒ recall-fallback reason=no-config"
else
  fail "no-config: EXO_BIN sin subcomando config ⇒ recall-fallback reason=no-config" \
    "log=$(cat "$LOGC")"
fi

# ------------------- camino feliz: bloque real, sin ningún fallback -------
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"

: > "$LOGC"
touch "$TMP/index-feliz.db"
run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db"
CTX_OK="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if contains "$CTX_OK" "Contrato de memoria" \
   && ! contains "$CTX_OK" "Tu memoria persistente es una KB" \
   && [ ! -s "$LOGC" ]; then
  pass "camino feliz: bloque real inyectado, sin fallback ni log de degradación"
else
  fail "camino feliz: bloque real inyectado, sin fallback ni log de degradación" \
    "ctx='$CTX_OK' log=$(cat "$LOGC" 2>/dev/null)"
fi
```

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected (antes de nada más, con el `exo-recall.sh` de HOY): los 5 casos
YA pasan — no hay bug de producción que arreglar aquí, así que el "rojo" de
esta tarea es la AUSENCIA de los tests, no un fallo del hook. Verifícalo
comentando cada bloque nuevo uno a uno y confirmando que sin él
`grep -c '^\[PASS\]' ` baja en 1 — es la forma de demostrar que el test SÍ
ejerce la rama que dice ejercer (falsación local, sin tocar el hook).

Concretamente, para el caso `no-engine`: cambia temporalmente
`EXO_BIN="$TMP/no-existe-bin"` por `EXO_BIN="$STUB_OK"` (definido más abajo
en el mismo fichero — muévelo arriba temporalmente para la prueba) y
reejecuta: el test debe fallar («evento=recall-fallback» pero
payload sin `no-engine`, o vacío). Revierte el cambio.

- [ ] **Step 2: Verlo verde de verdad**

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: `7 passed, 0 failed` (H5 ya suma 2 `pass` — las dos aserciones del
bloque "reafirmación tras compactar" — + los 5 casos nuevos).

- [ ] **Step 3: Commit**

```bash
git add plugins/exo/scripts/test-exo-recall.sh
git commit -m "test(e, exo-recall): cubre no-engine, no-index, no-config, su orden y el camino feliz"
```

---

### Task 3: `exo search` sin resultados imprime `no results`

**Lane:** mecánica (envelope JSON NO cambia — ver Global Constraints).
**Depende de otras tareas de E:** no. **Oráculo:** `cd engine && cargo test
--release --test search_no_results_cli` verde, con el test en rojo antes del
Step 2.

**Evidencia:** `docs/backlog.md:1422-1433`. `busca_cmd` (`engine/src/main.rs:932`),
rama sin `--json`: si `resultado.results` está vacío, el `for` no itera y la
función retorna `Ok(())` — ni un `no results` en stdout ni un aviso en
stderr, exit 0 igual que con resultados. `targets_cmd` (`main.rs:990`, rama
`else if resultado.candidatos.is_empty()`) ya imprime `no candidates` en el
caso análogo — es el contrato que se alinea.

**Superficie irreversible — declarado explícitamente**: el envelope JSON de
`search` (rama `if args.json`, `main.rs:975`) **NO cambia**. `results: []`
ya distinguía el caso vacío ahí; esta tarea toca solo la rama `else`
(texto plano). Ningún consumidor de `--json` ve una sola clave nueva.

**Files:**
- Modify: `engine/src/main.rs:970-979` (cuerpo de `busca_cmd`)
- Test: `engine/tests/search_no_results_cli.rs` (nuevo)

**Interfaces:**
- Consumes: `exo::buscador::{busca, busca_hybrid, busca_vector}` (sin
  cambios de firma), `Busqueda.results: Vec<Resultado>` (campo existente).
- Produces: mismo contrato de `targets_cmd` — `no results` en stdout cuando
  `results` está vacío y `--json` no está presente; sin cambios cuando hay
  resultados o cuando `--json` sí está presente.

- [ ] **Step 1: Test que falla**

Crea `engine/tests/search_no_results_cli.rs`:

```rust
//! `exo search` sin resultados: contrato análogo a `exo targets` (que ya
//! imprime `no candidates`, `engine/src/main.rs:1004`). Antes de esta
//! tarea, `busca_cmd` volvía `Ok(())` sin imprimir nada — una terminal en
//! blanco indistinguible de "no filtré la salida" (docs/backlog.md).
//!
//! No usa `tests/common/mod.rs`: pasa `--db` explícito, así que
//! `resuelve_db` corta en el flag antes de cargar config. `search` no tiene
//! `--kb`, así que la KB esperada sale de `resuelve_kb(None)` — sin
//! `$EXO_KB` ni config, resuelve a `None` (aviso `Option`, nunca error), lo
//! mismo que otros tests de `search` sin `EXO_CONFIG` puesto
//! (`kb_root_lectura_cli.rs::search_sin_kb_resoluble_no_avisa_y_no_falla`).

use std::path::Path;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// DB con UNA nota indexada por FTS, para poder buscar algo que SÍ matchea
/// (test de no-regresión) y algo que no matchea nada (el caso nuevo).
fn db_con_una_nota(dir: &Path) -> std::path::PathBuf {
    let db = dir.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/a', 'a.md', 'a', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('a', 'contenido buscable de a', 'kb/a')",
        [],
    )
    .unwrap();
    drop(conn);
    db
}

#[test]
fn sin_resultados_imprime_no_results_y_sale_0() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("zzz-query-que-no-matchea-nada")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "sin resultados sigue siendo exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout, "no results\n",
        "stdout debe ser exactamente 'no results', igual que 'no candidates' en targets: {stdout:?}"
    );
}

#[test]
fn con_resultados_no_imprime_no_results() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("buscable")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("no results"),
        "con resultados reales no debe aparecer 'no results': {stdout:?}"
    );
    assert!(
        stdout.contains("kb/a"),
        "la búsqueda debía seguir encontrando la nota: {stdout:?}"
    );
}

#[test]
fn json_sin_resultados_no_gana_ninguna_clave_nueva() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_con_una_nota(dir.path());

    let out = Command::new(bin())
        .args(["search", "--db"])
        .arg(&db)
        .arg("--json")
        .arg("zzz-query-que-no-matchea-nada")
        .env_remove("EXO_CONFIG")
        .env_remove("EXO_KB")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("no results"),
        "el texto 'no results' es SOLO del modo texto plano, nunca del envelope JSON: {stdout}"
    );
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "search");
    assert!(v["data"]["results"].as_array().unwrap().is_empty());
    // Ninguna clave nueva en `data`: mismo conjunto de claves que antes de
    // esta tarea (results, avisos-si-los-hay — nada de un flag "empty").
    let claves: std::collections::BTreeSet<&str> =
        v["data"].as_object().unwrap().keys().map(|s| s.as_str()).collect();
    assert!(
        !claves.contains("no_results") && !claves.contains("empty"),
        "el envelope no debe ganar una clave nueva para este caso: {claves:?}"
    );
}
```

Run: `cd engine && cargo test --release --test search_no_results_cli`
Expected: FAIL en `sin_resultados_imprime_no_results_y_sale_0` — panic
`assertion \`left == right\` failed` con `left: ""` (stdout vacío hoy) vs
`right: "no results\n"`. Las otras dos pasan ya (documentan comportamiento
actual sin tocar nada).

- [ ] **Step 2: Implementación**

`Edit` sobre `engine/src/main.rs`:

old_string:
```rust
    if args.json {
        envelope::emite("search", serde_json::to_value(&resultado)?);
    } else {
        for r in &resultado.results {
            println!("{}\t{}\t{:.4}", r.permalink, r.tipo, r.score);
        }
    }
    Ok(())
}

/// Candidatas de la KB para un tema (`exo::objetivos::busca_objetivos`).
```

new_string:
```rust
    if args.json {
        envelope::emite("search", serde_json::to_value(&resultado)?);
    } else if resultado.results.is_empty() {
        // Contrato alineado con `targets_cmd` (más abajo, `no candidates`):
        // una terminal en blanco no distingue "sin resultados" de "no filtré
        // la salida". El envelope JSON no cambia — `results: []` ya lo
        // distinguía ahí.
        println!("no results");
    } else {
        for r in &resultado.results {
            println!("{}\t{}\t{:.4}", r.permalink, r.tipo, r.score);
        }
    }
    Ok(())
}

/// Candidatas de la KB para un tema (`exo::objetivos::busca_objetivos`).
```

- [ ] **Step 3: Verlo verde**

Run: `cd engine && cargo test --release --test search_no_results_cli`
Expected: `test result: ok. 3 passed; 0 failed`.

Run: `cd engine && cargo test --release --test targets_cli`
Expected: sin cambios (`sin_candidatas_el_array_es_vacio_no_nulo` sigue
verde) — confirma que el contrato de `targets` no se tocó.

- [ ] **Step 4: Commit**

```bash
git add engine/src/main.rs engine/tests/search_no_results_cli.rs
git commit -m "feat(e, search-no-results): 'no results' en texto plano cuando no hay resultados, envelope JSON intacto"
```

---

### Task 4: la guarda «una DB sirve a una KB» no recomienda `--db` a `init`

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Oráculo:**
`cd engine && cargo test --release --test indexer --test inicia` verde, con
el test nuevo de `inicia.rs` en rojo antes del Step 2.

**Evidencia:** `docs/backlog.md:1041-1056`. `comprueba_kb_root`
(`engine/src/indexer.rs:471-482`) tiene un único mensaje para sus dos
llamadores: `indexer.rs:121` (dentro de `indexa`, usado por `exo index`/
`exo rebuild`, que sí tienen `--db`) e `inicia.rs:140` (dentro de
`valida_db_para_kb`, llamada por `init_cmd` en `main.rs:579,610` — `exo init`
no tiene `--db`, resuelve por `$EXO_DB`). El mensaje dice literalmente «usa
otra `--db` para esta», engañoso cuando lo dispara `init`.

**Cuidado con la regresión**: `engine/tests/indexer.rs:675`
(`indexar_otra_kb_existente_sobre_la_misma_db_falla_y_no_borra_la_primera`)
ya hace `assert!(msg.contains("exo rebuild"))` sobre el mensaje del camino
`index`/`rebuild` — ese texto NO cambia.

**Files:**
- Modify: `engine/src/indexer.rs:471-482`
- Modify: `engine/src/indexer.rs:121` (call site dentro de `indexa`)
- Modify: `engine/src/inicia.rs:133-141` (`valida_db_para_kb`)
- Test: `engine/tests/inicia.rs`

**Interfaces:**
- Produces: `pub enum OrigenComprobacion { IndexORebuild, Init }` en
  `engine/src/indexer.rs`, y `comprueba_kb_root(conn, kb_abs, origen:
  OrigenComprobacion) -> Result<()>` (firma con un parámetro nuevo).
- Consumes (Task 4 en sí): nada de otras tareas.
- **Aviso para D**: si D llega a necesitar comprobar `kb_root` desde algún
  comando nuevo (`rotate`/`stale`), debe pasar
  `OrigenComprobacion::IndexORebuild` (esos comandos sí tienen `--db`) —
  D no toca este fichero según su propio plan, se deja anotado por si acaso.

- [ ] **Step 1: Test que falla**

Añade a `engine/tests/inicia.rs` (al final del fichero):

```rust
#[test]
fn valida_db_para_kb_rechaza_otra_kb_sin_mencionar_un_flag_que_init_no_tiene() {
    let dir = tempfile::tempdir().unwrap();
    let kb_vieja = dir.path().join("vieja");
    let kb_nueva = dir.path().join("nueva");
    std::fs::create_dir_all(&kb_vieja).unwrap();
    std::fs::create_dir_all(&kb_nueva).unwrap();
    let db = dir.path().join("index.db");
    {
        let conn = exo::abre_db(&db).unwrap();
        exo::schema::crea_schema(&conn).unwrap();
        let kb_vieja_abs = std::fs::canonicalize(&kb_vieja).unwrap();
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)",
            [kb_vieja_abs.to_string_lossy().to_string()],
        )
        .unwrap();
    }

    let err = exo::inicia::valida_db_para_kb(&db, &kb_nueva).expect_err("otra KB debe rechazarse");
    let msg = format!("{err:#}");
    assert!(
        !msg.contains("--db"),
        "el mensaje de `exo init` no debe recomendar --db, que init no tiene: {msg}"
    );
    assert!(
        msg.contains("EXO_DB"),
        "el mensaje debe nombrar $EXO_DB, la vía real de `exo init` para otra DB: {msg}"
    );
    assert!(msg.contains("otra KB"), "sigue siendo el mismo guard: {msg}");
}
```

Run: `cd engine && cargo test --release --test inicia valida_db_para_kb_rechaza_otra_kb_sin_mencionar_un_flag_que_init_no_tiene`
Expected: FAIL — `assertion failed: !msg.contains("--db")` (el mensaje de hoy
sí contiene `--db`, para los dos llamadores).

- [ ] **Step 2: Implementación**

`Edit` sobre `engine/src/indexer.rs`:

old_string:
```rust
pub fn comprueba_kb_root(conn: &Connection, kb_abs: &Path) -> Result<()> {
    let Some(previo) = kb_root_conflicto(conn, kb_abs)? else {
        return Ok(());
    };
    bail!(
        "este índice es de otra KB que sigue en disco: {previo} (pediste {}). \
         Una DB sirve a UNA KB: usa otra --db para esta, o `exo rebuild --kb {} --db <esta db>` \
         si de verdad quieres reemplazar el índice",
        kb_abs.display(),
        kb_abs.display()
    )
}
```

new_string:
```rust
/// Distingue el remedio que ofrece `comprueba_kb_root`: `index`/`rebuild`
/// tienen `--db`, `init` no (resuelve por `$EXO_DB`, `db_de_init` en
/// `inicia.rs`). Mismo guard, mensaje distinto por llamador — antes era un
/// único texto que recomendaba `--db` incluso disparado desde `init`
/// (docs/backlog.md, "la guarda 'una DB sirve a una KB' recomienda --db a
/// init, que no lo tiene").
#[derive(Clone, Copy)]
pub enum OrigenComprobacion {
    IndexORebuild,
    Init,
}

pub fn comprueba_kb_root(
    conn: &Connection,
    kb_abs: &Path,
    origen: OrigenComprobacion,
) -> Result<()> {
    let Some(previo) = kb_root_conflicto(conn, kb_abs)? else {
        return Ok(());
    };
    let remedio = match origen {
        OrigenComprobacion::IndexORebuild => format!(
            "usa otra --db para esta, o `exo rebuild --kb {} --db <esta db>` \
             si de verdad quieres reemplazar el índice",
            kb_abs.display()
        ),
        OrigenComprobacion::Init => {
            "usa otro $EXO_DB para esta KB (`EXO_DB=<ruta> exo init …`), o borra/reemplaza \
             la DB actual si de verdad quieres reutilizarla"
                .to_string()
        }
    };
    bail!(
        "este índice es de otra KB que sigue en disco: {previo} (pediste {}). \
         Una DB sirve a UNA KB: {remedio}",
        kb_abs.display()
    )
}
```

old_string (call site dentro de `indexa`, línea 121):
```rust
    comprueba_kb_root(&conn, &kb_abs)?;
```

new_string:
```rust
    comprueba_kb_root(&conn, &kb_abs, OrigenComprobacion::IndexORebuild)?;
```

`Edit` sobre `engine/src/inicia.rs`:

old_string:
```rust
pub fn valida_db_para_kb(db: &Path, kb: &Path) -> Result<()> {
    if !db.exists() {
        return Ok(());
    }
    let conn = crate::abre_db(db)?;
    crate::schema::crea_schema(&conn)?;
    let kb_abs = std::fs::canonicalize(kb).unwrap_or_else(|_| kb.to_path_buf());
    crate::indexer::comprueba_kb_root(&conn, &kb_abs)
}
```

new_string:
```rust
pub fn valida_db_para_kb(db: &Path, kb: &Path) -> Result<()> {
    if !db.exists() {
        return Ok(());
    }
    let conn = crate::abre_db(db)?;
    crate::schema::crea_schema(&conn)?;
    let kb_abs = std::fs::canonicalize(kb).unwrap_or_else(|_| kb.to_path_buf());
    crate::indexer::comprueba_kb_root(&conn, &kb_abs, crate::indexer::OrigenComprobacion::Init)
}
```

- [ ] **Step 3: Verlo verde, y sin regresión en `index`/`rebuild`**

Run: `cd engine && cargo test --release --test inicia --test indexer`
Expected: `test result: ok` en los dos binarios. En particular
`indexer::indexar_otra_kb_existente_sobre_la_misma_db_falla_y_no_borra_la_primera`
sigue en verde (el mensaje de `index`/`rebuild` sigue mencionando
`exo rebuild`).

Run: `cd engine && cargo build --release 2>&1 | grep -i "error\|warning: unused" || echo "sin errores ni warnings de no-uso"`
Expected: `sin errores ni warnings de no-uso` — `OrigenComprobacion` se usa
en los dos call sites, sin variante muerta.

- [ ] **Step 4: Commit**

```bash
git add engine/src/indexer.rs engine/src/inicia.rs engine/tests/inicia.rs
git commit -m "fix(e, guarda-db): el mensaje de 'una DB sirve a una KB' ya no recomienda --db a init, que usa \$EXO_DB"
```

---

### Task 5: `engine/rust-toolchain.toml`

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Oráculo:** no es
un gate (no hay ciclo rojo-verde con exit code) — es una declaración que
`rustup` consume solo. Verificación: `rustup show active-toolchain` desde
`engine/` antes y después.

**Evidencia:** `docs/backlog.md:556-581`. `engine/Cargo.toml:8` declara
`rust-version = "1.95"`. `find . -iname "rust-toolchain*"` no encuentra nada
en el árbol. El job `msrv` de `ci.yml` pinea `dtolnay/rust-toolchain@1.95.0`
(`ci.yml:84`) para comprobar la MSRV; los jobs `lint` y `test` usan
`dtolnay/rust-toolchain@stable` (`ci.yml:30`, `:103`) — son la mayoría (2 de
3) y los que un desarrollador reproduce a mano con más frecuencia.

**Decisión de diseño (el backlog permitía `stable` o la MSRV; se documenta
la elección):** `channel = "stable"`, no `"1.95"`. Razón: fijar `1.95` haría
que `rustup` instale/seleccione localmente una versión más vieja que
`stable` para TODO uso de `cargo` en `engine/`, divergiendo de los dos jobs
mayoritarios de CI (`lint`, `test`) que sí usan `stable` — el job `msrv` ya
pinea su propio `1.95.0` de forma independiente y seguirá haciéndolo, así
que la MSRV queda comprobada igual. `channel = "stable"` resuelve
exactamente el síntoma medido (`rustup default` distinto en máquinas
distintas produce el mismo tropiezo dos veces) sin introducir una tercera
versión a mantener sincronizada a mano.

**Files:**
- Create: `engine/rust-toolchain.toml`

**Interfaces:** ninguna (fichero de configuración de `rustup`, no de Rust).

- [ ] **Step 1: Verificar el estado ANTES (sin el fichero)**

Run: `rustup show active-toolchain 2>&1`
Expected (anota el resultado tal cual salga en esta máquina — es la
variable que el ítem del backlog dice que varía "sin que nadie edite nada"):
algo como `stable-x86_64-unknown-linux-gnu (default)` — el toolchain activo
sale del default GLOBAL de `rustup`, no de nada declarado en el repo.

- [ ] **Step 2: Crear el fichero**

`engine/rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
```

- [ ] **Step 3: Verificar el efecto**

Run: `cd engine && rustup show active-toolchain 2>&1`
Expected: sigue resolviendo a `stable-<triple>`, pero ahora la fuente es el
fichero — confírmalo con `cd engine && rustup show 2>&1 | grep -A1 "active toolchain"`,
que debe decir algo con `overridden by '.../engine/rust-toolchain.toml'` (el
texto exacto lo pone `rustup`; lo que importa es que YA NO diga que viene
del default de la máquina).

Run: `cd engine && cargo check --all-targets --locked 2>&1 | tail -5`
Expected: compila sin error de "requires rustc X" (si esta máquina tiene
`stable` ≥ 1.95, que es el caso medido hoy — `rustc 1.98.0`).

- [ ] **Step 4: Commit**

```bash
git add engine/rust-toolchain.toml
git commit -m "chore(e, rust-toolchain): fija channel=stable — la maquina ya no depende de rustup default"
```

---

### Task 6: `test-hermetico.sh` con `--locked`, log completo y `upload-artifact`

**Lane:** mecánica, pero **rehace el ciclo rojo-verde** por tocar un gate ya
demostrado falsable (Global Constraints). **Depende de otras tareas de E:**
no. **Comparte fichero con Task 8** (`.github/workflows/ci.yml`, jobs
distintos) — despáchala antes o después de Task 8, no en paralelo.
**Oráculo:** los tres pasos del Step 3 (rojo provocado, verde, log
verificado en disco).

**Evidencia:** `docs/backlog.md:706-737` (diagnosticabilidad, a medias desde
`50aee95`) y `docs/backlog.md:757-788` sub-ítem «`--locked` no llega al job
`test`». `engine/scripts/test-hermetico.sh:19` corre `cargo test --release
--no-fail-fast` sin `--locked` (a diferencia de los jobs `lint`/`msrv`, que
sí lo tienen), a un fichero dentro de `$TMP`, que el `trap` del Step 8 borra
al salir — así que ni siquiera en fallo queda nada que subir después.

**Files:**
- Modify: `engine/scripts/test-hermetico.sh` (reescritura completa)
- Modify: `.github/workflows/ci.yml` (job `test`, step del gate + step nuevo
  de `upload-artifact`)

**Interfaces:**
- Produces: mismo contrato de salida (mensaje `test-hermetico: OK — …` en
  éxito; en fallo, el mismo desglose de tests caídos que hoy, más `--locked`
  y log completo en `$EXO_HERMETICO_LOG` cuando esa variable está puesta).
- Consumes (CI): `EXO_HERMETICO_LOG` (nueva variable de entorno, opcional —
  sin ella el script se comporta como localmente siempre se comportó, log en
  `$TMP` y se borra al salir).

- [ ] **Step 1: Ver el ciclo rojo-verde ACTUAL, para no perder la referencia**

Run (con el script de HOY, antes de tocarlo):
```bash
cd engine && cat >> tests/smoke.rs <<'EOF'

#[test]
fn falsacion_temporal_campana_e_task6() {
    panic!("rojo deliberado para re-demostrar test-hermetico.sh");
}
EOF
bash scripts/test-hermetico.sh; echo "exit=$?"
```
Expected: `test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit
101).` en stderr, con el nombre `falsacion_temporal_campana_e_task6` en el
bloque `--- tests que fallaron ---`, `exit=1`.

```bash
git -C /home/paul/Documentos/proyectos/exo checkout -- engine/tests/smoke.rs
bash engine/scripts/test-hermetico.sh; echo "exit=$?"
```
Expected: `test-hermetico: OK — …`, `exit=0`. (Este es el ciclo que YA
existía; se repite tras el Step 2 para probar que sigue vivo con `--locked`
y `tee`.)

- [ ] **Step 2: Implementación**

`engine/scripts/test-hermetico.sh` completo:

```bash
#!/usr/bin/env bash
# Gate: la suite tiene que correr sin `~/.exo/config.toml`. Sin esto, el CI de
# G5 en un runner limpio nace rojo y nadie se entera hasta que el runner existe.
#
# Apunta EXO_CONFIG a un fichero inexistente en vez de mover el config real:
# mover el de la máquina es destructivo y compite con el hook `Stop` que indexa.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# EXO_HERMETICO_LOG (opt-in, campaña E, docs/backlog.md "Un rojo del job test
# no se puede diagnosticar desde el CI"): por defecto el log vive en $TMP y
# muere con el trap de arriba — sirve para diagnosticar EN el momento, no
# después de que el proceso termine. El CI lo fija a una ruta FUERA de $TMP
# para poder subirlo con actions/upload-artifact tras un fallo.
LOG="${EXO_HERMETICO_LOG:-$TMP/out.txt}"

# --locked: `lint`/`msrv` ya lo tenían; el job que de verdad EJECUTA la
# suite (y release.yml, que llama a este mismo script) no. Sin esto, el job
# más importante podía resolver un árbol de dependencias distinto del
# Cargo.lock commiteado sin que nada lo dijera.
#
# `tee`, no una redirección silenciosa: el log queda en disco EN VIVO (para
# `upload-artifact` si el runner muere a mitad) y en pantalla/stdout de quien
# corre esto a mano. `PIPESTATUS[0]`: el exit code de `cargo test`, no el de
# `tee` — explícito, no depende de que a alguien se le ocurra quitar
# `set -o pipefail` de la línea de arriba en un cambio futuro.
EXO_CONFIG="$TMP/no-existe.toml" cargo test --release --locked --no-fail-fast 2>&1 | tee "$LOG"
EC=${PIPESTATUS[0]}

if [ "$EC" -ne 0 ]; then
  echo "test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit $EC)." >&2
  # Los NOMBRES de los tests que fallaron, no solo el binario — un gate que
  # no dice QUÉ falló delega el diagnóstico en quien lo lea (medido
  # 2026-09-10, con el CI de main llevando 7 corridas en rojo sin que el log
  # dijera el nombre).
  echo "--- tests que fallaron ---" >&2
  grep -E '^test .* \.\.\. FAILED$' "$LOG" >&2 || true
  sed -n '/^failures:$/,/^test result: FAILED/p' "$LOG" >&2 || true
  echo "--- resumen ---" >&2
  grep -E '^test result: FAILED|--test ' "$LOG" >&2 || true
  # Sigue sin haber un patrón específico para un error de COMPILACIÓN de la
  # suite (docs/backlog.md, deuda conocida y NO cerrada aquí): con --locked
  # y log completo, ese caso ahora al menos queda íntegro en $LOG (y, en CI,
  # en el artifact) para leerlo a mano — no hay grep que lo resalte todavía.
  exit 1
fi
echo "test-hermetico: OK — la suite corre sin ~/.exo/config.toml, con --locked; NO cubre la caché del modelo ONNX (~0,6 GB), que las suites de indexado siguen exigiendo."
```

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```yaml
      # El gate ya existe y ya se demostró falsable (2026-08-27). El CI lo
      # consume tal cual en vez de reinventar el comando de test.
      - name: Gate hermético — suite completa sin ~/.exo/config.toml
        shell: bash
        run: ./engine/scripts/test-hermetico.sh
```

new_string:
```yaml
      # El gate ya existe y ya se demostró falsable (2026-08-27, re-demostrado
      # con --locked en la campaña E). El CI lo consume tal cual en vez de
      # reinventar el comando de test.
      - name: Gate hermético — suite completa sin ~/.exo/config.toml
        shell: bash
        env:
          EXO_HERMETICO_LOG: ${{ github.workspace }}/hermetico.log
        run: ./engine/scripts/test-hermetico.sh
      # Sin esto, un rojo exclusivo de un SO de la matriz era irreproducible
      # en la máquina del autor: el log completo (no solo el resumen que
      # imprime stderr) queda descargable desde la pestaña Actions.
      # `if: always()`: justo lo que hace falta ver cuando el gate FALLA.
      - name: Subir el log completo del gate hermético
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: hermetico-log-${{ matrix.os }}
          path: hermetico.log
          if-no-files-found: error
```

- [ ] **Step 3: Rehacer el ciclo rojo-verde con `--locked` puesto**

Rojo provocado (mismo mecanismo que el Step 1, ahora contra el script
nuevo):
```bash
cd engine && cat >> tests/smoke.rs <<'EOF'

#[test]
fn falsacion_temporal_campana_e_task6() {
    panic!("rojo deliberado para re-demostrar test-hermetico.sh con --locked");
}
EOF
EXO_HERMETICO_LOG=/tmp/hermetico-falsacion.log bash scripts/test-hermetico.sh; echo "exit=$?"
```
Expected: `test-hermetico: la suite NO corre sin ~/.exo/config.toml (exit
101).`, `falsacion_temporal_campana_e_task6` en `--- tests que fallaron
---`, `exit=1`.

Run: `[ -s /tmp/hermetico-falsacion.log ] && echo "log persistido fuera de \$TMP: OK" || echo "FALLO: log no quedó en disco"`
Expected: `log persistido fuera de $TMP: OK` — a diferencia del script viejo
(que lo borraba con el `trap` de su propio `$TMP`), el log sobrevive porque
`EXO_HERMETICO_LOG` apunta fuera.

Verde:
```bash
git -C /home/paul/Documentos/proyectos/exo checkout -- engine/tests/smoke.rs
bash engine/scripts/test-hermetico.sh; echo "exit=$?"
rm -f /tmp/hermetico-falsacion.log
```
Expected: `test-hermetico: OK — …, con --locked; …`, `exit=0`.

Run: `git -C /home/paul/Documentos/proyectos/exo status --short engine/tests/smoke.rs`
Expected: vacío — `smoke.rs` vuelve exactamente a como estaba (el
`checkout --` del paso anterior lo garantiza; confírmalo, no lo asumas).

- [ ] **Step 4: Commit**

```bash
git add engine/scripts/test-hermetico.sh .github/workflows/ci.yml
git commit -m "ci(e, test-hermetico): --locked, log completo con tee y upload-artifact — ciclo rojo-verde rehecho"
```

---

### Task 7: `release.yml` valida versiones ANTES de los tres builds

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Oráculo:** el
Step 3 (verificación de sintaxis + grafo `needs:` con `python3`/`yaml`) más
la re-demostración del script subyacente (Step 1). **Límite de
verificación, declarado**: el orden real de ejecución de los jobs de
GitHub Actions NO se puede comprobar localmente sin disparar el workflow de
verdad (`push` de un tag o `workflow_dispatch`) — ambas son acciones de
"release", línea roja del config de fábrica (solo Paul). Esta tarea deja el
YAML verificado por estructura y por el comportamiento ya probado del
script que invoca; el orden de ejecución real queda para la primera release
real posterior a este merge, fuera del alcance de un executor de esta
campaña.

**Evidencia:** `docs/backlog.md:468-487`. `publish` (`release.yml:105-136`)
declara `needs: build` (matriz linux/windows/macos, `timeout-minutes: 60`
cada leg) y solo DENTRO de `publish`, tras el inventario de artifacts, corre
`scripts/test-versiones.sh "$TAG"` (`release.yml:132-136`) — un check de
segundos que no depende de ningún artifact de build. Medido el 2026-09-13:
`scripts/test-versiones.sh` no existía en el árbol del único tag publicado
(`v0.1.0`); un `workflow_dispatch` con `tag: v0.1.0` habría corrido los tres
builds completos (hasta 3 horas-runner) antes de fallar en `publish` por el
script inexistente.

**Files:**
- Modify: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `scripts/test-versiones.sh` (sin cambios — ya acepta un `TAG`
  opcional como `$1`, `scripts/test-versiones.sh:35-38`).
- Produces: job nuevo `version-check`, del que `build` depende con `needs:`.

- [ ] **Step 1: Re-demostrar que el script subyacente sigue siendo falsable**

Run: `bash scripts/test-versiones.sh v9.9.9`
Expected: `[FAIL] el tag 'v9.9.9' no casa con engine/Cargo.toml (0.1.0): el
binario diría otra versión`, exit 1.

Run: `bash scripts/test-versiones.sh v0.1.0`
Expected: `[OK] engine 0.1.0 · plugin <version actual>`, exit 0. (El script
en sí NO se toca en esta tarea — esto solo confirma que sigue siendo el
mismo gate ya demostrado falsable que `release.yml` va a mover más arriba.)

- [ ] **Step 2: Implementación**

`Edit` sobre `.github/workflows/release.yml`:

old_string:
```yaml
jobs:
  build:
    name: ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    timeout-minutes: 60
```

new_string:
```yaml
jobs:
  # Job barato (segundos, sin toolchain de Rust) que se dispara ANTES de la
  # matriz de builds (hasta 3×60 min). Antes de esta tarea, la comprobación
  # de versiones vivía DENTRO de `publish`, tras `needs: build` — un
  # `workflow_dispatch` con un tag que no casara con `engine/Cargo.toml`
  # gastaba la matriz entera antes de fallar (medido: v0.1.0 lo habría hecho,
  # docs/backlog.md).
  version-check:
    name: el tag casa con engine/Cargo.toml
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.inputs.tag || github.ref }}
      - name: scripts/test-versiones.sh
        shell: bash
        env:
          TAG: ${{ github.event.inputs.tag || github.ref_name }}
        run: bash scripts/test-versiones.sh "$TAG"

  build:
    name: ${{ matrix.target }}
    needs: version-check
    runs-on: ${{ matrix.os }}
    timeout-minutes: 60
```

El check que ya existía dentro de `publish` (`release.yml:130-136`,
«El tag casa con engine/Cargo.toml») **se deja tal cual**: es defensa en
profundidad barata (segundos) contra un desajuste que apareciera entre
`version-check` y `publish` (por ejemplo, un push a la rama del tag entre
medias) — no hace falta borrarlo para que el job nuevo cumpla su función.

- [ ] **Step 3: Verificación de estructura (sin disparar el workflow real)**

Run:
```bash
python3 -c "
import yaml
d = yaml.safe_load(open('.github/workflows/release.yml'))
assert 'version-check' in d['jobs'], 'falta el job version-check'
assert d['jobs']['build'].get('needs') == 'version-check', \
    f\"build.needs es {d['jobs']['build'].get('needs')!r}, esperaba 'version-check'\"
assert d['jobs']['publish'].get('needs') == 'build', 'publish debe seguir dependiendo de build'
print('OK: version-check -> build -> publish, en ese orden de needs')
"
```
Expected: `OK: version-check -> build -> publish, en ese orden de needs`.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci(e, release-versiones): version-check corre antes de los tres builds, no despues"
```

---

### Task 8: gate de rutas personales en CI + fix de `test-git-c-bash.sh:75-76`

**Lane:** mecánica. **Depende de otras tareas de E:** no. **Comparte
fichero con Task 6** (`.github/workflows/ci.yml`, jobs distintos —
despáchalas en secuencia). **Oráculo:** `bash scripts/test-rutas-personales.sh`
verde en el árbol, rojo en las dos falsaciones del Step 4.

**Evidencia:** `docs/backlog.md:636-656` (sub-propuesta «un validador») y
`docs/backlog.md:1415-1420` (`test-git-c-bash.sh:74-75`, hoy con
`/home/paul/Documentos/proyectos/code-graph-go`). **Decisión de Paul, ya
tomada, no reabrir**: el gate mira SOLO rutas (`/home/<user>`,
`C:\Users\<user>`, `/Users/<user>`), no nombres propios.

**Medido hoy contra el árbol real** (discovery idéntico al de
`scripts/test-shellcheck.sh`: índice de git, `.sh` + shebang `sh`/`bash`,
excluye `evals/` y `docs/`): **3 líneas** en **2 ficheros**, ninguna más —

```
plugins/exo/scripts/test-contrato-engine.sh:45:# Rutas estilo Windows: el binario es nativo y no entiende `/c/Users/...`.
plugins/exo/scripts/test-git-c-bash.sh:75:  "cd /home/paul/Documentos/proyectos/code-graph-go && git log --oneline -5" \
plugins/exo/scripts/test-git-c-bash.sh:76:  "git -C /home/paul/Documentos/proyectos/code-graph-go log --oneline -5"
```

`test-contrato-engine.sh:45` **es un falso positivo real** (no citado por el
dictamen, encontrado al medir): el comentario dice `` `/c/Users/...` `` —
contiene la subcadena `/Users/...`, aunque no sea una ruta de una máquina
concreta. Se reescribe en esta misma tarea (consecuencia directa de aplicar
el gate, no dispersión).

**Files:**
- Create: `scripts/test-rutas-personales.sh`
- Modify: `.github/workflows/ci.yml` (job `lint`, nuevo step)
- Modify: `plugins/exo/scripts/test-git-c-bash.sh:75-76`
- Modify: `plugins/exo/scripts/test-contrato-engine.sh:45`

**Interfaces:** ninguna nueva — script de CI, sin superficie Rust ni JSON.

- [ ] **Step 1: Test que falla (el gate detecta lo que ya está mal)**

Crea `scripts/test-rutas-personales.sh`:

```bash
#!/usr/bin/env bash
# Gate: ninguna ruta de una máquina concreta en el bash versionado.
#
# Decisión de Paul: SOLO rutas (/home/<user>, /Users/<user>,
# C:\Users\<user> o su forma C:/Users/<user>), nunca nombres propios — "Paul"
# o "kb-demo" sueltos NO cuentan, así que no hace falta tocar los fixtures de
# test-recall-inject.sh (EXO_KB_NAME="kb-demo", etc.).
#
# Precedente: affaan-m/ECC (clone 5064474d4d762dc9640234a41617cccb79185cec,
# scripts/ci/validate-no-personal-paths.js:41-42) cubre /Users/<nombre> y
# C:\Users\<nombre> pero NO /home/<user> — justo la única ofensora real de
# este repo (test-git-c-bash.sh). Copiarlo tal cual no bastaba.
#
# Mismo descubrimiento que test-shellcheck.sh: índice de git, .sh + shebang
# sh/bash, excluye evals/ (harness congelado — cita rutas de corridas
# pasadas como evidencia, no las produce) y docs/ (backlog.md documenta
# estas mismas rutas como HALLAZGOS, con cita; no son las que produce el
# hook).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

PATRON='(/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:[\\/]Users[\\/][A-Za-z0-9_.-]+)'

ficheros=()
while read -r modo blob _etapa ruta; do
  case "$ruta" in evals/*|docs/*) continue ;; esac
  case "$ruta" in
    *.sh) ficheros+=("$ruta") ;;
    *.*) : ;;
    *)
      [ "$modo" = "100755" ] || continue
      if git cat-file -p "$blob" | head -n 1 | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
        ficheros+=("$ruta")
      fi
      ;;
  esac
done < <(git ls-files -s)

if [ "${#ficheros[@]}" -eq 0 ]; then
  echo "test-rutas-personales: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

hallados=""
for f in "${ficheros[@]}"; do
  if match="$(grep -EnH "$PATRON" "$f" 2>/dev/null)"; then
    hallados="${hallados}${match}"$'\n'
  fi
done

if [ -n "$hallados" ]; then
  echo "test-rutas-personales: rutas de una máquina concreta en bash versionado:" >&2
  printf '%s' "$hallados" >&2
  echo "Arreglo: sustituye por una ruta genérica fuera de /home, /Users o C:\\Users (p.ej. /opt/proyectos/...) o por una variable." >&2
  exit 1
fi

echo "test-rutas-personales: OK — ${#ficheros[@]} scripts sin rutas personales"
```

Run: `chmod +x scripts/test-rutas-personales.sh && bash scripts/test-rutas-personales.sh; echo "exit=$?"`
Expected: el script no usa `pass`/`fail` con prefijo `[FAIL]` — imprime
directamente a stderr sus 3 líneas propias: `test-rutas-personales: rutas
de una máquina concreta en bash versionado:`, el/los `match` de
`grep -EnH` acumulados en `$hallados`, y `Arreglo: sustituye por una ruta
genérica...`; termina con `exit=1`. (Este ES el "test que falla" de esta
tarea: el árbol de HOY ya tiene el problema — no hace falta fabricar una
falsación para el rojo inicial.)

- [ ] **Step 2: Arreglar los dos ficheros reales**

`Edit` sobre `plugins/exo/scripts/test-git-c-bash.sh`:

old_string:
```bash
assert_rewrite "cd && git log con flags → rewrite" \
  "cd /home/paul/Documentos/proyectos/code-graph-go && git log --oneline -5" \
  "git -C /home/paul/Documentos/proyectos/code-graph-go log --oneline -5"
```

new_string:
```bash
assert_rewrite "cd && git log con flags → rewrite" \
  "cd /opt/proyectos/code-graph-go && git log --oneline -5" \
  "git -C /opt/proyectos/code-graph-go log --oneline -5"
```

(Nota: sustituir solo el nombre de usuario por uno genérico NO basta — el
gate mira la FORMA `/home/<lo-que-sea>`, no si el nombre es real. Por eso la
ruta sale por completo de `/home`.)

`Edit` sobre `plugins/exo/scripts/test-contrato-engine.sh`:

old_string:
```bash
# Rutas estilo Windows: el binario es nativo y no entiende `/c/Users/...`.
```

new_string:
```bash
# Rutas estilo Windows: el binario es nativo y no entiende las que monta Git
# Bash (letra de unidad + "Users" + nombre de perfil).
```

- [ ] **Step 3: Wiring en CI**

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```yaml
      - name: Versiones coherentes (plugin.json == marketplace.json)
        if: always()
        run: bash scripts/test-versiones.sh
```

new_string:
```yaml
      - name: Versiones coherentes (plugin.json == marketplace.json)
        if: always()
        run: bash scripts/test-versiones.sh
      - name: Sin rutas de una máquina concreta en el bash versionado
        if: always()
        run: bash scripts/test-rutas-personales.sh
```

- [ ] **Step 4: Verlo verde, y las dos falsaciones**

Run: `bash scripts/test-rutas-personales.sh; echo "exit=$?"`
Expected: `test-rutas-personales: OK — 45 scripts sin rutas personales`,
`exit=0`.

Falsación 1 (reintroducir una ruta `/home`):
```bash
printf '# /home/cualquiera/x\n' >> plugins/exo/scripts/_timeout.sh
bash scripts/test-rutas-personales.sh; echo "exit=$?"   # [FAIL] ... _timeout.sh:<N>:# /home/cualquiera/x, exit=1
git -C /home/paul/Documentos/proyectos/exo checkout -- plugins/exo/scripts/_timeout.sh
bash scripts/test-rutas-personales.sh; echo "exit=$?"   # OK de nuevo, exit=0
```

Falsación 2 (una ruta `C:\Users\`):
```bash
printf '# C:\\Users\\cualquiera\\x\n' >> plugins/exo/scripts/_timeout.sh
bash scripts/test-rutas-personales.sh; echo "exit=$?"   # [FAIL] ... exit=1
git -C /home/paul/Documentos/proyectos/exo checkout -- plugins/exo/scripts/_timeout.sh
bash scripts/test-rutas-personales.sh; echo "exit=$?"   # OK de nuevo, exit=0
git -C /home/paul/Documentos/proyectos/exo status --short plugins/exo/scripts/_timeout.sh   # vacío
```

- [ ] **Step 5: Commit**

```bash
git add scripts/test-rutas-personales.sh .github/workflows/ci.yml plugins/exo/scripts/test-git-c-bash.sh plugins/exo/scripts/test-contrato-engine.sh
git commit -m "ci(e, rutas-personales): gate de rutas /home,/Users,C:\Users en el bash versionado; fixture de test-git-c-bash.sh genérico"
```

---

### Task 9: `docs/backlog.md` sincronizado — cierra lo que E resuelve, caduca con cita lo demás

**Lane:** mecánica. **Depende de:** Tasks 1-8 (todas — cita sus commits).
Va **la última**. **Oráculo:** los greps del Step 2.

**Files:**
- Modify: `docs/backlog.md`

- [ ] **Step 1: Ediciones**

**1a. Cabecera — nuevo párrafo `> Última revisión`.** `Edit`, old_string es
la primera línea del bloque de cabecera actual:

old_string:
```
> Última revisión: **2026-09-13** (cierre de la campaña B —
```

new_string:
```
> Última revisión: **<fecha de ejecución>** (campaña E — hooks honestos +
> CI de coste trivial, `docs/superpowers/plans/2026-09-14-campana-e-hooks-honestos-y-ci.md`,
> mergeada vía <PR de E>. Cierra con evidencia: `inject-emitted` sin
> distinguir cabecera-sola de contenido real (Task 1, commit
> `<commit Task 1>`), suite de `exo-recall.sh` sin guards (Task 2, commit
> `<commit Task 2>`), `exo search` mudo sin resultados (Task 3, commit
> `<commit Task 3>`), el mensaje de la guarda «una DB sirve a una KB»
> recomendando `--db` a `init` (Task 4, commit `<commit Task 4>`),
> `engine/rust-toolchain.toml` ausente (Task 5, commit `<commit Task 5>`),
> `--locked`/log completo/`upload-artifact` en el gate hermético (Task 6,
> commit `<commit Task 6>`), el orden de `release.yml` (Task 7, commit
> `<commit Task 7>`), y el gate de rutas personales en CI (Task 8, commit
> `<commit Task 8>`). **Caduca con cita**, sin cerrar del todo: el truncado
> del bloque de arranque (acción a, HECHA desde antes de esta campaña —
> `main.rs:917-919`, `exo-recall.sh:89-91`; acciones b/c siguen abiertas,
> fuera de alcance de E) y la frase «`kb-demo` en 8 ficheros, tres de ellos
> de producción» del ítem "exo genérico" (el nombre ya sale de
> `exo config --json` en los tres; el resto del ítem sigue abierto). Fuera
> de esta campaña: cutover
> kbx→exo, paridad Go — campaña D, en paralelo.)
>
> Anterior: **2026-09-13** (cierre de la campaña B —
```

(Sustituye `<fecha de ejecución>`, `<PR de E>` y los ocho `<commit Task N>`
por los valores reales al ejecutar — cada uno es el commit que la propia
Task N de este plan acaba de crear, resoluble por el ejecutor sin ambigüedad
porque son commits de la misma rama que él mismo hizo.)

**1b. Tabla `## Estado`** — añade una fila de campaña, mismo formato que las
de A/B/C:

old_string:
```
| **Campaña C** | held-out pre-registrado, **sin cambio de producción** — mergeada a `main` el **2026-09-14** vía PR #16 (`cb25541`); veredicto en `evals/retrieval-heldout/verdict/c-verdict.md`. El held-out queda consumido; D6 (default de `exo search --type`) PENDIENTE-PAUL |
```

new_string:
```
| **Campaña C** | held-out pre-registrado, **sin cambio de producción** — mergeada a `main` el **2026-09-14** vía PR #16 (`cb25541`); veredicto en `evals/retrieval-heldout/verdict/c-verdict.md`. El held-out queda consumido; D6 (default de `exo search --type`) PENDIENTE-PAUL |
| **Campaña E** | 9 tasks (hooks honestos: `inject-empty`, suite de `exo-recall.sh`, `no results` en `search`; engine: mensaje de guarda parametrizado, `rust-toolchain.toml`; CI: `--locked`+log+artifact en el gate hermético, orden de `release.yml`, gate de rutas personales) — mergeada a `main` el **<fecha>** vía <PR de E>; plan en `docs/superpowers/plans/2026-09-14-campana-e-hooks-honestos-y-ci.md` |
```

**1c. Caducar con cita el ítem «El bloque de arranque va al 96% de su
cap…»** (Alta). `Edit`, insertando ANTES de la línea `**Acción, por orden de
coste:**` de ese ítem:

old_string:
```
No lo causó esta ola (aportó 28 B de esos 5.921), pero la ola lo hizo medible.
  **Acción, por orden de coste:** (a) que el truncado **grite**
```

new_string:
```
No lo causó esta ola (aportó 28 B de esos 5.921), pero la ola lo hizo medible.
  **Acción (a) HECHA, verificado en la campaña E (2026-09-14) — CADUCA con
  cita:** `engine/src/main.rs:917-919` emite `eprintln!("aviso: recall
  truncado por --cap-bytes={} …")` desde `2234887` (M2-08, comando `exo
  recall`), y `plugins/exo/scripts/exo-recall.sh:89-91` loguea
  `recall-fallback reason=truncated` desde `48083e5` (fusión a plugin único).
  El grito por stderr y el evento en el log que la acción (a) pedía ya
  existen los dos. **Siguen abiertas** las acciones (b) (pasada de
  `/distill` sobre `core-index`) y (c) (revisar el cap de 6.144) — ninguna la
  toca la campaña E, ambas fuera de su lista. El «Cruce (2026-09-09)» de más
  abajo proponía unificar esto con el ítem hermano de `inject-emitted` en un
  campo de envelope compartido: la campaña E **no adoptó** esa unificación
  (cambiaría el JSON que `compose-inject.sh` entrega a Claude Code, una
  superficie que pedía cautela) — en su lugar cerró el blind spot de
  `inject-emitted` con su propio evento aditivo (`inject-empty`, ver ítem de
  abajo), sin tocar ningún esquema.
  **Acción, por orden de coste:** (a) que el truncado **grite**
```

**1d. Cerrar con evidencia el ítem «`inject-emitted` se emite aunque no se
inyecte nada»** (Alta) — muévelo a `## Cerrado con evidencia`. Primero,
`Edit` para retirarlo de `## Alta` (old_string = el ítem completo, líneas
352-371 de HOY; cópialo verbatim del fichero real al ejecutar, no lo
re-teclees de memoria) y `new_string` = cadena vacía (retirarlo). Segundo,
`Edit` para insertarlo en `## Cerrado con evidencia`, con esta cabecera
añadida:

```
- [x] **`inject-emitted` se emite aunque no se inyecte nada: cerrado el
  <fecha> (campaña E, Task 1, `<commit Task 1>`).**
  `subagent-inject.sh` ahora loguea `inject-empty` ADEMÁS de `inject-emitted`
  cuando el bloque compuesto no supera el tamaño de su propia cabecera (71 B,
  medido con `printf '%s\n' "$CABECERA" | wc -c`) — el caso real: perfil
  `reducido` (agente `exo:executor`) sin KB resoluble. `inject-emitted` NO
  se retira (el contrato "el hook siempre entrega algo" no cambia) — el
  evento nuevo es aditivo, verificado contra el único consumidor del log
  (`plugins/exo/scripts/a1-gate.sh`, que no lee `inject-empty`, así que sus
  contadores `emitidos`/`sesiones`/`denom_u2` no se movieron). Test:
  `plugins/exo/scripts/test-subagent-inject.sh` caso 7.
  [texto histórico del ítem, sin tocar, para no perder la evidencia
  original]
```

(Mismo patrón: reproduce el texto histórico completo del ítem debajo de la
línea de cierre, como hace cada entrada existente de `## Cerrado con
evidencia` — cópialo del `## Alta` de hoy antes de moverlo, verbatim.)

**1e. Cerrar con evidencia el ítem «`exo-recall.sh` no tiene suite de
test»** — mismo mecanismo que 1d: mover de `## Alta` a `## Cerrado con
evidencia`, con cabecera:

```
- [x] **`exo-recall.sh` no tiene suite de test: cerrado el <fecha> (campaña
  E, Task 2, `<commit Task 2>`).**
  `plugins/exo/scripts/test-exo-recall.sh` pasa de 1 a 6 casos: cubre
  `no-engine`, `no-index`, `no-config`, el orden entre `no-engine` y
  `no-config` (la única causa se loguea una vez, nunca dos — el bug
  histórico de la Task 8 de la ola 1B, ya cerrado en código, ahora también
  en test) y el camino feliz completo (bloque real con `Contrato de
  memoria`, cero eventos de fallback). Corre en CI vía `scripts/test-plugin.sh`.
  [texto histórico del ítem]
```

**1f. Cerrar con evidencia (in situ, sin mover — el ítem completo se cierra
porque las dos "deudas hermana" ya estaban cerradas)** el ítem de la guarda
`--db`. `Edit`:

old_string:
```
  **Acción:** el mensaje deja de asumir un flag que no todos sus llamadores
  tienen — o se parametriza por comando (`init` → menciona `$EXO_DB`,
  `index`/`rebuild` → menciona `--db`), o se reescribe genérico («usa otro
  índice para esta KB: `--db` en `index`/`rebuild`, `$EXO_DB` en `init`»).
```

new_string:
```
  **Acción CERRADA (campaña E, Task 4, `<commit Task 4>`, 2026-09-14):**
  parametrizado por llamador — `engine/src/indexer.rs` gana
  `pub enum OrigenComprobacion { IndexORebuild, Init }`;
  `comprueba_kb_root` recibe el origen y arma el remedio según cuál sea
  (`--db`/`exo rebuild` para `index`/`rebuild`, `$EXO_DB` para `init`). Test:
  `engine/tests/inicia.rs::valida_db_para_kb_rechaza_otra_kb_sin_mencionar_un_flag_que_init_no_tiene`.
  Sin regresión: `engine/tests/indexer.rs::indexar_otra_kb_existente_sobre_la_misma_db_falla_y_no_borra_la_primera`
  (el mensaje de `index`/`rebuild` sigue mencionando `exo rebuild`) sigue
  verde.
```

Y marca el `[ ]` inicial del ítem como `[x]` (o muévelo a `## Cerrado con
evidencia` si prefieres seguir el patrón de mover — cualquiera de las dos es
correcta, elige mover para consistencia con 1d/1e).

**1g. `engine/rust-toolchain.toml`** — cerrar con evidencia (mover). Cabecera:

```
- [x] **Falta `rust-toolchain.toml`: cerrado el <fecha> (campaña E, Task 5,
  `<commit Task 5>`).**
  `engine/rust-toolchain.toml` con `channel = "stable"` — decisión razonada
  en el plan de E: `stable` en vez de `1.95` porque los jobs `lint`/`test`
  de CI (la mayoría) ya usan `dtolnay/rust-toolchain@stable`; el job `msrv`
  sigue pineando `1.95.0` de forma independiente, así que la MSRV declarada
  en `engine/Cargo.toml:8` sigue comprobada.
  [texto histórico del ítem]
```

**1h. `--locked` no llega al job `test` (sub-ítem de «Dos endurecimientos del
CI…»)** — cerrar IN SITU (el ítem padre queda abierto por `HF_HOME`, que E no
toca):

old_string:
```
    **Acción:** entra en la misma campaña que la deuda de diagnosticabilidad
    del script, arriba — las dos exigen tocarlo y por tanto rehacer su ciclo
    rojo-verde.
```

new_string:
```
    **Acción CERRADA (campaña E, Task 6, `<commit Task 6>`, 2026-09-14):**
    `engine/scripts/test-hermetico.sh` corre con `--locked`; ciclo
    rojo-verde rehecho (rojo provocado con un `#[test] { panic!() }`
    temporal en `engine/tests/smoke.rs`, revertido). `HF_HOME` sigue sin
    fijar — fuera del alcance de esta campaña, el ítem padre sigue abierto
    por esa mitad.
```

**1i. Diagnosticabilidad del job `test` (ítem «Un rojo… no se puede
diagnosticar»)** — anotar a medias, parcial adicional:

old_string:
```
  **Acción propuesta:** campaña propia — `tee` o un `EXO_HERMETICO_LOG`
  opt-in en el script (con su propio ciclo rojo-verde, porque el script es
  un gate ya demostrado falsable y tocarlo invalida esa evidencia), más
  `actions/upload-artifact` en el job. No se arregla aquí, solo se anota.
```

new_string:
```
  **Parcial CERRADO (campaña E, Task 6, `<commit Task 6>`, 2026-09-14):**
  `EXO_HERMETICO_LOG` opt-in + `tee` en el script, `actions/upload-artifact`
  en `ci.yml` (job `test`) subiendo el log completo por SO. **Sigue
  abierto**: un error de COMPILACIÓN de la suite sigue sin casar ningún
  patrón de grep del script — ahora al menos queda íntegro en el artifact
  para leerlo a mano, pero nada lo resalta. Fuera del alcance de E.
```

**1j. `release.yml` valida versiones tras los builds** — cerrar con
evidencia (mover):

```
- [x] **En `release.yml`, el check de versiones corría DESPUÉS de los tres
  builds: cerrado el <fecha> (campaña E, Task 7, `<commit Task 7>`).**
  Job nuevo `version-check` (barato, sin toolchain de Rust), del que `build`
  depende con `needs:`. El check original dentro de `publish` se deja como
  defensa en profundidad. **Límite de verificación declarado en el plan de
  E**: el orden real de ejecución no se comprobó disparando el workflow de
  verdad (acción de "release", fuera del alcance de un executor) — se
  verificó la estructura del YAML (`needs:` bien encadenado) y que
  `scripts/test-versiones.sh` sigue siendo el mismo gate ya demostrado
  falsable.
  [texto histórico del ítem]
```

**1k. Rutas personales — cerrar la sub-propuesta del validador (in situ,
ítem padre sigue abierto por `validate-hooks.js`)**:

old_string:
```
  **Acción:** los dos gates, cuando tengan dueño; ninguno se escribió en la
  campaña B (fuera de su alcance — B tocó CI de shellcheck y exec-bit, no
  este).
```

new_string:
```
  **Acción, sub-propuesta 1 (validador de rutas) CERRADA (campaña E, Task 8,
  `<commit Task 8>`, 2026-09-14):** `scripts/test-rutas-personales.sh`, mismo
  descubrimiento que `test-shellcheck.sh` (índice de git, `.sh` + shebang,
  excluye `evals/`/`docs/`), corre en `ci.yml` job `lint`. Cubre exactamente
  el patrón que Paul decidió (solo rutas: `/home/<user>`, `/Users/<user>`,
  `C:\Users\<user>`), sin detectar nombres propios. De paso arregló un falso
  positivo no anticipado por el dictamen del consultor:
  `test-contrato-engine.sh:45` mencionaba `` `/c/Users/...` `` en un
  comentario (subcadena que casaba el patrón sin ser una ruta real);
  reescrito. **Sigue abierta** la sub-propuesta 2 (`validate-hooks.js`/schema
  de `hooks.json`) — no pedida para esta campaña.
```

**1l. Cerrar con evidencia (mover) el ítem `test-git-c-bash.sh:74-75`**:

```
- [x] **Las 2 rutas `/home/paul/…` de `test-git-c-bash.sh:74-75`: cerradas el
  <fecha> (campaña E, Task 8, `<commit Task 8>`).**
  Fixture movido fuera de `/home` por completo (`/opt/proyectos/…`), no solo
  anonimizado: el gate de rutas personales de Task 8 mira la FORMA
  `/home/<lo-que-sea>`, no si el nombre es real — sustituir `paul` por un
  nombre genérico seguiría cayendo dentro del patrón.
  [texto histórico del ítem]
```

**1m. Frase «`kb-demo` en 8 ficheros, tres de ellos de producción», dentro
del ítem "exo genérico" (Alta)** — CADUCAR con cita, sin cerrar el ítem
entero:

old_string:
```
    **Re-medido el 2026-09-13, tras H22 (`e5398d5`+`1018802`) partir
    `distill/SKILL.md` en `SKILL.md` + `chequeos.md`:** el conteo cambia de
    forma (`git grep -c Paul -- plugins/exo`) pero no de fondo — ver el
    detalle arriba, en la entrada del 2026-09-04.
```

new_string:
```
    **Re-medido el 2026-09-13, tras H22 (`e5398d5`+`1018802`) partir
    `distill/SKILL.md` en `SKILL.md` + `chequeos.md`:** el conteo cambia de
    forma (`git grep -c Paul -- plugins/exo`) pero no de fondo — ver el
    detalle arriba, en la entrada del 2026-09-04.
    **CADUCA con cita (campaña E, 2026-09-14):** la frase «`kb-demo` en 8
    ficheros, tres de ellos de producción» está superada en el conteo (hoy
    son 9 — `README.md` se sumó después de esa medición). Verificado hoy
    (`git grep -l kb-demo -- plugins/exo/`, 9 ficheros): en NINGUNO el
    nombre está hardcodeado en lógica de runtime — `exo-recall.sh:62-70` y
    `recall-inject.sh:229-236` lo resuelven vía `exo config --json`, y los
    únicos usos literales de "kb-demo" que quedan son comentarios que
    EXPLICAN ese cambio, fixtures deliberadas de
    `test-recall-inject.sh` (`EXO_KB_NAME="kb-demo"` puesto por el propio
    test) y las skills `distill`/`recon-first`, que nombran la KB real de
    Paul por diseño. El resto del ítem (`Paul` en 5 ficheros, `kbx` como
    dependencia operativa de `distill`) **sigue abierto** — fuera del
    alcance de E.
```

- [ ] **Step 2: Oráculo**

Run: `grep -n "inject-emitted se emite aunque" docs/backlog.md`
Expected: la única coincidencia está bajo `## Cerrado con evidencia`
(comprueba con `awk '/^## Cerrado con evidencia/{c=1} /inject-emitted se emite aunque/{print c":"NR}' docs/backlog.md`
→ debe imprimir `1:<línea>`).

Run: `grep -n "no tiene suite de test" docs/backlog.md`
Expected: igual — solo bajo `## Cerrado con evidencia`.

Run: `grep -c "campaña E" docs/backlog.md`
Expected: ≥ 8 (una mención por cada cierre/caducidad de esta tarea).

Run: `grep -c 'usa otra `--db` para' docs/backlog.md`
Expected: `1` — el backlog cita `--db` entre backticks (`docs/backlog.md:1048`),
en el texto de la acción (a) de `comprueba_kb_root` que describe el camino
`index`/`rebuild`, que NO cambió. (El grep sin backticks, `"usa otra --db
para esta"`, no matchea nada en `docs/backlog.md` — oráculo roto, nunca dio
el `2` que este Step decía esperar.) Cero apariciones sueltas recomendando
`--db` para `init`.

- [ ] **Step 3: Commit**

```bash
git add docs/backlog.md
git commit -m "docs(e, backlog): cierra 6 items con evidencia, caduca 2 con cita — campaña E"
```
