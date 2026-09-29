# Línea base de mutation score, 2ª medición: bizkaia-now

- Enmienda 1, instrumento `review-package` en `026c166` (verificado). Rama B1 mapa (`aa7bf2c`), rango `cfc281b..73f9bfb` (35 commits), `master`.
- Parámetros: EXO_MUTATION_TIMEOUT=3600, EXO_MUTATION_MUTANT_TIMEOUT=60, EXO_MUTATION_EXCLUDE='alembic/versions/**'. mutmut 2.5.1 (venv3). Secuencial, después de music.
- Tiempo del script (`time`): **60:01,1** (user 10:38,8). Timeout de 3600 s.
- Mutantes acotados al diff (12 ficheros .py de `src/` y `scripts/`, sin migraciones): **285** (frente a 915 de antes).

## MUTACIÓN (literal)
```
## MUTACIÓN
MUTACIÓN: parcial (ver proyectos)
MUTACIÓN [raíz]: parcial (timeout 3600s) — progreso: 269/285 evaluados, 62 caught, 207 supervivientes
MUTACIÓN [web]: no disponible (stryker no instalado)
```
Progreso (no oficial, ver enmienda): 269/285, 62 caught (23%). `web`: `web/package.json` de HEAD no declara stryker, sigue `no disponible`.

## Muestreo (disparado por `parcial (timeout …)`, según la enmienda)
- n=150 sin reemplazo de los 285 ids de mutmut (1..285, tal como los lista la herramienta; comprobado contiguos), `random.Random(20260929).sample(ids, 150)`. Muestra en `bizkaia-muestreo/muestra.txt`.
- mutmut 2.5.1 sí permite correr ids sueltos (`mutmut run <id>`). Cada id se corrió con el mismo runner que el script: `cto 60 python -m pytest -x --assert=plain` (timeout 60 s por mutante, corte = caught).
- Resultado: **33 caught / 150 = 22,0%**; **IC95 Wilson [16,1%, 29,3%]**. 117 supervivientes, 0 timeouts, 0 sospechosos. Tiempo del muestreo: ~49 min.
- Coherente con el progreso parcial del script (23%). Es una tasa por mutante sobre el conjunto acotado, no entra en el agregado (rama `parcial`), se reporta aparte.

## Clasificación de 10 supervivientes (los 10 primeros ids supervivientes de la muestra)
| id | Fichero:línea | Mutación | Clase | Porqué |
|---|---|---|---|---|
| 3 | scripts/rematerialize.py:39 | clave `"classic"` a `"XXclassicXX"` | hueco real | `rematerialize.py` solo lo cubre la E2E (`tests/e2e/`, ignorada por `pytest` por defecto) |
| 4 | rematerialize.py:39 | `groups = None` | hueco real | idem |
| 5 | rematerialize.py:46 | `engine = None` | hueco real | idem |
| 6 | rematerialize.py:47 | `session_maker = None` | hueco real | idem |
| 8 | rematerialize.py:53 | `== "is_leisure"` a `!=` | hueco real | idem |
| 12 | rematerialize.py:61 | `evaluator = None` | hueco real | idem |
| 16 | rematerialize.py:72 | `paired = None` (sin `pair_events`) | hueco real | idem |
| 17 | rematerialize.py:77 | `+` a `-` en `total_missing` | hueco real | el aborto por juicios faltantes no tiene test en la suite por defecto |
| 19 | rematerialize.py:79 | texto de cabecera `=== rematerialize.py ===` | hueco real (bajo valor) | salida impresa no comprobada |
| 22 | rematerialize.py:86 | `before['jev']` a `before['XXjevXX']` | hueco real | idem, E2E-only |

Lectura: los 10 son huecos de la suite por defecto, todos en `rematerialize.py`. Sesgo: son los ids más bajos, y mutmut ordena por fichero, así que la lista no es representativa del resto. Vistos de pasada en la muestra, fuera del top 10: 122 (`decode("ascii")`, schemas.py:77), 152 (`is not None or` en window.py:56), 185 (`>=` a `>` en window.py:94), 212 (`self._threshold = None`, evaluator.py:113), que apuntan también a huecos. Sin clase equivalente ni muerto en estos 10.

## Desviaciones y notas de entorno
1. Rama `master`, no `main`.
2. `whatthepatch 1.0.7` no estaba en `venv3` (la vez anterior tampoco se había comprobado): se instaló ahí antes de correr (pip, scratchpad). Arreglo de entorno preventivo, no reintento. La corrida oficial fue la única del script.
3. Contenedor `postgis/postgis:16-3.4-alpine` en el puerto 55432 creado y **borrado** por mí (`linea-base2-pg`). Se aplicó de entrada `ALTER DATABASE bizkaia_now_test RESET search_path` (el arreglo documentado en la 1ª medición, evita el baseline rojo). `.pth` de `venv3` (antepone `<cwd>/src`) reutilizado.
4. Para el muestreo se extrajo `73f9bfb` con `git archive` a `bizkaia-muestreo/wt/` (el worktree del script se borra) y se regeneró el mismo conjunto de 285 mutantes con `--runner true` para tener el cache con los ids. Se pusieron los estados a `untested` (mutmut reutiliza estados cacheados y devolvía "survived" en 1 s) y se corrió cada id con el runner real. Un primer intento de esa fase salió mal por eso (todo cacheado) y se descartó sin usar sus datos; solo cuenta la segunda pasada. Los ids coinciden con la corrida oficial (mismo patch, mismos ficheros, mismos 285).
5. Repos intactos (`git status` limpio, `git worktree list` solo el árbol principal).
