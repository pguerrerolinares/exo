# Línea base de mutation score, 2ª medición: pguerrero-music

- Enmienda 1 (`linea-base-enmienda-1.md`), instrumento `review-package` en `026c166` (verificado con `git log -1 --format=%h -- .../review-package`).
- Rama: feat/portafolio-playlists, rango BASE c4a1ba1d1c512d443e6ccd60be85cd02fc78e14c .. HEAD c0bb059 (12 commits, mismo que el verdict).
- Parámetros: EXO_MUTATION_TIMEOUT=3600, EXO_MUTATION_MUTANT_TIMEOUT=60, EXO_MUTATION_EXCLUDE vacío. mutmut 2.5.1 + whatthepatch 1.0.7 (venv2 del scratchpad, con las deps de music). Corrida secuencial (sin nada más en marcha).
- Tiempo (`time`): **17:40,7** de reloj (user 7:58,9). Sin timeout, sin reintento, sin muestreo.
- Mutantes acotados al diff: **212** (frente a 923 de la vez anterior, mutmut sobre ficheros enteros).

## MUTACIÓN (literal)
```
## MUTACIÓN
MUTACIÓN: 139/212 (65%) — 73 superviviente(s)
```

Score: 139/212 = 65,6% (el script imprime 65% truncado). Entra en el agregado (no es parcial).

## Clasificación de 10 supervivientes
`review-package` solo lista ids de mutmut (truncado a 20 líneas de `mutmut results`, worktree borrado). Para leer los mutantes se extrajo `c0bb059` con `git archive` a `linea-base-2/music-clasif/` y se regeneró el mismo conjunto (212, mismo patch) con `--runner true`, solo para `mutmut show <id>`. No es una remedición, no cambia el score. Los ids 4..69 salen del listado de la corrida oficial.

| id | Fichero:línea | Mutación | Clase | Porqué |
|---|---|---|---|---|
| 4 | build_playlists.py:208 | `continue` a `break` (hidden_ids) | hueco real | ningún test tiene un playlist oculto seguido de otros no ocultos |
| 5 | build_playlists.py:264-265 | `written = build_playlists(...)` a `None` en `main()` | hueco real | `main()` no se ejecuta en tests |
| 30 | discovery.py:565 | texto del aviso de retiradas | hueco real (bajo valor) | el mensaje a stderr no se comprueba |
| 32 | discovery.py:566 | separador `", "` del aviso | hueco real (bajo valor) | idem |
| 56 | families.py:48 | texto del aviso de familia < 2 | hueco real (bajo valor) | mensaje no comprobado |
| 57 | families.py:49 | idem, 2ª mitad | hueco real (bajo valor) | idem |
| 58 | families.py:50 | `continue` a `break` (familia < 2) | hueco real | ningún test con una familia de 1 miembro seguida de otra válida |
| 61 | families.py:53 | clave `"limit"` a `"XXlimitXX"` | hueco real | los tests con `limit` no sondean el valor propagado (o coincide con DEFAULT_LIMIT) |
| 67 | families.py:65 | `.strip("-")` a `.strip("XX-XX")` | equivalente | el slug es `[a-z0-9-]`, `XX` nunca aparece, y `-` se sigue quitando |
| 69 | families.py:66 | `slug[:48]` a `slug[:49]` | hueco real | ningún test con título de slug > 48 caracteres |

Lectura: 9 huecos (5 de ellos mensajes a stderr o límites) y 1 equivalente. 0 código muerto en la muestra. Los ids 72 y 80 (hash `f"{day}:{videoId}"` sin pin, `continue`/`break` en :85) quedaron sin clasificar por el tope de 10.

## Desviaciones
1. Rama `master`, no `main` (igual que antes).
2. Dependencias solo en `venv2` del scratchpad (versiones libres, las mismas de la vez anterior).
3. Repo intacto: `git status` limpio y `git worktree list` solo el árbol principal.
4. La clasificación usó una regeneración de mutantes con `--runner true` (ver arriba); no forma parte de la medición.
