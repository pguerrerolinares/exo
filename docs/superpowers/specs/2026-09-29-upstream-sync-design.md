# upstream-sync — portar a exo lo que cambia en obra/superpowers

**Fecha:** 2026-09-29 · **Estado:** diseño validado por secciones en sesión; auditado (Fable) y corregido.

## Problema

Seis skills de exo (brainstorm, plan, orchestrate, tdd, debug, verify) son destilación de obra/superpowers. Portar a mano no escala: exo se quedó tres releases atrás (6.2 → 6.4.2) y la comparativa del 2026-09-29 encontró huecos reales. exo es un pack completo que reemplaza a superpowers, así que tiene que ir a la par.

## Decisiones

| Eje | Decisión | Descartado |
|---|---|---|
| Autonomía | El bot detecta, triagea, porta y abre PR; Paul mergea | Merge automático: los gates disponibles miden forma, no comportamiento de una skill en prosa |
| Reloj | Tags de release de obra/superpowers | PRs mergeados a `dev`: hay cambios sin liberar y reverts dentro del ciclo (#2214/#2335) |
| Atribución | API de GitHub (`gh pr list --state merged` con `files`, cualquier rama base) | `git log` sobre `main`: es lineal, squash de releases; 33 de 38 commits en `skills/` sin `(#N)` |
| Runtime | Routine cloud de Claude Code, semanal, con dos repos adjuntos: `pguerrerolinares/exo` y `obra/superpowers` | GitHub Action (API key aparte); cron local (depende de la máquina encendida) |
| Issues upstream | Fase 2 | Son hipótesis sin repro; el reloj de releases ya filtra |

## Estado versionado

`docs/upstream/ledger.md`, con cuatro bloques:

1. `upstream_tag: <tag>`: último tag revisado (línea propia, primera del bloque de estado).
2. Mapeo **fichero→fichero** upstream → exo, con excluidos declarados (`CREATION-LOG.md`, `test-pressure-*.md`, `find-polluter.sh`) y ficheros upstream sin equivalente (`re-review-prompt.md`).
3. Divergencias deliberadas: `| id | skill exo | qué diverge | motivo | fuente |`.
4. Filas: `| PR | skill | triage | estado | motivo | hash |`.
   - `triage` ∈ {aplica, parcial, ya cubierto, no aplica, duda}.
   - `estado` ∈ {propuesto, portado, rechazado, pendiente, —}.
   - `—` para no aplica y ya cubierto.

El ledger inicial lleva tag + mapeo + divergencias y **cero filas**.

`docs/upstream/sync-prompt.md` es el prompt del bot. El prompt guardado en la routine dice solo: "Lee y sigue docs/upstream/sync-prompt.md del repo exo."

## Pasada semanal

0. **Reconcilia** con `scripts/upstream-reconcile.sh`: `propuesto` con commit `port(upstream#N)` alcanzable desde `main` pasa a `portado`. Después:
   - Si hay un PR `upstream-sync` abierto (rama `upstream-sync/*`): latido y fin.
   - Los PRs `upstream-sync/*` cerrados sin merge cuyo rechazo aún no conste: sus filas se registran como `rechazado` (motivo = URL del PR) en el PR siguiente, y `upstream_tag` avanza igualmente para no reproponer la misma ventana.
1. **Clone y licencia:** si `upstream_tag` no está en `git tag` del clone, `git fetch --tags`; si sigue sin estar, latido con alerta y fin (un clone sin tags no es "sin tags nuevos"). Después, `LICENSE` de upstream contra `plugins/exo/LICENSES/superpowers.LICENSE`. Si difiere: latido con alerta y fin, sin portar.
2. **Retoma pendientes**, antes de detectar tags: las filas `pendiente` con motivo `tope`, `gate` o `respondida` se portan primero (mismo tope de 5 por PR, contando también lo nuevo), con el diff de `gh pr diff <N> -R obra/superpowers` limitado a los ficheros mapeados. Las `pendiente` por `duda` no se retoman solas: el latido las lista con su pregunta y Paul responde editando la fila en `main` (`triage` a su decisión, `estado` `pendiente`, motivo `respondida`); la pasada siguiente la trata como tope.
2b. **Detecta** tags posteriores a `upstream_tag`. Si no hay tags nuevos ni pendientes que retomar ni rechazos por registrar: latido y fin. Si hay:
   - Contenido: diff neto entre tags de los ficheros mapeados.
   - Atribución: PRs mergeados en la ventana de fechas de los tags que tocan ficheros mapeados. Se descartan los PRs de release dev→main: se atribuye a los PRs miembros.
3. **Triagea** por (PR, skill), citando líneas de exo y del ledger. Si choca con una divergencia → `no aplica`, citando su id. Si el fichero upstream no está en el mapeo → `triage=duda`, `motivo=mapeo: <fichero>`, y el PR propone la fila de mapeo sin portar nada. Si upstream elimina o recorta algo que exo no tiene → `no aplica`; si elimina algo que exo sí tiene, se evalúa si exo debe eliminarlo también o es divergencia.
4. **Porta** `aplica` y `parcial`, **máximo 5 portes por PR**; el resto queda `pendiente`, con motivo `tope`.
   - Un commit por PR upstream: `port(upstream#N): <resumen>`.
   - Cada commit actualiza la versión citada en la cabecera de atribución (`superpowers X.Y.Z`: `# Derived from superpowers …` en scripts, `(superpowers X.Y.Z, MIT ©` en markdown) de los ficheros que toca.
   - `duda` → `pendiente`, con la pregunta concreta en `motivo`.
   - Si el PR toca `plugins/exo/`, el último commit sube el PATCH de exo en `plugin.json` y `marketplace.json` (decisión de Paul: lo hace el bot; `test-versiones.sh` y `plugin-bump-gate.sh` lo exigen). Un PR solo-ledger no sube versión.
5. **Gates:**
   - `scripts/check-skill-refs.sh` y `scripts/test-plugin.sh` en verde.
   - Tabla obligatoria en el cuerpo del PR, `movimiento upstream → fichero:línea exo`, por porte.
   - Los bytes y tokens (`claude --plugin-dir ./plugins/exo plugin details exo`) son dato del informe, no gate.
6. **Abre PR** desde la rama `upstream-sync/<tag>` con título `upstream-sync <tag>`. Lleva los commits, el ledger (filas `propuesto` con hash, `upstream_tag` avanzado) y el informe.
7. **Latido:** comentario en el issue `upstream-sync: estado` (fijado) con fecha ISO, tag revisado, PRs vistos/propuestos/pendientes, pendientes por motivo (las `duda` con su pregunta), ratio de rechazo y una línea final fija `estado: ok` o `estado: alerta`. Es alerta si: licencia cambiada, ledger roto, clone sin tags, fallo a mitad, ratio de rechazo > 1/3 o cualquier salida anómala. Todas las salidas de los pasos 0-6 terminan aquí.

## Watchdog

Workflow `.github/workflows/upstream-watchdog.yml`, diario. Solo cuentan los comentarios con una línea `estado: ok|alerta`. Falla si el más reciente de ésos está en `alerta`, si tiene **más de 8 días**, si no hay ninguno o si el issue no existe. Un comentario humano no renueva el reloj. GitHub avisa por email del rojo. Cubre la routine pausada (conexión caducada a 72 h, cap diario) y el "verde = solo arrancó".

## Errores y límites

- **Reestructuración upstream:** `duda: mapeo` (paso 3).
- **Release grande:** tope de 5.
- **PR sin mergear:** solo latido (paso 0).
- **Revert dentro del ciclo:** el diff neto entre tags lo anula.
- **Cambio de licencia:** paso 1.
- **Degradación del triage:** el bot detecta los PRs `upstream-sync/*` cerrados sin merge y registra sus filas como `rechazado` (paso 0). El latido reporta el ratio de rechazo sobre las últimas 15 filas propuestas/portadas/rechazadas (ventana, para que un rojo no se vuelva permanente); solo las filas `propuesto` de un PR cerrado pasan a `rechazado`; si supera 1/3, `estado: alerta` con "re-corre el eval" (el eval de la primera pasada).
- **Sin verificar, se comprueba con un "Run now" antes de programar:** coste por pasada, `claude` en el PATH del sandbox, clone del repo adjunto.

## Primera pasada = eval pre-registrado

- `upstream_tag: v6.1.1`.
- La verdad conocida es la lista cerrada v6.1.1 → v6.4.2: 24 PRs, 37 filas (PR, skill). Vive **fuera del repo**, para que el bot no pueda leerla, y Paul fija las 11 filas de confianza media antes de correr.
- Las filas de confianza media que son choque de doctrina se deciden como divergencias del ledger inicial o como `aplica`.
- **Umbral pre-registrado:** 0 filas `ya cubierto` que en la verdad sean aplica/parcial, y ≥ 90% de acierto en la etiqueta de triage, con `aplica` y `parcial` como una sola clase (llevan a la misma acción: portar; decisión de Paul 2026-09-30). Lo puntúa `scripts/upstream-score.sh`.
- Si pasa, ese PR es el cierre de huecos del camino A. Si no pasa, no se programa la routine y se revisa el prompt.

## Fuera de alcance

document, distill, recon-first, reflejos y engine (sin original upstream); issues upstream (fase 2); merge automático.

## Testing

- `check-skill-refs`: un fixture que referencia una skill inexistente falla.
- `upstream-reconcile`: en un repo git temporal, la fila `propuesto` más el commit `port(upstream#N)` en `main` pasa a `portado`; sin el commit sigue en `propuesto`.
- `upstream-watchdog`: un latido de hace 9 días falla; uno de ayer, verde; sin issue, falla; último latido en alerta, falla; un comentario sin línea de estado no cuenta.
- `upstream-score`: fixtures de verdad y ledger con resultados conocidos.
- El juicio del triage solo se prueba con el eval de la primera pasada.
