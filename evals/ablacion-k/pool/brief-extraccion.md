# Brief fijo — extracción de reglas accionables (campaña K, errata E4)

Este texto se pasa **idéntico** a cada extractor. Solo cambia la lista de
notas del grupo.

---

Eres extractor de reglas para un experimento. Recibes una lista de notas
markdown de una KB personal (raíz `/home/paul/Documentos/proyectos/wisdom-paul`,
rutas relativas a esa raíz). Tu único trabajo es **listar las reglas
accionables** que contienen. No eliges, no puntúas y no redactas tareas.

## Qué es una regla accionable

Una afirmación prescriptiva sobre cómo debe actuar un agente de código o
cómo debe quedar su resultado, que cumpla **las tres** condiciones:

1. **Prescribe:** haz, no hagas, usa X, formato Y, orden Z. Los hechos de
   estado («v0.2.0 publicada», «el gate está abierto»), las opiniones sin
   consecuencia práctica y las descripciones de la arquitectura **no** son
   reglas.
2. **Es observable en una sesión:** un revisor que mire los comandos, los
   ficheros o la respuesta de esa sesión puede decir si se cumplió.
3. **Es concreta:** tiene sujeto y acción claros. «Sé riguroso» no lo es;
   «cita `fichero:línea` para cada afirmación sobre el código» sí.

Incluye las reglas de mantenimiento de la KB (presupuestos, tiers,
bitácoras): son accionables en tareas que tocan la KB. **No descartes** una
regla por parecerte buena práctica genérica. Márcala con `generica: true`.

## Formato de salida

Un JSON por línea (JSONL), en el fichero que se te indica, con estos campos:

- `id`: `"<grupo>-<n>"`, con n correlativo desde 1.
- `nota`: ruta relativa de la nota.
- `cita`: fragmento literal de la nota que contiene la regla, ≤ 200
  caracteres, copiado exacto.
- `regla`: paráfrasis imperativa, ≤ 30 palabras.
- `ambito`: una de `git`, `codigo`, `proceso`, `kb`, `estilo`,
  `dominio-proyecto`.
- `proyecto`: nombre del repo al que aplica, o `"transversal"`.
- `check`: una línea sobre cómo se verificaría el cumplimiento en una sesión.
- `generica`: `true` si un agente competente lo haría sin que se lo digan.

Deduplica dentro de tu grupo: si dos notas dicen lo mismo, un registro con
la cita más clara. No inventes reglas que no estén en el texto.

## Restricciones

- Lee **solo** las notas de tu lista. No uses `exo`, ni otras notas, ni el
  repo de exo.
- Escribe **solo** tu fichero de salida. No edites nada más ni hagas commit.
- Tu respuesta final: el número de reglas por nota y el total, nada más.
