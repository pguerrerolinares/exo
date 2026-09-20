# Diagnóstico por fila de las 55 in-sample — campaña J, T0 (descriptivo; no decide)

- no nulas: 55 · hist misses 6 (49/55) · S misses 12 (43/55) · hoy misses 14 (41/55)
- transición hist→S: hit→miss 7 · miss→hit 1 · miss→miss 5
- transición S→hoy: hit→miss 2 · miss→hit 0 · miss→miss 12
- etiquetas cuyo expected_permalink no existe en la KB de hoy: 0
- filas marcadas re-verificar (auditoría 2026-09-04): 11; de ellas miss en S: 7
- no nulas con ≥1 token raro (1 ≤ df ≤ ⌈0.25·174⌉ = 44): 52/55 — filas en las que F1 (FTS OR sobre raros) tendría canal léxico; de los misses en S: 11/12

## causas de los 12 misses en S (una fila puede llevar varias)

- vector-bajo-umbral: 1
- vector-lejos: 11
- fusion-desplaza: 0
- fts-and-vacio: 2
- fts-sin-relevante: 7
- rotacion-en-top5: 6
- etiqueta-reverificar: 7
- captura-error: 0
- plazas de archive/ en el top-5 de las no nulas: 114/275
- no nulas con FTS vacío (AND): 17/55

## causas de los 14 misses en hoy (una fila puede llevar varias)

- vector-bajo-umbral: 1
- vector-lejos: 12
- fusion-desplaza: 1
- fts-and-vacio: 2
- fts-sin-relevante: 7
- rotacion-en-top5: 6
- etiqueta-reverificar: 7
- captura-error: 0
- plazas de archive/ en el top-5 de las no nulas: 116/275
- no nulas con FTS vacío (AND): 17/55

## por fila (no nulas; rv=rango vector completo, sv=score vector, rva=rango vector ≥0.40, nF=candidatos FTS, rf=rango FTS, rh=rango hybrid, a5=archive en top-5, rot=rotación de la esperada en top-5, df0=tokens con df 0 / tokens)

| id | src | hist | S | hoy | rv_S | sv_S | rva_S | nF_S | rf_S | rh_S | a5_S | rot_S | df0 | causas_S | causas_hoy |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| m01 | log | hit | hit | miss | 3 | 0.536 | 3 | 29 | 4 | 5 | 1 | sí | 0/2 | — | fusion-desplaza |
| m02 | log | hit | hit | hit | 3 | 0.592 | 3 | 21 | 6 | 4 | 1 | sí | 0/2 | — | — |
| m03 | log | hit | hit | hit | 1 | 0.6 | 1 | 6 | 1 | 1 | 3 | no | 0/5 | — | — |
| m04 | log | hit | hit | hit | 2 | 0.568 | 2 | 1 | 1 | 2 | 1 | no | 0/3 | — | — |
| m05 | log | hit | miss | miss | 10 | 0.524 | 10 | 3 | 2 | — | 2 | no | 0/1 | vector-lejos | vector-lejos |
| m06 | log | hit | hit | hit | 2 | 0.576 | 2 | 50 | 1 | 2 | 1 | no | 0/3 | — | — |
| m07 | log | hit | hit | hit | 3 | 0.537 | 3 | 12 | — | 4 | 2 | no | 0/5 | — | — |
| m08 | log | hit | hit | hit | 1 | 0.501 | 1 | 0 | — | 1 | 3 | no | 1/6 | — | — |
| m10 | log | hit | hit | hit | 4 | 0.495 | 4 | 0 | — | 4 | 1 | no | 0/8 | — | — |
| m11 | log | hit | hit | hit | 1 | 0.563 | 1 | 0 | — | 1 | 2 | no | 0/4 | — | — |
| m12 | log | miss | miss | miss | 9 | 0.493 | 9 | 25 | 7 | 7 | 3 | sí | 0/2 | vector-lejos,rotacion-en-top5 | vector-lejos,rotacion-en-top5 |
| m13 | log | miss | miss | miss | 8 | 0.484 | 8 | 0 | — | 8 | 4 | no | 0/7 | vector-lejos,fts-and-vacio,etiqueta-reverificar | vector-lejos,fts-and-vacio,etiqueta-reverificar |
| m14 | log | hit | hit | hit | 3 | 0.566 | 3 | 10 | 1 | 1 | 1 | no | 0/6 | — | — |
| m15 | log | hit | hit | hit | 2 | 0.629 | 2 | 2 | 2 | 2 | 3 | no | 0/2 | — | — |
| m16 | log | hit | hit | hit | 4 | 0.548 | 4 | 1 | — | 4 | 3 | no | 0/4 | — | — |
| m17 | log | hit | miss | miss | 14 | 0.517 | 14 | 4 | 2 | 9 | 4 | no | 0/7 | vector-lejos | vector-lejos |
| m18 | log | hit | hit | hit | 4 | 0.527 | 4 | 1 | — | 5 | 4 | no | 0/8 | — | — |
| m19 | log | hit | hit | hit | 27 | 0.47 | 27 | 11 | 1 | 1 | 3 | no | 0/3 | — | — |
| m20 | log | hit | hit | hit | 2 | 0.423 | 2 | 3 | — | 5 | 2 | no | 0/4 | — | — |
| m21 | log | miss | miss | miss | 117 | 0.391 | — | 4 | — | — | 2 | no | 0/5 | vector-bajo-umbral,fts-sin-relevante | vector-bajo-umbral,fts-sin-relevante |
| m22 | log | hit | miss | miss | 31 | 0.472 | 31 | 3 | — | — | 3 | sí | 0/7 | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar |
| m23 | log | hit | hit | hit | 3 | 0.535 | 3 | 1 | — | 4 | 1 | sí | 0/8 | — | — |
| m24 | log | miss | miss | miss | 20 | 0.421 | 20 | 31 | — | — | 3 | no | 0/2 | vector-lejos,fts-sin-relevante,etiqueta-reverificar | vector-lejos,fts-sin-relevante,etiqueta-reverificar |
| m25 | log | hit | hit | hit | 5 | 0.476 | 5 | 3 | — | 5 | 3 | sí | 0/5 | — | — |
| m26 | log | hit | hit | hit | 1 | 0.562 | 1 | 1 | 1 | 1 | 2 | sí | 0/8 | — | — |
| m27 | log | hit | miss | miss | 6 | 0.532 | 6 | 1 | — | 6 | 3 | sí | 0/8 | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar |
| m28 | log | hit | hit | hit | 3 | 0.426 | 3 | 8 | 1 | 1 | 0 | no | 0/1 | — | — |
| m29 | log | hit | miss | miss | 12 | 0.45 | 12 | 2 | — | — | 2 | sí | 0/9 | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar |
| m30 | log | hit | miss | miss | 50 | 0.435 | 50 | 1 | — | — | 2 | sí | 0/6 | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar |
| m31 | log | hit | hit | hit | 3 | 0.489 | 3 | 31 | 2 | 2 | 2 | no | 0/1 | — | — |
| m32 | log | hit | hit | hit | 2 | 0.585 | 2 | 27 | 1 | 2 | 2 | sí | 0/2 | — | — |
| m33 | log | hit | hit | hit | 2 | 0.531 | 2 | 11 | 4 | 4 | 3 | no | 0/3 | — | — |
| m34 | log | hit | miss | miss | 54 | 0.448 | 54 | 3 | — | — | 4 | sí | 0/3 | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar | vector-lejos,fts-sin-relevante,rotacion-en-top5,etiqueta-reverificar |
| m35 | log | hit | hit | hit | 4 | 0.52 | 4 | 2 | 1 | 1 | 3 | no | 0/6 | — | — |
| m36 | log | hit | hit | hit | 3 | 0.549 | 3 | 2 | — | 3 | 3 | no | 0/7 | — | — |
| m37 | log | hit | hit | hit | 1 | 0.598 | 1 | 5 | 2 | 2 | 2 | no | 0/4 | — | — |
| m38 | log | hit | hit | hit | 2 | 0.484 | 2 | 0 | — | 2 | 2 | no | 0/7 | — | — |
| m39 | log | hit | hit | hit | 1 | 0.524 | 1 | 22 | 8 | 4 | 1 | no | 0/1 | — | — |
| m40 | log | hit | hit | hit | 1 | 0.555 | 1 | 6 | 4 | 2 | 2 | sí | 0/1 | — | — |
| m41 | log | hit | hit | hit | 3 | 0.527 | 3 | 1 | — | 4 | 0 | no | 0/5 | — | — |
| m42 | log | miss | hit | hit | 5 | 0.529 | 5 | 0 | — | 5 | 2 | no | 0/7 | — | — |
| m43 | log | hit | hit | hit | 1 | 0.532 | 1 | 5 | 2 | 2 | 0 | no | 0/5 | — | — |
| m44 | log | hit | hit | miss | 7 | 0.514 | 7 | 3 | 2 | 3 | 2 | no | 0/6 | — | vector-lejos |
| m45 | log | hit | hit | hit | 1 | 0.56 | 1 | 0 | — | 1 | 2 | no | 1/7 | — | — |
| m46 | log | hit | hit | hit | 1 | 0.563 | 1 | 1 | 1 | 1 | 3 | no | 0/6 | — | — |
| m47 | hard | hit | hit | hit | 2 | 0.497 | 2 | 0 | — | 2 | 2 | no | 1/22 | — | — |
| m48 | hard | hit | hit | hit | 3 | 0.438 | 3 | 0 | — | 3 | 1 | no | 2/26 | — | — |
| m49 | hard | hit | hit | hit | 2 | 0.516 | 2 | 0 | — | 2 | 3 | no | 2/22 | — | — |
| m50 | hard | hit | hit | hit | 3 | 0.524 | 3 | 0 | — | 3 | 1 | no | 1/21 | — | — |
| m51 | hard | hit | hit | hit | 3 | 0.481 | 3 | 0 | — | 3 | 1 | no | 3/25 | — | — |
| m52 | hard | miss | miss | miss | 7 | 0.471 | 7 | 0 | — | 7 | 1 | no | 1/25 | vector-lejos,fts-and-vacio | vector-lejos,fts-and-vacio |
| m53 | hard | hit | hit | hit | 3 | 0.458 | 3 | 0 | — | 3 | 2 | no | 2/29 | — | — |
| m54 | hard | hit | hit | hit | 2 | 0.507 | 2 | 0 | — | 2 | 2 | no | 1/23 | — | — |
| m55 | hard | hit | hit | hit | 2 | 0.504 | 2 | 0 | — | 2 | 2 | no | 1/21 | — | — |
| m56 | hard | hit | hit | hit | 4 | 0.495 | 4 | 0 | — | 4 | 1 | no | 2/26 | — | — |

## Lectura (T0; in-sample; descriptivo; no decide nada sobre el motor)

- Condiciones: hist = `metrics-engine-hybrid-b0.0-e0.6.md` (KB 138 notas, binario del sweep de 2026-07); S = captura de C (binario `4f4d2a8`, snapshot `885246d`, 174 notas); hoy = `~/.exo/index.db` de producción con el binario `d97eaae3a9a353025531ae01d1f337dc732975a4` el `2026-09-20`. Ninguna cifra es comparable entre estados en absoluto (cambian KB, distractores y binario a la vez; C §6 R1): solo se cruzan filas.
- El «6 de 55 que dejaron de acertar» del backlog es neto: hist→S es hit→miss 7, miss→hit 1, miss→miss 5.
- De los 12 misses en S, 11 son `vector-lejos` (la nota está admitida ≥0,40 pero fuera del top-5), 1 `vector-bajo-umbral`, 0 `fusion-desplaza`. El AND deja FTS vacío en 2 de esos misses y en 17/55 del total.
- 6 de los 12 misses tienen en su top-5 una rotación de la nota esperada (`archive/log/<misma base>`), y 7 de las 11 filas que la auditoría del 2026-09-04 mandó re-verificar (9 SUPERADA + 2 AMBIGUA) son miss en S. Lectura: buena parte de la caída es etiqueta superada por rotación, no motor. No se re-etiqueta (C §11); se anota para el estrato `archive` del gold de J.
- `archive/` ocupa 114/275 plazas del top-5 de las 55 en S.
- F1 (FTS OR sobre tokens raros, tope df 25 %): 52/55 queries tendrían canal léxico, 11 de los misses. Valor para D-J8: 0,25 se mantiene — cobertura 52/55 ≈ 94,5 % ≥ 80 %, cláusula no ejercida.
- S→hoy: hit→miss 2, miss→hit 0. Etiquetas muertas hoy: 0.
- Lo que este fichero NO permite concluir: nada sobre qué brazo adoptar (held-out de C consumido; el de J no existe), nada sobre «el binario empeoró».
