# 2d — reglas de proyecto en el system prompt

**Fecha:** 2026-10-06 · **Estado:** diseño validado por secciones en sesión, auditado por Fable (una ronda) y ajustado. Pendiente de revisión de Paul.
**Antecedentes:** el contorno de `2026-09-30-exo-recorte-mecanismo-design.md` («Contorno de la 2d») y el gate `evals/techo-reglas-2/verdict.md` (PASA, 6/10 frente a 0/10, PR #49). Esta spec enmienda el contorno en los puntos marcados **[enmienda]**.

## Qué resuelve

- Una regla mecánica que solo vale para un repo hoy vive en la KB y no llega al agente que trabaja en ese repo.
- El v1 (`additionalContext`) llegó a todas las corridas, pero el agente la trató como sugerencia: 4/11. El v2 (system prompt con framing de autoridad) dio 6/10 frente a 0/10.
- **Objetivo:** que la sesión principal de Claude Code en el repo X reciba, en el system prompt y con el framing sellado de #49, las reglas de la sección `## Reglas duras` de la nota-puerta de X. Si no las recibe y eso es una anomalía, Paul lo ve.

## Hechos verificados que condicionan el diseño

Sonda del 2026-10-06 en Linux, `claude` 2.1.291. Evidencia en el scratchpad de la sesión; no se versiona.

- Un plugin con un solo `hooks/hooks.json` puede llevar `"modules": ["./register.ts"]` y hooks de comando a la vez. Los dos funcionan.
- **`prompt.compose` no llega a los subagentes.** Tampoco llega el `additionalContext` de SessionStart. El evento `e` de compose (`model, promptModel, tools, outputStyle, traits, surfaces`) no distingue padre de subagente.
- Dentro del mod funcionan `$.session.cwd()`, `$.session.id()`, `$.fs.read/write/exists` y `$.process.run(argv)` (sin shell, timeout por defecto de 30 s). `exo` está en el PATH.
- compose se ejecuta **en cada prompt** en una sesión interactiva (`hb-<sid>` se reescribe al empezar cada turno) y una vez en `-p`. `--resume` lo vuelve a ejecutar. No se ha probado con `/compact` ni con cambio de modelo.
- Un `register.ts` roto da exit 0 en silencio, y los hooks de comando del plugin siguen funcionando. `claude plugin validate` detecta el error de parseo.
- El SessionStart de comando corre **antes** que el `session.start` del mod (25–240 ms de diferencia).
- `git rev-parse --git-common-dir` devuelve una ruta **relativa** desde subdirectorios (`../../.git`). Con `--path-format=absolute` (git ≥2.31) devuelve la ruta absoluta.
- Hoy **0** notas de `projects/` tienen `## Reglas duras`.
- **Por qué mod y no append:** `--append-system-prompt-file` es un flag de arranque. Un hook no puede añadirlo, así que en sesiones reales "append" significaría un wrapper de shell que desktop e IDE se saltan en silencio. El mod viaja con el plugin y cubre todas las superficies.
- **Política de org `cc-plugin-sec-default` (medido en W11, cuenta Team, 2026-10-07).** Los mods de plugins de usuario no corren todos sus eventos:

  | evento del mod | bajo la política |
  |---|---|
  | `session.start`, `prompt.submit`, `$.session.append` | corren |
  | `prompt.compose`, `prompt.context`, `classic.*` | bypassed |

  Por eso la entrega tiene un canal degradado: `prompt.submit` entrega `FRAMING`+reglas como `context` cuando compose no está vivo (`hb.via=submit`). El e2e (`scripts/e2e-reglas-proyecto.sh`) cubre positivo, forzado (`EXO_RULES_FORZAR_SUBMIT=1`, compose inerte), `--resume` con codeword cambiado (re-entrega) y control; en esta máquina salen todos con `via=submit`. Cuenta personal (compose vivo): no verificado. Límite del e2e: bajo la política `forzado` no discrimina el seam `EXO_RULES_FORZAR_SUBMIT` (compose ya está bypassed y `positivo` da `via=submit`; el e2e lo imprime como `[INFO]`); solo en cuenta personal `via=compose` delataría que FORZAR no llegó al mod.
- W11 no se sondea. Que Anthropic publique mods que no carguen en su app de escritorio de Windows no es plausible. Los fallos posibles en W11 son de **nuestro** código, y se mitigan abajo; el testigo de Stop caza el resto.

## Arquitectura

```
KB: projects/<repo>.md ── ## Reglas duras (≤10 líneas «- », mecánicas)
          ▲
  exo rules --cwd <dir> --json       (engine Rust; resolver puro, sin ONNX)
          ▲                                   ▲
  mod (prompt.compose)                 exo-recall.sh (SessionStart, ya existe)
   · añade la sección con el framing    · resuelve, guarda ss-<sid>
   · escribe hb-<sid> con su resultado  · reflex-log + systemMessage si es anomalía
          
  Stop (script existente, sentinel propio): compara ss-<sid> con hb-<sid>
```

- **El resolver va en el engine.** Toda la lógica (mapeo, candidatas, sección, skips) está en Rust y se testea allí. El mod y el hook solo consumen el resultado; no se duplica la lógica en TS ni en bash.
- **No hay scripts bash nuevos.** El resolver se llama desde `exo-recall.sh`, que ya lanza el engine. El testigo va dentro de un script de Stop existente. En W11 cada hook nuevo cuesta ~1,7 s p95.
- **Subagentes: fuera de v1 [enmienda].** El contorno pedía entrega por SubagentStart, pero la sonda muestra que ningún canal pasivo llega a los subagentes, y el v1 midió `additionalContext` como sugerencia. Bajo `exo:orchestrate` las reglas llegan al padre, que no edita, y no al executor. Es un **hueco declarado**; el canario `k-subagente` decidirá si hace falta algo. No se añade meta-línea al framing ("copia las reglas en el brief"): sería una regla de proceso sin medir que además altera el framing sellado.

## Resolver: `exo rules --cwd <dir> --json`

**Clave del repo:** `basename(dirname(git rev-parse --path-format=absolute --git-common-dir))`, con la ruta canonicalizada. Cubre la raíz, los subdirectorios y los worktrees. Nunca se parsea el stderr de git, porque está localizado.

**Git que falla:** si `git rev-parse` sale con ≠ 0, el engine sube desde el cwd por los ancestros buscando una entrada `.git` (fichero o directorio; respeta `GIT_CEILING_DIRECTORIES`). Si la hay, el repo está roto y es un error del engine (exit ≠ 0, visible como `error_engine`); si no la hay, es `sin_git`.

**Candidatas [enmienda]:** solo notas de **primer nivel**, `projects/*.md`, cuya clave coincida sin distinguir mayúsculas con:
- el `slug` del frontmatter, o
- el stem del fichero hasta ` — ` (o el stem entero si no lo lleva).

Tiene que haber exactamente 1 candidata; si no, es skip. Las subcarpetas de familia no compiten. Sin esta regla, `pguerrero-music` saldría `ambigua` por `projects/pguerrero-music/pguerrero-music — flujo de sync a Navidrome.md`.

- **Sin `repo_alias` [enmienda]:** YAGNI. Los repos activos casan por slug o por stem, y el parser de frontmatter no lee listas.
- **Consecuencia aceptada:** cge (repo `code-graph-engine`, nota-puerta en `projects/cge/`) queda en `sin_nota` hasta el primer caso real.

**Sección:**
- Va desde la línea exacta `## Reglas duras` hasta el siguiente `## `.
- Cada línea que empieza por `- ` es una regla, literal y con `\r` recortado.
- Las demás líneas no vacías se devuelven en `ignored_lines`; nunca se descartan en silencio.
- La raíz de la KB sale de la config del engine (`cfg.kb`, la misma que usa `exo config`). Sin config, `error_engine`.
- La KB se lee directamente de los `.md`, no del índice. La frescura es **por prompt**: una regla escrita a mitad de sesión entra en el siguiente prompt.

**Contrato:** el envelope del engine (`envelope::emite("rules", …)`):

```json
{"schema_version":2,"command":"rules","data":{"status":"ok","repo":"exo","note":"projects/exo — ….md","rules":["…"],"ignored_lines":[]}}
{"schema_version":2,"command":"rules","data":{"status":"skip","repo":"exo","reason":"ambigua","candidates":["…","…"]}}
```

- El exit es 0 siempre que el engine funcione; el estado va en `data`.
- El exit es ≠0 solo cuando el engine falla. Los consumidores lo tratan como `error_engine`.
- Se sube `ENGINE_MIN`. Si el engine es anterior y no conoce el subcomando, el error es `engine_stale`, no `error_engine`.

| reason | cuándo |
|---|---|
| `sin_git` | cwd fuera de un repo |
| `sin_nota` | 0 candidatas |
| `ambigua` | ≥2 candidatas (con `candidates`) |
| `sin_seccion` | la nota no tiene `## Reglas duras` |
| `seccion_vacia` | la sección existe pero no tiene líneas `- ` |
| `excede_cap` | más de 10 reglas: **no se entrega ninguna**; nunca se trunca |

**No soportado (declarado):** submódulos (la clave sale del directorio padre en `.git/modules/`) y `GIT_DIR` en el entorno.

## Mod

- Ficheros `register.ts` y `"modules"` en `plugins/exo/hooks/hooks.json`, junto a los hooks de comando.
- **`prompt.compose`:**
  - Llama a `$.process.run([exo, "rules", "--cwd", cwd, "--json"], {timeoutMs: 3000})`.
  - Para localizar `exo` sigue el mismo orden que `exo-recall.sh:34`: el PATH y después `~/.local/bin/exo`, `.exe` incluido.
  - Si el estado es `ok`, añade **al final** una sección con el framing sellado de #49 y una línea `- ` por regla:
    ```
    ## Reglas duras del proyecto
    Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.

    - <regla>
    ```
  - En cualquier otro caso no añade nada.
  - En todos los casos escribe el latido `~/.claude/exo-rules/hb-<sid>` con `{status, reason?, n, error?}`. Va en `~/.claude/` y no en `/tmp` por la diferencia de HOME entre Node y Git Bash en W11 (`install.ps1:35-40`).
- **Supuesto declarado:** compose corre después de que exista `session.id()`. El latido se escribe desde compose, no desde `session.start`, porque lo que se mide es la **entrega**, no la carga.
- **Extrapolación declarada:** #49 midió el framing con **una** regla. Con varias no está medido.

## Gritos

Todo skip y toda anomalía van a reflex-log (`project-rules-skip reason=…`). La tabla dice cuáles se ven además.

| caso | dónde | ¿se ve? |
|---|---|---|
| `sin_git`, `sin_nota`, `sin_seccion` | `exo-recall.sh` | no, solo log **[enmienda]** |
| `ambigua`, `seccion_vacia`, `excede_cap`, `error_engine`, `engine_stale` | `exo-recall.sh` | sí, `systemMessage` de una línea |
| no hay latido (`sin_latido`: el mod no cargó o ningún evento corrió) | Stop | sí, una vez por sesión; copy en `document-remind.sh` |
| latido sembrado `via=none` (`sin_canal`: el mod cargó pero ningún canal entregó) | Stop | sí, una vez por sesión; copy en `document-remind.sh` |
| latido ok con `via=submit` y compose no vivo (`entrega_degradada`) | Stop | sí, aviso diario (copy y `DEGRADADO_PREFIJO` en `document-remind.sh`, hoy `ℹ`); **pendiente**: el prefijo `ℹ` presupone que el canal degradado es útil; el eval `ars` (T3, brazo construido, sin lanzar, sin `verdict-ars.md`) decide si pasa a `⚠` o a solo log |
| el latido no coincide con `ss-<sid>` y el engine (`exo rules --cwd <cwd del Stop>`) no coincide con el latido; o latido corrupto/vacío; o `hb.status=error` con `ss` ok | Stop | sí, una vez por sesión |

- **[enmienda]** El contorno pedía una línea visible para cualquier skip. Con 0 secciones en la KB, eso sería ruido en cada arranque, y acostumbra a ignorar justo las anomalías.
- `exo-recall.sh` solo resuelve con `source` distinto de `compact`. Guarda su resultado en `~/.claude/exo-rules/ss-<sid>` y poda las entradas de más de 7 días.
- `ss-<sid>` es una foto de SessionStart y compose corre en cada prompt, así que una divergencia entre ambos no prueba fallo: la KB pudo cambiar. Solo en ese caso (sin error) el testigo consulta `exo rules`: si el engine coincide con el latido, calla (`reason=kb_cambio`) y reescribe `ss-<sid>`; si no, grita «el engine dice … n=N, el latido dice … n=M» (`no_entrego`); si `exo rules` falla, calla (`sin_verdad`). `hb.status=error` con `ss` skip solo se loguea (`hb_error`). Las ramas que callan por adjudicación no crean sentinel.
- El testigo de Stop usa un sentinel propio por sesión, como `document-remind.sh`, para no gritar en cada turno.
- **Cap de 6.144 B:** no se toca. La sección va por compose y la línea por `systemMessage`; ninguna pasa por `additionalContext`.

## `/document`

Se añade una frase en `plugins/exo/skills/document/routing.md`: una regla **mecánica** de un solo repo va además a `## Reglas duras` de su nota-puerta (una línea `- `, ≤10 por nota).

## Testing

- **Engine (Rust):**
  - Una KB de fixture con un caso por cada `reason`.
  - cwd en la raíz, en un subdirectorio y en un worktree.
  - CRLF y `ignored_lines`.
  - El caso `pguerrero-music`: la nota de subcarpeta no compite.
  - `excede_cap` con 11 reglas.
  - El contrato del envelope en `test-contrato-engine.sh`.
- **`exo-recall.sh`:** con un stub de `exo rules` para cada estado. Se comprueba qué se ve y qué va solo a reflex-log, el `ss-<sid>`, el filtro `compact` y `engine_stale` con un engine viejo.
- **Testigo de Stop:**
  - Sin latido, grita una vez y el sentinel lo calla.
  - Con un latido que no coincide, grita.
  - Con un latido que coincide, se calla.
- **Mod:**
  - En CI no hay `claude`. `claude plugin validate` corre en `test-plugin.sh` local con un **skip que grita** si falta el binario, y como gate obligatorio en `release-publish.sh`.
  - Una corrida headless con una KB de fixture y un codeword, con la lectura de ficheros denegada, confirma que la sección llega al system prompt.
  - Control: un repo sin sección, sin la sección en el prompt.

## Fuera de alcance

- Entrega a subagentes y a los executors de orchestrate: lo decide el canario `k-subagente`.
- Reporte visible de los conflictos regla-prompt (verdict §6, g2-97).
- `repo_alias`, submódulos y `GIT_DIR`.
- Auto-compact a mitad de turno: el contexto entregado por `prompt.submit` puede perderse hasta el siguiente prompt (v2).
- Re-entrega vía `$.session.append`/`session.compact`: sin medir.
- La migración de los reflejos bash a mods (frente «Mods de Claude Code»). La 2d solo introduce el primer módulo.
