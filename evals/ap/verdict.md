# Veredicto: `Ap` (selección de reglas de proyecto, spec §7.2 y §7.3)

Adjudicado con `preregistro.md` y la errata E1, y con la incidencia E2 (`erratas.md`). Tanda del 2026-10-25: 280 corridas, `claude` 2.1.310, `claude-sonnet-5-5`, KB en `552f06a`, 19,24 USD. Sin breaker y sin fugas. Comando: `python3 evals/ap/evaluar-ap.py ~/.cache/exo-campana-m evals/ap` (`resultado.json`).

## Veredicto: **`Ap` PASA**

| | valor |
|---|---|
| N (sondas de proyecto cubiertas) | 16 (N_MIN 16, k = 5) |
| Réplica de `ap1` | 15 de 16 pasan (r = 13); 0,825 por corrida (≥ 0,80) → **sí** |
| Primario pareado Δ = `ap` − `ap1` | **+0,000**, IC95 [−0,087, +0,100]; percentil 97,5 > 0 → no cae |
| Secundario absoluto | 15 de 16 pasan en `ap` (k = 13); aplica porque `ap1` ≥ s = 15 → no cae |
| Éxito por corrida | `ap` 0,825 · `ap1` 0,825 |

La condición «selección de proyecto» de §7.3 **no** se cumple. Las reglas de proyecto, sembradas y entregadas con el resto del bloque (4–10 de proyecto más 5 globales, 1,4–2,1 KB), se aplican igual que la regla sola. La cota inferior del Δ es −0,087: este brazo descarta una dilución de más de ~9 puntos por corrida. No demuestra que sea cero.

## Sensibilidades y descriptivos (no adjudican)

- **Sin las 27 corridas que no arrancaron** (E2): son 144 corridas cubiertas. `ap` 0,925, `ap1` 0,912, Δ +0,013 [−0,050, +0,088]. Los dos brazos vuelven al nivel de O (0,903), lo que confirma que el canal system prompt replica en estas sondas.
- **Completitud, con las 27 relanzadas** (2026-10-26, `corridas-e2/`; mismo `claude` 2.1.310, mismos bloques y pins; 27/27 con `result`, 2,47 USD, sin fugas): sustituyendo cada corrida fallida por su relanzamiento, `ap` 0,925, `ap1` 0,925, Δ **+0,000 [−0,050, +0,062]**, 15 de 16 sondas pasan en los dos brazos. Por tipo: iniciación `ap` 0,92 frente a `ap1` 1,00 (n = 5); restricción 0,93 frente a 0,89 (n = 11). Las no cubiertas siguen en 2/40. Es la mejor estimación del efecto; el veredicto sigue siendo el de la regla sellada.
- **Bootstrap por proyecto** (13 proyectos): Δ +0,000 [−0,080, +0,093].
- **Listas de proyecto con ≥ 8 reglas** (n = 10): Δ −0,020; 9 de 10 pasan en los dos brazos. Con la lista casi llena (`CAP` 10) no se ve dilución.
- **Por tipo** (`tipos.tsv`, etiquetado a ciegas): iniciación, n = 5: `ap` 0,80, `ap1` 0,84, Δ −0,040. Restricción, n = 11: `ap` 0,84, `ap1` 0,82, Δ +0,018. La dirección es la que predice #193, pero con n = 5 no se distingue del ruido.
- **`g1-140`** (finnk-publica-react) falla en los dos brazos (0,20 y 0,00). Es la regla que no funciona por este canal, no dilución.

## El hallazgo que el gate no mide: lo no cubierto no se aplica

| grupo | sondas | `ap` (corridas que cumplen) | referencia |
|---|---|---|---|
| proyecto, cubiertas | 16 | 0,825 | `ap1` 0,825 |
| proyecto, **no cubiertas** | 8 | **2/40** | O 22/24 · A0 0/24 |
| transversales | 16 | 1 de 16 sondas con éxito > 0 | cobertura no interpretable (§7.2) |

Con la siembra actual, `exo rules` entrega bien lo que entrega, pero solo cubre 16 de las 24 decisiones de proyecto que A0 no acierta. Las 8 restantes vuelven al default: 3 porque la siembra eligió otras reglas del proyecto y 5 porque no hay nota o sección (`wisdom-paul`, `CV`, `viz-components`). **El cuello de botella del diseño A es la cobertura de la siembra, no la dilución.** Esto no invalida A según §7.3, pero cambia la prioridad: el siguiente trabajo útil es la selección (qué regla entra en cada proyecto y qué repos no tienen nota), no recortar el bloque.

## Limitaciones

- El efecto se mide a la longitud de lista desplegada (mediana de 8 reglas de proyecto), no a un bloque más largo.
- n = 16 sondas, un solo modelo y un solo dueño. Las 16 cubiertas son algo más fáciles que la base de 24 (E1).
- Hubo 27 corridas que no arrancaron por E2. La regla sellada las cuenta como no cumple en los dos brazos por igual, y la sensibilidad sin ellas da la misma conclusión.
