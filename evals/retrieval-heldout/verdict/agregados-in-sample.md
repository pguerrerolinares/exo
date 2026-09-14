> In-sample: A0 se eligió sobre estas 55. DESCRIPTIVO, no decide (pre-registro §6 R2). Líneas 'decisiones' de abajo NO aplican.

# Agregados — campaña C (modo de decisión: lenient)

- filas: 56 · no nulas: 55 · nulas (corpus negativo): 1
- no nulas por estrato: hard=10, log=45
- queries no nulas con fusión activa (≥1 candidato FTS en base): 38
- capturas con error por índice: {'base': 0, 'solape': 0}

| brazo | hit@5 lenient [Wilson 95%] | hit@5 strict | hit@1 | MRR@10 |
|---|---|---|---|---|
| base:sellado | 43/55 [0.656, 0.871] | 43/55 | 10/55 | 0.4080 |
| base:vector | 41/55 [0.617, 0.842] | 41/55 | 10/55 | 0.4049 |
| base:fts | 20/55 [0.249, 0.496] | 20/55 | 10/55 | 0.2670 |
| base:rrf | 43/55 [0.656, 0.871] | 43/55 | 12/55 | 0.4415 |
| base:sellado-035 | 43/55 [0.656, 0.871] | 43/55 | 10/55 | 0.4080 |
| solape:sellado | 42/55 [0.637, 0.856] | 42/55 | 14/55 | 0.4401 |
| solape:rrf | 42/55 [0.637, 0.856] | 42/55 | 15/55 | 0.4584 |

## hit@5 por estrato

| brazo | hard | log |
|---|---|---|
| base:sellado | 9/10 | 34/45 |
| base:vector | 9/10 | 32/45 |
| base:fts | 0/10 | 20/45 |
| base:rrf | 9/10 | 34/45 |
| base:sellado-035 | 9/10 | 34/45 |
| solape:sellado | 8/10 | 34/45 |
| solape:rrf | 8/10 | 34/45 |

## corpus negativo (nulas con ≥1 resultado en top-5)

- base:sellado: 1/1
- base:vector: 1/1
- base:fts: 0/1
- base:rrf: 1/1
- base:sellado-035: 1/1
- solape:sellado: 1/1
- solape:rrf: 1/1

## pareadas (cand vs ref)

| cand | ref | ARREGLA | ROMPE | p McNemar exacto | IC95 ΔMRR@10 | regla GANA |
|---|---|---|---|---|---|---|
| base:vector | base:sellado | 0 | 2 | 0.5000 | [-0.0772, 0.0664] | NO GANA |
| base:fts | base:sellado | 2 | 25 | 0.0000 | [-0.2254, -0.0565] | NO GANA |
| base:rrf | base:sellado | 2 | 2 | 1.0000 | [-0.0330, 0.1051] | NO GANA |
| solape:sellado | base:sellado | 0 | 1 | 1.0000 | [-0.0049, 0.0762] | NO GANA |
| solape:rrf | base:rrf | 0 | 1 | 1.0000 | [-0.0171, 0.0575] | NO GANA |

## decisiones (NETO ≥ 3, ARREGLA ≥ 2.0·ROMPE, veto ΔMRR)

- R1 (H7): GENERALIZA
- R2 (H7b, RRF): SE QUEDA LA FUSIÓN ACTUAL
- R3 (H14a, solape): SE QUEDA EL TROCEADO ACTUAL
- R4 (H14b, late): no medido

## Con overlay fila 13 (H24)

# Agregados — campaña C (modo de decisión: lenient)

- filas: 56 · no nulas: 55 · nulas (corpus negativo): 1
- no nulas por estrato: hard=10, log=45
- queries no nulas con fusión activa (≥1 candidato FTS en base): 38
- capturas con error por índice: {'base': 0, 'solape': 0}

| brazo | hit@5 lenient [Wilson 95%] | hit@5 strict | hit@1 | MRR@10 |
|---|---|---|---|---|
| base:sellado | 43/55 [0.656, 0.871] | 43/55 | 10/55 | 0.4080 |
| base:vector | 41/55 [0.617, 0.842] | 41/55 | 10/55 | 0.4049 |
| base:fts | 20/55 [0.249, 0.496] | 20/55 | 10/55 | 0.2670 |
| base:rrf | 43/55 [0.656, 0.871] | 43/55 | 12/55 | 0.4415 |
| base:sellado-035 | 43/55 [0.656, 0.871] | 43/55 | 10/55 | 0.4080 |
| solape:sellado | 42/55 [0.637, 0.856] | 42/55 | 14/55 | 0.4401 |
| solape:rrf | 42/55 [0.637, 0.856] | 42/55 | 15/55 | 0.4584 |

## hit@5 por estrato

| brazo | hard | log |
|---|---|---|
| base:sellado | 9/10 | 34/45 |
| base:vector | 9/10 | 32/45 |
| base:fts | 0/10 | 20/45 |
| base:rrf | 9/10 | 34/45 |
| base:sellado-035 | 9/10 | 34/45 |
| solape:sellado | 8/10 | 34/45 |
| solape:rrf | 8/10 | 34/45 |

## corpus negativo (nulas con ≥1 resultado en top-5)

- base:sellado: 1/1
- base:vector: 1/1
- base:fts: 0/1
- base:rrf: 1/1
- base:sellado-035: 1/1
- solape:sellado: 1/1
- solape:rrf: 1/1

## pareadas (cand vs ref)

| cand | ref | ARREGLA | ROMPE | p McNemar exacto | IC95 ΔMRR@10 | regla GANA |
|---|---|---|---|---|---|---|
| base:vector | base:sellado | 0 | 2 | 0.5000 | [-0.0772, 0.0664] | NO GANA |
| base:fts | base:sellado | 2 | 25 | 0.0000 | [-0.2254, -0.0565] | NO GANA |
| base:rrf | base:sellado | 2 | 2 | 1.0000 | [-0.0330, 0.1051] | NO GANA |
| solape:sellado | base:sellado | 0 | 1 | 1.0000 | [-0.0049, 0.0762] | NO GANA |
| solape:rrf | base:rrf | 0 | 1 | 1.0000 | [-0.0171, 0.0575] | NO GANA |

## decisiones (NETO ≥ 3, ARREGLA ≥ 2.0·ROMPE, veto ΔMRR)

- R1 (H7): GENERALIZA
- R2 (H7b, RRF): SE QUEDA LA FUSIÓN ACTUAL
- R3 (H14a, solape): SE QUEDA EL TROCEADO ACTUAL
- R4 (H14b, late): no medido
