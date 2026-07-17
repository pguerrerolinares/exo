# metrics — minilm (hit@5, 55 queries etiquetadas)

- **hybrid**: 41/55
- **text**: 41/55
- **vector**: 41/55
- queries con observation-hit en top-5 (hybrid): 0 → []

## sweep de threshold (hybrid, filtro por score)
- thr=0.35: 41/55
- thr=0.4: 41/55
- thr=0.45: 41/55
- thr=0.5: 39/55
- thr=0.55: 36/55
- thr=0.6: 25/55
- thr=0.65: 18/55

## atribución de misses (hybrid, thr=None)
- MISS `Backlog frentes abiertos` → text=miss vector=HIT [fusion-miss]
- MISS `basic-memory limitaciones dolores contrato memoria v2` → text=miss vector=miss [both-miss]
- MISS `cge bitácora` → text=miss vector=miss [both-miss]
- MISS `cge evaluación head-to-head cgeo benchmark harness metodolog` → text=miss vector=miss [both-miss]
- MISS `cge motor code-graph bitácora backlog frentes` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp comparación eval` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp head to head cgeo benchmark` → text=miss vector=miss [both-miss]
- MISS `cmm seam roto h2h onyx bug /api handler` → text=miss vector=miss [both-miss]
- MISS `codebase-memory-mcp cmm bugs` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `la idea de que programar ya no es escribir código sino redac` → text=miss vector=miss [both-miss]
- MISS `reflex capa de reflejos plugin destilado canónico` → text=miss vector=miss [both-miss]
- MISS `reflex recall SessionStart hook basic-memory latencia` → text=miss vector=miss [both-miss]

## pareada minilm vs baseline: ARREGLA 7 ['blog notas publicar contenido web pguerrero divulgación posts', 'reflex cristalización efímero durable prior-art', '¿en qué punto de un flujo de procesamiento compensa meter inteligencia artificial generativa y dónde es mejor quedarse con reglas fijas baratas?', 'criterios para saber cuándo tirar a la basura una solución complicada porque la versión simple rinde igual o mejor con menos líos', 'cambio tres líneas en una fuente y se me reprocesa la base de conocimiento entera; quiero que solo se recalcule lo afectado por el cambio', 'esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo', 'un generador de informes de investigación que además te marca qué afirmaciones están contrastadas con fuentes y de cuáles no fiarte'] · ROMPE 2 ['cge motor code-graph bitácora backlog frentes', 'cmm seam roto h2h onyx bug /api handler']
