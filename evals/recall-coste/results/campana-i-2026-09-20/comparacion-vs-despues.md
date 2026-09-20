| id | p50 antes | p50 después | p95 antes | p95 después | fallidas antes | fallidas después |
|---|---|---|---|---|---|---|
| s10-jq-log-l200000 | 530 | 531 | 546 | 542 | 0 | 0 |
| s10-jq-log-l20000 | 57 | 58 | 61 | 61 | 0 | 0 |
| s10-jq-log-l2174 | 12 | 14 | 17 | 19 | 0 | 0 |
| s1b-query-refresh-sim0-n1000 | 2985 | 980 | 3025 | 994 | 0 | 0 |
| s1b-query-refresh-sim0-n174 | 1001 | 971 | 1019 | 1035 | 0 | 0 |
| s1b-query-refresh-sim0-n5000 | 10834 | 1081 | 11091 | 1097 | 0 | 0 |
| s1-query-refresh-n1000 | 2788 | 980 | 2838 | 987 | 0 | 0 |
| s1-query-refresh-n174 | 1002 | 965 | 1013 | 984 | 0 | 0 |
| s1-query-refresh-n5000 | 10768 | 1078 | 11035 | 1092 | 0 | 0 |
| s2-query-n1000 | 2779 | 978 | 2841 | 1005 | 0 | 0 |
| s2-query-n174 | 995 | 965 | 1010 | 978 | 0 | 0 |
| s2-query-n5000 | 10828 | 1067 | 11131 | 1081 | 0 | 0 |
| s3-index-sin-cambios-n1000 | 11 | 10 | 16 | 11 | 0 | 0 |
| s3-index-sin-cambios-n174 | 4 | 4 | 7 | 7 | 0 | 0 |
| s3-index-sin-cambios-n5000 | 28 | 26 | 32 | 32 | 0 | 0 |
| s4-arranque-content-n1000 | 11 | 13 | 20 | 22 | 0 | 0 |
| s4-arranque-content-n174 | 8 | 8 | 10 | 9 | 0 | 0 |
| s4-arranque-content-n5000 | 42 | 45 | 49 | 51 | 0 | 0 |
| s5-search-fts-n1000 | 10 | 10 | 11 | 11 | 0 | 0 |
| s5-search-fts-n174 | 3 | 6 | 6 | 8 | 0 | 0 |
| s5-search-fts-n5000 | 13 | 11 | 20 | 19 | 0 | 0 |
| s6-hook-entero-n1000 | 2959 | 1054 | 3059 | 1082 | 0 | 0 |
| s6-hook-entero-n174 | 1079 | 1027 | 1093 | 1045 | 0 | 0 |
| s6-hook-entero-n5000 | 5055 | 1165 | 5071 | 1183 | 0 | 0 |
| s7-config-jq-n1000 | 4 | 4 | 4 | 5 | 0 | 0 |
| s7-config-jq-n174 | 4 | 3 | 4 | 5 | 0 | 0 |
| s7-config-jq-n5000 | 4 | 2 | 5 | 3 | 0 | 0 |
| s8-kb-precommit-n1000 | 107 | 105 | 118 | 114 | 20 | 20 |
| s8-kb-precommit-n174 | 64 | 66 | 68 | 69 | 20 | 20 |
| s8-kb-precommit-n5000 | 326 | 327 | 340 | 346 | 20 | 20 |
| s9-git-log-una-nota-n1000 | 0 | 2 | 1 | 2 | 0 | 0 |
| s9-git-log-una-nota-n174 | 2 | 2 | 3 | 3 | 0 | 0 |
| s9-git-log-una-nota-n5000 | 3 | 3 | 3 | 3 | 0 | 0 |

C-H27	PASA
C-H4 (s3 n5000 28→26)	VACÍO
C-H17a (s4 n5000 p95 49→51)	NO APLICA (puerta cerrada)
C-H17b (s2 n5000-n174 = 1067-965)	NO DERIVAR
C-H23 (s8 n5000 p50=327)	NO ABRIR
  regresión s5-search-fts-n174	3→6
C-noregresión	FALLA
PreToolUse:Bash triple (git status), Linux: 41 ms

## Addendum: hook_ms real (decisión #12 de Paul) y hallazgos del propio instrumento

No pedido literalmente por los Steps 1-4 de la Task 6, pero es el número que
la decisión #12 dice que decide: `hook_ms` (reloj de pared interno del hook,
`_hook-ms.sh`) va emitido en el mismo `reflex-log.jsonl` que produce el
propio `bench.sh` durante `s6-hook-entero` (`BENCH_KEEP=1` lo dejó en disco).
p50/p95 calculados sobre las 24 líneas de cada tamaño (20 corridas medidas +
3-4 de warmup de hyperfine, indistinguibles en el log):

| N | muestras | hook_ms p50 | hook_ms p95 | min | max |
|---|---|---|---|---|---|
| 174 | 24 | 1016 ms | 1035 ms | 989 ms | 1042 ms |
| 1000 | 24 | 1039 ms | 1069 ms | 1023 ms | 1075 ms |
| 5000 | 24 | 1151 ms | 1173 ms | 1115 ms | 1173 ms |

**Umbral de 1.500 ms p95 (decisión #12): se cumple en Linux, con margen
amplio (1173 ms p95 en el peor caso medido, N=5000)**. Falta W11 (Task 7,
PAUL-STEP) para el veredicto real, porque el umbral es por SO.

### Aviso: `despues` no aísla limpiamente la ganancia de la campaña I

El plan (Evidencia de la Task 6) predijo que solo `s6-hook-entero` se
movería frente a `despues`, porque ninguna task de esta campaña toca
`engine/src`. La medición contradice esa predicción: `s1-query-refresh`,
`s1b-query-refresh-sim0` y `s2-query` — que NO pasan por
`recall-inject.sh`, solo invocan el binario `exo` directo — también caen
drásticamente (p.ej. `s2-query-n5000` p50 10828→1067 ms). Esto **no puede
ser obra de esta campaña**: son escenarios que el trabajo de I ni toca.

Causa más probable: `despues` se midió en el commit `41e01bf` (campaña A,
2026-09-13); entre ese commit y este (`2588384`, 2026-09-20) aterrizaron
commits de la **campaña G** en `engine/src` (`58656f2`, `a71e7c5`,
`e2e53e0`, `003f93a` — ninguno de la campaña I) que evidentemente cambiaron
el rendimiento del binario. `despues` ya no es una línea base limpia para
aislar la ganancia de I: la caída de `s6-hook-entero` frente a `despues`
mezcla la ganancia real de I (menos spawns/spawns fundidos) con esta mejora
de motor ajena. Dicho eso, `hook_ms` (tabla de arriba) es una medida
absoluta, no un delta contra `despues`, así que no depende de este
problema — es la lectura que importa para la decisión #12.

### Dos bugs del propio instrumento (`bench.sh`), preexistentes, no de esta campaña

1. **`EXO_CONFIG` obsoleto entre iteraciones de `N`.** `bench.sh` exporta
   `EXO_CONFIG="$D/config.toml"` tras generar cada tamaño y nunca lo
   resetea; el generador `kb_sintetica` (desde el commit `58656f2`,
   campaña G, 2026-09-15) necesita esa config para `pool_de_vocabulario()`.
   Al terminar cada iteración, `bench.sh` borra `$D` (`rm -rf`), así que la
   iteración SIGUIENTE arranca su generador con `EXO_CONFIG` apuntando a un
   fichero que ya no existe → falla dura ("no encuentro la config de exo").
   Reproducido de forma determinista en el primer intento (falló al pasar
   de N=174 a N=1000). No es un bug de la campaña I: el generador que lo
   dispara es de la G y nadie parece haber corrido el bench completo con
   más de un tamaño desde entonces. **Workaround usado, sin tocar
   `bench.sh`:** `BENCH_KEEP=1` (flag ya soportada por el propio script)
   — evita el `rm -rf` de cada `$D`, así el `config.toml` de la iteración
   anterior sigue existiendo cuando la siguiente lo necesita (la sección
   `[embeddings]` es idéntica para cualquier N, así que leer la de un `N`
   distinto no afecta el resultado). Deja temporales en `/tmp` (no en el
   repo); se han limpiado a mano tras el commit de esta task. No se abre
   fix aquí — fuera del alcance de la Task 6 («lee el instrumento», no lo
   repara) y del alcance de la campaña I (no toca `engine/examples/` ni la
   lógica de `bench.sh`).
2. **Glob `"$OUT"/s*.json` del resumen barre también `saturacion-vector-n*.json`**
   (empieza por "s", añadido por la campaña G) en el bucle final que arma
   `resumen.tsv`, produciendo 3 errores cosméticos de `jq` en stderr ("null
   cannot be sorted") por corrida. No corrompe `resumen.tsv` — verificado
   fila a fila contra la tabla esperada — pero es ruido que confundiría una
   lectura automatizada del stderr. Tampoco se toca aquí, mismo motivo.

### Triple `PreToolUse:Bash` en Linux (criterio de la Task 5, ver Step 3)

41 ms para las tres invocaciones seguidas (`git-c-bash.sh`,
`git-add-all-guard.sh`, `verify-before-commit.sh`) sobre el payload
`git status`. Cota inferior: Linux nunca es más lento que W11 en spawns de
shell; el número decisivo es el de la Task 7 (W11, PAUL-STEP).
