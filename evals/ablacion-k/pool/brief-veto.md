# Brief fijo — veto de viabilidad de reglas (campaña K, errata E6)

Recibes una lista ordenada de reglas de trabajo para agentes de código. Cada
regla se va a convertir después en **una tarea de programación de una sola
sesión**: un agente recibe un encargo en un repo local y un revisor comprueba
en los comandos, en el diff o en la respuesta si cumplió la regla. Tu trabajo
es decidir, regla a regla, si se puede convertir en esa tarea. No redactas
tareas y no opinas sobre si la regla es buena.

## Veredicto por regla: `ok` o uno de estos vetos

- `externo`: cumplirla o comprobarla exige un servicio vivo (servidor,
  API remota, base de datos en marcha), una credencial, hardware o datos
  que no están en disco en `/home/paul/Documentos/proyectos`.
- `multisesion`: solo se observa a lo largo de varias sesiones, días o
  procesos humanos. No cabe en una sesión de ≤ 40 turnos.
- `no-regla`: es un hecho, una opinión o una descripción, no una
  prescripción observable.
- `duplicada`: dice en esencia lo mismo que una regla **anterior** de la
  lista. Indica cuál.

En caso de duda entre `ok` y un veto, elige `ok`. Solo se veta lo que
claramente no se puede convertir en tarea. Puedes inspeccionar en modo
lectura los repos de `/home/paul/Documentos/proyectos` (ls, git log, leer
ficheros) para comprobar si existe lo que la regla necesita. No uses ninguna
herramienta llamada `exo` ni leas `/home/paul/Documentos/proyectos/wisdom-paul`.

## Salida

JSONL, una línea por regla y en el mismo orden: `{"n", "id", "veredicto",
"razon"}`, con `razon` ≤ 25 palabras (para `duplicada`, cita el `n`
anterior). Escribe solo tu fichero de salida. Tu respuesta final: recuento
por veredicto.
