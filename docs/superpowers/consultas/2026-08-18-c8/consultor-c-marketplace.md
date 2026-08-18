# Verdict consultor C — Marketplace, identidad y acción externa (C8: M1b-01, M1b-02)

- **Fecha**: 2026-08-18
- **Rol**: consultor independiente, adjudica (no asesora). Verificación primaria propia.
- **Alcance**: M1b-01 (rename vs repo nuevo + nombre destino), M1b-02 (workflow-lint), foto final de `marketplace.json`, runbook de Paul, riesgos externos.
- **Fuera de alcance** (otros consultores): mecánica de instalación de `process` (consultor A), barrido de dependencias de superpowers (consultor B). Donde toco sus fronteras, mi decisión queda robusta a las suyas.

## Decisiones (resumen)

| Ítem | Decisión |
|---|---|
| C1(i) | **RENAME** del repo `pguerrerolinares/agent-develop`. No repo nuevo. |
| C1(ii) | Repo destino: **`pguerrerolinares/exo-plugins`**. Campo `name` de `marketplace.json`: **`exo`** — se cambia A LA VEZ que el repo. Ids finales: `process@exo`, `reflex@exo`, `paul-profile@exo`, `workflow-lint@exo`. |
| C3 | `workflow-lint` **ENTRA**, tal cual está hoy: entrada en el marketplace con source al repo aparte (`pguerrerolinares/workflow-lint`, público). No se consolida al monorepo. |
| C4 | Foto final del JSON en §C4 (4 plugins: process, reflex, paul-profile, workflow-lint). |
| Extra | El cron `a1-freeze-watch` (residuo de M1a que C6 no ejecutó, ver evidencia) **se retira** en el mismo runbook: es el único consumidor operativo del string `agent-develop` fuera de la config de plugins. |

---

## 1. Verificación primaria del terreno

Todo lo del brief se comprobó. Un matiz factual que el brief no daba y cambia un riesgo:

| Afirmación | Estado | Evidencia |
|---|---|---|
| Marketplace en `agent-develop/.claude-plugin/marketplace.json`, name `agent-develop`, sirve paul-profile + reflex (relative path) + workflow-lint (repo externo) | ✅ | `cat` del fichero: `"name": "agent-develop"`, `pluginRoot: "./plugins"`; workflow-lint con `{"source":"github","repo":"pguerrerolinares/workflow-lint"}` |
| Repo `pguerrerolinares/agent-develop` PRIVADO | ✅ | `gh repo view --json visibility` → `"PRIVATE"`, default branch `master` |
| **`pguerrerolinares/workflow-lint` es PÚBLICO** (el brief no lo decía) | ✅ | `gh repo view` → `"visibility":"PUBLIC"`. Elimina el riesgo de auth para ese source |
| `pguerrerolinares/exo` existe, privado, distinto del marketplace | ✅ | `gh repo view` → `"PRIVATE"`, updated 2026-08-18. **No hay repo `exo-plugins`** en la lista de 68 repos del usuario → el nombre destino está libre |
| `extraKnownMarketplaces.agent-develop` con `autoUpdate: true`; `enabledPlugins` con 3 ids `@agent-develop` | ✅ | `~/.claude/settings.json` (5 ocurrencias de `agent-develop`) |
| Caché en `~/.claude/plugins/cache/agent-develop/<plugin>/<version>` | ✅ | Instalados: reflex **0.13.0**, paul-profile 0.5.0, workflow-lint 0.1.0. Huérfanas: reflex 0.6.0/0.8.0/0.11.0/0.12.0, paul-profile 0.2.1 (el plan ya las marca para C8, línea 264) |
| Clon del marketplace | ✅ | `~/.claude/plugins/marketplaces/agent-develop` es un clon git con `origin` → `agent-develop.git`, en el commit `fe4d2d6` (reflex 0.13.0). También registrado en `~/.claude/plugins/known_marketplaces.json` e `installed_plugins.json` |
| Uso de workflow-lint (`pluginUsage`) | ✅ | `workflow-lint@agent-develop`: **usageCount 39, último uso 2026-07-14** (hace 35 días). Comparativa: reflex 50.354, paul-profile 24.580 |
| Drift interno detectado | ⚠️ | `marketplace.json` declara reflex `0.11.0` pero `plugins/reflex/.claude-plugin/plugin.json` dice `0.13.0` (y 0.13.0 es lo instalado). La versión del catálogo está desincronizada — se corrige de paso en C4 |
| `process` en `exo/plugins/process` (7 skills) | ✅ | `brainstorm debug documenta orchestrate plan tdd verify`. **No tiene `.claude-plugin/plugin.json`** — precondición para entrar al catálogo (terreno del consultor A; lo dejo como precondición P1 del runbook) |
| Cron activo que rompería | ✅ | `crontab -l` → `7 9 * * * .../reflex/scripts/a1-freeze-watch.sh`. El script hardcodea `CACHE_DIR="$PLUGINS_DIR/cache/agent-develop"` (línea 45) y las keys jq `reflex@agent-develop` / `paul-profile@agent-develop` (líneas 100-101). El plan ya lo lista como deuda: «`crontab -r` pendiente, residuo de M1a → C6 (toca entorno)» (plan línea 261) — **C6 cerró sin ejecutarlo**; lo hereda este runbook |

### Conteo real de sitios que dependen del string `agent-develop` (grep ejecutado)

Operativos (rompen o quedan ciegos si cambia el name del marketplace):
1. `~/.claude/settings.json` — 5 ocurrencias (clave de `extraKnownMarketplaces`, campo `repo`, 3 keys de `enabledPlugins`).
2. `~/.claude/plugins/known_marketplaces.json` + `installed_plugins.json` — gestionados por el harness; se regeneran con remove/add.
3. `~/.claude/plugins/cache/agent-develop/` y `~/.claude/plugins/marketplaces/agent-develop/` — rutas derivadas del name; se regeneran.
4. `agent-develop/plugins/reflex/scripts/a1-freeze-watch.sh` (4 refs) + su cron. Su test `test-a1-freeze-watch.sh` (~20 refs) — muere con el cron.

No operativos (no rompen nada): `~/.claude.json` (`pluginUsage` ×3 histórico, `githubRepoPaths`, `projects` — el harness los regenera o son telemetría), 181 menciones en `exo/**/*.md` (docs/evals históricos), notas de la KB kb-demo, `fabrica/SKILL.md:11` (referencia una RUTA de disco `agent-develop/docs/...`, que no cambia porque **no renombramos el directorio local**). Los hooks de reflex usan `${CLAUDE_PLUGIN_ROOT}` (verificado en `hooks.json`), así que sobreviven a cualquier rename sin tocarlos.

### Verificación primaria de los redirects de GitHub (docs oficiales, fetch 2026-08-18)

De `docs.github.com/.../renaming-a-repository`:
- **Sí redirige**: web (issues, wikis, stars, watchers) y **«all `git clone`, `git fetch`, or `git push` operations targeting the previous location will continue to function as if made on the new location»**. La API REST también responde 301 al nombre viejo.
- **No redirige**: GitHub Pages y **actions referenciadas por workflows** («GitHub will not redirect calls to an action hosted by a renamed repository»). Ninguna de las dos aplica aquí (no hay Pages ni actions consumidas por nombre).
- **Qué los rompe**: «If you create a new repository under your account [...] do not reuse the original name [...] redirects will no longer work.»
- **Privado**: la privacidad no cambia los redirects; solo exige credenciales, que ya funcionan (`gh auth status`: logged in, keyring, scope `repo`; credential helper `!/usr/bin/gh auth git-credential` configurado — verificado).

De `code.claude.com/docs/en/plugin-marketplaces` (guardada en tool-results, líneas citadas):
- El sufijo de los ids sale del **campo `name` de `marketplace.json`**, no de la clave de settings: «marketplace name to remove [...] This is the `name` from `marketplace.json`, not the source you passed to `add`» (L1161); «Marketplace identifier [...] users see it when installing plugins (for example, `/plugin install my-tool@your-marketplace`)» (L161).
- Existe un mecanismo `renames` **para plugins** (v2.1.193+; instalada 2.1.234 — verificado), **no para el name del marketplace**. Cambiar el name = remove + add, sin migración automática.
- «Each user can register only one marketplace per name: adding a second marketplace with the same name replaces the first» (L161) — esto hace trivial una futura mudanza de repo manteniendo el name `exo`.
- Auto-update background de marketplaces privados por HTTPS: «the background refresh disables git credential helpers for its `git pull` [...] When the background pull fails, Claude Code falls back to re-cloning the marketplace from scratch. The re-clone does use your stored git credentials» (L677) — funciona, con posible intermitencia; irrelevante para repos de 5 MB (medido: `.git` de agent-develop = 5,4M).
- Source `git-subdir` existe para servir un plugin desde un subdirectorio de otro repo git, con sparse clone (tabla de Plugin sources).

---

## C1 — Adjudicación: RENAME a `exo-plugins`, name del catálogo a `exo`

### C1(i): rename, no repo nuevo

**Adjudicado: RENAME.** Fundamento:
1. Los redirects cubren exactamente la pieza frágil: el clon vivo de `~/.claude/plugins/marketplaces/` y cualquier referencia rezagada siguen funcionando desde el segundo cero, sin ventana en la que reflex (50k usos, sirve los hooks de cada sesión) se quede sin servir. Con repo nuevo, la spec misma ya señalaba el «riesgo de reflex servido stale» (spec línea 170).
2. Se conserva historia, issues y el registro tal cual; el coste es un comando.
3. Repo privado: verificado que no cambia nada de los redirects (solo auth, ya resuelta por el helper de gh).

**Alternativa que consideré en serio y descarto** (disenso §"Qué busqué para objetar"): mudar el marketplace al repo `exo` directamente (la spec original firmó «agent-develop se absorbe al monorepo», spec líneas 14-15 y 169). La descarto para C8 porque exige mover físicamente reflex y paul-profile (con desarrollo activo: reflex 0.13.0 mergeado ayer, `fe4d2d6`) — trabajo real no presupuestado en un C8 que se define como «cutover, no desarrollo» (plan línea 148-150), y el plan vigente (2026-08-17, posterior a la spec) redujo M1b a rename-vs-nuevo. La decisión de nombre de abajo deja esa absorción futura a coste casi cero si algún día se hace.

### C1(ii): nombre destino y campo `name`

**Adjudicado: repo `pguerrerolinares/exo-plugins`, y el campo `name` de `marketplace.json` cambia A LA VEZ a `exo`.**

- `exo` a secas para el repo es imposible: `pguerrerolinares/exo` ya existe (el engine). `exo-plugins` describe lo que el repo contiene (los plugins + el catálogo), kebab-case, libre (verificado contra los 68 repos del usuario), y no colisiona con nombres reservados de Anthropic (lista verificada en la doc, L166).
- El `name` del JSON pasa a `exo` porque es lo que Paul ve a diario: los ids `reflex@exo`, `process@exo` son la identidad del framework. No hace falta que coincida con el nombre del repo (doc L1161: son cosas distintas).
- **Aquí disiento parcialmente del plan**: el plan/spec vendían el rename como «mantiene la identidad» (es decir, `@agent-develop` intacto, coste cero). Es verdad que dejar el name intacto costaría ~0. Pero C8 ya obliga a cirugía de `enabledPlugins` ese mismo día (M3-01: superpowers off + process on, más el install de `process`): la ventana de cambio está abierta de todos modos, el coste marginal de renovar el sufijo es ~10 minutos una única vez, y la alternativa es arrastrar `@agent-develop` para siempre en un framework que se llama exo. Es la única oportunidad barata; después vuelve a costar un remove/add completo.
- **Coste exacto del cambio de `name`, contado** (ver §1): 5 ocurrencias en `settings.json` (las escribe el propio flujo remove/add + un ajuste manual de `enabledPlugins`), 2 ficheros de estado del harness que se regeneran solos, 2 directorios de caché/clon que se regeneran solos, y 1 script con cron (4 refs) + su test (~20 refs) **que se retiran** en vez de migrarse, porque el gate que vigilaban está cerrado por el régimen §0 (GATE-CALENDARIO-D cerrado, plan líneas 30-38) y su retirada ya estaba adjudicada como deuda de C6 (plan línea 261). Nada más: cero scripts en exo, cero hooks (usan `${CLAUDE_PLUGIN_ROOT}`), cero en la KB con efecto operativo.
- El name `exo` es además **robusto a la absorción futura**: si un día el catálogo se muda al repo `exo` (spec §M1a), basta re-añadir el marketplace desde el repo nuevo — mismo name ⇒ «replaces the first» (doc L161) — sin tocar ids ni `enabledPlugins` nunca más.
- **No se renombra el directorio local** `/home/paul/Documentos/proyectos/agent-develop` en C8: hay rutas de disco escritas en docs y en `fabrica/SKILL.md:11`; renombrarlo compra estética y rompe referencias. YAGNI. (Si Paul quiere, es un `mv` + `remote -v` otro día; nada del runbook depende de ello.)

---

## C3 — Adjudicación: `workflow-lint` ENTRA, tal cual

**Adjudicado: entra en el marketplace nuevo, con la misma entrada de hoy** (source `github` → `pguerrerolinares/workflow-lint`, público, versión 0.1.0). No se consolida al monorepo.

Datos delante:
- Uso real: 39 usos, último 2026-07-14 (35 días). Bajo pero no muerto; está `enabled` y su skill dispara ante cualquier uso del tool Workflow.
- Coste de mantenerlo: **una entrada JSON que ya existe y un repo público que no se toca desde junio**. Cero mantenimiento, cero auth (público — verificado).
- Coste de sacarlo: Paul pierde el lint preflight de workflows y queda un plugin instalado-huérfano que igualmente habría que limpiar.
- Coste de consolidarlo al monorepo: mover un repo funcional para no ganar nada. El régimen §0 (YAGNI, cerrar ya) mata esta opción, y la spec ya lo había firmado igual: «Quedan fuera: workflow-lint (repo propio, referenciado por el marketplace como hoy)» (spec línea 15).

---

## C4 — Foto final de `marketplace.json` tras C8

```json
{
  "name": "exo",
  "description": "Marketplace personal del framework exo — perfil, reflejos y proceso de Paul Guerrero",
  "owner": {
    "name": "Paul Guerrero",
    "email": "pguerrerolinares@gmail.com"
  },
  "metadata": {
    "version": "0.2.0",
    "pluginRoot": "./plugins"
  },
  "plugins": [
    {
      "name": "process",
      "source": {
        "source": "git-subdir",
        "url": "https://github.com/pguerrerolinares/exo.git",
        "path": "plugins/process"
      },
      "description": "Proceso destilado (sustituye a superpowers): brainstorm, plan, orchestrate, tdd, debug, verify, documenta.",
      "version": "0.1.0",
      "author": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" }
    },
    {
      "name": "paul-profile",
      "source": "./plugins/paul-profile",
      "description": "fabrica: campaign harness A-thin (sesiones-fábrica largas con gate de merge asíncrono).",
      "version": "0.6.0",
      "author": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" }
    },
    {
      "name": "reflex",
      "source": "./plugins/reflex",
      "version": "0.13.0",
      "author": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" }
    },
    {
      "name": "workflow-lint",
      "source": {
        "source": "github",
        "repo": "pguerrerolinares/workflow-lint"
      },
      "version": "0.1.0",
      "author": { "name": "Paul Guerrero", "email": "pguerrerolinares@gmail.com" }
    }
  ]
}
```
(Las `description` de reflex y workflow-lint se conservan de las actuales; las omito aquí por brevedad, no se borran.)

Justificación por entrada:
- **process** (nueva): es el objeto de M3-01. Source adjudicado por defecto: `git-subdir` al repo `exo` (su fuente de verdad, donde co-evoluciona con el engine y sus evals `prep-m3`); sparse clone, repo de 5 MB, credenciales ya resueltas. **Robustez frente al consultor A**: si A adjudica copiar `process` físicamente a `exo-plugins/plugins/process`, la única línea que cambia es `"source": "./plugins/process"` — el resto de esta foto y todo el runbook quedan idénticos. Precondición en cualquier caso (P1): `process` necesita `.claude-plugin/plugin.json` (hoy no lo tiene — verificado).
- **paul-profile 0.6.0**: M3 mata `orchestrate-personal` (sustituida por `process:orchestrate`, plan M3-02) y M3-03 actualiza `fabrica`; queda como plugin de una sola skill (`fabrica`), coherente con la spec («paul-profile menos fabrica» se absorbe; fabrica queda como plugin de instancia, spec líneas 14-15). Bump a 0.6.0 por el cambio de contenido. **Robustez**: si otro consultor mueve `fabrica` a `process`, paul-profile queda vacío ⇒ se elimina su entrada y se añade `"renames": { "paul-profile": null }` al nivel raíz (mecanismo verificado en doc L1006-1036, disponible en la 2.1.234 instalada); nada más cambia.
- **reflex 0.13.0**: sin cambios de contenido en C8; se corrige el drift del catálogo (decía 0.11.0 con plugin.json en 0.13.0 — verificado). Regla de higiene que deja esto resuelto a futuro: la versión del catálogo se sincroniza con `plugin.json` en cada release.
- **workflow-lint 0.1.0**: adjudicado en C3, entrada intacta.
- **Se retira**: nada más — no hay más plugins hoy. `metadata.version` 0.1.0 → 0.2.0 (cambio de identidad del catálogo).

---

## C2 — Runbook literal

Precondiciones (bloquean el arranque):
- **P1** `exo/plugins/process/.claude-plugin/plugin.json` existe y el cutover M3 está listo para activarse (terreno del consultor A; si A elige copia física en vez de git-subdir, ajustar la entrada de process según §C4).
- **P2** paul-profile actualizado (M3-03 fabrica, retirada de orchestrate-personal) y commiteado en agent-develop local.
- **P3** Cerrar TODAS las sesiones de Claude Code antes de la Fase 2 (evita plugins resolviéndose a mitad de sesión).

### Fase 0 — Preparación local, sin push `[ORQUESTADOR]`

```bash
# 0.1 — editar el catálogo con la foto final de §C4 (name "exo", process, versiones sincronizadas)
#        fichero: /home/paul/Documentos/proyectos/agent-develop/.claude-plugin/marketplace.json
# 0.2 — commit local (NO push):
git -C /home/paul/Documentos/proyectos/agent-develop add .claude-plugin/marketplace.json plugins/paul-profile
git -C /home/paul/Documentos/proyectos/agent-develop commit -m "feat(marketplace): identidad exo — name exo, alta de process, sync de versiones (M1b)"
```

### Fase 1 — Rename del repo `[PAUL]`

```bash
gh repo rename exo-plugins -R pguerrerolinares/agent-develop --yes
# Verificar:
gh repo view pguerrerolinares/exo-plugins --json name,visibility
# Esperado: {"name":"exo-plugins","visibility":"PRIVATE"}
```

### Fase 2 — Remote local y push `[PAUL]`

```bash
git -C /home/paul/Documentos/proyectos/agent-develop remote set-url origin https://github.com/pguerrerolinares/exo-plugins.git
git -C /home/paul/Documentos/proyectos/agent-develop push origin master
# Verificar:
git -C /home/paul/Documentos/proyectos/agent-develop remote -v
# Esperado: origin  https://github.com/pguerrerolinares/exo-plugins.git (fetch y push)
git -C /home/paul/Documentos/proyectos/agent-develop log origin/master --oneline -1
# Esperado: el commit de la Fase 0
```

### Fase 3 — Re-registro del marketplace `[PAUL]` (sesiones cerradas, seguido y sin pausa)

```bash
claude plugin marketplace remove agent-develop
claude plugin marketplace add pguerrerolinares/exo-plugins
claude plugin marketplace list
# Esperado: aparece "exo" con source pguerrerolinares/exo-plugins (y agent-develop ya no está)

claude plugin install reflex@exo
claude plugin install paul-profile@exo
claude plugin install workflow-lint@exo
claude plugin install process@exo
```

Si `marketplace remove` se niega por plugins activos: `claude plugin disable reflex@agent-develop paul-profile@agent-develop workflow-lint@agent-develop` primero y repetir.

### Fase 4 — settings.json `[PAUL]` (editar `~/.claude/settings.json`)

En `enabledPlugins`: eliminar las tres keys `*@agent-develop` y dejar (coordinado con M3-01 del consultor A):

```json
"workflow-lint@exo": true,
"reflex@exo": true,
"paul-profile@exo": true,
"process@exo": true,
"superpowers@claude-plugins-official": false
```

En `extraKnownMarketplaces`: confirmar que la entrada vieja `agent-develop` ya no existe y que la nueva quedó con auto-update:

```json
"exo": {
  "source": { "source": "github", "repo": "pguerrerolinares/exo-plugins" },
  "autoUpdate": true
}
```

### Fase 5 — Retirar el cron residual `[PAUL]`

```bash
crontab -l | grep -v "a1-freeze-watch" | crontab -
crontab -l | grep a1 ; echo "exit=$?"
# Esperado: sin líneas, exit=1
```

(Adjudicado: el gate A1 que vigilaba está cerrado por régimen §0 — GATE-CALENDARIO-D cerrado, plan líneas 30-38 — y su retirada ya era deuda declarada de C6, plan línea 261. Si se prefiriera conservarlo, habría que reescribir sus rutas `cache/agent-develop` y keys `@agent-develop`; no compensa: vigila un freeze que ya no existe.)

### Fase 6 — Verificación end-to-end `[PAUL]`

```bash
ls ~/.claude/plugins/cache/exo/
# Esperado: paul-profile  process  reflex  workflow-lint
git -C ~/.claude/plugins/marketplaces/exo remote -v
# Esperado: origin https://github.com/pguerrerolinares/exo-plugins.git
claude plugin list 2>/dev/null | grep "@exo"
# Esperado: los 4 plugins @exo, enabled
```

Y una sesión de humo: arrancar `claude` en cualquier proyecto y comprobar que (a) el recall de arranque de reflex aparece, (b) las skills `process:*` están listadas, (c) no hay warning de `plugin-not-found`.

### Fase 7 — Limpieza (solo tras Fase 6 verde) `[PAUL]`

```bash
rm -rf ~/.claude/plugins/cache/agent-develop
# (entierra de paso las cachés huérfanas reflex 0.6.0/0.8.0/0.11.0/0.12.0 y paul-profile 0.2.1 — deuda del plan línea 264)
```

### Rollback (cualquier fase)

```bash
# 1. Deshacer el rename (los redirects del nombre nuevo pasan a apuntar al viejo):
gh repo rename agent-develop -R pguerrerolinares/exo-plugins --yes
# 2. Revertir el catálogo y pushear:
git -C /home/paul/Documentos/proyectos/agent-develop revert --no-edit HEAD
git -C /home/paul/Documentos/proyectos/agent-develop remote set-url origin https://github.com/pguerrerolinares/agent-develop.git
git -C /home/paul/Documentos/proyectos/agent-develop push origin master
# 3. Re-registrar como antes:
claude plugin marketplace remove exo
claude plugin marketplace add pguerrerolinares/agent-develop
claude plugin install reflex@agent-develop
claude plugin install paul-profile@agent-develop
claude plugin install workflow-lint@agent-develop
# 4. Restaurar las keys viejas en enabledPlugins (y reactivar superpowers si M3 también se revierte).
```

Mientras la Fase 7 no se haya ejecutado, la caché vieja sigue en disco: el rollback es completo y sin pérdida. Único punto de no retorno parcial: crear un repo NUEVO llamado `agent-develop` (mataría los redirects) — no hacerlo nunca.

---

## C5 — Riesgos externos y mitigaciones

| # | Riesgo | Mitigación concreta |
|---|---|---|
| 1 | Reutilizar el nombre viejo (crear otro repo `agent-develop`) rompe los redirects de GitHub silenciosamente | Prohibición explícita en el runbook. Además, tras Fase 2-3 NADA depende ya de los redirects (remote y settings apuntan al nombre nuevo): los redirects quedan solo como red de seguridad |
| 2 | Auto-update background de marketplace privado por HTTPS: el pull deshabilita credential helpers → falla → re-clone con credenciales, «may fail intermittently» (doc L677) | Riesgo preexistente (ya opera así hoy) y benigno: repo de 5,4 MB, el re-clone con el helper de gh funciona; verificado `gh auth status` con scope `repo` y helper configurado. Si molestara: pasar el remote a SSH (los background pulls sí autentican por ssh-agent, doc L677) |
| 3 | Plugins sin resolver a mitad de sesión durante el switch (hooks de reflex muertos = sesiones sin recall) | P3: sesiones cerradas; Fases 3-4 se ejecutan seguidas (< 5 min); verificación de humo antes de reabrir trabajo |
| 4 | `plugin-cache-miss` tras el re-add para sources remotos (workflow-lint, process) — comportamiento documentado (doc L1026) | El runbook instala explícitamente los 4 con `claude plugin install` en Fase 3; no se confía en migración automática (no existe para cambio de name de marketplace) |
| 5 | El cron `a1-freeze-watch` (diario, 09:07) queda ciego o da falsas alarmas con las rutas nuevas | Se retira en Fase 5 (gate cerrado por §0; deuda de C6 ya declarada). Verificación con `crontab -l` |
| 6 | `marketplace remove` + `add` deja residuo o se niega con plugins enabled | Paso alternativo documentado (disable previo); Fase 4 verifica a mano `extraKnownMarketplaces` y `enabledPlugins`; Fase 6 verifica caché, clon y listado |
| 7 | Push del catálogo con name `exo` ANTES del re-add: el registro viejo (`agent-develop`) cargaría un catálogo cuyo name no coincide | Orden del runbook: el push (Fase 2) y el re-add (Fase 3) van en la misma ventana con sesiones cerradas; la inconsistencia vive minutos y sin consumidores |
| 8 | `process` servido desde repo privado `exo` vía git-subdir necesita credenciales | Instalación interactiva usa el helper de gh (verificado). Si el consultor A opta por copia física al marketplace, el riesgo desaparece entero |
| 9 | Sesiones/agentes de la propia campaña C8 (worktree `.worktrees/c8-m3` activo) pisándose con el switch | El switch es la ÚLTIMA acción de C8, tras el merge del cutover M3; lo ejecuta Paul con la fábrica parada |

---

## Qué busqué para objetar (mandato de disenso)

1. **Intenté refutar el rename** con dos alternativas serias: (a) repo nuevo limpio — cae por el riesgo de reflex stale en el intervalo y porque no aporta nada que el rename no dé; (b) mudar el catálogo al monorepo `exo` ya, que es lo que la spec original firmó (líneas 14-15, 169: «agent-develop se absorbe al monorepo», «M1a: crear repo, absorber historia») — encontré que **M1a nunca se ejecutó como estaba firmado** (el repo exo existe pero no absorbió ni la historia ni reflex/paul-profile, que siguen desarrollándose en agent-develop; evidencia: `fe4d2d6` ayer). Ejecutar la absorción hoy es desarrollo real en una campaña definida como cutover. La objeción queda registrada como drift spec↔realidad, no como bloqueo: el name `exo` adjudicado hace la absorción futura casi gratuita (mismo name ⇒ replace, doc L161).
2. **Intenté refutar mi propio cambio de `name`** (la spec vendía «identidad `@agent-develop` no cambia — preferido», línea 170): la defensa del status quo es real (coste 0). La rebatí con el conteo: el coste del cambio es una ventana que C8 abre de todos modos (M3-01 toca `enabledPlugins` sí o sí) + un cron que ya estaba condenado; y el statu quo perpetúa una identidad muerta en cada id. Si Paul prefiere el coste cero absoluto, la variante degradada es: mismo runbook saltando el cambio de `name` (ids siguen `@agent-develop`, solo Fases 1-2 y el fix del `repo` en settings) — la dejo nombrada como **variante mínima**, pero NO es la adjudicada.
3. **Busqué consumidores ocultos del string** con grep en exo, agent-develop, kb-demo, `~/.claude/settings.json`, `~/.claude.json`, hooks, crontab y systemd —lo único operativo fuera de la config fue el cron a1 (y su test). Busqué también procesos vivos (`pgrep`): nada corriendo.
4. **Busqué razones para expulsar workflow-lint** (uso bajo: 39 vs 50k de reflex; último uso hace 35 días): no las hay que superen el coste-cero de mantenerlo y el coste real de sacarlo; además la spec ya lo firmó dentro. También comprobé que su repo es público (elimina el único riesgo plausible, el de auth).
5. **Falsedades del brief**: ninguna sustantiva. Dos matices: workflow-lint es público (el brief no lo afirmaba pero convenía saberlo) y el catálogo declara reflex 0.11.0 estando 0.13.0 instalado (drift que la foto C4 corrige).
