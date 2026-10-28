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

**Dispatch:** subagente genérico, solo lectura, `model` explícito (sonnet de
suelo; top si la decisión es cara). Lecturas obligatorias como rutas de
fichero: al consultor no le llega perfil de inyección.

## Consulta

```
Subagent (general-purpose):
  description: "Consultor: [TEMA]"
  model: [MODEL]
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
    Escribe la nota completa en [NOTE_FILE], como mucho [N] caracteres, en
    este orden:
    1. Recomendación (arriba, una frase) y por qué.
    2. Condición que la invalidaría, medible (qué valor, de qué instrumento).
    3. Qué medir para confirmarla, con receta cerrada.
    4. Análisis por opción. Etiqueta cada afirmación: [E] evidencia (di si
       leíste la fuente entera o solo el abstract), [O] opinión, [sin dato].
    5. Desacuerdos con este brief, si los hay.

    Tu mensaje final: menos de 15 líneas — recomendación, condición de
    invalidación, premisas que no pudiste verificar y la ruta de la nota.
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

- Las cifras críticas del consultor (contrastes, tamaños, umbrales,
  versiones) son estado declarado: recalcúlalas con una herramienta antes de
  que entren en la spec.
- Una premisa `[sin dato]` de la que depende la recomendación se verifica con
  un comando antes de implementar (tú o un agente barato).
- La condición de invalidación va a la spec tal cual: es lo que se
  pre-registra, no un estadístico elegido en frío.

**Placeholders:** `[TEMA]`, `[DOMINIO]`, `[MODEL]` — REQUIRED. `[NOTE_FILE]` —
REQUIRED: fichero en el scratchpad o junto a la spec. `[N]` — tope de
caracteres (por defecto 6000).
