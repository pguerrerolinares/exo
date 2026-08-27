---
permalink: "{{KB_NAME}}/core/doctrina"
title: Doctrina de trabajo con agentes
tags: [doctrina, agentes, core]
tier: stable
semilla: true
---

# Doctrina de trabajo con agentes

Un agente que escribe código o toca un sistema real necesita más que
capacidad: necesita reglas de cuándo parar, cuándo delegar y cuándo exigirse
evidencia antes de decir "hecho". Esta nota reúne esas reglas. Es la puerta:
cada principio con desarrollo propio vive en `learnings/` y aquí solo se
enuncia y se enlaza.

## Delegar en vez de acumular

Cuando el trabajo tiene piezas independientes, la tentación es hacerlas todas
en el mismo hilo de contexto. Es un error de escala: cada lectura, cada
intento fallido y cada exploración que no llevó a nada se queda pegada al
contexto del que coordina, y ese contexto es el recurso más caro y más
limitado que hay. La alternativa es delegar la implementación y la
investigación voluminosa a un agente fresco por tarea, y quedarse solo con la
conclusión. Desarrollo: [[Un orquestador delega la lectura pesada y se queda solo con la conclusión|learnings/orquestador-limpio]].

## La evidencia va antes que la afirmación

"Ya funciona", "ya pasa el test", "ya está arreglado" no son hechos hasta que
alguien ha corrido el comando y ha visto el resultado. Un agente que reporta
sin haber verificado no está mintiendo necesariamente — está confundiendo la
intención con el resultado. La disciplina correcta es siempre la misma:
corre el comando, enseña el output real, y solo entonces afirma.

## El fallo que no avisa es el que importa

No todos los fallos son iguales de peligrosos. El que sale con un mensaje de
error rojo se detecta solo. El que sale con forma válida, con exit 0, con un
check que nunca podría haber fallado — ese es el que se cuela. Vigilar por
ese tipo de fallo, no solo por el fallo ruidoso, es la parte del trabajo que
más se olvida. Desarrollo: [[El fallo más caro es el que no avisa|learnings/fallo-silencioso]].

## Terreno desconocido: verificar antes de seguir computando

Cuando el mismo error se repite, o el terreno es nuevo, la reacción natural
es seguir intentando variaciones. Con frecuencia es la reacción equivocada:
el intento número cuatro sobre un supuesto falso falla exactamente igual que
los tres anteriores. Antes de seguir computando, conviene parar y buscar —
verificar el supuesto que se está dando por hecho. Desarrollo:
[[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]].

## El cuello de botella suele estar en el encargo, no en el agente

Un agente capaz, con un encargo ambiguo, no falla ruidosamente: interpreta la
ambigüedad de la forma más razonable que encuentra y sigue adelante, y el
resultado se desvía de lo que hacía falta sin que nadie se entere hasta
después. Subir la capacidad del agente no arregla esto. Lo que lo arregla es
un encargo más claro. Desarrollo:
[[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]].

## Cambios pequeños, en el estilo de alrededor

Un cambio que toca solo lo necesario, en el idioma y las convenciones ya
presentes en el código que lo rodea, es más fácil de revisar, más fácil de
deshacer si hace falta, y deja menos superficie donde algo pueda salir mal
sin que se note. Ampliar el alcance de un cambio "ya que se está aquí" es
casi siempre un coste que nadie pidió pagar.

## La revisión escala con el riesgo, no es uniforme

No todo cambio necesita el mismo nivel de escrutinio. Un cambio trivial
(renombrar, una constante, un getter) no justifica un ciclo de revisión
completo. Un cambio con lógica real — una decisión que puede estar mal, un
efecto que puede no dispararse, un caso límite que puede quedar sin cubrir —
sí lo justifica. Calibrar el esfuerzo de revisión al riesgo real del cambio,
en vez de aplicar siempre el máximo o siempre el mínimo, es lo que hace que
la disciplina sea sostenible en vez de un lastre.
