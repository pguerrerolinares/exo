# Consultor Fable — ¿"subir el cap" es un apaño sistémico? (M6-06)

**Fecha**: 2026-08-22 · **Consultor**: Fable (caps sistémicos)
**Brief**: `brief-caps.md` · **Método**: verificación primaria contra el engine vivo
(10 queries nuevas, 30 hits, lectura pura sin `--refresca`), historia git de exo,
agent-develop (pre-monorepo de reflex), kb-demo y kbx. Scripts en scratchpad.

---

## 1. Veredicto en una línea

**Paul tiene razón en el diagnóstico y no en la literalidad**: "siempre hemos
subido" es falso (2 subidas en 6 caps; el de A1 aguantó dos incidentes sin moverse
y kbx solo ha bajado), pero el patrón real es peor que el que él nombra — **todos
los caps nacidos de una medición puntual optimista han mordido en días y se han
parcheado; todos los nacidos como presupuesto deliberado con mecanismo de
disciplina han aguantado**. El apaño no es subir: es cómo nacen. Y el cap de
M6-06 nació mal: el "~970 B medido" de la spec es el **mínimo** de la
distribución, no el típico.

## 2. Verificación primaria (independiente del recon del brief)

Muestra propia: 10 queries sustantivas distintas de las del brief, contra
`exo recall --limite 4 --cap-bytes 1400 --json` (la invocación real del hook),
compuestas con la lógica exacta de `recall-inject.sh`:

| métrica | brief (27 hits) | mi muestra (30 hits) | veredicto |
|---|---|---|---|
| bloque 3-hit, mediana | 1137 B (950–1236) | **1187 B (935–1314)** | confirmado; el ~970 B de la spec ≈ mínimo |
| queries racionadas por cap 1024 | 7/10 | **7/10** | confirmado exacto |
| prefijo KB repetido por hit | 44 B ×3 | **44 B ×3, 30/30 hits** | confirmado |
| título redundante con fichero | 26/27 | **20/21 únicos** (laxo: `:`→`-` del filesystem) | confirmado; mi 1ª pasada estricta dio 19/30 por no plegar la sustitución de `:` |

**Hallazgo nuevo (tercera redundancia)**: 12/16 snippets únicos empiezan con
`# ` y **10/16 repiten el título literal dentro del snippet**
(`- …/kbx-bitacora.md — kbx-bitacora · # kbx-bitacora Bitácora…`). Un hit típico
dice el nombre de la nota **tres veces**: fichero, campo título y header del
snippet. Redundancia total del bloque ≈ 280–340 B de ~1187 (~25%).

**Corrección al brief**: quitar solo las dos redundancias que propone NO basta.
Medido: mediana baja a ~914 B pero la cola sigue — sin presupuesto por hit,
1–4/10 bloques siguen sobre 1024 según la suerte de rutas/títulos largos. La
única variante con fit determinista es redundancia fuera + presupuesto por hit
(§5): **0/10 racionadas, max 1010 B**.

## 3. Inventario de caps y su historia real

| cap | valor | nacimiento | incidentes | movimiento |
|---|---|---|---|---|
| `kbx_budget_max` (tiers core 8.500 / stable 12.500 + techos por nota) | trinquete F1, sellado en `.kbx-ratchet.json` | doctrina deliberada | notas obesas recurrentes | **solo bajó**: doctrina-agentes 27000→20000 (`5da6c59`), splits ejecutados (`b3df97c`, `89e5a77`) |
| A1 `compose-inject.sh` | 2048 B | spec transporte, 3 consultores | (a) el canario se comía el presupuesto (U1=0% por construcción); (b) truncado silencioso de `rutas()` (F3.1) | **nunca subió**: (a) `BUDGET=2048−canario`; (b) evento `inject-truncated` (`ce8ef80`) |
| SessionStart `EXO_RECALL_CAP` | 6144 B | memoria-v2 (2026-07-03), guard de oversize | bloque real 4,5–5 KB, con holgura | **nunca se movió** |
| SessionStart `EXO_RECALL_LIMITE` | 5→**10** | "con 5 basta", optimista | perdía notas del mismo día (hallazgo del gate M6) | **subió ×2** (`21d8cb2`, 2026-08-17) |
| engine `--cap-bytes` default | 2048 | m2-08 | ninguno: los hooks lo pasan explícito | no |
| **M6-06 bloque** | 1024 B | "bloque real medido = ~970 B" — un ejemplo, no una distribución | raciona 7/10 queries a 2 punteros | en cuestión; **ya nació parcheado**: la implementación lleva `--limite 4 --cap-bytes 1400` interno donde la spec decía `--limite 3 --cap-bytes 1024`, "para que quede margen" |

**El patrón**: la casa tiene dos regímenes. Donde el cap es un **presupuesto con
mecanismo** (trinquete + split en kbx; sustracción del canario y rastro en A1),
el cap aguanta o baja. Donde el cap se fijó con **una medición puntual y ε de
margen** (límite 5 del digest, el 1024 de M6-06), muerde en días y la reacción es
subir o compensar. La objeción de Paul detecta el segundo régimen; el primero
demuestra que la casa ya sabe hacerlo bien.

## 4. ¿Aplica la doctrina del trinquete aquí?

Aplica **por analogía, con traducción**. Los dos tipos de cap no son el mismo
animal:

- **Caps de contenido autorado** (kbx): acotan algo que un autor puede
  reescribir. El cap es herramienta de disciplina; subirlo es rendirse. De ahí
  "si una nota no cabe, **pártela**, no subas el techo".
- **Caps de transporte compuesto** (todos los de inyección): acotan un bloque
  cuyo tamaño es `formato × N`. Aquí nadie "parte" nada — pero el análogo exacto
  de "pártela" es "**quita la grasa del formato**". Subir el techo sin mirar el
  formato es la misma rendición.

Y en M6-06 la grasa existe y es un cuarto del bloque: prefijo ×3, título que
redunda en el 95% de los hits, y snippet que re-repite el título. **Subir el cap
a 1280/1536 sería pagar para siempre ~300 B/turno de repetición literal en el 86%
de los turnos, y consagrar el defecto de nacimiento.** Es exactamente lo que la
casa se prohibió en kbx, traducido a transporte.

## 5. Adjudicación para M6-06 — FIRMADA

**El cap 1024 se queda. Cambia la composición del bloque, solo en el hook.
Cero cambios de engine.** Tres movimientos en `recall-inject.sh` (§ composición),
~25 líneas con sus tests:

1. **Prefijo una vez**: la cabecera declara la raíz
   (`…material de la KB en /home/…/kb-demo…`); los hits llevan ruta
   relativa. Ahorra 132 B, cuesta 47. Autocontenido: la raíz viaja en el mismo
   bloque, sin acoplarse a que el core-index del arranque esté vivo.
2. **Título solo cuando aporta**: se omite si coincide con el nombre de fichero
   bajo la normalización laxa (plegado de acentos + `:`→`-`), reutilizando la
   maquinaria de folding que el script ya tiene para el gate. Ahorra ~80 B por
   bloque. Ídem el título repetido al frente del snippet (`# titulo …`): se pela.
3. **Presupuesto por hit derivado del cap, no otro número mágico**:
   `hit ≤ (1024 − overhead_cabecera_footer) / 3`, calculado en el script desde
   las constantes reales (~280 B/hit). Si un hit lo excede, se recorta el
   **snippet** a frontera de palabra con elipsis — nunca la ruta. El único
   parámetro libre del diseño pasa a ser el 1024; todo lo demás se deriva.

**Resultado medido** (mis 10 queries): 0/10 racionadas, mediana ~918 B,
max 1010 B. El invariante de la spec — *"el cap protege del outlier, no raciona"*
— pasa a ser verdad **por construcción**, no por suerte. La llamada interna
`--limite 4 --cap-bytes 1400` (margen de fetch, ya revisada) no se toca.

**Trade-offs firmados**:
- La ruta deja de ser copy-pasteable directa a `Read`: el modelo une raíz (en la
  cabecera del propio bloque) + relativa. Una concatenación trivial; el riesgo de
  que falle es mínimo y el fallo es benigno (un `Read` errado y reintento).
- En hits de ruta larga el snippet baja a ~130–170 B. Suficiente para su función
  — decidir **si** leer la nota, no sustituir su lectura; hoy la mitad de esos
  bytes eran el título repetido.
- +~25 líneas en la sección de composición de un script que ya carga maquinaria
  más pesada (el gate). Coste puntual; el parche de subir pagaba renta perpetua.

## 6. Regla de diseño para el futuro (para doctrina, una línea cada una)

- **Nacimiento**: *un cap se fija sobre el peor caso por construcción del formato
  × N (o sobre percentil alto de muestra real; nunca sobre un ejemplo), y los
  tamaños internos se derivan del cap — no al revés.*
- **Mordida**: *si un cap muerde en el caso normal, el bug está en el formato o
  en el nacimiento del cap; se arregla eso una vez — no se sube el número cada
  vez que duele.*
- **Familias**: *contenido autorado → trinquete (solo baja; si no cabe, se
  parte); transporte compuesto → derivado del formato (el cap es guard de
  outlier y debe ser inalcanzable por construcción en el caso normal).*

## 7. Considerado y descartado

- **Subir el cap a un número derivado (~1536 = peor caso del formato actual)**:
  honesto como derivación, y es la opción de coste cero en código. Descartado
  porque paga renta perpetua de ~300 B/turno de repetición literal, deja el fit
  probabilístico (bloques de 3 notas de `archive/sesiones/` con rutas de ~100 B
  aún pueden racionar) y contradice la doctrina traducida (§4). Si Paul prefiere
  cerrar hoy con una línea, esta es la única subida defendible — pero es peor.
- **Acortar el snippet globalmente (200→120 B)**: ataca al componente equivocado
  primero. La mitad de la grasa no es snippet útil sino nombre repetido; con el
  presupuesto por hit el recorte solo ocurre donde hace falta.
- **Menos hits completos (2) en vez de 3 flexibles**: es lo que el cap ya hace
  hoy por accidente (70% de queries → 2 punteros). El 3º puntero cuesta ~90 B
  netos tras quitar grasa y es la diferencia entre triaje ancho y estrecho;
  el criterio de cierre del item ("un prompt trae **sus notas**") pide ancho.
- **Presupuesto por hit sin quitar redundancia**: con el formato actual,
  296 B/hit deja ~170 B de snippet en rutas cortas pero ~60 B en largas —
  mutila donde la grasa sigue intacta. La combinación es lo que funciona.
- **Tocar el engine** (snippet más corto de fábrica, rutas relativas en el JSON):
  prohibido por el brief y además innecesario — todo lo anterior es composición
  en el hook. Si algún día el engine emite `ruta_rel`, el hook se simplifica;
  no es requisito.

## Anexo — procedencia

Mediciones: `scratchpad/mide-bloques.py`, `variantes.py`, `snip-header.py`
(lectura pura contra `~/.exo/index.db`, sin `--refresca`, DB intacta).
Historia: `git -C exo log -S`, `git -C agent-develop log --follow`
(`21d8cb2`, `ce8ef80`, `838891c`, `bc76c8d`), `git -C kb-demo log -S`
(`5da6c59`, `b3df97c`), `kb-demo/.kbx-ratchet.json`,
`core/core-index.md:19` (trinquete F1).
