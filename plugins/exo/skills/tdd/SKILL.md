---
name: tdd
description: Usa al implementar cualquier feature o bugfix, antes de escribir código de producción. Test primero, verlo fallar por la razón esperada, código mínimo, verde, refactor.
---

# tdd

Destilado de `test-driven-development/SKILL.md` (superpowers 6.4.2, MIT © 2025 Jesse Vincent).

Escribe el test primero. Velo fallar. Código mínimo para pasar. Si no viste
el test fallar, no sabes si testea lo correcto — ese es el principio.

## Regla central

No hay **conducta** de producción sin un test que falló primero (lo trivial de «Qué no testear» no es conducta y queda fuera sin pedir permiso). Código escrito
antes del test se BORRA y se reimplementa desde los tests — no se guarda
"como referencia", no se "adapta" mientras escribes el test. Borrar
significa borrar.

Excepciones legítimas SOLO con permiso explícito del humano: prototipos
desechables, código generado.

## Ciclo Red-Green-Refactor

- **RED**: escribe un test mínimo que muestre el comportamiento esperado —
  un comportamiento por test (si el nombre necesita "and", pártelo), nombre
  que describe la conducta, código real (mocks solo si es inevitable).
- **Verify RED (obligatorio)**: corre el test y confirma que falla — no que
  erra — por la razón esperada (feature ausente, no un typo). Si pasa de
  primeras, el test está mal.
- **GREEN**: el código más simple que pasa el test. Sin features extra, sin
  refactor ajeno, sin "mejoras" más allá del test — YAGNI.
- **Verify GREEN (obligatorio)**: el test pasa, el resto sigue verde, output
  pristine (sin errores ni warnings). Si falla, se arregla el código, no el
  test. **Verde = la suite del proyecto entera** (el comando completo que
  define el proyecto), no solo el fichero del test que acabas de escribir.
  Un test nuevo verde con otra parte de la suite roja no es verde.
  Aunque tu tarea nombre un solo fichero de test, ese scope acota el
  entregable, no tu verificación. Todo fallo que muestre esa corrida, aunque
  no lo causaras, va en tu report por su nombre: un test rojo que viste pasar
  sin mencionarlo falsea el report por omisión.
- **REFACTOR**: solo en verde — quita duplicación, mejora nombres, extrae
  helpers. Sin añadir comportamiento.

## Qué no testear

One-liners, glue, constantes y prosa para humanos no llevan test: solo
ganan uno si validan, normalizan, derivan, imponen o causan side effects.
Los documentos que instruyen a agentes (skills, prompts) se prueban por la
conducta del agente que los consume, nunca con grep del texto. Los tests
de la conducta viajan en el mismo commit que la implementación.

## Bug fix

Primero un failing test que reproduce el bug, luego el ciclo completo. Nunca
arregles un bug sin test.

## Antes de declarar completo

Checklist: un test por cada conducta nueva, cada test visto fallar por la razón
esperada, todo verde, output pristine, edge cases cubiertos. Si no puedes
marcar todo, no fue TDD — empieza de nuevo.

## Cuando te atascas

No sabes testear ⇒ escribe la API deseada o pregunta. Test complicado ⇒
diseño complicado, simplifica la interfaz. Todo requiere mock ⇒
acoplamiento, inyecta dependencias.

## Escribir buenos tests

Antes de escribir un test, añadir mocks o helpers de test, lee
`anti-patterns.md`: cada test nombra el fallo que caza.
