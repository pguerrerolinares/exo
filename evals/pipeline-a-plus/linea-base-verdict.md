# Verdict: línea base de mutation score, pipeline pre-A+

> Medido el 2026-09-29 según `linea-base-preregistro.md` (`5141d60`). Los informes por repo, con la tabla de candidatas y los descartes, están en `linea-base/`.

## Resultado

| Repo | Rama (merge) | Rango | Instrumento | Resultado | Tiempo |
|---|---|---|---|---|---|
| exo | campaña L (`7e29f42`, 09-20) | `5efe812..f79d817` | cargo-mutants 27.1.0 `--in-diff` | **13/13 (100%)** | 26:52 |
| pguerrero-music | `feat/portafolio-playlists` (`275b29f`, 09-24) | `c4a1ba1..c0bb059` | mutmut 2.5.1 | `parcial (timeout 3600s)`, 923 mutantes | 1:00:01 |
| bizkaia-now | B1 mapa (`aa7bf2c`, 09-28) | `cfc281b..73f9bfb` | mutmut 2.5.1; stryker | `parcial (timeout 3600s)`, 915 mutantes; `web`: `no disponible (stryker no instalado)` | 1:00:06 |

**Agregado del pre-registro:** 13/13 con n=1 rama. **No sirve como línea base.** Solo una rama entró en el agregado, con 13 mutantes viables y 0 supervivientes. No tiene resolución para detectar que A+ "empeore", porque el único movimiento posible es hacia abajo.

## Datos no oficiales (fuera del agregado; se reportan porque se observaron)

- **music:** el progreso de mutmut llegó a 129/923: 109 caught, 7 sospechosos y 13 supervivientes. Se quedó congelado en 129 desde las 07:48 hasta el timeout.
- **bizkaia:** 200/915, con 79 muertos y 121 supervivientes. La muestra está sesgada hacia migraciones, `rematerialize.py` y `diff.py`. De los 10 supervivientes clasificados, 6 son **huecos reales**: `rematerialize.py` y `diff.py` solo los cubre la E2E, que `pytest` ignora por defecto. Otros 2 no son alcanzables y 2 son equivalentes.

## Hallazgos sobre el instrumento (`review-package`, exo 1.5.0)

1. **mutmut muta ficheros enteros, no las líneas del diff.** Solo cargo-mutants admite `--in-diff`. Por eso en Python salen ~920 mutantes por rama: con el default de 600 s nunca termina, y con 3.600 s tampoco. En repos Python la mutación que A+ promete por tarea **no es operable** con el instrumento actual.
2. **Un mutante colgado bloquea toda la corrida.** En music el contador estuvo parado más de 40 min en 129. Falta diagnosticarlo. La hipótesis es un mutante que genera un bucle infinito sin timeout por mutante efectivo.
3. **Con timeout se pierde el progreso.** `parcial (timeout)` descarta el conteo parcial (caught/supervivientes) y `mutmut results` no se ejecuta antes de borrar el worktree temporal. Es un caso de la ley «degradar con forma válida»: la salida es correcta en forma pero tira un dato que existía.
4. **stryker depende de que el proyecto lo declare.** Si no está en el `package.json` de HEAD, el resultado es `no disponible`. El comportamiento es correcto, pero en la práctica deja fuera todo el TS de bizkaia.

## Desviaciones y ambigüedades del pre-registro (declaradas)

- **"Código de producción" no estaba definido.** En exo, el agente descartó las dos ramas más recientes (J fase 1, `01be694` y `aaf66c0`) porque su único código mutable es el harness Python de evals. Si ese harness cuenta como producción, la rama seleccionada habría sido `aaf66c0`.
- music y bizkaia usan `master`, no `main`. Se aplicó la misma regla.
- bizkaia: la primera corrida tuvo el baseline de tests en rojo por el `search_path` de la imagen postgis. Se arregló el entorno, no un parámetro del script, y la corrida oficial es la segunda.
- Las dependencias de los proyectos se instalaron solo en venvs del scratchpad. music y bizkaia corrieron a la vez y compartieron carga.

## Qué hace falta para tener línea base

Hay que enmendar el pre-registro antes de medir otra vez. La decisión es del dueño. Opciones:

- (a) Arreglar el instrumento, acotando mutmut a las líneas del diff o con muestreo aleatorio de mutantes con semilla e IC binomial.
- (b) Subir el timeout.
- (c) Base solo en Rust, con más ramas de exo.
