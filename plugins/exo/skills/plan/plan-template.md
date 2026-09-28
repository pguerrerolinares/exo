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

- Ola 1, en paralelo: T1, T2. [por qué son independientes]
- Ola 2: T3 (wiring), consume T1-T2.

## Review Focus

- [≤5 inputs o fallos que la spec implica y nadie nombra: condición y
  comportamiento esperado, más probable primero. Cada uno lleva su test en
  la tarea dueña.]
```

Regla de olas: dos tareas comparten ola si sus `Files` son disjuntos y
ninguna consume, directa o transitivamente, algo que produce la otra.

## Tarea

```markdown
### Task N: [Nombre del componente]

**Files:**
- Create: `path/exacto/nuevo.ext`
- Modify: `path/exacto/existente.ext`
- Test: `path/exacto/al/test.ext`

**Interfaces:**
- Consumes: `firma exacta` (de Task M)
- Produces: `nombre(param: Tipo) -> Retorno`

**Tests:**
- `nombre_del_test`: `entrada` da `salida exacta de la spec`. Falla si
  [el fallo concreto que caza].

**Notas:** [solo un algoritmo que firma y tests no determinan, o copy exacto
que fija la spec. Si no hay, omite el campo.]
```

## Formato parseable (lo lee `task-dag`)

- `**Files:**` e `**Interfaces:**` son líneas propias, con sub-bullets `- `.
- Files: `Create|Modify|Test: <path>`, un path por bullet, entre backticks.
- Interfaces: `Consumes|Produces: <firma>`, un bullet por firma.
- El encabezado es `### Task N: <nombre>` con N entero; `task-brief` extrae
  por él.
- Un plan que no cumple esto no rompe nada: `orchestrate` cae a secuencial y
  lo deja visible.

## Qué contiene una tarea (recordatorio)

- Test: nombre + aserción con valores de la spec; la aserción nombra el fallo
  que caza.
- Código: firma, fichero, valores. El executor escribe el cuerpo.
- Tarea trivial: se pliega en la que la necesita. Diff esperado ≲ 400
  líneas. Wiring: una tarea por ola.
