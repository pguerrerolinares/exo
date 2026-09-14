# Rotación de bitácoras

**Cuándo cargar:** en el paso 0, cuando la corrida en seco trae
`data.rotations` no vacío.

Corre primero en seco y revisa el resultado:

    $EXO_BIN rotate --kb $KB_ROOT --json

Si `data.rotations` viene vacío, no hay nada que rotar: sigue directo al paso
1. Si trae entradas, repite con `--apply` y verifica antes de continuar:

**Si `--apply` sale con exit 1**: no es "nada se aplicó" — `rotate_cmd`
(`engine/src/main.rs::rotate_cmd`) sigue barriendo `log/` aunque una nota
falle (frontmatter sin cerrar, fichero ilegible) y solo hace `bail!` (exit 1)
**después** de emitir el envelope. El envelope en stdout ya lista las
rotaciones que SÍ se aplicaron (`data.rotations`); cada nota que falló va
por stderr, una línea `rotate: <ruta>: <error>` por fallo. Revisa esas notas
a mano (por qué falló, si hace falta cerrar un frontmatter roto) antes de
commitear — **no vuelvas a correr `--apply` a ciegas**: reintentar sin
entender el fallo puede repetir el mismo error, y las notas que sí rotaron
ya movieron bytes a `archive/log/` en esta misma corrida.

1. `git -C $KB_ROOT status --porcelain` —
   deben aparecer las bitácoras modificadas y los nuevos ficheros en
   `archive/log/`.
2. Conservación: para cada bitácora tocada, compara el número de cabeceras
   `## ` de **antes** de rotar —
   `git -C $KB_ROOT show HEAD:<ruta-de-la-nota> | grep -c '^## '`
   (el cambio aún no está commiteado, así que `HEAD` sigue teniendo el
   contenido previo al `--apply`) — contra la suma de cabeceras `## ` en la
   nota viva actual más las del fichero nuevo en `archive/log/`
   (`grep -c '^## ' <ruta-nota-viva> <ruta-archivo-nuevo>`, sumando ambos
   conteos). Deben coincidir. Si no cuadra, hay una entrada perdida — no
   sigas, revísalo.
3. Nada se borra, misma regla de oro que el resto de la skill. Si algo salió
   mal y hay que revertir:
   - Nota viva: `git -C $KB_ROOT checkout -- <ruta-de-la-nota>`
     (con el pathspec explícito de la nota — `git checkout --` sin ruta detrás
     no hace nada y no avisa).
   - Ficheros nuevos en `archive/log/`: `checkout` no los toca porque están
     sin trackear; bórralos a mano con las rutas exactas que ya listó
     `git status --porcelain` en el punto 1 (p.ej.
     `rm <ruta-nueva-en-archive-log>`).
   - Verifica con `git -C $KB_ROOT status --porcelain`
     que no queda nada pendiente, y repórtalo — no continúes al paso 1 con el
     repo en ese estado.

Va antes del budget check a propósito: mueve bytes fríos fuera de las notas
calientes, así que el paso 1 evalúa el presupuesto ya sobre el estado
reducido.
