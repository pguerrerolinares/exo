# metrics — criterio 2 del gate (semántica load-bearing), corregido tras fix de medición

Control FTS real: `results/textfts.jsonl`, solo filas `search_type=text`
(56 queries, 0 filas error), capturado con
`default_search_type=text` forzado en config (index post-purge, modelo minilm
sin tocar — el FTS no depende del modelo de embeddings).

Definición (gate.md criterio 2): ≥3 queries etiquetadas con HIT en vector u
hybrid y MISS en text (hit@5) ⇒ semántica LOAD-BEARING ⇒ decide Rust; si no ⇒ Go.

## jina-es: n = 26 / 55 labeled

Umbral gate (≥3): CUMPLE — semántica LOAD-BEARING

- Frente 9 lighthouses Fase 4 divergencia core split thin-core
- G1 bucket inferencia tipos cge 55,7
- KC7 Desbloqueo Prisma
- SDD spec-driven development repo nuevo scaffold React
- agent-solve-it copiloto Solve It recon
- ai-news plataforma noticias
- al diseñar componentes visuales reutilizables, cómo decidir qué parte debe funcionar en cualquier framework y qué parte puede acoplarse al host
- aquel torneo de programación de juegos donde mi jugador que aprendía solo durante la partida ganaba menos que la versión de reglas sencillas y hubo que dar marcha atrás
- basic-memory limitaciones dolores contrato memoria v2
- blog notas publicar contenido web pguerrero divulgación posts
- cambio tres líneas en una fuente y se me reprocesa la base de conocimiento entera; quiero que solo se recalcule lo afectado por el cambio
- cge bitácora
- cge motor code-graph bitácora backlog frentes
- consolida bug kbx detectado
- esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo
- extractor de rutas mock.patch 0-FP gate P0
- fabrica campaign harness config gate merge asíncrono roadmap
- fabrica gate merge kill-criteria worktree main-guard ventanas autorizadas
- fabrica roadmap campana lighthouses diversidad bots Fase 3
- fábrica campañas agent-develop harness A-thin
- la herramienta que revisa si una web es usable por personas con discapacidad y evita repetir comprobaciones en páginas que no han cambiado
- lighthouses contest bot Horus MadeInHeaven Pegasus e33
- pguerrero-music
- reflex cristalización efímero durable prior-art
- solve-it recon reto cripto concurso autosolver
- ¿en qué punto de un flujo de procesamiento compensa meter inteligencia artificial generativa y dónde es mejor quedarse con reglas fijas baratas?

## minilm: n = 26 / 55 labeled

Umbral gate (≥3): CUMPLE — semántica LOAD-BEARING

- Backlog frentes abiertos
- Frente 9 lighthouses Fase 4 divergencia core split thin-core
- G1 bucket inferencia tipos cge 55,7
- KC7 Desbloqueo Prisma
- SDD spec-driven development repo nuevo scaffold React
- agent-solve-it copiloto Solve It recon
- ai-news plataforma noticias
- al diseñar componentes visuales reutilizables, cómo decidir qué parte debe funcionar en cualquier framework y qué parte puede acoplarse al host
- aquel torneo de programación de juegos donde mi jugador que aprendía solo durante la partida ganaba menos que la versión de reglas sencillas y hubo que dar marcha atrás
- blog notas publicar contenido web pguerrero divulgación posts
- cambio tres líneas en una fuente y se me reprocesa la base de conocimiento entera; quiero que solo se recalcule lo afectado por el cambio
- consolida bug kbx detectado
- criterios para saber cuándo tirar a la basura una solución complicada porque la versión simple rinde igual o mejor con menos líos
- esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo
- extractor de rutas mock.patch 0-FP gate P0
- fabrica campaign harness config gate merge asíncrono roadmap
- fabrica gate merge kill-criteria worktree main-guard ventanas autorizadas
- fabrica roadmap campana lighthouses diversidad bots Fase 3
- fábrica campañas agent-develop harness A-thin
- la herramienta que revisa si una web es usable por personas con discapacidad y evita repetir comprobaciones en páginas que no han cambiado
- lighthouses contest bot Horus MadeInHeaven Pegasus e33
- pguerrero-music
- reflex cristalización efímero durable prior-art
- solve-it recon reto cripto concurso autosolver
- un generador de informes de investigación que además te marca qué afirmaciones están contrastadas con fuentes y de cuáles no fiarte
- ¿en qué punto de un flujo de procesamiento compensa meter inteligencia artificial generativa y dónde es mejor quedarse con reglas fijas baratas?

