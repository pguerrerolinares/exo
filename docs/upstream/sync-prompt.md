# upstream-sync — prompt del bot

Lo lee y sigue la routine semanal. El prompt guardado en la routine es solo: "Lee y sigue docs/upstream/sync-prompt.md del repo exo." Diseño y motivos: `docs/superpowers/specs/2026-09-29-upstream-sync-design.md`.

Tienes dos repos adjuntos: `pguerrerolinares/exo` (donde trabajas) y `obra/superpowers` (solo lectura). Paul mergea; tú detectas, triageas, portas y abres PR. Nunca mergees.

Estado: `docs/upstream/ledger.md` (léelo entero antes de empezar: `upstream_tag`, mapeo, excluidos, sin equivalente, divergencias, filas).

**Regla de salida:** toda salida, sea cual sea el paso donde termines, acaba en el paso 7 (latido). Si no llegas a comentar en el issue, la pasada no cuenta: el watchdog lo pone rojo a los 8 días.

## Paso 0 · Reconcilia

1. `bash scripts/upstream-reconcile.sh docs/upstream/ledger.md`. Promueve `propuesto` a `portado` (con hash) si `main` contiene un commit `port(upstream#N): …`. Exit 2 = ledger roto: latido con alerta y fin. Cada línea de stdout es una fila que sigue `propuesto` sin evidencia (`propuesto-sin-evidencia #N skill`) o una fila corrupta (`propuesto-malformado …`): anótalas en el informe.
2. Las filas `propuesto` cuyo PR `upstream-sync` se cerró sin merge pasan a `rechazado`, con la URL de ese PR en `motivo` y `—` en `hash`. Lista los PRs con `gh pr list -R pguerrerolinares/exo --search "upstream-sync in:title" --state closed --json number,title,url,mergedAt`.
3. Detecta un PR abierto con `gh pr list -R pguerrerolinares/exo --search "upstream-sync in:title" --state open`. Si hay uno: latido ("PR abierto, sin trabajo nuevo") y fin. Lo que `upstream-reconcile.sh` haya reescrito en el ledger NO se commitea si la pasada sale sin PR (aquí ni en ninguna otra salida temprana): se descarta y se rehace en la pasada siguiente.

## Paso 1 · Licencia

Compara el `LICENSE` de la raíz del repo adjunto `obra/superpowers`, en el tag a revisar (`<hasta>`, o `upstream_tag` si no hay tags nuevos; `git show <tag>:LICENSE`), con `plugins/exo/LICENSES/superpowers.LICENSE`. Si difieren: latido con alerta ("licencia cambiada") y fin. No portes nada.

## Paso 2 · Detecta

1. En el clone de `obra/superpowers`: `git tag --sort=version:refname`, ignorando prereleases (`-rc`, `-beta`, `-alpha`…). Tags posteriores a `upstream_tag`, en ese orden. Si no hay: latido y fin. Si hay, revisa hasta el más reciente (`<hasta>`); `<desde>` es `upstream_tag`.
2. **Contenido:** diff neto `<desde>..<hasta>` solo de los ficheros del mapeo del ledger. Es lo que hay que portar; un revert dentro del ciclo ya viene anulado.
3. **Atribución:** PRs mergeados entre las fechas de ambos tags que tocan ficheros mapeados:
   `gh pr list -R obra/superpowers --state merged --search "merged:<desde>..<hasta>" --limit 200 --json number,title,baseRefName,files`
   (`--limit 200` es obligatorio: sin él `gh` corta a 30 resultados en silencio) con `<desde>` y `<hasta>` como fechas ISO de los tags. Cualquier rama base cuenta. Descarta los PRs de release (dev→main): su contenido se atribuye a los PRs miembros. Cada hunk del diff neto se atribuye a un PR; el que no puedas atribuir va como `duda` con la pregunta en `motivo`.

## Paso 3 · Triagea

Una fila por (PR, skill exo). Valores de `triage`: `aplica`, `parcial`, `ya cubierto`, `no aplica`, `duda`. Lee el fichero exo mapeado antes de decidir; cada veredicto cita líneas de exo (`fichero:línea`) y, si aplica, la fila del ledger. La evidencia es el contenido actual de ese fichero exo: lo que digan README, specs, planes o changelogs sobre portes previos no es evidencia (un porte declarado puede ser parcial). `ya cubierto` exige citar las líneas exo que cubren el cambio entero.

- Choca con una divergencia del ledger → `no aplica`, citando su id (D1…). D3 y D5 son forma de porte, no estado actual: si el cambio upstream toca ese terreno, porta con esa forma.
- Fichero en `Excluidos` → no genera fila. Fichero en `Sin equivalente` → `no aplica`, citando esa sección, salvo la excepción de su línea: en `executing-plans/` y `receiving-code-review/`, una regla de review o de ledger independiente del modo inline se evalúa contra `orchestrate/` (reviewer-prompt.md, SKILL.md u olas.md); no la descartes por el fichero.
- Fichero upstream que no está en ninguna sección del ledger → `duda: mapeo`. El PR propone la fila de mapeo en el ledger y no porta nada de ese fichero.
- `ya cubierto` solo si puedes señalar la línea de exo que ya hace lo mismo. Una cobertura parcial es `parcial`, no `ya cubierto`. Ante la duda entre `ya cubierto` y `aplica`/`parcial`, no elijas `ya cubierto`.
- `aplica` y `parcial` llevan a la misma acción (portar); distingue igualmente para el informe.

Toda fila lleva `triage` no vacío. `estado` de las filas: `—` para `no aplica` y `ya cubierto`; `propuesto` para lo que portes; `pendiente` para `duda` y para lo que caiga fuera del tope. Una fila `pendiente` por tope lleva su triage real (`aplica`/`parcial`), no `duda`. `motivo`: obligatorio en `pendiente` (`tope`, o la pregunta concreta de la duda) y en `no aplica` (id de la divergencia).

## Paso 4 · Porta

Porta las filas `aplica` y `parcial`. **Máximo 5 portes por PR de exo**: el resto queda `pendiente`, motivo `tope`, y se porta en la pasada siguiente.

1. Rama `upstream-sync/<tag>` (`<tag>` = `<hasta>`) desde `main`.
2. **Un commit por PR upstream**, con subject exacto `port(upstream#N): <resumen>`. Un PR upstream que toca varias skills exo son varias filas pero cuenta como un solo commit; el tope cuenta portes (filas), no commits.
3. Porta destilando a la voz de exo (español, denso, sin copiar prosa upstream), no pegando. Respeta las divergencias; en `orchestrate/scripts/review-package` no toques la sección MUTACIÓN.
4. Cada commit actualiza la versión citada en la cabecera de atribución de los ficheros que toca: `# Derived from superpowers X.Y.Z` en scripts, `(superpowers X.Y.Z, MIT ©` en markdown, con la versión de `<hasta>`. Un fichero tocado sin cabecera de atribución, pero con contenido derivado nuevo, la gana.
5. `duda` → `pendiente`, con la pregunta concreta en `motivo`. No portes nada dudoso.

Ejemplo de commit (número inventado): `port(upstream#9xxx): renombra la sección Foo de bar-script`.

## Paso 5 · Gates

1. `bash scripts/check-skill-refs.sh` y `bash scripts/test-plugin.sh` en verde. Si fallan, arréglalo dentro del porte; si no puedes, deja ese porte como `pendiente` con la causa en `motivo` y no lo incluyas.
2. **Tabla obligatoria** en el cuerpo del PR, una fila por porte: `movimiento upstream → fichero:línea exo`. Sin tabla no hay PR.

   | movimiento upstream | fichero:línea exo |
   |---|---|
   | #9xxx: qué cambió en upstream, en una frase | plugins/exo/skills/foo/bar-script:120 |

3. Bytes y tokens (`claude --plugin-dir ./plugins/exo plugin details exo`) van al informe como dato, no como gate. Si `claude` no está en el PATH del sandbox, anótalo en el informe y sigue.

## Paso 6 · Abre PR

Desde la rama `upstream-sync/<tag>`, título `upstream-sync <tag>`, hacia `main`. Contiene:

- los commits `port(upstream#N)`;
- `docs/upstream/ledger.md` actualizado: filas nuevas (`propuesto` con el hash del commit, `pendiente`, `no aplica`, `ya cubierto`, `rechazado`) y `upstream_tag: <tag>` avanzado;
- en el cuerpo: la tabla del paso 5, el informe (PRs vistos, triage por fila con sus citas, pendientes con motivo, bytes y tokens, filas de `propuesto-sin-evidencia`) y las preguntas de `duda`.

Si no hay nada que portar (todo `no aplica`, `ya cubierto` o `duda: mapeo`), abre igualmente el PR solo con el ledger: avanzar `upstream_tag` es lo que evita revisar dos veces la misma ventana. Solo se avanza `upstream_tag` en el PR; si el PR se rechaza, vuelve a revisarse.

## Paso 7 · Latido

Comentario en el issue `upstream-sync: estado` (fijado). Si no existe, créalo y fíjalo. Contenido, una línea por campo:

- fecha ISO de hoy;
- tag revisado (`<hasta>`, o el vigente si no había nuevo);
- PRs upstream vistos / propuestos / pendientes (números);
- URL del PR abierto, o el motivo de salida ("PR abierto", "sin tags nuevos", "licencia cambiada", "ledger roto").

Sin latido no hay pasada. Un fallo a mitad también termina aquí: comenta qué paso falló y por qué.

## Límites

- Reestructuración upstream (ficheros nuevos, movidos, borrados): `duda: mapeo`.
- Release grande: el tope de 5 portes reparte el resto entre pasadas.
- Nada de issues en upstream: fase 2.
- Nunca mergees, nunca fuerces push a `main`, nunca edites ficheros fuera de un porte, `docs/upstream/ledger.md` o el informe.
