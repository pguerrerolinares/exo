---
name: distill
description: Consolidación offline de la KB (sleep-time compute manual): colapsa bitácoras/sesiones en destilados, chequea presupuestos por tier, promueve doctrina repetida a core y refresca core-index. Usar al cerrar un frente o semanalmente.
---

# distill

**Qué es:** el "sleep-time compute" manual de la KB kb-demo. Mientras `/document`
escribe en caliente (al cerrar una sesión), `/distill` es mantenimiento offline —
Paul lo invoca al cerrar un frente o semanalmente para que la KB no crezca sin control
ni se llene de doctrina repetida sin destilar.

**Regla de oro: nada se borra.** Todo movimiento es `git mv` o edición; si algo deja de
vivir en su sitio original, se mueve a `archive/` o `log/`, nunca se elimina. El commit
final es el registro de qué se movió y por qué.

## Procedimiento (6 pasos)

### Resolución de rutas — antes de cualquier paso

Antes de ejecutar cualquier paso de este procedimiento, resuelve dos valores
con el mismo seam que usa `plugins/exo/scripts/test-contrato-engine.sh`:

- `$KB_ROOT` — raíz de la KB:
  `${EXO_KB:-$(exo config --json | jq -r '.data.kb.path // empty')}`. Si sale
  vacío, es **abstención ruidosa**: para y dile a Paul que ni `$EXO_KB` ni
  `exo config --json` resolvieron nada — no sigas con el procedimiento.
- `$KBX_BIN` — binario `kbx`: `${KBX_BIN:-$(command -v kbx)}`. `kbx` es una
  herramienta externa que puede no estar instalada en esta máquina (p.ej.
  Windows, donde su build todavía no está decidido). Si `$KBX_BIN` sale
  vacío, dilo explícitamente y **salta cada paso que dependa de `kbx`** en
  vez de fingir que corrió — no hay abstención silenciosa que valga para un
  paso que simplemente no se ejecutó. Es la misma disciplina que ya aplica
  el paso "Falla-fuerte" del Budget check (para con mensaje accionable si
  el binario que toca falta): generalízala al resto de usos de `kbx` en este
  procedimiento.
- `$EXO_BIN` — binario `exo`: `${EXO_BIN:-$(command -v exo)}`.

**Este skill invoca dos binarios, y lo dice por escrito**: `exo` para
`budget`/`ratchet`/`lint` (cutover G4c) y `kbx` para `rotate`/`stale`/
`diff-since`, que todavía no tienen destino en `exo`. Un skill que finge
haber migrado del todo es una trampa para el día que `kbx` no esté
instalado — mejor declarar la frontera tal cual está.

Todos los comandos de las secciones siguientes usan `$KB_ROOT`, `$KBX_BIN` y
`$EXO_BIN` — ninguna ruta literal.

### 0. Rotación de bitácoras (antes de cualquier chequeo)

**Precondición dura:** `git -C $KB_ROOT status --porcelain`
debe salir **vacío** antes de tocar nada. Si no sale vacío, **para** y pide a
Paul que commitee o guarde su trabajo antes de rotar — no sigas por tu
cuenta. El baseline de conservación del punto 2 (`git show HEAD:<ruta>`)
asume que `HEAD` es el estado justo antes de rotar, y la reversión del punto
3 (`git checkout -- <ruta>`) descarta lo que no esté commiteado sin forma de
recuperarlo: con el árbol sucio, ambas cosas quedan mal por construcción.

Corre `$KBX_BIN rotate --kb $KB_ROOT --json`. Si `data.rotations` trae entradas, sigue `rotacion.md`; si el binario no trae `rotate`, sáltalo y ve al paso 1.

### 1. Budget check

Corre `$EXO_BIN budget --json` y `$EXO_BIN ratchet --kb $KB_ROOT --json`. exit 3 = hay trabajo; detalle e interpretación en `chequeos.md`.

**Falla-fuerte:** si el binario no está o el schema-canary rompe, **para** con un mensaje accionable — no degrades a mano, /distill es offline y deliberado.

### 1b. Gate de deriva + priorización

Corre `$EXO_BIN lint --json` y `$KBX_BIN stale --json`. Señales de inyección rota: `chequeos.md`.

### 2. Split canon/bitácora por cada core/stable obeso

La nota canónica es el **estado vivo**; todo lo fechado/histórico se mueve a
`log/<slug>-bitacora.md`. Antes de partir, evicción editorial y test del
título: `consolidacion.md`.

### 3. Archivar sesiones de frentes cerrados

**Escanea solo lo cambiado.** No re-escanees toda la KB: corre
`$KBX_BIN diff-since distill/last --json`
(`{data:{ref,resolved,notes:[{path,permalink,status,insertions,deletions}]}}`)
para ver qué notas cambiaron desde la última consolidación.

**Bootstrap (el tag aún no existe — `git tag -l` está vacío hoy):** si
`distill/last` no existe, `diff-since` fallará al resolver el ref. Eso **no**
es fallo-fuerte: haz un **full scan** (sin `diff-since`) esta vez. Al terminar
el paso 5 (commit), crea/mueve el tag al HEAD del repo KB:

    git -C $KB_ROOT tag -f distill/last HEAD

Es la única mutación del repo KB que hace esta skill más allá de commitear notas.

Para cada nota en `sesiones/` que pertenezca a un frente ya cerrado: escribe un resumen
de 1-3 líneas a la bitácora del proyecto correspondiente (`log/<proyecto>-bitacora.md`)
y `git mv` la sesión completa a `archive/sesiones/`. Actualiza los `[[wikilinks]]`
entrantes que apuntaban a esa sesión para que no queden rotos.

### 4. Promover doctrina repetida

Si un patrón o principio aparece repetido en ≥3 bitácoras (señal de que ya no es
anécdota sino doctrina estable), propón su promoción a `[[doctrina-agentes]]` o al
`learning` que corresponda. No lo hagas automático — es una propuesta a revisar, no
una escritura silenciosa.

### 5. Refrescar índice y commit

Actualiza `[[core-index]]` para que refleje los cores y destilados activos (altas,
bajas de sección, nuevos punteros a bitácoras).

> Tras podar y partir, corre `$EXO_BIN ratchet --kb $KB_ROOT --seal`. Es **atómico**: o
> sella todo o no sella nada, y si falla lista cada techo sin 15% de aire con su
> objetivo de poda. Esa lista no es un error del sello: es trabajo que falta.
> **Nunca subas un techo para que pase** — el trinquete lo rechazará en el
> commit de todos modos.

Luego commit scoped con las mismas reglas git de Paul:

- `git -C $KB_ROOT add <ruta1> <ruta2> ...` —
  **nunca** `git add -A`, **nunca** `git add .`.
- **Nunca** `cd` encadenado con `git`; usa siempre `git -C <path>`.
- **No hagas push** — esa decisión es de Paul.
- Tras el commit, avanza el marcador de consolidación:
  `git -C $KB_ROOT tag -f distill/last HEAD`.
  (No se pushea; es un marcador local para el `diff-since` de la próxima corrida.)

## Delegación

Delegable a un subagente sonnet: los pasos 1-4 son mecánicos/de revisión y el subagente
puede volcar los splits/movimientos propuestos como un **diff revisable antes de
commit** — no comitees a ciegas lo que produzca el subagente, revísalo tú primero
(o Paul).
