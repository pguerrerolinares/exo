# Pipeline de desarrollo A+ — plan como contrato, olas paralelas, mutación por herramienta

> Spec de brainstorm, 2026-09-29. Rama `auditoria-skills`. Se implementa en **dos
> planes** (§7). Estado: pendiente de revisión del dueño.

## 1. Problema

Hay dos quejas del dueño, y las dos se han medido (evidencia en §8):

1. **`exo:plan` tarda muchísimo porque el plan casi implementa la solución.**
   - n=110 planes: el 62% de las líneas son bloques de código.
   - El 87–98% de ese código acaba literal en el repo (n=9, cota superior).
   - Los planes crecen: mediana de 24,5 KB en julio frente a 84 KB en septiembre.
   - Cuando el plan se delega a un subagente tarda 18–56 min y hasta 212 turnos.
   - La causa es mecánica: la sección `No-placeholders` de `plan/SKILL.md` y `plan-template.md` exigen "código completo en cada paso". Upstream diagnosticó lo mismo (obra/superpowers#2333, v6.4.2).
2. **Tests basura.**
   - Muestra de n=57: el 19% no testea comportamiento (constantes, strings, frágiles, tautológicos).
   - Se concentra en tareas pequeñas y mecánicas: <10 KB de plan dan 6/16, el resto 5/41 (Fisher p=0,057).
   - No depende del origen: 8/45 en tests copiados del plan frente a 3/12 en los del executor, p=0,68.
   - Las reviews por tarea casi nunca detectan tests débiles; lo hacen solo cuando se pide mutación.

Hay dos requisitos añadidos: **velocidad** (paralelizar lo que se paralelice bien) y **comentarios en código "los justos"**.

## 2. Decisiones

- Scope: el pipeline de desarrollo, es decir `plan`, `orchestrate`, `executor`, `tdd` y los scripts de orchestrate. `brainstorm`, `verify` y `debug` quedan fuera de esta ronda.
- Se implanta **A+**. El **test-writer separado queda en cuarentena**. Los datos no muestran que el autor importe (p=0,68), el único apoyo directo es AgentCoder (con specs tipo docstring) y un test-writer Sonnet malinterpreta una spec ambigua igual que el executor.
- Sin A/B formal: con n=2-3 y una sola corrida, el ruido es del tamaño del efecto. La decisión sale de una **serie antes/después sobre las próximas 10 tareas reales** (§6).
- Modelos:
  - executor y reviewer por tarea: Sonnet 5.5;
  - review whole-branch: Opus.
  - Lo respalda el empate de Sonnet 5.5 con Opus 5.5 en tareas acotadas (CursorBench −2,3; Terminal-Bench 4 en empate según Vals AI) a mitad de precio.
  - El claim "Sonnet 5.5 > Opus 5.5 en agentic coding" **no se sostiene**. Solo gana en Terminal-Bench 4.0, con cifras de Anthropic y condiciones no comparables, y pierde FrontierCode por 8,2 puntos.

## 3. Arquitectura

```
brainstorm ─► spec
plan ─► CONTRATO (sin cuerpos de código)
        header: Goal · Architecture · Tech Stack · Global Constraints (verbatim) · Spec: <ruta>
        por tarea: Files · Interfaces (Consumes/Produces, firmas) · Tests (nombre + aserción
                   con valores exactos de la spec) · Review Focus (≤5) · Notas (solo algoritmo
                   que firma+tests no determinan)
        tarea = unidad con resultado verificable, diff esperado ≲ 400 líneas; tareas triviales
                se pliegan en la que las necesita; wiring concentrado en 1 tarea por ola
orchestrate
   task-dag PLAN ─► olas
   por ola (≥2 tareas ⇒ 1 worktree por tarea, dispatches en un mensaje):
      executor (Sonnet) ─► review-package (diff + mutación por herramienta) ─► reviewer (Sonnet)
   merge de ola ─► suite completa ─► siguiente ola
   final: review whole-branch (Opus) con MERGE_BASE
```

**Pipeline de reviews.** La tarea N+1 de la misma ola no espera a que se revise la N. Una ola nueva empieza cuando la anterior está mergeada y en verde. Los findings Critical/Important de una tarea ya mergeada van a un fix dispatch antes de abrir la siguiente ola.

## 4. Componentes

### Plan 1: prosa de skills

| Fichero | Cambio |
|---|---|
| `skills/plan/SKILL.md` | "No-placeholders" pasa a **"Qué contiene una tarea"**: el paso de test lleva nombre + aserción con valores; el de código lleva firma, fichero y valores de la spec, y el executor escribe el cuerpo. El lector deja de ser "cero contexto" y pasa a ser un ingeniero capaz que conoce la interfaz y el test. Se quitan los "2-5 min". Se añaden Review Focus, la regla de wiring, el plegado de tareas triviales y la regla de que la aserción declara el fallo que caza. El self-review suma **proporción** ("un plan más largo que el código que describe ya es el código") a cobertura, placeholders y tipos. El handoff sigue siendo `exo:orchestrate`. |
| `skills/plan/plan-template.md` | Se reescribe sin bloques de código obligatorios, con el formato de tarea de §3. Files e Interfaces llevan un formato parseable por `task-dag` (§4, plan 2). |
| `skills/orchestrate/SKILL.md` | La paralelización pasa de "solo dominios independientes" a **olas de `task-dag`**. Se añade el worktree por tarea en olas de ≥2 tareas, la suite completa tras el merge, el pipeline de reviews y el `Ruling:` en el ledger (el executor puede desviarse del contrato, incluso en un test del plan, dejando el motivo registrado; el reviewer lo trata como finding obligatorio). Se mantiene la paridad `exo:executor`, el ledger y las red lines. |
| `skills/orchestrate/implementer-prompt.md` | El executor escribe el cuerpo de tests y código. Si se desvía, `Ruling:`. |
| `skills/orchestrate/reviewer-prompt.md` | (a) Lee los mutantes supervivientes del package. (b) Pase de over-engineering separado del de corrección: una línea por hallazgo con tag `delete\|stdlib\|native\|yagni\|shrink` y cierre `net: -N lines possible`. (c) Check de comentarios: la proporción de líneas de comentario del diff no supera la del fichero que toca, y cada comentario contiene un porqué. Si no, finding Minor. |
| `agents/executor.md` | Escalera antes de escribir: ¿hace falta? → ¿existe en el codebase? → stdlib/plataforma → dependencia instalada → mínimo código. Salvedad literal: la escalera acorta la solución, nunca la lectura, y el cambio mínimo en el sitio equivocado es otro bug. Sin abstracciones especulativas (una interfaz con una implementación, config para una constante). **Comentarios**: solo un porqué que firma y cuerpo no contestan (invariante, workaround con enlace, decisión contraintuitiva, unidad de un valor mágico); prohibido parafrasear la línea siguiente, poner comentarios de sección o dejar TODO sin issue; test del borrado. **Qué no testear**: one-liners, glue, constantes, texto. Se aligera la verificación explícita pero se mantiene enseñar el output real de la suite. |
| `skills/tdd/anti-patterns.md` | Se sustituye por un port de `writing-good-tests.md` (superpowers 6.2.0): el test nombra el fallo que caza, expectativas derivadas independientemente, sin change-detectors ni tests de presencia de string, mutation check. Se mantienen las reglas de mocks actuales. |
| `skills/tdd/SKILL.md` | La suite del proyecto define "verde", no el fichero del test (#2110). Remite a qué no testear. |
| `README.md` del plugin, `LICENSES/`, `plugin.json` | Base superpowers 6.1.1 → 6.4.2 (la parte portada); atribución MIT a ponytail (DietrichGebert); bump de versión minor. |

### Plan 2: scripts

| Fichero | Contrato |
|---|---|
| `skills/orchestrate/scripts/task-dag` (nuevo) | Entrada: la ruta del plan. Salida (stdout, JSON): `{"olas": [[<ids de tarea>], ...], "avisos": [...]}`. Dos tareas comparten ola si sus `Files` son disjuntos **y** ninguna consume algo que produce la otra (transitivo). Un plan no parseable o ambiguo da una ola por tarea en orden y `avisos` no vacío. Exit 0 en los dos casos; exit ≠0 solo si falta el fichero. |
| `skills/orchestrate/scripts/review-package` (modificar) | Añade una sección `MUTACIÓN`: detecta la herramienta por el repo (`Cargo.toml` → `cargo-mutants`, `pyproject.toml`/`setup.py` → `mutmut`, `package.json` → `stryker`), la corre solo sobre los ficheros de producción del diff con un timeout configurable (por defecto 10 min) y lista score y supervivientes. Si no hay herramienta, hay timeout o falla, escribe `MUTACIÓN: no disponible\|parcial (<motivo>)`, visible y sin bloquear. |

## 5. Manejo de errores

Principio: cualquier degradación es **visible**; nunca hay fallo silencioso ni bloqueo por una herramienta ausente.

- **DAG no parseable**: secuencial, con `DAG: secuencial (<motivo>)` en el ledger.
- **Conflicto al mergear una ola**: el orquestador despacha un executor de integración con los dos diffs. Si no lo resuelve, `BLOCKED` y pregunta al humano. Nunca `-X ours/theirs` a ciegas.
- **Suite roja tras el merge**: no se abre la siguiente ola. Fix dispatch con el output completo.
- **Mutación no disponible o parcial**: el reviewer la trata como "cannot verify", el mecanismo que ya existe.
- **`Ruling:` que cambia una interfaz**: las tareas de olas posteriores que la consumen reciben el Ruling en su brief.

## 6. Testing y medición

- **Scripts (plan 2)**: TDD con fixtures de planes reales (bien formado, ambiguo, con wiring y ciclo) y un repo mínimo por lenguaje para la detección de mutación, con la herramienta falseable (ausente o con timeout).
- **Prosa de skills (plan 1)**: sin tests de strings sobre skills, que es justo el antipatrón que prohibimos. Se valida en uso.
- **Serie de medición** sobre las próximas 10 ramas reales. El ledger registra por rama:
  - wall-clock y turnos de la fase plan;
  - KB del plan y fracción de código;
  - mutation score por herramienta;
  - % de tests basura (muestra clasificada por el reviewer final);
  - número y ancho de las olas.
- **Línea base**: plan delegado de 18–56 min, 62% de código, 19% de basura, más un mutation score medido sobre 2-3 ramas antiguas antes de empezar la serie.
- **Criterio pre-registrado**:
  - A+ vale si la mediana del tiempo de plan baja ≥50%, la basura baja del 19% y el mutation score no empeora frente a la base.
  - El test-writer sale de cuarentena solo si, tras las 10 ramas, el mutation score queda por debajo de la base o la basura sigue por encima del 10%.
- **Límite**: es una serie antes/después sin control, y los cambios en el tipo de tarea confunden la comparación.

## 7. Partición

- **Plan 1: prosa de skills y plantilla.** Da la ganancia del tiempo de plan desde el primer día. `orchestrate/SKILL.md` describe las olas, pero mientras no exista `task-dag` las calcula el orquestador a mano con la misma regla.
- **Plan 2: los scripts `task-dag` y `review-package`**, con mutación.
- Cada plan deja el plugin funcionando.

## 8. Evidencia (resumen; los informes están en el scratchpad de la sesión)

- Upstream: obra/superpowers v6.4.2 (`8ca22db`, PR #2333); v6.4.1 (#2319 Review Focus, #2110 suite = verde, #2318 Native); v6.2.0 `writing-good-tests.md`.
- Literatura:
  - Fan et al. arXiv 2609.20804: con modelos fuertes el planning ahorra coste, no da precisión.
  - Liu et al. arXiv 2604.12147.
  - AgentCoder arXiv 2312.13010.
  - Chen et al. arXiv 2602.07900.
  - TDFlow arXiv 2510.23761.
- Guía de Anthropic: "Prefer general instructions over prescriptive steps"; Sonnet 5.5 "tends to add tests, documentation… even when you don't ask".
- ponytail (DietrichGebert, MIT):
  - escalera y no-abstracciones;
  - benchmark propio con Haiku 4.5, n=4, −54% de líneas;
  - fix en la causa raíz de 1/6 a 6/6 en Sonnet 4.6 y Opus 4.8;
  - no mide corrección ni tests.
- Consultor (Fable): diagnóstico de autoría no probado, veto → `Ruling`, mutación por herramienta, olas por DAG, regla de comentarios, A/B de n=3 = teatro.
- Nota: esto no solapa con la campaña K (rama `campana-k`), que mide la memoria con la doctrina de proceso apagada.
