---
permalink: "{{KB_NAME}}/learnings/construir-con-llms-y-descartar"
title: El LLM es el operador de último recurso, y lo que no mide se aparca
tags: [llm, pragmatismo, descarte, medicion]
tier: stable
semilla: true
---

# El LLM es el operador de último recurso, y lo que no mide se aparca

Un LLM en un pipeline no es el motor: es el operador de última instancia. Primero va el determinismo barato; el LLM entra solo donde el determinismo se rinde. Lo mismo vale para construir: el humano fija el contrato y las decisiones de alto impacto, el agente ejecuta, y toda complejidad añadida tiene que demostrar con una métrica que supera a la alternativa simple. Si no la supera, se aparca. El coste de la complejidad no es lineal: cada capa nueva añade modos de fallo más rápido de lo que añade valor.

## Dónde meter un LLM (y dónde no)

- **Filtro determinista primero.** Si una regla (regex, keyword, lookup) cubre ≥80% de los casos, impleméntala y manda al LLM solo el residuo ambiguo. Un pre-filtro de dos fases es el patrón de mayor ROI en pipelines de alto volumen: la mayoría de las entradas se resuelven a coste cero de API.
- **`null > inventado`.** El postprocesado rechaza explícitamente los campos inválidos, con validadores específicos del campo (un validador propio de cada campo) y no con un "no está vacío". Un null obliga a completar; un valor erróneo corrompe el dato sin avisar.
- **Contrato de salida estructurado.** Esquema primero, prompt derivado del esquema, validador defensivo después. La salida es fiable entre proveedores porque el contrato la fija, no el prompt.
- **Batch para verificar, individual como respaldo.** Agrupar N afirmaciones en una llamada con respuesta en array JSON abarata mucho la verificación; si el parseo falla, se reintenta de una en una.
- **Skip-by-existence desde el primer día.** Antes de relanzar una fase cara, comprueba si su salida ya existe y tiene contenido; un `--force` para rehacerla. Retrofitarlo cuesta más que diseñarlo.
- **Reduce la entrada antes de enviarla.** Elige la representación más barata que aún lleve señal suficiente y presupuesta tokens por nivel.
- **El contexto de dominio pesa más que la elección de modelo.** Contexto específico escrito a mano convierte salida genérica en experta, y no lo replica quien use el mismo modelo.
- **Cuándo no meter LLM**: si una regla cubre ≥80%, si el valor está en el resultado final y no en el procesado, si el coste de API rebasa el presupuesto antes de producción, o si ya hay un modelo grande y otro más duplica complejidad en vez de colapsarla.

## Quién decide y cómo se construye

- **Contract-first.** Antes de código, el spec o plan que fija arquitectura, alternativas descartadas y cierre de excepciones. El agente ejecuta el contrato escrito, no el que pretendías: contrato ambiguo produce código correcto que resuelve el problema equivocado. Calidad de contrato > calidad de prompt (ver [[La claridad del encargo es el cuello de botella, no la capacidad del agente|learnings/el-brief-es-el-cuello-de-botella]]).
- **El humano decide la arquitectura.** Descartar un enfoque, elegir el objetivo primario o revertir una versión compleja son decisiones humanas; al agente se le delega la implementación del pivote, no el pivote.
- **Verify-before-build.** Antes de invertir semanas, que un agente distinto del que diseñó mida empíricamente las afirmaciones verificables del diseño. Una tarde de auditoría puede refutar el ingrediente estrella.
- **El reviewer detecta, la medición decide.** Un hallazgo de review es una hipótesis a medir, no una orden: un fix recomendado puede acertar el mecanismo y errar el impacto.
- **Mide antes de confiar en la complejidad añadida.** Sin ground truth las regresiones son invisibles. "Más elegante" o "más correcto en teoría" no cuenta sin métrica. Valida la métrica desde la fuente antes de medir con ella: ver [[Un negativo no vale si el instrumento no está validado|learnings/instrumento-validado-antes-de-medir]].

## Cuándo descartar

- **Regresión de complejidad.** Si la versión simple tenía una métrica medible, la compleja tiene que superarla en esa misma métrica; si empeora, revierte sin dudar. Si una simple gana de forma consistente, sospecha primero del entorno de evaluación: ver [[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]].
- **Ratio coste/señal.** Cuantifícalo antes de comprometerte: la alternativa simple que da el 80% del valor en una fracción del tiempo gana. El momento de abandonar no es cuando algo falla, sino cuando señal/coste cae por debajo de la alternativa simple.
- **Dependencia rota, no diseño roto.** Separa ambos: el diseño puede reimplementarse sin la dependencia (ver [[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]]).
- **80/20.** Entrega el mínimo que aporta valor, valídalo, y solo entonces diseña la fase 2: la complejidad de la fase 2 se financia con la validación de la fase 1.
- **Pre-registra los kill-criteria** antes de medir, y no relances la batería cuando disparan. Un ancla que solo se respeta cuando el resultado queda lejos no es un ancla.
- **Aparca, no borres.** Prefijo `_` o carpeta `_deprecated/` para lo que no está listo: conserva la opción y el porqué queda en el historial de git.
- **Reutiliza antes de construir.** Comprueba si una base existente ya resuelve el problema; evalúa cualquier dependencia exótica contra la alternativa estándar (SQLite, filtro por palabras clave) antes de adoptarla.

## Cuándo no aplica

- Si el problema no tiene regla determinista razonable (entrada abierta o ambigua), el LLM puede ser el primer paso; el principio sigue siendo acotarle la entrada y validarle la salida.
- Descartar por métrica presupone una métrica válida y un entorno de evaluación fiel al de despliegue; sin eso, mide primero el instrumento, no el candidato.
- Aparcar con `_deprecated/` no sustituye limpiar: lo que lleva tiempo sin motivo para volver, se retira.

Relacionado: [[Doctrina de trabajo con agentes|core/doctrina]].
