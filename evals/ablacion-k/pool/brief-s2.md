# Brief fijo — prompts de S2 (campaña K, errata E5.4)

Vas a escribir encargos de programación para un agente de código (Claude
Code headless, ≤ 40 turnos, sin nadie a quien preguntar). Cada encargo sale
de un commit real de un proyecto open source (wagtail o django-oscar). Tienes
**solo** el mensaje del commit y el código en el estado **anterior** al
commit. El agente tendrá que implementar el cambio, y lo juzgarán los tests
originales del commit, que ni tú ni él veis.

## Por tarea

- Entrada: el `mensaje` del commit (en `s2-base.json`) y la fuente en el
  estado padre: `/home/paul/.cache/exo-ablacion-k/fuentes/<id>/`. Esta
  fuente es **solo lectura**; para probar algo, copia al scratchpad.
- Escribe un encargo en castellano, como una issue bien escrita: qué
  comportamiento falta o falla, dónde (módulo, vista, endpoint, función) y
  qué se espera. Puedes leer la fuente para que el encargo sea concreto:
  nombres reales de clases, rutas y parámetros.
- **No** des la implementación, **no** pegues código de solución y **no**
  nombres ficheros ni funciones de test. Tampoco inventes requisitos que el
  mensaje no respalde. Si el mensaje es ambiguo, redacta el encargo en el
  sentido más literal del mensaje.
- Termina el encargo con: «Añade o ajusta los tests que haga falta.»

## Prohibido

No abras `/home/paul/.cache/exo-ablacion-k/s2/` (los clones con historial:
contienen la solución), ni `gold/s2/*/tests_ref/`, ni ninguna web. No uses
`git log`/`git show` fuera de la fuente (la fuente no tiene historial).

## Salida

En `/home/paul/.cache/exo-ablacion-k/gold/s2/<id>/tarea.json`:
`{"setup": false, "prompt": "...", "estrato": "S2", "check": {"tipo": "script"}}`.
Tu respuesta final: los ids hechos, y cualquier mensaje que no diera para un
encargo razonable.
