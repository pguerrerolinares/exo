# metrics — textfts (hit@5, 55 queries etiquetadas)

- **hybrid**: 41/55
- **text**: 18/55
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
- MISS `cmm seam roto h2h onyx bug /api handler` → text=HIT vector=miss [fusion-miss]
- MISS `codebase-memory-mcp cmm bugs` → text=miss vector=miss [both-miss]
- MISS `coste workflows multi-agente tokens lección` → text=miss vector=miss [both-miss]
- MISS `fabrica campaña` → text=miss vector=miss [both-miss]
- MISS `la idea de que programar ya no es escribir código sino redac` → text=HIT vector=miss [fusion-miss]
- MISS `reflex capa de reflejos plugin destilado canónico` → text=miss vector=miss [both-miss]
- MISS `reflex recall SessionStart hook basic-memory latencia` → text=miss vector=miss [both-miss]
