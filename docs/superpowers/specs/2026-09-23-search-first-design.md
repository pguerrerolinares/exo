# Reflejo `search-first`: buscar en la KB antes del primer trabajo sustantivo

**Fecha:** 2026-09-23 · **Estado:** aprobada en brainstorming, pendiente de review de la spec escrita
**Origen:** incidente de la sesión de la Task 5 de la campaña I (abajo)

## 1. El incidente

Paul pidió ejecutar la Task 5 de la campaña I (fundir los tres guards de
`PreToolUse:Bash`). El agente principal recibió al arrancar el `core-index`,
que dice «Antes de trabajo sustantivo, busca contexto (exo search --type
hybrid, exo targets)» (`wisdom-paul/core/core-index.md:17`), y del hook
`UserPromptSubmit` un top-3 automático irrelevante. No ejecutó `exo search`
ni `exo targets`: hizo `Grep` en el repo, leyó el plan y despachó al
ejecutor. Al preguntarle, lo reconoció como un atajo («el plan del repo ya
especifica la task»).

## 2. Diagnóstico (informe del consultor, verificado ejecutando)

- **La regla es prosa.** Vive en un bloque de arranque que se trunca en
  silencio a 6.144 B y no tiene disparador concreto. Es el caso de
  `learnings/Una regla que se cita y se ignora es un comentario.md`
  («documentación ≠ enforcement»: el memory packet en prosa se cumplió un 8 %)
  y la ley #3 de «Fallo silencioso» (contrato por prosa).
- **El top-3 ruidoso da la sensación de que ya se buscó**, aunque no cubre
  lo que hace falta.
- **En este caso la búsqueda no habría aportado nada**: el plan vivía en el
  repo `exo`, fuera de la KB, y tres `exo search --type hybrid` bien
  formadas devolvieron el mismo ruido. El fallo es de procedimiento, no de
  información. Pero la regla existe para la memoria entre sesiones, que
  `Grep` no ve, y el hueco real está medido en otro sitio: el estrato
  `prompt` del held-out acierta 11/22 (`backlog/Backlog — exo.md:43-55`,
  ítem aparcado «Tasa de re-explicación»).

## 3. Decisiones (brainstorming con Paul)

| Pregunta | Decisión |
|---|---|
| Propósito de la v1 | **Corregir y medir**: aviso por `additionalContext` y registro en reflex-log. Nunca bloquea. |
| Qué cuenta como búsqueda | **Solo la KB**: `exo search`/`exo targets` por Bash. (exo no tiene servidor MCP: comprobado el 2026-09-23, ninguna transcripción tiene llamadas `mcp__*exo*`; si algún día lo tiene, se añade su nombre al patrón.) Grep/Read en el repo no cuentan (con ellos el incidente habría pasado sin aviso). |
| Cómo se detecta | **Leer la transcripción** una vez por sesión, en el primer trabajo sustantivo. Se descartan marcar desde `bash-guards.sh` (estado repartido en dos hooks, mezcla un sensor de proceso con los guards de git) y auditar en `Stop` (solo mide y salta en cada respuesta, no una vez por sesión). |
| Ámbito | Solo el agente principal. Un subagente que ejecuta una tarea concreta está exento por doctrina (`core-index`, ROUTING DE PROCESO). |

## 4. Diseño

### 4.1 Componente

`plugins/exo/scripts/search-first.sh`, reflejo `search-first`. Warn-only,
siempre exit 0. Se cablea en `plugins/exo/hooks/hooks.json`:

```json
{
  "matcher": "^(Agent|Task|Edit|Write|NotebookEdit)$",
  "hooks": [{ "type": "command", "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/search-first.sh" }]
}
```

### 4.2 Flujo

1. **Camino rápido, en bash puro y sin `jq`** (el coste de cada llamada a
   partir de la primera):
   - `INPUT="$(cat)"`; se saca `session_id` con `[[ $INPUT =~ ... ]]`.
   - Si `agent_id` viene con valor (`"agent_id":"<algo>"`), es un subagente:
     `exit 0` **sin** crear el sentinel, porque el sentinel es por sesión y
     lo comparten el agente principal y sus subagentes (mismo patrón que
     `clean-orchestrator-research.sh`).
   - Si existe `/tmp/claude-search-first-${SID:-nosession}`: `exit 0`.
2. **Primera vez en la sesión:**
   - `command -v jq` o `search-first-skip` (motivo `sin-jq`).
   - `transcript_path` con `jq`, no con regex: en Windows la ruta llega con
     `\\` escapados.
   - Un `jq` sobre la transcripción JSONL que solo mira los bloques
     `tool_use` de los mensajes del asistente
     (`select(.type=="assistant") | .message.content[]? | select(.type=="tool_use")`;
     forma comprobada contra una transcripción real el 2026-09-23) y da
     positivo si `name=="Bash"` y `input.command` casa con
     `(^|[;&|/[:space:]])exo(\.exe)?[[:space:]]+(search|targets)([[:space:]]|$)`
     (la clase incluye `/` para reconocer `exo` invocado por ruta, p.ej.
     `./engine/target/release/exo.exe search "x"` o `~/.local/bin/exo search`).
   - **No vale un `grep` a pelo sobre la transcripción.** El pie de
     `recall-inject.sh` escribe «exo search --type hybrid» en cada prompt y
     llega a la transcripción como texto inyectado: el `grep` daría siempre
     positivo (comprobado: una transcripción real de este proyecto contiene
     5 veces el texto y ninguna llamada).
3. **Resultado:**
   - Positivo: `reflex_log "search-first-ok"`, sin salida.
   - Negativo: `reflex_log "search-first"` y en stdout
     `{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"<aviso>"}}`.
     Aviso (≤3 líneas): «Reflejo search-first: primer trabajo sustantivo de
     la sesión sin `exo search`/`exo targets` previo. Si el tema puede tener
     historia en la KB, busca antes (`exo search --type hybrid "<tema>"`);
     si no, dilo en una línea y sigue.»
   - En los dos casos se crea el sentinel: como mucho un aviso por sesión, y
     el log registra también las sesiones que sí buscaron (hace falta el
     denominador para la tasa).
4. **Si falla algo:** sin `transcript_path`, con el fichero ausente o
   ilegible, o si falla el `jq`, se registra `search-first-skip` con el
   motivo, no se avisa y se crea el sentinel. Si el sensor no sabe, calla.

### 4.3 Limitaciones aceptadas

- Si lo primero de la sesión es un Edit trivial, el aviso sale igualmente.
  Es un falso positivo que se ignora y que el log deja medible.
- `echo "exo search"` como dato cuenta como búsqueda: el patrón mira el
  comando, no si se ejecutó. Es un falso negativo aceptado.
- Una búsqueda hecha por un subagente no la ve el agente principal en su
  transcripción. Eso es lo que se quiere: el contexto tiene que llegar al
  que decide.

## 5. Regla del `core-index`

En `wisdom-paul/core/core-index.md:17` se cambia la frase «Antes de trabajo
sustantivo, busca contexto (exo search --type hybrid, exo targets)» por una
de la misma longitud, con disparador concreto:

> **Antes del primer Agent/Edit/Write de la sesión**: `exo search --type
> hybrid "<tema>"` o `exo targets`, o di en una línea por qué no aplica (el
> reflejo `search-first` avisa si no).

`core-index` es un índice: se cambia la frase, no se comprime nada alrededor.
Commit aparte en `wisdom-paul`, con `exo ratchet` en verde.

## 6. Tests

`plugins/exo/scripts/test-search-first.sh`, con transcripciones JSONL de
fixture y `REFLEX_LOG_FILE` temporal:

1. `tool_use` Bash con `exo search --type hybrid "x"`: `search-first-ok`, stdout vacío.
2. **Trampa:** solo aparece el pie del recall («exo search --type hybrid») como texto, sin `tool_use`: aviso.
3. `cd x && exo targets` dentro de un encadenado: ok.
4. `echo "exo search"`: ok (falso negativo documentado en §4.3).
5. Subagente (`agent_id` con valor): stdout vacío, sin log y sin sentinel.
6. Segunda llamada con el sentinel creado: silencio, y `jq` no llega a invocarse (se comprueba con un `jq` falso que deja marca, como en los tests de la campaña I).
7. Sin `transcript_path`, o apuntando a un fichero que no existe: `search-first-skip`, sin aviso.
8. Sin `jq` en el PATH: exit 0, `search-first-skip`.
9. La salida del caso negativo es un JSON válido (`jq -e '.hookSpecificOutput.additionalContext'`).
10. `exo search` en un mensaje del usuario (no `tool_use`): aviso.

## 7. Medición (W11)

Se publica en el commit de cableado y en `docs/backlog.md`:

- Camino rápido (sentinel creado), 20 repeticiones: p50/p95 por llamada.
- Primera llamada con una transcripción real de unos 2 MB: p50/p95.
- **Criterio:** si el camino rápido pasa de 150 ms p50, antes de mergear se
  estudia reducir el matcher a `^(Agent|Task)$` (menos llamadas, se pierde el
  caso de Edit directo sin delegar).

## 8. Comprobación de que funciona

Tras una o dos semanas de uso:

```bash
jq -r 'select(.reflex|test("^search-first")) | .reflex' ~/.claude/reflex-log.jsonl | sort | uniq -c
```

La tasa `ok / (ok + search-first)` es la cifra que se compara en el tiempo.
Se enlaza desde el ítem aparcado `backlog/Backlog — exo.md:43-55` como
instrumento de la junta «recuperada → usada»; no se abre un ítem nuevo. Sin
umbral pre-registrado: es un sensor de estreno, y la subida a bloqueo sigue
el criterio de falsos positivos de «Una regla que se cita y se ignora es un
comentario» (§«Cuándo tocaría revisar el nunca bloquear»).

## 9. Entrega

- Plugin **1.3.3** (`plugin.json`, `marketplace.json`).
- `hooks.json` (§4.1); una fila más en las tablas de hooks de `README.md` y
  `plugins/exo/README.md` (lo exige `scripts/test-docs-vivos.sh`), y el
  recuento de comandos; mención en `docs/arquitectura.md`.
- Gates: `scripts/test-plugin.sh`, `scripts/test-hooks-json.sh`,
  `scripts/test-docs-vivos.sh`, `scripts/test-versiones.sh`,
  `scripts/test-exec-bit.sh`.
- Commits: script + test; cableado + versión + docs; `docs/backlog.md` con
  las cifras; en `wisdom-paul`, la regla (§5) y el enlace del ítem (§8).

## 10. Fuera de alcance (YAGNI)

- Mejorar la consulta de `recall-inject.sh`: no habría resuelto este caso, y
  es un hook que corre en cada prompt con el p95 de W11 ya por encima del
  umbral.
- Reforzar `subagent-inject.sh`: el que se saltó la búsqueda fue el agente
  principal.
- Bloquear (exit 2).
