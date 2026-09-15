# Veredicto del bench — campaña A (recall por prompt)

- Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md` (commit `816c8a7`)
- Línea base: `evals/recall-coste/results/baseline/` (`entorno.txt` pegado abajo)
- Después: `evals/recall-coste/results/despues/` (`entorno.txt` pegado abajo)

### entorno.txt (baseline)
```
etiqueta baseline
fecha 2026-09-13T11:44:14+02:00
commit bb1e8a8a54d9bcaf0f1c49dd73c10b6934d6f66f
src_y_scripts_vs_3c1918f vacio
exo exo 0.1.0
Linux paul-vostro-15-3530 7.0.0-31-generic #31~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Mon Aug 10 09:38:02 UTC 2 x86_64 x86_64 x86_64 GNU/Linux
nproc 12
uptime  11:44:14 up 12:36,  1 user,  load average: 0,37, 1,35, 2,17
hyperfine 1.20.0
jq-1.7
git version 2.43.0
```

### entorno.txt (después)
```
etiqueta despues
fecha 2026-09-13T13:13:47+02:00
commit 41e01bfde29176748a5df328f67651ce3b8f6e7c
src_y_scripts_vs_3c1918f CAMBIOS
exo exo 0.1.0
Linux paul-vostro-15-3530 7.0.0-31-generic #31~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Mon Aug 10 09:38:02 UTC 2 x86_64 x86_64 x86_64 GNU/Linux
nproc 12
uptime  13:13:47 up 14:05,  1 user,  load average: 0,81, 6,83, 5,90
hyperfine 1.20.0
jq-1.7
git version 2.43.0
```

**No ejecutadas en esta campaña** (documentado, no es un vacío): Task 8
(D1=A1, la guarda no necesita `--db`), Task 10 (puerta C-H17a CERRADA en la
línea base: `s4 n5000 p95=47 ≤ 250`), Task 12 (D5 pendiente: sin la medición
de W11, la puerta C-H10 solo tiene el dato Linux) y Task 15 (manual de Paul,
D5). El código de estas tres decisiones **no está en la rama**.
**Actualización 2026-09-15:** Task 15 corrida en W11 y C-H10 CERRADA en las
dos máquinas ⇒ la Task 12 queda **no ejecutada por puerta**, no pendiente
(ver la sección W11).

## Predicciones (línea base)
```
P1 (H27: tope KNN a N>=1000, ok a 174)	PASA
P2 (s2 n174 p50=952)	CUMPLIDA
P3 (refresh n174 = 969-952)	CUMPLIDA
P4 (s7 n174 p50=4)	CUMPLIDA
PUERTA C-H17a (s4 n5000 p95=47)	CERRADA: Task 10 no se ejecuta
PUERTA C-H10 Linux (s7 n174 p50=4)	CERRADA (falta W11, D5)
```

## Criterios
```
C-H27	PASA
C-H4 (s3 n5000 67→28)	PASA
C-H17a (s4 n5000 p95 47→49)	NO APLICA (puerta cerrada)
C-H17b (s2 n5000-n174 = 10828-995)	DERIVAR a C/backlog
C-H23 (s8 n5000 p50=326)	NO ABRIR
C-noregresión	PASA
```

## Tabla antes/después

| id | p50 antes | p50 después | p95 antes | p95 después | fallidas antes | fallidas después |
|---|---|---|---|---|---|---|
| s10-jq-log-l200000 | 532 | 530 | 540 | 546 | 0 | 0 |
| s10-jq-log-l20000 | 57 | 57 | 61 | 61 | 0 | 0 |
| s10-jq-log-l2174 | 12 | 12 | 17 | 17 | 0 | 0 |
| s1b-query-refresh-sim0-n1000 | 979 | 2985 | 1027 | 3025 | 20 | 0 |
| s1b-query-refresh-sim0-n174 | 981 | 1001 | 986 | 1019 | 0 | 0 |
| s1b-query-refresh-sim0-n5000 | 1057 | 10834 | 1084 | 11091 | 20 | 0 |
| s1-query-refresh-n1000 | 977 | 2788 | 988 | 2838 | 20 | 0 |
| s1-query-refresh-n174 | 969 | 1002 | 988 | 1013 | 0 | 0 |
| s1-query-refresh-n5000 | 1050 | 10768 | 1055 | 11035 | 20 | 0 |
| s2-query-n1000 | 962 | 2779 | 982 | 2841 | 20 | 0 |
| s2-query-n174 | 952 | 995 | 972 | 1010 | 0 | 0 |
| s2-query-n5000 | 980 | 10828 | 998 | 11131 | 20 | 0 |
| s3-index-sin-cambios-n1000 | 20 | 11 | 26 | 16 | 0 | 0 |
| s3-index-sin-cambios-n174 | 8 | 4 | 10 | 7 | 0 | 0 |
| s3-index-sin-cambios-n5000 | 67 | 28 | 68 | 32 | 0 | 0 |
| s4-arranque-content-n1000 | 18 | 11 | 21 | 20 | 0 | 0 |
| s4-arranque-content-n174 | 8 | 8 | 9 | 10 | 0 | 0 |
| s4-arranque-content-n5000 | 43 | 42 | 47 | 49 | 0 | 0 |
| s5-search-fts-n1000 | 8 | 10 | 10 | 11 | 0 | 0 |
| s5-search-fts-n174 | 3 | 3 | 6 | 6 | 0 | 0 |
| s5-search-fts-n5000 | 11 | 13 | 21 | 20 | 0 | 0 |
| s6-hook-entero-n1000 | 1019 | 2959 | 1033 | 3059 | 0 | 0 |
| s6-hook-entero-n174 | 1060 | 1079 | 1071 | 1093 | 0 | 0 |
| s6-hook-entero-n5000 | 1097 | 5055 | 1135 | 5071 | 0 | 0 |
| s7-config-jq-n1000 | 3 | 4 | 4 | 4 | 0 | 0 |
| s7-config-jq-n174 | 4 | 4 | 5 | 4 | 0 | 0 |
| s7-config-jq-n5000 | 3 | 4 | 5 | 5 | 0 | 0 |
| s8-kb-precommit-n1000 | 108 | 107 | 122 | 118 | 20 | 20 |
| s8-kb-precommit-n174 | 64 | 64 | 68 | 68 | 20 | 20 |
| s8-kb-precommit-n5000 | 328 | 326 | 342 | 340 | 20 | 20 |
| s9-git-log-una-nota-n1000 | 2 | 0 | 2 | 1 | 0 | 0 |
| s9-git-log-una-nota-n174 | 0 | 2 | 1 | 3 | 0 | 0 |
| s9-git-log-una-nota-n5000 | 3 | 3 | 3 | 3 | 0 | 0 |

## Derivaciones

- **C-H17b → DERIVAR.** Item para la campaña C: `s2 n5000−n174 = 10828 − 995 =
  9833 ms` (KNN/HashMap de todos los trozos en `buscador.rs:286-294`, KNN en
  `:286`). Este número no existía en la línea base porque el KNN fallaba con
  exit 1 antes de tocar ese código (H27); al arreglarse H27 (Task 3), el coste
  real de recorrer 95.000 trozos queda expuesto por primera vez. **No se
  arregla en A**, tal y como fija el pre-registro: es fichero de C
  (`buscador.rs`), no de A.
- **C-H23 → ninguna.** `s8 n5000 p50 = 326 ms`, muy por debajo de los 2.000 ms
  del umbral. No se abre item de backlog.
- H29 (`.git` en el walker): `s3 n5000 p50` = 67 ms (antes) → 28 ms (después).
  Solo es dato; sin criterio pre-registrado, no decide. La caída no es efecto
  del walker (H29, no tocado en esta campaña) sino de H4 (Task 6:
  `resuelve_destinos` sin escrituras nulas), que es lo único que cambió en el
  camino de `exo index`.
- `rebuild` extrapolado: `s9 n5000 p50 × 5000 = 3 ms × 5000 = 15.000 ms` (15 s)
  en `git log` por nota, sobre 5.000 notas. Es una extrapolación lineal del
  coste de `git_epoch_de` por nota, **no una medida** de un `exo rebuild` real
  (que además re-embebe ~95.000 trozos; fuera de alcance del pre-registro).
- H5: `s10 l2174 / l20000 / l200000 p50 = 12 / 57 / 530 ms` → dato para D3.
  Sin cambio antes/después porque D3=T (Task 11, variante tail) no toca el
  filtro `jq` que mide `s10`, solo cuántas líneas le llegan en producción.

## W11 (Task 15)

Corrido el **2026-09-15** en la W11 de Paul (Git Bash), `exo 0.1.0`, plugin
`exo@exo 1.1.2 @ ba4b75f`. Salida literal y observaciones en
`evals/recall-coste/results/w11-2026-09-15.txt`.

```
p50_ms 2312 p95_ms 2568
config_jq_p50_ms 68
```

- **C-H10, mitad W11: `config_jq_p50_ms = 68 ≤ 100` ⇒ CERRADA.** Con la mitad
  Linux también cerrada (`s7 n174 p50 = 4 ≤ 30`), la puerta queda **CERRADA
  en las dos máquinas: la Task 12 no se ejecuta**. `exo config` + `jq` es un
  3% del hook en W11.
- **Observación, no decide:** el hook entero en W11 (p50 2312 ms) duplica al
  de Linux (`s6 n174` 1079 ms). `exo recall` en caliente son ≈1,1 s de reloj;
  el ≈1,2 s restante es coste de spawn de Git Bash (≈20 procesos a 25-60 ms).
  Margen frente al timeout de 5 s del hook: ≈2,4 s en p95.
- **Observación, no decide (H3):** `recall-latencia.sh` mide
  `elapsed_ms + refresh_ms`, que en W11 ve ≈1,0 s de ≈2,3 s reales. El
  criterio de reapertura (p95 > 1.500 ms) no dispararía en W11 con la latencia
  medida hoy. Va al backlog; no se cambia aquí un umbral pre-registrado.
- Task 15 Step 3: `recall-latencia.sh` sobre el log de W11 → `INSUFICIENTE`
  (3 disparos con `elapsed_ms`).

## Lo que buscó este veredicto para objetar

- **C-H27 (PASA):** se buscó el mensaje `k value in knn query too large` en
  el `stderr` de los seis escenarios afectados (`s1`, `s1b`, `s2` × `n1000`,
  `n5000`) en `despues/` — no aparece en ninguno (`grep` vacío) — y se
  confirmó `rc=0` en los seis `.rc` directos, no solo en el resumen de
  hyperfine. No se encontró nada que objete el PASA.
- **C-H4 (PASA):** se comprobó que la línea base no caía en el caso «sin
  margen» (`67 ms > 50 ms`, así que el criterio aplica y no queda VACÍO) antes
  de aceptar el PASA `28 ≤ 0,6 × 67 = 40,2`.
- **C-noregresión (PASA):** el criterio pre-registrado solo cubre `s5` y `s3`.
  Se revisó a mano el resto de escenarios no regresados (`s4`, `s7`, `s9`,
  `s10`) buscando una subida oculta que el criterio no vería: ninguno sube más
  allá del ruido de hyperfine (±1-2 ms). `s8` falla 20/20 en ambas corridas
  con el mismo mensaje de `kb-precommit.sh` («deja el commit pendiente y
  díselo a Paul») — comportamiento preexistente del gate, no una regresión de
  esta campaña, y fuera de lo que `C-noregresión` decide.
- **Los saltos grandes en `s1`/`s1b`/`s2`/`s6` a `n1000` y `n5000`** (de ~1 s a
  2,8–11 s) se buscaron como posible bug de doble medición o de un
  `--refresh` descontrolado: se descartó comparando `s1` (con refresh) contra
  `s2` (sin refresh) en `despues` — la diferencia entre ambos a cada `N` se
  mantiene pequeña (~200 ms), igual que en la línea base, así que el salto no
  es el refresh: es el KNN real que antes fallaba y ahora corre. Coherente con
  que `s6` (el hook entero) escale en la misma proporción que `s1`.
- **`entorno.txt` de `despues`:** se verificó `src_y_scripts_vs_3c1918f
  CAMBIOS` (esperado: hay 12 tasks integradas) y que la carga de 1 minuto al
  arrancar la corrida (0,81) seguía por debajo de 1,0 pese a que la carga de 5
  y 15 minutos (6,83 / 5,90) todavía arrastraba la sesión previa — se dejó
  correr porque el criterio del plan mira el primer minuto, no las ventanas
  largas, y así se verificó antes de lanzar (Step 1).
- **Qué no se pudo objetar por falta de dato:** la puerta C-H10 solo tiene el
  lado Linux (`s7 n174 p50 = 4 ms`, muy por debajo de 30); sin W11 no hay con
  qué objetar el otro lado, y el veredicto lo deja explícito como «no medido
  (D5)» en vez de asumir que Windows se comporta igual. **Cerrado el
  2026-09-15** con la medición de la sección W11: Windows **no** se comporta
  igual (config+jq 68 ms frente a 4), pero no llega al umbral de 100.
- **W11 (2026-09-15):** la primera corrida del bloque, con la ruta del hint
  (`plugins/exo/scripts`), dio `p50_ms 40` y exit 0 **sin que el hook llegara
  a existir**. Se descartó por el `No such file or directory` de stderr y por
  el log vacío, y se repitió con `<installPath>/scripts`. El 2312 ms vale
  porque las 20 iteraciones dejaron 20 `recall-inject-emitted`.
