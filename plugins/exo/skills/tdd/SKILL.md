---
name: tdd
description: Usa al implementar cualquier feature o bugfix, antes de escribir código de producción. Test primero, verlo fallar por la razón esperada, código mínimo, verde, refactor.
---

# tdd

Escribe el test primero. Velo fallar. Código mínimo para pasar. Si no viste
el test fallar, no sabes si testea lo correcto — ese es el principio.

## Regla central

No hay código de producción sin un test que falló primero. Código escrito
antes del test se BORRA y se reimplementa desde los tests — no se guarda
"como referencia", no se "adapta" mientras escribes el test. Borrar
significa borrar.

Excepciones legítimas SOLO con permiso explícito del humano: prototipos
desechables, código generado, ficheros de configuración.

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
  test.
- **REFACTOR**: solo en verde — quita duplicación, mejora nombres, extrae
  helpers. Sin añadir comportamiento.

## Bug fix

Primero un failing test que reproduce el bug, luego el ciclo completo. Nunca
arregles un bug sin test.

## Antes de declarar completo

Checklist: un test por función nueva, cada test visto fallar por la razón
esperada, todo verde, output pristine, edge cases cubiertos. Si no puedes
marcar todo, no fue TDD — empieza de nuevo.

## Cuando te atascas

No sabes testear ⇒ escribe la API deseada o pregunta. Test complicado ⇒
diseño complicado, simplifica la interfaz. Todo requiere mock ⇒
acoplamiento, inyecta dependencias.

## Anti-patrones de testing

Antes de añadir mocks o utilidades de test, lee `anti-patterns.md`: nunca
testear comportamiento de mocks, nunca añadir métodos test-only a clases de
producción, nunca mockear sin entender la dependencia.
