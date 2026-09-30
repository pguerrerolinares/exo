# Olas: mecánica

Pre-flight en tabla destilado de `subagent-driven-development/SKILL.md` (superpowers 6.4.2, MIT © 2025 Jesse Vincent).

Regla: dos tareas comparten ola si sus `Files` son disjuntos y ninguna
consume, directa o transitivamente, algo que produce la otra (`Interfaces`).

## Calcular

- `scripts/task-dag PLAN` imprime `{"olas": [[ids]…], "avisos": [...]}`. Cada
  aviso se copia tal cual al ledger:
  - `DAG: secuencial (<motivo>)`: formato antiguo, Files ilegibles, `Consumes`
    sin `@Task M` o con `@Task` inexistente. Una tarea por ola.
  - `DAG: reordenado (Task i consume Task j posterior)`: consumo hacia delante
    sin ciclo; las olas ya vienen en orden topológico (desempata la numeración).
  - `DAG: error (<ciclo>)` (olas `[]`, exit 0): ciclo o autoconsumo. Es una
    decisión del humano: entra en la pregunta batcheada del pre-flight, antes de
    empezar (corregir el plan o fijar el orden). Nunca se ejecuta con error.
- La sección `## Olas` del plan es orientativa; manda `task-dag`.
- Solo si el script falla: a mano con la misma regla, leyendo `Files` e
  `Interfaces`. Ambiguo o no parseable ⇒ una tarea por ola, en orden, y
  `DAG: secuencial (<motivo>)` en el ledger.

## Pre-flight: tabla

La salida del pre-flight es una tabla en el ledger, no un veredicto: «plan
limpio» sin filas no es un scan que corriste. Los pares de tareas que
comparten fichero o interfaz los da `task-dag` (olas y avisos): cópialos, no
los recalcules a mano. Añade solo una fila por tarea de coherencia interna:
¿los tests que especifica concuerdan con el código que especifica, y los
ficheros que crea con los que toca después? Cada hallazgo entra en la pregunta
batcheada al humano, junto a su fila.

## Ejecutar una ola

- Ola de 1 tarea: flujo normal.
- Ola de ≥2: un worktree por tarea (rama propia desde el HEAD de la ola) y
  todos los dispatches en UN mensaje. Cada executor commitea en su worktree.
  Pasa siempre rutas ABSOLUTAS del worktree principal para el brief y el
  report (el `Ruling:` va en el report; el orquestador lo copia al ledger).
- Prompt paralelo: scope, self-contained, constraints, output esperado. Al
  volver: lee cada summary, verifica que no choquen, spot-check.
- **Sin poder crear worktrees** (p.ej. sesión ya aislada en uno): paralelo en
  el MISMO worktree, `Files` disjuntos, y los executors **no commitean**
  (`[COMMIT_POLICY]` = no commitees). El orquestador:
  - commitea en serie con rutas explícitas (`git add <rutas de Files>`, nunca
    `-A`/`.`), registrando BASE_i = HEAD antes de cada commit, y hace el
    package por tarea con BASE_i..HEAD_i;
  - sin commit, cada executor corre solo los tests de su tarea; la suite
    completa la corre el orquestador tras commitear todas;
  - compara `git status --porcelain` con la unión de `Files`: cualquier
    fichero de más es BLOCKED o Ruling.
- Pipeline de reviews: la tarea N+1 no espera a que se revise la N; package +
  reviewer en cuanto cada una da DONE.

## Cerrar una ola

0. Merge solo de tareas con ambos verdictos ✅ (Critical/Important arreglados
   en su worktree, con re-review). Si una tarea sigue en review, la ola no
   cierra.
1. Merge de las ramas de la ola (en el mismo worktree no hay merge).
2. Suite completa. Roja ⇒ NO se abre la siguiente ola; fix dispatch con el
   output completo.
3. Conflicto de merge ⇒ executor de integración con los dos diffs. Si no
   resuelve, BLOCKED y pregunta al humano. Nunca `-X ours/theirs` a ciegas.
4. Solo hallazgos tardíos: Critical/Important de una tarea ya mergeada ⇒
   fix dispatch antes de abrir la siguiente ola.
5. Ola nueva solo con la anterior mergeada y en verde.

## Fix loop

Se activa con spec ❌, cualquier Critical/Important o un ⚠️ que confirmaste
como gap real. Antes de entrar salen dos rutas: los Minor no entran nunca
(`Task N: minor (deferred): <una línea>` al ledger, y el final los triagea), y
un finding plan-mandated o que choca con el texto del plan lo decide el humano.

Una ronda = un fix + una re-review acotada; **máximo 5 por tarea**:

- **Rondas 1-3: reanuda al executor original** con los findings verbatim (su
  contexto sigue intacto). Si no puedes escribir a un subagente vivo, despacha
  uno fresco con el brief, el report file y los findings: el report file es la
  memoria persistente en ambos casos.
- **Rondas 4-5: executor fresco en un modelo mayor** que el atascado, con
  brief, report file, findings abiertos y «un executor previo lo intentó [N]
  veces; ahora es tuyo, lee el report file para ver qué se probó». Tres
  resumes sin converger suelen significar que no ve su propio problema.
- **Toda ronda:** el fix re-corre los tests que cubren lo enmendado, añade su
  fix report al MISMO report file y devuelve el contrato corto. Antes de la
  re-review confirma tests cubridores + comando + output. Nombra los tests en
  el mensaje del fix: un arreglo de una línea no necesita la suite entera.
- **Re-review acotada:** `scripts/review-package PLAN FIX_BASE HEAD` (FIX_BASE =
  head que vio la review anterior) y `reviewer-prompt.md` con la lista de
  findings, brief, report y el path del package. Verdicta cada finding
  ADDRESSED / NOT ADDRESSED y marca roturas nuevas solo en el diff del fix;
  una rotura nueva Critical/Important se suma a los abiertos; lo demás va al
  ledger como minor diferido, nunca alarga el loop. Modelo: tier bajo-medio.
- Tras cada ronda, ledger: `Task N: fix round R/5 (X addressed, Y open — <findings>; commits a7..b7)`.
- El padre nunca arregla findings él mismo: su contexto queda para coordinar y
  un fix del controller se salta la review.

**Breaker.** Si tras la ronda 5 quedan findings abiertos, deja de despachar y
adjudica cada uno tú (tienes plan y contexto entre tareas):

- reviewer equivocado o punto discutible ⇒ aparca: `Task N: parked — <finding>
  — ruling: <por qué el código se queda>`;
- real pero nada posterior depende de él ⇒ aparca igual, con ruling «real y
  diferido»;
- real y load-bearing (una tarea posterior lo usa, o revela un defecto del
  plan) ⇒ PARA: `Task N: BLOCKED — <motivo>` y reporta al humano con el
  finding, el texto del plan y el historial de fixes.

Adjudicar antes del tope es pre-juzgar con otro nombre. Toda adjudicación es
entrada de ledger: el descarte silencioso está prohibido. Cierre:
`Task N: complete (commits …, <K> parked)` tras un breaker.

Review final: si devuelve findings, UN fix con la lista completa y exactamente
una re-review acotada del rango del fix; los residuos se adjudican como en el
breaker. No hay segunda ola de fixes: lo load-bearing residual sube al humano.

## Mutación (review-package)

`EXO_MUTATION=0` la desactiva; `EXO_MUTATION_TIMEOUT` fija el timeout en
segundos (por defecto 600; uno inválido avisa en la sección y usa 600). Muta en
un worktree temporal de `HEAD`: el árbol del usuario no se toca y vale
cualquier `BASE..HEAD`. En monorepo hay además una línea `MUTACIÓN [<dir>]: …`
por proyecto. Las tres herramientas mutan solo las líneas del diff: cargo con
`--in-diff`, `mutmut` 2.x con `--use-patch-file` (requiere `whatthepatch`:
`pip install 'mutmut[patch]'`, si falta da `no disponible`) y stryker con un rango
`fichero:inicio-fin` por hunk. Un diff que solo borra líneas da `no disponible`
sin llamar a la herramienta; `mutmut` 3.x siempre da `no disponible` (no acota
por CLI).

`EXO_MUTATION_MUTANT_TIMEOUT` (60 s por defecto; inválido avisa y usa 60) es el
timeout por mutante y aplica SOLO a mutmut: mutmut 2.5.1 no tiene timeout
configurable (corta a baseline×10, en una suite lenta más de 40 min colgado), así
que el script envuelve su `--runner` (el de su config, `[tool.mutmut]` en
`pyproject.toml` o `[mutmut]` en `setup.cfg`; si no, el default `python -m pytest -x
--assert=plain`) con un corte de N s. Un mutante cortado así cuenta como caught
(convención estándar: timeout = detectado); un baseline que tarde más de N s da
`parcial`. Stryker NO recibe `--timeoutMS`: conserva su default (5000 ms +
netTime×1,5), que ya corta. En cargo no aplica. En mutmut, 🤔 (sospechoso) es un
mutante matado lento y cuenta como caught.

`EXO_MUTATION_EXCLUDE` (vacío por defecto) son pathspecs de git separados por
COMAS, relativos a la raíz del repo (`alembic/versions/**,*_pb2.py,src/gen`): esos
ficheros salen del conjunto a mutar antes de agrupar por proyecto (migraciones,
código generado). Si excluyen todo, `no disponible (diff sin código de producción)`.

`EXO_MUTATION_SAMPLE=<N>` (150 por defecto; `0` lo desactiva y deja el comportamiento
de 1.5.1; inválido avisa y usa 150) y `EXO_MUTATION_SEED=<s>` (20260929) activan el
muestreo, SOLO en mutmut 2.x: `mutmut` enumera los mutantes del diff sin ejecutar tests
y, si hay más de N, corre solo N ids elegidos sin reemplazo con la semilla fija. La línea
sale `MUTACIÓN [<dir>]: c/n (p%) IC95 [a%, b%] — muestra de n/total (semilla s)` (IC95 de
Wilson) y ese proyecto NO entra en el agregado: se reporta aparte. Con N o menos mutantes
corre todos, como siempre. Cargo y stryker no muestrean (corren completos). El timeout
global cubre enumeración y muestra; si salta, `parcial` con el progreso de la muestra.
La selección es la del protocolo del verdict 2, `random.Random(s).sample(sorted(ids), n)`,
ejecutada con el python de mutmut (el de su shebang). Ninguna degradación es muda: sin ids
de `mutmut result-ids` o sin python la línea acaba en `(muestreo no disponible: <motivo>;
corrida completa)`; cargo y stryker con más de N mutantes acaban en `(muestreo no aplica:
<herramienta>)`. Con N o menos no se añade nada.

Con el timeout global, `parcial (timeout Ns) — progreso: k/n evaluados, c caught,
s supervivientes` conserva el conteo (mutmut y stryker; cargo y una salida
ilegible dan `progreso: no disponible`). En stryker `k/n` excluye NoCoverage,
`caught` incluye Runtime/CompileError y la línea llega con hasta 10 s de retraso:
no es comparable con el score final.

- Sin sección `MUTACIÓN:`, o `no disponible (<motivo>)`/`parcial (<motivo>)` ⇒
  esa línea al ledger; no bloquees ni corras mutación a mano.
- Coste: techo 600 s × N proyectos por package. `EXO_MUTATION=0` en re-reviews
  de fixes sin lógica nueva. En la review final, según si las tareas ya pasaron
  mutación: si todas la pasaron, `0`; si no, se deja activa.

## Ruling

El executor puede desviarse del contrato del plan (incluso de un test) si lo
deja como `Ruling: T<n> — <qué cambia> — <por qué>` en su report; el
orquestador (único escritor del ledger) lo copia allí. El reviewer lo trata como finding obligatorio. Si el Ruling cambia una interfaz, las
tareas de olas posteriores que la consumen lo reciben en su brief.

## Métricas de la serie (al cerrar la rama)

Las escribe el orquestador, una línea en el ledger: wall-clock y turnos de la fase plan (los lee del header del plan); KB del plan y
fracción de código; mutation score por herramienta; % de tests basura
(muestra clasificada por el reviewer final); nº y ancho de las olas.
