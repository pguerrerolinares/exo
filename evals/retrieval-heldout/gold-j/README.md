# Gold J — cómo se construyó (100 % agéntico, fiabilidad medida)

Contrato: `docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`
§3 (estratos, jueces, suelo) y §11 (kill-criteria). El gold vive fuera del
repo (`~/.local/share/exo-evals/j-heldout/gold-j.jsonl`); aquí solo el método,
la plantilla y los recuentos. Decisión de Paul del 2026-09-19 (`propuesta.md`
§7): la KB la consumen agentes, no Paul; el patrón de relevancia es la
utilidad para el agente, y el gold lo etiquetan jueces LLM de dos familias.

## Pipeline

1. **Queries** por estrato: `prompt` (prompts reales que dispararon
   `recall-inject` desde el 2026-09-13, `pool.py`), `agent-search` (comandos
   `exo search` reales de agentes, minados de los transcripts con autorización
   de Paul y limpiados con `limpia_agent_search.py`: criterio en su docstring;
   **estrato pendiente en este kit, ver «Estado del kit» abajo**), `hard`
   (paráfrasis por un agente que no vio ningún gold), `archive` (hechos que
   hoy viven solo en `archive/log/`), `negativo` (temas ausentes; ausencia
   comprobada con `grep` de `topic_terms` sobre la KB entera y, además, por
   los jueces). Anti-fuga: nada con Jaccard ≥ 0,8 frente a las 55 ni a las
   147 de C.
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

Este kit **no incluye juicio** (Task 7 del plan). La fábrica paró el juicio
con Kimi porque el control de gasto del breaker de `juez.py` tiene un fallo
detectado en review adversarial (el libro de gasto puede mentir en un caso
concreto); el arreglo de ese breaker es de otro frente y no se ha tocado
aquí. Nadie llamó a ninguna API de pago para construir este kit.

**Además, el estrato `agent-search` no se pudo generar.** El Step 3 del plan
(re-extracción de comandos `exo search` reales desde
`~/.claude/projects/**/*.jsonl`, con autorización de Paul del 2026-09-19)
está bloqueado: el clasificador de permisos del harness deniega la lectura
de esa ruta por motivo «PII Data Handling» / «Sensitive-Source Provenance»,
tanto con el script del plan como con un `grep` directo. Es un bloqueo
distinto de la autorización de D-J11 — D-J11 autoriza el *diseño* del
experimento (minar esas queries), no es un permiso del harness sobre sus
propios transcripts, y el clasificador no lo reconoce como tal. No se usó el
fichero crudo preexistente de 221 líneas (`~/.exo/priv-j/agent-search-raw.txt`,
de la sesión de planificación): el propio plan lo describe como truncado por
el escape `\"` de `grep`, así que meterlo habría sido peor que no tenerlo —
parecería un estrato válido sin serlo.

Este kit se generó **sin `agent-search`**: `queries.jsonl`, `candidatos.jsonl`
y `paquetes.jsonl` tienen 229 filas (`prompt` 139 · `hard` 30 · `archive` 20 ·
`negativo` 40), no 289-349 como habría sido con `agent-search` (60-120 filas
más, cota del plan). **Riesgo real sobre el suelo de no nulas (60, §11)**
(actualizado con el recall real del kit medido en «Recall del kit» abajo —
`archive` 14/20, no el 20/20 que el §3 asumía en el mejor caso):

- **Recuento estimado de no nulas con el kit tal cual: ≈ 52-73, mediana ≈ 64**
  (`prompt` 20-30 + `hard` 20-25 + `archive` recortado por el recall real a
  ≈ 8-14 en vez de 12-16 + `negativo` 0). **Probabilidad de quedar por
  debajo del suelo de 60: ~35-45 %** — no es una formalidad, es una apuesta
  real con el kit en su estado actual.
- **Si se repite el Step 7 del bloque `archive` (`q170`-`q189`, ver «Recall
  del kit» arriba) antes de juzgar:** el rango sube a **≈ 55-76** y el
  riesgo de no llegar al suelo baja a **~30 %**. Barato (un lote de 20
  candidatos con un agente fresco) frente al coste de fallar (abajo).
- **El otro suelo que nadie citaba explícitamente: `archive` ≥ 8 filas no
  nulas** (§11, `no_nulas_por_estrato.archive`) para que D-C (la decisión
  sobre penalizar `archive/`) pueda tomarse. Lo esperado tras acuerdo sobre
  las 14 candidatas reales son **10-12 filas**: pasa, pero **sin margen** —
  un acuerdo algo peor de lo esperado en ese estrato concreto (no en el
  total) deja a D-C sin decidir, aunque el gold global sí llegue al suelo
  de 60.
- **Lo que se pierde si el suelo global falla NO es el dinero:** el kit tal
  cual son ≈ 0,45-0,6 M tokens de entrada para Kimi (ver «Recuentos» abajo)
  ⇒ **≈ $2 con kimi-k3, no los ≈ $6** que cita la Task 7 Step 3 del plan
  (esa cifra es para el job completo de ≈ 280 filas CON `agent-search`, que
  este kit no tiene). Lo que se pierde de verdad: **las 229 queries quedan
  consumidas** (§11, «Prohibido: … re-juzgar filas», y el gold entero queda
  «consumido» tras el verdict) — no se pueden re-juzgar ni reciclar en un
  segundo intento, y **`agent-search` quedaría fuera de este gold para
  siempre** (el estrato con las únicas queries reales de agentes, D-J11):
  cualquier necesidad futura de retrieval-para-agentes exigiría un gold
  nuevo desde cero, no una ampliación de este.
- **Recomendación del review (no del kit — la decisión es de Paul):**
  esperar a que se resuelva el permiso de lectura de `agent-search` (punto 1
  de «Para completar» abajo) antes de juzgar. Es el único estrato con
  queries reales lanzadas por agentes en producción; sin él, el gold
  responde una pregunta distinta de la que D-J11 firmó (relevancia para
  `prompt` + generadas, no para el patrón de uso real de `exo search` por
  agentes). Con `agent-search` (40-70 no nulas esperadas) el objetivo total
  92-141 deja margen mucho más cómodo sobre el suelo, y el riesgo de arriba
  desaparece. Pero esto es una recomendación, no un bloqueo: si Paul decide
  juzgar con el kit tal cual (o tras repetir el Step 7 de `archive`),
  asumiendo el ~30-45 % de riesgo y la pérdida irreversible de las 229
  queries si falla, esa es una decisión válida y suya.

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

**Sigue pendiente:** una re-extracción real que siga el Step 3 al pie de la
letra (parseo del JSON de los bloques `tool_use`/`Bash`, no un grep de
texto), o la decisión explícita de Paul de juzgar sin `agent-search` con el
riesgo ya documentado arriba.

**Para completar, el día que Paul decida:**

1. El permiso de lectura de `~/.claude/projects` **ya no es el bloqueo**
   (addendum «tercer intento» abajo: se entregó un fichero crudo con ese
   permiso ya concedido). El bloqueo actual es de **calidad de la
   extracción**: hace falta una extracción que, en el propio paso de parseo
   del JSON, aísle el argumento de query de cada `exo search` (como hizo la
   campaña C, ver el addendum) en vez de volcar la línea de comando completa
   para que un filtro de texto la limpie después. Alternativa: decidir
   explícitamente juzgar sin `agent-search`, asumiendo el riesgo ya
   documentado arriba.
2. Si se resuelve (1): correr el Step 3 del plan (Task 6) para producir
   `agent-search.jsonl`, añadir esas filas a `queries.jsonl` (continuando la
   numeración `qNNN` a partir de `q230` para no reordenar ni tener que
   regenerar los candidatos/paquetes ya hechos — desviación deliberada del
   orden `prompt → agent-search → hard → archive → negativo` de §3, anotada
   aquí), generar sus candidatos (Step 7) y añadirlos a `candidatos.jsonl`, y
   regenerar `paquetes.jsonl` completo con `juez.py paquetes`. **Matiz
   (review final de rama, 2026-09-20): esto solo es sano si las 229 filas
   actuales NO se han juzgado todavía.** Si el paso 4 (juicio) ya corrió
   sobre las 229 antes de decidir añadir `agent-search`, añadir filas y
   volver a correr el juicio no es una ampliación limpia: es o bien "gold
   modificado tras congelar" (§11, si ya se congeló) o, si aún no se
   congeló pero ya se juzgó, exige volver a juzgar las 229 originales junto
   con las nuevas para que el acuerdo se calcule sobre el conjunto
   completo de una sola vez — nunca parchear un `acuerdo.py` ya corrido con
   las filas nuevas por separado (eso sería, de facto, "repetir los jueces
   hasta que pasen" sobre un subconjunto, prohibido en §11). Orden correcto:
   pasos 1-2 (resolver `agent-search` y añadirlo a `queries.jsonl`) **antes**
   del paso 4 (juicio), no después.
3. Que el arreglo del breaker de `juez.py` esté cerrado y mergeado (otro
   frente de la fábrica).
4. Correr el juicio (Task 7 del plan), con `$PRIV_J = ~/.local/share/exo-evals/j-heldout`:

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
   Constraints) es tope $10, gasto esperado ≈ $6 con kimi-k3.

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

## Recuentos (kit sin `agent-search`; sin texto de queries)

- snapshot `S_J`: `6bf57d513dc2ad53e815a4debafe6982f6998151` (detached en
  `$PRIV_J/kb-snap`, 32 ficheros en `archive/log`)
- `queries.jsonl`: 229 filas · por estrato `prompt 139 · hard 30 · archive 20
  · negativo 40 · agent-search 0 (PENDIENTE)`
- `candidatos.jsonl`: 229 filas (1:1 con `queries.jsonl`) · con lista de
  candidatos vacía: 66 (40 del estrato `negativo`, donde vacía es el
  resultado esperado; 26 del estrato `prompt`, sin candidata plausible
  encontrada por el agente de candidatos — `hard` y `archive` no tuvieron
  ninguna vacía, esperable porque su `author_expected` viene de una nota
  real del snapshot)
- `paquetes.jsonl`: 229 paquetes · chars por paquete: p50 8.296 · p90 16.463
  · máx 21.287 · total 1.782.082 (≈ 0,45-0,6 M tokens estimados para Kimi;
  bajo porque falta `agent-search`)
- acuerdo / gold: no calculado — Task 7 no se ejecutó (ver «Estado del kit»)
- coste Kimi: $0 — ninguna llamada a la API en este kit
