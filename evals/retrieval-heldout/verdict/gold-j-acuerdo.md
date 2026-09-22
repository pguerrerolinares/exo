# Acuerdo entre jueces — gold J (fable × Kimi, ciegos)

- filas juzgadas por ambos: 285 · acuerdo (estricto+lenient): 249 (0.874) · estricto: 240 · κ estricto: 0.810
- κ / p_o sin `negativo` ni candidatos vacíos: 0.780 / 0.834 (sobre 217 filas; descriptivo, no decide)
- suelo pre-registrado: κ ≥ 0.6 y acuerdo ≥ 0.7 → PASA

| estrato | juzgadas | acuerdo | p_o | κ | entran al gold | no nulas |
|---|---|---|---|---|---|---|
| agent-search | 56 | 55 | 0.982 | 0.924 | 55 | 53 |
| archive | 20 | 17 | 0.850 | 0.738 | 8 | 8 |
| hard | 30 | 30 | 1.000 | 0.965 | 30 | 30 |
| negativo | 40 | 40 | 1.000 | 1.000 | 40 | 0 |
| prompt | 139 | 107 | 0.770 | 0.662 | 107 | 54 |

## auditoría del sesgo léxico (fracción de tokens de la query ≥4 letras presentes en la nota esperada)

- filas acordadas no nulas: n=145 · mediana 0.5 · con solape ≥ 0,5: 74
- desacuerdos con alguna nota propuesta: n=36 · mediana 0.33 · con solape ≥ 0,5: 10
- lectura: si los desacuerdos se concentran en solape bajo, los jueces acuerdan sobre todo donde la query repite la nota (sesgo léxico); el subconjunto «léxicamente difícil» (solape < 0,5) de las no nulas es el guard de D-A (borrador §6).

## auditoría léxica por estrato (F8: separa sesgo léxico del juez de candidatos pobres en un estrato)

| estrato | filas | tasa descarte | mediana solape acordadas | mediana solape descartadas |
|---|---|---|---|---|
| agent-search | 56 | 0.018 | 1.0 | 1.0 |
| archive | 20 | 0.600 | 0.38 | 0.43 |
| hard | 30 | 0.000 | 0.47 | None |
| negativo | 40 | 0.000 | None | None |
| prompt | 139 | 0.230 | 0.33 | 0.33 |
- composición del subconjunto «léxicamente difícil» (solape < 0,5, guard de D-A §6) por estrato: agent-search=5, archive=7, hard=16, prompt=43

- descartes: 45 (archive sin expected en archive/=9, desacuerdo=36)
