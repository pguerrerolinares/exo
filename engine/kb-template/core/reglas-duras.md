---
permalink: "{{KB_NAME}}/core/reglas-duras"
title: Reglas duras globales
tags: [reglas, core]
tier: stable
semilla: true
---

# Reglas duras globales

Las reglas de esta nota valen para todos los repos y las inyecta el hook al
arrancar cada sesión, junto a las del proyecto (`exo rules`, campo `global`).
La nota es opcional: una KB sembrada antes de que existiera funciona igual,
sin reglas globales.

## Reglas duras

Una regla por línea con guion, máximo 10 y solo reglas mecánicas (algo que un
agente cumpla o compruebe sin criterio). Con la sección vacía no se inyecta
nada; añade las reglas debajo de esta línea.
