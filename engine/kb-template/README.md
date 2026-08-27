---
permalink: "{{KB_NAME}}/README"
title: "{{KB_NAME}}"
tags: [contrato, readme]
tier: stable
semilla: true
---

# {{KB_NAME}}

Esta carpeta es una base de conocimiento (KB): un conjunto de notas en
Markdown pensado para que tanto una persona como un agente puedan
encontrar, en cualquier momento, "qué se sabe" sobre un proyecto o un tema,
sin tener que releer todo el historial de trabajo.

## Qué hay aquí

- **`core/`** — la doctrina e identidad estables de esta KB: los principios
  de fondo que casi nunca cambian.
- **`projects/`** — una nota por proyecto con su estado actual, ya
  destilado. Es el sitio para entender rápido "dónde está esto ahora".
- **`learnings/`** — principios reutilizables, extraídos de la experiencia,
  que no dependen de un proyecto concreto.
- **`log/`** — bitácoras cronológicas: qué se hizo y cuándo, una por
  proyecto, en formato append (nunca se reescribe una entrada antigua).
- **`archive/`** — material retirado de circulación activa pero conservado
  por si hace falta consultarlo más adelante.

Para el contrato completo de cómo un agente debe leer y escribir en esta
estructura (qué va a cada carpeta, la regla de oro de routing), ver
`AGENTS.md` en esta misma carpeta.

## Cómo se indexa

Las notas de esta KB se recorren periódicamente con un indexador: una
herramienta que lee cada fichero, extrae su frontmatter (título, tags,
permalink) y su contenido, y construye un índice de búsqueda a partir de
ello. No hace falta mantener ningún índice a mano — se regenera a partir de
las notas cada vez que se ejecuta.

Si llegaste aquí por `exo init`, el comando es `exo index`.

Cada nota debe llevar un frontmatter mínimo para que el indexador la trate
correctamente:

```yaml
---
permalink: {{KB_NAME}}/carpeta/nombre-de-la-nota
title: Título de la nota
tags: [algún-tag]
tier: stable   # o "log" para las notas de bitácora
---
```

## Cómo se busca

Una vez indexada, la KB se consulta mediante el buscador asociado al
indexador (por título, por tag, o por contenido de texto libre, según lo que
soporte la herramienta concreta que se esté usando). Con `exo`, es
`exo search "lo que buscas"`. El flujo habitual de
un agente es: buscar primero si ya existe una nota relacionada con lo que se
quiere anotar o consultar, y solo si no existe, plantearse crear una nueva
siguiendo la regla de oro descrita en `AGENTS.md`.
