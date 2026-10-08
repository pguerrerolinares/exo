---
permalink: "{{KB_NAME}}/learnings/la-prosa-no-es-enforcement"
title: Una regla que se cita y se ignora es un comentario
tags: [enforcement, gates, hooks, agentes, doctrina]
tier: stable
semilla: true
---

# Una regla que se cita y se ignora es un comentario

Un contrato cuyo cumplimiento depende de que un modelo "haga lo correcto" no es un contrato. No es pesimismo sobre los modelos: es lo observado repetidamente en frentes distintos. Un agente puede conocer la regla, citarla y violarla en el mismo turno. Lo que cambia el comportamiento es el mecanismo (hook, test, gate, paso obligatorio, canal con autoridad), no la redacción. Cómo se comprueba el trabajo, en [[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]].

## Por qué la prosa no basta

- **Saber la regla no es cumplirla.** Un contrato escrito en prosa, en el sitio correcto, se incumple de forma sistemática; se sustituye por inyección mecánica del mismo contenido.
- **Una regla de parada no frena el grinding.** Los anuncios de parada se siguen de más continuaciones. Lo que rompe el bucle es delegar a un subagente fresco (sin sunk-cost) o un compromiso externo fijado antes de empezar. Ver [[En terreno desconocido, verificar el supuesto antes de seguir computando|learnings/recon-first]].
- **Los modelos saben que necesitan la herramienta y no la llaman.** El fallo es de ejecución, no de conocimiento; por eso la corrección tiene que ser mecánica.
- **En conversaciones multi-turn la fiabilidad cae.** Estudios externos miden caídas notables: asumen pronto y, si fallan un turno, no se recuperan. Los benchmarks de instruction-following sobreestiman la adherencia real.

## Convierte la lección en mecanismo

- **La lección se convierte en enmienda o no existe.** Si un aprendizaje no acaba en un hook, un test, un gate o un paso obligatorio del pipeline, no se ha aprendido: se ha anotado. Una lección que vive solo en la bitácora se pierde.
- **Mecaniza lo verificable; reserva la prosa para el juicio y mídela.** Si el cumplimiento se comprueba con un patrón (ruta mal formada, flag ausente, formato de contrato), va a hook o guard. Si exige criterio, va a prosa y se mide; un sensor en modo solo-registro silencia el aviso conservando el canal de medición.
- **Un warn-only llega tarde; actúa antes.** Un corrector que avisa tras ejecutar es estructuralmente tardío y con agentes efímeros no acumula nada. Funciona actuar antes: reescritura silenciosa del input, doctrina en el system prompt o un guard `PreToolUse`. Detalle del mecanismo (gate y ejecución en comandos separados) en [[El harness tiene hechos que muerden en producción: verificarlos contra la versión viva|learnings/hechos-del-harness-claude-code]].
- **Un sensor asciende a bloqueo por su tasa de falsos positivos, no por doctrina.** Warn-only es el default correcto para estrenar un sensor. Cuando su tasa de FP medida lo permite, se asciende a bloqueo (un exit code 2 de hook bloquea de forma determinista y devuelve el motivo), y de uno en uno.
- **Enrutar es parte del enforcement.** Documentado no es enrutado, y enrutado no es ejecutado. Una técnica que está escrita pero que ningún paso del flujo llega a consultar, no se aplica nunca: añade la ruta que lleva hasta ella.
- **Vigila el bucle, no solo la regla.** Un síntoma de que el enforcement murió es que el fichero donde vive la regla deja de cambiar mientras el trabajo sigue avanzando en otro sitio.

## La autoridad la pone el canal

- **La misma regla literal rinde distinto según cómo llega.** Inyectada como contexto adicional, el agente la trata como una sugerencia. En el system prompt, con framing de autoridad, la cumple más y la cita como obligación. Orden de autoridad: system prompt > contexto adicional > prosa en un fichero.
- **Las reglas mecánicas se cumplen por cualquier canal; las de principio o proceso mejoran con autoridad y aun así fallan más.** Por eso las segundas necesitan además medición.
- **Transporte mecánico.** Doctrina y memoria que deben llegar a un subagente las inyecta un hook (`SubagentStart`), no el modelo que escribe el brief. En los briefs, los punteros van como rutas de fichero.
- **Contract-first**: el agente ejecuta el contrato escrito, no el que pretendías. Ver [[El LLM es el operador de último recurso, y lo que no mide se aparca|learnings/construir-con-llms-y-descartar]].
- **Hipótesis preliminar: las reglas blandas decaen tras compactar el contexto y fijarlas (pinning) sería barato.** Un estudio apunta a que las reglas que viven en el historial se pierden más que las duras; reinyectarlas es poco costoso. No aplica a las del system prompt. Sin validar en este entorno: evalúa un hook `PreCompact` antes de construir nada.

## Vigencia y escritura en memoria

- **La vigencia de una nota se ata a la identidad de lo que describe (hash, ruta) y la comprueba un script, no la prosa.** Una nota ligada a ficheros se marca obsoleta cuando estos cambian; en la KB, `exo stale`. Una "revisa que siga vigente" escrita es un comentario.
- **La escritura en memoria pasa por un gate explícito.** Lo que entra a la memoria de un agente se filtra (p. ej. destilado con revisión humana), no se acumula por defecto: la memoria sin vigencia ni filtro devuelve lo antiguo como si fuera cierto.

## Diseño de gates

El diseño de gates y sensores (señales que no separan baseline de deuda nueva, gates que solo vigilan a quien se declaró, reglas cuya intersección no tiene salida) es el terreno de [[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]]. Aquí queda la regla de cabecera: antes de añadir una regla a un gate, calcula su intersección con las existentes y nombra el estado imposible si lo hay.

## Cuándo no aplica

- **Juicio sin clasificador determinista** (p. ej. cuándo delegar la ejecución): se queda en prosa, y se mide.
- **No bloquees por defecto.** Un sensor nuevo empieza en warn o log-only; sin tasa de FP medida, un bloqueo prematuro genera más fricción que protección.
- **Reglas que ya viven en el system prompt** no necesitan pinning para sobrevivir a la compactación.

## Relacionado

- [[Doctrina de trabajo con agentes|core/doctrina]]
- [[El fallo más caro es el que no avisa, y cada forma tiene su remedio|learnings/fallo-silencioso]]
