---
name: distill
description: Consolidación offline de la KB (sleep-time compute manual): colapsa bitácoras/sesiones en destilados, chequea presupuestos por tier, promueve doctrina repetida a core y refresca core-index. Usar semanalmente (y al cerrar un frente).
---

# distill

**Qué es:** el "sleep-time compute" manual de la KB kb-demo. Mientras `/document`
escribe en caliente (al cerrar una sesión), `/distill` es mantenimiento offline —
El dueño de la KB lo invoca **semanalmente** (y al cerrar un frente) para que la KB no crezca sin control
ni se llene de doctrina repetida sin destilar. Cada pasada hace, sin excepción,
la promoción de doctrina (paso 4) y el barrido por calor (`chequeos.md`); si no
hay nada que promover o barrer, **dilo** en el resumen — no lo calles.

**Objetivo de poda: ≤60 % del techo** de la nota (`kbx_budget_max` si existe, si
no el nominal del tier) para toda nota que se evicte. Salir de `no_air` (85 %) no
basta: `/document` muerde a 85 %, y podar justo ahí lo hace morder en el primer
cierre tras la pasada. Si el 60 % no se alcanza sin tocar reglas ni decisiones
vivas, la salida es **partición** (test del título, `consolidacion.md`); nunca se
recorta prosa viva para llegar.

**Regla de oro: nada se borra.** Todo movimiento es `git mv` o edición; si algo deja de
vivir en su sitio original, se mueve a `archive/` o `log/`, nunca se elimina. El commit
final es el registro de qué se movió y por qué.

## Procedimiento (6 pasos, más la revisión de sidecars)

### Resolución de rutas — antes de cualquier paso

Antes de ejecutar cualquier paso de este procedimiento, resuelve dos valores
con el mismo seam que usa `plugins/exo/scripts/test-contrato-engine.sh`:

- `$KB_ROOT` — raíz de la KB:
  `${EXO_KB:-$(exo config --json | jq -r '.data.kb.path // empty')}`. Si sale
  vacío, es **abstención ruidosa**: para y dile al dueño de la KB que ni `$EXO_KB` ni
  `exo config --json` resolvieron nada — no sigas con el procedimiento. La
  KB tiene que ser la raíz de un repo git, y sin él cada subcomando se
  comporta distinto: `targets` falla con un mensaje que nombra la condición
  y el remedio (`git init`; `exo::gitx::es_repo_git`, decisión A2 de G4b);
  `stale` falla con el error crudo de git (necesita el último commit de cada
  nota); `ratchet` se abstiene con exit 0 (no hay historia contra la que
  medir); `budget`, `lint` y `rotate` no miran git y funcionan igual.
- `$EXO_BIN` — binario `exo`: `${EXO_BIN:-$(command -v exo)}`.

**Desde la campaña D (2026-09-14) este skill invoca un solo binario.** `exo`
cubre `budget`/`ratchet`/`lint` (cutover G4c) y, desde ahora,
`rotate`/`stale` también. `history` y `diff-since` **no existen en `exo`**
—no se portan, se sustituyen por `git` directo (paso 3)— así que ningún
paso de este procedimiento depende ya de `kbx`.

Todos los comandos de las secciones siguientes usan `$KB_ROOT` y `$EXO_BIN`
— ninguna ruta literal, y ningún `$KBX_BIN`.

### Primero — sidecars `.distill/*-moved.md` pendientes

Primer paso de cada pasada, antes de rotar: `ls $KB_ROOT/.distill/*-moved.md`.
Los deja el auto-distill de `/document` (formato en
`plugins/exo/skills/document/auto-distill.md`), que commitea directo en la rama
principal; el padre de `/document` aplica el sidecar, y los que quedan son los
que no llegó a aplicar. Por cada uno: `exo write append --from` en la bitácora
(relativo al cwd; idempotente por el rechazo de cabeceras duplicadas, engine ≥
0.7.0: con uno anterior la doble aplicación duplica sin avisar), verifica que cada
bloque quedó literal en destino, borra el sidecar y commitea.

Después audita los commits aterrizados desde `distill/last`:
`git -C $KB_ROOT log --fixed-strings --grep='docs(kb): auto-distill' distill/last..HEAD`.
Cada bloque movido trae su línea `Movido: … — cerrado|superado: «<cita>»`; un
bloque sin cita, o cuya cita no lo cierra de verdad, se devuelve al canon con un
commit (desde el sidecar o la bitácora).

- **Partición ya ejecutada** (línea `Hija:` en el commit): revisa el título de
  la hija con el test de `consolidacion.md` y que cada puntero `→ [[hija]]` de la
  madre apunta a bloques literales en ella; lista la hija en `[[core-index]]`
  si la madre está.
- **Propuesta de partición** (commit `--allow-empty`): ejecútala aquí con el
  test del título (`consolidacion.md`) o descártala.
- **Un fallo se revierte** con `git revert` del commit.
- **Nunca rehacerlos automáticamente:** quemaría una pasada por semana por
  nota caliente.

Con el árbol limpio tras esto, sigue al paso 0.

### 0. Rotación de bitácoras (antes de cualquier chequeo)

**Precondición dura:** `git -C $KB_ROOT status --porcelain -- log/ archive/log/`
debe salir **vacío** antes de tocar nada (pathspec, no global: un cambio ajeno
en otra nota no invalida la rotación). Si no sale vacío, **para** y pide a
al dueño de la KB que commitee o guarde su trabajo antes de rotar — no sigas por tu
cuenta. El baseline de conservación del punto 2 (`git show HEAD:<ruta>`)
asume que `HEAD` es el estado justo antes de rotar, y la reversión del punto
3 (`git checkout -- <ruta>`) descarta lo que no esté commiteado sin forma de
recuperarlo: con el árbol sucio, ambas cosas quedan mal por construcción.

Corre `$EXO_BIN rotate --kb $KB_ROOT --json`. Si `data.rotations` trae entradas, sigue `rotacion.md`; si viene vacío, sigue directo al paso 1.

### 1. Budget check

Corre `$EXO_BIN budget --json` y `$EXO_BIN ratchet --kb $KB_ROOT --json`. exit 3 = hay trabajo; detalle e interpretación en `chequeos.md`.

**Falla-fuerte:** si el binario no está o el schema-canary rompe, **para** con un mensaje accionable — no degrades a mano, /distill es offline y deliberado.

### 1b. Gate de deriva + priorización

Corre `$EXO_BIN lint --json` y `$EXO_BIN stale --json`. Señales de inyección rota: `chequeos.md`.

### 1c. Cobertura de reglas duras

Corre `../../scripts/cobertura-reglas.sh` (relativo al directorio de este skill), con `EXO_BIN` y `EXO_KB=$KB_ROOT`. Es informativo: lista las notas al `CAP` y los repos donde hubo sesiones sin reglas. Qué hacer con cada fila: `chequeos.md`.

### 2. Split canon/bitácora por cada core/stable obeso

La nota canónica es el **estado vivo**; todo lo fechado/histórico se mueve a
`log/<slug>-bitacora.md`. Antes de partir, evicción editorial y test del
título: `consolidacion.md`.

### 3. Archivar sesiones de frentes cerrados

**Legado v1: solo aplica si `git -C $KB_ROOT ls-files sesiones/` no está vacío.**
Desde el contrato v2 `/document` no crea notas de sesión (lo que era una sesión
es una entrada fechada en `log/<proyecto>-bitacora.md`) y el archivado lo hace
`exo rotate` del paso 0 (a `archive/log/`). Si `ls-files` sale vacío, salta el
paso con una línea en el resumen («paso 3: sin `sesiones/` trackeadas, archivado
por `exo rotate`») y sigue al 4; un directorio `sesiones/` vacío o sin trackear
no cuenta como «no hay `sesiones/`».

**Escanea solo lo cambiado.** No re-escanees toda la KB: `exo` no trae
`diff-since` (decisión de la campaña D — no se porta: con git ya delante,
duplicarlo dentro del binario no añade nada que `git diff`/`git log` no den
ya). Corre en su lugar:

    git -C $KB_ROOT diff --stat distill/last..HEAD -- '*.md'
    git -C $KB_ROOT diff --name-status distill/last..HEAD -- '*.md'

para ver qué notas cambiaron desde la última consolidación (el segundo
comando da el estado por fichero: `A`/`M`/`D`).

**Bootstrap (el tag aún no existe — `git tag -l` está vacío hoy):** si
`distill/last` no existe, los dos `git diff` de arriba fallan al resolver la
referencia. Eso **no** es fallo-fuerte: haz un **full scan** (sin diff) esta
vez. Al terminar el paso 5 (commit), crea/mueve el tag al HEAD del repo KB:

    git -C $KB_ROOT tag -f distill/last HEAD

Es la única mutación del repo KB que hace esta skill más allá de commitear notas.

Para cada nota en `sesiones/` que pertenezca a un frente ya cerrado: escribe un resumen
de 1-3 líneas a la bitácora del proyecto correspondiente (`log/<proyecto>-bitacora.md`)
y `git mv` la sesión completa a `archive/sesiones/`. Actualiza los `[[wikilinks]]`
entrantes que apuntaban a esa sesión para que no queden rotos.

### 4. Promover doctrina repetida

**Obligatorio en cada pasada.** Si un patrón o principio aparece repetido en ≥3
bitácoras (señal de que ya no es anécdota sino doctrina estable), propón su
promoción a `[[doctrina-agentes]]` o al `learning` que corresponda. No lo hagas
automático — es una propuesta a revisar, no una escritura silenciosa. Si no hay
candidatos, escribe «sin doctrina que promover» en el resumen.

### 5. Refrescar índice y commit

Actualiza `[[core-index]]` para que refleje los cores y destilados activos (altas,
bajas de sección, nuevos punteros a bitácoras).

> Tras podar y partir, corre `$EXO_BIN ratchet --kb $KB_ROOT --seal`. Es **atómico**: o
> sella todo o no sella nada, y si falla lista cada techo sin 15% de aire con su
> objetivo de poda. Esa lista no es un error del sello: es trabajo que falta.
> **Nunca subas un techo para que pase** — el trinquete lo rechazará en el
> commit de todos modos.

Notas nuevas **solo por partición** (`consolidacion.md`): ninguna otra pasada
crea notas. Si se creó alguna, `[[core-index]]` la lista y la madre queda como puerta.

Luego commit scoped con las mismas reglas git del dueño de la KB:

- `git -C $KB_ROOT add <ruta1> <ruta2> ...` —
  **nunca** `git add -A`, **nunca** `git add .`.
- **Nunca** `cd` encadenado con `git`; usa siempre `git -C <path>`.
- **No hagas push** — esa decisión es del dueño de la KB.
- Tras el commit, avanza el marcador de consolidación:
  `git -C $KB_ROOT tag -f distill/last HEAD`.
  (No se pushea; es un marcador local para el `diff-since` de la próxima corrida.)

## Delegación

Delegable a un subagente sonnet: los pasos 1-4 son mecánicos/de revisión y el subagente
puede volcar los splits/movimientos propuestos como un **diff revisable antes de
commit** — no comitees a ciegas lo que produzca el subagente, revísalo tú primero
(o el dueño de la KB).
