# Auditoría adversarial — Campaña K, fase 0 y Task 0 (2026-09-29)

- **Rol**: auditor fresco. No participé en el pre-registro, la Task 0 ni la fase 0.
- **Contrato**: `docs/superpowers/plans/2026-09-23-campana-k-preregistro-ablacion.md` congelado en `f324818` (committer date `2026-09-28T23:38:55+02:00` = `21:38:55Z`, igual que `uso.py:21`).
- **Recomputación propia, sin `uso.py`**: script en el scratchpad de la sesión (`audit/f0_recompute.py`, variantes `S` = semántica del script reimplementada, `L` = §3 literal, `LG` = literal + guardas del informe, más sensibilidades). Fuentes primarias: `~/.claude/reflex-log.jsonl`, `~/.claude/projects/*/<sid>.jsonl`, `~/.exo/index.db` (read-only, tabla `notas`, 199 filas), frontmatter de `wisdom-paul` (HEAD `389a0da`, working tree limpio).
- Este fichero solo lleva agregados: ni permalinks por fila, ni prompts, ni contenido de notas. Ids de sesión, si aparecen, truncados a 8.
- Solo lectura: no se editó nada del repo ni de la KB salvo este fichero; no se lanzó ningún `claude -p` ni corrida de la ablación.

## 1. Tabla de verificación

| # | comprobación | resultado |
|---|---|---|
| V1 | Hora de congelación | `git log -1 --format=%cI f324818` → `2026-09-28T23:38:55+02:00`. Coincide con la constante del script (`uso.py:21`) e informe §1 |
| V2 | Eventos `recall-inject-emitted` | 365 en el log; **357** con `ts` < congelación; 8 posteriores (5 de la sesión que corre esta rama), correctamente excluidos |
| V3 | Recuentos 301 / 56 / 74 / 897 / 0 no mapeados | **Reproducidos exactamente** (glob `*/<sid>.jsonl`; ninguna sesión aparece en dos proyectos) |
| V4 | Tiers del índice core 5 · stable 63 · log 113 · sin tier 18 | Reproducido. (Un `grep '^tier: core'` da 7 ficheros, pero en 2 de `docs/superpowers/` la línea está en el cuerpo, líneas 47–89, no en el frontmatter: el script lee bien. Los 37 `.md` de la KB que no están en el índice son todos de `.superpowers/`) |
| V5 | Fila principal `main` a+b+c | Variante `S`: **U 0,409 · U_base 0,179 · lift +0,229 · IC95 [+0,150, +0,312]**. Idéntica a `informe.md` §3 y a `~/.cache/exo-ablacion-k/fase0/resumen.txt` |
| V6 | Desglose a / b-exo / b-otro / c / sin exo / +bash | 0,060/0,007 · 0,296/0,080 · 0,259/0,123 · 0,030/0,013 · 0,262/0,133 · 0,429/0,179: **idénticos** a la tabla del informe, IC95 incluidos |
| V7 | Fila a fila contra `detalle.jsonl` oficial (301 filas × main/main_base × 5 criterios) | **0 discrepancias** |
| V8 | «78 coincidencias» de `b: otra salida` (informe §2) | 78 eventos: 68 con origen en `Bash`, 5 en `Edit`, 5 en `Read` |
| V9 | Ámbito `todo` | 0,468 / 0,243 / +0,226 [+0,151, +0,310]: coincide con `resumen.txt` |
| V10 | Bloque inyectado en el transcript | Va como entrada `attachment` (`hook_success`), no como `user`/`assistant`: **no se autocuenta** como tocado |
| V11 | Truncado a segundos (`ts <= ts_ev`) | 0 entradas `user`/`assistant` del mismo segundo del evento mencionan un inyectado; lag mediana evento→primera entrada del asistente 5,1 s (p10 2,3 s). Sin efecto |
| V12 | Write/MultiEdit en (a) frente a Read/Edit literal | a = 0,060 / 0,007 en ambas lecturas: **sin efecto** |
| V13 | (b) sobre cualquier `tool_result` (script) frente a solo Grep/Bash (§3) | U/U_base/lift **idénticos** (0,409/0,179/+0,229); solo baja `b_otro` U 0,259→0,246 (esos 10 eventos ya estaban tocados por otro criterio) |
| V14 | §3 literal sin guardas (`L`) | **U 0,445 · U_base 0,203 · lift +0,243 [+0,154, +0,327]**; (c) 0,150/0,053. Es la «primera cifra» que el informe no publica |
| V15 | Guardas: aplican igual a U y U_base | Sí (misma función `tocados`, `uso.py:98-123`). Efecto: −11 eventos en U, −7 en U_base |
| V16 | Títulos de dos palabras genéricas | No existen: 0 títulos de exactamente dos palabras en el índice (116 de una palabra, 83 de ≥3) |
| V17 | Control con otras semillas (200 sorteos, misma regla) | U_base media **0,206**, min 0,166, p5 0,176, p95 0,239, max 0,272. El 0,179 de la semilla pre-registrada cae en el percentil ≈6 |
| V18 | Muestra abierta contra transcripts (semilla `20260929`: 12 eventos tocados + 6 del control) | U: 3 uso directo real (cat/sed/Edit de la nota), 5 co-ocurrencia en salida de `exo search`, **4 listados** (`exo budget`, `exo ratchet`, `.kbx-ratchet.json`, `ls` de la KB) que casan con cualquier nota. Control: 3 reales (búsqueda por nombre, `exo search` + cita `[[…]]`, tail), 1 co-ocurrencia, **2 listados** (`git status` en la KB) |
| V19 | Fuentes circulares | 3 eventos `b_otro` con origen en un comando que lee `reflex-log.jsonl` (el agente ve los `permalinks=` de la inyección); 2 de ellos sin otro criterio |
| V20 | Sesiones de la muestra por proyecto | exo 105 · wisdom-paul 99 · bizkaia-now 28 · pguerrero-music 22 · ai-news 21 · **`-tmp-task0-isolated-run-eval2026-09-16` 20 · `-tmp-task0-isolated-run` 6** |
| V21 | Los 56 sin transcript | 49 eventos de 10 sesiones del 22–26 ago (transcripts ausentes) + **7 eventos con `session_id` no-UUID** (`''`, `count-test`, `strace-sess`): pruebas del hook, no sesiones |
| V22 | Task 0: coste, turnos y tokens de las 6 sondas desde `out-p*-*.jsonl` | **6/6 exactos** con `task0-recon.md`: p1 0,0568/0,0670 USD · 4/6 turnos · 75.156/123.865 in; p2 0,0765/0,1064 · 8/8 · 147.271/184.041; p3 0,0397/0,0629 · 4/5 · 68.110/102.120. Todas `completed`, `err-*.log` de 0 bytes, `duration_ms` 6,8–53,3 s, modelo `claude-sonnet-5-5` |
| V23 | Falso positivo `p1-a0` | `out-p1-a0.jsonl` línea 5: la cadena está dentro de un `tool_result` de `grep` sobre `plugins/exo/scripts/testdata/golden-recall-inject/recorte-snippet.txt`. En A0: 0 eventos `hook_*`, `init` sin hooks (plugins solo `agents-md`, `telemetry`), 0 apariciones fuera de `tool_result`. **Falso positivo real** |
| V24 | A3 en las sondas | 2 eventos `system/hook_*` por corrida (SessionStart + UserPromptSubmit), 1 `recall-inject-emitted` en cada `reflex-p*-a3.jsonl`, 0 en el log real de la fase 0 |
| V25 | A0 en las sondas toca la KB o `exo`? | 0 `tool_use` con `wisdom-paul`/`.exo/` y 0 invocaciones de `exo` en las 3 corridas A0. Pero las 3 sondas son tareas de código sin necesidad de memoria (ver B1) |
| V26 | Cambios al pre-registro tras congelar | `git diff f324818 06450e0 -- <pre-registro>`: solo D2 (10 líneas) y la línea final de §12. `git diff 06450e0 campana-k -- <pre-registro>`: 0 bytes |
| V27 | Historia del pre-registro antes de congelar | El fichero **nace en `f324818`** (no hay versión en `main` ni anterior). Que §3 no cambiara entre el 23 y el 28 con 314→357 eventos ya en el log **no es verificable** |
| V28 | Harness repo vs scratchpad | `run.sh` difiere solo en rutas de salida y en el `grep '"type":"result"'` final; `a3.json` idéntico |
| V29 | Salidas persistidas aparte (`tool-results/`) | 30 de 4.517 `tool_result` en las 48+26 sesiones llevan marcador de persistencia externa. Sesgo declarado (informe §5), pequeño |

## 2. Hallazgos por severidad

### Bloqueante (para arrancar la fase 1; no afecta a las cifras de la fase 0)

**B1. El CLAUDE.md global ordena usar exo y la KB, y D3 lo mete en A0 y A1.** `~/.claude/CLAUDE.md:19`: «La memoria persistente vive en la KB … servida por el engine `exo` (`exo search --db ~/.exo/index.db …`) … Úsala SIEMPRE por defecto para buscar/guardar memoria». En una tarea S1 (que depende de memoria) el agente de A0 está instruido a llamar a `exo`:
- con `exo` en el PATH (Task 0 «Pendiente»: el stub para A1 no está hecho y A0 comparte el problema) es una **fuga real**, o dispara el circuit breaker de §9 en cada corrida S1 de A0;
- con stub, el agente tiene la ruta del índice en el prompt y puede leerlo con python/sqlite, o `grep` la KB por ruta. **Ni §9 ni la errata propuesta detectan el acceso a la KB/índice por ruta** (la errata solo mira hooks, `additionalContext` y `exo` en Bash).
- Las sondas no ejercitaron el caso: son tareas de código (V25). «Canario A0: no ve ninguno» (`task0-recon.md:33`) prueba el canal de hooks, no el aislamiento de la memoria.
- Resolver antes del paso 2 de §12: o se declara errata sobre D3 (el CLAUDE.md de los brazos sin la sección «Memoria de sesiones», que es instrucción de exo y no «memoria nativa»), o se aísla HOME/KB/índice para A0/A1, y en cualquier caso se añade al detector de fuga el acceso a rutas de la KB y de `~/.exo` en A0.

### Importante

**I1. La muestra de la fase 0 no es solo «producción».** 26 de los 301 eventos (y 26 de las 74 sesiones) vienen de `/tmp/task0-isolated-run*`: corridas de evaluación del 15–16 sep con `claude-sonnet-5`, prompts de 70 caracteres, una inyección por sesión (V20). Excluyéndolas: **U 0,444 · U_base 0,196 · lift +0,247 [+0,163, +0,339]** sobre 275 eventos y 48 sesiones. El sentido es conservador para exo (esas sesiones tienen U 0,038), pero «74 sesiones» y «en producción» (informe §1, §4) describen mal la muestra. Además, de los 56 sin transcript, 7 son pruebas del hook con `session_id` sintético (V21), no «sesiones sin persistencia o ya purgadas».

**I2. El IC95 no incluye la incertidumbre del control.** El bootstrap remuestrea sesiones condicionado a un único sorteo del control. Con 200 semillas alternativas U_base recorre [0,166, 0,272], media 0,206 (V17); la semilla pre-registrada da 0,179 (percentil ≈6). El lift esperado sobre sorteos es ≈+0,20, no +0,229, y el IC reportado es demasiado estrecho. No es manipulación (la semilla está en §3), pero el informe debería promediar sobre sorteos o meter el sorteo dentro del bootstrap y declararlo.

**I3. El criterio (b) es sensible a salidas-listado, y la magnitud del lift depende de ello.** `exo budget`, `exo ratchet`, `cat .kbx-ratchet.json`, `git status` en la KB, `ls`/`find` y bucles de `exo search` con varias queries devuelven decenas de notas y casan con cualquier permalink; en la muestra abierta 4 de 12 hits de U y 2 de 6 del control son de este tipo (V18), y 3 eventos vienen de leer el propio `reflex-log.jsonl` (V19). Sensibilidad, descartando `tool_result` con ≥ N notas distintas y lecturas del reflex-log:

| N | U | U_base | lift | IC95 |
|---|---|---|---|---|
| sin filtro (informe) | 0,409 | 0,179 | +0,229 | [+0,150, +0,312] |
| 40 | 0,405 | 0,179 | +0,226 | [+0,144, +0,309] |
| 20 | 0,352 | 0,153 | +0,199 | [+0,121, +0,286] |
| 10 | 0,259 | 0,093 | +0,166 | [+0,078, +0,265] |

El signo del lift es robusto; la magnitud (+0,17 a +0,23) no la fija §3. Debe reportarse como sensibilidad, con el umbral declarado.

**I4. Informe §4: «La mayor parte del lift es redundancia con la búsqueda agéntica» dice más de lo que sostienen los datos.** Sin `b: salida de exo` el lift es +0,130 de +0,229: **sobrevive el 57 %**; solo 44/301 eventos (14,6 %) están tocados únicamente por esa vía. Y «redundancia» es una lectura causal (que el agente lo habría encontrado igual) cuando la inyección puede ser lo que le lleva a buscar. Frase sostenible: «≈43 % del lift desaparece si no se cuentan las salidas de `exo search`».

**I5. D2 tras la congelación va más allá de «rellenar el hueco» (§12).** Dentro de lo permitido: la unidad (§10 decía «USD o en tokens»; tokens de cuota entra) y el número (240M, cota alta declarada como no medida). Fuera: (a) **tandas de 40 con revisión discrecional** («si el ritmo es sostenible», «bastante más de lo estimado») es una regla de parada opcional sin umbral numérico, que se decide con resultados parciales a la vista; (b) **`--max-budget-usd 3` por corrida** es una regla nueva que no dice qué es una corrida cortada por presupuesto (¿infra de §9 o fallo de tarea?), y A3 consume +17–58 % más tokens (Task 0), así que el corte es asimétrico contra A3. Además el relleno se hizo en `06450e0` (tras la Task 0) y §12 lo situaba tras la Task 4: inocuo, pero es desviación del calendario declarado. Propuesta: declarar (a) y (b) como errata en el verdict, con umbral de parada numérico y ciego a resultados, y regla explícita para corridas con presupuesto agotado (misma regla para ambos brazos).

### Menor

**M1. Guarda de títulos de una palabra: justificada en dirección, tosca en alcance, y la cifra literal no se publica.** En la lectura literal los hits de (c) por título los dominan `jev-shadow` (33, nombre de proyecto), `AGENTS` (5), `README`, `recon`; pero la guarda también borra citas específicas (`exo-bitacora` 5, `doctrina-agentes` 4). Efecto: (c) 0,150→0,030; a+b+c −11 eventos en U y −7 en U_base; lift +0,243→+0,229. La corrección va **contra** la hipótesis de uso, así que no huele a ajuste favorable; pero se hizo después de ver 0,445 y el informe no reporta la fila literal (V14). Reportar ambas. Las rutas relativas en salidas de git existen (7 eventos `git`-only en U, 6 en el control) y están cubiertas por I3.

**M2. Control: «3 notas aleatorias» frente a n = len(perms).** 9 eventos con n < 3 (6 con 1, 3 con 2) sortean 1–2 notas de control. Exclusión de los inyectados del pool y sorteo sin reemplazo: no están en §3, son razonables, y quedan sin declarar. Efecto ≈ 0.

**M3. (b) partido.** No cambia U ni U_base (a+b+c incluye ambos); es un desglose añadido, no una corrección. Bien como descriptivo; la clasificación por regex (`exo … search|recall|targets`) deja `kbx targets` y `exo budget/ratchet` en `otra salida`.

**M4. Reproducibilidad ligada al estado del índice.** El pool del control depende de `~/.exo/index.db` (199 notas, mtime 23:34 del 28) y del frontmatter actual; una nota nueva cambia la secuencia del RNG. Registrar sha256 de la tabla `notas` (o volcado ordenado) en el informe.

**M5. «Con subagentes (`todo`) las cifras son casi iguales».** U 0,468 frente a 0,409 (+6 pp) y U_base 0,243 frente a 0,179; lo casi igual es el lift (+0,226 frente a +0,229).

**M6. Errata §9.** El falso positivo es real (V23). La propuesta es correcta en lo esencial, con tres matices: (a) «brazo que no debería tenerlos» debe fijarse por brazo **y por `hook_event`** (A1/A2 sí llevan SessionStart); (b) le falta el detector de acceso a la KB/índice por ruta en A0 (B1); (c) `exo` invocado desde `python3 subprocess` u otro wrapper no casa con «`tool_use` de Bash que invoca `exo`»: mejor buscar `exo` en el `command` completo y en el `reflex-*.jsonl` (que ya está).

**M7. Composición del control.** 73/113 notas `log` están en `archive/`; el 38 % de los permalinks inyectados (403/1.056) también. Es lo que §3 prescribe; solo condiciona la lectura de «no es al azar».

**M8. No verificable.** Que las reglas de §3 no se tocaran entre el 23 y el 28 de septiembre (V27); el texto de las tres preguntas del canario (los ficheros `canario-*.jsonl` dan «NO NO NO» en A0 y «SI SI NO» en A3, coherente con lo declarado, pero no reconstruí las preguntas).

## 3. Veredicto por punto

1. **Fase 0, recomputación — CONFIRMADO CON RESERVAS.** Los recuentos, la fila principal, el desglose y las 301 filas se reproducen sin discrepancias desde las fuentes primarias (V3–V9). Reservas: I1 (26 eventos de harness en una muestra descrita como producción), I2 (el IC omite la varianza del control; U_base 0,179 está en la cola baja de sus sorteos), I3 (la magnitud del lift depende de un filtro de listados que §3 no contempla: +0,17 a +0,23).
2. **Fidelidad a §3 — CONFIRMADO CON RESERVAS.** Desviaciones sin declarar: n en vez de 3 en el control, exclusión de inyectados, (b) sobre cualquier `tool_result`, Write/MultiEdit en (a). Todas con efecto medido ≈ 0 (V12, V13, M2). El truncado a segundos no muerde (V11). El orden del RNG es determinista y correcto, pero depende del índice (M4).
3. **Las dos correcciones — CONFIRMADO CON RESERVAS.** La guarda es post hoc pero contraria a la hipótesis y aplicada por igual a U y U_base; el partido de (b) no cambia U. Reservas: la fila literal (0,445/0,203/+0,243) tiene que estar en el informe; la guarda de una palabra borra citas legítimas; y quedan falsos positivos sin tratar (listados, reflex-log) que I3 cuantifica.
4. **Lectura §4 — REFUTADO en una frase, CONFIRMADO en el resto.** «La mayor parte del lift es redundancia» no se sostiene (57 % del lift sobrevive sin salidas de exo; solo el 14,6 % de los eventos se toca únicamente por esa vía). «41 % frente a 18 %», «6 %» y «3 %» son exactos; «casi iguales» para `todo` es impreciso (M5).
5. **Task 0 — CONFIRMADO, con un agujero en la errata.** Costes, turnos y tokens 6/6 exactos; el falso positivo de `p1-a0` es real y A0 no tuvo hooks ni inyección. La errata de §9 es sólida para el canal de hooks pero deja fuera la fuga por ruta a la KB/índice y la instrucción del CLAUDE.md global de usar `exo` (B1, M6).
6. **Congelación — CONFIRMADO CON RESERVAS.** Solo cambió D2 y la línea de §12 (V26); la unidad y el tope caben en el hueco. Las tandas con parada discrecional y el `--max-budget-usd 3` por corrida son reglas nuevas: deben ir al verdict como errata declarada con umbral numérico y regla simétrica para corridas cortadas (I5).

**Qué busqué para tumbar y no encontré:** discrepancias entre el script y las fuentes (0 en 301 filas), autoconteo del bloque inyectado (no ocurre, V10), efecto del truncado a segundos (0), colisiones de sesión entre proyectos (0), efecto de Write/MultiEdit o de (b) fuera de Grep/Bash (0), cambios al pre-registro más allá de D2 (0 bytes).
