# Línea base de mutation score: pguerrero-music

- Rama: feat/portafolio-playlists (merge 275b29f, 2026-09-24). El repo usa `master`, no `main` (desviación de nombre).
- Rango: BASE c4a1ba1d1c512d443e6ccd60be85cd02fc78e14c (merge-base del primer padre) .. HEAD c0bb059 (segundo padre). 12 commits.
- Ficheros mutados: build_playlists.py, discovery.py, families.py, prune_playlists.py, sync_library.py (923 mutantes según mutmut).
- Tiempo: 1:00:01 (time; tope EXO_MUTATION_TIMEOUT=3600).
- Herramienta: mutmut 2.5.1 (venv2 del scratchpad). Baseline de tests en el worktree: 447 passed.

## Selección
| Merge (2026-09-01..28) | Plan en docs/superpowers/plans | Decisión |
|---|---|---|
| 275b29f feat/portafolio-playlists (09-24) | 2026-09-24-portafolio-playlists.md (34 marcas ```) | SELECCIONADA: más reciente, diff toca .py de producción |
| e419ea0 n7-housekeeping (09-13) | no hay plan en plans/ | descartada |
| 9638adf n4-dedupe-merge (09-13) | no hay plan | descartada |
| d3ccb3b n3-health-warnings (09-13) | no hay plan | descartada |
| 617ecbd n2-subprocess-safety (09-13) | no hay plan | descartada |
| 26e6c52 n1-shared-persistence (09-13) | no hay plan | descartada |
| 887c839 fabrica/roadmap (09-13) | no hay plan | descartada |
| 36fcde4 chore/yt-dlp (09-13) | no hay plan | descartada |
(Planes existentes: solo 2026-08-04 (fuera de rango) y 2026-09-24.) Como ya había una candidata más reciente, no hizo falta más comprobación de las N1-N7.

## MUTACIÓN (literal)
MUTACIÓN: parcial (timeout 3600s)

## Progreso observado (línea de mutmut, no entra en el score)
Última línea de progreso: 129/923, 🎉109 ⏰0 🤔7 🙁13. El contador quedó clavado en 129 desde las 07:48 (unos 10 min tras arrancar) hasta el timeout; siguieron apareciendo procesos pytest, pero mutmut no avanzó ni escribió `.mutmut-cache`. Causa no diagnosticada (posible mutante colgado). La medición corrió en paralelo con la de bizkaia-now (carga compartida), no se sabe si influyó.

## Clasificación de supervivientes
No realizada: la medición es `parcial`, el worktree temporal se borra al terminar y `mutmut results` no llegó a ejecutarse (el script solo lo lee si obtiene la línea final). Pre-registro: sin reintento con otros parámetros. Solo consta el recuento parcial (13 supervivientes, 7 sospechosos de 129 evaluados), sin fichero:línea.

## Desviaciones
1. Rama principal `master`, no `main`.
2. Dependencias del proyecto (ytmusicapi, yt-dlp 2026.8.19, mutagen, beets, pyacoustid) instaladas en $S/venv2 (scratchpad), no en el sistema; faltaban. Resultado: ytmusicapi 1.12.3, yt-dlp 2026.8.19, mutagen 1.48.1, beets 2.14.1, pyacoustid 1.3.1 (versiones libres, no las del uv.lock).
3. Resultado `parcial`: según el pre-registro no entra en el agregado.
4. Ejecución concurrente con otra medición (bizkaia-now).
5. Repo intacto: sin commits ni cambios, `git worktree list` solo muestra el árbol principal.
