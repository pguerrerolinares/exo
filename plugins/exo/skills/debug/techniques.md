# Técnicas de soporte

**Cuándo cargar:** durante la Fase 1 (root cause) o Fase 4 (fix) de
`SKILL.md`, cuando el bug está profundo en la pila de llamadas, cuando ya
encontraste la causa y quieres cerrar la puerta a que vuelva, o cuando un
test usa un `sleep`/timeout arbitrario. Destilado de
`systematic-debugging/root-cause-tracing.md`,
`systematic-debugging/defense-in-depth.md` y
`systematic-debugging/condition-based-waiting.md` (superpowers 6.1.1, MIT ©
2025 Jesse Vincent).

## Root-cause tracing

Los bugs suelen manifestarse lejos de su origen (un fichero creado en el
sitio equivocado, un git init en el directorio equivocado): el instinto es
arreglar donde aparece el error, pero eso trata el síntoma.

Traza hacia atrás por la cadena de llamadas hasta el trigger original:
observa el síntoma, identifica la causa inmediata (¿qué código lo
provoca?), pregunta qué llamó a eso, y sigue subiendo — ¿qué valor se
pasó?, ¿de dónde vino ese valor? — hasta llegar a la fuente real. Si no
puedes trazar a mano, instrumenta con un stack trace antes de la operación
peligrosa (en tests, `console.error` — el logger puede estar suprimido) y
corre para capturarlo. Arregla en la fuente, nunca solo donde aparece el
error.

## Defense-in-depth

Una sola validación se siente suficiente, pero un único check lo puede
saltar un code path distinto, un refactor, o un mock. Valida en CADA capa
por la que pasan los datos, para que el bug sea estructuralmente
imposible:

1. **Entry point** — rechaza input obviamente inválido en el límite de la
   API.
2. **Lógica de negocio** — asegura que el dato tiene sentido para esta
   operación concreta.
3. **Guards de entorno** — impide operaciones peligrosas en contextos
   específicos (p.ej. rechazar una escritura destructiva fuera de un
   directorio temporal durante tests).
4. **Instrumentación de debug** — captura contexto (directorio, cwd,
   entorno, stack) para forense futuro.

Aplica el patrón: traza el data flow (¿dónde se origina el valor malo?,
¿dónde se usa?), mapea todos los checkpoints por los que pasa, añade
validación en cada capa, y testea cada una intentando saltarte la
anterior. Las capas se complementan — code paths distintos saltan la
validación de entrada, los mocks saltan la de lógica de negocio, casos de
borde de plataforma necesitan el guard de entorno. No pares en un solo
punto de validación.

## Condition-based waiting

Los tests flaky suelen adivinar tiempos con delays arbitrarios
(`setTimeout`, `sleep`) — eso crea condiciones de carrera: el test pasa en
una máquina rápida y falla bajo carga o en CI.

Espera la condición real que te importa, no una suposición sobre cuánto
tarda: en vez de `await sleep(50); expect(getResult()).toBeDefined()`,
usa un polling genérico (`waitFor(() => condición, descripción,
timeoutMs)`) que compruebe la condición cada pocos milisegundos con un
timeout explícito y un mensaje de error claro. Evita dos errores comunes:
pollear demasiado rápido (desperdicia CPU — cada 10ms es razonable) y no
poner timeout (el loop cuelga para siempre si la condición nunca se
cumple).

Un timeout arbitrario SÍ es correcto cuando: (1) primero esperaste la
condición que dispara el comportamiento, (2) el tiempo fijo que sigue está
basado en un timing conocido (no adivinado), y (3) comentas por qué. Fuera
de ese caso, si estás testeando el propio comportamiento de timing
(debounce, throttle), el timeout no es el anti-patrón — documenta por qué
hace falta.
