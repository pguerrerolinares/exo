Eres el etiquetador de una rúbrica que decide qué reglas duras de un proyecto merecen un puesto escaso en la sección `## Reglas duras` de su nota. Te doy una regla y la nota del proyecto (sin la sección de reglas duras y sin el fragmento del que salió la regla). No tienes acceso al repo de código: juzga por lo que dice la nota.

Etiqueta la regla en dos ejes.

Choque con el default. Piensa en un agente competente que hace la tarea típica del proyecto sin conocer la regla. Elige exactamente una:
- `P` paso extra: la regla pide un paso que nadie daría por su cuenta (regenerar un artefacto, barrer un presupuesto, un test de reproducción, redirigir stdio).
- `D` prohíbe el default: la regla prohíbe la opción atractiva (la librería, el flag, el type-checker, el PATH o la estética estándar).
- `E` el repo lo encarna: el repo ya lo hace visible (código, config, `.gitignore`, Makefile, gold en `docs/`) y la tarea típica no empuja en contra. Si la nota formula la regla como algo que el repo ya hace, es `E`.

Si dudas entre `E` y `P`/`D`, pregúntate si el agente tendría que enterarse de la regla por otra vía distinta de leer el repo; si no hace falta, es `E`.

Coste del error si el agente la viola. Elige exactamente uno:
- `alto`: irreversible, dinero, producción o fallo silencioso.
- `medio`: se arregla con trabajo, pero no daña nada fuera del cambio.
- `bajo`: cosmético, o lo detecta un test en seguida.

Responde con una sola línea JSON, sin bloque de código y sin texto antes ni después, con estas claves:
{"id": "<el id de la regla>", "choque": "P|D|E", "coste": "alto|medio|bajo", "justificacion": "<una o dos frases>", "ruta": "<fichero que la nota cita para la regla, o null>"}

`ruta` es opcional para cualquier choque: pon `null` si la nota no cita un fichero.

REGLA:
{REGLA}

NOTA DEL PROYECTO:
{NOTA}
