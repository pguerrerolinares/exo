# Pre-registro: brazo `Ap` (selección de reglas de proyecto, #190)

Queda sellado con el commit que crea este fichero. Ese commit va antes de sembrar las reglas que medirá `Ap` y antes de cualquier corrida. Las erratas posteriores van a `evals/ap/erratas.md`; este fichero no se edita después.

- Spec: `docs/superpowers/specs/2026-10-25-reglas-literales-arranque-design.md` §7.2 y §7.3.
- Umbrales: `evals/ap/umbral.py`. Su salida, con `--potencia`, es la tabla de abajo y tiene que reproducirse byte a byte.
- Revisión previa: un consultor fresco sobre la propuesta de #190. Lo que cambió en esa revisión está en «Desviaciones».

## Qué adjudica

Si las reglas de proyecto, sembradas y recortadas al `CAP` tal como se despliegan, se siguen aplicando con las demás reglas delante, o si la selección las diluye. Es la condición «selección de proyecto» de §7.3: si `Ap` cae, el diseño A no compensa y el siguiente paso es E.

## Diseño fijo

- **Sondas:** las 40 de la campaña M, con su gold (`~/.cache/exo-campana-m/gold/s1`, con los checks de `g1-21` y `g0-43` corregidos en #185). Solo adjudican las 24 sondas de **proyecto** que salgan cubiertas. Las 16 transversales se reportan aparte y no adjudican (§7.2: la cobertura de las globales no es interpretable).
- **Las 24 de proyecto:** `g0-101 g0-110 g0-147 g0-149 g0-66 g0-67 g0-87 g0-9 g0-96 g1-110 g1-136 g1-14 g1-140 g1-147 g1-15 g1-157 g1-170 g1-82 g2-138 g2-141 g2-143 g2-154 g2-160 g2-186`. Las duras en el brazo O son `g0-110` (1/3), `g0-66` (1/3), `g0-101`, `g1-136` y `g1-15` (2/3).
- **Cubierta:** la regla gold de la sonda aparece literal en el bloque que `exo rules` entrega para el repo de la sonda, después del recorte al `CAP` vigente. N = número de sondas de proyecto cubiertas.
- **Brazos** (k=3, canal `arp` de `evals/ablacion-k/harness/correr.sh`: `--append-system-prompt-file` con `claude-md.md` y el bloque, con las restricciones de `a0`):
  - `ap`: el bloque desplegado del repo de la sonda, con globales y de proyecto, renderizado byte a byte como `bloque()` de `plugins/exo/hooks/register.ts`. Corre en las 40 sondas.
  - `ap1`: solo la regla gold, en su lista y con el mismo framing. Corre en las N cubiertas. Es el control de canal, porque el canal system prompt no se ha medido nunca en estas sondas.
- **Unidad:**
  - Para el gate pareado, la fracción de réplicas que cumplen (`check.rc == 0`) en cada sonda.
  - Para el gate absoluto y la réplica, una sonda **pasa** si cumple en ≥2 de 3 réplicas.
- **Modelo nulo:** el éxito por corrida p_i de cada sonda sigue una Beta(2,86, 0,31). Está ajustada por momentos a las 24 sondas de proyecto del brazo O de M: 65/72 = 0,903, distribución 19/3/2/0 en 3/3, 2/3, 1/3 y 0/3, y ρ = 0,24. De ahí, q = P(pasa | nula) = 0,9353.

## Réplica (antes de adjudicar nada)

Hay réplica si **las dos** cosas se cumplen:

- `ap1` tiene ≥ r(N) sondas que pasan;
- `ap1` tiene ≥ 0,80 de éxito por corrida, contando las 3·N corridas.

Sin réplica, el canal no está al nivel de O en estas sondas. Entonces se reporta y no se adjudica: un canal débil no se lee como dilución. Por ruido, la réplica falla un 2–3 % de las veces (tabla).

## Gates (solo con réplica)

- **Primario, pareado:** d_i = fracción de `ap` − fracción de `ap1` en cada sonda cubierta. Se calcula con `delta_bootstrap(ap, ap1)` de `evals/campana-m/analizar.py` (semilla 20261024, 10.000 remuestreos). `Ap` **cae** si el límite superior del intervalo, el percentil 97,5, es < 0. Equivale a un test unilateral al 97,5 %.
- **Secundario, absoluto:** `Ap` **cae** si menos de k(N) sondas pasan en `ap`. Solo adjudica si `ap1` tiene ≥ s(N) sondas que pasan, es decir, si el control está claramente a nivel. Este gate caza la dilución de tipo todo o nada, como g0-33 en techo-reglas-3 (2/2 → 0/4), que el pareado ve peor.
- **Regla:** `Ap` cae si cae el primario **o** el secundario. Si no cae ninguno, `Ap` pasa.

## Bandas sin salida, cerradas

- **`ap1` entre r(N) y s(N)−1:** hay réplica, pero el control está en el borde. Solo adjudica el pareado. Así, una diferencia de 1–3 sondas por ruido no basta para que `Ap` caiga.
- **Sin réplica y el pareado muestra dilución:** hay canal débil y dilución a la vez. Se reporta y no se adjudica, igual que en techo-reglas-3.
- **N < 18:** la potencia baja de 0,6 con todos los modelos. Solo se reporta.

## Tabla sellada (`python3 evals/ap/umbral.py --potencia`)

```
q = P(sonda >= 2/3 | nula) = 0.9353  (Beta(2.86, 0.31))
N  | réplica Ap1 >= r | cae abs. si Ap < k | abs. solo si Ap1 >= s | P(no réplica) | α abs.
18 | 14 | 14 | 16 | 0.005 | 0.005
19 | 15 | 15 | 17 | 0.006 | 0.006
20 | 16 | 16 | 18 | 0.008 | 0.008
21 | 17 | 17 | 19 | 0.010 | 0.010
22 | 18 | 18 | 20 | 0.012 | 0.012
23 | 19 | 19 | 21 | 0.014 | 0.014
24 | 20 | 20 | 22 | 0.017 | 0.017

N  | P(cae): nula · moderada · fuerte · pierde3 · pierde5 | P(sin réplica | nula)
18 | 0.03 · 0.68 · 0.96 · 0.43 · 0.90 | 0.034
21 | 0.03 · 0.77 · 0.98 · 0.43 · 0.89 | 0.029
24 | 0.03 · 0.81 · 0.97 · 0.44 · 0.86 | 0.024
```

- La regla de cada umbral:
  - r(N) es el máximo r con P(`ap1` < r | nula) ≤ 0,025.
  - k(N) es el máximo k con P(`ap` < k | nula) ≤ 0,025.
  - s(N) = ⌊q·N⌋.
- La primera tabla es exacta. La segunda simula la regla entera con 2.000 simulaciones y 1.000 remuestreos por celda (semilla 20261025). En esas filas, «P(cae)» cuenta solo cuando hay réplica.
- Las columnas de la segunda tabla:
  - «Moderada» y «fuerte» multiplican p por 4,5/5,5 y 2,7/5,5 (el modelo de techo-reglas-3).
  - «PierdeX» deja X reglas a p = 0,05 y las demás intactas.
  - «P(sin réplica | nula)» es la probabilidad de que la réplica falle sin dilución, por ruido.

## Se reporta siempre (descriptivo, no adjudica)

- N, y qué sondas de proyecto quedan fuera de la cobertura y por qué. Las que se espera que desborden son las de exo, kbx, wisdom-paul, llm-apps y cge, los proyectos con la lista llena.
- La longitud del bloque, en reglas y en bytes, de cada sonda cubierta, y el estrato de listas de ≥ 8 reglas.
- Las transversales en `ap`: su cobertura y su aplicación.
- El éxito por corrida de `ap` y de `ap1`, y la mediana de `tokens_in`.
- Independencia: las sondas del mismo proyecto comparten bloque (24 sondas en 17 proyectos). El bootstrap remuestrea por sonda. Como sensibilidad se reporta también un bootstrap por proyecto, que no adjudica.

## Limitaciones declaradas

- `Ap` mide la dilución a la longitud **mediana** de lista, no al `CAP` lleno. Las sondas cubiertas no se eligieron al azar: §5.2 prioriza las reglas que A0 no cumple, así que las sondas que se quedan fuera son justo las de listas llenas. El resultado es una cota optimista. techo-reglas-3 tiene la misma limitación.
- Si se pierden del todo 3 de 24 reglas, `Ap` lo caza en ~0,44 de los casos. Esa es la dilución que este brazo no ve.
- Las p_i salen de 3 corridas por sonda. Si la sobredispersión real es mayor que la de la Beta ajustada, el α del absoluto sube. El del pareado no depende del nivel del canal.

## Condiciones que invalidan este diseño

- `ap1` tiene < 0,80 de éxito por corrida. Ya es cláusula de réplica: no se adjudica, y lo siguiente es volver a techo-reglas-2 para separar el efecto del canal del efecto del framing.
- N < 18.
- La dilución depende del tipo de regla (#193: las de proceso frente a las mecánicas) y las sondas cubiertas son casi todas mecánicas. Entonces `Ap` puede pasar aunque en producción la selección diluya. Ningún brazo de esta spec lo mide.

## Desviaciones respecto a #190 y a la propuesta inicial

- **Base:** #190 pedía calibrar contra el 0,55 por corrida de techo-reglas-2. Ese número sale de otras 10 tareas: con esa base, q = 0,575 y la dilución moderada no se detecta (potencia ≤ 0,48). La base pasa a ser el brazo O de M en las 24 sondas de proyecto, que son las mismas sondas. La diferencia de canal la cubre `ap1`.
- **Umbral absoluto como único gate:** se descartó, porque depende mucho del nivel del canal. Con una pérdida de canal de ×0,90 sin dilución, el α sube a 0,20.
- **Pareado:** el descarte inicial (potencia 0,20–0,30) venía de convertir cada sonda en sí/no antes de emparejar. Sobre fracciones, el pareado es el gate primario.
- **Umbral de réplica:** r(N) y no el umbral de caída anterior (21 de 24), que hacía fallar la réplica un 4–7 % por puro ruido.

## Pendiente antes de lanzar

- Los brazos `ap` y `ap1` en `correr.sh` (`modo=arp` con su `K_REGLA_FILE` por sonda) y el script que genera los bloques desde `exo rules`. No existen todavía.
- `evaluar-ap.py`, que aplique las reglas de arriba tal cual y use `delta_bootstrap` de `analizar.py`.
- Ensayo sin `claude` (`K_ENSAYO=1`), versión de `claude` fijada en `claude-version.txt`, cero re-intentos y el breaker de M.
- Coste estimado: 120 corridas de `ap` y 3·N de `ap1` (≤ 72), unos 17 USD a 0,09 USD por corrida.
