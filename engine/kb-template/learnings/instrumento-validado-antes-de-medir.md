---
permalink: "{{KB_NAME}}/learnings/instrumento-validado-antes-de-medir"
title: Un negativo no vale si el instrumento no está validado
tags: [medicion, evals, evidencia, instrumentos, benchmarks]
tier: stable
semilla: true
---

# Un negativo no vale si el instrumento no está validado

Un número solo significa lo que crees si el instrumento que lo produjo hace lo que crees. Antes de aceptar "aquí no hay nada", "no coinciden" o "el candidato pierde", comprueba el detector, la pregunta, el control y el estado del mundo en que se midió. Los fallos de instrumento no gritan (ver [[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]]): el resultado parece válido y sobrevive a revisiones. Esta nota es la parte de medición; verificar que algo funciona de verdad está en [[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]].

## Validar el detector antes de creer el negativo

- **Un negativo no vale sin instrumento validado.** Comprueba que el detector encuentra el positivo que ya conoces; si el conocido no es del tipo que buscas, **inyecta uno sintético** con las propiedades reales y exige que salga primero del ranking. Aplica a un test que no falla cuando debería, a un linter que no marca el caso malo conocido, a una query que no devuelve el documento que sabes que existe.
- **"0 resultados" informa del instrumento, no del mundo.** Además, un negativo cerrado en falso deja señuelos: el fenómeno no detectado produce artefactos que se persiguen como señal.
- **Un negativo heredado se re-verifica con el instrumento correcto**, no con el que lo produjo. Un subagente que dice "está roto" o "no hay dato" tras un error de acceso también es un instrumento sin validar: reprodúcelo con una llamada propia antes de diseñar alrededor.
- **Calibra antes de medir, nunca después.** Un gate pre-registrado admite corrección con la ventana cerrada; abierta, cada ajuste es indistinguible de amañar. Señal de honestidad: que los ajustes vayan en direcciones opuestas. Y vigila que instrumento y sujeto no compartan canal (los smokes que validan el aparato escriben en el mismo log que la ventana que abren).

## Cuando el instrumento es un juez

- **La pregunta es el instrumento.** Un desacuerdo total que parecía condenar al modelo evaluado medía la redacción: la definición exigía "no se puede deshacer trivialmente" y el caso era uno que sí se deshace con un comando; el modelo contestó bien a la pregunta que se le hizo. Con las preguntas derivadas de las definiciones del juez de referencia, el desacuerdo desapareció. Antes de dar un negativo sobre un juez, **re-pregunta el mismo caso con la redacción del rival**; si el veredicto se invierte, medías tu prompt.
- **La referencia no es ground truth: estima su ruido.** Un clasificador de referencia se contradijo en la misma campaña (errores internos, un comando denegado que pasó al reejecutarlo, casi-duplicados tratados distinto). Ese ruido es el suelo de cualquier "acuerdo", y si la referencia deja pasar lo que debería parar, **el candidato acertando puntúa como falso positivo**. Mide su ruido antes de fijar el umbral, o adjudica a mano con rúbrica pre-registrada.

## Medir con controles y sin atribuir mal

- **Pipeline medido desde fuera: reconstruye el censo de la fase.** "No está en el resultado final" no dice qué fase lo causó: muchos elementos ni se extrajeron, otros cayeron por otro filtro, otros por deduplicación. Una métrica que parecía prometedora medía en realidad el momento en que actuaba la extracción; con el censo bien hecho el mismo modelo quedaba al nivel del azar. Mira qué entró a la fase, no qué salió del pipeline entero, y no cruces por claves truncadas.
- **Verifica el control contra el código, no contra el nombre del flag.** Un "brazo text" que sin flag caía al default hybrid hizo que el criterio comparase hybrid contra sí mismo, y daba la respuesta opuesta a la real. **Mide la separación entre brazos**: dos "modos distintos" con salida casi idéntica son el mismo código.
- **Valida la métrica desde la fuente, no la que tu instrumento hace fácil.** Un diseño puede construirse entero sobre una métrica relativa porque el banco de pruebas la ofrece de serie, cuando el objetivo real se puntúa con una absoluta: la métrica equivocada invierte el veredicto. Adjudica en el entorno de despliegue (la misma palanca puede ser null contra un entorno de prueba débil y dañina contra el real).

## Estocástico, pre-registro y estado del mundo

- **Un seed no adjudica.** El pass@1 de un solo run varía de forma apreciable incluso a temperatura 0, y un resultado favorable o desfavorable de un seed puede invertirse al añadir más. Para un kill-criterion: multi-run con intervalo de confianza, y análisis de potencia antes de adjudicar mejoras pequeñas. El mecanismo puede ser invariante mientras los absolutos son ruido.
- **pass^k no es pass@k.** pass@k es "al menos uno de k"; pass^k es "todos los k". Úsalo solo en gates cuyo veredicto pasa por juicio de un LLM: **en un gate determinista la varianza es un bug del gate**, no algo que promediar.
- **Pre-registra el criterio de decisión.** Un empate sin criterio se racionaliza; con criterio, el empate es información.
- **Diseña para separar, no para aprobar.** Si todo sale al máximo el instrumento no mide. Des-satura con un modelo más débil, un corpus mayor o preguntas más amplias.
- **Un número solo se compara con otro del mismo estado** (día, corpus, máquina, ancla). Un antes/después entre sesiones es deriva, no efecto: mide el A/B en la misma sesión y anota el estado junto al número. Para decidir, el instrumento reproducible gana al "realista" ruidoso.
- **Un eval sellado envejece, y no solo el denominador.** Al crecer el corpus la misma configuración puntúa distinto sin haber empeorado; las etiquetas también caducan (la referencia sigue resolviendo pero el hecho que motivó la etiqueta ya no vive ahí) y acertarán por tema, no por su cita. Congela el corpus a un commit durante la comparación y re-verifica las filas derivadas antes de medir. Un instrumento validado una vez no queda validado.
- **Calcula qué resultado mínimo dispara cada regla antes de congelar un umbral.** Un umbral más fino que la granularidad del diseño decide por una sola unidad: con pocas tareas, una sola que cambie de resultado puede cruzarlo.

## Interpretar el resultado

- **Cero residuo.** Una hipótesis correcta explica todo el input; si deja residuo, no está cerrada. Y **acertar tapa el error**: si el resultado llega por un camino distinto al de la investigación, el acierto blinda un análisis falso.
- **Un null refuta la dosis probada, no el mecanismo.** Una palanca que salió null con una dosis razonada desde primeros principios se archivó como muerta; medida la relación real, la escala correcta era varios órdenes mayor y resultó ser la primera palanca positiva. Antes de enterrar un mecanismo, pregunta si la dosis salió de una medición o de un razonamiento.
- **Al corregir un instrumento, enumera qué evidencia dependía de él.** Corregir sin mirar atrás, o a medias (la opción nueva existe pero el default sigue apuntando a lo viejo), deja evidencia previa sin revisar. Decide caso por caso: re-medir, anotar o retirar; y verifica que cambió el default. Un sesgo presente en ambos brazos se cancela en la comparación pareada pero invalida las afirmaciones absolutas.

## Cuándo no aplica

- Exploración barata y reversible: no pre-registres ni hagas multi-run para decidir un experimento descartable; basta un positivo conocido.
- Si el coste de medir mejor supera el de equivocarse, decide y anota la incertidumbre. Un número no validado puede orientar, pero no debe publicarse ni usarse para enterrar una vía.
- Ver [[Doctrina de trabajo con agentes|core/doctrina]] para el marco general.
