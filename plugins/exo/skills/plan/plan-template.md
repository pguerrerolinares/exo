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
- El encabezado es `### Task N: <nombre>` con N entero; `task-brief` extrae
  por él.
- Un plan que no cumple esto no rompe nada: `orchestrate` cae a secuencial y
  lo deja visible.
