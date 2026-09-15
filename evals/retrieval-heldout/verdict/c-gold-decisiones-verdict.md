# Verdict del consultor-gate — campaña C: gold del held-out y decisiones D4/D5/S2/S4

- **Fecha**: 2026-09-14T00:43+02:00
- **Rol**: consultor-gate delegado (régimen de gates delegado, precedente `evals/retrieval-fase0/verdict/labels.md:4`: "consultor-gate (régimen §8, adjudicación en lugar de Paul)"). No participé en ninguna fase de C. Propongo; firma Paul.
- **Objeto**: `$PRIV/gold.jsonl` (147 filas), package `.superpowers/fabrica/packages/c-retrieval-heldout-gold.md`, decisiones D4, D5, S2, S4.
- **Método**: solo filesystem sobre `$PRIV/kb-snap/` (Read/grep), validador oficial y scripts propios de cálculo. **Cero** `exo`/`kbx`/basic-memory, cero `captura.py`/`metricas.py`, cero lectura de `muestra.jsonl`/`pool.jsonl`/`hard-candidatas.jsonl`/`humo-cap.jsonl`, cero escritura en `gold.jsonl`, el pre-registro, el plan o el package. Este fichero no contiene texto de queries, prompts, `notes` ni contenido de notas de la KB; el detalle por fila con permalinks vive en `$PRIV/consultor-gate.md`.

## (a) Verificación primaria

```
python3 evals/retrieval-heldout/harness/valida_gold.py --gold $PRIV/gold.jsonl --kb $PRIV/kb-snap --in-sample $PRIV/in-sample-55.jsonl
→ exit 0 · {"filas": 147, "no_nulas": 92, "nulas": 55, "no_nulas_por_estrato": {"prompt": 22, "agent-search": 34, "hard": 36}, "con_acceptable": 39, "errores": 0}
sha256sum $PRIV/gold.jsonl → 1b21bbddd2e402ccb3dab5dd8253774c9f51749d43f6523d061fe2dd7f5fe148   (coincide con el package)
git -C $PRIV/kb-snap rev-parse HEAD → 885246df3a428fa3895c209030eb90a8e254005f   (coincide con el package y con kb-snap.commit)
```

Comprobaciones propias adicionales (script en scratchpad, solo recuentos):

- 174 notas `.md` en el snapshot, 174 permalinks únicos de frontmatter (+2 líneas `permalink:` embebidas en el cuerpo de un plan, ya señaladas en `labels.md`); los 92 `expected` y los 33 `acceptable` existen exactos.
- Ninguna query supera 1.500 caracteres (máxima: c027, 1.328) ni empieza por `-`: el filtro de §7 del pre-registro se cumple. 0 duplicados de query normalizada dentro del gold.
- Diff `gold.pre-verificacion.jsonl` → `gold.jsonl`: 41 filas cambiadas, exactamente las 41 `CORREGIR` del verificador (19 → null, 6 cambian `expected`, 16 añaden/cambian `acceptable`). Aplicación literal confirmada.
- `expected` en `archive/`: prompt 1/22 · agent-search 9/34 · hard 14/36.
- Sub-tipos de `hard`: 18 cortas (ids impares c113…c147, ≤5 palabras) · 18 largas (ids pares c112…c146, 16–30 palabras, forma de pregunta). Con `acceptable`: 10/18 cortas, 6/18 largas.
- Filas revisadas contra el snapshot: las 55 nulas, las 39 con `acceptable`, las 41 cambiadas, las 20 al azar del package y las 3 más flojas (c066, c012, c065); además todas las `hard` (36) y todas las `agent-search` (39). Total: 147/147 miradas, con lectura de contenido en las ~60 con alguna duda.

## (b) GATE del gold

**RECHAZADO** por 6 filas: **c019, c114, c124, c140, c142, c143**. Motivo único: `acceptable_permalinks` omitidos bajo la misma política que el verificador aplicó en sus 16 correcciones de `hard` (c113, c115, c116, c119, c122, c125, c126, c129, c133, c134, c135, c141, c145, c146, c147 y c102): existe una segunda nota con **el mismo contenido en entrada o sección propia** (canon↔bitácora, bitácora viva↔rotación archivada, o dos rotaciones archivadas solapadas) que el verificador marcó `CORRECTO`/"Único" sin serlo. Criterio del pre-registro §3: "Es el criterio de `evals/retrieval-fase0/verdict/labels.md:33`, fila 13: 'un retriever que devuelva cge-bitacora sería castigado siendo razonablemente correcto'". Y `labels.md:70`: "query genérica/estado → canon; query histórica/detalle fechado → bitácora" — en las seis, el `expected` respeta esa regla y se conserva; lo que falta es el otro lado del par.

Por qué bloquea y no es cosmético: D5 (abajo) decide en modo lenient; con NETO ≥ 3 como umbral, 6 filas (6,5 % de las 92) con conjunto de relevantes incompleto bastan para fabricar o borrar un GANA, y el sesgo no es simétrico: castiga al brazo que devuelva la copia no listada (en c019, la rotación archivada tiene 6 menciones del hito frente a 2 en la bitácora viva: un brazo léxico irá al archivo y se contará como fallo). Coste de la corrección: edición de 6 líneas + `valida_gold.py`; no exige re-etiquetar ni nueva pasada de juicio.

Clase de corrección por fila (permalinks exactos en `$PRIV/consultor-gate.md` §A):

| id | estrato | corrección |
|---|---|---|
| c019 | prompt | `acceptable` = las dos rotaciones archivadas solapadas que llevan la entrada fechada completa del hito; se retira la ficha canon (una mención en lista) |
| c114 | hard | `acceptable` += ficha canon del proyecto (bullet propio con el caso) + bitácora viva del proyecto (sección propia del caso). La propia nota `expected` remite a esa ficha para el caso entero |
| c124 | hard | `acceptable` += canon de perfil (bullet propio con la cita literal del patrón) |
| c140 | hard | `acceptable` += rotación archivada de doctrina (mismo texto, fuente fechada del learning) |
| c142 | hard | `acceptable` += bitácora viva de desarrollo agéntico (dos entradas propias con el mismo hecho y la misma regla) |
| c143 | hard | `acceptable` += rotación archivada de doctrina (sección propia con el mismo patrón) |

Simulación de las 6 correcciones sobre una copia en scratchpad (el gold real no se ha tocado): `valida_gold.py` exit 0, `{"filas": 147, "no_nulas": 92, "nulas": 55, "con_acceptable": 44, "errores": 0}`, sha256 esperado tras aplicarlas literal **`614ae599c6f9d9500b66636644970fa2b009362f9eb124b008a3222376c43a75`**. Si el sha resultante coincide, no hace falta otra ronda de consultor: **GATE APROBADO condicionado a ese sha**. Si Paul prefiere no aplicarlas, el gold actual es aprobable con la salvedad declarada en el verdict final de que 6 filas subestiman lenient en la dirección canon/archivo.

Recomendadas, no bloqueantes (misma clase pero con más margen de juicio; detalle en §B del privado): c013, c020, c065, c141 (segundo aceptable), c144. Sin objeción al resto: las 55 nulas se sostienen por inferibilidad desde el texto solo (comprobado con grep de sus términos: 0 hits para los identificadores de c048, c061, c108; ≥3 notas sin dominante para c021, c025, c041, c042, c062, c078); las 6 filas con `expected` cambiado (c036, c053, c065, c079, c104, c125) son correctas o defendibles; los 33 `acceptable` heredados no son comodines (cada uno contiene el hecho preguntado, no solo el tema).

## (c) Decisiones

### D4 — criterio de adopción: **(β) NETO ≥ 3 y ARREGLA ≥ 2·ROMPE, con veto de MRR**

Cita (pre-registro §6): "**Regla GANA (una sola, para todos los candidatos):** `NETO ≥ 3` **y** `ARREGLA ≥ 2·ROMPE` **y** el límite superior del IC95 de ΔMRR@10 (X − A0) es `≥ 0`". Y §7: "la regla GANA de §6 es una **regla de decisión con tasas de error declaradas** y no una afirmación de significación (Webber, Moffat & Zobel, CIKM 2008)".

Re-derivación propia con **N = 92** (script `d4_caracteristicas.py`: suma exacta multinomial sobre (ARREGLA, ROMPE); sin veto MRR, como la tabla de §7). Primero calibré los supuestos (a = P(ARREGLA), b = P(ROMPE) por query) para reproducir la tabla de §7 en N=60/N=100: "igual, disc 5 %" a=b=0,025 (0,07/0,13 ✓); "igual, disc 10 %" a=b=0,05 (0,15/0,16 ✓); "+8 pp" a=0,10, b=0,02 (0,81/0,93 ✓); "+10 pp, disc 16 %" a=0,13, b=0,03 (0,86/0,93 ✓); "+5 pp": la tabla se reproduce con a=0,06, b=0,01 (disc 7 %), no con disc 10 %; lo declaro como supuesto no explícito del §7.

| Escenario verdadero | (3,2) N=60 | (3,2) **N=92** | (3,2) N=100 | (α) NETO≥0, N=92 | (γ) McNemar p<0,05, N=92 | (4,2) N=92 |
|---|---|---|---|---|---|---|
| X igual a A0, disc 5 % | 0,07 | **0,12** | 0,13 | 0,60 | 0,00 | 0,05 |
| X igual a A0, disc 10 % | 0,15 | **0,17** | 0,16 | 0,57 | 0,01 | 0,12 |
| X igual, disc 16–20 % | — | **0,12–0,09** | — | 0,55 | — | 0,12–0,09 |
| X peor 5 pp | 0,01 | **0,01** | 0,00 | 0,08 | 0,00 | 0,00 |
| X mejor 5 pp (disc 7–10 %) | 0,58 | **0,72–0,80** | 0,74–0,83 | 0,96 | 0,21 | 0,63 |
| X mejor 8 pp | 0,81 | **0,92** | 0,93 | 0,99 | 0,51 | 0,88 |
| X mejor 10 pp, disc 16 % | 0,86 | **0,92** | 0,93 | 1,00 | 0,61 | 0,92 |

Lectura: con N=92 la regla está a un punto de las cifras de N=100; la probabilidad de adoptar un candidato peor sigue ≈0. Con ROMPE = r la regla exige ARREGLA ≥ max(r+3, 2r): 0→3, 1→4, 2→5, 3→6, 4→8.

Trade-off asumido y por qué no (α), (γ) ni (4,2): (α) adopta un candidato **igual** el 55–60 % de las veces — en R2 eso es cambiar la fusión de producción sin ganancia medida, y §6 dice "**Si nada gana, el resultado válido es 'se queda como está'**". (γ) ve una mejora real de 8 pp solo el 51 % de las veces; la campaña cerraría "sin cambios" por diseño (§7: "ningún test de significación va a ver diferencias de 5 pp"). (4,2) baja el falso positivo de 0,12 a 0,05 pero cuesta 4 puntos de potencia a +8 pp y cambia los números escritos en §6 sin que el N real lo justifique (§9: "Los valores 3 y 2 son la recomendación de D4"). Además, para R2 la discordancia real es menor que la de la tabla: §2.4 "la fusión solo puede cambiar el ranking en las queries con al menos un candidato FTS", y el estrato `prompt` (22 filas en frase natural) apenas los tiene; menos discordancia ⇒ menos falso positivo (fila "disc 5 %").

### D5 — relevancia para decidir: **lenient decide, strict descriptivo; overlay de la fila 13: sí**

Cita (pre-registro §3): "**Relevantes de una fila:** en modo *lenient*, `{expected} ∪ acceptable_permalinks`; en modo *strict*, solo `{expected}`" y "Cada uno exige en `notes` una frase que empiece por `aceptable:` y diga por qué un usuario razonable quedaría servido. Es el criterio de `labels.md:33`, fila 13". §5: descriptivo "hit@5 en el modo que D5 no elija". Overlay: §3 "Las 55 se reportan con y sin overlay. `evals/retrieval-fase0/gate.md` y los números históricos de M0/M2 no se tocan"; `labels.md:72`: "Si en fases posteriores el eval admite `acceptable_permalinks` secundarios, empezar por ahí".

Trade-off asumido: lenient hace que la calidad de los `acceptable` sea load-bearing (44/92 filas tras la corrección; 16/36 en `hard`). Por eso el gate de arriba es RECHAZADO y no "aprobado con notas". Strict decidiría sobre una distinción que el snapshot no sostiene: 14/36 `hard` tienen `expected` en `archive/` con copia viva o rotación solapada (verificador §0 y §6.3), así que strict mediría "¿devuelve la rotación que el autor de la query miró?" en vez de "¿sirve al usuario?". Overlay sí: el in-sample "no decide" (§6 R2: "Las 55 in-sample se reportan como pareada **sesgada en contra de RRF**… y no deciden"), reportar ambas versiones no cuesta nada y cierra la observación abierta de `labels.md:72`.

### S2 — citas literales de prompts dentro de la KB: **(i) declarar como sesgo, sin tocar filas**

Cita (pre-registro §7): "**Sesgo residual declarado:** etiquetar con grep favorece la coincidencia léxica, es decir, a A2. El estrato `hard` y la verificación adversarial (Task 4) lo mitigan, pero no lo anulan." La fuga prompt→KB es un sub-caso de ese sesgo ya declarado. §6 R2: "RRF y A0 comparten la misma admisión de candidatos (el umbral se aplica antes de fusionar): solo difieren en el orden" — la ventaja léxica entra igual en A0, A3 y A4.

Cuantificación propia (script `s2_literal.py`: n-gramas normalizados de cada query contra el texto normalizado de su nota esperada y del resto del snapshot): filas no nulas con un fragmento literal de **≥ 4 palabras** en la nota esperada: **5/92** (c009, c051, c093, c098, c110); con ≥ 6 palabras: **0/92**. Las 5 son nombres de entidad (un stack nombrado, títulos de retos), no muletillas de prompt, y en 4 de ellas el fragmento aparece en ≥ 2 notas. Las dos filas que el verificador señaló como fuga (c012, c041) ya son null por inferibilidad, y c013 no comparte ningún fragmento literal ≥ 4 palabras con su nota. Filas no nulas cuya **única** señal sea una cita literal del prompt: **0**. (ii) no tendría nada que excluir y exigiría otra pasada de juicio; el trade-off asumido es que fragmentos de 2–3 palabras quedan sin medir, y se declara en el verdict final como parte del sesgo léxico de §7.

### S4 — desglose `hard` corta/larga: **(i) sí, descriptivo, declarado en §9 antes de congelar**

Cita (pre-registro §5): "**Descriptivas, sin peso en la decisión:** … hit@5 por estrato (`source`)". Un sub-tipo dentro de `hard` es la misma clase de métrica. No es un brazo nuevo (§11 prohíbe "añadir brazos después de congelar", no descriptivas antes de congelar).

Regla de asignación fijada aquí, sin leer queries: `hard-corta` = ids **impares** c113, c115, …, c147 (18 filas, ≤ 5 palabras); `hard-larga` = ids **pares** c112, c114, …, c146 (18 filas, 16–30 palabras). Discrepo del package en la mecánica: el plan Task 5 dice "Modify: preregistro (§9 y §10 únicamente; bloque de estado de la cabecera)", así que el desglose **no se añade a §5** sino como línea `S4` en §9 con esa regla literal. Trade-off: 18 filas por sub-tipo no deciden nada (Wilson ±20 pp); sirve para no mezclar la señal de "matching léxico de cabeceras" (verificador §6.4: 4 casi literales + 6 al borde, todas cortas) con la de paráfrasis real.

## (d) Disenso: qué busqué para objetar

- **Contra el verificador (encontrado):** 6 filas `CORRECTO`/"Único" con una segunda nota de contenido idéntico en entrada propia (c019, c114, c124, c140, c142, c143) y 5 más con margen (c013, c020, c065, c141, c144). En c114 la objeción sale de la propia nota esperada, que remite a otra nota para el caso entero. Su política de aceptables fue correcta pero aplicada de forma incompleta: 16 añadidos, ≥ 6 omitidos de la misma clase.
- **Contra el verificador (no encontrado):** intenté rescatar nulos a no-nulos. c041: "T7/T8" aparece en 4 notas, no en 1 (null correcto). c021: el identificador de modelo no existe en el snapshot y el clasificador nombrado se usa en ≥ 5 proyectos (null correcto). c025: el PaaS nombrado aparece en 12 notas (null correcto). c048: el script nombrado no existe; existe uno de nombre parecido con un bug distinto (null correcto). c046: hay una research que compara exo con otros frameworks de verificación, pero el texto no nombra exo (null correcto por inferibilidad). c012: null se sostiene por inferibilidad (fórmula de aprobación) con independencia de la fuga. c066: null; la única ancla léxica es una palabra de proceso, no un proyecto.
- **Contra el verificador (comodines):** busqué `acceptable` que fueran "misma familia de tema" sin el hecho preguntado. No encontré ninguno en los 33 heredados; el más flojo es c053 (una nota inbox de gotchas de otro tema hermano, defendible como aceptable, no como expected — que es justo lo que corrigió el verificador).
- **Contra los `expected` cambiados:** c036 (puerta del backlog en vez de capítulo elegido por sesión), c079 y c104 (mejor nota no buscada), c125 (canon por `labels.md:70`) — correctos. c053 y c065: defendibles; en c065 la ficha canon de exo tiene la decisión en una línea literal y sería mejor aceptable que la ficha del marketplace (recomendada, §B privado). c082 y c035 quedan como DEFENDIBLE con expected↔acceptable invertibles.
- **Contra el package (D4):** el package afirma "N real = 92, dentro de la tabla del pre-registro (entre N=60 y N=100)"; lo re-derivé en vez de interpolarlo y comprobé que la fila "+5 pp" de §7 no se reproduce con discordancia 10 % (sí con 7 %). No cambia la elección.
- **Contra el package (S4):** recomienda "añadir al pre-registro un desglose" sin decir dónde; el plan limita la Task 5 a §9/§10. Lo resuelvo poniéndolo en §9 con ids, no en §5.
- **Contra el package (recuentos):** dice "14 de 36 en `hard`" con aceptable; son 16/36 antes de mi corrección (c119 y c128 ya los tenían) y 20/36 después.
- **Contra el package (S2):** su argumento "el efecto favorece a A2, que es testigo" es incompleto: A2 no es candidato, pero R1 compara A0 contra A2 y un A2 inflado por fugas empuja R1 hacia INDETERMINADO/NO GENERALIZA. Lo cuantifiqué (0 filas con única señal literal) precisamente porque el argumento del package no bastaba solo.
- **Contra el package (D5):** convergo en (a) y overlay sí, pero no por su motivo ("devolver el canon vivo en vez del archivo cuenta como fallo" es descriptivo del síntoma); el motivo es §3 + `labels.md:33`, y la consecuencia es que el gate del gold tiene que ser más duro con los aceptables, no más blando.
- **Lo que no pude comprobar:** la corrección de las 45 no nulas fuera del alcance mínimo la hice por grep de términos y lectura de cabeceras, no leyendo cada nota entera; y ningún ranking de ningún brazo (prohibido), así que no sé cuántas de las 6 filas RECHAZADAS discordarían de hecho entre brazos.

## (e) Líneas para firmar

```
PROPUESTA (consultor fable, 2026-09-14T00:43:08+02:00) — pendiente de firma de Paul

GATE: GOLD RECHAZADO c019 c114 c124 c140 c142 c143 — acceptable_permalinks omitidos (segunda nota con el mismo contenido en entrada propia; corrección exacta por fila en $PRIV/consultor-gate.md §A). Tras aplicarlas literal: valida_gold exit 0 con con_acceptable=44 y sha256=614ae599c6f9d9500b66636644970fa2b009362f9eb124b008a3222376c43a75 ⇒ GATE: GOLD APROBADO 2026-09-14 sin nueva ronda de consultor.
D4 NETO>=3 ARREGLA>=2*ROMPE
D5 lenient overlay-fila13=si
S2 i
S4 i   (línea en §9: hit@5 descriptivo por sub-tipo hard-corta = ids impares c113…c147, hard-larga = ids pares c112…c146; sin peso en la decisión)
```
