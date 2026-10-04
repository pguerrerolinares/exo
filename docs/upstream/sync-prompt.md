# upstream-sync — prompt del bot

Lo lee y sigue la routine semanal. El prompt guardado en la routine es solo: "Lee y sigue docs/upstream/sync-prompt.md del repo exo." Diseño y motivos: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`.

Tienes dos repos adjuntos: `pguerrerolinares/exo` (donde trabajas) y `obra/superpowers` (solo lectura). Paul mergea; tú detectas, triageas, portas y abres PR. Nunca mergees.

Estado: `docs/upstream/ledger.md` (léelo entero antes de empezar: `upstream_tag`, mapeo, excluidos, sin equivalente, divergencias, filas).

**Regla de salida:** toda salida, sea cual sea el paso donde termines, acaba en el paso 7 (latido). Si no llegas a comentar en el issue, la pasada no cuenta: el watchdog lo pone rojo a los 15 días.

## Paso 0 · Reconcilia

**Preparación (antes del 0.1).** Si `command -v gh` falla: `sudo apt-get install -y -qq gh`, **sin `apt-get update` antes** (falla con 403 en PPAs y el `&&` cortaría la instalación). Si sigue sin `gh`, el latido por gh es imposible: no hay otro canal, así que dilo en el resumen final de la sesión y termina.

En este entorno GraphQL da 403 (`GitHub GraphQL is not available from Claude Code sessions`). **Está prohibido usar los subcomandos `pr`, `issue` y `search` de gh** (`gh` seguido de `pr …`, etc.): solo `gh api` sobre REST. `gh auth status` puede decir "token invalid"; es cosmético, la REST funciona con `GH_TOKEN`. El push va por `git push origin <rama>`.

**Paginación (proxy).** los endpoints de búsqueda están bloqueados (`sessions are bound to their configured repositories`) y la auto-paginación de `gh api` falla porque sigue enlaces `repositories/{id}` (403). Nunca uses ninguna de las dos: pagina a mano con `&per_page=100&page=N`, empezando en `page=1` y subiendo hasta que una página venga vacía. Todo listado de abajo se entiende así; la API corta a 30 resultados en silencio si no pones `per_page`.

1. `bash scripts/upstream-reconcile.sh docs/upstream/ledger.md`. Promueve `propuesto` a `portado` (con hash) si `main` contiene un commit `port(upstream#N): …`. Exit 2 = ledger roto: latido con alerta y fin. Cada línea de stdout es una fila que sigue `propuesto` sin evidencia (`propuesto-sin-evidencia #N skill`) o una fila corrupta (`propuesto-malformado …`): anótalas en el informe.
2. **Rechazos.** Lista los PRs cerrados sin merge: `gh api 'repos/pguerrerolinares/exo/pulls?state=closed&per_page=100&page=N' --jq '.[] | select(.merged_at == null and (.head.ref | startswith("upstream-sync/"))) | {number, ref: .head.ref, title, url: .html_url}'`. Para cada uno cuyo rechazo aún no conste (ninguna fila `rechazado` con su URL en `motivo`): saca las filas de su cambio en el ledger; **solo las que ese PR dejaba en `propuesto` pasan a `rechazado`** (las `pendiente`, `no aplica` y `ya cubierto` se copian tal cual, y las `pendiente` se retoman en el paso 2.1) (`gh api repos/pguerrerolinares/exo/pulls/<N> -H 'Accept: application/vnd.github.diff'`, solo el trozo de `docs/upstream/ledger.md`) y regístralas en el PR de esta pasada como `rechazado`, con la URL del PR en `motivo` y `—` en `hash`. Si no puedes recuperar esas filas (falla la petición del diff u otro error), no inventes: el latido de esta pasada lleva `estado: alerta` con el PR afectado. Su tag es lo que sigue a `upstream-sync/` en `.head.ref` (sin un posible sufijo `-rN`): la ventana ya está revisada, así que `<desde>` del paso 2 y el `upstream_tag` que avances no pueden quedar por detrás del tag más reciente rechazado; así no se repropone lo mismo.
3. Detecta un PR abierto: `gh api 'repos/pguerrerolinares/exo/pulls?state=open&per_page=100&page=N' --jq '.[] | select(.head.ref | startswith("upstream-sync/")) | {number, ref: .head.ref, url: .html_url}'` (no busques por título). Si hay uno: latido ("PR abierto, sin trabajo nuevo") y fin. Lo que `upstream-reconcile.sh` haya reescrito en el ledger NO se commitea si la pasada sale sin PR (aquí ni en ninguna otra salida temprana): se descarta y se rehace en la pasada siguiente.

## Paso 1 · Clone y licencia

Si `upstream_tag` no aparece en `git tag` del clone de `obra/superpowers`: `git fetch --tags`. Si sigue sin aparecer: latido `estado: alerta` ("clone sin tags") y fin; un clone sin tags no es "sin tags nuevos".

Compara el `LICENSE` de la raíz del repo adjunto `obra/superpowers`, en el tag a revisar (`<hasta>`, o `upstream_tag` si no hay tags nuevos; `git show <tag>:LICENSE`), con `plugins/exo/LICENSES/superpowers.LICENSE`. Si difieren: latido con alerta ("licencia cambiada") y fin. No portes nada.

## Paso 2 · Detecta

1. **Retoma pendientes, antes de mirar tags.** Filas `pendiente` con motivo `tope`, `gate: …` o `respondida` (ver paso 3 para `respondida`): pórtalas primero, con el mismo tope de 5 por PR de exo, contando también lo nuevo. Su diff: `gh api repos/obra/superpowers/pulls/<N> -H 'Accept: application/vnd.github.diff'`, limitado a los ficheros mapeados. Contrástalo con el estado actual del fichero exo, como en el paso 3. La fila se actualiza en su sitio (`pendiente` → `propuesto`, con hash), sin duplicarla. Las `pendiente` por `duda` no se retoman solas: esperan la respuesta de Paul.
2. En el clone de `obra/superpowers`: `git tag --sort=version:refname`, ignorando prereleases (`-rc`, `-beta`, `-alpha`…). Tags posteriores a `upstream_tag`, en ese orden. Si no hay tags nuevos y tampoco hay pendientes que retomar ni rechazos por registrar: latido y fin. Si no hay tags nuevos pero sí una de esas dos cosas, abre PR con eso (`<hasta>` es `upstream_tag`). Si hay tags, revisa hasta el más reciente (`<hasta>`); `<desde>` es `upstream_tag` (ver paso 0.2 si hay rechazos).
3. **Contenido:** diff neto `<desde>..<hasta>` solo de los ficheros del mapeo del ledger. Es lo que hay que portar; un revert dentro del ciclo ya viene anulado.
4. **Atribución:** PRs mergeados entre las fechas de ambos tags que tocan ficheros mapeados:
   - números: `gh api 'repos/obra/superpowers/pulls?state=closed&sort=updated&direction=desc&per_page=100&page=N' --jq '.[] | select(.merged_at != null and .merged_at >= "<desde>" and .merged_at <= "<hasta>") | .number'`, con `<desde>` y `<hasta>` como fechas ISO de los tags. Ordenado por `updated` descendente: sigue paginando hasta una página vacía, o hasta que una página entera quede por debajo de `<desde>` en `updated_at`;
   - ficheros de cada PR: `gh api 'repos/obra/superpowers/pulls/<N>/files?per_page=100&page=N' --jq '.[].filename'`;
   - rama base: `gh api repos/obra/superpowers/pulls/<N> --jq .base.ref`;
   - de un commit del diff neto a su PR: `gh api repos/obra/superpowers/commits/<sha>/pulls --jq '.[].number'`.

   Cualquier rama base cuenta. Descarta los PRs de release (dev→main): su contenido se atribuye a los PRs miembros. Cada hunk del diff neto se atribuye a un PR; el que no puedas atribuir va como `duda` con la pregunta en `motivo`.

## Paso 3 · Triagea

**Filas.** La columna `skill` es SIEMPRE una skill de exo (brainstorm, plan, orchestrate, tdd, debug, verify), nunca la de upstream (`writing-plans`, `writing-skills`…). Un PR upstream produce exactamente una fila por cada skill de exo cuyo fichero mapeado toca, incluidas las `no aplica`: no omitas la fila de una skill de exo tocada solo porque el veredicto sea `no aplica`. Los ficheros de skills upstream sin mapeo (`writing-skills`, `using-superpowers`, `diagnosing-superpowers`, `finishing-a-development-branch`…) no generan filas de skill: se listan una sola vez en el informe del PR, bajo "sin mapeo", no en la tabla de filas.

Valores de `triage`: `aplica`, `parcial`, `ya cubierto`, `no aplica`, `duda`. Lee el fichero exo mapeado antes de decidir; cada veredicto cita líneas de exo (`fichero:línea`) y, si aplica, la fila del ledger. La evidencia es el contenido actual de ese fichero exo: lo que digan README, specs, planes o changelogs sobre portes previos no es evidencia (un porte declarado puede ser parcial), y la versión citada en la cabecera de atribución de un fichero exo tampoco prueba cobertura.

**Doctrina de exo al decidir** = las Divergencias del ledger y sus "Decisiones de doctrina (portar)". Un cambio que implementa una decisión de doctrina es `aplica`/`parcial`.

- Choca con una divergencia del ledger → `no aplica`, citando su id (D1…). D3 y D5 son forma de porte, no estado actual: si el cambio upstream toca ese terreno, porta con esa forma.
- Fichero en `Excluidos` → no genera fila. Fichero en `Sin equivalente` → `no aplica`, citando esa sección, salvo la excepción de su línea: en `executing-plans/` y `receiving-code-review/`, una regla de review o de ledger independiente del modo inline se evalúa contra `orchestrate/` (reviewer-prompt.md, SKILL.md u olas.md); no la descartes por el fichero.
- Fichero upstream que no aparece en NINGUNA sección del ledger (mapeo, excluidos, sin equivalente) → `triage=duda`, `motivo=mapeo: <fichero upstream>`, en la fila de la skill de exo más cercana si la hay; si no hay ninguna, va al "sin mapeo" del informe con la propuesta. El PR propone la fila de mapeo en el ledger y no porta nada de ese fichero. Un fichero de skill upstream que el ledger ya conoce sin mapeo no es `duda`.
- **`no aplica` frente a `ya cubierto`.** `no aplica` = el cambio no debe entrar en exo (divergencia del ledger, harness ajeno, fichero excluido o sin equivalente, recorte de algo que exo no tiene). `ya cubierto` = el cambio sí es deseable y exo ya lo tiene entero. Un recorte upstream de algo que exo ya tiene recortado es `ya cubierto`, no `no aplica`. Si el cambio solo elimina algo que exo no tiene → `no aplica`; si además añade algo, evalúa lo añadido. Si elimina algo que exo sí tiene, evalúa si exo debe eliminarlo también (`aplica`/`parcial`) o si es una divergencia.
- **`ya cubierto` exige tabla.** Antes de marcarlo, escribe la tabla `movimiento upstream → fichero:línea exo` con una línea por CADA movimiento del hunk (un hunk puede llevar varios: renombre, regla nueva, ejemplo, borrado). Si falta la línea de uno solo, es `parcial`. Ante la duda entre `ya cubierto` y `parcial`, `parcial`.
- **`duda` solo por conflicto de doctrina.** `duda` únicamente cuando el cambio contradice o reescribe doctrina de exo que el ledger no resuelve (ni divergencia ni decisión tomada), o por mapeo (regla anterior). Si es compatible con la doctrina de exo y mejora la skill, `aplica`/`parcial`. Rule and continue: no preguntes lo que puedes decidir tú. Cada `duda` lleva en `motivo` la pregunta concreta y qué decidirías tú.
- `aplica` y `parcial` llevan a la misma acción (portar); distingue igualmente para el informe.

Toda fila lleva `triage` no vacío. `estado` de las filas: `—` para `no aplica` y `ya cubierto`; `propuesto` para lo que portes; `pendiente` para `duda` y para lo que caiga fuera del tope o no pase los gates. Una fila `pendiente` por tope o gate lleva su triage real (`aplica`/`parcial`), no `duda`. `motivo`: obligatorio en `pendiente` (`tope`, `gate: <causa>`, o la pregunta concreta de la duda y qué decidirías) y en `no aplica` (id de la divergencia). Nunca uses `|` dentro de `motivo` (rompe la tabla): usa `/`.

**Cómo responde Paul a una `duda`.** El latido la lista con su pregunta. Paul edita la fila en `main`: pone en `triage` su decisión (`aplica`, `parcial`, `ya cubierto` o `no aplica`) y en `estado` `pendiente` con `motivo` `respondida`. En la pasada siguiente, la fila `respondida` con `aplica`/`parcial` se trata como `tope` (paso 2.1); con `ya cubierto`/`no aplica` solo pasa a `estado` `—`.

## Paso 4 · Porta

Porta las filas `aplica` y `parcial`. **Máximo 5 portes por PR de exo**: el resto queda `pendiente`, motivo `tope`, y se porta en la pasada siguiente.

1. Antes de crear la rama, pon el clone de exo al día: `git -C <exo> fetch origin main && git checkout -B <rama> origin/main` (un clone por detrás de `origin/main` obliga a rebasar después). Rama `upstream-sync/<tag>` (`<tag>` = `<hasta>`) desde `origin/main`. Si esa rama ya existe en el remoto (pasada que solo retoma pendientes, con `<hasta>` = `upstream_tag`), usa `upstream-sync/<tag>-r2`, `-r3`… El título sigue siendo `upstream-sync <tag>`.
2. **Un commit por PR upstream**, con subject exacto `port(upstream#N): <resumen>`. Un PR upstream que toca varias skills exo son varias filas pero cuenta como un solo commit; el tope cuenta portes (filas), no commits.
3. Porta destilando a la voz de exo (español, denso, sin copiar prosa upstream), no pegando. Respeta las divergencias; en `orchestrate/scripts/review-package` no toques la sección MUTACIÓN.
4. Cada commit actualiza la versión citada en la cabecera de atribución de los ficheros que toca: `# Derived from superpowers X.Y.Z` en scripts, `(superpowers X.Y.Z, MIT ©` en markdown, con la versión de `<hasta>`. Un fichero tocado sin cabecera de atribución, pero con contenido derivado nuevo, la gana.
5. `duda` → `pendiente`, con la pregunta concreta en `motivo`. No portes nada dudoso.
6. **Versión.** Si el PR toca algo bajo `plugins/exo/`, el último commit (`chore(plugin): exo X.Y.Z`) sube el PATCH de exo en `plugins/exo/.claude-plugin/plugin.json` y en `.claude-plugin/marketplace.json`, con el mismo número en ambos (`scripts/test-versiones.sh` lo exige; `scripts/plugin-bump-gate.sh` exige el bump si `plugins/exo/` cambia). Parte de la versión de `main`. Un PR solo-ledger no sube versión. No toques `engine/Cargo.toml` ni `plugins/exo/ENGINE_MIN`.

Ejemplo de commit (número inventado): `port(upstream#9xxx): renombra la sección Foo de bar-script`.

## Paso 5 · Gates

1. `bash scripts/check-skill-refs.sh` y `bash scripts/test-plugin.sh` en verde. Si el PR toca `plugins/exo/`, también `bash scripts/test-versiones.sh` y `BASE=origin/main bash scripts/plugin-bump-gate.sh`. Si fallan, arréglalo dentro del porte; si no puedes, deja ese porte como `pendiente` con motivo `gate: <causa>` y no lo incluyas.
2. **Tabla obligatoria** en el cuerpo del PR, una fila por porte: `movimiento upstream → fichero:línea exo`. Sin tabla no hay PR.

   | movimiento upstream | fichero:línea exo |
   |---|---|
   | #9xxx: qué cambió en upstream, en una frase | plugins/exo/skills/foo/bar-script:120 |

3. Bytes y tokens (`claude --plugin-dir ./plugins/exo plugin details exo`) van al informe como dato, no como gate. Si `claude` no está en el PATH del sandbox, anótalo en el informe y sigue.

## Paso 6 · Abre PR

Push: `git push origin upstream-sync/<tag>`. Crea el PR con `gh api repos/pguerrerolinares/exo/pulls -f title='upstream-sync <tag>' -f head='upstream-sync/<tag>' -f base=main -F body=@<fichero>` (cuerpo escrito antes en un fichero temporal; imprime `.html_url` con `--jq .html_url`), título `upstream-sync <tag>`, hacia `main`. Contiene:

- los commits `port(upstream#N)`;
- `docs/upstream/ledger.md` actualizado: filas nuevas (`propuesto` con el hash del commit, `pendiente`, `no aplica`, `ya cubierto`, `rechazado`) y `upstream_tag: <tag>` avanzado;
- en el cuerpo: la tabla del paso 5, el informe (PRs vistos, triage por fila con sus citas, pendientes con motivo, bytes y tokens, filas de `propuesto-sin-evidencia`) y las preguntas de `duda`.

Si no hay nada que portar (todo `no aplica`, `ya cubierto` o `duda` de mapeo), abre igualmente el PR solo con el ledger: avanzar `upstream_tag` es lo que evita revisar dos veces la misma ventana. `upstream_tag` solo avanza en el PR. Si Paul lo cierra sin merge, el paso 0.2 registra sus filas como `rechazado` y la ventana no se reprocesa.

## Paso 7 · Latido

Comentario en el issue `upstream-sync: estado` (#30): `gh api repos/pguerrerolinares/exo/issues/30/comments -F body=@<fichero>`. Si el número no responde, búscalo por título exacto: `gh api 'repos/pguerrerolinares/exo/issues?state=all&per_page=100&page=N' --jq '.[] | select(.title == "upstream-sync: estado") | .number'`; si no existe, créalo con `gh api repos/pguerrerolinares/exo/issues -f title='upstream-sync: estado' -F body=@<fichero>`. Fijarlo no es posible por REST: dilo en el latido. Contenido, una línea por campo:

- fecha ISO de hoy;
- tag revisado (`<hasta>`, o el vigente si no había nuevo);
- PRs upstream vistos / propuestos / pendientes (números);
- pendientes por motivo, siempre: `tope: n · gate: n · duda: n · respondida: n`. Lista cada `duda` con su pregunta y recuerda cómo responder (paso 3);
- ratio de rechazo sobre una ventana: las **últimas 15 filas** en `propuesto`, `portado` o `rechazado` (el ledger es append-only, el orden de filas es cronológico): `rechazado` / esas filas; si supera 1/3, `estado: alerta` con "re-corre el eval";
- URL del PR abierto, o el motivo de salida ("PR abierto", "sin tags nuevos", "licencia cambiada", "ledger roto", "clone sin tags");
- última línea, exacta y sola: `estado: ok` o `estado: alerta`. El watchdog solo cuenta los comentarios con esa línea. Es `alerta` si: licencia cambiada, ledger roto (exit 2), clone sin tags, fallo a mitad de la pasada, ratio de rechazo > 1/3, o cualquier salida anómala que no encaje en las demás. Si no, `ok`.

Sin latido no hay pasada. Un fallo a mitad también termina aquí: comenta qué paso falló y por qué.

## Límites

- Reestructuración upstream (ficheros nuevos, movidos, borrados que no aparecen en ninguna sección del ledger): `triage=duda`, `motivo=mapeo: <fichero>`.
- Release grande: el tope de 5 portes reparte el resto entre pasadas.
- Nada de issues en upstream: fase 2.
- Nunca mergees, nunca fuerces push a `main`, nunca edites ficheros fuera de un porte, `docs/upstream/ledger.md`, el informe o el bump de versión del paso 4.6.
