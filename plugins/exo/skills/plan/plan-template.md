# Plantilla de plan

**Cuándo usar:** al escribir el plan completo, para el header y el formato de
cada tarea. Destilado de `writing-plans/SKILL.md` (superpowers 6.4.2, MIT ©
2025 Jesse Vincent). Ningún bloque de código es obligatorio.

## Header

```markdown
# Plan: [feature]

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** [una frase]

**Architecture:** [2-3 frases]

**Tech Stack:** [tecnologías/librerías clave]

**Spec:** `docs/superpowers/specs/<fichero>.md`

**Global Constraints:**
- [requisito project-wide, valores exactos verbatim de la spec, una línea
  cada uno; toda tarea los hereda]

## Setup del worktree

[Comandos literales que deja un worktree recién creado listo para correr
tests (instalar dependencias, generar código). Omite la sección si no hace
falta nada.]

## Cierre de ola

[Comandos literales, uno por stack, que componen la «suite completa» de
este repo: unit, generadores, e2e. Lo que no esté aquí no se corre al cerrar
una ola.]
- `comando stack 1` → verde = [qué output]
- `comando e2e` → verde = [qué output]

## Mutación

[Instrumento de mutación por directorio, para lo que `review-package` no
autodetecta (cargo-mutants, mutmut 2.x, stryker). Un comando que imprime
`cazados/total` en su última línea, o `manual` si el stack no tiene
instrumento: entonces el executor declara la tabla de mutación manual en su
report. Si todo se autodetecta, escribe `manual`.]
- `android`: `./scripts/pitest-diff.sh`
- `web`: manual

## Excluir de la review

[Pathspecs que `review-package` quita del stat y del diff (generados,
binarios, lockfiles). Sin `/`, cruzan directorios (`*.png` = cualquier png).
Lo excluido sale listado en la cabecera del package. Omite si no hay.]
- `*.png`
- `app/src/generated/**`

## Recursos compartidos

[Locks, puertos, dispositivos, BD destructiva, config git: qué es
compartido entre worktrees y qué tarea lo toca. «ninguno» si no hay.]

## Olas

[Orientativa: las olas reales las calcula `orchestrate/scripts/task-dag` desde
`Files` e `Interfaces`; si difieren, manda `task-dag`.]

- Ola 1, en paralelo: T1, T2. [por qué son independientes]
- Ola 2: T3 (wiring), consume T1-T2.
```

## Tarea

```markdown
### Task N: [Nombre del componente]

**Files:**
- Create: `path/exacto/nuevo.ext`
- Modify: `path/exacto/existente.ext`
- Test: `path/exacto/al/test.ext`

**Interfaces:**
- Consumes: `firma exacta` @Task M
- Produces: `nombre(param: Tipo) -> Retorno`

**Tests:** (cada test se escribe y se ve fallar antes del código, `exo:tdd`;
sin tests: `Tests: n/a — <motivo>`)
- `nombre_del_test`: `entrada` da `salida exacta de la spec`. Falla si
  [el fallo concreto que caza].

**Verificación:** `comando` → output esperado (verde = suite entera del
proyecto)

**Review Focus:**
- [≤5 inputs o fallos que la spec implica y nadie nombra, más probable
  primero; cada uno con su test arriba.]

**Notas:** [solo un algoritmo que firma y tests no determinan, o copy exacto
que fija la spec. Si no hay, omite el campo.]
```

## Formato parseable (lo lee `task-dag`)

- `**Files:**` e `**Interfaces:**` son líneas propias, con sub-bullets `- `.
- Files: `Create|Modify|Test: <path>`, un path por bullet, entre backticks.
- Interfaces: `Produces: <firma>` y `Consumes: <firma> @Task M`, un bullet
  por firma; M entero, obligatorio en Consumes.
- Mutación: `` - `<dir>`: `<comando>` `` o `` - `<dir>`: manual ``; `<dir>`
  relativo a la raíz, `.` = raíz (toma cualquier extensión: acompáñalo de
  `EXO_MUTATION_EXCLUDE`). Una línea mal formada sale como aviso en el package.
- El encabezado es `### Task N: <nombre>` con N entero; `task-brief` extrae
  por él.
- `## Setup del worktree`, `## Cierre de ola`, `## Mutación`,
  `## Excluir de la review` y `## Recursos compartidos` van en el header, antes de la primera tarea: `task-brief` extiende la
  última tarea hasta el final del fichero.
- `task-dag --lint` exige en el header `## Cierre de ola` (≥1 comando entre
  backticks), `## Mutación` (`manual` si no hay instrumento), `## Recursos
  compartidos` y `**Spec:**`; por tarea `**Tests:**` con ítem o `n/a — <motivo>`.
- Un plan que no cumple esto no rompe nada: `orchestrate` cae a secuencial y
  lo deja visible.
