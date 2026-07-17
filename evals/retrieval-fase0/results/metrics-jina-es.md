# metrics — jina-es (hit@5, 55 queries etiquetadas)

- **hybrid**: 43/55
- **text**: 43/55
- **vector**: 43/55
- queries con observation-hit en top-5 (hybrid): 0 → []

## sweep de threshold (hybrid, filtro por score)
- thr=0.35: 43/55
- thr=0.4: 42/55
- thr=0.45: 40/55
- thr=0.5: 33/55
- thr=0.55: 29/55
- thr=0.6: 25/55
- thr=0.65: 19/55

## atribución de misses (hybrid, thr=None)
- MISS `Backlog frentes abiertos` → text=miss vector=miss [both-miss]
- MISS `cge bitácora` → text=miss vector=HIT [fusion-miss]
- MISS `cge evaluación head-to-head cgeo benchmark harness metodolog` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp comparación eval` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp head to head cgeo benchmark` → text=miss vector=miss [both-miss]
- MISS `codebase-memory-mcp cmm bugs` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `criterios para saber cuándo tirar a la basura una solución c` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `reflex capa de reflejos plugin destilado canónico` → text=miss vector=miss [both-miss]
- MISS `reflex recall SessionStart hook basic-memory latencia` → text=miss vector=miss [both-miss]
- MISS `un generador de informes de investigación que además te marc` → text=miss vector=miss [both-miss]

## pareada jina-es vs baseline: ARREGLA 7 ['basic-memory limitaciones dolores contrato memoria v2', 'blog notas publicar contenido web pguerrero divulgación posts', 'reflex cristalización efímero durable prior-art', '¿en qué punto de un flujo de procesamiento compensa meter inteligencia artificial generativa y dónde es mejor quedarse con reglas fijas baratas?', 'la idea de que programar ya no es escribir código sino redactar el encargo con tanta precisión que la máquina no decida el diseño por ti', 'cambio tres líneas en una fuente y se me reprocesa la base de conocimiento entera; quiero que solo se recalcule lo afectado por el cambio', 'esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo'] · ROMPE 0 []

## pareada jina-es vs minilm: ARREGLA 4 ['basic-memory limitaciones dolores contrato memoria v2', 'cge motor code-graph bitácora backlog frentes', 'cmm seam roto h2h onyx bug /api handler', 'la idea de que programar ya no es escribir código sino redactar el encargo con tanta precisión que la máquina no decida el diseño por ti'] · ROMPE 2 ['criterios para saber cuándo tirar a la basura una solución complicada porque la versión simple rinde igual o mejor con menos líos', 'un generador de informes de investigación que además te marca qué afirmaciones están contrastadas con fuentes y de cuáles no fiarte']

_Nota: `analyze.py <arm> <arm-vs>` sobrescribe la sección pareada del fichero en
cada corrida (no acumula comparaciones) — la sección "vs baseline" arriba se
regeneró con `analyze.py jina-es baseline` (comparación canónica del gate) y
esta sección "vs minilm" se añadió a mano desde la corrida separada
`analyze.py jina-es minilm`, para preservar ambas comparaciones en el mismo
fichero sin perder ninguna._
