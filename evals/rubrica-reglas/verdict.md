# Veredicto: validación de la rúbrica de selección (criterio 2, spec §5.3)

Adjudicado con la regla sellada de la spec §5.3 (`kappa(b) ≤ 0` → la rúbrica pierde el criterio 2). Corrida del 2026-10-26: 53 reglas del universo sellado (`sello.json`, n = 53), `claude` 2.1.310, `claude-sonnet-5-5`, prompt `99455d65…`, KB en el commit sellado. Coste total: **1,34 USD** (`coste_total_usd=1.34085`), sin tope ni fallo de proceso. Comando: `python3 evals/rubrica-reglas/validar.py --etiquetas evals/rubrica-reglas/etiquetas-validacion.jsonl --universo evals/rubrica-reglas/universo.txt --pool <pool> --criba <criba> --salida evals/rubrica-reglas/resultado.json --sello evals/rubrica-reglas/sello.json` (`resultado.json`).

## Veredicto: **D se mantiene** (criterio 2 válido)

| | (b) etiquetador | baseline §5.2 (`git`/`codigo` → se queda) |
|---|---|---|
| kappa contra la criba A0 | **0,155** | 0,109 |
| acierto | 0,604 | 0,528 |
| analizadas | 53 de 53 (0 `sin_etiqueta`) | 53 |

`kappa(b)` = 0,155 > 0, así que la validación es válida y la rúbrica conserva el choque con el default como criterio 2.

## Lo que el veredicto no dice

- **El efecto es pequeño.** Un kappa de 0,155 es un acuerdo leve. Δkappa frente al baseline = +0,046, con IC95 [−0,247, +0,320]: no se distingue de §5.2. Es lo que la spec preveía con n = 53, y por eso el Δ es descriptivo y no decide.
- **El etiquetador se queda casi todo.** Marca `P`/`D` en 44 de 53 reglas (83 %), y de esas 26 son `no_cumple` (precisión de «se queda» 0,59, frente al 0,55 de la base). Como filtro de lo que entra aporta poco.
- **Lo que sí informa es `E` («sale»)**, y es justo lo que usa la rúbrica para elegir qué desplazar. De las 9 reglas marcadas `E`, 6 son `cumple` (0,67, frente al 0,45 de `cumple` en el universo). Con n = 9 es una pista, no una medida.

| choque | n | `no_cumple` | proporción |
|---|---|---|---|
| P (paso extra) | 30 | 18 | 0,60 |
| D (prohíbe el default) | 14 | 8 | 0,57 |
| E (el repo lo encarna) | 9 | 3 | 0,33 |

- **Cita no encontrada en 12 de 53** (`citas_no_encontradas`). En esas reglas el etiquetador no tenía la cita en la prosa, así que no hubo nada que recortar. Cuadra con el 41/53 medido en la review de la spec.

## Limitaciones

- n = 53, un solo modelo y un solo dueño. La verdad es la criba A0 de la campaña M, con el sesgo que la spec §8 declara: un `cumple` puede deberse a que la tarea no empujaba en contra.
- Forma débil del criterio: el etiquetador ve la regla y la nota, no el repo ni la sesión. En producción `/document` tiene la sesión y aplica además el veto.
- Coste por llamada: 1,34 USD / 53 ≈ 0,025 USD, por encima de los 0,016 USD del humo de aislamiento, porque el prompt lleva la nota.

## Siguiente paso

Hacer el re-corte de las 8 notas al CAP con esta rúbrica: etiquetar las candidatas que faltan, hacer el control de 20 al azar, revisar las etiquetas y reescribir las secciones. Según la spec §5.4, esto va en su propio plan, junto con las líneas `kb:` y el aviso de ids desconocidos.
