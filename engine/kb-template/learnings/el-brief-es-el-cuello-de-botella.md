---
permalink: "{{KB_NAME}}/learnings/el-brief-es-el-cuello-de-botella"
title: La claridad del encargo es el cuello de botella, no la capacidad del agente
tags: [agentes, delegacion, comunicacion]
tier: stable
semilla: true
---

# La claridad del encargo es el cuello de botella, no la capacidad del agente

Cuando un agente capaz produce un resultado que no era el que hacía falta, la
conclusión fácil es que el agente no dio la talla. Casi siempre la causa real
es otra: el encargo dejaba espacio para más de una interpretación razonable,
el agente eligió una de ellas con toda su capacidad, y el resultado se
desvió sin que nada en el proceso avisara — porque desde dentro de esa
interpretación, todo lo que hizo era coherente y correcto.

## Por qué pasa así de silenciosamente

Un agente sin contexto no sabe lo que no le han dicho. Ante ambigüedad, no
se detiene a preguntar salvo que se le indique explícitamente que puede
hacerlo: rellena el hueco con la lectura más plausible y sigue. El resultado
tiene toda la forma de un trabajo bien hecho — porque, dada la premisa que
asumió, lo es. El desajuste está en la premisa, no en la ejecución, y por
eso no se nota hasta que alguien compara el resultado con lo que en realidad
hacía falta.

## Qué hacer con esto

Subir la capacidad del agente no arregla un encargo ambiguo: un agente más
capaz interpretará la ambigüedad de forma más sofisticada, no la eliminará.
Lo que sí lo arregla es invertir en el encargo antes de delegar: qué se
quiere lograr y por qué, qué ya se probó o se descartó, qué decisiones puede
tomar el agente por su cuenta y cuáles no, y qué forma debe tener el
resultado. Un encargo que explica el propósito, no solo el paso a paso,
permite que el agente rellene los huecos inevitables en la dirección
correcta en vez de en una plausible cualquiera.

## Cuándo aplica y cuándo no

Aplica sobre todo al delegar en un agente sin memoria de la conversación
previa: parte de cero, y todo lo que no esté en el encargo, no lo tiene. No
sustituye la necesidad de que el propio encargo esté bien pensado — un brief
detallado pero mal razonado sigue produciendo un resultado mal razonado, solo
que con más precisión.
