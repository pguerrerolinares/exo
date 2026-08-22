# Brief — Consultor Fable independiente: ¿existe una solución mejor?

## Por qué te llaman a ti

Cuatro auditores y un coordinador han pasado un día sobre este problema. Han
producido una propuesta (v2) ratificada por unanimidad. **Paul sigue sin estar
convencido**, y ha pedido explícitamente un consultor *independiente* que
investigue cómo se resuelve esto en la comunidad y en la literatura.

Tu independencia es el encargo, no una formalidad: **tienes permiso explícito
para concluir que la v2 está mal enfocada, que el problema está mal planteado, o
que la respuesta correcta es no hacer nada.** Nadie te va a pedir que ratifiques
lo ya firmado, y un informe que valida el consenso por inercia no vale nada aquí.

Pero tampoco se te pide que encuentres algo mejor a toda costa: si tras
investigar concluyes que la v2 es lo mejor disponible y que la insatisfacción de
Paul no tiene remedio técnico, **dilo y ahórranos el trabajo**. Es una respuesta
legítima y probablemente la más valiosa si es la verdadera.

## El problema, en su forma más despojada

Una persona genera doctrina y aprendizajes a un ritmo dado. Esa producción es
**exógena**: no se puede reducir sin dejar de pensar. Debe vivir en algún sitio
donde un agente LLM pueda encontrarla y usarla. El sitio actual —notas markdown
canónicas con techo de bytes— se llena, y llenarse duele siempre en el mismo
momento: al cerrar la sesión, cuando ya no hay tiempo ni ganas de decidir qué
sobra.

La pregunta de Paul, literal, dos veces: *"llevamos con ese problema casi desde
el inicio y se creó consolida, pero me da la sensación que es un apaño más que
una solución"* y *"consolida ya lo hemos realizado varias veces y siempre
muerde"*.

## Lo que la auditoría ya estableció (verifícalo, no lo asumas)

Todo en `docs/superpowers/consultas/2026-08-22-presupuestos/`. Léelo entero:
`sintesis.md` (la v2 y los números canónicos), los cuatro informes, `contexto.md`
(ojo: contiene una premisa falsa, marcada) y `addendum-ronda-2.md` (contiene dos
cifras retractadas, marcadas).

Los hechos que sobrevivieron a la confrontación, resumidos:

- **El ciclo converge**: medido cima-contra-cima de ciclos consecutivos, el canon
  va de 453.261 a 310.994 B (−31,4%). Lo que parece crecimiento es la mitad
  ascendente de un ciclo que acaba más abajo. Cautela declarada: n=2.
- **El muro contiene**, pero menos de lo que se creyó: factor ~9-21× (tres
  rebajas sucesivas por errores de anclaje).
- **La reposición es inmediata**: una poda de 21,7 KB se repuso en +2.709 B la
  misma tarde. Cuatro podas, cuatro reposiciones.
- **El crecimiento está repartido** (21 de 27 notas con techo crecen; top-3 = 53%),
  no concentrado como se llegó a afirmar.
- **Dos notas son "imanes-área"**: `doctrina-agentes` (80% de su crecimiento en
  headings sin relación entre sí) y `desarrollo-agentico` (65% afines).
- **El sistema real** no es el declarado: 4 de 5 notas core viven de un techo
  propio por encima del nominal.
- **La KB entera creció +163%** en 7 semanas (570.910 → 1.498.996 B), casi la
  mitad de ella archivo.

## Lo que ya se investigó, para que no lo repitas

Un consultor previo cubrió, con fuentes verificadas: MemGPT (2310.08560), Letta
memory blocks, Generative Agents (2304.03442), sleep-time compute (2504.13171),
Mem0 (2504.19413), Zep/Graphiti (2501.13956), LoCoMo (2402.17753), memoria
episódica (2502.06975), Luhmann vía Schmidt (2018), evergreen notes de
Matuschak, LSM/RocksDB, Elasticsearch ILM. Su conclusión: la arquitectura
(core acotado + retrieval + consolidación offline) es estado del arte; lo que no
tiene análogo es bloquear al escritor en el write path.

**No repitas ese trabajo. Constrúyelo o refútalo.**

## Dónde creo que hay hueco (sugerencias, no límites)

1. **La unidad.** Todo el sistema asume que la unidad de conocimiento es *el
   fichero*, y el techo se aplica al fichero. ¿Y si la unidad correcta fuese
   otra —el bloque, la afirmación, el par pregunta-respuesta— y el problema del
   tamaño se disolviera al cambiarla? Mira qué hacen los sistemas que no tienen
   este problema: wikis grandes, bases de datos de conocimiento, sistemas de
   documentación que llevan décadas creciendo sin que nadie los pode a mano.
2. **La pregunta radical**: ¿por qué hay canon? Si hay retrieval híbrido sobre
   todo el corpus, ¿qué función cumple exactamente una capa "destilada" separada
   del log? La auditoría la asumió sin cuestionarla. Si la respuesta es "ninguna
   que el retrieval no cubra", eso disuelve el problema entero en vez de
   gestionarlo. Y si tiene función, nómbrala con precisión: eso también es útil.
3. **Herramientas reales de PKM** (Obsidian, Logseq, Roam, Dendron, org-roam) y
   sus comunidades: llevan años con usuarios cuyas notas crecen. ¿Qué prácticas
   han emergido de verdad —no las de los blogs de productividad, las que la
   gente sostiene durante años—? ¿Alguien pone límites de tamaño? Si nadie lo
   hace, ¿qué hacen en su lugar y por qué funciona?
4. **Papers posteriores o no cubiertos**: memoria jerárquica, compresión de
   contexto, agentes que gestionan su propia memoria, "context engineering". Y
   en particular: ¿alguien ha estudiado el coste de la *fricción editorial*
   sobre el humano, que es lo que aquí duele?
5. **La hipótesis incómoda**: que Paul esté insatisfecho no porque el sistema
   funcione mal, sino porque **le pide decisiones editoriales que no quiere
   tomar**, y ninguna arquitectura elimina eso — solo lo mueve de sitio. Si es
   así, la conclusión honesta es "esto no es un problema de ingeniería" y
   conviene decirlo claro.

## Restricciones

- KB personal de un solo autor. **Régimen §0: cerrar cosas, no construir
  maquinaria.** Una propuesta que exija un subsistema nuevo tiene una barra muy
  alta, y debes decir explícitamente qué se retira a cambio.
- El mantenimiento cuesta **tiempo de Paul**. Una solución que pida más
  disciplina humana no es una solución.
- Sin métricas nuevas ni ventanas de medición.
- Existe un **trinquete** firmado: los techos por nota solo pueden bajar. Si tu
  propuesta lo toca, justifícalo muy bien.
- **Fuentes reales o ninguna.** Si no estás seguro de que un paper existe, no lo
  cites: aquí una cita falsa se detecta y quema el informe entero. Distingue
  siempre evidencia de opinión tuya.

## Material

- Expediente: `docs/superpowers/consultas/2026-08-22-presupuestos/`
- KB: `~/Documentos/proyectos/kb-demo` (repo git con historia completa)
- kbx (Go): `~/Documentos/proyectos/kbx` — `internal/budget`, `internal/rotate`
- exo: `~/Documentos/proyectos/exo` — engine Rust, plugins, specs
- Índice: `~/.exo/index.db`. **Cópialo si vas a escribir.** `sqlite3` CLI no está
  instalado; usa `python3` con el módulo `sqlite3`.
- **Solo lectura**: no toques la KB, ni los repos, ni hagas commits.

## Entrega

Informe en `docs/superpowers/consultas/2026-08-22-presupuestos/consultor-independiente.md`.

Respuesta final: ~35 líneas. Tu veredicto sobre la v2 (mejor disponible / mal
enfocada / innecesaria), lo que aportes de fuera con su fuente, y —si la hay— la
solución que nadie ha considerado. Si tu conclusión es que no hay nada mejor,
esa es tu entrega y no hace falta rellenarla.
