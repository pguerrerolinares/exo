# Verdict adjudicado — campaña C: retrieval fuera de muestra (H7, H7b, H14, H24)

- **Fecha**: 2026-09-14
- **Rol**: adjudicador (fable fresco, Task 10 del plan). No participé en ninguna fase de C: ni gold, ni verificación, ni consultor-gate, ni capturas, ni informe.
- **Contrato**: pre-registro `docs/superpowers/plans/2026-09-13-campana-c-preregistro.md`, congelado en `b5129410b9ba1a3ddbeb030ffb076ea69ba9fac4`. Toda decisión lleva cita textual (bloque `>`) del §6; sin cita, la decisión es inválida. Los números no se renegocian.
- **Base de evidencia de todas las cifras held-out**: `measured / held-out / N=92 / binario 4f4d2a8214c5db2eba4acf285a318bae4af2a83a / snapshot 885246df3a428fa3895c209030eb90a8e254005f`. Las cifras de las 55: `measured / in-sample (sesgado)`. Salvo indicación, modo **lenient** (D5, §9).
- Este fichero no contiene texto de queries ni permalinks por fila (repo público, §8). El detalle por fila y mi trail de adjudicación viven en `.superpowers/fabrica/verdicts/c-retrieval-heldout.md` (gitignored).

## 0. Verificación primaria propia (antes de leer las decisiones)

| comprobación | resultado |
|---|---|
| `git diff b512941 -- <pre-registro>` | vacío (0 bytes): el pre-registro no se tocó después de congelar |
| `sha256sum $PRIV/gold.jsonl` | `614ae599…c43a75` = §10 · 147 filas |
| Re-corrida de `metricas.py informe` con los argumentos de `agregados.md`, a fichero del scratchpad | `diff` contra el `agregados.md` commiteado = **solo** la sección "hit@5 por sub-tipo hard (S4…)" añadida por script. Nada más difiere |
| `detalle.jsonl` privado vs el que produjo mi corrida | idénticos |
| Recomputación **sin `metricas.py`** (script propio desde `cap-{base,solape}.jsonl` + gold: fusión sellada, vector, FTS, RRF, sellado-035) | 0 discrepancias con `detalle.jsonl` en hit@5 / rr@10 / n_resultados en 147 filas × 7 brazos |
| Fidelidad propia (sellado offline == lista `hybrid` del binario, orden) | 0/147 discrepancias en `base` y 0/147 en `solape` |
| Pareadas recomputadas (ARREGLA/ROMPE, McNemar exacto binomial, IC95 bootstrap con la semilla y con una variante propia) | `base:vector` 3/4 p=1,0000 · `base:rrf` 1/0 p=1,0000 · `solape:sellado` 4/2 p=0,6875 · `base:fts` 0/41 · `solape:rrf` vs `base:rrf` 4/2. IC95 ΔMRR con semilla idénticos a 4 decimales; con mi variante, iguales a 2–3 decimales y mismo signo del límite superior |
| Sección S4 recomputada (regla §9: impares c113…c147 = corta, pares c112…c146 = larga; 18+18; comprobé además que las impares tienen ≤5 palabras y las pares 16–30) | 11/18–12/18 · 9/18–12/18 · 8/18–0/18 · 11/18–12/18 · 11/18–12/18 · 12/18–13/18 · 12/18–13/18: coincide celda a celda |
| Tabla de potencia §7 re-derivada (multinomial exacta sobre (ARREGLA, ROMPE)) | reproduce 0,07/0,13 · 0,15/0,16 · 0,81/0,93 · 0,86/0,93 y la potencia McNemar 0,38/0,66. La fila "+5 pp" solo se reproduce con discordancia 7 % (ver §8, errata) |
| Muestra de discordantes abierta contra el snapshot | 6 filas (5 por semilla `20260914` sobre las 12 discordantes únicas + la única discordante de R2, añadida y declarada): **c124, c141, c129, c123, c073, c107**. En las 6, las notas relevantes existen en el snapshot, la esperada contiene el hecho citado en `notes`, y el top-5 de cada brazo es el que dicta la captura. Hit/miss **real en 6/6**. Dos de ellas (c073, c141) tienen en el top-5 de A0 una nota que discutiblemente sería aceptable: un error del gold, si lo es, que **favorece a A0** y no cambia ninguna decisión (§10) |

Capturas con error: 0 en ambos índices (§11: sin circuit breaker disparado). Gold aprobado por Paul (línea `GATE: GOLD APROBADO 2026-09-14` en el package, sha coincidente).

## 1. Decisiones R1–R4

### R1 — H7, ¿generaliza el sellado? → **GENERALIZA**

Cifras (`measured / held-out / N=92 / binario 4f4d2a8 / snapshot 885246d`):

- A0 frente a A1 (`base:vector`): ARREGLA(A0) = **4**, ROMPE(A0) = **3** (la pareada `base:vector` vs `base:sellado` da 3/4). A1 NO GANA: NETO = −1.
- A0 frente a A2 (`base:fts`): ARREGLA(A0) = **41**, ROMPE(A0) = **0**. A2 NO GANA.
- hit@5 absoluto de A0: **64/92 = 69,6 %**, Wilson 95 % **[59,5 %, 78,0 %]**.

> **GENERALIZA** si A0 no pierde contra ninguno de sus componentes: en las pareadas A0 frente a A1 y A0 frente a A2, `ARREGLA(A0) ≥ ROMPE(A0)`.

4 ≥ 3 y 41 ≥ 0: se cumple. Y ninguna de las dos alternativas dispara el NO GENERALIZA:

> **NO GENERALIZA** si A1 o A2 cumplen la regla GANA contra A0: un componente sin fusión es claramente mejor fuera de muestra.

**En rojo, sin maquillar:** frente a A1 el margen es **una sola query** (4/3). Con un ARREGLA(A0) menos sería 3/3 (sigue GENERALIZA, por el `≥`) y con dos menos, 2/4 → INDETERMINADO. Por eso abrí las 4 filas ARREGLA(A0) (c087, c117, c123, c143) además de la muestra: en las cuatro, el top-5 de vector no contiene el hecho preguntado en ninguna nota, y A0 acierta porque FTS mete la nota correcta (tres son archivadas o de bitácora, léxicamente exactas). Son misses reales de vector. Las 3 ROMPE(A0) (c073, c107, c141) son hits reales de vector; en dos, A0 tiene en su top-5 una nota que un usuario razonable daría por válida (ver §10). El error del gold, si lo hay, empuja hacia 4/2 o 4/1, no hacia INDETERMINADO.

Descriptivo que pide el §6 (firma del sobreajuste): NETO de A0 frente a A3 (RRF) in-sample = **0** (pareada 2/2) y held-out = **−1** (pareada 1/0).

> Descriptivo: el NETO de A0 frente a A3 in-sample y held-out. Si la ventaja de A0 sobre una fusión sin afinar existe solo in-sample, esa es la firma del sobreajuste.

No hay ventaja de A0 sobre RRF ni in-sample ni held-out: **no aparece la firma del sobreajuste** por este criterio.

El nivel absoluto **no se compara con 48/55**:

> El nivel absoluto del hit@5 de A0 en el held-out se reporta con su Wilson y **no se compara con 48/55**: la fuente de las queries cambia (§7) y la diferencia mezclaría sobreajuste con cambio de distribución.

### R2 — H7b, RRF → **SE QUEDA LA FUSIÓN ACTUAL**

Cifra: `base:rrf` vs `base:sellado`: ARREGLA = **1**, ROMPE = **0**, NETO = 1, p McNemar = 1,0000, IC95 ΔMRR@10 = [−0,0109, +0,0657]. hit@5 65/92 vs 64/92; MRR 0,4999 vs 0,4730.

> Se adopta RRF **solo si A3 cumple GANA contra A0** en el held-out. Si no, se queda la fusión actual.

> **Regla GANA (una sola, para todos los candidatos):** `NETO ≥ 3` **y** `ARREGLA ≥ 2·ROMPE` **y** el límite superior del IC95 de ΔMRR@10 (X − A0) es `≥ 0`

NETO = 1 < 3: NO GANA. RRF no rompe nada (0) y arregla 1; el veto de MRR no se activa. Las 55 in-sample (`measured / in-sample (sesgado)`): 2/2, NETO 0, "pareada **sesgada en contra de RRF**, porque A0 se eligió sobre ellas, y no deciden". Solo 47/92 queries tienen fusión activa (§2.4), así que RRF y A0 solo podían discrepar en esas 47; discreparon en el top-5 en 1.

### R3 — H14a, solape → **SE QUEDA EL TROCEADO ACTUAL**

Cifra: `solape:sellado` vs `base:sellado`: ARREGLA = **4**, ROMPE = **2**, NETO = 2, p McNemar = 0,6875, IC95 ΔMRR@10 = [−0,0335, +0,0333]. hit@5 66/92 vs 64/92; MRR 0,4741 vs 0,4730.

> El troceado con solape se adopta si A4 cumple GANA contra A0 **y** el p95 de latencia de `exo search --type hybrid` sobre su índice es ≤ 1,25× el de `base` (§7, protocolo de latencia).

NETO = 2 < 3: NO GANA en retrieval. El guard de latencia no llega a decidir (p95 solape 1,0258 s ≤ 1,25 × 1,0240 s = 1,2800 s lo habría pasado; ver §5). ARREGLA ≥ 2·ROMPE sí se cumple (4 ≥ 4) y el IC de MRR no veta: la regla cae **solo por NETO**, a una query del umbral. Es exactamente el caso que el §6 contempla y que no se renegocia:

> **Empate o ambigüedad** (por ejemplo, un umbral de la regla que cae justo en el borde por una query con error de captura): lo adjudica fable con este texto delante y cita textual. Los números no se renegocian.

No hay error de captura (0/147 en `solape`), y las 6 discordantes de esta pareada son reales: abrí 4 de ellas (c124, c129, c141 en la muestra; c115 y c145 por el volcado): los ARREGLA vienen de que el nuevo troceado sube una nota aceptable (canon o bitácora) al top-5, y los ROMPE de que el trozo que puntuaba en `base` deja de existir con el paso de 720 (el solape cambia los cortes, no solo añade trozos). No hay borde que adjudicar: 2 < 3.

> **Si nada gana, el resultado válido es "se queda como está".** No se toca `buscador.rs` ni `trozos.rs`. La campaña cierra con el verdict y el backlog actualizado. NO GANA no significa "son iguales" (§7, potencia).

### R4 — H14b, late chunking → **NO MEDIDO** (D3 = no)

> `D3` brazo late chunking: `no — solo solape; A5, late:rrf y R4 no se miden` (Paul, 2026-09-13: "No, solo solape")

> **R4 — H14b (late chunking, si D3 = sí).** [...] Aunque R4 pase, **late chunking no se implementa en producción en esta campaña** [...] R4 GANA abre una spec de producción aparte.

No medido no es perder. No se abre spec aparte. Sigue siendo backlog con la viabilidad de §2.6 intacta.

### H24 — `acceptable_permalinks` y overlay de la fila 13

Overlay reportado con y sin (`agregados-in-sample.md`): las 55 dan **cifras idénticas** con y sin overlay (la fila 13 acierta o falla igual en todos los brazos). `gate.md` y los históricos no se tocan. El held-out sí usa aceptables (44/92 filas): lenient decide (D5), strict va como descriptivo (§4).

## 2. Base de evidencia por cifra

| cifra | valor | base de evidencia |
|---|---|---|
| hit@5 A0 | 64/92 [0,595, 0,780] | measured / held-out / N=92 / binario 4f4d2a8 / snapshot 885246d |
| pareada A1 vs A0 | 3/4 | ídem |
| pareada A2 vs A0 | 0/41 | ídem |
| pareada A3 (RRF) vs A0 | 1/0, IC ΔMRR [−0,011, +0,066] | ídem |
| pareada A4 (solape) vs A0 | 4/2, IC ΔMRR [−0,034, +0,033] | ídem |
| p95 base / solape | 1,0240 s / 1,0258 s | measured / intento 2 / hyperfine 30 runs / binario 4f4d2a8 (§5) |
| hit@5 A0 in-sample | 43/55 (con y sin overlay) | measured / in-sample (sesgado) / binario 4f4d2a8 / snapshot 885246d |
| pareada A3 vs A0 in-sample | 2/2 | measured / in-sample (sesgado en contra de RRF) |
| 48–49/55 histórico | — | measured / in-sample / **otro binario y otra KB (138 notas)** · no comparable (§6 R1) |
| S4, estratos, corpus negativo, hit@1, MRR | ver `agregados.md` | measured / held-out / descriptivos sin peso |

## 3. Potencia al N real (92): qué diferencia NO podía verse

Re-derivación propia (multinomial exacta sobre (ARREGLA, ROMPE); regla NETO ≥ 3 y ARREGLA ≥ 2·ROMPE, sin veto MRR, como la tabla del §7):

| escenario verdadero | N=60 (§7) | **N=92** | N=100 (§7) |
|---|---|---|---|
| X igual a A0, discordancia 5 % | 0,07 | **0,12** | 0,13 |
| X igual a A0, discordancia 10 % | 0,15 | **0,17** | 0,16 |
| X peor 5 pp | 0,00 | **0,00** | 0,00 |
| X mejor 2 pp (a=0,05, b=0,03) | 0,27 | **0,38** | 0,39 |
| X mejor 3 pp (a=0,06, b=0,03) | 0,37 | **0,50** | 0,51 |
| X mejor 5 pp (disc. 7 %, la que reproduce la fila del §7) | 0,58 | **0,80** | 0,83 |
| X mejor 5 pp (disc. 10 %) | 0,57 | **0,72** | 0,74 |
| X mejor 8 pp | 0,81 | **0,92** | 0,93 |
| X mejor 10 pp, discordancia 16 % | 0,86 | **0,92** | 0,93 |

Lectura aplicada a lo que salió:

- **Solape (4/2, NETO 2 = +2,2 pp observados):** una mejora real de 2–3 pp se adopta el 38–50 % de las veces con N=92. Es decir: la campaña **no podía distinguir** "solape mejora 2–3 pp" de "solape es igual". Lo que sí descarta con fuerza es "solape es peor 5 pp" (0,00) y, con menos, "mejor 8 pp o más" (habría ganado el 92 %).
- **RRF (1/0):** en las 47 queries con fusión activa, RRF y A0 son casi la misma lista en el top-5. Con discordancia tan baja, la regla no tenía nada que ver: ni "igual" ni "mejor 2 pp" se separan.
- **Vector (3/4):** una mejora real de 5 pp de vector sobre A0 se habría visto el 72–80 %; no apareció y NETO es negativo. Lo compatible con los datos va de "vector algo peor" a "vector igual en hit@5 y mejor en orden" (§6, descriptivo a).
- Nada de lo anterior es una afirmación de significación: los tres p de McNemar (1,00 · 1,00 · 0,69) no ven nada, como el §7 anticipaba ("ningún test de significación va a ver diferencias de 5 pp entre dos fusiones sobre los mismos candidatos").

> **Conclusión honesta:** con un N etiquetable (60–100) ningún test de significación va a ver diferencias de 5 pp entre dos fusiones sobre los mismos candidatos. Por eso la regla GANA de §6 es una **regla de decisión con tasas de error declaradas** y no una afirmación de significación

## 4. Filas en rojo (sin maquillar)

- **A0 pierde en hit@5 contra tres celdas**: `base:rrf` 65, `solape:sellado` 66, `solape:rrf` 67, frente a 64. Todas dentro del ruido (Wilson ±9 pp), ninguna GANA, pero **A0 no es la mejor celda de la tabla en ninguna métrica**.
- **A0 pierde en hit@1 y MRR@10 contra vector puro**: hit@1 29/92 vs 37/92; MRR 0,4730 vs 0,5144 (ΔMRR +0,041, IC95 [−0,008, +0,090]). En strict, la pareada vector vs A0 es 2/4 y RRF vs A0 es **3/0**. La fusión sellada mete notas léxicas por encima de la correcta: en las 3 filas donde vector arregla y A0 rompe, A0 tiene la nota relevante en el puesto 6 de las tres. Mecanismo visto en las filas: con un solo candidato FTS, `f/f_max = 1` le da 0,6 fijo y salta por encima de cualquier trozo vectorial ≤ 0,6 (c141); con muchos candidatos FTS de términos comunes, los 2–3 primeros ocupan el top (c107).
- **Estratos donde A0 cae**: `agent-search` 30/34 frente a 31/34 de vector y de RRF; `hard` 23/36 frente a 25/36 de solape (y 25/36 de solape:rrf). En `prompt` empatan todos a 11/22 salvo FTS (1/22).
- **hard-larga**: FTS 0/18 con candidatos = 0/18: en las paráfrasis largas la fusión es inerte y A0 ≡ vector (12/18 en ambos). En `hard-corta` vector 9/18 vs A0 11/18: ahí la fusión ayuda.
- **In-sample**: 43/55 (lenient = strict: las 55 no tienen aceptables) frente al 49/55 de `metrics-engine-hybrid-b0.0-e0.6.md` (otro binario, KB de 138 notas). El pre-registro prohíbe compararlos (§6 R1, cita arriba), y lo cumplo: solo constato que el "48/55" del README y de `docs/arquitectura.md` §6 describe otra KB y otro binario (hand-off a B, plan Task 11).
- **Corpus negativo**: 54/55 nulas devuelven algo en el top-5 con umbral 0,40 (55/55 con 0,35). El umbral no abstiene.
- **Latencia**: el intento 1 se descartó (§5). Los tiempos de rebuild (5.335 s base vs 2.752 s solape) no son comparables (carga de máquina) y ninguna regla los usa.

## 5. Latencia: dos intentos, descarte del primero

Hechos (ficheros `lat-*-intento1.json` y `lat-*.json`, mismo binario, misma query, `hyperfine --warmup 3 --runs 30`):

- Intento 1: `base` 30 runs estables a 1,911–1,987 s (p95 1,956 s); `solape` **bimodal por bloques contiguos**: los 10 primeros runs a 9,20 · 8,90 · 8,89 · 8,88 · 8,85 · 8,87 · 8,85 · 8,86 · 8,90 · 5,39 s, los 20 siguientes a 0,93–0,97 s (p95 8,899 s).
- Intento 2: `base` 0,988–1,041 s (p95 1,0240 s); `solape` 0,996–1,032 s (p95 1,0258 s). Ratio 1,002.
- Marcas de tiempo: capturas de `solape` 14:42 → intento 1 14:47–15:49 → intento 2 15:51 → `detalle.jsonl`/informe 15:52. Compatible con "descartado antes de correr el informe".

Veredicto: **descarte legítimo, con una salvedad de proceso.** Legítimo porque (1) el plan condiciona la medición a "Correr los tres seguidos, sin otra carga pesada en la máquina" y el intento 1 la incumple de forma observable: el `base` del intento 1 corre a 1,94 s, el doble de su valor estacionario (1,00 s) y **más lento que el modo rápido de `solape` en ese mismo intento (0,95 s)**, lo que no tiene explicación por el índice y sí por carga externa durante todo `base` y el primer tercio de `solape`; (2) el intento 2 sigue el protocolo del §7 sin cambios; (3) se conservaron ambos ficheros. La salvedad: el pre-registro **no fija un criterio de descarte** de series, así que esto es una desviación declarada, no una regla aplicada. Consecuencia sobre las decisiones: **ninguna**. R3 solo consulta el p95 si A4 GANA en retrieval, y no gana (NETO 2). Además el resultado de R3 es el mismo con cualquiera de los dos intentos: con el intento 1 el guard habría fallado (8,90 > 1,25 × 1,96) y con el 2 pasa; en ambos casos "se queda el troceado actual".

## 6. Descriptivos anotados por el orquestador: qué se sostiene y qué no permiten concluir

| (a)–(d) | ¿se sostiene? | base | qué NO permite concluir |
|---|---|---|---|
| (a) vector supera a A0 en hit@1 (37 vs 29) y MRR@10 (0,5144 vs 0,4730); IC95 ΔMRR cruza 0 | **sí**, cifras exactas | measured / held-out / N=92 | que vector sea "mejor": hit@5 63 vs 64 y pareada 3/4; MRR es veto y no criterio de adopción (§5). Tampoco que sea igual (§6: "NO GANA no significa 'son iguales'"). Es una señal de orden, no de cobertura |
| (b) el corpus negativo casi entero devuelve top-5 con 0,40 | **sí**: 54/55 (A0, vector, RRF, solape); 55/55 con 0,35; FTS 18/55 | measured / held-out / 55 nulas | que esos resultados sean "malos": las nulas lo son por inferibilidad (la nota no se identifica desde el texto), no por ausencia de contenido afín. Ni que haya que subir el umbral: el held-out queda consumido (§11) y el umbral es parámetro afinable |
| (c) FTS casi nulo en `prompt` y en `hard` largas | **sí**: fusión activa en 6/22 `prompt`, 0/18 `hard-larga` (12/18 en `hard-corta`, 29/34 `agent-search`); hit@5 FTS 1/22 y 0/18 | measured / held-out | que la fusión sea inútil: en `hard-corta` A0 11/18 vs vector 9/18 y en las 4 filas ARREGLA(A0) frente a vector la nota entra por FTS. Consecuencia de diseño ya prevista en §2.4 (AND implícito): el hook de memoria corre de hecho como vector puro |
| (d) in-sample 43/55 frente a 48–49/55 histórico | **sí como hecho**: 43/55 con binario 4f4d2a8 y snapshot S; 49/55 en `metrics-engine-hybrid-b0.0-e0.6.md` con otro binario y KB de 138 notas | in-sample (sesgado) · histórico no comparable | **nada sobre el binario**: cambian KB (138→174 notas, rotaciones a `archive/`), distractores y binario a la vez; el §6 prohíbe la comparación. Solo justifica actualizar el texto público que cita 48/55 y un item de backlog para explicar las 6 filas que dejaron de acertar (hoy sin diagnóstico por fila) |

## 7. Qué hace la Task 12 y qué va a spec aparte

- **Task 12: nada.** R2 no adopta RRF, R3 no adopta solape. Plan Task 11: "Si R2 y R3 no adoptan nada, **la campaña cierra aquí** y la Task 12 se marca `no aplica`". No se toca `buscador.rs` ni `trozos.rs`.
- **Spec aparte: ninguna.** R4 no se midió (D3 = no); "R4 GANA abre una spec de producción aparte" no aplica.
- **Held-out consumido** (§11): cualquier ajuste futuro de β, umbral, `K_C` o troceado necesita un held-out nuevo. Este no vale para afinar nada de lo que aquí se observa en rojo.

## 8. Erratas del pre-registro (anotadas aquí, el texto no se edita)

1. **Tabla §7, fila "X mejor 5 pp" (0,58 / 0,83)**: solo se reproduce con discordancia ≈7 % (a=0,06, b=0,01), no con la del 10 % que sugieren las filas vecinas; el supuesto no está declarado. Señalada por el consultor-gate; confirmada por mi re-derivación. Con discordancia 10 % la fila sería 0,57 / 0,74. No cambia D4 ni ninguna decisión.
2. **§7, "Ante +5 pp: ≤0,37 incluso con N=150"** (potencia McNemar): depende del supuesto de discordancia; con a=0,075, b=0,025 sale 0,40 a N=150. Cosmética.
3. **§10, "Binario de medición: pendiente"**: el commit quedó en `condiciones.md` (`4f4d2a8`), como el propio §10 prevé. No es errata, es un puntero.

## 9. Propuesta para el backlog (Task 11; aquí solo se propone, no se redacta)

- (a) **Orden dentro del top**: vector supera a A0 en hit@1/MRR y RRF a A0 en strict 3/0. Mecanismo concreto visto en filas: `f/f_max` con un único candidato FTS vale 1 → 0,6 fijo. Hipótesis para un diseño nuevo (no afinar sobre este held-out): normalización FTS que dependa de la fuerza absoluta o del número de candidatos. Requiere held-out nuevo.
- (b) **Abstención**: 54/55 nulas devuelven top-5. El hook inyecta casi siempre algo. Diseño de abstención (no umbral: calibración por distribución de scores, o gap top1–top2) y medición con corpus negativo propio.
- (c) **FTS para frases naturales**: AND implícito deja `prompt` y `hard-larga` sin fusión (§2.4). Candidato: OR con mínimo de términos, o consulta FTS solo con tokens raros. Held-out nuevo.
- (d) **Diagnóstico por fila de las 55**: qué 6 filas dejaron de acertar entre el 49/55 histórico y el 43/55 actual (KB vs binario). Es descriptivo, no decisión, y no lo permite este verdict.
- (e) **Solape**: 4/2 sin llegar a NETO 3; con dos misses por "el trozo desaparece". Si se retoma, medir con un paso distinto necesita held-out nuevo; el 20 % era un valor único declarado.
- (f) **Texto público**: README y `docs/arquitectura.md` citan 48/55 sin la cifra held-out (hand-off a la campaña B).
- (g) **Late chunking**: sigue viable (§2.6), no medido; sin prioridad derivada de C.

## 10. Disenso: qué busqué para objetar

- **Contra el informe (no encontrado):** recomputé todo sin `metricas.py` desde las capturas crudas; 0 discrepancias. Mi fidelidad propia (lista `hybrid` del binario vs mi fusión) da 0/147 en ambos índices. El `diff` del informe re-corrido es solo la sección S4, que recomputé y coincide.
- **Contra R1 (buscado a fondo, no encontrado):** el 4/3 frente a vector es el punto más frágil de la campaña. Abrí las 4 filas donde A0 arregla (c087, c117, c123, c143) buscando una nota aceptable omitida en el top-5 de vector que convirtiera el miss en hit: en c087 la rotación archivada que vector devuelve no contiene la auditoría; en c117 la única nota ≥0,40 de vector no menciona el segundo modelo; en c123 ninguna de las 5 contiene el patrón (una comparte un término con otro sentido); en c143 el rank 1 de vector no tiene el hecho, que vive en la esperada y en la rotación aceptable (que vector no devuelve). Misses reales. En sentido contrario sí encontré dos filas discutibles (c073: la bitácora viva del proyecto, en el top-5 de A0, cubre la query genérica; c141: una bitácora en el top-5 de A0 relata una instancia concreta de lo que la query pregunta). Ambas, de re-etiquetarse, favorecerían a A0 (4/2 o 4/1). No re-etiqueto: está prohibido (§11) y el criterio `labels.md:70` ("query genérica/estado → canon") respalda la elección del etiquetador en c073.
- **Contra R3 (buscado, no encontrado):** ¿hay una query con error de captura que deje el NETO en el borde? No: 0 errores. ¿Los 2 ROMPE son artefactos? No: en c129 y c115 el trozo que puntuaba desaparece con el paso 720 y ninguna otra nota del top-5 de solape lleva el hecho. ¿Los 4 ARREGLA dependen de aceptables? Tres sí (c124 canon, c135 y c141 bitácora/learning) y uno no (c145, la esperada). D5 fija lenient; en strict la pareada es 3/3. Lo declaro; no cambia el NO GANA.
- **Contra el descarte de latencia (buscado):** miré si la serie bimodal era intercalada (patología del índice) o contigua (episodio externo): contigua, 10 lentos seguidos y 20 rápidos. Miré si `base` del intento 1 era comparable: no, corre al doble de su valor estacionario. Miré si el descarte podía haber salvado a solape: no, R3 no llega al guard. Objeción que queda en pie: no había criterio de descarte pre-registrado.
- **Contra el gold (heredado, declarado):** S2 (sesgo léxico por citas literales, 0/92 con única señal literal según el consultor; sin excluir filas). 44/92 filas con aceptables hacen lenient load-bearing; las 6 correcciones del consultor entraron antes de congelar y el sha lo confirma. En la muestra, 3 de 6 hits dependen de un aceptable.
- **Contra el pre-registro:** la fila "+5 pp" del §7 (errata confirmada) y la ausencia de criterio de descarte de latencia. Ninguna afecta a una decisión.
- **Lo que no pude comprobar:** que nadie miró los agregados antes de descartar el intento 1 (solo tengo marcas de tiempo compatibles); la fidelidad de los brazos `vector`, `fts` y `rrf` frente al binario (no hay oráculo para ellos por diseño; derivan de las mismas listas capturadas que sí pasan el oráculo); y no re-corrí capturas ni hyperfine (prohibido).

## 11. Decisiones firmadas que este verdict honra

- **D4**: NETO ≥ 3 y ARREGLA ≥ 2·ROMPE con veto ΔMRR, aplicados literalmente.
- **D5**: lenient decide; strict reportado como descriptivo (§4).
- **S2**: sesgo de citas literales declarado (§10), sin excluir filas.
- **S4**: desglose `hard` corta/larga reportado como descriptivo (§4), sin peso.
- **Overlay fila 13**: reportado con y sin, cifras idénticas.

**Resultado de la campaña C: R1 GENERALIZA · R2 se queda la fusión actual · R3 se queda el troceado actual · R4 no medido · Task 12 no aplica · sin spec aparte.**
