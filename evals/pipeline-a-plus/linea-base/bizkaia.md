# Línea base de mutation score: bizkaia-now

## Rama y rango
- Rama principal del repo: `master` (no `main`; el pre-registro dice `main`, desviación de nombre solamente).
- Merges a master del 2026-09-01 al 2026-09-28: `aa7bf2c` (2026-09-28, B1), `00a23b4` (2026-09-23, sub-proyecto A) y `53458af` (2026-09-23, merge(ola6) interno de la rama A).

| Merge | Plan con bloques de código | Diff con producción mutable | Resultado |
|---|---|---|---|
| aa7bf2c B1 mapa (2026-09-28) | sí: `docs/superpowers/plans/2026-09-24-bizkaia-now-b1-mapa.md` (66 fences) | sí: Python en `src/`, `scripts/`, `alembic/`; TS/TSX en `web/` | **ELEGIDA** (la más reciente) |
| 00a23b4 sub-proyecto A (2026-09-23) | sí: `2026-09-23-bizkaia-now-data-pipeline.md` | sí | descartada: hay una más reciente |
| 53458af merge(ola6) | (interno de A) | | descartada: es un merge interno de la rama A, no una rama a master |

- Rango: `merge-base(aa7bf2c^1, aa7bf2c^2)..aa7bf2c^2` = `cfc281b..73f9bfb` (35 commits). Hay merge commit, así que no se aplicó la regla de rango alternativa.

## Comando
`EXO_MUTATION_TIMEOUT=3600 PATH=<venv3>/bin:<js>/node_modules/.bin:<cargo>/bin:$PATH review-package cfc281b 73f9bfb .../linea-base/bizkaia.diff`, cwd en el repo.
Tiempo (`time`): **1:00:06,87** de reloj (681 s de usuario). Repo intacto: `git status` limpio, sin worktrees residuales.

## Sección MUTACIÓN (literal)
```
## MUTACIÓN
MUTACIÓN: parcial (ver proyectos)
MUTACIÓN [raíz]: parcial (timeout 3600s)
MUTACIÓN [web]: no disponible (stryker no instalado)
```
**Score oficial: ninguno. La rama es `parcial` y NO entra en el agregado** (regla del pre-registro).
- `raíz` (Python, mutmut 2.5.1): 915 mutantes generados; el timeout de 3600 s cortó la corrida.
- `web` (TS/TSX, 22 ficheros de producción): `web/package.json` de HEAD no declara stryker. No es un problema de `node_modules`: no hay herramienta de mutación configurada en el proyecto.

## Datos parciales (NO oficiales, solo para interpretar)
Se copió el `.mutmut-cache` a los ~3400 s, antes del timeout. Estado: 200 de 915 mutantes evaluados, 79 muertos y 121 supervivientes (60% de los evaluados sobrevive). mutmut recorre los ficheros en orden fijo, así que la muestra es sesgada: migraciones alembic, `scripts/rematerialize.py` y `api/routes/diff.py`. Faltan `schemas`, `window`, `pipeline/*`, `dedup`, etc. **No extrapolar a un score.**

## Clasificación de 10 supervivientes (del parcial; IDs de mutmut)
| # | Mutante | Fichero:línea | Clase | Porqué |
|---|---|---|---|---|
| 7 | nombre de constraint en `drop_constraint` de upgrade | alembic/versions/0003_diff_verdict_four_verdicts.py:17 | no alcanzable (en la medición) | la BD de test ya está en `head`; `upgrade head` es no-op y la migración no se ejecuta. Con BD virgen en cada mutante caería. |
| 13 | ídem en `downgrade` | 0003…py:26 | muerto/no alcanzable | ningún test ejecuta `downgrade`. |
| 24 | `depends_on = None` a `""` | 0004_canonical_event_start_time_known.py:15 | equivalente | alembic trata `""` y `None` igual (sin dependencias). |
| 34 | `engine = build_engine(url)` a `None` | scripts/rematerialize.py:49 | hueco real | el script solo lo cubre la E2E `tests/e2e/process/`, excluida de `pytest` por defecto (`--ignore=tests/e2e`). |
| 46 | `sum(judged.missing)+sum(paired.missing)` con `-` | rematerialize.py:79 | hueco real | el aborto por juicios faltantes no tiene test en la suite por defecto. |
| 62 | `by_mode["classic"]` a `["XXclassicXX"]` | rematerialize.py:109 | hueco real | igual: solo E2E. |
| 92 | `_START_TIME_TOLERANCE_MIN` 5 a 6 | api/routes/diff.py:24 | hueco real | ningún test en el borde de la tolerancia de 5 min. |
| 98 | `["is_leisure"]` a `["XXis_leisureXX"]` | diff.py:50 | hueco real | la razón is_leisure de un jev sin clásico solo la cubre la E2E TE4 (ignorada por defecto). |
| 116 | `date_from is None` a `is not None` en el atajo sin filtros | diff.py:96 | equivalente (casi) | el atajo es una optimización; sin él, `any(...)` da True con candidatos no vacíos (siempre hay jev o algún clásico). |
| 130 | `alias="from"` a `"XXfromXX"` | diff.py:107 | hueco real | ningún test de unidad usa `?from=` contra `/diff`. |
| (160) | `continue` a `break` en el bucle de clásicos sin jev | diff.py:166 | hueco real | fuera del top 10 pedido; anotado por su relevancia (dropea diffs). |

Lectura: de los 10 clasificados, 5 (o 6) son huecos reales asociados a código que solo cubre la E2E excluida por defecto, 2 no alcanzables (migraciones) y 2 equivalentes. Patrón dominante: la protección de B1 vive en `tests/e2e/`, que `pytest` ignora y que mutmut no ejecuta.

## Desviaciones y notas de entorno
1. Rama `master`, no `main`.
2. Instalación de dependencias solo en el scratchpad: `venv3` (Python 3.12, deps de `pyproject.toml` de HEAD + mutmut 2.5.1). Ni `venv` (mutmut 3.8.0) ni `venv2` (2.5.1, pero sin deps del proyecto) servían tal cual.
3. Un test (`test_generate_candidate_pairs_order_is_independent_of_hash_seed`) lanza un subproceso con `env` vacío, así que necesita el paquete importable sin `PYTHONPATH`. Se añadió al `venv3` un `.pth` que antepone `<cwd>/src` a `sys.path`, para que importe el código del worktree mutado y no una copia instalada. No toca el repo.
4. BD: contenedor propio `postgis/postgis:16-3.4-alpine` en el puerto 55432 (el de `docker-compose.test.yml`), borrado al acabar.
5. **Primera corrida (`bizkaia-run1.diff`, 1:31)**: `MUTACIÓN: parcial (ver proyectos)`, raíz `parcial (salida no reconocida)`. El baseline salió rojo: `tests/test_alembic_env.py::test_alembic_check_ignores_postgis_objects` falla porque la imagen fija `search_path` de la BD a `"$user", public, topology, tiger`; alembic refleja las tablas de tiger/topology con `schema=None` y `include_object` (que filtra por schema) no las excluye. Es un fallo del repo con esta imagen, independiente de A+. Se aplicó `ALTER DATABASE … RESET search_path` (arreglo de entorno, no parámetro de `review-package`) y se repitió: es la corrida oficial (`bizkaia.diff`). Ambas se declaran; ambas dan parcial.
6. Sin reintento: aunque la rama acabe `parcial`, no se reintentó con otros parámetros (p. ej. más tiempo). Con ~90 s/mutante (la suite completa tarda ~60 s) los 915 mutantes necesitarían ~20 h en serie.
7. Los datos parciales y la clasificación salen del `.mutmut-cache` copiado antes del timeout (`bizkaia-mutmut-cache-parcial.sqlite`), no de la salida de `review-package`, que en `parcial` no lista supervivientes.

## Consecuencia para el pre-registro
bizkaia-now no aporta al agregado (Σcaught/Σviables). Para medir el pipeline pre-A+ aquí haría falta mutmut con `--paths-to-mutate` acotado o un timeout de muchas horas; ambos cambian el pre-registro, y la decisión es de Paul.
