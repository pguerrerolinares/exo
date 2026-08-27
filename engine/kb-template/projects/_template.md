---
permalink: "{{KB_NAME}}/projects/_template"
title: Plantilla de proyecto
tags: [plantilla]
tier: stable
semilla: true
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
- **Enlaces relacionados**: a `learnings/` relevantes o a otras notas de
  `projects/` con las que este proyecto se relaciona.

## Cómo se actualiza

Un avance del proyecto va como **delta** sobre esta nota: se edita el campo
que corresponda (normalmente "estado actual" y, si procede, "decisiones
vivas"), no se añade un párrafo nuevo al final describiendo el avance como si
fuera un registro cronológico. Para el registro cronológico está la bitácora
del proyecto en `log/`: ahí sí va un apunte corto y con fecha por cada
avance, en modo append.
