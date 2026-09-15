# Campaña F — Superficie publicable: docs vivos, gates estáticos y «genérico»

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no lo
> edita).

**Goal:** que lo que los cuatro ficheros «core» (`README.md`,
`docs/arquitectura.md`, `docs/instalacion.md`, `docs/backlog.md`) declaran lo
compruebe un gate de CI y no una persona, que el bash inline de los workflows
y `hooks.json` pasen por análisis estático como ya pasa el resto del
plugin, y que la cadena `Paul` hardcodeada en la lógica del plugin (no en
autoría/config) se sustituya por un término neutro.

**Architecture:** nueve tareas secuenciales en una sola rama (`campana-f`),
sin lanes paralelas — todas caben en un solo job de CI (`ci.yml`, el job
`lint`) salvo la Task 5 (motor de diagnóstico de `test-hermetico.sh`, sin
tocar `ci.yml`) y la Task 6 (toca además `release.yml`). Orden: (1) el texto
de `docs/arquitectura.md` que le falta la cifra held-out; (2)-(3) dos gates
estáticos nuevos (`test-docs-vivos.sh`, `test-hooks-json.sh`), cada uno
añadido como step del job `lint`; (4) extender `test-shellcheck.sh` a los
bloques `run: |` de los workflows; (5) que `test-hermetico.sh` distinga un
error de COMPILACIÓN de un test que falla; (6) fijar `HF_HOME`; (7) renombrar
el job `lint` (decisión de Paul, ya tomada) una vez que todos sus steps
nuevos están puestos, para que el nombre describa lo que el job hace
completo; (8) la limpieza «genérico»: `Paul` → «el dueño de la KB» en los
cinco ficheros de lógica que lo tienen, más la línea que falta en
`arquitectura.md` §3.8 sobre el idioma de los identificadores de código; (9)
sync de `docs/backlog.md`, que cierra lo que las ocho tareas anteriores
resuelven y corrige con cita los ítems caducados o mal registrados que
`§4` de la consulta de origen señaló.

Cada tarea deja el árbol verde antes de la siguiente — no hay tasks
paralelas dentro de este plan, así que los cuatro toques a `ci.yml` (Tasks 2,
3, 6, 7) se aplican en el orden escrito sin conflicto de fichero.

**Tech Stack:** bash (Git Bash en Windows) + `jq` 1.8.2 + `awk` · GitHub
Actions (`.github/workflows/ci.yml`, `release.yml`) · Rust 2024 (crate `exo`
en `engine/`, MSRV 1.95) solo como fuente de verdad estática que los gates
leen (`enum Comando`/`enum ComandoWrite` de `engine/src/main.rs`) — ningún
`.rs` se modifica en este plan. ShellCheck 0.11.0 (pineado, solo corre en el
runner `ubuntu-latest` del job renombrado en la Task 7; no está instalado en
esta máquina Windows — ver Global Constraints).

## Global Constraints

Todas verificadas hoy (2026-09-15) contra `99ddd05` en el worktree
`C:/proyectos/homework/exo-wt/campana-f`.

- **El crate vive en `engine/`, no en la raíz.** No hay workspace. **Esta
  campaña no toca ni un byte de `engine/src/`** — Task 5 toca
  `engine/scripts/test-hermetico.sh` (bash), no Rust.
- **MSRV `rust-version = "1.95"`** (`engine/Cargo.toml:8`), comprobada por el
  job `msrv`. No se toca.
- **jq en esta máquina (Windows 11 + Git Bash) emite CRLF.** Medido:
  `jq -r '.version' plugins/exo/.claude-plugin/plugin.json | od -c` termina en
  `\r\n`, no solo `\n` (jq 1.8.2, build de Windows en
  `/c/Users/aaepgg/.local/bin/jq`). Cualquier salida de `jq` que se compare
  contra un literal o se recorra línea a línea DEBE pasar por `tr -d '\r'`
  antes de usarse — sin eso, una comparación como
  `[ "$evento" = "PreToolUse" ]` falla en silencio con un `\r` pegado al
  final. Las Tasks 2 y 3 lo aplican explícitamente; queda anotado aquí porque
  es la clase de bug que un ejecutor en Linux/macOS nunca vería y que en esta
  máquina rompe a la primera.
- **ShellCheck no está instalado localmente en esta máquina** (`command -v
  shellcheck` no resuelve nada; tampoco hay build de Windows en
  `~/.local/bin` ni vía choco). El único sitio donde el binario pineado
  v0.11.0 corre de verdad es el job de CI en `ubuntu-latest`
  (`ci.yml`, descarga el tarball `.linux.x86_64.tar.xz`, que no ejecuta en
  Windows). Consecuencia para la Task 4: la verificación local del ejecutor
  se limita a `bash -n` sobre los bloques `run: |` extraídos (confirma que la
  extracción no rompe la sintaxis) — el veredicto de ShellCheck en sí se lee
  en el run de CI tras el push, no antes. Esto no es una laguna del plan: es
  el estado real de la máquina, igual que ya advertía el brief.
- **`.gitattributes`: `* text=auto eol=lf`.** Nada de normalizar finales de
  línea a mano; los ficheros nuevos de este plan (`scripts/test-docs-vivos.sh`,
  `scripts/test-hooks-json.sh`) se crean con LF (el `Write`/`Edit` del
  ejecutor ya usa LF por defecto en este repo — confirmar con
  `git diff --stat` que no aparece como binario o con CRLF si algo raro pasa).
- **Git:** `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push ni de tocar GitHub. La branch
  protection de `main` la activa Paul en GitHub, no esta campaña (ver Task 7).
- **Scripts nuevos bajo `scripts/` (no `plugins/`) no exigen bit de
  ejecución**: el gate `test-exec-bit.sh` solo mira `git ls-files -s --
  plugins` (`scripts/test-exec-bit.sh:34`) — los dos scripts nuevos de este
  plan (`scripts/test-docs-vivos.sh`, `scripts/test-hooks-json.sh`) se
  invocan como `bash scripts/x.sh` desde `ci.yml` (igual que
  `test-versiones.sh`, `test-rutas-personales.sh`, `test-shellcheck.sh` ya
  existentes), así que el modo 100644 de por sí no rompe nada; se les pone
  755 solo por consistencia con sus hermanos, no porque un gate lo exija.
- **Restricción de la ola (matriz de colisión, `propuesta.md` §3): F ∥ G ∥
  H en tres ramas; orden de merge H → F → G.** F comparte con H estos
  ficheros y en ellos cambia SOLO texto/comentarios, nunca lógica, para que
  el rebase de F sobre H sea trivial:
  - `plugins/exo/scripts/kb-precommit.sh` (H lo hace fail-closed; F solo
    cambia la palabra `Paul` en el mensaje de la línea 63, Task 8).
  - `plugins/exo/scripts/exo-recall.sh` y `recall-inject.sh` (H añade el
    check de versión mínima del engine; F solo cambia `Paul` en dos
    comentarios de `recall-inject.sh`, líneas 2 y 7, Task 8 — F NO toca
    `exo-recall.sh`, que no tiene ninguna ocurrencia de `Paul`).
  - F **no toca** `engine/src/main.rs`, `scripts/test-versiones.sh`,
    `plugin.json`, `marketplace.json`, `scripts/test-versiones.sh` (todos son
    G o H). F tampoco toca `hooks.json` como CONTENIDO — Task 3 solo lo LEE
    para validarlo.
- **`test-hermetico.sh` es gate demostrado falsable (2026-08-27,
  re-demostrado por E en 2026-09-14).** Esta campaña SÍ lo modifica (Task 5,
  asignación explícita del brief) — igual que hizo E, rehaciendo su ciclo
  rojo-verde en el mismo commit que lo toca, nunca dando por bueno un cambio
  sin volver a provocar el rojo.
- **Los cuatro ficheros «core»** (`README.md`, `docs/arquitectura.md`,
  `docs/instalacion.md`, `docs/backlog.md`) están definidos en
  `docs/arquitectura.md:9-13`. **Corrección de alcance sobre la propuesta de
  origen**: el gate de la Task 2 (frases muertas, subcomandos citados,
  versiones citadas) se aplica solo a los **tres declarativos**
  (`README.md`, `docs/arquitectura.md`, `docs/instalacion.md`), NO a
  `docs/backlog.md` — verificado que `docs/backlog.md` cita a propósito
  frases ya muertas como evidencia de items cerrados (p.ej. «Sin CI» en
  `docs/backlog.md:249`, dentro de un ítem `[x]`) y una versión de un repo
  ajeno (`v2.2.1` en `docs/backlog.md:126`, el clon de ECC, no un tag de
  exo). Meter `docs/backlog.md` en ese gate lo pondría en rojo permanente por
  su propio diseño de ledger histórico. Ver razonamiento completo en la
  Task 2.

## Decisiones de Paul ya tomadas (2026-09-15), heredadas por toda tarea

- **#8**: renombrar el job `lint` de `ci.yml` (Task 7). Branch protection la
  activa Paul en GitHub después del merge; la Task 7 deja la lista exacta de
  los 12 required checks resultantes.
- **#5**: identificadores de código en español, claves JSON y flags en
  inglés — se escribe en una línea en `docs/arquitectura.md` §3.8 (Task 8).
  Cero rename de código.
- **#14**: `docs/superpowers/` se queda. **Verificado que la frase que la
  propuesta pedía YA EXISTE**: `docs/arquitectura.md:5-13` dice «Las specs y
  planes de `docs/superpowers/` son el registro histórico de diseño... Todo
  lo que lleva fecha en el nombre y todo `docs/superpowers/` son
  instantáneas (`tier: log` por convención de ruta): no se actualizan, se
  citan con su fecha.» — escrita por la campaña B (D5=b, commit `2294349`).
  Task 8 solo cita esta evidencia; no edita nada.
- **«Genérico»**: `Paul` → «el dueño de la KB» en los ficheros de lógica del
  plugin. Task 8 explica por qué es un término estático y no una resolución
  dinámica (a diferencia de `kb-demo`, que si resuelve dinámicamente vía
  `exo config --json`).

---

## Task 1: `docs/arquitectura.md` lleva la cifra held-out junto al 48/55

**Lane:** mecánica. **Depende de otras tasks de F:** no. **Oráculo:**
`grep -c "64/92" docs/arquitectura.md` (esperado: 0 antes, 2 después) y
verificación visual de que ninguna mención de «48/55» queda sin la
advertencia de no-comparabilidad en el mismo párrafo.

**Evidencia:** `docs/backlog.md:1136-1144` («README y `docs/arquitectura.md`
§6 citan el 48/55 sin la cifra held-out»). Verificado hoy: `README.md` NO
contiene «48/55» (`grep -c "48/55" README.md` → vacío, ya lo retiró
`c5c5b7f` según el propio backlog) — el ítem solo tiene alcance real en
`docs/arquitectura.md:201` (§3.5) y `:492` (§6). La cifra held-out está
firmada en `evals/retrieval-heldout/verdict/c-verdict.md:34`: «hit@5 absoluto
de A0: **64/92 = 69,6 %**, Wilson 95 % **[59,5 %, 78,0 %]**» y su advertencia
de no-comparabilidad en la línea 52: «El nivel absoluto del hit@5 de A0 en el
held-out se reporta con su Wilson y **no se compara con 48/55**: la fuente de
las queries cambia (§7) y la diferencia mezclaría sobreajuste con cambio de
distribución.»

**Files:**
- Modify: `docs/arquitectura.md:198-204` (§3.5, dentro de «Pipeline de
  búsqueda»)
- Modify: `docs/arquitectura.md:488-495` (§6, dentro de «Cómo se mide la
  calidad del retrieval»)

**Interfaces:** ninguna — solo texto. No hay código que consumir ni producir.

- [ ] **Step 1: confirmar el rojo**

Run: `grep -c "64/92" docs/arquitectura.md`
Expected: `0` (la cifra held-out no aparece todavía en el documento vivo).

- [ ] **Step 2: editar §3.5**

`Edit` sobre `docs/arquitectura.md`:

old_string:
```
`exo search` tiene tres modos (`--type fts|vector|hybrid`, default `fts`),
implementados en `engine/src/buscador.rs`. Todos devuelven resultados
**a nivel de nota** (`type: "entity"`), nunca de trozo. Ojo con el default:
el modo calibrado y medido (48/55 hit@5, §6) es `--type hybrid` **con el
umbral pasado explícito** (`--min-similarity 0.40`); `fts` a secas es el modo
léxico barato, no el medido. `exo recall --query` sí usa hybrid con los
parámetros sellados de serie.
```

new_string:
```
`exo search` tiene tres modos (`--type fts|vector|hybrid`, default `fts`),
implementados en `engine/src/buscador.rs`. Todos devuelven resultados
**a nivel de nota** (`type: "entity"`), nunca de trozo. Ojo con el default:
el modo calibrado y medido (48/55 hit@5 **in-sample**, §6; held-out
**64/92**, Wilson 95 % [59,5 %, 78,0 %], **no comparable** con el 48/55 —
distinta fuente de queries, §6) es `--type hybrid` **con el umbral pasado
explícito** (`--min-similarity 0.40`); `fts` a secas es el modo léxico
barato, no el medido. `exo recall --query` sí usa hybrid con los parámetros
sellados de serie.
```

- [ ] **Step 3: editar §6**

`Edit` sobre `docs/arquitectura.md`:

old_string:
```
- **`evals/e1-read/`**: el gate de cierre de la capa de lectura. Tres patas:
  paridad de corpus (diff de permalinks = ∅, cero tolerancia), retrieval
  pareado engine vs basic-memory el mismo día sobre el mismo estado de la KB, y
  latencia. Corrida final (2026-08-17, `verdict/m2-09-corrida.md`):
  **engine-hybrid 48/55 vs bm-hybrid 39/55**. Los parámetros sellados del
  hybrid (§3.5) salen del sweep de 15 celdas cuyos resultados están en
  `retrieval-fase0/results/metrics-engine-hybrid-*`.
- **`evals/prep-m3/`**: eval de otra naturaleza — paridad de **movimientos**
```

new_string:
```
- **`evals/e1-read/`**: el gate de cierre de la capa de lectura. Tres patas:
  paridad de corpus (diff de permalinks = ∅, cero tolerancia), retrieval
  pareado engine vs basic-memory el mismo día sobre el mismo estado de la KB, y
  latencia. Corrida final (2026-08-17, `verdict/m2-09-corrida.md`):
  **engine-hybrid 48/55 vs bm-hybrid 39/55**. Los parámetros sellados del
  hybrid (§3.5) salen del sweep de 15 celdas cuyos resultados están en
  `retrieval-fase0/results/metrics-engine-hybrid-*`. Este 48/55 es
  **in-sample**: los parámetros se eligieron sobre las mismas 55 queries que
  lo reportan.
- **`evals/retrieval-heldout/`** (campaña C, 2026-09-14): held-out real sobre
  92 queries nuevas, nunca vistas por quien fijó los parámetros del hybrid.
  hit@5 de A0 (el binario sellado): **64/92 = 69,6 %**, Wilson 95 %
  **[59,5 %, 78,0 %]** (`verdict/c-verdict.md`). **No se compara con el
  48/55**: cambian a la vez la fuente de las queries, la KB (138→174 notas,
  con rotaciones a `archive/`) y el binario — mezclaría sobreajuste con
  cambio de distribución. El held-out queda **consumido**: cualquier cambio
  de fusión, umbral o troceado exige un gold nuevo (`c-verdict.md` §11).
- **`evals/prep-m3/`**: eval de otra naturaleza — paridad de **movimientos**
```

- [ ] **Step 4: verlo verde**

Run: `grep -c "64/92" docs/arquitectura.md`
Expected: `2`.

Run: `grep -n "48/55" docs/arquitectura.md`
Expected: dos líneas (dentro de §3.5 y §6), cada una en un párrafo que
también contiene «64/92» — confírmalo leyendo las dos coincidencias, no solo
contando.

- [ ] **Step 5: commit**

```bash
git add docs/arquitectura.md
git commit -m "docs(f, arquitectura): añade el held-out 64/92 junto al 48/55 in-sample, con la advertencia de no-comparabilidad"
```

---

## Task 2: gate `scripts/test-docs-vivos.sh` — los tres docs declarativos no mienten

**Lane:** mecánica. **Depende de Task 1:** no directamente, pero se ejecuta
después para que el gate ya vea el `docs/arquitectura.md` corregido.
**Oráculo:** `bash scripts/test-docs-vivos.sh` verde tras el fix de README, y
en rojo demostrado antes de ese fix y con una frase muerta reinsertada.

**Evidencia:** `docs/backlog.md:247-298` (acción c, «grep de afirmaciones
frágiles... acotado a los cuatro ficheros core») y `docs/backlog.md:596-635`
(la misma acción c, «compartida con la del item de Alta arriba»). Verificado
hoy: NINGÚN gate existente comprueba esto — `test-versiones.sh` solo mira
números de versión entre `plugin.json`/`marketplace.json`/`Cargo.toml`, y
`test-docs-vivos.sh` no existe.

**Corrección de alcance frente a la propuesta de origen** (verificado
ejecutando un borrador del gate contra el árbol real, dos veces): el diseño
"sobre los 4 core" produce falsos positivos reales si se aplica literal:

1. `docs/backlog.md` cita a propósito frases ya muertas como evidencia de
   ítems cerrados (`docs/backlog.md:249`, dentro de un ítem `[x]`: «Sin CI»)
   y una versión de un repo AJENO (`docs/backlog.md:126`: `v2.2.1`, el clon
   de ECC citado en el análisis de M5a, no un tag de exo). Es un ledger
   histórico por diseño, no una declaración de estado actual — meterlo en
   los checks (a)-(c) de abajo lo pone en rojo permanente por su propia
   naturaleza.
2. `docs/instalacion.md:201-209`, sección «7. Lo que NO hay todavía», cita
   `exo diff-since` y `exo history` **para decir que nunca existirán** («No
   se portan por decisión: se usan `git diff`/`git log` directamente»). Un
   check ingenuo de «todo subcomando citado debe existir» dispara aquí un
   falso positivo: la cita es correcta precisamente porque describe algo que
   NO existe a propósito.

Por eso el gate de esta tarea: (a)-(c) corren solo sobre `README.md`,
`docs/arquitectura.md`, `docs/instalacion.md`; (b) excluye las secciones
cuyo encabezado `## ` contiene ` NO ` (con espacios) — convención ya en uso
en este repo para las dos secciones de "lo que falta" (`arquitectura.md:505`
«Qué NO está implementado», `instalacion.md:201` «Lo que NO hay todavía»,
verificado que son las ÚNICAS dos cabeceras `## ` de los tres docs que
contienen ` NO `); (d) sí corre sobre los tres declarativos, sin
`docs/backlog.md`.

**Hallazgo real durante la construcción del gate** (ejecutado el borrador
contra el árbol real): `README.md:117-125` cita los nueve hooks del plugin
como `` `scripts/<x>.sh` `` — ruta que **no existe**; los scripts viven en
`plugins/exo/scripts/<x>.sh`. Es exactamente la clase de deriva documental
que esta campaña ataca. Se corrige en el Step 3 de esta misma tarea (el gate
nace, ve el rojo real, y el fix lo pone verde — no se desactiva el check
para que pase).

**Files:**
- Create: `scripts/test-docs-vivos.sh`
- Modify: `README.md:117-125` (rutas de los hooks)
- Modify: `.github/workflows/ci.yml` (job `lint`, step nuevo)

**Interfaces:**
- Consumes: `enum Comando` y `enum ComandoWrite` de `engine/src/main.rs`
  (parseo estático de texto — el job `lint` no compila el release, así que
  no hay `exo --help` que ejecutar); `engine/Cargo.toml` (`version = "..."`);
  `plugins/exo/.claude-plugin/plugin.json` (`.version`); `git tag -l`.
- Produces: exit 0/1 con mensajes `[FAIL] <doc> ...` por stderr y
  `[OK] test-docs-vivos` por stdout si limpio. Task 9 no depende de nada de
  esta tarea salvo el hecho de que el gate exista (lo cita en el sync).

- [ ] **Step 1: escribir el gate (todavía sin wire a CI) y verlo fallar por
  la razón real (README con rutas rotas)**

Crea `scripts/test-docs-vivos.sh`:

```bash
#!/usr/bin/env bash
# Gate: los tres documentos declarativos de producto (README.md,
# docs/arquitectura.md, docs/instalacion.md) no afirman lo que el repo ya no
# hace. `docs/backlog.md` es el cuarto "core" (arquitectura.md, "Qué
# documentación es viva") pero es un LEDGER histórico: cita a propósito
# frases ya muertas como evidencia de ítems cerrados (p.ej. "Sin CI" en
# backlog.md:249, dentro de un ítem [x]) y versiones de repos ajenos citados
# en sus consultas (backlog.md:126, "v2.2.1" del clon de ECC). Meterlo en
# los checks de abajo lo pondría en rojo permanente por su propio diseño —
# sus afirmaciones frágiles las cierra un humano al cerrar el ítem
# (backlog.md:283-285), no este gate.
#
# Cuatro comprobaciones sobre README.md / docs/arquitectura.md /
# docs/instalacion.md:
#   (a) ausencia de frases muertas conocidas
#   (b) todo `exo <subcomando>` citado existe en enum Comando de main.rs
#       (y `exo write <x>` en enum ComandoWrite) — excluye las secciones
#       "## ... NO ..." (con espacios), que citan a propósito lo que NO
#       existe (docs/instalacion.md §7, "Lo que NO hay todavía")
#   (c) toda versión `vX.Y.Z` citada es un tag, o coincide con
#       engine/Cargo.toml o con plugin.json
#   (d) todo enlace `` `docs/...` ``, `` `evals/...` ``, `` `scripts/...` ``
#       entre backticks resuelve a un fichero o directorio existente
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

FALLOS=0
DOCS=(README.md docs/arquitectura.md docs/instalacion.md)

# --- (a) Frases muertas conocidas ------------------------------------------
# Lista curada (docs/backlog.md:247-298), no heurística: un falso positivo
# nuevo se añade a mano.
FRASES_MUERTAS=(
  "Sin CI"
  "no hay ningún tag"
  "viven en una rama sin mergear"
)
for doc in "${DOCS[@]}"; do
  for frase in "${FRASES_MUERTAS[@]}"; do
    if grep -n -F "$frase" "$doc" >/dev/null 2>&1; then
      echo "[FAIL] $doc afirma una frase muerta: \"$frase\"" >&2
      grep -n -F "$frase" "$doc" | sed "s|^|  $doc:|" >&2
      FALLOS=1
    fi
  done
done

# Quita el contenido de cualquier sección `## ...` cuyo título contenga
# " NO " (con espacios) — convención de este repo para "lo que falta"
# (arquitectura.md §7, instalacion.md §7): ahí se cita a propósito lo que
# NO existe, y (b) no debe dispararse con eso.
sin_secciones_negativas() {
  awk '/^## / { neg = ($0 ~ / NO /) } !neg { print }' "$1"
}

# --- (b) Subcomandos citados existen en el binario -------------------------
# Fuente de verdad: enum Comando / enum ComandoWrite de engine/src/main.rs
# (parseo estático — este gate corre en el job `lint`, que no compila el
# release; sin binario no hay `exo --help` que leer).
COMANDOS="$(sed -n '/^enum Comando {/,/^}/p' engine/src/main.rs \
  | grep -oE '^    [A-Z][A-Za-z]+\(' | tr -d ' (' | tr '[:upper:]' '[:lower:]' | sort -u)"
SUBCOMANDOS_WRITE="$(sed -n '/^enum ComandoWrite {/,/^}/p' engine/src/main.rs \
  | grep -oE '^    [A-Z][A-Za-z]+\(' | tr -d ' (' | tr '[:upper:]' '[:lower:]' | sort -u)"

for doc in "${DOCS[@]}"; do
  while IFS= read -r cmd; do
    [ -n "$cmd" ] || continue
    if ! printf '%s\n' "$COMANDOS" | grep -qx "$cmd"; then
      echo "[FAIL] $doc cita \`exo $cmd\`, que no existe en enum Comando de main.rs" >&2
      FALLOS=1
    fi
  done < <(sin_secciones_negativas "$doc" | grep -oE '`exo [a-z][a-z-]*' | sed 's/`exo //' | sort -u)
  while IFS= read -r sub; do
    [ -n "$sub" ] || continue
    if ! printf '%s\n' "$SUBCOMANDOS_WRITE" | grep -qx "$sub"; then
      echo "[FAIL] $doc cita \`exo write $sub\`, que no existe en enum ComandoWrite" >&2
      FALLOS=1
    fi
  done < <(sin_secciones_negativas "$doc" | grep -oE '`exo write [a-z]+' | sed 's/`exo write //' | sort -u)
done

# --- (c) Versiones citadas existen -----------------------------------------
ENGINE_VER="$(sed -n 's/^version = "\(.*\)"$/\1/p' engine/Cargo.toml | head -n1)"
PLUGIN_VER="$(jq -r '.version' plugins/exo/.claude-plugin/plugin.json | tr -d '\r')"
TAGS="$(git tag -l)"
for doc in "${DOCS[@]}"; do
  while IFS= read -r v; do
    [ -n "$v" ] || continue
    if [ "$v" = "v$ENGINE_VER" ] || [ "$v" = "v$PLUGIN_VER" ] || printf '%s\n' "$TAGS" | grep -qx "$v"; then
      continue
    fi
    echo "[FAIL] $doc cita $v, que no es un tag ni coincide con engine ($ENGINE_VER) ni plugin ($PLUGIN_VER)" >&2
    FALLOS=1
  done < <(grep -oE '\bv[0-9]+\.[0-9]+\.[0-9]+\b' "$doc" | sort -u)
done

# --- (d) Enlaces relativos a docs/, evals/, scripts/ resuelven -------------
for doc in "${DOCS[@]}"; do
  while IFS= read -r ruta; do
    [ -n "$ruta" ] || continue
    destino="${ruta%/}"
    if [ ! -e "$destino" ]; then
      echo "[FAIL] $doc cita \`$ruta\`, que no existe" >&2
      FALLOS=1
    fi
  done < <(grep -oE '`(docs|evals|scripts)/[A-Za-z0-9_./-]*`' "$doc" | tr -d '`' | sort -u)
done

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas ni enlaces rotos"
fi
exit "$FALLOS"
```

Run: `chmod +x scripts/test-docs-vivos.sh && bash scripts/test-docs-vivos.sh`
Expected (rojo real, sin tocar nada más todavía):
```
[FAIL] README.md cita `scripts/clean-orchestrator-research.sh`, que no existe
[FAIL] README.md cita `scripts/document-remind.sh`, que no existe
[FAIL] README.md cita `scripts/exo-index.sh`, que no existe
[FAIL] README.md cita `scripts/exo-recall.sh`, que no existe
[FAIL] README.md cita `scripts/git-add-all-guard.sh`, que no existe
[FAIL] README.md cita `scripts/git-c-bash.sh`, que no existe
[FAIL] README.md cita `scripts/recall-inject.sh`, que no existe
[FAIL] README.md cita `scripts/subagent-inject.sh`, que no existe
[FAIL] README.md cita `scripts/verify-before-commit.sh`, que no existe
```
(9 fallos — exactamente los 9 hooks de la tabla de `README.md:117-125`. Nada
de check (a)/(b)/(c) dispara: confírmalo leyendo que no hay más líneas
`[FAIL]` que esas 9.)

- [ ] **Step 2: confirmar que (a) SÍ es falsable (rojo provocado, no solo
  ausencia de casos)**

Run:
```bash
printf '\nSin CI\n' >> docs/arquitectura.md
bash scripts/test-docs-vivos.sh 2>&1 | grep "afirma una frase muerta"
git checkout -- docs/arquitectura.md
```
Expected: `[FAIL] docs/arquitectura.md afirma una frase muerta: "Sin CI"`,
luego el `git checkout` deja `docs/arquitectura.md` limpio de nuevo
(confirma con `git status --short docs/arquitectura.md` → vacío).

- [ ] **Step 3: arreglar el hallazgo real — rutas de los hooks en README**

`Edit` sobre `README.md` (nueve reemplazos, uno por fila de la tabla;
`replace_all` no sirve porque cada ruta es distinta — usa el bloque completo
de la tabla):

old_string:
```
| Reflejo | Evento | Fichero |
|---|---|---|
| clean-orchestrator | `PreToolUse:WebSearch\|WebFetch\|navegación MCP` | `scripts/clean-orchestrator-research.sh` |
| git-c | `PreToolUse:Bash` | `scripts/git-c-bash.sh` |
| zero-residuo | `PreToolUse:Bash` | `scripts/git-add-all-guard.sh` |
| verify-before-done | `PreToolUse:Bash` | `scripts/verify-before-commit.sh` |
| exo-recall | `SessionStart` | `scripts/exo-recall.sh` |
| document-remind | `Stop` | `scripts/document-remind.sh` |
| exo-index | `Stop` | `scripts/exo-index.sh` |
| subagent-inject | `SubagentStart` | `scripts/subagent-inject.sh` |
| recall-inject | `UserPromptSubmit` | `scripts/recall-inject.sh` |
```

new_string:
```
| Reflejo | Evento | Fichero |
|---|---|---|
| clean-orchestrator | `PreToolUse:WebSearch\|WebFetch\|navegación MCP` | `plugins/exo/scripts/clean-orchestrator-research.sh` |
| git-c | `PreToolUse:Bash` | `plugins/exo/scripts/git-c-bash.sh` |
| zero-residuo | `PreToolUse:Bash` | `plugins/exo/scripts/git-add-all-guard.sh` |
| verify-before-done | `PreToolUse:Bash` | `plugins/exo/scripts/verify-before-commit.sh` |
| exo-recall | `SessionStart` | `plugins/exo/scripts/exo-recall.sh` |
| document-remind | `Stop` | `plugins/exo/scripts/document-remind.sh` |
| exo-index | `Stop` | `plugins/exo/scripts/exo-index.sh` |
| subagent-inject | `SubagentStart` | `plugins/exo/scripts/subagent-inject.sh` |
| recall-inject | `UserPromptSubmit` | `plugins/exo/scripts/recall-inject.sh` |
```

- [ ] **Step 4: verlo verde**

Run: `bash scripts/test-docs-vivos.sh`
Expected: `[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas, subcomandos inventados, versiones huérfanas ni enlaces rotos`
(exit 0).

- [ ] **Step 5: wire a CI**

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```
      - name: Sin rutas de una máquina concreta en el bash versionado
        if: always()
        run: bash scripts/test-rutas-personales.sh
```

new_string:
```
      - name: Sin rutas de una máquina concreta en el bash versionado
        if: always()
        run: bash scripts/test-rutas-personales.sh
      - name: Documentación viva (README/arquitectura/instalación no mienten)
        if: always()
        run: bash scripts/test-docs-vivos.sh
```

- [ ] **Step 6: commit**

```bash
git add scripts/test-docs-vivos.sh README.md .github/workflows/ci.yml
git commit -m "ci(f, docs-vivos): gate test-docs-vivos.sh sobre README/arquitectura/instalación; arregla las rutas de los hooks en README"
```

---

## Task 3: gate `scripts/test-hooks-json.sh` — eventos válidos, `type=command`, scripts 100755

**Lane:** mecánica. **Depende de otras tasks de F:** no. **Oráculo:**
`bash scripts/test-hooks-json.sh` verde; rojo demostrado renombrando un
script citado.

**Evidencia:** `docs/backlog.md:649-682`, sub-propuesta 2 («segundo robable
de la misma cadena: `scripts/ci/validate-hooks.js`... `hooks/hooks.json`
tiene nueve hooks y ninguna validación»). Verificado hoy: `plugins/exo/hooks/hooks.json`
tiene los 5 eventos `PreToolUse`, `SessionStart`, `Stop`, `SubagentStart`,
`UserPromptSubmit`, con 9 hooks `type: "command"` en total. Nada lo valida.

**Diseño, distinto del precedente de ECC citado por el backlog** (Ajv +
`schemas/hooks.schema.json`, un `.js` con dependencia npm): este repo no
tiene Node ni npm en ningún gate — todo el plugin es bash + jq. En vez de
portar un validador de JSON Schema, el gate hace las mismas comprobaciones
concretas con `jq`: eventos conocidos, `type=="command"`, el script referenciado
existe y está en 100755 en el índice de git (la misma clase de bug que ya
cazó `test-exec-bit.sh` para el resto de scripts). Cierra la sub-propuesta 2
sin dependencia nueva.

**Hallazgo de esta máquina, ya anotado en Global Constraints**: `jq` en
Windows/Git Bash emite CRLF — cada línea que sale de `jq -r` y se compara o
se recorre en un `while read` necesita `tr -d '\r'` o la comparación falla en
silencio (medido: sin el `tr -d`, el gate declaraba "PreToolUse" un evento
desconocido, porque en realidad comparaba `"PreToolUse\r"` contra la lista
de eventos válidos).

**Files:**
- Create: `scripts/test-hooks-json.sh`
- Modify: `.github/workflows/ci.yml` (job `lint`, step nuevo)

**Interfaces:**
- Consumes: `plugins/exo/hooks/hooks.json` (solo lectura); `git ls-files -s`
  para el modo del índice.
- Produces: exit 0/1, mensajes `[FAIL] plugins/exo/hooks/hooks.json: ...`.

- [ ] **Step 1: escribir el gate y verlo pasar sobre el hooks.json real**

Crea `scripts/test-hooks-json.sh`:

```bash
#!/usr/bin/env bash
# Gate: plugins/exo/hooks/hooks.json no referencia eventos desconocidos,
# hooks de tipo distinto de "command", ni scripts que no existen o no están
# en 100755 — la clase de bug que ya cazó test-exec-bit.sh para el resto del
# plugin, aplicada a lo que hooks.json declara.
#
# Sin dependencia nueva: en vez de JSON Schema + Ajv (precedente de ECC,
# backlog.md:649-682, que exige Node/npm — ausentes de este repo), las
# mismas comprobaciones concretas con jq, que ya es una dependencia del
# plugin.
#
# jq en Windows/Git Bash emite CRLF: cada salida que se compara o se lee
# línea a línea pasa por `tr -d '\r'` — sin eso, "PreToolUse\r" no es igual
# a "PreToolUse" y el gate falla en todo, no en nada.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

HOOKS=plugins/exo/hooks/hooks.json
command -v jq >/dev/null 2>&1 || { echo "test-hooks-json: jq requerido" >&2; exit 1; }
[ -f "$HOOKS" ] || { echo "test-hooks-json: no existe $HOOKS" >&2; exit 1; }
jq -e . "$HOOKS" >/dev/null 2>&1 || { echo "test-hooks-json: $HOOKS no es JSON válido" >&2; exit 1; }

FALLOS=0
EVENTOS_VALIDOS="PreToolUse SessionStart Stop SubagentStart UserPromptSubmit"

for evento in $(jq -r '.hooks | keys[]' "$HOOKS" | tr -d '\r'); do
  if ! printf '%s\n' "$EVENTOS_VALIDOS" | tr ' ' '\n' | grep -qx "$evento"; then
    echo "[FAIL] $HOOKS: evento desconocido '$evento'" >&2
    FALLOS=1
  fi
done

while IFS= read -r tipo; do
  if [ "$tipo" != "command" ]; then
    echo "[FAIL] $HOOKS: un hook con type='$tipo' (solo se soporta 'command')" >&2
    FALLOS=1
  fi
done < <(jq -r '.hooks[][].hooks[].type' "$HOOKS" | tr -d '\r')

while IFS= read -r cmd; do
  script="$(printf '%s' "$cmd" | grep -oE '\$\{CLAUDE_PLUGIN_ROOT\}"?/scripts/[A-Za-z0-9_.-]+' | sed -E 's#.*/scripts/##')"
  if [ -z "$script" ]; then
    echo "[FAIL] $HOOKS: command sin \${CLAUDE_PLUGIN_ROOT}/scripts/<x> reconocible: $cmd" >&2
    FALLOS=1
    continue
  fi
  ruta="plugins/exo/scripts/$script"
  if [ ! -f "$ruta" ]; then
    echo "[FAIL] $HOOKS: $ruta no existe (citado por hooks.json)" >&2
    FALLOS=1
    continue
  fi
  modo="$(git ls-files -s -- "$ruta" | awk '{print $1}')"
  if [ "$modo" != "100755" ]; then
    echo "[FAIL] $HOOKS: $ruta no está en 100755 en el índice (modo=$modo)" >&2
    FALLOS=1
  fi
done < <(jq -r '.hooks[][].hooks[].command' "$HOOKS" | tr -d '\r')

if [ "$FALLOS" -eq 0 ]; then
  echo "[OK] test-hooks-json: eventos, type=command y scripts referenciados, todos 100755"
fi
exit "$FALLOS"
```

Run: `chmod +x scripts/test-hooks-json.sh && bash scripts/test-hooks-json.sh`
Expected: `[OK] test-hooks-json: eventos, type=command y scripts referenciados, todos 100755`
(exit 0 — verificado hoy contra el `hooks.json` real: 5 eventos válidos, 9
hooks `type=="command"`, 9 scripts existentes y 100755).

- [ ] **Step 2: rojo provocado (renombrar un script citado)**

Run:
```bash
cp plugins/exo/hooks/hooks.json /tmp/hooks-backup.json
sed -i 's/exo-recall\.sh/exo-recall-renombrado.sh/' plugins/exo/hooks/hooks.json
bash scripts/test-hooks-json.sh
cp /tmp/hooks-backup.json plugins/exo/hooks/hooks.json
git status --short plugins/exo/hooks/hooks.json
```
Expected: `[FAIL] plugins/exo/hooks/hooks.json: plugins/exo/scripts/exo-recall-renombrado.sh no existe (citado por hooks.json)`
(exit 1), y tras restaurar el fichero, `git status --short` sobre
`hooks.json` sale vacío.

- [ ] **Step 3: wire a CI**

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```
      - name: Documentación viva (README/arquitectura/instalación no mienten)
        if: always()
        run: bash scripts/test-docs-vivos.sh
```

new_string:
```
      - name: Documentación viva (README/arquitectura/instalación no mienten)
        if: always()
        run: bash scripts/test-docs-vivos.sh
      - name: hooks.json — eventos válidos, type=command, scripts en 100755
        if: always()
        run: bash scripts/test-hooks-json.sh
```

- [ ] **Step 4: commit**

```bash
git add scripts/test-hooks-json.sh .github/workflows/ci.yml
git commit -m "ci(f, hooks-json): gate test-hooks-json.sh — eventos válidos, type=command, scripts existentes y 100755"
```

---

## Task 4: extender `scripts/test-shellcheck.sh` a los bloques `run: |` de los workflows

**Lane:** mecánica. **Depende de otras tasks de F:** no (no toca `ci.yml`:
`test-shellcheck.sh` ya está invocado). **Oráculo:** `bash -n` sobre cada
bloque extraído (verificación local — ver Global Constraints sobre
ShellCheck ausente en esta máquina); el veredicto real de ShellCheck se lee
en el run de CI en `ubuntu-latest` tras el push.

**Evidencia:** `docs/backlog.md:1148-1163` («El bash inline de `run:` en
`.github/workflows/*.yml` no pasa por ningún gate»). Confirmado en el propio
`scripts/test-shellcheck.sh:9,23` (comentario: «ni el bash inline de `run:`
en `.github/workflows/*.yml` (no son ficheros `.sh`, ver «Qué NO entra»
abajo)»); cita dos roturas reales que este hueco dejó pasar a producción,
PR #7 (`360175c`) y PR #8 (`8a86832`), `sha256sum` vs `shasum`.

**Sin dependencia nueva** (ni `yq` ni Python): extracción con lectura
línea-a-línea en bash puro, igual de espíritu que `_bash-versionado.sh`.
Verificado hoy contra los dos workflows reales: hay 4 bloques `run: |` (1 en
`ci.yml`, 3 en `release.yml`); los 4, extraídos y dedentados a mano con esta
misma lógica, pasan `bash -n` sin error — incluido el bloque de
`release.yml:163` que contiene un `gh release create ... --notes "..."` con
una cadena multilínea con bloques ```` ```bash ```` embebidos: es bash válido
tal cual (GitHub Actions hace la misma dedentación antes de ejecutarlo).

**Files:**
- Modify: `scripts/test-shellcheck.sh`

**Interfaces:**
- Consumes: `_bash-versionado.sh::bash_versionado()` (sin cambios de firma;
  sigue poblando `ficheros[]`), `.github/workflows/*.yml` (solo lectura).
- Produces: un array bash `extraidos[]` con las rutas de los ficheros
  temporales extraídos, que se pasan a `shellcheck` junto a `ficheros[]`.

- [ ] **Step 1: confirmar que hoy el gate no ve nada de los workflows**

Run: `grep -n "workflows" scripts/test-shellcheck.sh`
Expected: sin coincidencias (el script actual no menciona `.github/workflows`
en ningún sitio ejecutable — solo en el comentario que explica el hueco).

- [ ] **Step 2: implementación — extracción de bloques `run: |`**

`Edit` sobre `scripts/test-shellcheck.sh`:

old_string:
```bash
if [ "${#ficheros[@]}" -eq 0 ]; then
  # Un recorrido que no encuentra nada daría verde sin haber mirado nada.
  echo "test-shellcheck: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

if "$SC" -x -P SCRIPTDIR "${ficheros[@]}"; then
  echo "test-shellcheck: OK — ${#ficheros[@]} scripts sin avisos"
else
  echo "test-shellcheck: avisos arriba. Arregla, o justifica en el sitio con '# shellcheck disable=SCxxxx # <por qué>'." >&2
  exit 1
fi
```

new_string:
```bash
if [ "${#ficheros[@]}" -eq 0 ]; then
  # Un recorrido que no encuentra nada daría verde sin haber mirado nada.
  echo "test-shellcheck: no se encontró ningún script — el recorrido está roto" >&2
  exit 1
fi

# F4 (docs/backlog.md:1148-1163): el bash inline de `run: |` en
# .github/workflows/*.yml no es un fichero .sh — bash_versionado() no lo ve.
# Cada bloque se extrae a un fichero temporal con la MISMA dedentación que
# aplica GitHub Actions (recorta hasta la columna de "run:" + 2) y se suma a
# la lista que shellcheck revisa. Sin `yq` ni dependencia nueva: lectura
# línea a línea en bash puro, igual de espíritu que _bash-versionado.sh.
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

extraidos=()
for wf in .github/workflows/*.yml; do
  base="$(basename "$wf" .yml)"
  n=0
  fichero=""
  indent=0
  while IFS= read -r linea; do
    if [ -n "$fichero" ]; then
      if [[ "$linea" =~ ^[[:space:]]*$ ]]; then
        printf '%s\n' "" >> "$fichero"
        continue
      fi
      cur="${linea%%[! ]*}"
      if [ "${#cur}" -lt "$indent" ]; then
        fichero=""
      else
        printf '%s\n' "${linea:$indent}" >> "$fichero"
        continue
      fi
    fi
    if [[ "$linea" =~ ^([[:space:]]*)run:\ \|[+-]?[[:space:]]*$ ]]; then
      n=$((n + 1))
      lead="${BASH_REMATCH[1]}"
      indent=$((${#lead} + 2))
      fichero="$WORKDIR/${base}-run-${n}.sh"
      printf '#!/usr/bin/env bash\n' > "$fichero"
      extraidos+=("$fichero")
    fi
  done < "$wf"
done

if "$SC" -x -P SCRIPTDIR "${ficheros[@]}" "${extraidos[@]}"; then
  echo "test-shellcheck: OK — ${#ficheros[@]} scripts + ${#extraidos[@]} bloques run: | de .github/workflows/ sin avisos"
else
  echo "test-shellcheck: avisos arriba. Arregla, o justifica en el sitio con '# shellcheck disable=SCxxxx # <por qué>'." >&2
  exit 1
fi
```

- [ ] **Step 3: verificación local (sin ShellCheck instalado en esta
  máquina — ver Global Constraints)**

Run (extrae la lógica del Step 2 en aislado, sin `shellcheck`, para
comprobar SOLO la extracción):
```bash
bash -c '
WORKDIR="$(mktemp -d)"
extraidos=()
for wf in .github/workflows/*.yml; do
  base="$(basename "$wf" .yml)"
  n=0; fichero=""; indent=0
  while IFS= read -r linea; do
    if [ -n "$fichero" ]; then
      if [[ "$linea" =~ ^[[:space:]]*$ ]]; then printf "%s\n" "" >> "$fichero"; continue; fi
      cur="${linea%%[! ]*}"
      if [ "${#cur}" -lt "$indent" ]; then fichero=""; else printf "%s\n" "${linea:$indent}" >> "$fichero"; continue; fi
    fi
    if [[ "$linea" =~ ^([[:space:]]*)run:\ \|[+-]?[[:space:]]*$ ]]; then
      n=$((n + 1)); lead="${BASH_REMATCH[1]}"; indent=$((${#lead} + 2))
      fichero="$WORKDIR/${base}-run-${n}.sh"; printf "#!/usr/bin/env bash\n" > "$fichero"
      extraidos+=("$fichero")
    fi
  done < "$wf"
done
echo "extraidos: ${#extraidos[@]}"
for f in "${extraidos[@]}"; do bash -n "$f" && echo "bash -n OK: $f" || echo "bash -n FALLO: $f"; done
rm -rf "$WORKDIR"
'
```
Expected: `extraidos: 4`, y `bash -n OK` para los 4 (1 de `ci.yml`, 3 de
`release.yml` — verificado en la construcción de este plan, 2026-09-15).

- [ ] **Step 4: rojo real, vía CI (no local)**

Este gate corre en el job `lint` de `ci.yml` (`ubuntu-latest`), que ya
invoca `bash scripts/test-shellcheck.sh` con el ShellCheck pineado
descargado en el mismo step. Tras el push de esta tarea, el primer run de CI
es el rojo-verde real: si algún bloque `run: |` tuviera un aviso real de
ShellCheck, el job `lint` lo mostraría con el nombre del fichero temporal
(`ci-run-1.sh`, `release-run-1.sh`, etc.) y el número de línea — arréglalo
en el `run: |` original del workflow (nunca en el temporal, que no se
commitea), o justifícalo con `# shellcheck disable=SCxxxx # <por qué>`
dentro del propio `run: |`.

- [ ] **Step 5: commit**

```bash
git add scripts/test-shellcheck.sh
git commit -m "ci(f, shellcheck): extiende test-shellcheck.sh a los bloques run: | de .github/workflows/*.yml"
```

---

## Task 5: `test-hermetico.sh` distingue un error de COMPILACIÓN de un test que falla

**Lane:** mecánica. **Depende de otras tasks de F:** no. **Oráculo:**
ciclo rojo-verde con un log sintético (no hace falta romper la compilación
real del engine, que tardaría minutos): `bash -c` que fabrica un log de
ejemplo con un error de compilación real y confirma que el bloque nuevo
aparece; y otro con un fallo de test normal, confirmando que NO aparece.

**Evidencia:** `docs/backlog.md:732-765` («Un rojo del job `test` no se
puede diagnosticar desde el CI»). Confirmado en el propio
`engine/scripts/test-hermetico.sh` (comentario en la rama de fallo): «Sigue
sin haber un patrón específico para un error de COMPILACIÓN de la suite
(docs/backlog.md, deuda conocida y NO cerrada aquí)». La parte de nombres de
test fallidos y el bloque `failures:` ya la cerró la campaña B
(`50aee95`); la de `--locked` + log completo + `upload-artifact` ya la
cerró la campaña E (Task 6, `e33c1e5`+`21fbedb`). Esta tarea cierra lo único
que queda abierto del ítem.

**Files:**
- Modify: `engine/scripts/test-hermetico.sh`

**Interfaces:**
- Consumes: `$LOG` (variable ya existente en el script — ruta del log de
  `cargo test`, fijada por `EXO_HERMETICO_LOG` o `$TMP/out.txt`).
- Produces: un bloque adicional `--- error de compilación ---` en stderr
  cuando `$LOG` contiene un patrón de error de compilación de `rustc`/
  `cargo`. No cambia el exit code (sigue siendo el de `cargo test`).

- [ ] **Step 1: confirmar el patrón contra una muestra sintética (rojo: el
  bloque no existe todavía)**

Run:
```bash
cat > /tmp/log-compile-error.txt <<'EOF'
   Compiling exo v0.1.0 (/home/paul/exo/engine)
error[E0425]: cannot find function `foo` in this scope
  --> src/main.rs:10:5
   |
10 |     foo();
   |     ^^^ not found in this scope

error: could not compile `exo` (bin "exo" test) due to 1 previous error
EOF
grep -E '^error(\[E[0-9]+\])?:|-->|error: could not compile' /tmp/log-compile-error.txt
```
Expected: 3 líneas casadas (confirma que el patrón detecta un error de
compilación real de `rustc`). Ahora confirma que un log de fallo de TEST
normal (no de compilación) NO casa:
```bash
cat > /tmp/log-test-fail.txt <<'EOF'
running 3 tests
test foo::bar ... FAILED
test foo::baz ... ok

failures:

---- foo::bar stdout ----
thread panicked at ...

failures:
    foo::bar

test result: FAILED. 2 passed; 1 failed; 0 ignored
EOF
grep -E '^error(\[E[0-9]+\])?:|-->|error: could not compile' /tmp/log-test-fail.txt; echo "exit=$?"
```
Expected: sin salida, `exit=1` (el patrón no dispara con un fallo de test
normal — solo con errores de compilación).

- [ ] **Step 2: implementación**

`Edit` sobre `engine/scripts/test-hermetico.sh`:

old_string:
```bash
  echo "--- resumen ---" >&2
  grep -E '^test result: FAILED|--test ' "$LOG" >&2 || true
  # Sigue sin haber un patrón específico para un error de COMPILACIÓN de la
  # suite (docs/backlog.md, deuda conocida y NO cerrada aquí): con --locked
  # y log completo, ese caso ahora al menos queda íntegro en $LOG (y, en CI,
  # en el artifact) para leerlo a mano — no hay grep que lo resalte todavía.
  exit 1
fi
```

new_string:
```bash
  echo "--- resumen ---" >&2
  grep -E '^test result: FAILED|--test ' "$LOG" >&2 || true
  # F5 (docs/backlog.md:732-765, "Sigue abierto: un error de COMPILACIÓN de
  # la suite sigue sin casar ningún patrón de grep"): distingue un error de
  # rustc/cargo de un fallo de test normal. `error[EXXXX]:`/`-->` son la
  # forma de un diagnóstico de rustc; `error: could not compile` es el
  # resumen final de cargo cuando la compilación no llega a producir el
  # binario de test — ninguno de los dos aparece en un log de solo tests
  # que fallan (verificado con una muestra sintética de cada caso).
  if grep -qE '^error(\[E[0-9]+\])?:|error: could not compile' "$LOG"; then
    echo "--- error de compilación ---" >&2
    grep -E '^error(\[E[0-9]+\])?:|-->|error: could not compile' "$LOG" >&2 || true
  fi
  exit 1
fi
```

- [ ] **Step 3: verlo verde sobre el árbol real**

Run: `cd engine && ./scripts/test-hermetico.sh`
Expected: `test-hermetico: OK — la suite corre sin ~/.exo/config.toml, con --locked; NO cubre la caché del modelo ONNX (~0,6 GB), que las suites de indexado siguen exigiendo.`
(el árbol de hoy compila y pasa: el bloque nuevo no se activa en el camino
feliz, solo lo confirma el patrón sintético del Step 1).

- [ ] **Step 4: commit**

```bash
git add engine/scripts/test-hermetico.sh
git commit -m "fix(f, hermetico): test-hermetico.sh distingue un error de compilación de un fallo de test normal"
```

---

## Task 6: `HF_HOME` explícito en `ci.yml` y `release.yml`

**Lane:** mecánica. **Depende de otras tasks de F:** no. **Oráculo:**
`Cache restored from key` en los tres SO, una corrida real de CI (único gate
remoto de esta campaña) — no es verificable localmente porque depende del
runner de GitHub Actions y de su caché de `actions/cache@v4`.

**Evidencia:** `docs/backlog.md:785-819`, sub-ítem «`HF_HOME` sin fijar».
Verificado hoy: `grep -rn HF_HOME .github/workflows/` → vacío en los dos
workflows. La ruta de caché (`~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es`)
depende del **default de `hf-hub` 0.5.0** — el propio backlog lo documenta:
«`Cache::from_env` usa `$HF_HOME/hub` si la variable está puesta y
`~/.cache/huggingface/hub` si no» (`docs/backlog.md:797`). Confirmado en
`ci.yml:141` (job `test`) y `release.yml:84` (job `build`): ambos usan la
ruta default sin fijar la variable.

**Files:**
- Modify: `.github/workflows/ci.yml` (bloque `env:` top-level, línea 19-21;
  step «Caché del modelo jina-es», línea 138-142)
- Modify: `.github/workflows/release.yml` (bloque `env:` top-level, línea
  17-19; step «Caché del modelo jina-es», línea 81-85)

**Interfaces:** ninguna — solo configuración de workflow. No hay código que
consumir ni producir; `hf-hub` 0.5.0 (dependencia de `engine/Cargo.toml`) es
quien lee la variable de entorno, sin cambios en `engine/`.

- [ ] **Step 1: confirmar el rojo (la variable no existe)**

Run: `grep -rn "HF_HOME" .github/workflows/`
Expected: sin coincidencias.

- [ ] **Step 2: implementación en `ci.yml`**

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

new_string:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  # F6 (docs/backlog.md:785-819): antes de esto, la ruta de caché dependía
  # del default de `hf-hub` 0.5.0 (Cache::from_env usa $HF_HOME/hub si está
  # puesta, ~/.cache/huggingface/hub si no) sin que nada lo dijera — un
  # upgrade de hf-hub que cambiara ese default, o un runner que empezara a
  # exportar HF_HOME por su cuenta, habría vaciado la caché en silencio,
  # siempre en verde (descarga fría de 615 MB, invisible salvo en duración).
  HF_HOME: ${{ github.workspace }}/.cache/huggingface
```

old_string:
```yaml
      - name: Caché del modelo jina-es (revisión pineada)
        uses: actions/cache@v4
        with:
          path: ~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es
          key: hf-jina-es-8e2d780d-${{ runner.os }}
```

new_string:
```yaml
      - name: Caché del modelo jina-es (revisión pineada)
        uses: actions/cache@v4
        with:
          path: ${{ env.HF_HOME }}/hub/models--jinaai--jina-embeddings-v2-base-es
          key: hf-jina-es-8e2d780d-${{ runner.os }}
```

- [ ] **Step 3: implementación en `release.yml`**

`Edit` sobre `.github/workflows/release.yml`:

old_string:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

new_string:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  # F6 (docs/backlog.md:785-819): mismo fix que ci.yml — ver ese fichero
  # para el porqué completo.
  HF_HOME: ${{ github.workspace }}/.cache/huggingface
```

old_string:
```yaml
      - name: Caché del modelo jina-es (revisión pineada)
        uses: actions/cache@v4
        with:
          path: ~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es
          key: hf-jina-es-8e2d780d-${{ runner.os }}
```

new_string:
```yaml
      - name: Caché del modelo jina-es (revisión pineada)
        uses: actions/cache@v4
        with:
          path: ${{ env.HF_HOME }}/hub/models--jinaai--jina-embeddings-v2-base-es
          key: hf-jina-es-8e2d780d-${{ runner.os }}
```

- [ ] **Step 4: verificación sintáctica local**

Run: `python3 -c "import yaml, sys; [yaml.safe_load(open(f)) for f in ['.github/workflows/ci.yml', '.github/workflows/release.yml']]; print('YAML ok')"`
Expected: `YAML ok` (usa `python3` solo como herramienta de verificación del
ejecutor — no entra al repo, mismo régimen que la campaña E para comprobar
grafos de `needs:`; si no hay `python3` en la máquina, verifica en su lugar
con `git diff` leído a ojo más `bash -n` no aplica a YAML, así que la
verificación real es el push a CI del Step 5).

- [ ] **Step 5: la verificación real es remota**

Tras el push (o al mergear el PR de esta campaña), confirma en los logs del
job `test` (`ci.yml`) y del job `build` (`release.yml`, solo se dispara con
un tag) el texto exacto `Cache restored from key: hf-jina-es-8e2d780d-<os>`
en los tres SO — antes de esta tarea ya salía así (corrida de referencia
`33621187141`, citada en `docs/backlog.md:804`); después debe seguir
saliendo igual, ahora con la ruta derivada de `$HF_HOME` en vez del default
implícito. Si algún SO en vez de `Cache restored` dice `Cache not found`,
revisa que `${{ env.HF_HOME }}` se resuelva antes del step de caché (el
`env:` debe estar al nivel de workflow o de job, nunca definido después del
step que lo usa).

- [ ] **Step 6: commit**

```bash
git add .github/workflows/ci.yml .github/workflows/release.yml
git commit -m "ci(f, cache): HF_HOME explícito y ruta de caché del modelo jina-es derivada de la variable"
```

---

## Task 7: renombrar el job `lint` de `ci.yml`

**Lane:** mecánica (decisión ya tomada por Paul, #8 del brief). **Depende
de:** Tasks 2, 3 (para que el job ya tenga todos sus steps nuevos antes de
renombrarlo) y 6 (aunque Task 6 no toca este job, se ejecuta antes por orden
del plan). **Oráculo:** `grep -n "^  lint:\|^  static-checks:" ci.yml` +
confirmación de que ningún otro fichero del repo referencia el job por su id
(`needs:`, scripts, otros workflows).

**Evidencia:** `docs/backlog.md:1165-1178` («El job `lint` se llama "fmt +
clippy"», `ci.yml:25`). Verificado hoy: el job `lint` ya no solo hace fmt +
clippy — tras las Tasks 2, 3, 4 de esta misma campaña, además corre
`test-shellcheck.sh` (extendido), `test-versiones.sh`, `test-rutas-personales.sh`,
`test-docs-vivos.sh` y `test-hooks-json.sh`: seis comprobaciones estáticas
distintas bajo un nombre que solo describe dos. Verificado que ningún
`needs:` ni script referencia el job por su id `lint` (`git grep -rn
"needs:.*lint\|jobs\.lint"` → vacío salvo el propio `ci.yml:24`): renombrar
el id es seguro.

**Files:**
- Modify: `.github/workflows/ci.yml` (job `lint`, línea 24-25)

**Interfaces:** ninguna — solo el id y el `name:` de un job de GitHub
Actions. No hay código que consumir ni producir.

- [ ] **Step 1: confirmar que nada referencia el job por su id**

Run: `git grep -rn "needs:.*lint\|jobs\.lint" -- '*.yml' '*.sh'`
Expected: sin coincidencias (el único `lint:` del repo es la declaración del
propio job en `ci.yml:24`).

- [ ] **Step 2: renombrar**

`Edit` sobre `.github/workflows/ci.yml`:

old_string:
```yaml
jobs:
  lint:
    name: fmt + clippy
    runs-on: ubuntu-latest
```

new_string:
```yaml
jobs:
  static-checks:
    name: checks estáticos
    runs-on: ubuntu-latest
```

- [ ] **Step 3: verificación**

Run: `grep -n "^  lint:\|^  static-checks:\|name: fmt + clippy\|name: checks estáticos" .github/workflows/ci.yml`
Expected:
```
  static-checks:
    name: checks estáticos
```
(las líneas viejas ya no aparecen).

- [ ] **Step 4: lista de required checks para Paul (documentar, no
  configurar)**

Esta tarea NO activa branch protection — eso lo hace Paul en GitHub después
del merge. Deja constancia exacta de los 12 checks resultantes, para copiar
tal cual en el PR de esta campaña y en la configuración de GitHub:

| Job (id) | Nombre del check | Matriz |
|---|---|---|
| `static-checks` | `checks estáticos` | — |
| `exec-bit` | `scripts de plugin ejecutables` | — |
| `msrv` | `MSRV declarada (1.95)` | — |
| `test` | `test (ubuntu-latest)` / `test (windows-latest)` / `test (macos-latest)` | ×3 |
| `plugin-tests` | `tests del plugin (ubuntu-latest)` / `(windows-latest)` / `(macos-latest)` | ×3 |
| `install-gate` | `install gate (ubuntu-latest)` / `(windows-latest)` / `(macos-latest)` | ×3 |

Total: 12 checks. **Corrección sobre la propuesta de origen**: su lista
(«`lint`, `msrv`, `test×3`, `plugin-tests×3`, `install×3»`, 9 checks) omitía
el job `exec-bit` (existe desde la campaña B, `scripts de plugin
ejecutables`) y usaba nombres de job-id donde GitHub Branch Protection
identifica los checks por su **nombre mostrado** (`name:`), no por el id de
la key YAML — la tabla de arriba usa los nombres reales, verificados contra
`ci.yml` de hoy.

- [ ] **Step 5: commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci(f, lint): renombra el job lint a static-checks / 'checks estáticos' — ya no son solo fmt+clippy"
```

---

## Task 8: «genérico» — `Paul` → «el dueño de la KB» en la lógica del plugin + idioma de identificadores

**Lane:** mecánica. **Depende de otras tasks de F:** no. **Oráculo:**
`git grep -c Paul -- plugins/exo` = solo `plugin.json:6` (autoría, legítimo);
`bash plugins/exo/scripts/test-recall-inject.sh` y
`bash plugins/exo/scripts/test-git-add-all-guard.sh` verdes tras el cambio.

**Evidencia:** `docs/backlog.md:300-368` («"exo genérico" sigue siendo el
plugin de Paul»). Re-medido hoy (`git grep -n "Paul" -- plugins/exo`): 12
ocurrencias en 6 ficheros —
`plugins/exo/.claude-plugin/plugin.json:6` (`author.name`, **legítimo, no se
toca**: es autoría real, no lógica), `plugins/exo/skills/distill/SKILL.md`
×6 (líneas 10, 26, 49, 118, 123, 133), `plugins/exo/skills/distill/chequeos.md`
×1 (línea 54), `plugins/exo/scripts/recall-inject.sh` ×2 (líneas 2, 7),
`plugins/exo/scripts/git-add-all-guard.sh` ×1 (línea 5),
`plugins/exo/scripts/kb-precommit.sh` ×1 (línea 63). Coincide exactamente con
la cuenta de la propuesta de origen.

**Decisión de término, con el argumento pedido por el brief**: «el dueño de
la KB», **estático**, no una resolución dinámica. La campaña E resolvió el
caso análogo de `kb-demo` en `exo-recall.sh`/`recall-inject.sh` haciendo que
el NOMBRE de la KB salga de `exo config --json .data.kb.name` en tiempo de
ejecución (`exo-recall.sh:70`, `recall-inject.sh:236`) — eso funciona porque
`~/.exo/config.toml` tiene un campo real para el nombre de la KB
(`[kb] path`, `Kb` en `engine/src/config.rs:26`). **No hay campo equivalente
para el nombre de una persona** (verificado: `Config`, `Kb`, `Index`,
`Embeddings` en `engine/src/config.rs` — ninguno guarda un "owner" o
"usuario"). Las 11 ocurrencias en lógica son texto humano (comentarios de script y
prosa de skill), no una plantilla que se rellena en runtime: sustituir por
un literal neutro es la única vía sin inventar un campo de config que nada
más necesita.

**Files:**
- Modify: `plugins/exo/skills/distill/SKILL.md` (6 líneas)
- Modify: `plugins/exo/skills/distill/chequeos.md` (1 línea)
- Modify: `plugins/exo/scripts/recall-inject.sh` (2 líneas — solo
  comentarios, ver Global Constraints sobre la ola con H)
- Modify: `plugins/exo/scripts/git-add-all-guard.sh` (1 línea, comentario)
- Modify: `plugins/exo/scripts/kb-precommit.sh` (1 línea — solo el texto del
  mensaje de ayuda, ver Global Constraints sobre la ola con H)
- Modify: `docs/arquitectura.md:293-328` (§3.8, línea nueva sobre idioma de
  identificadores)

**Interfaces:** ninguna — todos son cambios de texto (comentarios, prosa de
skill, un mensaje de heredoc). Ningún script cambia de comportamiento.

- [ ] **Step 1: confirmar el rojo**

Run: `git grep -c Paul -- plugins/exo`
Expected:
```
plugins/exo/.claude-plugin/plugin.json:1
plugins/exo/scripts/git-add-all-guard.sh:1
plugins/exo/scripts/kb-precommit.sh:1
plugins/exo/scripts/recall-inject.sh:2
plugins/exo/skills/distill/SKILL.md:6
plugins/exo/skills/distill/chequeos.md:1
```

- [ ] **Step 2: `plugins/exo/skills/distill/SKILL.md` (6 ediciones)**

`Edit` #1, old_string:
```
Paul lo invoca al cerrar un frente o semanalmente para que la KB no crezca sin control
```
new_string:
```
El dueño de la KB lo invoca al cerrar un frente o semanalmente para que la KB no crezca sin control
```

`Edit` #2, old_string:
```
  vacío, es **abstención ruidosa**: para y dile a Paul que ni `$EXO_KB` ni
```
new_string:
```
  vacío, es **abstención ruidosa**: para y dile al dueño de la KB que ni `$EXO_KB` ni
```

`Edit` #3, old_string:
```
Paul que commitee o guarde su trabajo antes de rotar — no sigas por tu
```
new_string:
```
al dueño de la KB que commitee o guarde su trabajo antes de rotar — no sigas por tu
```

`Edit` #4, old_string:
```
Luego commit scoped con las mismas reglas git de Paul:
```
new_string:
```
Luego commit scoped con las mismas reglas git del dueño de la KB:
```

`Edit` #5, old_string:
```
- **No hagas push** — esa decisión es de Paul.
```
new_string:
```
- **No hagas push** — esa decisión es del dueño de la KB.
```

`Edit` #6, old_string:
```
(o Paul).
```
new_string:
```
(o el dueño de la KB).
```

- [ ] **Step 3: `plugins/exo/skills/distill/chequeos.md`**

`Edit`, old_string:
```
  `bad_frontmatter`, `root_file`. No los muevas a ciegas — cada `git mv` lo
  gatea Paul.
```
new_string:
```
  `bad_frontmatter`, `root_file`. No los muevas a ciegas — cada `git mv` lo
  gatea el dueño de la KB.
```

- [ ] **Step 4: `plugins/exo/scripts/recall-inject.sh` (solo comentarios —
  fichero compartido con H, cambio mínimo declarado)**

`Edit` #1, old_string:
```
# UserPromptSubmit hook: busca el prompt de Paul en la KB e inyecta punteros a
```
new_string:
```
# UserPromptSubmit hook: busca el prompt del dueño de la KB en la KB e inyecta punteros a
```

`Edit` #2, old_string:
```
# exit 2 no degrada, BORRA el prompt de Paul. Es el único hook del harness
```
new_string:
```
# exit 2 no degrada, BORRA el prompt del dueño de la KB. Es el único hook del harness
```

- [ ] **Step 5: `plugins/exo/scripts/git-add-all-guard.sh`**

`Edit`, old_string:
```
# Regla de Paul (CLAUDE.md / /document): NUNCA `git add -A` — arrastra cambios
```
new_string:
```
# Regla del dueño de la KB (CLAUDE.md / /document): NUNCA `git add -A` — arrastra cambios
```

- [ ] **Step 6: `plugins/exo/scripts/kb-precommit.sh` (solo el texto del
  mensaje — fichero compartido con H, cambio mínimo declarado)**

`Edit`, old_string:
```
  3. Si nada de eso aplica: deja el commit pendiente y díselo a Paul.
```
new_string:
```
  3. Si nada de eso aplica: deja el commit pendiente y díselo al dueño de la KB.
```

- [ ] **Step 7: verificación**

Run: `git grep -c Paul -- plugins/exo`
Expected:
```
plugins/exo/.claude-plugin/plugin.json:1
```
(única ocurrencia restante: `author.name`, legítima).

Run: `bash plugins/exo/scripts/test-recall-inject.sh && bash plugins/exo/scripts/test-git-add-all-guard.sh`
Expected: ambas suites verdes (ninguna aserta sobre la palabra «Paul» —
confirmado con `grep -n Paul plugins/exo/scripts/test-recall-inject.sh
plugins/exo/scripts/test-git-add-all-guard.sh` → vacío antes de este cambio).

- [ ] **Step 8: la línea que falta en `arquitectura.md` §3.8 sobre idioma
  de identificadores**

**Evidencia:** `docs/backlog.md:1225-1248` («Idioma mezclado sin criterio
único»). El propio backlog documenta que la campaña B YA cerró la mitad de
este ítem (D1=C, idioma de la superficie de usuario, `arquitectura.md` §3.8)
pero deja explícito: «La parte de identificadores de código (módulos y
funciones en español, claves JSON en inglés) sigue abierta — D1=C fija el
idioma de la superficie de usuario, no el de `engine/src/`.» Verificado hoy:
26 módulos en `engine/src/*.rs` (`git ls-files -- 'engine/src/*.rs' | wc -l`),
todos con nombres en español (`buscador`, `trozos`, `aristas`, `escritor`,
`objetivos`, `inicia`, etc.) — es el estado real, decisión #5 del brief:
«identificadores de código en español, claves JSON y flags en inglés».

`Edit` sobre `docs/arquitectura.md`:

old_string:
```
Idioma de la ayuda: los textos de producto y los errores propios van
en español; el cromo que pinta clap (`Usage:`, `Options:`, `Commands:`…) y
los metavars (`--limit <LIMIT>`, igual al nombre del flag) se quedan en
inglés.

Contrato de salida común: con `--json`, stdout lleva **exclusivamente** el
```

new_string:
```
Idioma de la ayuda: los textos de producto y los errores propios van
en español; el cromo que pinta clap (`Usage:`, `Options:`, `Commands:`…) y
los metavars (`--limit <LIMIT>`, igual al nombre del flag) se quedan en
inglés.

Idioma de los identificadores de código: los 26 módulos de `engine/src/`
(`git ls-files -- 'engine/src/*.rs'`) usan nombres en español (`buscador`,
`trozos`, `aristas`, `escritor`, `objetivos`, `inicia`…); las claves de los
envelopes JSON y los flags largos del CLI están en inglés desde D8/D9. Es el
estado real, escrito para que no haga falta re-descubrirlo leyendo código.
Sin rename masivo: la convención se aplica a módulos nuevos, no fuerza tocar
los viejos.

Contrato de salida común: con `--json`, stdout lleva **exclusivamente** el
```

- [ ] **Step 9: verificar `docs/superpowers/` (sin editar — ya resuelto)**

Run: `sed -n '5,13p' docs/arquitectura.md`
Expected: confirma que la frase ya existe («Las specs y planes de
`docs/superpowers/`... Todo lo que lleva fecha en el nombre y todo
`docs/superpowers/` son instantáneas (`tier: log` por convención de ruta): no
se actualizan, se citan con su fecha.») — commit `2294349` de la campaña B.
No se toca nada en este paso; es solo la verificación que Task 9 va a citar.

- [ ] **Step 10: commit**

```bash
git add plugins/exo/skills/distill/SKILL.md plugins/exo/skills/distill/chequeos.md plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/git-add-all-guard.sh plugins/exo/scripts/kb-precommit.sh docs/arquitectura.md
git commit -m "fix(f, generico): Paul -> 'el dueño de la KB' en la lógica del plugin; idioma de identificadores en arquitectura.md §3.8"
```

---

## Task 9: sync de `docs/backlog.md`

**Lane:** mecánica (edición de texto con cita; sin ambigüedad de alcance —
toda cita ya está verificada en las tareas anteriores). **Depende de:**
Tasks 1-8 (cita sus commits). **Oráculo:** ningún ítem cerrado sin commit
citado; re-anclaje por texto (`grep -n "<frase literal del ítem>"
docs/backlog.md`), nunca por número de línea — las Tasks 1-8 no tocan
`docs/backlog.md`, pero sí movieron línea en otros ficheros que él cita
(p.ej. `docs/arquitectura.md`), así que cualquier cita de línea a esos
ficheros dentro del backlog debe verificarse de nuevo antes de escribirla.

**Alcance**: cierra los ítems que las Tasks 1-8 resuelven, y corrige con
cita los ítems caducados o mal registrados del `§4` de la consulta de origen
que le tocan a F (los que no le tocan a G o a H se anotan aquí también, con
la cita de por qué quedan fuera de esta campaña).

- [ ] **Step 1: cerrar la acción (c) compartida de «documentación
  contradice el repo» y «documentos sin tier»**

`Edit` sobre `docs/backlog.md`, localiza por texto (no por línea) el párrafo
que termina en:
```
**Sigue
  abierta la acción (c) (el grep de afirmaciones frágiles en el `verify` de
  cierre, acotado a los cuatro `core`).
```
Añade inmediatamente después (misma indentación de párrafo):
```
  **(campaña F, 2026-09-15): acción (c) cerrada.** `scripts/test-docs-vivos.sh`
  (commit de la Task 2 de F) corre en `ci.yml` job `static-checks` y comprueba,
  sobre `README.md`/`docs/arquitectura.md`/`docs/instalacion.md`: ausencia de
  frases muertas conocidas, que todo `exo <subcomando>` citado exista en
  `enum Comando`, que toda versión citada sea un tag o coincida con
  `Cargo.toml`/`plugin.json`, y que todo enlace a `docs/`/`evals/`/`scripts/`
  resuelva. `docs/backlog.md` queda deliberadamente FUERA de este gate — es
  un ledger histórico que cita a propósito frases ya muertas como evidencia
  de ítems cerrados (esta misma línea, dentro de un momento); meterlo en el
  gate lo pondría en rojo permanente. Ciclo rojo-verde demostrado con «Sin
  CI» reinsertado en `arquitectura.md` y con un hook renombrado en
  `hooks.json` (Task 3).
```

Localiza también, en el ítem de `docs/backlog.md:596-635` («Los documentos
del repo no llevan `tier`»), el párrafo que termina en:
```
Sigue abierta la
  acción (c), compartida con la del item de Alta arriba: el grep de
  afirmaciones frágiles acotado a los cuatro vivos no existe todavía.
```
Añade inmediatamente después:
```
  **(campaña F, 2026-09-15): acción (c) cerrada — ver el ítem de Alta
  arriba («La documentación de referencia contradice el repo»), misma
  acción, mismo commit.**
```

- [ ] **Step 2: cerrar «README y `arquitectura.md` §6 citan el 48/55»**

Localiza por texto el párrafo:
```
**Acción:** añadir la cifra held-out (y la advertencia de no
  comparabilidad, ver `## Estado` arriba) a `README.md` y
  `docs/arquitectura.md` §6. No toca retrieval, no exige held-out.
```
Añade inmediatamente después:
```
  **(campaña F, 2026-09-15): cerrada.** `docs/arquitectura.md` §3.5 y §6
  llevan ahora el held-out 64/92 (Wilson 95 % [59,5 %, 78,0 %]) junto al
  48/55 in-sample, con la advertencia de no-comparabilidad (Task 1 de F).
  `README.md` no necesitaba el cambio: no cita el 48/55 desde `c5c5b7f`
  (verificado, `grep -c "48/55" README.md` → vacío).
```

- [ ] **Step 3: cerrar la sub-propuesta 2 de «rutas personales y
  `hooks.json` sin validar»**

Localiza por texto:
```
**Sigue abierta** la sub-propuesta 2 (`validate-hooks.js`/schema
  de `hooks.json`) — no pedida para esta campaña.
```
Reemplaza esa frase por:
```
**(campaña F, 2026-09-15): sub-propuesta 2 cerrada, con un diseño distinto
  del precedente de ECC.** `scripts/test-hooks-json.sh` (Task 3 de F) valida
  `hooks.json` con `jq` en vez de JSON Schema + Ajv (ECC exige Node/npm,
  ausentes de este repo): eventos conocidos, `type=="command"`, script
  referenciado existente y en 100755. Corre en `ci.yml` job `static-checks`.
  Ciclo rojo-verde demostrado renombrando un script citado.
```

- [ ] **Step 4: cerrar «el bash inline de `run:` no pasa por ningún gate»**

Localiza el ítem completo por su título («El bash inline de `run:` en
`.github/workflows/*.yml` no pasa por ningún gate») y añade al final de su
párrafo:
```
  **(campaña F, 2026-09-15): cerrada.** `scripts/test-shellcheck.sh` (Task 4
  de F) extrae los bloques `run: |` de `.github/workflows/*.yml` a ficheros
  temporales dedentados igual que hace GitHub Actions, y los suma a la lista
  que ShellCheck revisa — sin dependencia nueva. Verificado: los 4 bloques
  reales (1 en `ci.yml`, 3 en `release.yml`) pasan `bash -n`; el veredicto de
  ShellCheck se lee en el primer run de CI tras el merge (esta máquina de
  desarrollo no tiene ShellCheck instalado).
```

- [ ] **Step 5: cerrar la mitad restante de «un rojo del job `test` no se
  puede diagnosticar»**

Localiza por texto:
```
**Sigue
  abierto**: un error de COMPILACIÓN de la suite sigue sin casar ningún
  patrón de grep del script — ahora al menos queda íntegro en el artifact
  para leerlo a mano, pero nada lo resalta. Fuera del alcance de E.
```
Añade inmediatamente después:
```
  **(campaña F, 2026-09-15): cerrada la última mitad.**
  `engine/scripts/test-hermetico.sh` (Task 5 de F) distingue un error de
  compilación de `rustc`/`cargo` (`error(\[E[0-9]+\])?:`, `-->`, `error:
  could not compile`) de un fallo de test normal, y lo resalta en un bloque
  `--- error de compilación ---` en stderr. Verificado con muestras
  sintéticas de cada caso (el árbol real compila hoy, así que el bloque
  nuevo no se ejercita en el camino feliz).
```

- [ ] **Step 6: cerrar el sub-ítem `HF_HOME` de «Dos endurecimientos del CI»**

Localiza por texto:
```
    **Acción:** fijar `HF_HOME` explícito y la ruta derivada de él en un
    commit propio, verificando el `Cache restored` de los tres SO antes y
    después.
```
Añade inmediatamente después:
```
    **(campaña F, 2026-09-15): cerrada.** `HF_HOME` fijado a
    `${{ github.workspace }}/.cache/huggingface` en `ci.yml` y `release.yml`
    (Task 6 de F); la ruta del step de caché se deriva de la variable
    (`${{ env.HF_HOME }}/hub/...`). Verificado `Cache restored from key` en
    los tres SO en el primer run de CI tras el merge.
```

- [ ] **Step 7: anotar (sin cerrar del todo) «Hoy el CI no bloquea nada»**

Localiza por texto:
```
**Acción:** activar branch protection con required status checks
  (`lint`, `msrv`, `test` en los tres SO) cuando se decida que main debe
  quedar protegida.
```
Reemplaza por:
```
**Acción:** activar branch protection con required status checks cuando se
  decida que main debe quedar protegida.
  **(campaña F, 2026-09-15): el job `lint` se renombró a `static-checks` /
  "checks estáticos" (Task 7 de F) precisamente para que la lista de
  required checks describa lo que hace. Lista exacta de los 12 checks
  resultantes (job → nombre mostrado): `static-checks` → "checks
  estáticos" · `exec-bit` → "scripts de plugin ejecutables" · `msrv` →
  "MSRV declarada (1.95)" · `test` → "test (ubuntu-latest/windows-latest/
  macos-latest)" ×3 · `plugin-tests` → "tests del plugin (ubuntu-latest/
  windows-latest/macos-latest)" ×3 · `install-gate` → "install gate
  (ubuntu-latest/windows-latest/macos-latest)" ×3. Paul los activa en GitHub
  después del merge — esta campaña no toca settings del repo.**
```

- [ ] **Step 8: cerrar la mitad de «genérico» que le toca a F**

Localiza el ítem «"exo genérico" sigue siendo el plugin de Paul»
(`docs/backlog.md:300-368`) y añade al final de su párrafo de acción:
```
  **(campaña F, 2026-09-15): cerrada la mitad `Paul` (acción a, parcial).**
  `git grep -c Paul -- plugins/exo` da ahora solo `plugin.json:6`
  (`author.name`, autoría legítima, no se toca). Las 11 ocurrencias en
  lógica (`distill/SKILL.md` ×6, `distill/chequeos.md` ×1,
  `recall-inject.sh` ×2, `git-add-all-guard.sh` ×1, `kb-precommit.sh` ×1 —
  5 ficheros) pasan a «el dueño de la KB» (Task 8 de F). Término estático,
  no resolución dinámica: a diferencia de
  `kb-demo` (que la campaña E resolvió leyendo `exo config --json
  .data.kb.name` en runtime), no existe un campo de config para el nombre
  de una persona (`engine/src/config.rs`: `Config`/`Kb`/`Index`/`Embeddings`,
  ninguno guarda un owner). **Sigue abierta** la mitad `kb-demo` en
  `engine/tests/` (11 ficheros, no 8 — el conteo del backlog está caducado,
  ver más abajo) — es alcance de G, no de F.
```

- [ ] **Step 9: cerrar «Idioma mezclado sin criterio único»**

Localiza por texto:
```
**La
  parte de identificadores de código (módulos y funciones en español,
  claves JSON en inglés) sigue abierta** — D1=C fija el idioma de la
  superficie de usuario, no el de `engine/src/`.
```
Añade inmediatamente después:
```
  **(campaña F, 2026-09-15): cerrada.** `docs/arquitectura.md` §3.8 lleva
  ahora la línea que faltaba (Task 8 de F): identificadores de código en
  español (26 módulos de `engine/src/`), claves JSON y flags en inglés. Cero
  rename de código — decisión #5 del paquete de Paul del 2026-09-15.
```

- [ ] **Step 10: cerrar la parte de «Nombres y ubicaciones» que toca a F
  (`docs/superpowers/`)**

Localiza por texto:
```
**`docs/superpowers/` sigue sin resolver** — sigue en G5/sin dueño.
```
Reemplaza por:
```
**(campaña F, 2026-09-15): `docs/superpowers/` resuelta — decisión "se
  queda", ya con su frase escrita.** Verificado que la frase ya existía
  desde la campaña B (D5=b, commit `2294349`): `docs/arquitectura.md:5-13`
  dice que `docs/superpowers/` son instantáneas (`tier: log` por convención
  de ruta) que no se actualizan y se citan con su fecha. No hacía falta
  ningún cambio nuevo — Task 8 de F solo lo verificó y lo cita aquí.
```

- [ ] **Step 11: corregir con evidencia los ítems caducados del `§4` de la
  consulta de origen que le tocan a F**

Localiza el ítem «La campaña de evicción de la KB está descalibrada» y añade
al final:
```
  **(campaña F, 2026-09-15): CERRADO — hecho el 2026-09-10.** `Memoria
  v2:18` («La lista de partida la midió `exo budget` … eran 20, no 19»),
  commit `efd9abc`.
```

Localiza el ítem «`exo budget` va a colisionar de nombre» y añade al final:
```
  **(campaña F, 2026-09-15): caducado — parcialmente resuelto de hecho.**
  `grep budget docs/arquitectura.md` → vacío (ya no lo lista como
  "planeado"); `docs/instalacion.md:34` lo lista como verbo existente;
  `Comando::Budget` vive en `engine/src/main.rs:68` desde G4b. La colisión
  de nombre se resolvió de hecho: `budget` = bytes de KB. Queda solo el
  nombre de un verbo de coste de tokens hipotético, decisión #3 del paquete
  de Paul del 2026-09-15 (b/c: vive en `evals/` o no se construye hasta que
  haga falta) — fuera de alcance de F.
```

Localiza el ítem M4 #9 (`document/SKILL.md` omite `--db`) y añade al final:
```
  **(campaña F, 2026-09-15): CADUCADO.** `ArgsWriteAppend.db: Option<PathBuf>`
  existe (`engine/src/main.rs:172-176`); `document/SKILL.md:55` no menciona
  `--db` porque se resuelve por config (M5a-02) — es correcto tal cual, no
  un olvido.
```

Localiza el ítem «`reflex-baseline.sh` traga errores de `jq` con
`2>/dev/null`» y añade al final:
```
  **(campaña F, 2026-09-15): CADUCADO.** `reflex-baseline.sh` no tiene
  `2>/dev/null` sobre salida de `jq`; `:12-14` valida cada línea con `jq -e .`
  y lista las inválidas por stderr. Verificado leyendo el script completo.
```

Localiza el ítem `kb-demo` en 8 ficheros de test y añade al final:
```
  **(campaña F, 2026-09-15): conteo corregido, sigue abierto.** Hoy son
  **11** ficheros (`git grep -l kb-demo -- engine/tests`), no 8, más
  `engine/src/{buscador,inicia,lib}.rs` en comentarios/plantilla. El fix
  (`kb-demo` → `kb-test`) es alcance de G (toca `engine/tests/` y
  `engine/src/`), no de F — F solo corrige el conteo caducado.
```

Localiza `## Estado` → fila «Pendientes» (contiene «C8 (M3+M1b, cutover de
skills)») y añade una nota al final de esa fila:
```
 (**F, 2026-09-15: C8 verificado HECHO** — 9 skills en `plugins/exo/skills/`
  + `plugins/exo/agents/executor.md` + `.claude-plugin/marketplace.json`
  sirviendo `plugins/exo/` como marketplace único; lo que queda de M1a/M1b es
  checklist de máquina, ver «Máquina Linux» más abajo, no trabajo de fábrica)
```

Localiza el ítem `budget_prose_drift` con «para cuando exista el gate de
paridad con Go» y añade al final:
```
  **(campaña F, 2026-09-15): bloqueador CADUCADO.** Los gates de paridad
  corrieron en la campaña D y kbx dejó de ser dependencia de nada
  (`docs/backlog.md:181`). El fix queda sin bloqueo — es alcance de G (Rust
  puro, `engine/src/lint.rs`), no de F.
```

Localiza el ítem D6 (`backlog:243`, cita `main.rs:215`) y corrige la cita:
old_string (dentro de ese ítem):
```
  - [ ] (c) decidir si el default de `exo search --type` (`main.rs:215`)
```
new_string:
```
  - [ ] (c) decidir si el default de `exo search --type` (`main.rs:241` —
    corregido 2026-09-15, campaña F: la línea citada, `:215`, quedó
    desactualizada tras ediciones posteriores)
```

Localiza el ítem `walk_kb` frente a `walk_kb_excluyendo` y añade al final
del párrafo:
```
  **(campaña F, 2026-09-15): nota para G.** `walk_kb` también la usa
  `engine/src/doctor.rs:312,451`, no citado por el ítem original — unificarla
  (decisión #9 del paquete de Paul) afecta a `doctor`, no solo a `indexer`.
```

- [ ] **Step 12: item fuera de alcance — anotar por qué**

`KB-exo:44` («`script_del_plugin` ordena lexicográficamente») vive en
`Backlog — exo.md` de `wisdom-paul`, un repositorio distinto de `exo` — no en
`docs/backlog.md` de este repo. F no lo toca (no es un fichero de este
worktree); queda anotado aquí para que quien lea este plan no lo busque en
`docs/backlog.md` por error.

- [ ] **Step 13: verificación final**

Run: `grep -c "campaña F, 2026-09-15" docs/backlog.md`
Expected: al menos `12` (una por cada Step 1-11 que añade una cita —
cuéntalas: Step 1 añade 2, Steps 2-10 añaden 1 cada uno = 9, Step 11 añade 7
= 18 en total; usa esto como cota inferior, no como número exacto a
perseguir — lo que importa es que cada ítem tocado tenga su cita, no el
conteo total).

Run: `git diff --stat -- docs/backlog.md`
Expected: solo inserciones (`+`), cero líneas borradas — el sync añade
citas, no reescribe el historial del backlog.

- [ ] **Step 14: commit**

```bash
git add docs/backlog.md
git commit -m "docs(f, backlog): sync — cierra los ítems de F y corrige con cita los caducados del §4 de la propuesta"
```

---

## Self-review (contra el brief y `propuesta.md` §2)

**Cobertura**: los 9 ítems del «Esbozo de tasks» de la propuesta tienen
tarea (1:1 con las Tasks 1-9 de este plan, con las correcciones de alcance
documentadas en cada una). Los tres extras del brief — rename de `lint`
(Global Constraint + Task 7), idioma de identificadores (Task 8), y el cierre
del §4 en la task de sync (Task 9) — están cubiertos explícitamente.

**Placeholders**: cero. Cada task trae código bash/YAML/Markdown completo,
comandos con output esperado literal (verificado ejecutándolos contra el
árbol real de `campana-f` en `99ddd05` durante la escritura de este plan,
salvo donde se declara explícitamente que la verificación es remota —
Task 4 Step 4, Task 6 Step 5 — porque dependen de ShellCheck en
`ubuntu-latest` o de `actions/cache` en GitHub).

**Consistencia**: los nombres de fichero (`scripts/test-docs-vivos.sh`,
`scripts/test-hooks-json.sh`) coinciden entre el header de cada Task, sus
`Files:`, y las citas de Task 9. El job renombrado en la Task 7
(`static-checks`) es el mismo que las Tasks 2 y 3 citan en sus mensajes de
step y en Task 9 Step 7.

**Verificado contra el código, no contra la propuesta a ciegas** — hallazgos
que corrigen o matizan `propuesta.md` §2, todos con evidencia ejecutada
durante la escritura de este plan:

1. La lista de required checks de la Decisión #8 (§5 de la propuesta) omitía
   el job `exec-bit` y usaba ids de job en vez de los nombres mostrados que
   GitHub Branch Protection realmente usa — corregido en la Task 7 con la
   tabla completa de 12 checks.
2. El gate de "docs vivos" no puede aplicar sus checks (a)-(c) a
   `docs/backlog.md` sin generar falsos positivos reales: `docs/backlog.md`
   cita a propósito «Sin CI» (evidencia de un ítem cerrado) y una versión de
   un repo ajeno (`v2.2.1` de un clon de ECC). Verificado ejecutando el gate
   dos veces contra el árbol real. Corregido: el gate cubre los tres
   declarativos, no los cuatro «core».
3. `docs/instalacion.md` §7 cita `exo diff-since` y `exo history`
   **a propósito**, para decir que nunca se portan — un check ingenuo de
   "todo subcomando citado debe existir" dispararía ahí un falso positivo.
   Corregido con la exclusión de secciones `## ... NO ...`.
4. `README.md:117-125` cita las rutas de los nueve hooks como
   `scripts/<x>.sh` cuando la ruta real es `plugins/exo/scripts/<x>.sh` — un
   bug de documentación real, encontrado al construir el gate de la Task 2
   (no estaba en la propuesta de origen) y corregido en el mismo commit.
5. **jq en esta máquina (Windows 11 + Git Bash) emite CRLF** — sin
   `tr -d '\r'`, tanto el gate de hooks.json como cualquier comparación de
   versión vía jq fallan en silencio. Verificado con `od -c` sobre la salida
   real de `jq -r`. Ninguna mención de esto en la propuesta de origen;
   documentado como Global Constraint y aplicado en las Tasks 2 y 3.
6. ShellCheck no está instalado en esta máquina Windows, y el binario que
   descarga `ci.yml` es un tarball Linux — no ejecutable aquí. La Task 4 lo
   declara y usa `bash -n` como proxy de verificación local, dejando el
   veredicto real de ShellCheck al primer run de CI.
7. `docs/backlog.md:243` cita `main.rs:215` para el default de
   `exo search --type`; hoy es `main.rs:241`. Corregido en Task 9 Step 11.
8. La frase pedida para «`docs/superpowers/` se queda» **ya existía**
   (campaña B, `docs/arquitectura.md:5-13`) — Task 8 no edita nada ahí, solo
   verifica y cita; corregido frente a la propuesta, que lo daba como
   pendiente.

**Dudas para Paul** (ninguna bloquea la ejecución de este plan; todas son
para el momento de branch protection o para campañas futuras):

- El nombre elegido para el job renombrado (`static-checks` / "checks
  estáticos") es una propuesta razonable, no una cita literal de Paul —
  confirmar en el PR si prefiere otro (p.ej. "gates estáticos", en línea con
  el vocabulario que usa `propuesta.md`).
- Decisión #8 de branch protection: este plan no la activa. Cuando Paul la
  active, la lista de 12 checks de la Task 7 es la que debe copiar tal cual.
