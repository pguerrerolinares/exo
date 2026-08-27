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

## El shim del pre-commit — 2026-08-27

**Respaldo antes de tocar nada.** El fichero
`kb-demo/.git/hooks/pre-commit` no vive en ningún repo — no hay commit que
lo respalde ni `git checkout` que lo recupere. Copia literal guardada en el
scratchpad de la sesión (fuera de cualquier repo) antes del primer edit, y
verificada con `diff` contra el original:
`kb-demo-pre-commit.2026-08-27.bak` (contenido idéntico al shim que dejó
el 24-08: rama de fallo `echo … ; exit 0`).

### Step 1 — el fallo demostrado, no asumido

```
$ ls -d "$HOME"/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh 2>/dev/null; echo "GLOB_VIEJO_EXIT=$?"
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.16.0/scripts/kb-precommit.sh
/c/Users/paul/.claude/plugins/cache/exo/reflex/0.17.0/scripts/kb-precommit.sh
GLOB_VIEJO_EXIT=0

$ ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh 2>/dev/null; echo "GLOB_NUEVO_EXIT=$?"
GLOB_NUEVO_EXIT=2
```

Hoy (plugin `exo` aún no instalado — eso es la Task 8) el glob viejo resuelve
y el nuevo no. Con el shim sin tocar, esto se habría ido por la rama
`exit 0` en cuanto el rename llegara: commit permitido, gate mudo.

### Step 2 — cambio de criterio: permisivo → ruidoso, con fallback

Contenido nuevo de `kb-demo/.git/hooks/pre-commit`: prueba primero el
glob `exo/exo/*`, si no resuelve cae al glob `exo/reflex/*` **avisando por
stderr** («usando el plugin reflex (viejo) — migra a exo»), y solo si
**ninguno** de los dos resuelve sale `exit 1` con dos líneas explicando por
qué el gate no corrió. Antes, la ausencia de script terminaba en `exit 0`
silencioso; ahora requiere `--no-verify` explícito para saltarse el gate.

`plugins/exo/scripts/kb-precommit.sh` no citaba la ruta vieja ni el nombre
`reflex` en ningún comentario (comprobado con
`grep -n 'reflex\|cache/exo' kb-precommit.sh` → cero líneas): nada que tocar
ahí, más allá de lo que ya hizo la Task 3-bis dentro del plugin.

### Step 3 — los tres estados, verificados

**a) con el plugin viejo instalado (estado real de hoy):**

```
$ bash "$HOOK" </dev/null; echo "EXIT_A=$?"
pre-commit: usando el plugin reflex (viejo) — migra a exo
... (el gate de kbx corre, notier: <ficheros>, rechaza por motivos propios de kbx)
EXIT_A=1
```

El aviso de fallback sale. El `EXIT_A=1` de la invocación manual es el gate de
`kbx` (ratchet/budget) rechazando por su propia lógica — **preexistente**,
confirmado corriendo el shim de respaldo (el de antes de este cambio) en el
mismo estado del repo: mismo rechazo, mismo `notier:` listado, exit 1 igual.
No lo introduce este cambio.

**b) sin ningún plugin (`HOME` vacío) — el test que importa:**

```
$ HOME=/tmp/home-vacio bash "$HOOK" </dev/null; echo "EXIT_B=$?"
pre-commit: NO encuentro kb-precommit.sh de ningún plugin (exo ni reflex).
pre-commit: el gate de la KB NO ha corrido. Instala el plugin, o usa --no-verify a sabiendas.
EXIT_B=1
```

Antes daba `0` (commit permitido, gate mudo). Ahora da `1` con dos líneas en
stderr explicando por qué. Es la propiedad que esta task existe para cerrar.

**c) demostración con un commit real, no con la invocación manual:**

Cambio inocuo en `kb-demo/README.md` (una línea HTML comment), `git add`,
`git commit` de verdad:

```
$ git commit -m "test: verificar el shim del pre-commit tras Task 4 (se revierte)"
pre-commit: usando el plugin reflex (viejo) — migra a exo
[main a9f65e0] test: verificar el shim del pre-commit tras Task 4 (se revierte)
 1 file changed, 2 insertions(+)
```

El commit real **sí pasó** (a diferencia de la invocación manual del punto
(a), que usa un snapshot vía `checkout-index` distinto al índice real que
prepara `git commit`) — la KB sigue pudiendo commitear con el plugin viejo
como fallback, con el aviso de migración visible por stderr. Revertido acto
seguido con `git reset --hard b4a04e7` (el HEAD anterior a la prueba): la KB
queda sin commits nuevos y sin cambios sueltos, verificado con
`git log --oneline -3` y `git status --short` limpio tras el reset.

### Qué se conserva del shim viejo y por qué

- El comentario de por qué existe un shim y no un `ln -sf` (core.symlinks=false
  en esta máquina): se conserva literal, sigue siendo cierto.
- El `sort -V | tail -1` para quedarse con la versión más alta: se conserva,
  mismo criterio, ahora aplicado primero al glob `exo` y solo si falla al
  glob `reflex`.
- El fallback a `reflex` **no se retira** — es explícitamente el Step 6 de la
  Task 8, cuando las dos máquinas estén migradas al plugin `exo`. Retirarlo
  ahora habría dejado la KB bloqueada, porque el plugin `exo` todavía no está
  instalado en ninguna máquina.
- Lo que **no** se conserva: el criterio de "ausencia de script = commit
  permitido". Es exactamente el fallo silencioso que motiva esta task.

## Cutover — ejecutado el 2026-08-27 en W11

Orden real: **push → binario → plugin**. El Step 1½ (binario antes que plugin)
no era higiene: abajo está la medida de lo que evitó.

### Steps 1 y 1½ — push y binario

Tres repos empujados (`exo` 46 commits — los 31 de la ola 1A que nunca
salieron, más los de esta ola—, `exo-plugins` 1, `kb-demo` 1), los tres
fast-forward y con el árbol limpio.

El binario ya estaba compilado del merge de la ola 1A y **se verificó antes de
instalarlo**, no después:

```
$ target/release/exo.exe config --json | jq -r .data.kb.path
C:/proyectos/homework/kb-demo
$ target/release/exo.exe search --json probe | jq -r .schema_version
2
```

Respaldo del v1 en `~/.local/bin/exo-v1.exe` (24-08, 29.375.488 B) **antes** de
copiar. Tras instalar: `binario v2 OK`.

> **Nota de shell.** Los comandos del plan son de Git Bash. En PowerShell,
> `install` no existe (`Copy-Item … -Force`) y `&&` es error de parseo en PS 5.1
> (`; if ($?) { … }`). Y ojo: **el `cp` del respaldo no se repite** después de
> instalar el v2, o se sobrescribe la red de rollback con el binario nuevo.

### La ventana de degradación, medida en vivo

Entre instalar el binario y cambiar el plugin, los scripts v1 instalados leen
`.data.notas` / `.data.truncado` contra un binario que ya emite `notes` /
`truncated`. El log lo registra entero:

```
12:37:17  recall-inject-emitted    n_hits=3 bytes=986      <- sano: v1 + v1
12:39:04  recall-inject-degraded   reason=error err=envelope-ilegible   <- ventana
12:41:19  recall-inject-degraded   reason=error err=envelope-ilegible   <- ventana
12:41:54  recall-inject-emitted    n_hits=3 bytes=923      <- sano: v2 + v2
12:41:56  inject-emitted           type=exo:executor perfil=reducido bytes=1273
```

**Dos lecciones, opuestas y las dos útiles:**

1. **Aquí el instrumento gritó.** `recall-inject` no sirvió un bloque vacío: lo
   declaró con una razón precisa (`envelope-ilegible`). Contrasta con el
   `inject-emitted bytes=70` documentado arriba, que **no** grita. Mismo
   sistema, dos hooks, disciplinas distintas — y por eso el de arriba es deuda.
2. **El hook de arranque sobrevivió la ventana** (`exo-recall.sh` siguió
   sirviendo sus 5.921 B). Solo cayó el recall por prompt. Que una mitad
   aguante es precisamente lo que hace difícil notar la otra.

### Step 2 — el plugin

```
$ jq -r '.exo.source' ~/.claude/plugins/known_marketplaces.json
{ "source": "github", "repo": "pguerrerolinares/exo" }

$ jq -r '.plugins | keys[]' ~/.claude/plugins/installed_plugins.json
context7@claude-plugins-official
exo@exo
equipo-x@equipo-x-standards
```

`process@exo` y `reflex@exo` desaparecen de los instalados. El marketplace
apunta al repo `exo`, no a `exo-plugins.git`: **B2 ejecutado**.

**Desviación respecto al expected del plan.** El Step 2 esperaba que los
directorios `process/` y `reflex/` **desaparecieran** de la caché. No
desaparecen: siguen en disco `process/1.0.0/`, `reflex/0.16.0/` y
`reflex/0.17.0/` aunque ya no estén instalados. Consecuencia concreta: **el
fallback del shim sigue resolviendo**. Hoy es inocuo —el glob de `exo` gana
porque va primero— y además conviene para el rollback, así que **se dejan a
propósito** hasta el Step 6. Anotado para que nadie lo lea como un cutover a
medias.

### Step 3 — los hooks, contra el plugin nuevo

```
$ echo '{"session_id":"t","source":"startup"}' | bash $P/exo-recall.sh \
    | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
1                       # un 0 sería el fallback embebido: el arranque mentiría

recall-inject OK
subagent-inject OK
```

**La verificación post-cutover que esta ola añadió** —la que exige contenido y
no solo un evento— pasa:

```
$ echo '{"agent_type":"exo:executor"}' | bash $P/subagent-inject.sh \
    | jq -r '.hookSpecificOutput.additionalContext' | wc -c
1287                    # umbral: > 1000. Un 70 con `inject-emitted` sería el fallo
```

**Esto es lo que compró el Step 1½.** Con el plugin nuevo instalado antes que
el binario, `exo:executor` habría recibido 70 bytes de cabecera y el log habría
dicho `inject-emitted` — éxito aparente, doctrina ausente, en el agente que más
la necesita.

### Step 4 — el gate de la KB

```
$ ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh
/c/Users/paul/.claude/plugins/cache/exo/exo/1.0.0/scripts/kb-precommit.sh
```

El glob nuevo resuelve, así que el shim ya no usa el fallback. Y el script es
**idéntico** al que servía el plugin viejo (`diff -q` sin salida): lo movió un
`git mv`, no una reescritura, así que el gate no cambia de comportamiento.

### Pendiente

- **Step 5 — los nueve skills con prefijo `exo:`.** No se puede verificar desde
  la sesión que hizo el cutover: el harness resuelve el catálogo al arrancar.
  **Requiere sesión nueva.**
- **Step 6 — retirar el fallback a `reflex` del shim** y limpiar los tres
  directorios rancios de caché. Solo cuando **la máquina Linux** también esté
  migrada.

## Rollback

En este orden. La red existe entera: nada se ha borrado.

1. **Marketplace y plugins.**
   ```bash
   claude plugin uninstall exo@exo
   claude plugin marketplace remove exo
   claude plugin marketplace add pguerrerolinares/exo-plugins   # commit anterior a bdfbb02
   claude plugin install reflex@exo && claude plugin install process@exo
   ```
   Las cachés de `process/1.0.0` y `reflex/0.17.0` **siguen en disco**, así que
   esto es reversible incluso sin red.
2. **El binario.**
   ```bash
   cp ~/.local/bin/exo-v1.exe ~/.local/bin/exo.exe
   exo config --json     # debe fallar: 'unrecognized subcommand' == es el v1
   ```
   Sin esto quedan scripts v1 contra binario v2 — **la misma ventana de arriba,
   pero permanente**.
3. **El shim de la KB.** Restaurar desde
   `scratchpad/kb-demo-pre-commit.2026-08-27.bak`. Ojo: el shim viejo es el
   **permisivo** (`exit 0` si no encuentra el script). Volver a él reabre el
   gate en silencio; hazlo solo si de verdad vas a revertir la ola entera.
4. **Los repos.** Los tres commits están empujados. Revertir sería `git revert`,
   no `reset`: hay historia pública (privada, pero compartida entre dos
   máquinas).

