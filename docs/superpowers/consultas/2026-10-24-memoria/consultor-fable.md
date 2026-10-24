# Consultor — diseño de la etapa 1 (memoria entre sesiones)

## 1. Recomendación
**No corras O1 tal cual. Corre 4 brazos (A0, ORÁCULO = regla literal por `SessionStart`, X = exo, N = auto memory nativa) sobre ~40 sondas-suelo cribadas (A0 falla, el oráculo cumple), k=3, y mide cada brazo como fracción del techo recuperada; N frente a X por no inferioridad con margen 0,15. L se descarta, G queda opcional.**

Por qué: K mostró X ≈ A0 (Δ +0,06, IC [−0,025, +0,16]). Una NI «frente a X» sin que X haya demostrado efecto sobre A0 en las mismas sondas la pasa cualquier brazo que no haga nada: *assay sensitivity* (ICH E10; D'Agostino, Massaro y Sullivan 2003, *Stat Med*) [E, conocidos de antes; no releídos hoy]. techo-reglas-2 da el ancla: regla entregada, 6/10 tareas-suelo cumplen frente a 0/10 [E, verdict entero]. El oráculo es «retrieval perfecto»; la pregunta pasa a ser medible: ¿qué fracción del techo recupera cada acceso, a qué coste?

## 2. Condición que la invalida
Si en las sondas cribadas el ORÁCULO recupera < 0,30 (menos de 12/40 con ≥2/3 réplicas `check.rc`=0), no hay techo que repartir: el cuello es cumplimiento, no acceso, y la etapa 1 pasa a «simplificar exo a entrega de reglas duras». Segunda: si el IC95 bootstrap de Δ(X−A0) tiene límite inferior ≤ 0, exo no tiene efecto de acceso demostrado y se decide solo por coste (tokens_in de `meta.json`), sin NI.

## 3. Receta
1. **Pool.** Reglas de `pool/reglas-g*.jsonl` + decisiones fechadas de la KB (O3). Candidatas ≈ 200. Las 13 suelo de K entran directas; las 6 que el oráculo volteó en techo-reglas-2, sin cribar.
2. **Criba en dos pasos, pre-registrada.** (a) A0 × k=2: queda la que da 0/2 (~0,08 USD/corrida → ~32 USD). (b) ORÁCULO `ar` × k=2 sobre las supervivientes: queda la que da ≥1/2 (~8 USD). Objetivo: 40 sondas. Sesgo declarado: la criba sobre A0 induce regresión a la media; no importa porque A0 se vuelve a medir con réplicas nuevas.
3. **Principal.** 40 sondas × {A0, ORÁCULO, X, N} × k=3 = 480 corridas ≈ 50 USD. Más 15 neutras (S2) × {A0, X, N} × k=2 ≈ 9 USD para daño, margen 0,15 (granularidad 1/15; lección de K: 3/20 > 0,10).
4. **Métrica.** Éxito por tarea en {0,⅓,⅔,1}. Primaria: Δ emparejada, bootstrap sobre tareas y signos, como K. Secundaria: R = (brazo − A0)/(ORÁCULO − A0). Coste: tokens_in y USD de `meta.json`.
5. **Reglas.** NI de N vs X: `N NO INFERIOR` si límite inferior IC95 de Δ(N−X) > −0,15. Si además N cuesta menos tokens_in (mediana), gana N: exo se abandona o se reduce al escritor. Si X gana con límite inferior > 0 y Δ ≥ 0,15, exo sobrevive y se abre la etapa 2. Resto: `NO CONCLUYENTE`, y manda el coste (D4 del pre-registro).
6. **Harness.** `correr.sh` se extiende con un `case` por brazo (precedente `arp`/`ars`) [E, leído entero]. Antes de correr: (i) `autoMemoryEnabled:false` fijo en la línea 81 → por brazo; N con `autoMemoryDirectory` a copia fresca por corrida; (ii) **I2 sigue abierto**: sin `git -C $P/kb checkout -- . && git clean -fdq` entre corridas [E]; (iii) `fugas.py`: añadir N al dict `HOOKS` y vigilar escrituras al snapshot y a la memoria; (iv) canario de que `--setting-sources ""` + `--settings` carga `MEMORY.md` [sin dato]; (v) volcar el bloque inyectado por corrida.
Presupuesto ~100 USD con criba; si el pool no da 40 sondas-suelo, parar.

## 4. Análisis por opción
**Potencia de K** [E, reconstruida del verdict: 34 concordantes + {1,1,1,½,−½,−½}; los crudos de `~/.cache` ya no existen]. Bootstrap reproducido exacto. SD por tarea 0,30, SE 0,047. Potencia para Δ=0,10: **0,56**. n (80 %, bilateral): 70 para 0,10; 31 para 0,15; 18 para 0,20.

**NI: margen y N** [E, cálculo propio]. Con sondas que discriminan la SD por tarea sube a ~0,45 (k=2) o ~0,35–0,40 (k=3). Δ verdadera 0, 80 % unilateral: margen 0,10 → 76–125 tareas; 0,15 → 34–56; 0,20 → 19–31. Con 40 sondas y k=3 el semiancho del IC es ≈0,12–0,14: 0,15 es lo mínimo defendible; 0,10 no cabe en 100 USD.

**O1 (5 brazos).** **N como se describe no cabe**: 286 líneas > 200 y solo título+ruta pesan 35 KB > 25 KB [E, medido; límites verificados hoy en la doc oficial]. Índice de core+stable (84 notas, ~9,5 KB) cabe; eso hace de N «índice + Read bajo demanda», casi G, por eso G es opcional (el A1 que K nunca corrió). **L (180k tokens)**: tokens_in ≈ 10× A3 (K: mediana 132k por corrida; L mete 180k por turno) y, si la ventana es 200k, no deja sitio a la tarea [sin dato]. Variante: **L' = solo core (69 KB ≈ 17k tokens) en system prompt**; ver §6. Sin oráculo, O1 no tiene assay sensitivity.

**O2 (A0, N, X con más N).** Bien en potencia, mal en interpretación: sin oráculo ni sondas-suelo repite K con otra etiqueta. Mi receta es O2 + oráculo + criba.

**O3 (reutilizar S1 filtrado + decisiones fechadas).** Sí, base del pool. Las 13 suelo de K son pocas; 10 ya pasaron por techo-reglas-2 y sus 4 «no cumple» incluyen regla mal escrita y conflicto regla-tarea (fallo del gold, no del acceso): el filtro por oráculo limpia eso. Checks que miran nombres de comando (g0-115) se reescriben sobre efectos [E, verdict §4/§6].

**Sondas con A0 al 57 %.** No se construyen: se criban (A0 falla *y* oráculo acierta). Elegirlas a ojo es el sesgo de autor que §5 quería evitar.

**Estudio de escritura.** Mal como decisión, bien como piloto. Con 10 sesiones no hay inferencia; y los brazos difieren en el disparador (`/document` explícito frente a «cuando el modelo decide»): en `-p` casi nunca escribe auto memory sin pedírselo; medirías el disparador, no el almacén [O]. Arreglo: mismo disparador («cierra la sesión y persiste lo decidido») y medir (a) ¿la decisión D aparece? (juez a ciegas), (b) bytes, (c) minutos del dueño, (d) lectura de vuelta: sesión N+1 resuelve una sonda derivada de D. Descriptivo; no decide la etapa 1.

## 5. Desacuerdos con el brief
1. NI «frente a X» es inválida sin ancla positiva; K ya dijo X≈A0. Sin oráculo, la etapa 1 solo concluye por coste.
2. N con una línea por nota no cabe (líneas y bytes). Decidir qué 200 líneas van es curación, no acceso; hay que declararlo.
3. L es inviable al coste y quizá al contexto; L' (core en sysprompt) sí es informativo.
4. El estudio de escritura, tal como está, confunde disparador con almacén.
5. O1 omite el estrato neutro: el daño de K venía de ahí; se sigue midiendo con margen acorde a la granularidad.
6. 100 USD dan para 4 brazos con criba; 5 piden ~130. El pool no se compra.

Sin verificar: ventana y precio de sonnet-5-5; que auto memory cargue con `--setting-sources ""`; reparto core/stable de las reglas del pool (jsonl privado).

## 6. Tras el debate
- **Oráculo primario = `ar`** (regla por `SessionStart`, sin framing: el canal de X). techo-reglas-2 ya midió la diferencia en las mismas 10 tareas: `ar` 5/10, `arp` 6/10, mecánicas 4/5 en ambos [E]. Canal+framing ≈ 1 tarea en 10; segundo oráculo sobra (el canal es de la 2d). El oráculo no entra en el NI: criba y da R; techo 0,5 en vez de 0,6 estrecha R, no la primaria.
- **Coste primario = Δ tokens_in emparejada frente a A0 sobre todas las sondas** (suelo + neutras), más la sobrecarga fija por turno (bloque X 5,9 KB; MEMORY.md ~9,5 KB). 30 corridas neutras solas no bastan: tokens_in es de cola pesada (K: mediana 109k, media 156k). El neutro se reporta aparte.
- **Prevalencia: ya medida.** E4: 484 prompts reales, 6 correcciones (5 «otra vez»), 0 citas de un learning [E]: ≈1 %. Desempate declarado: prevalencia × Δ(X−A0) frente a sobrecarga fija por sesión. Con 1 %, exo tiene que ganar por mucho en el suelo para pagar su 22 %.
- **Contra (b)/L'.** E4: las reglas del pool salen de core+stable (47 notas, 449 KB); core es ~69 KB, ≈15 %. Sin estratificar por tier, L' solo alcanza las sondas con regla en core y falla el resto por construcción: mide cobertura del core, no acceso; y si se acerca al oráculo, sigue confundido (canal + contenido). Para que informe: ≥15 sondas-core estratificadas y ~160 USD. G no tiene ese problema pero duplica a N. Veredicto: 4 brazos; L' solo con estratificación declarada.
