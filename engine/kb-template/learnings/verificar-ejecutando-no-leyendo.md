---
permalink: "{{KB_NAME}}/learnings/verificar-ejecutando-no-leyendo"
title: La lectura verifica coherencia; solo la ejecución verifica verdad
tags: [verificacion, agentes, testing, ejecucion, mutation-testing]
tier: stable
semilla: true
---

# La lectura verifica coherencia; solo la ejecución verifica verdad

Una prosa técnica bien escrita es internamente coherente tanto si es cierta como si no, así que quien solo lee no tiene señal contra la que chocar. Lo mismo vale para un plan, una spec, un README o el informe de un subagente. Los defectos que importan salen de **ejecutar** el artefacto, de comprobar una afirmación contra su fuente, o de ambas cosas. Esta nota recoge cómo se comprueba que lo que vuelve de un agente vale. Los mecanismos por los que un instrumento falla sin gritar están en [[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]]; el caso de medir con un instrumento que nadie ha probado, en [[Un negativo no vale si el instrumento no está validado|learnings/instrumento-validado-antes-de-medir]].

## Ejecutar antes que leer

- **Ejecutar > leer.** Pide al verificador "no te fíes de la prosa ni de los comentarios; recalcula desde los datos crudos". Cambia la clase de hallazgo que encuentra.
- **Documentación ≠ artefacto.** Cuando se auditaron herramientas ajenas, en todas el artefacto no hacía lo que su documentación decía (un gate descrito como bloqueante que devolvía "permitir" en ambas ramas; un "cero peticiones de red" impreso sin comprobarlo). Lo cazó siempre leer el código ejecutado o ejecutarlo, nunca leer la doc.
- **Los defectos salen de ejecutar el plan, no de revisarlo.** Una spec pasó dos reviews adversariales y los ejecutores encontraron al escribir código: una contradicción entre el snippet del plan y su propio test, dos tests verdes por la razón equivocada (sus fixtures caían en una rama de exención y la línea cubierta nunca corría) y un mensaje de error que enterraba su causa. Una spec coherente describe igual de bien un sistema que funciona y uno que no. El ciclo test-primero (`exo:tdd`) es un instrumento de detección que la revisión no sustituye.
- **El ejecutor que para ante una contradicción vale más que el que adivina.** Si edita el test "para que pase", el hallazgo se pierde. Una decisión de producto bien tomada sigue decidiendo casos que nadie previó, así que muchas veces no hace falta escalar.
- **Un stub escrito desde tus supuestos no verifica la herramienta.** Prueba contra el binario o servicio real y conduce el producto publicado.

## Verificadores con contexto limpio

- **Contexto limpio > más capacidad.** El verificador no comparte tus supuestos, así que comprueba lo que tú das por hecho. Mándalo a comprobar una afirmación contra su fuente, no a "revisar".
- **Dale la lente correcta**: el fallo típico no es un error de lógica visible, es una desalineación silenciosa entre lo que un artefacto declara y lo que hace. Con esa lente una primera pasada encontró bloqueantes que revisiones previas no habían visto. Añade los fallos de rondas anteriores y pídele que audite también lo que él propuso.
- **Ante fallo en cadena, cambia el punto de partida**: un agente limpio sin tus hipótesis. Detalle en [[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]].
- **No te fíes de los informes de tus propios verificadores.** Más de una vez una comprobación a mano evitó aplicar una corrección del auditor que estaba mal. Verifica también la sospecha: acusar de fabricación es una afirmación más y necesita su evidencia. Comprueba primero, comunica después.
- **La review final de rama ve lo que ninguna review de tarea puede ver**: deriva entre módulos, contratos duplicados a ambos lados de una frontera, caminos que cruzan varias tareas sin cobertura. Ver [[El padre coordina y valida; el ejecutor implementa|learnings/orquestador-limpio]].
- **Tiene coste y rendimiento decreciente.** Aplícalo donde un fallo sale caro; si ya no aporta hallazgos, para.

## El verificador también se verifica

- **Muta el verificador.** El padre no valida leyendo el diff ni fiándose del informe del hijo: muta el código y exige que la suite se entere. Distingue "tiene tests" de "tiene tests que detectan algo". Mejor aún si lo hace el revisor y no el implementador. Hay que limpiar bytecode cacheado: una mutación del mismo tamaño y mtime puede dar un falso "sobrevive". Mejor con herramienta que a mano.
- **RED contra el código pre-fix.** Corre el test nuevo contra la versión anterior al arreglo (`git show <sha>:<fichero>`) y pega el fallo. Un test de regresión que nadie ha visto fallar no es una regresión cubierta. Exige que sea por test, no por suite, y que un error de colección o de import cuente como RED sucio: si el fix añade un símbolo, el pre-fix revienta al importar y todo "falla", tautológico incluido.
- **Un verificador debe equivocarse rechazando, no aceptando.** Uno que acepta lo corrupto no se distingue de uno que no existe.
- **El oráculo vive fuera del loop.** Si el agente decide solo cuándo ha terminado, el juez es el mismo componente falible que hace la tarea. Todo guardrail serio mueve el criterio de parada fuera del loop. En la literatura, la inmensa mayoría de los re-chequeos que un agente se hace a sí mismo solo confirman lo que ya había decidido.
- **Un LLM-judge solo no basta.** En la literatura, los jueces pierden buena parte de los hacks y casi todos los semánticos, y una porción relevante de parches "resueltos" pasa los tests siendo incorrecta (preprints; tómalo como dirección, no como cifra). Verificar un sistema no determinista con otro no determinista compone la incertidumbre: ancla con tests, hashes, ejecución real.
- **El gate sobre el brazo barato.** Pregunta de checklist: "¿sobre qué brazo corre esto, y es el brazo donde vive el código nuevo?".

## Lo que tu máquina no ve

- **Ejecutar verifica verdad solo en la plataforma donde ejecutas.** Un linter limpio sobre código condicionado a otro SO es un verde que no compiló esa parte, y los tests de esa rama ni se pueden correr. Dos tareas validadas solo en un SO fallaron en los otros dos (un comando que se comporta distinto entre plataformas y cuyo error se silenciaba).
- **Declara en el commit qué no has visto correr.** Un "tests verdes" que calla que tres nunca se ejecutaron es el fallo silencioso que el código persigue.
- **CI en varios SO antes de `main`.** Es la regla mecánica, no una intención.
- **La suite verde no caza la pérdida de datos.** En código que muta algo irreemplazable, presupuesta una pasada adversarial y un experimento sobre datos reales, y verifica por bytes, no por unidades de dominio.

## Cuándo no aplica

- **Cambios triviales** (renombrar, una constante): compilar basta. Escala el esfuerzo al riesgo (`exo:verify`).
- **Cuando el oráculo cuesta más que el error**: si no hay forma barata de ejecutar, dilo en vez de sustituirla por una lectura más cuidadosa.
- **Leer sigue siendo útil** para el diseño y para encontrar *qué* ejecutar; lo que no hace es probar que funciona.
