# Anti-patrones de testing

**Cuándo cargar:** al escribir o cambiar tests, añadir mocks, o si sientes la
tentación de añadir métodos test-only a código de producción. Destilado de
`test-driven-development/testing-anti-patterns.md` (superpowers 6.1.1, MIT ©
2025 Jesse Vincent).

Core principle: testea lo que el código hace, no lo que hacen los mocks. Los
mocks son una herramienta para aislar, no la cosa que se testea.

## 1. Nunca testear comportamiento de mocks

Afirmar sobre un elemento mockeado ("existe el mock") no prueba nada del
componente real — el test pasa si el mock está presente y falla si no, sin
decir nada de la conducta real.

**Gate**: antes de afirmar sobre cualquier elemento mockeado, pregúntate "¿
estoy testeando el componente o la existencia del mock?". Si es lo segundo,
para — borra la aserción o desmockea el componente y testea la conducta real.

## 2. Nunca añadir métodos test-only a clases de producción

Un método que solo usan los tests (p.ej. un `destroy()` que ningún camino de
producción llama) contamina la clase de producción, es peligroso si se llama
por accidente, y viola YAGNI. La limpieza de test va en utilidades de test,
no en la clase.

**Gate**: antes de añadir un método a una clase de producción, pregúntate si
solo lo usan los tests; si es así, no lo añadas — va en test-utils.

## 3. Nunca mockear sin entender la dependencia

Mockear "por si acaso" puede romper efectos secundarios de los que el propio
test depende (p.ej. mockear algo que escribe la config que el test necesita
leer después). El test pasa o falla por la razón equivocada.

**Gate**: antes de mockear cualquier método, pregúntate qué efectos
secundarios tiene el método real y si el test depende de alguno. Si el test
depende de un efecto secundario, mockea en un nivel más bajo (la operación
lenta/externa real), no el método de alto nivel del que depende el test. Si
no estás seguro de qué necesita el test, córrelo primero con la
implementación real y observa.
