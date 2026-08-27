---
permalink: "{{KB_NAME}}/log/_template"
title: _template
tags: [plantilla]
tier: log
semilla: true
---

# Plantilla de bitácora

Una entrada de bitácora es un apunte corto que registra **qué pasó y
cuándo**, en el momento en que pasa. Es el complemento cronológico del
destilado canónico de un proyecto (`projects/`): mientras el destilado dice
"así está el proyecto ahora", la bitácora dice "esto es lo que se hizo, en
este orden".

## Formato: append, no edición

Cada avance añade una entrada nueva al final del fichero de bitácora del
proyecto correspondiente. No se reescriben ni se resumen entradas
anteriores — eso sería tratar la bitácora como si fuera el destilado. Si una
entrada antigua queda obsoleta o contradicha por una posterior, se deja tal
cual: la bitácora es un registro histórico, no un documento vivo que se
mantiene consistente con el presente.

## Qué lleva cada entrada

- **Fecha** (o momento identificable) al inicio de la entrada.
- **Qué se hizo**: una o dos frases, en el nivel de detalle suficiente para
  reconstruir el avance si hace falta, sin repetir todo el contexto — para
  eso está el destilado del proyecto.
- Opcionalmente, un enlace a la nota de `projects/` que esta entrada
  actualiza, si el avance también se ha volcado ahí como delta.

## Cuándo NO usar la bitácora

Si lo que quieres registrar es un principio reutilizable más allá de este
proyecto, no va en la bitácora: va como learning en `learnings/`. La
bitácora es específica de un proyecto y de un momento; el learning es
genérico y perdura.
