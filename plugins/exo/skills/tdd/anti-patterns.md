# Escribir buenos tests

**Cuándo cargar:** al escribir o cambiar tests, añadir mocks, o si sientes la
tentación de añadir métodos test-only a código de producción. Destilado de
`test-driven-development/writing-good-tests.md` (superpowers 6.4.2, MIT ©
2025 Jesse Vincent), que sustituye al antiguo `testing-anti-patterns.md`
(6.1.1).

Un test existe para cazar una rotura concreta. Dos principios:

1. Todo test nombra el fallo que caza.
2. Todo test ejercita la cosa real.

TDD estricto da ambos casi gratis: un test visto fallar contra código real ya
demostró que puede fallar, y solo se gana un mock cuando la dependencia real
resulta lenta o externa.

## Principio 1: nombra el fallo

Antes de escribir el cuerpo: ¿qué cambio en producción debería hacer fallar
este test, y ese cambio es un bug o una decisión? Un test se gana su sitio si
caza una rama equivocada, un side effect ausente, un argumento erróneo, un
caso límite o un contrato roto.

**Expectativas derivadas de forma independiente.** Literales y fixtures
comprobados a mano; preferible table-driven con `want` literal. Una
expectativa calculada por el código bajo test (o sus helpers) pasa haga lo
que haga ese código:

```
❌ expected = build(q); assert build(q) == expected      # espejo, siempre true
✅ assert build({tag: "urgent"}) == 'tag:"urgent"'       # literal a mano
```

**Sin change-detectors.** Si solo una decisión intencional puede romper el
test (valor de una constante, redacción exacta de un mensaje, estructura
privada), salta en cada rediseño y duerme ante los bugs. Testea la conducta
que depende de la decisión: no `MAX_RETRIES == 5`, sino "una llamada fallida
se reintenta 5 veces y la sexta no ocurre".

**Conducta, no texto.** Afirmar que un script, skill o config contiene una
línea exacta (string-presence) solo prueba que la fuente es la fuente. Corre
el script con inputs controlados y afirma outputs, side effects o exit codes.
Un documento que instruye a agentes se prueba por la conducta del agente que
lo consume; la prosa para humanos no lleva test.

**Tu código, no el framework.** Testea el contrato de tus fronteras (la ruta
que registras, la query que emites, el payload que produces). La mecánica
upstream es test de sus mantenedores. Si el comportamiento upstream te
sorprendió, un characterization test estrecho que nombre la suposición.
Dentro de tu código igual: constructores, getters, constantes y forwarding
trivial solo ganan test si validan, normalizan, dan default, derivan,
imponen o causan side effects; si no, afirma el primer resultado visible al
consumidor que dependa de ellos.

**Gate** — antes del cuerpo del test:
- No puedes nombrar el cambio que lo haría fallar ⇒ rediseña sobre una conducta observable.
- "Cambió el texto fuente" ⇒ ejecuta el artefacto y afirma sus efectos.
- Solo fallan decisiones intencionales ⇒ es change-detector; testea la conducta que depende de la decisión.
- ¿La expectativa reusa la lógica o helpers del código? ⇒ literal o fixture a mano.

## Principio 2: ejercita lo real

**El mock no gana aserciones.** Afirmar sobre un elemento mockeado ("existe
el mock") pasa si el mock está y falla si no, sin decir nada del componente.
Afirma la conducta del componente real; si lo que miras es el mock, desmockea
o borra la aserción.

**Mockea al nivel correcto.** Antes de reemplazar un método, aprende todos
sus side effects; mockea la operación lenta o externa y deja real lo que el
test necesita. Ejemplo típico: mockear un método de alto nivel que escribe la
config que la detección de duplicados lee después ⇒ el test pasa o falla por
la razón equivocada. Mockea el arranque lento del servidor, no la escritura
de config. Si dudas, corre primero contra la implementación real y observa.

**Dobles específicos.** Si argumentos, número de llamadas u orden son parte
del contrato, aféralos: un fake que acepta cualquier cosa no verifica nada.
Cada rama (éxito, error, malformado) con su propio fixture o spy, para que la
rama equivocada no satisfaga la expectativa.

**Refleja los datos reales por completo.** Mockea la estructura completa como
existe en realidad, no solo los campos que lee tu test. Los mocks parciales
fallan en silencio cuando el código downstream lee un campo omitido.

**Producción lleva solo métodos de producción.** La limpieza que solo usan
los tests (un `destroy()` que ningún camino real llama) va en test utilities,
no en la clase. Contamina, es peligroso si se llama por accidente y viola
YAGNI. Pregunta: ¿solo lo llaman tests? ¿esta clase es dueña del ciclo de
vida del recurso? Si no, va a test-utils.

**Mejor real que mock complejo.** Si el setup del mock crece más que la
lógica del test, faltan métodos que tienen los componentes reales, o el test
se rompe al cambiar el mock ⇒ integration test con componentes reales.

**Gate** — antes de añadir un mock o helper de test:
- Lista los side effects del método real; deja reales los que el test usa y mockea el nivel lento/externo por debajo.
- Las respuestas mock reflejan la estructura real completa.
- ¿Método que solo llaman los tests? ⇒ test utilities.
- ¿Vas a afirmar sobre el propio mock? ⇒ desmockea o borra la aserción.

## Mutation check

Antes de terminar, muta mentalmente el código de producción; para cada
mutación realista, al menos un test debe fallar:

- constante o argumento equivocado;
- handler de rama equivocado;
- cambio de estado o side effect ausente;
- retorno vacío o por defecto;
- validación ausente para cero, vacío, nil, no autorizado o malformado.

Una mutación que nada caza marca conducta desprotegida, o un test tautológico.

## Warning signs

- Setup y aserción comparten el mismo objeto, garantizando igualdad.
- El test solo puede fallar por panic, crash o selector ausente.
- Falla en cada cambio intencional y nunca en una rotura accidental.
- Los valores esperados se esconden tras loops, builders o helpers.
- Hace grep del texto fuente, o afirma que un símbolo borrado sigue borrado.
- Seguiría importando aunque solo quedara el framework.
- Existe por coverage, sin comprobar ningún side effect ni resultado.
- Una aserción mira un test ID `*-mock`, o falla si quitas el mock.
- Un método se llama solo desde ficheros de test.
- El setup del mock es más de la mitad del test, o no sabes explicar por qué hace falta.
- Mockear "por si acaso".
