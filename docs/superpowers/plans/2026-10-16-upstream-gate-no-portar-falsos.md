# Plan: el gate de upstream-sync cuenta los «no aplica» falsos y exige verdad sellada (#59)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que `scripts/upstream-score.sh` tumbe el gate ante cualquier fila que el bot deja sin portar (`ya cubierto` o `no aplica`) cuando la verdad dice portar, y que solo puntúe contra una verdad cuyo sha256 se selló en el ledger antes de la pasada.

**Architecture:** dos piezas con ficheros disjuntos. T1 cambia el script de score y sus tests (clase de error y comprobación del sello). T2 cambia la documentación y el estado: campo `verdad_sello` en el ledger, reclasificación de #2258/plan, regla para el bot en `sync-prompt.md` y umbral y procedimiento en la spec. El contrato entre ambas es el formato de la línea `verdad_sello`.

**Tech Stack:** bash + awk (`scripts/upstream-score.sh`), `sha256sum` (coreutils; presente en Git Bash y en ubuntu-latest).

**Plan:** ~15 min, 6 turnos, ~7 KB

**Spec:** sin spec nueva (diseño aprobado en chat el 2026-10-16; fuente issue #59). Spec afectada: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md` (§ «Primera pasada = eval pre-registrado»).

**Global Constraints:**
- Clase única `no-portar-falsos`: fila emparejada por (PR, skill) cuyo triage en el ledger es `ya cubierto` o `no aplica` y cuya etiqueta en la verdad es `aplica` o `parcial`. Umbral pre-registrado: 0.
- Línea de salida exacta: `no-portar-falsos N (ya cubierto: A, no aplica: B)`, seguida de una línea por fila con el formato actual `  <pr> <skill> (verdad: <etiqueta>)`. La línea `ya-cubierto-falsos` desaparece.
- rc: 0 si `no-portar-falsos` = 0 y `pct >= umbral`; 1 si no; 2 en uso incorrecto, tabla ausente, fila malformada, umbral inválido o verdad sin sellar.
- Formato del sello en el ledger (línea propia, fuera de tablas): `verdad_sello: <sha256 de 64 hex en minúscula> <desde>..<hasta>`. Sin verdad sellada: `verdad_sello: —`.
- Verdad sin sellar = la línea `verdad_sello:` falta, vale `—` o su sha256 no coincide con `sha256sum <verdad.md>`. En esos casos, mensaje por stderr que contiene `verdad sin sellar` y exit 2.
- El bot nunca edita la línea `verdad_sello`.
- #2258/plan: `triage aplica`, `estado pendiente`, `motivo respondida`, `hash —`. D8 no se toca.
- Fuera de alcance: el etiquetado real del próximo tag tras v7.0.0.

## Olas

- Ola 1, secuencial: T1, T2. Los `Files` son disjuntos y ninguna consume código de la otra (solo el formato del sello, fijado en Global Constraints). Van en secuencia por simplicidad: son dos tareas pequeñas.

### Task 1: score con clase `no-portar-falsos` y verdad sellada

**Files:**
- Modify: `scripts/upstream-score.sh`
- Test: `scripts/test-upstream-score.sh`

**Interfaces:**
- Produces: `upstream-score.sh <verdad.md> <ledger.md> [<umbral_pct>]` (uso sin cambios; semántica de rc según Global Constraints)

**Tests:** (en `scripts/test-upstream-score.sh`; los nuevos se ven fallar antes del código, `exo:tdd`)
- Adaptación previa: `led_head` emite `verdad_sello: <sha> v0..v1` calculado sobre la verdad de cada caso. Hazlo con un helper que reciba la ruta de la verdad, o sellando después de escribirla. Todos los casos actuales siguen pasando con la salida nueva: donde hoy esperan `ya-cubierto-falsos N`, pasan a esperar `no-portar-falsos N`.
- `no_aplica_falso_tumba`: `build20` y luego `#101 tdd` pasa a `no aplica` en el ledger → rc=1, la salida contiene `no-portar-falsos 1 (ya cubierto: 0, no aplica: 1)` y `101 tdd (verdad: aplica)`. Falla si un `no aplica` falso solo resta del porcentaje: 19/20 = 95 % ≥ 90, así que hoy da rc=0.
- `ya_cubierto_falso_sigue_tumbando`: `#101 tdd` pasa a `ya cubierto` → rc=1 y `no-portar-falsos 1 (ya cubierto: 1, no aplica: 0)`. Falla si la fusión de clases pierde la subclase original.
- `no_aplica_con_verdad_no_aplica_acierta`: ledger y verdad con `no aplica` en la misma fila → cuenta como acierto, `no-portar-falsos 0`. Falla si la clase cuenta `no aplica` sin mirar la verdad.
- `parcial_en_verdad_tambien_cuenta`: verdad `parcial`, ledger `no aplica` → `no-portar-falsos 1`. Falla si solo se compara contra `aplica`.
- `sello_ausente_exit2`: ledger sin línea `verdad_sello:` → rc=2 y stderr contiene `verdad sin sellar`. Falla si puntúa sin sello.
- `sello_guion_exit2`: `verdad_sello: —` → rc=2, `verdad sin sellar`.
- `sello_distinto_exit2`: sello de otra verdad (sha de un fichero distinto) → rc=2, `verdad sin sellar`. Falla si basta con que la línea exista.

**Verificación:** `bash scripts/test-upstream-score.sh` → `FAIL=0`, rc=0; `shellcheck scripts/upstream-score.sh scripts/test-upstream-score.sh` → sin salida (si `shellcheck` no está en el PATH, dilo en el report). Positivo conocido sobre el ledger real: `git show main:docs/upstream/ledger.md > $TMP/l.md`; verdad mínima `$TMP/g.md` con la tabla `| PR | título | base | skill exo | etiqueta | evidencia | conf. |` y una fila `| #2258 | x | dev | plan | aplica | ev | alta |`; inserta en `$TMP/l.md`, tras `upstream_tag:`, `verdad_sello: $(sha256sum $TMP/g.md | cut -d' ' -f1) v6.1.1..v6.4.2`. Entonces `bash scripts/upstream-score.sh $TMP/g.md $TMP/l.md` → rc=1 y la salida contiene `no-portar-falsos 1 (ya cubierto: 0, no aplica: 1)` y `2258 plan (verdad: aplica)`. Pega el output en el report.

**Review Focus:**
- CRLF en el fichero de verdad (Windows): el sha256 es el de los bytes tal cual y no se normaliza. Lo cubre `sello_distinto_exit2` de forma implícita; documenta en el comentario de cabecera que el sello es sobre bytes.
- Dos líneas `verdad_sello:` en el ledger: cuenta la primera. Añade `sello_duplicado_usa_la_primera` (la primera es correcta y la segunda basura → rc según el score).
- La línea `verdad_sello:` dentro de una tabla o con espacios finales: tolerar espacios finales. Añade el test `sello_con_espacio_final_vale`.
- `sha256sum` imprime `\*fichero` en algunos modos: compara solo el primer campo.

**Notas:** actualiza el comentario de cabecera del script: Uso, Exit y el nuevo requisito del sello.

### Task 2: ledger, prompt del bot y spec

**Files:**
- Modify: `docs/upstream/ledger.md`
- Modify: `docs/upstream/sync-prompt.md`
- Modify: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`

**Interfaces:**
- Produces: línea `verdad_sello: —` en `docs/upstream/ledger.md`, justo después de `upstream_tag: v7.0.0`, separada por una línea en blanco como el resto de la cabecera.

**Tests:** n/a — solo documentación y estado. Lo cubren las suites existentes (abajo).

**Cambios exactos:**
- `ledger.md:138`: `| #2258 | plan | no aplica | — | D8 | — |` → `| #2258 | plan | aplica | pendiente | respondida | — |`. No muevas la fila (el ledger es append-only y el orden es cronológico).
- `sync-prompt.md`: en la sección del entregable del ledger (alrededor de la línea 91), añade una línea: el bot no edita nunca la línea `verdad_sello:`, que la sella el mantenedor antes de una pasada puntuada.
- Spec, § «Primera pasada = eval pre-registrado», viñeta «Umbral pre-registrado»: sustituye «0 filas `ya cubierto` que en la verdad sean aplica/parcial» por «0 filas `ya cubierto` o `no aplica` que en la verdad sean aplica/parcial (`no-portar-falsos`, #59)»; el resto de la viñeta se mantiene.
- Spec, nueva subsección `## Pasadas puntuadas siguientes` tras esa sección, con tres viñetas:
  - la verdad cubre un rango de tags que el bot no ha triageado, y el mantenedor la etiqueta sin abrir el PR del bot;
  - se guarda fuera del repo y su sello (`verdad_sello: <sha256> <desde>..<hasta>`) se commitea en `main` antes de la pasada;
  - el script se niega a puntuar sin sello (exit 2), y v6.1.1..v6.4.2 y v6.4.2..v7.0.0 ya no sirven de held-out.

**Verificación:** `bash scripts/test-upstream-reconcile.sh` y `bash scripts/test-upstream-watchdog.sh` → `FAIL=0`; `bash scripts/upstream-reconcile.sh docs/upstream/ledger.md HEAD` sobre una copia temporal del ledger → rc=0 y sin `propuesto-malformado` (la línea nueva no rompe el parser).

**Review Focus:**
- El reconcile o el watchdog leen la cabecera del ledger por posición: compruébalo con la ejecución de la Verificación.
- La fila #2258/plan con `estado pendiente`: el paso 2.1 del sync-prompt la retoma porque su `motivo` es `respondida`. Cita la línea del prompt en el report.
