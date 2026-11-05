---
permalink: "{{KB_NAME}}/projects/_template"
title: Plantilla de proyecto
tags: [plantilla]
tier: stable
semilla: true
repos: []
---

# Plantilla de proyecto

Una nota de `projects/` es el **destilado canónico** de un frente de trabajo:
el estado actual, condensado, no un diario de lo que ha ido pasando. Cuando
el proyecto tiene un frente activo, esta nota es la que un agente debe leer
para entender "dónde está esto ahora" sin tener que reconstruirlo a partir
del historial completo de la bitácora.

## Campos que suele llevar el destilado

- **Qué es**: una o dos frases que sitúan el proyecto — objetivo, alcance,
  para quién es.
- **Estado actual**: en qué punto está ahora mismo. Esto se sobrescribe con
  cada avance relevante, no se acumula histórico aquí.
- **Decisiones vivas**: las decisiones de diseño o de enfoque que siguen en
  pie y condicionan el trabajo futuro. Una decisión que ya no aplica se
  retira, no se tacha ni se deja como nota al margen.
- **Frentes abiertos / próximos pasos**: qué queda pendiente, si aplica.
- **Reglas duras**: las reglas mecánicas del proyecto (sección `## Reglas
  duras`, más abajo).
- **Enlaces relacionados**: a `learnings/` relevantes o a otras notas de
  `projects/` con las que este proyecto se relaciona.

## Cómo se actualiza

Un avance del proyecto va como **delta** sobre esta nota: se edita el campo
que corresponda (normalmente "estado actual" y, si procede, "decisiones
vivas"), no se añade un párrafo nuevo al final describiendo el avance como si
fuera un registro cronológico. Para el registro cronológico está la bitácora
del proyecto en `log/`: ahí sí va un apunte corto y con fecha por cada
avance, en modo append.

## Reglas duras

Esta sección la lee `exo rules` y el hook la inyecta al arrancar una sesión
en el repo. Para que la ficha llegue a su repo, `exo rules` casa la carpeta
del repo git con la ficha por, en este orden de prioridad: `slug`, `repos:`
(lista de carpetas o URLs de clonado; es lo que cubre un proyecto de varios
repos) y, por último, el nombre del fichero. Rellena `repos:` en el
frontmatter con las carpetas de los repos de este proyecto.

Una regla por línea, con guion, máximo 10, y solo reglas mecánicas: algo que
un agente pueda cumplir o comprobar sin criterio (un comando que no se
ejecuta, un fichero que no se toca). Con la sección vacía no se inyecta nada.
Comprueba la cobertura con `exo rules --coverage`.

Una nota de `projects/` que no es la ficha de ningún repo (un anexo de hechos
técnicos, por ejemplo) lleva `anexo: true` en el frontmatter: `exo doctor` no
le exige `repos:` ni esta sección y `exo rules --coverage` la marca `anexo`.
