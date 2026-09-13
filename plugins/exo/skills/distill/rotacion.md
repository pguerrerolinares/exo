# Rotación de bitácoras

**Cuándo cargar:** en el paso 0, cuando la corrida en seco trae
`data.rotations` no vacío.

Corre primero en seco y revisa el resultado:

    $KBX_BIN rotate --kb $KB_ROOT --json

Requiere un build de `kbx` que incluya `rotate`: el binario instalado puede no
traer todavía el subcomando, porque la feature vive en una rama sin mergear.
Si no está disponible, sáltalo y continúa directo al paso 1.

Si `data.rotations` viene vacío, no hay nada que rotar: sigue directo al paso
1. Si trae entradas, repite con `--apply` y verifica antes de continuar:

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
