# Agregados — campaña C (modo de decisión: lenient)

- filas: 147 · no nulas: 92 · nulas (corpus negativo): 55
- no nulas por estrato: agent-search=34, hard=36, prompt=22
- queries no nulas con fusión activa (≥1 candidato FTS en base): 47
- capturas con error por índice: {'base': 0, 'solape': 0}

| brazo | hit@5 lenient [Wilson 95%] | hit@5 strict | hit@1 | MRR@10 |
|---|---|---|---|---|
| base:sellado | 64/92 [0.595, 0.780] | 54/92 | 29/92 | 0.4730 |
| base:vector | 63/92 [0.584, 0.771] | 52/92 | 37/92 | 0.5144 |
| base:fts | 23/92 [0.173, 0.347] | 21/92 | 14/92 | 0.1976 |
| base:rrf | 65/92 [0.607, 0.790] | 57/92 | 35/92 | 0.4999 |
| base:sellado-035 | 64/92 [0.595, 0.780] | 54/92 | 29/92 | 0.4743 |
| solape:sellado | 66/92 [0.618, 0.799] | 54/92 | 29/92 | 0.4741 |
| solape:rrf | 67/92 [0.630, 0.809] | 56/92 | 34/92 | 0.4941 |

## hit@5 por estrato

| brazo | agent-search | hard | prompt |
|---|---|---|---|
| base:sellado | 30/34 | 23/36 | 11/22 |
| base:vector | 31/34 | 21/36 | 11/22 |
| base:fts | 14/34 | 8/36 | 1/22 |
| base:rrf | 31/34 | 23/36 | 11/22 |
| base:sellado-035 | 30/34 | 23/36 | 11/22 |
| solape:sellado | 30/34 | 25/36 | 11/22 |
| solape:rrf | 31/34 | 25/36 | 11/22 |

## corpus negativo (nulas con ≥1 resultado en top-5)

- base:sellado: 54/55
- base:vector: 54/55
- base:fts: 18/55
- base:rrf: 54/55
- base:sellado-035: 55/55
- solape:sellado: 54/55
- solape:rrf: 54/55

## pareadas (cand vs ref)

| cand | ref | ARREGLA | ROMPE | p McNemar exacto | IC95 ΔMRR@10 | regla GANA |
|---|---|---|---|---|---|---|
| base:vector | base:sellado | 3 | 4 | 1.0000 | [-0.0081, 0.0897] | NO GANA |
| base:fts | base:sellado | 0 | 41 | 0.0000 | [-0.3536, -0.1985] | NO GANA |
| base:rrf | base:sellado | 1 | 0 | 1.0000 | [-0.0109, 0.0657] | NO GANA |
| solape:sellado | base:sellado | 4 | 2 | 0.6875 | [-0.0335, 0.0333] | NO GANA |
| solape:rrf | base:rrf | 4 | 2 | 0.6875 | [-0.0396, 0.0250] | NO GANA |

## decisiones (NETO ≥ 3, ARREGLA ≥ 2.0·ROMPE, veto ΔMRR)

- R1 (H7): GENERALIZA
- R2 (H7b, RRF): SE QUEDA LA FUSIÓN ACTUAL
- R3 (H14a, solape): SE QUEDA EL TROCEADO ACTUAL
- R4 (H14b, late): no medido

## hit@5 por sub-tipo hard (S4, descriptivo, modo lenient; hard-corta = ids impares c113…c147, hard-larga = ids pares c112…c146)

| brazo | hard-corta | hard-larga |
|---|---|---|
| base:sellado | 11/18 | 12/18 |
| base:vector | 9/18 | 12/18 |
| base:fts | 8/18 | 0/18 |
| base:rrf | 11/18 | 12/18 |
| base:sellado-035 | 11/18 | 12/18 |
| solape:sellado | 12/18 | 13/18 |
| solape:rrf | 12/18 | 13/18 |
