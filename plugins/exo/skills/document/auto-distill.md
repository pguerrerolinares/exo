# Auto-distill (brief del subagente)

**Cuándo cargar:** en el paso 5 de `SKILL.md`, cuando el gate mordió y no existe
`distill/auto/<slug>`. Quien corre `/document` lanza el subagente (`model:
sonnet`, contexto limpio, síncrono, tope 10 min) y le pasa el bloque de abajo
literal, sustituyendo `<kb>`, `<ruta>`, `<slug>`, `<techo>` y `<wt>`. `<ruta>` va
**relativa a la raíz de la KB** (la `path` de `search` y el trailer `Gate:` ya
lo son; si tienes una absoluta, relativízala). `<wt>` =
`${TMPDIR:-/tmp}/exo-distill-auto-<slug>`, fuera de la KB.

## Brief

Eres el auto-distill de la KB `<kb>`, para la nota `<ruta>` (relativa a la
raíz de la KB; siempre la lees, editas y añades como `<wt>/<ruta>`, nunca la
del checkout principal) (techo `<techo>` B:
`kbx_budget_max` del frontmatter o nominal del tier, core 8.500 / stable 12.500).
Objetivo: dejarla en ≤60 % del techo **sin perder nada**: mueves bloques
enteros verbatim; no resumes ni reescribes prosa.

1. Si ya existe la rama `distill/auto/<slug>`, es «ya lanzado»: termina sin
   error. Si no, crea un worktree en esa rama desde `HEAD`
   (`git -C <kb> worktree add <wt> -b distill/auto/<slug>`). **`--kb <wt>` en
   todo comando `exo`**: sin él `exo write` escribe en el checkout principal.
   Nunca escribas en la rama principal, ni merge, ni push.
2. Lee `<wt>/<ruta>` y `git -C <wt> log -p -n 10 -- <ruta>` (solo 10: el histórico
   entero pesa cientos de KB). Sin el log evictarías por edad, que es rotación
   por orden de llegada.
3. Evicción editorial. Pregunta por bloque: *¿qué párrafo ya no paga su sitio?*
   Candidatos: lo obvio, lo superado por una decisión posterior, el detalle de
   una iteración cuya conclusión ya está escrita, el ejemplo que repite el
   texto. No toques `## For agents` ni `## Reglas duras`. La edad no es motivo:
   un bloque de hace meses puede llevar la decisión que sigue mandando.
   **Test de salida, con cita literal o el bloque se queda:**
   - *cerrado*: su propio texto lo declara terminado sin nada pendiente
     («CERRADA», «MERGEADA», «descartado», «sustituido por»), o
   - *superado*: otra línea del canon o de la bitácora lo sustituye; cítala.

   Sin ninguna de las dos, se queda, y más aún si lleva una marca de decisión o
   regla (`Decidido`, `decisión`, `regla`, `nunca`, `siempre`, `CUARENTENA`) o
   describe un mecanismo en marcha (routine, hook, watchdog, cron, ledger,
   instalación local). Un bloque que mezcla lo cerrado con una decisión viva
   no se parte: se queda entero.
4. Cada bloque que sale del canon va a `<wt>/.distill/<slug>-moved.md`,
   verbatim, bajo su propia cabecera `## AAAA-MM-DD — movido del canon: <tema
   concreto>` (una por bloque; nunca varios bajo una genérica). **No toques la
   bitácora ni crees notas**: main sigue haciendo append a la bitácora y dos
   appends en EOF conflictan en cada merge. En la rama solo cambian el canon y
   el sidecar.
5. Mueve TODO lo que pase el test de salida, llegues o no al 60 %. Si con eso
   no llegas, no fuerces ni recortes: añade además una **propuesta de
   partición** (títulos que pasen el test del título de
   `distill/consolidacion.md`, qué secciones van a cada hija, bytes
   resultantes) en el cuerpo del mismo commit. Partir es decisión del
   mantenedor. Subir el techo (`kbx_budget_max`) no es alternativa y no la
   propongas: sin llegar al 60 %, la única salida es la partición.
6. Verifica antes de commitear: (a) conservación, todo bloque que sale del
   canon aparece literal en el sidecar; (b) `wc -c` de la nota ≤60 % del techo
   (solo si llegaste; si no, que la propuesta de partición esté en el cuerpo); (c) `exo budget --kb <wt>` sin offenders; (d) `exo lint
   --kb <wt>` limpio. Si algo falla, **no commitees**: deja el informe.
7. Commit en tu rama, con rutas explícitas (`git -C <wt> add <ruta>
   .distill/<slug>-moved.md`, solo lo que exista): subject `docs(kb): auto-distill
   <slug> — <evicción|propuesta de partición|evicción + partición>`; cuerpo con una línea
   `Nota: <ruta>` (relativa a la KB; es lo que resuelve el aviso de gate
   pendiente), bytes antes/después y una línea por bloque movido:
   `Movido: <tema> — cerrado|superado: «<cita literal>»`. Una propuesta
   de partición sin bloques movidos no tiene nada que añadir: `git -C <wt> commit
   --allow-empty`. Después quita el worktree
   (`git -C <kb> worktree remove --force <wt>`); la rama se queda.
8. Tope 10 min, best-effort (nada te corta): anota `date +%s` al arrancar y
   abstente si pasan 600 s. Si vence, o si no puedes verificar, abstención ruidosa: no
   commitees y devuelve un informe con qué hiciste, qué falló y el estado del
   worktree. Antes de terminar quita el worktree
   (`git -C <kb> worktree remove --force <wt>`) y, como no hay commit tuyo,
   borra la rama (`git -C <kb> branch -D distill/auto/<slug>`): si no, «rama
   existe = ya lanzado» bloquearía para siempre el siguiente gate. Tu mensaje
   final es ese informe, o el hash del commit y la rama.

## Después (no es del subagente)

El mantenedor, o `/distill` al revisar las ramas `distill/auto/*`, decide el
merge. En main, tras mergear (el conflicto esperado es la línea de estado del
canon, a mano): `exo write append --from $KB_ROOT/.distill/<slug>-moved.md
<permalink-bitacora>` (`--from` es relativo al cwd; idempotente por el rechazo
de cabecera duplicada, engine ≥ 0.7.0: con uno anterior la doble aplicación
duplica sin avisar) y borrar el sidecar. Tras el merge,
`git -C $KB_ROOT branch -d distill/auto/<slug>`: la rama viva bloquea el
siguiente auto-distill de la nota. Una propuesta de partición la ejecuta el
mantenedor o el distill manual (el auto-distill no crea notas); ejecutada o
descartada, `git -C $KB_ROOT branch -D distill/auto/<slug>`.
