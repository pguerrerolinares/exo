# metrics — engine-hybrid-m209 (hit@5, 55 queries etiquetadas)

- **hybrid**: 48/55
- **text**: 0/55
- **vector**: 0/55
- queries con observation-hit en top-5 (hybrid): 0 → []

## sweep de threshold (hybrid, filtro por score)
- thr=0.35: 48/55
- thr=0.4: 48/55
- thr=0.45: 46/55
- thr=0.5: 38/55
- thr=0.55: 23/55
- thr=0.6: 14/55
- thr=0.65: 0/55

## atribución de misses (hybrid, thr=None)
- MISS `Frente 9 lighthouses Fase 4 divergencia core split thin-core` → text=miss vector=miss [both-miss]
- MISS `cge evaluación head-to-head cgeo benchmark harness metodolog` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `esa utilidad de terminal de solo lectura que da a las sesion` → text=miss vector=miss [both-miss]
- MISS `fabrica campaign harness config gate merge asíncrono roadmap` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `fabrica roadmap campana lighthouses diversidad bots Fase 3` → text=miss vector=miss [both-miss]

## pareada engine-hybrid-m209 vs bm-m209: ARREGLA 13 ['Backlog frentes abiertos', 'backlog frentes abiertos cge ORM', 'blog notas publicar contenido web pguerrero divulgación posts', 'cge bitácora', 'cmm codebase-memory-mcp comparación eval', 'cmm codebase-memory-mcp head to head cgeo benchmark', 'codebase-memory-mcp cmm bugs', 'consolida bug kbx detectado', 'kbx bitacora', 'pguerrero-music-bitacora', 'reflex capa de reflejos plugin destilado canónico', 'reflex recall SessionStart hook basic-memory latencia', 'criterios para saber cuándo tirar a la basura una solución complicada porque la versión simple rinde igual o mejor con menos líos'] · ROMPE 4 ['fabrica campaign harness config gate merge asíncrono roadmap', 'fabrica roadmap campana lighthouses diversidad bots Fase 3', 'Frente 9 lighthouses Fase 4 divergencia core split thin-core', 'esa utilidad de terminal de solo lectura que da a las sesiones un resumen estructural barato de mis notas para no gastar tokens leyéndolo todo']
