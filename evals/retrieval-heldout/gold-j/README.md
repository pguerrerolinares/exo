# Gold J — cómo se construyó (100 % agéntico, fiabilidad medida)

Contrato: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
§3 (estratos, jueces, suelo) y §11 (kill-criteria). El gold vive fuera del
repo (`~/.local/share/exo-evals/j-heldout/gold-j.jsonl`); aquí solo el método,
la plantilla y los recuentos. Decisión de Paul del 2026-09-19 (`propuesta.md`
§7): la KB la consumen agentes, no Paul; el patrón de relevancia es la
utilidad para el agente, y el gold lo etiquetan jueces LLM de dos familias.

## Pipeline

1. **Queries** por estrato: `prompt` (prompts reales que dispararon
   `recall-inject` desde el 2026-09-13, `pool.py`), `agent-search` (queries
   `exo search` reales de agentes, minadas de los transcripts con autorización
   de Paul, ya aisladas por el parseo del JSON de los bloques `tool_use`
   —igual que hizo la campaña C— y limpiadas con un criterio propio,
   documentado en el addendum del 2026-09-20 «cuarto intento, resuelto»
   porque el formato de entrada no encajaba con `limpia_agent_search.py`:
   ver «Estado del kit» abajo), `hard` (paráfrasis por un agente que no vio
   ningún gold), `archive` (hechos que hoy viven solo en `archive/log/`),
   `negativo` (temas ausentes; ausencia comprobada con `grep` de
   `topic_terms` sobre la KB entera y, además, por los jueces). Anti-fuga:
   nada con Jaccard ≥ 0,8 frente a las 55 ni a las 147 de C.
2. **Candidatos** (≤ 5 por query) por un agente filesystem-only, con al menos
   2 por navegación temática y no por grep. Es el cuello léxico declarado.
   Se despachó en 4 lotes de ≤ 60 filas, cada uno un subagente fresco
   distinto del generador de queries, sin acceso a `author_expected` ni
   `topic_terms` (se le pasó una copia de `queries.jsonl` con solo
   `id`/`query`/`source`, para no depender solo de la disciplina del brief).
3. **Dos jueces ciegos e independientes** sobre el mismo paquete (query +
   candidatos con título y primeros 4.000 caracteres; sin etiquetas, sin
   rankings, sin el otro juez): **fable** (Claude) y **Kimi** (Moonshot,
   `juez.py`, modelo fijado en el pre-registro §10). Kimi recibe trozos de la
   KB: **envío autorizado explícitamente por Paul el 2026-09-19**.
4. **Acuerdo** (`acuerdo.py`): una fila entra solo si los dos jueces
   coinciden (estricto, o el expected de uno es aceptable para el otro). Los
   desacuerdos se descartan (sin tercer juez) y se cuentan. Suelo
   pre-registrado: κ de Cohen ≥ 0,60 y acuerdo ≥ 0,70 sobre todas las filas
   juzgadas; por debajo, J PARA (es un resultado).
5. **Auditoría del sesgo léxico**: solape query↔nota en acordadas frente a
   descartadas, y el subconjunto «léxicamente difícil» como guard de D-A.

## Estado del kit — qué falta para juzgar

Este kit **no incluye juicio** (Task 7 del plan) — pero, a diferencia de
cuando se escribió esta sección por primera vez, **ya no hay ningún
bloqueo técnico para correrlo**. La fábrica había parado el juicio con
Kimi porque el control de gasto del breaker de `juez.py` tenía un fallo
detectado en review adversarial (el libro de gasto podía mentir en un caso
concreto); **ese arreglo ya está cerrado y mergeado** (commit `bb82cab`,
identidad por nonce a prueba de reanudación con test de resume tras
`kill -9` real, más `6fcb3bb` de compatibilidad documental; review
adversarial independiente: Approved). El techo de gasto es ahora
estructural (≈$10,02 con tolerancia 0 = tope + una llamada de overshoot),
no depende de que un bug no se dispare. Nadie llamó a ninguna API de pago
para construir este kit.

**El estrato `agent-search` ya está incorporado (2026-09-20, cuarto
intento).** Los tres intentos anteriores fallaron por motivos distintos
(bloqueo de permiso, formato de extracción roto dos veces — ver los tres
addenda abajo, conservados como historia). El cuarto partió de un fichero
ya en el formato correcto: una query por línea, ya aislada por el parseo
del JSON de los bloques `tool_use` (el mismo método con el que la campaña C
extrajo su propio `agent-search`, `pool.pool_comandos`), no de una línea de
comando completa. Detalle del método de limpieza y los números exactos en
el addendum «cuarto intento — resuelto» más abajo.

`queries.jsonl`, `candidatos.jsonl` y `paquetes.jsonl` tienen ahora **285
filas** (`prompt` 139 · `agent-search` 56 · `hard` 30 · `archive` 20 ·
`negativo` 40). **Riesgo recalculado sobre el suelo de no nulas (60, §11)**
(mismas bases que antes para `prompt`/`hard`/`archive`/`negativo` — el
recall real de `archive`, 14/20 y no el 20/20 del mejor caso, sigue sin
repetirse, ver «Recall del kit» abajo — más la contribución nueva de
`agent-search`):

- **`agent-search`: ≈ 30-49 no nulas esperadas tras acuerdo**, de 56 filas
  juzgables. Dos anclas para el rango: (a) el modelo conservador del propio
  §3 (40-70 no nulas de 60-120 filas, ratio implícito 58-67 %) escalado a
  N=56 da ≈ 33-38; (b) el precedente empírico de C, que con este mismo tipo
  de query rindió **34 no nulas de 39 filas juzgadas (87 %)** — mucho más
  alto que el modelo conservador del plan, porque una query real de agente
  contra una KB que sí tiene contenido relevante acierta más que `prompt`
  (que incluye mucho operativo sin tema) o que una paráfrasis generada. Uso
  el rango combinado 30-49 para no sobreestimar con solo un precedente. Señal
  de apoyo: el generador de candidatos encontró al menos una candidata
  plausible en **54/56 filas** (2 vacías, ambas queries cortas de una o dos
  palabras sin nota plausible), un recall de cobertura muy por encima del
  de `archive` (14/20).
- **Recuento total estimado de no nulas con el kit completo: ≈ 78-118,
  mediana ≈ 98** (`prompt` 20-30 + `agent-search` 30-49 + `hard` 20-25 +
  `archive` recortado por el recall real a ≈ 8-14 en vez de 12-16 +
  `negativo` 0; mediana = suma de los puntos medios de cada rango). **Cae
  dentro o muy cerca del objetivo pre-registrado 92-141**; incluso en el
  escenario más pesimista plausible (20+20+8+30 = 78, sumando el mínimo de
  cada estrato salvo negativo) el total queda **por encima del suelo de 60
  con margen amplio** (18 filas de colchón en el peor caso, frente a estar
  pegado al suelo o por debajo como con el kit de 229). **La apuesta real
  del ~35-45 % de caer bajo el suelo que
  documentaba el kit sin `agent-search` prácticamente desaparece**: el
  escenario que la generaba (kit sin `agent-search`, mediana ≈64 pegada al
  suelo) ya no es el estado del kit. Juzgar deja de ser una apuesta sobre
  el suelo global; sigue siendo una apuesta más pequeña y ya documentada
  sobre el suelo específico de `archive` (siguiente punto).
- **El suelo que sigue sin margen: `archive` ≥ 8 filas no nulas** (§11,
  `no_nulas_por_estrato.archive`) para que D-C (la decisión sobre penalizar
  `archive/`) pueda tomarse. Sin tocar en este trabajo (fuera de alcance de
  esta tarea): lo esperado tras acuerdo sobre las 14 candidatas reales
  siguen siendo **10-12 filas** — pasa, pero sin margen. Repetir el Step 7
  de `archive` (`q170`-`q189`) antes de juzgar sigue siendo la mitigación
  barata recomendada, sin cambios respecto a lo ya documentado.
- **Coste si se juzga el kit completo:** ≈ 0,68-0,91 M tokens de entrada
  para Kimi (ver «Recuentos» abajo, calculado sobre los 2.480.983 caracteres
  reales de `paquetes.jsonl` + system prompt, no una extrapolación) ⇒
  **≈ $2,7-$4,5 con kimi-k3** (entrada a $3/M + salida estimada a $15/M),
  compatible con la cifra ≈ $6 que citaba la Task 7 Step 3 del plan para el
  job completo (≈280 filas) — esa cifra era más conservadora (más margen
  para reintentos/notas largas); con `--tope-usd 10` hay margen de sobra en
  cualquiera de las dos estimaciones.
- **Lo que se pierde si se juzga y el suelo falla sigue sin ser el
  dinero:** las 285 queries quedan consumidas (§11, «Prohibido: … re-juzgar
  filas»); no se pueden re-juzgar ni reciclar en un segundo intento. Pero
  con el riesgo recalculado arriba, fallar el suelo global es ahora un
  evento de cola, no la apuesta central que era con el kit de 229 filas.

## Addendum 2026-09-20 — segundo intento de `agent-search`, sigue sin poder generarse

Se recibió un nuevo fichero crudo (`$PRIV_J/agent-search-raw-2026-09-20.txt`,
293 comandos `exo search` supuestamente únicos, extraídos y deduplicados sin
contexto de conversación) como sustituto del de 221 líneas ya descartado
arriba. Se corrió `limpia_agent_search.py` **sin modificar**, con las mismas
dos listas de exclusión (las 55 y las 147 de C): `{"entrada": 293, "salida":
46, "descartes": {"no_parsea": 42, "marcador": 195, "vacia": 1,
"dup-exclusion": 0, "dup-pool": 9}}`.

Inspección manual de las 46 filas supervivientes: **0 son queries usables**.
Todas son fragmentos de una sola palabra o de puntuación (artículos,
conectores, restos de markdown) — nada que un agente pudiera plausiblemente
haber escrito como argumento de `exo search`. Causa raíz: el fichero crudo
**no es** lo que pide el Step 3 (parseo de bloques `tool_use` con
`name=="Bash"` cuyo `input.command` contiene `exo search`), sino, aparentemente,
una captura tipo grep de cualquier texto de transcript que **menciona** la
frase "exo search" — prosa de specs, informes de benchmark, mensajes de
commit, documentos de planificación. Evidencia: 0 de las 293 líneas
contienen un carácter `"` literal (una query real multi-palabra citada lo
tendría); 35/293 terminan en un flag de CLI sin texto de query, el mismo
patrón de truncamiento que ya invalidó el fichero de 221 líneas.

Por el mismo criterio ya aplicado arriba: meter estas 46 filas sería peor que
no tener `agent-search` — parecería un estrato real de queries de agentes
siendo ruido. **No se usó**: ninguna fila entró en `queries.jsonl` /
`candidatos.jsonl` / `paquetes.jsonl`, ningún número de este README cambió,
`agent-search` sigue en 0. `agent-search-raw-2026-09-20.txt` y el
`agent-search.jsonl` de 46 filas resultante quedan en `$PRIV_J` como
evidencia de auditoría, marcados como no consumibles.

**Sigue pendiente (histórico, a fecha de este segundo intento):** una
re-extracción real que siga el Step 3 al pie de la letra (parseo del JSON
de los bloques `tool_use`/`Bash`, no un grep de texto), o la decisión
explícita de Paul de juzgar sin `agent-search` con el riesgo ya
documentado arriba. **Ya resuelto**: ver el addendum «cuarto intento —
resuelto» más abajo — esta re-extracción se hizo y `agent-search` ya está
incorporado.

**Para completar, el día que Paul decida (actualizado 2026-09-20, con el
breaker de `juez.py` ya arreglado):**

1. ~~El permiso de lectura de `~/.claude/projects`~~ y ~~la calidad de la
   extracción de `agent-search`~~ **ya no son el bloqueo**: el estrato está
   incorporado (56 filas, `q230`-`q285`), con candidatos y paquetes
   regenerados y 1:1. Ver el addendum «cuarto intento — resuelto» abajo para
   el método y los números completos. Las 229 filas originales (`q001`-
   `q229`) **no se habían juzgado todavía** cuando se añadió este estrato
   (verificado: no existe `$PRIV_J/kimi.jsonl` ni `$PRIV_J/gold-j.jsonl` en
   este directorio), así que añadirlo ahora, antes de cualquier juicio, es
   la ampliación limpia que el matiz de más abajo exige — no hace falta
   volver a juzgar nada porque nada se ha juzgado aún.
2. ~~Que el arreglo del breaker de `juez.py` esté cerrado y mergeado~~ **ya
   está**: commit `bb82cab` (identidad por nonce a prueba de reanudación,
   con test de resume tras `kill -9` real) + `6fcb3bb` (compatibilidad
   documental), review adversarial independiente Approved. El techo de
   gasto es ahora estructural (≈$10,02 con tolerancia 0 = tope + una
   llamada de overshoot), no depende de que un bug no se dispare. **No
   queda ninguna precondición técnica pendiente** — lo único que falta es
   la decisión de Paul de juzgar, con el coste (≈$2,7-$4,5) y el riesgo
   (suelo global ya no es la apuesta que era; el suelo específico de
   `archive`, ≥8, sigue sin margen) ya documentados arriba.
3. Correr el juicio (Task 7 del plan) sobre las **285 filas** actuales, con
   `$PRIV_J = ~/.local/share/exo-evals/j-heldout`:

   ```bash
   # fable: subagente fresco por lotes, ve solo paquetes.jsonl (brief exacto en plan, Task 7)
   # kimi:
   python3 evals/retrieval-heldout/harness/juez.py modelos --env-keys /home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys
   python3 evals/retrieval-heldout/harness/juez.py kimi \
     --paquetes "$PRIV_J/paquetes.jsonl" \
     --env-keys /home/paul/Documentos/proyectos/wisdom-ai-news/.env-keys \
     --model <id fijado en el paso anterior, kimi-k3> \
     --out "$PRIV_J/kimi.jsonl" \
     --precio-entrada 3.00 --precio-salida 15.00 --tope-usd 10
   # acuerdo:
   python3 evals/retrieval-heldout/harness/acuerdo.py \
     --candidatos "$PRIV_J/candidatos.jsonl" \
     --fable "$PRIV_J/fable.jsonl" --kimi "$PRIV_J/kimi.jsonl" \
     --gold-out "$PRIV_J/gold-j.jsonl" \
     --descartes-out "$PRIV_J/descartes.jsonl" \
     --informe-out "$PRIV_J/informe-acuerdo.md" \
     --snap "$PRIV_J/kb-snap" --kappa-min 0.60 --po-min 0.70
   ```

   Precondición de `--tope-usd 10`: el presupuesto autorizado (plan, Global
   Constraints) es tope $10, gasto esperado ≈ $2,7-$4,5 con kimi-k3 sobre
   las 285 filas del kit completo (cálculo en «Recuentos» abajo).

## Addendum 2026-09-20 (bis) — tercer intento de `agent-search`, el formato del crudo está roto

Se entregó un tercer fichero (`$PRIV_J/agent-search-cmds.txt`), esta vez con
el permiso de `~/.claude/projects` ya resuelto y descrito como «169 comandos
`exo search` únicos, extraídos parseando los bloques `tool_use`». La
inspección mecánica no confirma esa descripción:

- El fichero tiene **2.808 líneas no vacías**, no 169. Causa: el Step 3 exige
  aplanar los saltos de línea de cada comando a un espacio antes de escribir
  una línea por comando (`agent-search-cmds.txt` no lo hizo); varios
  comandos capturados escriben ficheros con heredoc (`cat > … <<'EOF' … EOF`)
  y ese cuerpo multilínea —con sus propias líneas en blanco— quedó tal cual
  en el fichero, indistinguible de un separador de registro. Partir por
  líneas en blanco tampoco resuelve el límite: de 429 bloques así obtenidos,
  solo 53 contienen la subcadena `exo search`; los otros 376 son prosa/markdown
  sin relación aparente, evidencia de que la extracción capturó más que
  bloques `tool_use`/`Bash` aislados que contuvieran `exo search`.
- Solo **184 líneas físicas** contienen la subcadena `exo search` (no 169).
- Se corrió `limpia_agent_search.py` **sin modificar**, tal cual pide la
  tarea, sobre el fichero entregado: `{"entrada": 2808, "salida": 2,
  "descartes": {"no_parsea": 1225, "marcador": 1581, "vacia": 0,
  "dup-exclusion": 0, "dup-pool": 0}}`.
- Diagnóstico adicional (no se usó para el gold, solo para separar el
  problema de aplanado del resto): se repitió la limpieza restringida a esas
  184 líneas. Resultado idéntico en las supervivientes: `{"entrada": 184,
  "salida": 2, "descartes": {"no_parsea": 18, "marcador": 164, "vacia": 0,
  "dup-exclusion": 0, "dup-pool": 0}}`. Las 2 filas que sobreviven en ambos
  casos son la misma pareja: un fragmento de 2 caracteres y uno de 8, sin
  forma de query — misma firma de fallo que las 46 filas ya descartadas en
  el addendum anterior.
- Clasificación automática (recuento, no texto) de las 184 líneas: ≈80 tienen
  forma de invocación directa (`exo search --…` / `exo search "…"`), pero 27
  son bucles `for … in` sobre varias queries a la vez, 24 citan `exo search`
  como cadena literal dentro de un `grep` (comandos que buscan la frase, no
  que buscan CON el comando), 38 la mencionan entre backticks dentro de
  prosa/specs, y 13 redirigen el resultado a `jq`. Causa raíz de por qué
  incluso las ≈80 de forma directa casi no sobreviven: `MARCADORES` en
  `limpia_agent_search.py` (no tocado) trata el carácter `|` como marcador de
  doc/test, y prácticamente toda invocación real en este corpus canaliza la
  salida a `jq`/`head`/`grep` para poder leerla — el filtro las descarta como
  si fueran ejemplos de spec. Es un desajuste estructural frente al método de
  C (`pool.jsonl` de C ya trae solo `{query, session_id, ts}`: la query se
  aisló en el propio parseo del JSON, sin pasar una línea de comando
  compuesta por un filtro de texto).
- Techo teórico (solo diagnóstico, **no se usó para el gold**): saltándose
  `MARCADORES` por completo y aplicando solo `pool.query_de_comando` + dedupe
  a las 184 líneas, salen **14** queries distintas, de las cuales 6/14 son
  fragmentos de menos de 8 caracteres sin espacio. Ni siquiera en el mejor
  caso posible este fichero se acerca al rango 60-120 esperado.

**Conclusión, con el mismo criterio ya aplicado dos veces en este README:**
meter estas 2 filas (o las 14 del techo teórico, con 6/14 fragmentos) sería
peor que no tener `agent-search` — parecería un estrato real siendo ruido.
**No se usó ninguna fila.** `queries.jsonl`, `candidatos.jsonl` y
`paquetes.jsonl` siguen en 229 filas, sin cambios (mismos sha256 que antes
de este intento). El gold de C no se tocó (`no_nulas: 92`, `errores: 0`,
sha256 `614ae599…c43a75`, verificado con `valida_gold.py` en este mismo
intento). Ningún número de este README cambia por este addendum salvo el
punto 1 de «Para completar» arriba (el permiso ya no es el bloqueo; la
calidad de la extracción sí lo es). `agent-search-cmds.txt` y los ficheros
de diagnóstico de este intento quedan en `$PRIV_J` como evidencia de
auditoría, marcados como no consumibles.

## Addendum 2026-09-20 (ter) — cuarto intento, `agent-search` resuelto: 56 queries reales incorporadas

Se entregó un cuarto fichero, esta vez en el formato correcto: **65 queries
únicas, una por línea, JSONL `{"query", "session_id", "ts"}`** — la query ya
aislada por el parseo del JSON de los bloques `tool_use`/Bash (partiendo el
comando con `shlex` y quedándose con los argumentos posicionales), no una
línea de comando completa. Es el mismo formato que produce
`pool.pool_comandos` (el método con el que la campaña C extrajo su propio
`agent-search`), confirmado por inspección directa del fichero antes de usar
nada de él (la lección de los tres intentos previos).

**Por qué no se usó `limpia_agent_search.py` tal cual**: su función `limpia()`
espera líneas de **comando completo** (`exo search --json "..."`) y llama a
`pool.query_de_comando`, que hace su propio `shlex.split` interno para
aislar la query; aplicarla a filas que ya son solo la query (sin `exo
search` delante) no tiene nada que parsear y todo se habría descartado como
`no_parsea`. Reconstruir una línea sintética `exo search "<query>"` por fila
tampoco sirve: no arregla el problema real (ver debajo) y sería fabricar una
línea de comando que nunca existió. Se aplicó en su lugar una limpieza
propia, documentada aquí, reutilizando sin modificar `pool.normaliza` y
`pool.jaccard` (mismas funciones, mismo criterio de anti-fuga que usa
`limpia_agent_search.py`) y el mismo regex `MARCADORES` del script para
descartar restos de doc/prosa.

**Hallazgo de validación (la muestra sí importaba)**: inspeccionando las 65
filas antes de tocar nada aparecieron dos problemas reales, ninguno de los
cuales es "la query es ruido" en el sentido de los tres intentos previos —
es un artefacto mecánico del propio parseo:

1. **58 de 65 queries llevaban un sufijo de redirección de shell pegado**
   (`… 2>&1`, `… 2>/dev/null`, `… 2>$SP/e1`, rutas de fichero tras `2>`).
   Causa: `shlex.split` no entiende la sintaxis de redirección de bash —
   `2>&1` no es un flag (no empieza por `-`), así que `query_de_comando` lo
   trata como el último "positional" y lo devuelve como si fuera parte de la
   query (p. ej. `exo search "<query real>" 2>/dev/null` → query extraída
   `"<query real> 2>/dev/null"`). Se corrigió recortando ese sufijo con un
   regex (`\s+\d*>&?\d*\S*$`, aplicado repetidamente hasta punto fijo) — la
   query real que queda debajo es genuina en el 100 % de los casos
   inspeccionados.
2. **2 de 65 eran el literal `$q`** (una vez con `2>&1`, otra con
   `2>/dev/null`): el nombre de una variable de bucle de shell
   (`for q in ...; do exo search "$q" ...`), no el valor real buscado en
   tiempo de ejecución. El parseo del bloque `tool_use` capturó el texto del
   comando tal cual se escribió, no su expansión — no hay forma de
   recuperar la query real desde ahí. **Descartadas** (no son ruido de
   contenido, son la variable en vez del valor).
3. **3 de 65 caían en el regex `MARCADORES` de `limpia_agent_search.py`**
   (reutilizado sin modificar, aplicado a la query ya sin el sufijo de
   redirección): 2 eran prosa de documentación, no queries — fragmentos de
   una nota/plan que citan código entre backticks y contienen `$` (el
   propio contenido son extractos de este mismo proceso de extracción,
   evidencia de que el fichero crudo mezcló prosa con queries reales) —, y
   1 era el residuo de redirección a secas, sin query delante (el regex de
   redirección no recorta un sufijo que no tiene nada delante, y el propio
   `MARCADORES` ya lo descarta porque contiene `>`).
4. **2 duplicados internos** tras normalizar: dos pares de filas con la
   misma query salvo el sufijo de redirección ya recortado en el punto 1
   (una de una sola palabra, otra de varias).

Recuento de la limpieza propia: `{"entrada": 65, "descartes": {"var_shell_literal": 2, "marcador": 3, "dup_interno": 2}, "salida_previa_a_antifuga": 58}`.

**Anti-fuga** (Jaccard ≥ 0,8 o forma normalizada idéntica, frente a las 55
in-sample **y** las 147 de C — mismas dos listas de exclusión que usa
`limpia_agent_search.py`, mismas funciones `pool.normaliza`/`pool.jaccard`
sin modificar): 2 filas más fuera por coincidencia exacta (forma normalizada
idéntica) con una query ya presente en las 55 o en las 147 de C — una de
ellas era la superviviente única del dedupe interno del punto 4 anterior, y
aun así coincidía con una query ya usada en C. **Salida final: 56 queries.**

Juicio de calidad sobre las 56 supervivientes: son queries reales, técnicas,
cortas, sobre varios de los proyectos y frentes activos del dominio de
trabajo de Paul que ya cubre la KB — exactamente lo que se esperaría que un
agente tecleara como argumento de `exo search` mientras trabaja, no prosa
ni ejemplos de test. Perfil: longitud mín 3 /
mediana 31 / máx 67 caracteres; 44/56 (79 %) con ≥3 palabras, 5/56 de una
sola palabra (nombres de proyecto, comandos de prueba o términos aislados,
reales y extraídos de comandos reales, pero genéricos: es esperable que
varias terminen en `expected: null` tras el juicio, igual que ocurre con
las queries operativas del estrato `prompt`, sin que eso invalide la fila).

**56 queda por debajo del rango 60-120 que el §3 esperaba** para este paso
(cota de la Task 6, no del gold final) — pero por encima del umbral de
30 que el propio Step 3 usa para marcar un estrato como "flojo"
(`docs/superpowers/plans/2026-09-19-campana-j-fase1-diagnostico-y-preregistro.md`
Task 6 Step 3: "Si `<n>` < 30, el estrato se conserva pero se declara
flojo"). No se infló el número para llegar a 60: 56 es el resultado
honesto de aplicar el criterio del script sin modificarlo más la
anti-fuga real. El impacto sobre el riesgo del suelo global se recalcula
en «Estado del kit» arriba — es pequeño, porque el recall de candidatos y
el precedente de C sugieren un ratio de no-nulas alto para este estrato
pese al recuento algo bajo.

**Candidatos**: generados por un subagente fresco filesystem-only (mismo
método del Step 7: navegación temática desde `core-index.md` y títulos +
grep de sinónimos, ≤5 por query, permalinks del frontmatter real, lista
vacía sin candidata plausible), con la lista de queries redactada a solo
`id`/`query`/`source` (sin `session_id`/`ts`, que no forman parte del
schema del gold y no se necesitan para candidatos). **54/56 filas con al
menos una candidata**; 2 vacías (ambas de una o dos palabras, sin nota
plausible). Los 56 permalinks propuestos existen todos en el snapshot
(`valida_gold.permalinks_snapshot`, `0` inválidos). Auditoría del método
(sin citar query ni permalink): al menos 2 candidatas por fila proceden de
seguir un puntero desde `core-index.md` o de navegar un directorio por
título de fichero (verificado sobre una muestra manual de la salida), no de
grep — cumple la regla del brief de que al menos 2 candidatas por fila
salgan de navegación temática.

`queries.jsonl` recibió las 56 filas **al final, con ids `q230`-`q285`**
(nunca intercaladas): el bloque `negativo` (`q190`-`q229`) queda exactamente
igual —verificado por sha256 de ese rango antes y después del append, mismo
hash `ee089593…00073f`—, así que su paridad par/impar (que decide el
split calibración/evaluación de G1) no cambia. `candidatos.jsonl` se
regeneró concatenando las 56 filas nuevas (1:1 con `queries.jsonl`,
verificado por comparación de listas de `id` en el mismo orden).
`paquetes.jsonl` se regeneró completo con `juez.py paquetes` (285 paquetes,
1:1 con `candidatos.jsonl`). El gold de C no se tocó (`no_nulas: 92`,
`errores: 0`, sha256 `614ae599…c43a75`, reverificado con `valida_gold.py`
en este mismo intento). Las dos suites de test siguen en verde (`test_harness.py`
49 OK, `test_gold_j.py` 33 OK) sin cambios, porque son tests de las
funciones del harness, no de los datos.

Ficheros de auditoría en `$PRIV_J` (privados, no en git): `agent-search-queries.jsonl`
(las 65 crudas recibidas), `queries-q230-q285-redactadas.jsonl` (input al
subagente de candidatos, solo `id`/`query`/`source`), `candidatos-q230-q285.jsonl`
(su salida), y un directorio `backup-pre-agent-search-<timestamp>/` con las
copias de `queries.jsonl`/`candidatos.jsonl`/`paquetes.jsonl` de antes de
este intento, por si hiciera falta revertir.

## Recall del kit (generador de candidatos) — F7, review de rama 2026-09-20

Métrica del kit: fracción de queries cuyo `author_expected` (la nota que el
generador de queries de `hard`/`archive` tenía en mente al escribirla, no
visto por el agente de candidatos) aparece entre las ≤5 candidatas que ese
agente propuso. Es el techo de no-nulas que el paso de juicio puede alcanzar
para esos dos estratos: si `author_expected` no está en `candidatos`, ningún
juez -por bueno que sea- puede recuperarlo como `expected`.

- **`hard`: 29/30.** Sano: muy por encima del rango 20–25 no nulas esperado
  tras acuerdo (§3 del pre-registro).
- **`archive`: 14/20** (bloque `q170`–`q189` completo). Bajo el rango 12–16
  esperado en el mejor caso, y a un solo miss adicional del suelo de 8 filas
  archive del §11 (`no_nulas_por_estrato.archive`) por debajo del cual D-C no
  se decide. Con acuerdo lenient imperfecto (< 100 %) sobre esas 14, el
  estrato `archive` real del gold puede terminar más cerca de 10–11 filas
  útiles que de las 12–16 que el §3 espera.
- **`agent-search`: 54/56 con ≥1 candidata** (`q230`–`q285`, cuarto intento,
  2026-09-20). No hay `author_expected` para este estrato (queries reales de
  producción, no generadas), así que no hay un "techo" que comparar; la
  métrica análoga es simplemente cuántas filas llegan al juicio con algo que
  juzgar. 2 vacías, ambas queries cortas/genéricas de una o dos palabras —
  ver el addendum «cuarto intento». Recall de cobertura muy por encima del
  de `archive`.

**Antes de juzgar** (no ejecutado aquí, documentado como paso previo): repetir
el Step 7 del plan (generación de candidatos) **solo para el bloque
`q170`-`q189`** con un agente de candidatos fresco, verificando mecánicamente
que aplicó la regla de §3 «la rotación archivada **y** la bitácora viva
cuando la query menciona un hecho fechado» (el motivo más probable del
recall bajo en `archive` es no incluir ambas). **Lo que NO vale**: inyectar
`author_expected` en la lista de candidatos para forzar el recall a 20/20 —
eso rompería la ceguera del generador de candidatos (§3: «sin ver
`author_expected` ni `topic_terms`») y invalidaría el estrato entero, no lo
arreglaría.

## Recuentos (kit completo con `agent-search`; sin texto de queries)

- snapshot `S_J`: `6bf57d513dc2ad53e815a4debafe6982f6998151` (detached en
  `$PRIV_J/kb-snap`, 32 ficheros en `archive/log`)
- `queries.jsonl`: **285 filas** · por estrato `prompt 139 · agent-search 56
  · hard 30 · archive 20 · negativo 40`
- `candidatos.jsonl`: 285 filas (1:1 con `queries.jsonl`, verificado por
  comparación de listas de `id` en el mismo orden) · con lista de candidatos
  vacía: **68** (40 del estrato `negativo`, donde vacía es el resultado
  esperado; 26 del estrato `prompt`, sin candidata plausible encontrada por
  el agente de candidatos; 2 del estrato `agent-search` nuevo — `hard` y
  `archive` no tuvieron ninguna vacía, esperable porque su
  `author_expected` viene de una nota real del snapshot)
- `paquetes.jsonl`: 285 paquetes (1:1 con `candidatos.jsonl`) · **regenerado
  el 2026-09-20 con `MAX_CHARS = 12.000`** (desviación F7 del pre-registro):
  texto total **5.409.490 caracteres** (antes 2.480.983 con el tope de
  4.000). Cuerpos de candidata truncados: **175/619 (28 %)**, frente a
  573/619 (93 %) con el tope viejo; filas con alguna candidata truncada:
  118/217 (54 %), frente a 216/217 (100 %). Candidatos idénticos en
  permalink y orden a la versión de 4.000 (diff programático, 0 diferencias).
- **tokens/coste estimados para Kimi** (sobre los 2.480.983 caracteres
  reales de `paquetes.jsonl` + `SISTEMA` de `juez.py`, 894 caracteres × 285
  llamadas = 254.790 chars más; conversión chars→tokens en el rango 3-4
  chars/token, sin `tiktoken` disponible en el entorno): entrada **≈
  0,68-0,91 M tokens** ⇒ **$2,05-$2,74** a $3/M; salida estimada (JSON
  corto por fila: `expected_permalink`, ≤2 `acceptable`, notas de 1-2
  frases) **≈ 43.000-100.000 tokens** ⇒ **$0,64-$1,50** a $15/M. **Total
  estimado ≈ $2,7-$4,5 con kimi-k3**, por debajo de los ≈ $6 que citaba la
  Task 7 Step 3 del plan para un job de tamaño similar (esa cifra llevaba
  más margen para reintentos/notas largas); tope autorizado $10 en
  cualquiera de los dos casos.
- **coste Kimi REAL**: `kimi-k3`, `temperature 1` (F6), 285 llamadas (ok
  285, errores 0, desconocidas 0), **1.863.353 / 167.629 tokens**, **$8,1045**
  (tope $20). La estimación de arriba ($2,7-4,5) se hizo con el kit de 4.000
  caracteres y no aplica al kit regenerado (×2,18 de texto); se deja por
  trazabilidad. Antes de la corrida válida hubo tres parciales, todas fuera
  del gold y preservadas: el humo con `temperature 0` (HTTP 400, 1 llamada
  desconocida y no facturada), la corrida sobre el kit de 4.000 detenida en
  17 filas por decisión de Paul, y una con timeout de 120 s detenida en 3
  filas.
- **acuerdo** (`acuerdo.py` exit 0, 2026-09-22): 285 filas juzgadas por
  ambos · p_o **0,874** · κ **0,810** (suelo 0,60 ∧ 0,70: **PASA**) · sin
  `negativo` ni candidatos vacíos: 0,780 / 0,834 (descriptivo) · descartes
  **45** (36 por desacuerdo, 9 de `archive` sin `expected` en `archive/`).
  Detalle por estrato en `evals/retrieval-heldout/verdict/gold-j-acuerdo.md`.
- **gold** (`valida_gold.py` exit 0, `errores: 0`, con exclusión de las 55
  in-sample y del gold de C): **240 filas · 145 no nulas** · no nulas por
  estrato `prompt 54 · agent-search 53 · hard 30 · archive 8` · nulas `prompt
  53 · negativo 40 · agent-search 2` · con acceptable 87 · sha256
  `5902ebc44b22447f609ce12ac0e3f015175e0786c8836ab48c54a9af3a6cf86a` ·
  `chmod 444`.
- **suelos del pre-registro con el N real**: no nulas 145 ≥ 60 ✓ · nulas de
  `negativo` 40 ≥ 24 ✓ · no nulas de `archive` **8 ≥ 8 ✓, a ras** — el
  único sin margen, en el estrato peor servido por el recorte (65 % de sus
  cuerpos siguen truncados a 12.000) y con la tasa de descarte más alta
  (60 %). Resultado frágil: se declara, no se celebra.
- **efecto medido del recorte** (mismo juez fable, mismo prompt, única
  variable `MAX_CHARS` 4.000 → 12.000, 285 pares): cambia el **11 %** de
  las etiquetas (31/285) — 13 `null`→nota, 4 nota→`null`, 14 nota→otra —,
  con **25 % en `archive`** y 0 % en `hard` y `negativo`. El recorte fabrica
  negativos falsos unas tres veces más de lo que fabrica positivos falsos.
