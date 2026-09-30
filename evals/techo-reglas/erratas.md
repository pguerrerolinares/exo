# Erratas — test de techo de reglas

Errores posteriores al sello del pre-registro (`1921aee`). Ninguno cambia umbrales ni criterios.

## E1 — `gold/s1/g2-154/check.sh` modificado y restaurado tras las corridas (2026-09-30)

- **Qué pasó:** el subagente que recogía la evidencia para el verdict ejecutó `sed -i` con una ruta relativa sobre `~/.cache/exo-ablacion-k/gold/s1/g2-154/check.sh`. La línea 27 pasó de `g.get("findings")` a `g.get("problems")`.
- **Cuándo:** después de que la tanda terminara y de que el resultado se commiteara (`2fc8424`). Los `check.rc` de g2-154 (1 y 1) se calcularon con el check original.
- **Restauración:** se copió desde `gold-activo.tar` del tarball de K (`~/.cache/exo-ablacion-k-registro.tar.gz`). Con `cmp`, los 17 `check.sh` de `tareas.tsv` son idénticos a `gold-activo.tar`.
- **Efecto sobre el gate:** ninguno.
