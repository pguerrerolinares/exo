# Verdict consultor A — mecánica del cutover (C8: M3 + M1b)

Consultor independiente, dispatch fresco, 2026-08-18. Adjudica A1..A4 con
verificación primaria propia. Entorno: Claude Code 2.1.234 (`claude --version`),
HEAD de exo `89dbe95`, working tree limpio salvo `?? .worktrees/` y sin commits
sin push (`git -C exo log origin/main..main` vacío).

## Contexto verificado (y dónde el brief se desvía)

- `plugins/process/` existe con exactamente `README.md`, `LICENSES/superpowers.LICENSE`
  (MIT © 2025 Jesse Vincent, verificado por lectura) y `skills/{brainstorm,debug,
  documenta,orchestrate,plan,tdd,verify}` — 7 skills, cada una con `SKILL.md` con
  frontmatter `name` + `description`. **No hay `.claude-plugin/plugin.json`**
  (`find` sobre el árbol) ni entrada en ningún `marketplace.json` (leído el de
  agent-develop completo). Confirmado: hoy no es instalable.
- Paridad: `evals/prep-m3/verdict/gate-prep-m3-impl.md:287-289` — «Paridad
  135/135, 0 movimientos nuevos sin cita, 0 descartes resucitados … VEREDICTO
  FINAL: MERGED» (tras el fix `fc51e09` del 134/135 inicial). Cierto.
- Visibilidad (gh, primario): `exo` PRIVATE, `agent-develop` PRIVATE,
  `workflow-lint` **PUBLIC** (matiz relevante para A1).
- `GATE-CALENDARIO-D`: cerrado, no derogado (plan §0, líneas 34-38: D corrida
  2026-08-02, verdict NO-PASS firmado; «M1b/M3/M6 quedan desbloqueados»). El
  cutover está desbloqueado.
- **Desviación del brief #1**: el guard vivo del recall ya NO es el de
  `compose_base` con FALLBACK por oversize. El hook de SessionStart de reflex
  0.13.0 es `exo-recall.sh` (plugin.json de reflex 0.13.0 lo declara; cache
  `~/.claude/plugins/cache/agent-develop/reflex/0.13.0/`), que llama
  `exo recall … --cap-bytes 6144` (`exo-recall.sh:36` `EXO_CAP=…6144`) y ante
  oversize **trunca con aviso logueado** (`exo-recall.sh:49-52`, evento
  `truncated`), no cae al FALLBACK. El FALLBACK-por-oversize era del script
  viejo `basic-memory-recall.sh:77` (`if [ "${#CORE}" -gt 6144 ]`), aún
  presente pero no cableado como recall. El límite operativo sigue siendo
  6.144 **bytes** (cap del engine) — el número del brief vale, el mecanismo no.
  Bonus: el propio texto de core-index («pasarse hace caer el arranque al
  FALLBACK») está stale respecto a esto.
- **Desviación del brief #2**: la coletilla del «contador de no-disparos» que
  la spec prep-m3 §6.2-3 pre-registra está **retirada** por el plan
  (§Campaña 8: «Sin contador de no-disparos (retirado por el régimen §0)»).
  Gana el plan (firmado por Paul 2026-08-17, posterior a la spec). Afecta al
  texto de A3.

---

## A1 — Cómo se instala `process`. DECISIÓN: exo es su propio marketplace, registrado por path local

**Se hace (b): `exo/.claude-plugin/marketplace.json` en la raíz de exo con
`process` como source de path relativo, registrado con
`claude plugin marketplace add /home/paul/Documentos/proyectos/exo`. No (a),
no (c), no skills-dir.**

Evidencia de que (b) funciona pieza a pieza, sin inventar schema:

1. **Source de path relativo dentro del marketplace**: es EXACTAMENTE el patrón
   que ya sirve `paul-profile` y `reflex` hoy —
   `agent-develop/.claude-plugin/marketplace.json` entradas
   `"source": "./plugins/paul-profile"` y `"source": "./plugins/reflex"`,
   instaladas y cacheadas en `~/.claude/plugins/cache/agent-develop/{paul-profile/0.5.0,reflex/0.13.0}`
   (`installed_plugins.json`). Doc oficial
   (code.claude.com/docs/en/plugin-marketplaces, fetch 2026-08-18): «Relative
   path … Local directory within the marketplace repo. Must start with `./`.
   Resolved relative to the marketplace root» y «marketplace root, which is
   the directory containing `.claude-plugin/`» — o sea
   `exo/.claude-plugin/marketplace.json` + `"./plugins/process"` apunta a
   `exo/plugins/process`. Encaja con el layout ya mergeado sin mover nada.
2. **Registro por path local**: `claude plugin marketplace add --help`
   (binario real, 2.1.234): «Add a marketplace from a URL, **path**, or GitHub
   repo». El walkthrough de la doc usa literalmente
   `/plugin marketplace add ./my-marketplace`.
3. **Sin push**: todo lo anterior es local. Es el ÚNICO camino ejecutable hoy:
   cualquier variante github necesita que `plugin.json`/`marketplace.json`
   estén EN GitHub, y hoy no existen ni en local (y el push es de Paul).

Por qué NO las otras:

- **(a) `process` en agent-develop apuntando al repo exo**: requiere (i) editar
  y **pushear** agent-develop (push = Paul, config §Ejecución de gates
  líneas 316-318) y (ii) un plugin-source `github` de **repo privado**
  (`pguerrerolinares/exo`). Los repos privados como source funcionan con las
  credenciales git del usuario (doc: «Claude Code supports installing plugins
  from private repositories … uses your existing git credential helpers»;
  evidencia local: el marketplace agent-develop entero es PRIVATE y se clona/
  actualiza — `known_marketplaces.json` lastUpdated 2026-08-17, clon git en
  `~/.claude/plugins/marketplaces/agent-develop/.git`), pero el único
  plugin-source `github` funcionando en esta máquina (`workflow-lint`) es
  PUBLIC — el caso privado-como-plugin-source no está probado aquí, y la doc
  además avisa de auto-updates intermitentes con marketplaces privados (el
  background pull deshabilita credential helpers HTTPS). No rompe, pero es el
  camino con más incógnitas y no es ejecutable hoy. Queda como forma natural
  POST-M1b si Paul un día publica; no prejuzga el rename (M1b-01 sigue siendo
  decisión de Paul, plan línea 159).
- **(c) plugin por path sin marketplace**: no existe como vía de instalación —
  `claude plugin install` solo instala «from available marketplaces» (help del
  binario). Lo más cercano es `~/.claude/skills/<name>` (auto-load como
  `<name>@skills-dir`, help de `claude plugin init`), pero eso saca a `process`
  del mecanismo `enabledPlugins` y rompe la simetría del rollback de un flag.
  Descartado.

Matiz operativo que la secuencia A4 incorpora: al instalar, Claude Code **copia**
el plugin a `~/.claude/plugins/cache` (doc: «when users install a plugin, Claude
Code copies the plugin directory to a cache location»). Ediciones posteriores en
`exo/plugins/process` NO se propagan solas: hay que bump de `version` +
`claude plugin marketplace update exo` + `claude plugin update process@exo`.

## A2 — Qué le falta a `plugins/process/`. DECISIÓN: un solo fichero, `plugin.json`; el resto ya está

Comparación fichero a fichero contra los dos plugins funcionando:

| Pieza | paul-profile (vivo) | reflex (vivo) | process hoy | Falta |
|---|---|---|---|---|
| `.claude-plugin/plugin.json` | sí | sí | **NO** | **SÍ** |
| `skills/*/SKILL.md` con frontmatter | sí | sí | sí (7/7, verificado) | no |
| hooks/agents/scripts | hooks+scripts | hooks+agents+scripts | scripts (de orchestrate) | no — opcionales; process no declara hooks a propósito |
| README / LICENSES | README en reflex | — | README + LICENSES/superpowers.LICENSE | no (M3-04 ya satisfecho en repo desde prep-M3, spec prep-m3 §8) |

`claude plugin validate /home/paul/Documentos/proyectos/exo/plugins/process` →
«✔ Validation passed» (exit 0) — los componentes ya son válidos tal cual,
incluido el `:` dentro de las descriptions.

**Contenido adjudicado de `plugins/process/.claude-plugin/plugin.json`** —
campo a campo, cada uno presente en `paul-profile/.claude-plugin/plugin.json` y
`reflex/.claude-plugin/plugin.json` reales (leídos):

```json
{
  "name": "process",
  "description": "Skills del framework agéntico de Paul — el proceso de trabajo completo (brainstorm, plan, orchestrate, tdd, debug, verify, documenta) destilado de superpowers (MIT, Jesse Vincent) más doctrina propia. Sustituye a superpowers y a paul-profile:orchestrate-personal en el uso diario.",
  "version": "1.0.0",
  "author": {
    "name": "Paul Guerrero",
    "email": "pguerrerolinares@gmail.com"
  },
  "keywords": ["process", "skills", "framework", "exo"]
}
```

- `name` «process»: debe coincidir con el `name` de la entrada del marketplace
  (así funcionan paul-profile/reflex; `claude plugin tag --help` confirma que
  se valida «that plugin.json and any enclosing marketplace entry agree»).
- `version` «1.0.0» y se **bumpea en cada cambio**: doc §version: «If set …
  the plugin is pinned to this string and users only receive updates when it
  changes». Es el mecanismo de update que reflex ya usa (0.11.0→0.13.0 en
  `installed_plugins.json`).
- `description`/`author`/`keywords`: mismos campos y forma que los dos ejemplos
  vivos. Sin declaración de `skills`: ninguno de los dos plugins vivos la usa;
  el autodescubrimiento de `skills/` es el default.
- **No** se añade hooks.json ni nada más: process es solo skills por diseño
  (spec prep-m3 §2 «Layout del plugin» — el layout mergeado ES el de la spec).

**Y el `exo/.claude-plugin/marketplace.json`** (calcado del de agent-develop,
que funciona, con los campos `name/owner/metadata.pluginRoot/plugins[]`):

```json
{
  "name": "exo",
  "description": "Marketplace del framework exo — capa thin (skills de proceso) sobre el engine",
  "owner": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" },
  "metadata": { "version": "0.1.0", "pluginRoot": "./plugins" },
  "plugins": [
    {
      "name": "process",
      "source": "./plugins/process",
      "description": "Proceso de trabajo completo: brainstorm · plan · orchestrate · tdd · debug · verify · documenta",
      "version": "1.0.0",
      "author": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" }
    }
  ]
}
```

El plugin id resultante es `process@exo`. Verificación pre-cutover sin
instalar: `claude plugin validate <ambos paths>` (el subcomando valida «a
plugin or marketplace manifest», help del binario).

## A3 — Sustituto de using-superpowers. DECISIÓN: un bullet en «Doctrina compacta» de core-index; cabe con 1.419 bytes de margen

**Dónde**: `core-index`, no CLAUDE.md global ni skill propia. Razones:
(i) está adjudicado dos veces por fuentes firmadas — plan M3-05 («su sustituto
es una línea de routing en `core-index`») y spec prep-m3 §6.1 (diseña la línea
para core-index); no hay base citable para moverlo. (ii) using-superpowers
funciona porque un hook SessionStart lo FUERZA
(`cache/…/superpowers/6.3.0/hooks/hooks.json`, matcher `startup|clear|compact`)
— una skill dentro de process solo dispararía si el modelo la elige: usar una
skill para forzar el uso de skills es circular, y añadir un hook nuevo
contradice el «sustituto mínimo» de framework §5.2. (iii) core-index ya se
inyecta cada arranque por el recall de reflex y lo mantiene `/consolida`
(línea final de la propia nota); `~/.claude/CLAUDE.md` es un extracto
regenerado desde la KB — meter ahí routing operativo del harness invierte su
contrato («Fuente de verdad: nota [[Paul - perfil de trabajo]]»).

**Texto EXACTO** (bullet final de la sección `## Doctrina compacta`): base = el
texto propuesto en spec prep-m3 §6.1, MENOS la coletilla del contador de
no-disparos (retirado por plan §Campaña 8/«régimen §0»), MÁS la cláusula de
compliance que es el movimiento esencial de using-superpowers («Invoke relevant
or requested skills BEFORE any response or action — including clarifying
questions» + la regla del 1%, SKILL.md 6.3.0 leído; el resto — announce,
red-flags table, platform adaptation — es la prosa que framework §5.2 manda
tirar):

```
- ROUTING DE PROCESO (plugin `process`, sustituye a superpowers): brainstorm (diseño antes de código) · plan (spec→plan) · orchestrate (ejecutar plan multi-tarea) · tdd (test primero) · debug (bug o atasco) · verify (antes de declarar hecho) · documenta (cierre de sesión). Si una aplica — aunque sea al 1% — invócala ANTES de responder o actuar, incluidas las preguntas aclaratorias; no lo racionalices. Subagente ejecutando una tarea concreta: exento.
```

**Medida**: la línea son **469 bytes** (`wc -c`). core-index hoy: **4.255
bytes** (`wc -c`). Total 4.725 ≤ 6.144 → **cabe, no se recorta nada**. Margen
resultante: 1.419 bytes. Contra el cap real del bloque compuesto: el bloque de
`exo recall` (core-index + hasta 10 recientes) «ronda los 4,5 KB sobre un cap
de 6144» (`exo-recall.sh:39`); +469 ≈ 5,0 KB, sigue dentro, y el mecanismo
actual además trunca-con-aviso en vez de tirar el bloque (ver Contexto,
desviación #1).

Movimientos de using-superpowers preservados: invoke-before-any-response ✓,
regla del 1% / anti-racionalización ✓, exención de subagentes (SUBAGENT-STOP) ✓,
routing al catálogo ✓. Descartados con cita (framework §5.2: «se tira la prosa,
los gritos y los gates dogmáticos»): announce «Using [skill]…», tabla de red
flags, platform adaptation, precedencia de user instructions (ya es default del
harness).

Al mismo tiempo que se añade el bullet, la cabecera stale de core-index sobre
el FALLBACK puede corregirse o dejarse — NO es condición de M3-05; queda en
residuos.

## A4 — Orden, atomicidad y quién ejecuta qué. DECISIÓN de la tensión: SÍ, editar `enabledPlugins` es línea roja ⇒ acción de Paul

**La pregunta binaria primero.** Config
`.superpowers/fabrica/config.md:320-321`, dentro de «Línea roja que NUNCA se
delega» (§Ejecución de gates): *«Cambios a `.claude/settings.json` o a guards
PreToolUse (perímetro de permisos del propio harness)»*. **SÍ cae dentro**:

1. La letra nombra el **fichero**, no una sección del fichero. `enabledPlugins`
   vive en `~/.claude/settings.json` (leído: ahí están los 12 flags, incluido
   `superpowers@claude-plugins-official: true`). No existe otro settings.json
   en juego: `exo/.claude/` solo contiene `RESUME.md` (find) — la única lectura
   con referente real es el global.
2. La glosa «perímetro de permisos del propio harness» no lo estrecha, lo
   confirma: `enabledPlugins` decide qué hooks corren — apagar/encender plugins
   es exactamente mover el perímetro del harness (reflex, que ES los guards
   PreToolUse, se activa por esa misma clave).
3. Coherencia con el resto de la clase (plan §0: «Acciones destructivas o
   externas siguen siendo de Paul») y con la regla de cierre del propio config:
   las líneas rojas las relaja Paul con `OVERRIDE` registrado, no un consultor
   interpretando el paréntesis a la baja. Si Paul quiere delegar el toggle, es
   un OVERRIDE de una línea, barato y explícito.

Consecuencia: `claude plugin install/enable/disable` (escriben
`enabledPlugins`/`installed_plugins.json`) y el `marketplace add` (registra en
`known_marketplaces.json`/`extraKnownMarketplaces`) son **de Paul**. Son 4
comandos, ~2 minutos, y forman un único gesto atómico.

**Secuencia adjudicada (M3-01):**

*Fase 1 — orquestador, rama de campaña, antes del día D:*

1. Crear `plugins/process/.claude-plugin/plugin.json` y
   `.claude-plugin/marketplace.json` (contenidos A2).
2. Evidencia: `claude plugin validate plugins/process` y
   `claude plugin validate .claude-plugin/marketplace.json` en verde.
3. M3-03 (actualizar fabrica: referencias `superpowers:subagent-driven-development`
   y `paul-profile:orchestrate-personal` → `process:orchestrate`) y M3-05
   (bullet A3 en `kb-demo/core/core-index.md`, commit en kb-demo SIN
   push). M3-04: ya satisfecho (LICENSES + README/Atribución verificados);
   solo re-confirmar en el review-package.
4. Gate de merge (consultor fable, régimen §Ejecución de gates) → `GATE-EXEC`
   → merge a main de exo. **Hasta aquí nada ha tocado el entorno vivo**: el
   marketplace no está registrado, superpowers sigue activo.

*Fase 2 — Paul, el día D, en ESTE orden (process ON antes de superpowers OFF —
así la línea roja «no desactives superpowers sin process listo el mismo día»
se cumple incluso si un paso intermedio falla: no hay ventana sin skills):*

5. `claude plugin marketplace add /home/paul/Documentos/proyectos/exo`
6. `claude plugin install process@exo`  (queda enabled por defecto —
   `defaultEnabled` default true, doc §marketplace schema)
7. Verificar: `claude plugin list` muestra `process@exo` enabled.
8. `claude plugin disable superpowers@claude-plugins-official`
9. Reiniciar sesión. Probes: las skills aparecen como `process:*`; y el probe
   M3-02 de spec §5.3 (dispatch `subagent_type: reflex:executor` en reflex-log
   — el texto está en `orchestrate/SKILL.md:14`, verificado).

*Rollback (un gesto, Paul):*
`claude plugin enable superpowers@claude-plugins-official` +
`claude plugin disable process@exo`, reiniciar sesión. superpowers queda
instalado-pero-apagado ≥ un ciclo real (spec §5.3.3); se desinstala «cuando el
ciclo cierre sin carencias» (§5.3.4) — sin contador, por el régimen §0.

*Post-cutover, cada vez que se toquen las skills:* bump `version` en
`plugin.json` + entrada del marketplace (mismo commit), y Paul (o quien él
autorice, ya que `marketplace update`/`plugin update` no tocan settings.json —
solo cache y `installed_plugins.json`; adjudico que ESO no es línea roja):
`claude plugin marketplace update exo && claude plugin update process@exo`.

## Qué busqué para objetar

- **«El brief miente sobre el estado de process»**: intenté encontrar
  plugin.json o registro en marketplace — no existen (find + lectura de
  marketplace.json de agent-develop + installed_plugins.json). El brief es
  correcto ahí. La paridad 135/135 la verifiqué contra el verdict real, no me
  la creí (RECHAZADA inicial en L9 → fix fc51e09 → MERGED L287).
- **«Repo privado rompe el source github»** (hipótesis a favor de path local):
  parcialmente refutada — el marketplace privado agent-develop funciona con
  credenciales git del usuario (evidencia local + doc). PERO el único
  plugin-source `github` vivo (workflow-lint) resultó ser PUBLIC, así que
  «plugin privado por source github» no tiene prueba local; elegí un camino
  que no depende de ello.
- **«La línea no cabe en core-index»**: refutada midiendo (469 + 4.255 =
  4.725 ≤ 6.144). También busqué que el cap real fuera otro: encontré que el
  mecanismo cambió (exo-recall trunca-con-aviso, no FALLBACK) — desviación #1
  del brief, y stale-text en la propia nota.
- **«El texto de la spec §6.1 va tal cual»**: refutada — su coletilla depende
  del contador de no-disparos que el plan C8 retira explícitamente. Contradicción
  spec↔plan real; la resolví por cronología y firma (plan 2026-08-17, Paul).
- **«enabledPlugins NO es línea roja»** (la lectura estrecha del paréntesis):
  la construí en serio — `enabledPlugins` no es `permissions`, y el plan llama
  al rollback «de un flag» sin asignarlo. La descarté porque la letra nombra el
  fichero completo, el único settings.json existente es el global, y estrechar
  una línea roja por interpretación es exactamente lo que el config prohíbe a
  un consultor. El SÍ es además barato: son 4 comandos de Paul.
- **«Hay una vía sin marketplace»**: busqué en el CLI real — `plugin install`
  solo instala desde marketplaces; skills-dir existe pero rompe la simetría del
  rollback. No hay tercera vía mejor.

## Residuos / escaladas

1. **Cachés huérfanas** (deuda C8 declarada en plan §Deuda suelta): confirmadas
   y la lista real es MAYOR que la anotada — reflex `0.6.0, 0.8.0, 0.11.0,
   0.12.0` y paul-profile `0.2.1` en `~/.claude/plugins/cache/agent-develop/`.
   Borrarlas toca `~/.claude` fuera de settings.json: no es línea roja literal,
   pero es entorno vivo — recomiendo dejarlo en el gesto de Paul del día D
   (`claude plugin prune` existe en el CLI para esto, help verificado).
2. **Stale-text de core-index** sobre el FALLBACK (desviación #1): corregirlo
   es un retoque de la misma edición M3-05 o de `/consolida`; no bloquea.
3. **README de process** dice «destilado de superpowers 6.1.1»; la instalada
   viva es 6.3.0. La paridad se verificó contra 6.1.1 por decisión de spec —
   no es error, pero si algún movimiento nuevo de 6.2/6.3 duele, la doctrina
   es «se añade si duele» (framework §5.2). Informativo.
4. **M1b-01 (rename de agent-develop) y M1b-02 (workflow-lint)**: siguen
   siendo decisión/acción de Paul (plan línea 159, «acción externa»). Este
   verdict no las prejuzga: el marketplace `exo` local convive con
   agent-develop tal cual está.
5. Escalada NINGUNA: las cuatro preguntas cierran con fuente citable.
