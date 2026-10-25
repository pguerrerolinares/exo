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
   bitácora**: main sigue haciendo append a la bitácora y dos appends en EOF
   conflictan en cada merge. En la rama solo cambian el canon, el sidecar y,
   si hay partición mecánica (paso 5), la hija nueva.
5. Mueve TODO lo que pase el test de salida, llegues o no al 60 %. Si con eso
   no llegas, no fuerces ni recortes: la salida es la **partición**. Subir el
   techo (`kbx_budget_max`) no es alternativa y no la propongas. Diséñala
   (títulos que pasen el test del título de `distill/consolidacion.md`, qué
   secciones van a cada hija, bytes resultantes) y decide si es **mecánica**:
   solo mueve secciones `##` o párrafos enteros, verbatim, a **una** hija, y
   con eso la madre queda ≤60 % y la hija ≤60 % de su techo.
   - **Mecánica → ejecútala en tu rama.** Escribe el cuerpo de la hija con tu
     tool `Write` en `${TMPDIR:-/tmp}/exo-distill-auto-<slug>-hija.md`, fuera de
     `<wt>`: frontmatter con los `tags` de la madre, los bloques verbatim en su
     orden original y `## Relations` con `- capitulo_de [[<título madre>]]`.
     Créala con `exo write new --kb <wt> --dir <dir de la madre> --title
     "<título>" --tier <tier de la madre> --from <ese fichero>`. Un exit 3
     (candidatas duplicadas) en el que solo salen la madre o sus hijas se
     repite con `--force`. Cualquier otra candidata significa que el tema ya
     tiene nota: no crees nada y deja la propuesta. En la madre, cada bloque
     movido se cambia por una línea `→ [[<título hija>]]` en su sitio, y
     `## Relations` gana `- capitulo [[<título hija>]]` (la convención de la KB).
   - **No mecánica** (hay que partir un párrafo, reescribir prosa o repartir
     entre varias hijas) → no la ejecutes: la propuesta va en el cuerpo del
     commit, como hasta ahora.

   En los dos casos, partir es decisión del mantenedor. Tú no haces merge;
   solo le dejas la rama hecha o la propuesta.
6. Verifica antes de commitear: (a) conservación: todo bloque que sale del
   canon aparece literal en el sidecar, o en la hija si fue por partición
   (compáralo con `grep -F` bloque a bloque, no a ojo); (b) **solo si llegaste
   al 60 %, por evicción o por partición mecánica**: `wc -c` de la nota ≤60 %
   del techo y, si hay hija, la hija ≤60 % del suyo (si no llegaste, la
   propuesta de partición va en el cuerpo y (b) no aplica); (c) `exo budget --kb <wt>`: sus
   offenders son un subconjunto de los de `exo budget --kb <kb>` (ninguno nuevo)
   y la nota tocada no pesa más que antes (`wc -c` ≤ el de `<kb>`); si llegaste
   al 60 % tampoco es offender. Seguir sobre el techo sin crecer es válido
   (evicción parcial, solo propuesta); (d) `exo lint
   --kb <wt> --baseline <kb>` sale 0 (exit 3 = hallazgos que tu rama introduce:
   no commitees). Es mecánico: solo cuentan los hallazgos que `<kb>` no tiene;
   una nota que ya excedía y sigue excediendo (con otro tamaño) es el mismo
   hallazgo; los previos **no bloquean**: cita en una línea del cuerpo del commit
   `Lint previo: <n> hallazgo(s) ajenos` (sin lista) y sigue. No juzgues tú qué
   hallazgo es «tuyo». Si `--baseline` no existe (exit 2, engine < 0.11.0), abstención ruidosa
   citando `exo --version`. Todo camino tiene salida: evicción completa o
   partición mecánica cumplen a–d; evicción parcial con propuesta, (a), (c) y
   (d) con (b) no aplicable; solo propuesta (sin bloques movidos, nota
   intacta), (c) y (d) trivialmente. Si una partición mecánica falla (a) o (b),
   deshazla (borra la hija, devuelve los bloques a la madre) y sigue por la
   propuesta. Si falla otra cosa, **no commitees**: deja el informe.
7. Commit en tu rama, con rutas explícitas (`git -C <wt> add <ruta>
   .distill/<slug>-moved.md <ruta hija>`, solo lo que exista): subject `docs(kb): auto-distill
   <slug> — <evicción|partición|propuesta de partición|evicción + partición|evicción + propuesta de partición>`;
   cuerpo con una línea `Nota: <ruta>` (relativa a la KB; es lo que resuelve el
   aviso de gate pendiente), bytes antes/después (de la hija también, si la
   hay), una línea `Hija: <ruta hija>` si partiste, y una línea por bloque
   movido: `Movido: <tema> — cerrado|superado: «<cita literal>»`, o `Movido:
   <tema> — partición → <título hija>`. Una propuesta de partición sin bloques
   movidos no tiene nada que añadir: `git -C <wt> commit
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
siguiente auto-distill de la nota. Una partición mecánica ya ejecutada se
revisa y se mergea igual (la hija entra con el merge; `/distill` la añade a
`[[core-index]]` si la madre está listada). Una propuesta de partición la
ejecuta el mantenedor o el distill manual; ejecutada o descartada,
`git -C $KB_ROOT branch -D distill/auto/<slug>`.
