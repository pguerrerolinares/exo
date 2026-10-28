# Brief de consultor de campo

**Cuándo usar:** antes de presentar al humano los Enfoques de una decisión de diseño
(siempre; el mensaje lleva el veredicto del consultor y su condición de
invalidación), y
antes de pasar de spec a plan cuando la spec descansa en hechos comprobables
(defaults de librería, cifras, límites de plataforma): review adversarial con
ejecutables. También al reconsultar con mediciones.

Un consultor no es el advisor: trabaja con datos que no están en tu
transcript, así que rinde lo que rinde su brief. Los fallos que esta plantilla
evita, todos reales: una recomendación implementada cuya premisa se
comprobaba en un minuto; cifras de contraste erróneas que llegaron al diseño;
una propuesta imposible en el plan gratuito de la forja; el consultor
alineándose con la propuesta que le pasó el orquestador.

**Dispatch:** `subagent_type: exo:consultor` (solo lectura; trae la disciplina
y el formato de retorno, el brief trae la decisión y los datos). `model`
explícito solo para subir del sonnet por defecto cuando la decisión es cara.
Le llega el perfil `reducido` (reglas duras y cores); aun así, las lecturas
obligatorias van como rutas de fichero.

## Consulta

```
Agent (subagent_type: exo:consultor):
  description: "Consultor: [TEMA]"
  model: [MODEL]   # omítelo para el sonnet por defecto
  prompt: |
    Eres consultor de [DOMINIO]. Tu nota decide algo real: discrepa si los
    datos lo piden, también de este brief. Alinearte con lo que intuyes que
    quiero oír es el fallo que más cuesta.

    ## Decisión
    [UNA LÍNEA: qué hay que decidir]

    ## Ya cerrado (no lo reabras)
    [decisiones tomadas y por qué, una línea cada una]

    ## Datos
    [cada dato con ancla: file:line, valor exacto (hex, ms, px), versión de
    librería/SO/dispositivo]

    ## Hechos de los que depende tu diagnóstico: compruébalos tú
    [premisas que, si son falsas, cambian la respuesta — p. ej. «el default
    de X es Y». Verifícalas con fuente, bytecode, comando o cálculo; si no
    puedes, márcalas [sin dato] y di cómo cambiaría tu recomendación]

    ## Restricciones del entorno
    [plan de la forja (gratuito, privado/público), dispositivos e
    instrumentos disponibles, presupuesto, lo ya medido y con qué]

    ## Opciones
    [≥2 opciones mínimas]. Añade otras si las ves mejores.

    ## Lecturas obligatorias
    [rutas de fichero]

    ## Entrega
    Tu mensaje final es la nota completa, como mucho [N] caracteres, en el
    formato de tu perfil: recomendación, invalidación medible, qué medir,
    opciones con [E]/[O]/[sin dato], desacuerdos con este brief. No escribes
    ficheros: la nota la guarda quien te despacha.
```

## Reconsulta con datos

Tras medir, vuelve con **los datos crudos y la regla de decisión, nunca tu
propuesta**: si se la pasas y se suma, no distingues convicción de deferencia.
Mismo template, con estos cambios:

- `## Datos` lleva las mediciones crudas (n, receta, instrumento), no un
  resumen tuyo.
- `## Opciones` lleva todas, sin decir cuál prefieres, y pide para la que los
  datos parecen favorecer «argumenta por qué X sería peor».
- Segunda opinión fresca solo si la decisión es cara.

## Al recibir la nota

- Guarda la nota tú en [NOTE_FILE]; el consultor es de solo lectura.
- Las cifras críticas del consultor (contrastes, tamaños, umbrales,
  versiones) son estado declarado: recalcúlalas con una herramienta antes de
  que entren en la spec.
- Una premisa `[sin dato]` de la que depende la recomendación se verifica con
  un comando antes de implementar (tú o un agente barato).
- La condición de invalidación va a la spec tal cual: es lo que se
  pre-registra, no un estadístico elegido en frío.

**Placeholders:** `[TEMA]`, `[DOMINIO]` — REQUIRED. `[MODEL]` — opcional:
quita la línea `model:` para el sonnet por defecto. `[NOTE_FILE]` — REQUIRED,
solo para ti: dónde guardas la nota (scratchpad o junto a la spec). `[N]` —
tope de caracteres (por defecto 6000).
