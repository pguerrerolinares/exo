# Erratas de `Ap`

Cada errata queda sellada por el commit que la añade. Ninguna toca `preregistro.md`.

## E1 — k=5 con N desde 16, y descriptivos de tipo, posición y similitud (2026-10-25, antes de cualquier corrida)

**Motivo.** Antes de construir el harness se midió la cobertura con `exo rules` sobre la KB en `552f06a`: solo **16 de las 24** sondas de proyecto tienen su gold literal en el bloque desplegado. Es un dato del despliegue y no un resultado, porque no se ha lanzado ninguna corrida de `ap` ni de `ap1`. Con N = 16 el pre-registro solo reportaría (N_MIN = 18), y con k=3 la potencia del pareado contra dilución moderada sería de 0,65.

Las 8 que se caen y por qué:

| sonda | proyecto | motivo |
|---|---|---|
| `g1-136`, `g2-138`, `g2-143` | code-graph-engine, exo, foss-jam-kit | la siembra (9b57258) eligió otras reglas del proyecto |
| `g0-9`, `g1-14`, `g1-15` | wisdom-paul | `sin_nota`: la KB no tiene nota de proyecto para sí misma |
| `g1-170` | CV | `sin_nota`: excluido en la siembra |
| `g0-87` | viz-components | `sin_seccion`: excluido en la siembra |

Se descarta sembrar estas sondas: hacerlo sabiendo qué hay que cubrir convierte la cobertura en algo diseñado para el test.

**Cambios** (sustituyen a lo correspondiente de `preregistro.md`):

- **k = 5** réplicas por sonda en `ap` (40 sondas) y en `ap1` (las N cubiertas).
- Una sonda **pasa** si cumple en ≥ 3 de 5 réplicas (mayoría, como el ≥2 de 3 original).
- **N_MIN = 16.**
- Umbrales y potencia: `python3 evals/ap/umbral.py --potencia --k 5`. Sin `--k`, el script sigue reproduciendo byte a byte la tabla de `preregistro.md`. La réplica conserva además su cláusula de ≥ 0,80 por corrida. El pareado no cambia: es `delta_bootstrap` sobre la fracción de réplicas por sonda (ahora quintos), con el percentil 97,5.
- **Tope de gasto:** `K_TOPE_USD = 30` para esta tanda (280 corridas, ~25 USD a 0,09 USD por corrida).
- **KB fijada:** `wisdom-paul` en `552f06a`. Los bloques se generan desde ese commit y sus sha256 se sellan en `bloques.sha256`.

Tabla sellada:

```
q = P(sonda >= 3/5 | nula) = 0.9466  (Beta(2.86, 0.31))
N  | réplica Ap1 >= r | cae abs. si Ap < k | abs. solo si Ap1 >= s | P(no réplica) | α abs.
16 | 13 | 13 | 15 | 0.009 | 0.009
17 | 14 | 14 | 16 | 0.011 | 0.011
18 | 15 | 15 | 17 | 0.014 | 0.014
19 | 16 | 16 | 17 | 0.016 | 0.016
20 | 17 | 17 | 18 | 0.020 | 0.020
21 | 18 | 18 | 19 | 0.023 | 0.023
22 | 18 | 18 | 20 | 0.005 | 0.005
23 | 19 | 19 | 21 | 0.006 | 0.006
24 | 20 | 20 | 22 | 0.008 | 0.008

N  | P(cae): nula · moderada · fuerte · pierde3 · pierde5 | P(sin réplica | nula)
16 | 0.04 · 0.82 · 0.98 · 0.55 · 0.94 | 0.028
20 | 0.04 · 0.89 · 0.98 · 0.67 · 0.95 | 0.021
24 | 0.04 · 0.93 · 0.99 · 0.44 · 0.94 | 0.019
```

**Descriptivos nuevos** (no adjudican; salen de la revisión de literatura, ver #193):

- **Tipo de regla** (`tipos.tsv`): iniciación (I: obliga a una acción opcional, falla por omisión) frente a restricción (R: fija cómo se hace lo que ya se iba a hacer, falla por comisión). Lo etiquetaron a ciegas dos etiquetadores (sonnet y fable), que solo veían el texto de la regla, sin id ni resultados y en orden barajado. Acuerdo 0,95, κ = 0,886. Un desacuerdo se marca `ambigua`. Sobre las 40: 12 I, 26 R y 2 ambiguas. De las 16 cubiertas: **5 I y 11 R**. `Ap` reporta el Δ por tipo, y con 5 sondas I eso es descriptivo, no una prueba.
- **Posición** de la gold en su lista y **similitud máxima** con las demás reglas del bloque (`difflib.SequenceMatcher.ratio`), por sonda, en `cobertura.tsv`.
- **Si `Ap` pasa**, se reporta además la cota inferior del intervalo del Δ (percentil 2,5), para que «no cae» no se lea como «no hay dilución».

**Sesgo declarado.** Entre las 8 sondas que se caen hay dos de las duras del brazo O (`g1-136` y `g1-15`, las dos 2/3). Las 16 cubiertas son algo más fáciles que la base de 24, y la Beta se ajustó sobre las 24. Con una base más alta, el gate absoluto es algo más conservador, mientras que el pareado no depende del nivel. Ninguna de las 16 cubiertas tiene 0/3 en O; `g0-110` y `g0-66` tienen 1/3.
