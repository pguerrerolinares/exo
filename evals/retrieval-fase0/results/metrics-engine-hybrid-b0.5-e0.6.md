# metrics — engine-hybrid-b0.5-e0.6 (hit@5, 55 queries etiquetadas)

- **hybrid**: 48/55
- **text**: 0/55
- **vector**: 0/55
- queries con observation-hit en top-5 (hybrid): 0 → []

## sweep de threshold (hybrid, filtro por score)
- thr=0.35: 48/55
- thr=0.4: 48/55
- thr=0.45: 45/55
- thr=0.5: 41/55
- thr=0.55: 33/55
- thr=0.6: 30/55
- thr=0.65: 30/55

## atribución de misses (hybrid, thr=None)
- MISS `Backlog frentes abiertos` → text=miss vector=miss [both-miss]
- MISS `cge bitácora` → text=miss vector=miss [both-miss]
- MISS `cge evaluación head-to-head cgeo benchmark harness metodolog` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `esa utilidad de terminal de solo lectura que da a las sesion` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `reflex capa de reflejos plugin destilado canónico` → text=miss vector=miss [both-miss]

## pareada engine-hybrid-b0.5-e0.6 vs jina-es: ARREGLA 6 ['cmm codebase-memory-mcp comparación eval', 'cmm codebase-memory-mcp head to head cgeo benchmark', 'codebase-memory-mcp cmm bugs', 'reflex recall SessionStart hook basic-memory latencia', 'criterios para saber cuándo tirar a la basura una solución complicada porque la versión simple rinde igual o mejor con menos líos', 'un generador de informes de investigación que además te marca qué afirmaciones están contrastadas con fuentes y de cuáles no fiarte'] · ROMPE 1 ['esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo']
