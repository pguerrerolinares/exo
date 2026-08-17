# metrics — bm-m209 (hit@5, 55 queries etiquetadas)

- **hybrid**: 39/55
- **text**: 39/55
- **vector**: 40/55
- queries con observation-hit en top-5 (hybrid): 0 → []

## sweep de threshold (hybrid, filtro por score)
- thr=0.35: 39/55
- thr=0.4: 38/55
- thr=0.45: 37/55
- thr=0.5: 30/55
- thr=0.55: 26/55
- thr=0.6: 21/55
- thr=0.65: 14/55

## atribución de misses (hybrid, thr=None)
- MISS `Backlog frentes abiertos` → text=miss vector=miss [both-miss]
- MISS `backlog frentes abiertos cge ORM` → text=miss vector=miss [both-miss]
- MISS `blog notas publicar contenido web pguerrero divulgación post` → text=miss vector=miss [both-miss]
- MISS `cge bitácora` → text=miss vector=HIT [fusion-miss]
- MISS `cge evaluación head-to-head cgeo benchmark harness metodolog` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp comparación eval` → text=miss vector=miss [both-miss]
- MISS `cmm codebase-memory-mcp head to head cgeo benchmark` → text=miss vector=miss [both-miss]
- MISS `codebase-memory-mcp cmm bugs` → text=miss vector=miss [both-miss]
- MISS `consolida bug kbx detectado` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `criterios para saber cuándo tirar a la basura una solución c` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `kbx bitacora` → text=miss vector=miss [both-miss]
- MISS `pguerrero-music-bitacora` → text=miss vector=miss [both-miss]
- MISS `reflex capa de reflejos plugin destilado canónico` → text=miss vector=miss [both-miss]
- MISS `reflex recall SessionStart hook basic-memory latencia` → text=miss vector=miss [both-miss]
