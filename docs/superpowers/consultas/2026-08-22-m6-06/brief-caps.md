# Brief — Consultor Fable: ¿"subir el cap" es un apaño sistémico?

## Rol

Eres consultor Fable del régimen de gates delegado del proyecto exo. Paul ha
levantado una objeción de fondo a mitad de la implementación de M6-06, y quiere que
la investigues antes de que yo aplique el parche fácil.

**No toques código de producción, no commitees, no edites specs.** Scripts de
medición en tu scratchpad, sí. Veredicto en
`docs/superpowers/consultas/2026-08-22-m6-06/consultor-caps.md`.

## La objeción de Paul, literal

> "los caps no es la primera vez que dan problema y siempre hemos subido, no es un
> apaño esto? debe existir mejores soluciones"

Tiene toda la pinta de tener razón, y por eso no quiero adjudicarlo yo: yo soy quien
iba a proponer subir el cap.

## El caso concreto

M6-06 es un hook `UserPromptSubmit` que inyecta hasta 3 punteros a notas de la KB en
cada prompt sustantivo. El bloque tiene un cap de **1024 B** fijado por la spec
(`specs/2026-08-22-m6-06-recall-punto-de-uso-design.md` §2.3/§D6), justificado así:
*"bloque real medido con 3 hits = ~970 B. El cap protege del outlier, no raciona el
presupuesto"*.

**Medido por mí contra el engine real (10 queries, 27 hits reales):** el bloque con
3 hits tiene mediana **1137 B** (rango 950–1236). O sea, el `~970 B` de la spec es
el **mínimo** de la distribución, no el típico. Consecuencia: con cap 1024, **7 de
cada 10 queries entregan 2 punteros en vez de 3**, y el cap está racionando el
presupuesto — exactamente lo que la spec decía que NO debía hacer.

### Anatomía del bloque (medida, 27 hits reales)

| componente | bytes | % del bloque de 3 hits |
|---|---|---|
| snippets (×3) | 567 | 50% |
| prefijo `/home/paul/Documentos/proyectos/kb-demo/` **repetido en cada hit** | 132 | 12% |
| cabecera + footer | 132 | 12% |
| títulos (×3) | ~99 | 9% |

Dos datos que me parecen la clave, y que quiero que verifiques y explotes o descartes:

- **El prefijo de la KB va tres veces**: 44 B por hit, 132 B por bloque.
- **El título es redundante con el nombre de fichero en 26 de 27 hits** (p. ej.
  `- /home/…/log/kbx-bitacora.md — kbx-bitacora`).

Quitando ambas redundancias el bloque típico baja a ~906 B y **cabe en 1024 sin
tocar el techo**. Pero no sé si eso es la solución correcta o solo un parche más
elegante, y hay un trade-off real: la ruta absoluta es directamente abrible con
`Read` por el modelo, y una ruta relativa exige que sepa la raíz de la KB (que sí
está en el `core-index` que se inyecta en el arranque de cada sesión, pero eso es
acoplamiento entre dos hooks).

## Lo que quiero que investigues

### 1. ¿Es un patrón sistémico? (la pregunta de Paul)

Inventaria los caps del framework y su historia real: `EXO_RECALL_CAP=6144` en
`plugins/reflex/scripts/exo-recall.sh` (con su guard y su nota de "límite real"),
el cap 2 KB de `compose-inject.sh`, el `--cap-bytes` por defecto de `exo recall`
(2048, `engine/src/main.rs`), el `kbx_budget_max` de la KB, y este 1024. Busca en
la KB (`exo search --db ~/.exo/index.db --type hybrid --json "..."`) y en los specs
del repo si esos números se han movido, cuántas veces, y en qué dirección.

**Contrasta con la doctrina que la propia casa ya tiene sobre esto**: el trinquete
F1 de kbx sella `kbx_budget_max` de modo que **solo puede bajar**, y la regla escrita
es *"si una nota no cabe: pártela, no subas el techo ni la mutiles"*. Está en
`core/core-index.md` de la KB. Si esa doctrina aplica aquí, subir el cap es
exactamente lo que la casa se prohibió a sí misma — dilo alto. Y si NO aplica,
explica por qué este cap es de otra naturaleza.

### 2. ¿Cuál es la solución correcta para ESTE cap?

Candidatas que se me ocurren, sin que te limites a ellas ni asumas que alguna es
buena:

- **Quitar la redundancia** (ruta relativa + título solo cuando aporte algo sobre el
  nombre de fichero). Ahorro medido: ~231 B/bloque. ¿Qué se pierde?
- **Acortar el snippet** en el hook (hoy ~189 B de media, tope 200, los genera el
  engine). Es el 50% del bloque. ¿Cuánto snippet hace falta para que un puntero
  cumpla su función? Ojo: su función es que el modelo decida SI leer la nota, no
  sustituir su lectura.
- **Presupuesto por hit** en vez de por bloque (p. ej. 340 B por hit × 3), que hace
  el tamaño predecible y no dependiente de la suerte del ranking.
- **Menos hits pero completos** vs **más hits recortados**.
- Cualquier cosa mejor que se te ocurra, incluida "el cap correcto aquí es otro
  número y aquí está el argumento por el que no es un parche".

### 3. La pregunta de segundo orden

Si el patrón es real —que los caps de esta casa se fijan con una medición optimista
y luego se suben—, ¿hay una regla de diseño que lo prevenga en el futuro? Algo del
tipo "todo cap se fija sobre el percentil 90 de una muestra real, no sobre un
ejemplo", o "los caps se expresan por unidad y no por agregado". Si la hay, dila en
una línea para que pueda ir a la doctrina. Si crees que no hay regla general y cada
cap es su propio problema, dilo también — es una respuesta válida.

## Restricciones

1. **Cero cambios en el engine (Rust)** es la propiedad que hace M6-06 barato. Si tu
   propuesta obliga a tocarlo, dilo alto: cambia el coste del item y hay que
   escalarlo a Paul.
2. Régimen §0: proyecto personal, cerrar ya, sin métricas nuevas ni ventanas.
3. YAGNI: si tu arreglo es más caro que el problema, no es un arreglo.
4. Lo ya construido de M6-06 (gate, búsqueda, composición) está cerrado y revisado;
   solo está en juego **cómo se dimensiona y compone el bloque**.

## Contexto

- Spec: `docs/superpowers/specs/2026-08-22-m6-06-recall-punto-de-uso-design.md`
- Verdict que la originó: `docs/superpowers/consultas/2026-08-22-m6-06/consultor-m6-06.md` (D5, D6)
- Implementación viva: `plugins/reflex/scripts/recall-inject.sh` (rama `m6-06-recall-punto-de-uso`)
- Índice: `~/.exo/index.db` (**cópialo** si vas a escribir; no lo toques en sitio).
  `sqlite3` CLI no está instalado; usa `python3 -c` con el módulo `sqlite3`.

## Formato

1. Veredicto en una línea a la pregunta de Paul: ¿es un apaño sistémico, sí o no?
2. El inventario de caps con su historia, y qué dice el patrón.
3. Adjudicación FIRMADA de qué hacer con el cap de M6-06, con el trade-off explícito.
4. La regla de diseño para el futuro, o el argumento de por qué no la hay.
5. Qué consideraste y descartaste, con la razón.

Respuesta final a mí: ~30 líneas. El detalle va al fichero.
