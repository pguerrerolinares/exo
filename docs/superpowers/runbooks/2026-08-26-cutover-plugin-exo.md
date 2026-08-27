# Cutover del plugin único `exo` — runbook

> Baseline, cutover y rollback de la ola 1B (G2 de la spec de exo genérico).
> La sección «Baseline» se captura ANTES de tocar nada: después no se puede
> distinguir «esto ya estaba roto» de «lo rompí yo».
>
> Plan: `docs/superpowers/plans/2026-08-26-ola1b-plugin-exo.md`.

## Baseline (antes del cutover) — 2026-08-27

### Hallazgo previo: el check del plan no era falsable

El Step 1 de la Task 1 mandaba contar eventos así:

```bash
grep -c '"event":"index"' ~/.claude/reflex-log.jsonl
```

**Ese check no puede dar nunca un número distinto de 0**, por dos razones
independientes, y ninguna tiene que ver con si el hook funciona:

1. **La clave no existe.** `grep -c '"event"' ~/.claude/reflex-log.jsonl` → `0`.
   El log usa `"reflex":"<nombre>"`, no `"event"`. Los nueve nombres vivos son
   `git-c` (46), `inject-emitted` (45), `recall-inject-emitted` (29),
   `recall-fallback` (16), `git-c-rewrite` (4), `recall-inject-degraded` (2),
   `zero-residuo`, `verify-before-done` y `compact`.
2. **El indexado con éxito no se loguea ahí, por diseño.** `exo-index.sh` solo
   llama a `reflex_log` en la rama de fallo (`index-fallback`, `:36-41`); el
   camino feliz lanza `exo index --json` detached y **redirige su salida a otro
   fichero** (`LOG="${EXO_INDEX_LOG:-$HOME/.claude/exo-index.log}"`, `:29`).

Un check que devuelve `0` tanto si el hook funciona como si no, no mide nada:
es exactamente el instrumento que no grita. Sustituido abajo por el artefacto
real. **El plan queda corregido en el mismo commit que este runbook.**

### Step 1 — ¿el hook `Stop` reindexa de verdad en esta máquina? SÍ

Artefacto real: `~/.claude/exo-index.log`, donde aterriza el envelope de cada
corrida detached.

```
$ ls -l ~/.claude/exo-index.log
-rw-r--r-- 1 paul 1049089 8524 Aug 27 12:26 /c/Users/paul/.claude/exo-index.log

$ grep -c '"command":"index"' ~/.claude/exo-index.log
65

$ tail -1 ~/.claude/exo-index.log
{"command":"index","data":{"borradas":0,"indexadas":0,"saltadas":170,"trozos_embebidos":0,"trozos_reusados":0},"schema_version":1}

$ stat -c '%y %n' ~/.exo/index.db
2026-08-27 11:05:54.474341300 +0200 /c/Users/paul/.exo/index.db

$ jq -r 'select(.reflex=="index-fallback")' ~/.claude/reflex-log.jsonl
(ninguna — ninguna corrida degradó)
```

**Cierra el item 3 de la ola 0 de la spec** («el fix está instalado pero no hay
evidencia de que se haya ejecutado»): 65 corridas, la última hoy a las 12:26,
minutos antes de capturar esta baseline. *Instalado* Y *ejecutado*.

### Step 1-bis — el desfase binario↔scripts está VIVO, no es hipotético

El mismo artefacto lo demuestra sin buscarlo. El envelope de arriba dice:

- `"schema_version":1` — el contrato v2 salió en la ola 1A (mergeada hoy).
- Claves en español: `borradas`, `indexadas`, `saltadas`, `trozos_embebidos`,
  `trozos_reusados` — renombradas a inglés por D8 en esa misma ola.

Es decir: **el binario instalado escribe envelopes v1 en disco ahora mismo,
cada vez que se cierra un turno**, mientras los scripts del repo ya hablan v2.
Es el item Alta #3 de `docs/backlog.md` ocurriendo en vivo, y la razón de que
la Task 8 lleve el Step 1½ (instalar el binario ANTES que el plugin).

Evidencia cruzada: `~/.local/bin/exo.exe` es del **24-08 17:11**, anterior al
merge de la ola 1A (**27-08 10:13**).

### Step 2 — plugins instalados (el estado al que hay que poder volver)

```
$ ls -d ~/.claude/plugins/cache/exo/*/*/
/c/Users/paul/.claude/plugins/cache/exo/process/1.0.0/
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.16.0/
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.17.0/
```

`installed_plugins.json` (`version: 2`), entradas vivas:

| Plugin | Versión | installPath | gitCommitSha |
|---|---|---|---|
| `context7@claude-plugins-official` | `b819188d2eea` | `cache/claude-plugins-official/context7/b819188d2eea` | `6d5f7944…` |
| `equipo-x@equipo-x-standards` | `1.4.0` | `cache/equipo-x-standards/equipo-x/1.4.0` | `d0fdc4df…` |
| `process@exo` | `1.0.0` | `cache/exo/process/1.0.0` | `8c58467f…` |
| `reflex@exo` | `0.17.0` | `cache/exo/reflex/0.17.0` | `67c077db…` |

Nota: `reflex/0.16.0/` sigue en caché pero **no** está en
`installed_plugins.json` — es residuo del despliegue del 23-08, ya anotado como
deuda. Importa para el cutover porque el shim de la KB hace `sort -V | tail -1`
sobre ese glob: hoy resuelve a `0.17.0` por versión, no por ser el instalado.

Confirmado además lo que la adjudicación de B2 daba por bueno: **`paul-profile`
y `workflow-lint` no están instalados** en esta máquina. Retirarlos del
marketplace no rompe nada aquí.

### Step 3 — el gate de la KB funciona HOY

```
$ ls -d "$HOME"/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.16.0/scripts/kb-precommit.sh
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.17.0/scripts/kb-precommit.sh
```

El glob resuelve a **dos** ficheros reales; el shim se queda con `0.17.0`
(`sort -V | tail -1`). El shim vive en `kb-demo/.git/hooks/pre-commit`,
fuera de todo repo, y su rama de fallo es:

```bash
echo "pre-commit: no encuentro kb-precommit.sh del plugin reflex — commit permitido" >&2
exit 0
```

**El gate está cerrado hoy y se abre solo si el glob deja de matchear.** Con el
plugin renombrado a `exo`, el glob `…/cache/exo/reflex/*/…` deja de matchear y
el commit pasa con un aviso por stderr que nadie mira. Es el riesgo 5 de la
spec y la razón de existir de la Task 4.

## Hallazgo durante la ejecución (2026-08-27) — el desfase apaga la inyección, y lo loguea como éxito

Salió al validar la Task 3-bis, midiendo el perfil que se inyecta a cada
subagente. **Es la prueba concreta de por qué el binario va antes que el
plugin** (Task 8, Step 1½): mucho más fuerte que el argumento del mtime.

### La cadena

`compose-inject.sh` resuelve la KB en este orden (`:19-29`):

```
--kb explícito  >  $EXO_KB  >  exo config --json | .data.kb.path
```

El último eslabón llama a **`exo config`**, un subcomando que **nació en la ola
1A de hoy** (`43b9e36`). El binario instalado es del 24-08:

```
$ ~/.local/bin/exo.exe config --json
error: unrecognized subcommand 'config'
```

Con los scripts del repo (que ya usan esa cadena) contra el binario instalado
(que no la entiende), la KB **no se resuelve**. Y el perfil `reducido` —el del
agente `executor`— es *solo rutas, sin doctrina*: sin KB no tiene nada que
emitir.

### La medida

```
$ compose-inject.sh --type exo:executor --kb /c/proyectos/homework/kb-demo
1274 bytes   (con rutas — correcto)

$ compose-inject.sh --type exo:executor          # lo que hace el hook real
71 bytes     (solo la cabecera)
```

| Perfil | Con KB | Sin KB |
|---|---|---|
| `reducido` (`exo:executor`) | 1274 B | **71 B — nada** |
| `doctrina` / `ejecucion` | 1752 B | 784 B (la doctrina estática sobrevive) |

`reducido` es el único perfil que se queda **en cero**, porque es el único
hecho solo de rutas. El agente que más disciplina necesita es el que se queda
sin nada.

### Por qué es fallo silencioso y no un error

`subagent-inject.sh` loguea `inject-emitted` igual, con el tamaño real:

```
$ jq -r 'select(.reflex=="inject-emitted") | .payload' ~/.claude/reflex-log.jsonl | sort | uniq -c
     29 type=reflex:executor perfil=reducido bytes=1209     <- firings reales, sanos
      5 type=exo:executor    perfil=reducido bytes=70       <- las pruebas de hoy
     14 type=general-purpose perfil=ejecucion bytes=1751
```

Un evento llamado **`inject-emitted`** para una inyección que no inyectó nada.
El instrumento dice «emitido» y el número que lo desmiente (`bytes=70`) está en
el payload, donde nadie lo mira. Ausencia de error ≠ evidencia de efecto.

Las 29 entradas de `1209` son firings reales del plugin **instalado** (scripts
viejos, que resolvían la KB por el literal cableado que la ola 1A retiró). O
sea: hoy funciona porque el plugin viejo no depende del binario nuevo. **En
cuanto el plugin nuevo entre sin el binario nuevo, se apaga.**

### Consecuencia operativa

El Step 1½ de la Task 8 no es higiene, es la condición de que el cutover no
deje mudos a todos los `exo:executor`. Y su check —`schema_version == 2` sobre
el envelope— vale también para esto: el mismo binario que emite v2 es el que
entiende `exo config`.

**Verificación post-cutover (va en el Step 3 de la Task 8):** después de
instalar binario y plugin, disparar un subagente real y exigir en el log
`type=exo:executor perfil=reducido` con **bytes > 1000**. Un `bytes=70` con
`inject-emitted` es el fallo, y sin ese umbral se lee como éxito.

## Cutover

> Lo rellena la Task 8.

## Rollback

> Lo rellena la Task 8, Step 7.
