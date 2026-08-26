# Ola 1B — Plugin único `exo` · Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `process:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** fusionar los plugins `process` y `reflex` en uno solo llamado `exo`,
con los nueve skills bajo el prefijo `exo:`, sin que ningún gate quede
desconectado en silencio durante el cutover.

**Architecture:** `plugins/exo/` nace por `git mv` de los dos plugins actuales
para conservar la historia; los dos skills en español se renombran
(`documenta`→`document`, `consolida`→`distill`); y el inventario de cutover
—diez puntos, incluido un shim que vive fuera de este repo— se barre en el
mismo commit que el rename, porque el fallo de referirse a un plugin que ya no
existe es **silencioso** en al menos dos de esos diez sitios.

**Tech Stack:** Claude Code plugins (skills, agents, hooks) · Bash (Git Bash en
Windows 11) · jq · marketplace `exo` en el repo `exo-plugins`.

## Global Constraints

- Spec fuente: `docs/superpowers/specs/2026-08-26-exo-generico-design.md` (v2),
  sección G2. Ante conflicto, manda la spec.
- **Nombre y versión, verbatim:** plugin `exo`, versión `1.0.0`. Id nuevo, no
  continuación de `reflex 0.17.0`.
- **Los nueve skills:** `brainstorm`, `plan`, `orchestrate`, `tdd`, `debug`,
  `verify`, `document`, `distill`, `recon-first`. Agente: `executor`.
- **A1 (adjudicada):** `process` y `reflex` se retiran del marketplace de
  golpe, sin deprecación. `exo-plugins` es privado, así que no hay terceros:
  solo las dos máquinas de Paul.
- **A2 (adjudicada):** `paul-profile` NO entra en el plugin `exo`, pero SÍ hay
  que repuntarlo — referencia `process:orchestrate` y `reflex:executor`, que
  mueren con A1.
- **La ventana G2→G4, decidida en la spec:** el rewiring de `distill`/`document`
  a verbos `exo` **se mueve entero a G4**. En esta ola siguen llamando a `kbx`.
  No los medio-arregles aquí.
- **Fuera de scope:** portar comandos de kbx (es G4), la KB semilla (G3),
  CI/release/instalador (G5).
- **Working dir:** `C:\proyectos\homework\exo`. Shell: Git Bash.
- **El exit code no es evidencia.** Cada verificación de este plan mira un
  artefacto: un fichero, un log, una ruta resuelta. Nunca solo un `$?`.
- **PRECONDICIÓN DURA: la ola 1A tiene que estar completa y commiteada.** La
  spec declara G1 y G2 paralelos y lógicamente lo son, pero **comparten el
  directorio `plugins/reflex/scripts/`**: 1A lo edita (contrato v2 del envelope
  y fin del literal `kb-demo`) y este plan lo mueve entero con `git mv`. Si
  se ejecutan en paralelo, el `git mv` se lleva la versión sin migrar y el
  conflicto no da error: da scripts que hablan v1 en una ruta nueva.
  Verifícalo antes de empezar:

```bash
cd /c/proyectos/homework/exo
grep -rn 'kb-demo' plugins/reflex/scripts/*.sh | grep -v 'test-'
grep -rn 'data\.notas\|has("notas")\|data\.truncado' plugins/reflex/scripts/*.sh
```

  Expected: **cero líneas en ambos**. Si sale algo, la ola 1A no está terminada
  y este plan no debe arrancar.

---

### Task 1: Baseline falsable ANTES de tocar nada

Dos cosas que hay que saber ciertas antes del rename, porque después no se
podrá distinguir «esto ya estaba roto» de «lo rompí yo».

**Files:**
- Create: `docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md` (sección «Baseline»)

**Interfaces:**
- Consumes: nada.
- Produces: el runbook de cutover, que las Tasks 4 y 8 amplían.

- [ ] **Step 1: ¿El hook `Stop` reindexa de verdad en esta máquina?**

Es el item 3 de la ola 0 de la spec: el fix de portabilidad está instalado
(reflex 0.17.0) pero **no hay evidencia de que se haya ejecutado nunca**.

```bash
stat -c '%y %n' ~/.exo/index.db
grep -c '"event":"index"' ~/.claude/reflex-log.jsonl 2>/dev/null || echo 0
tail -3 ~/.claude/reflex-log.jsonl
```

Anota el mtime y el conteo. Luego **cierra una sesión de Claude Code de
verdad** en este repo y repite:

```bash
stat -c '%y %n' ~/.exo/index.db
grep -c '"event":"index"' ~/.claude/reflex-log.jsonl
```

Expected: mtime posterior **y** conteo mayor. Si el mtime no se mueve, el hook
no reindexa y eso es un hallazgo que hay que anotar en el runbook **antes** de
seguir — no lo arregles aquí, pero déjalo escrito.

- [ ] **Step 2: Inventariar qué plugins están instalados y en qué versión**

```bash
ls -d ~/.claude/plugins/cache/exo/*/*/ 2>/dev/null
cat ~/.claude/plugins/installed_plugins.json 2>/dev/null | jq . 2>/dev/null | head -40
```

Anota la salida literal en el runbook: es el estado al que hay que poder volver.

- [ ] **Step 3: Comprobar que el gate de la KB funciona HOY**

```bash
cat /c/proyectos/homework/kb-demo/.git/hooks/pre-commit
ls -d "$HOME"/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh
```

Expected: el shim existe **y** el glob resuelve a un fichero real. Si el
segundo comando no devuelve nada, el gate ya está abierto y hay que decirlo
antes de que el rename se lleve la culpa.

- [ ] **Step 4: Crear el runbook con la baseline**

Crear `docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md` con la
salida literal de los tres pasos anteriores bajo una sección `## Baseline
(antes del cutover)`, y una sección `## Rollback` vacía que la Task 8 rellena.

- [ ] **Step 5: Commit**

```bash
cd /c/proyectos/homework/exo
git add docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md
git commit -m "docs(runbook): baseline falsable antes del cutover del plugin exo"
```

---

### Task 2: Fusionar los dos plugins en `plugins/exo/` conservando historia

**Files:**
- Create: `plugins/exo/.claude-plugin/plugin.json`
- Create: `plugins/exo/README.md`
- Modify (por `git mv`): todo `plugins/process/**` y `plugins/reflex/**`
- Delete: `plugins/process/`, `plugins/reflex/` (vacíos tras el `mv`)

**Interfaces:**
- Consumes: nada.
- Produces, para las Tasks 3-8: el árbol `plugins/exo/` con
  `skills/`, `agents/`, `hooks/hooks.json`, `scripts/`, `LICENSES/`.

- [ ] **Step 1: Mover con `git mv`, no con `cp`**

`git mv` conserva la detección de renames y con ella el `git log --follow` de
cada script. Copiar y borrar la pierde.

```bash
cd /c/proyectos/homework/exo
mkdir -p plugins/exo
git mv plugins/process/skills plugins/exo/skills
git mv plugins/process/LICENSES plugins/exo/LICENSES
git mv plugins/reflex/agents plugins/exo/agents
git mv plugins/reflex/hooks plugins/exo/hooks
git mv plugins/reflex/scripts plugins/exo/scripts
git mv plugins/reflex/skills/consolida plugins/exo/skills/consolida
git mv plugins/reflex/skills/recon-first plugins/exo/skills/recon-first
git status --short | head -40
```

- [ ] **Step 2: Verificar que no queda nada suelto**

```bash
cd /c/proyectos/homework/exo
find plugins/process plugins/reflex -type f 2>/dev/null
```

Expected: solo los dos `.claude-plugin/plugin.json` y los dos `README.md`
viejos, que se borran en el paso siguiente. Cualquier otra cosa es un fichero
que el `git mv` se dejó.

- [ ] **Step 3: Crear el manifest del plugin nuevo**

Crear `plugins/exo/.claude-plugin/plugin.json`:

```json
{
  "name": "exo",
  "description": "Framework de trabajo agéntico con memoria persistente. Nueve skills de proceso (brainstorm · plan · orchestrate · tdd · debug · verify · document · distill · recon-first), el agente de rol `executor` con la doctrina en su system prompt, y la capa de reflejos: guardrails deterministas en PreToolUse/SessionStart/UserPromptSubmit/SubagentStart/Stop que activan conocimiento procedural en el punto de acción. El recall lo sirve el engine `exo` desde SQLite. Abstención por defecto. Destila el catálogo de obra/superpowers (MIT, © 2025 Jesse Vincent) más doctrina propia.",
  "version": "1.0.0",
  "author": {
    "name": "Paul Guerrero",
    "email": "pguerrerolinares@gmail.com"
  },
  "keywords": ["exo", "memory", "hooks", "guardrails", "orchestration", "skills", "reflex"]
}
```

- [ ] **Step 4: Borrar los manifests y READMEs viejos, escribir el nuevo**

```bash
cd /c/proyectos/homework/exo
git rm -r plugins/process plugins/reflex
```

Crear `plugins/exo/README.md` fusionando los dos anteriores. Debe cubrir: los
nueve skills con una línea cada uno, el agente `executor`, la tabla de hooks
con su evento y su fichero, y la sección de atribución a superpowers que ya
existía en `plugins/process/README.md` (**no la pierdas**: es la obligación
de la licencia MIT).

- [ ] **Step 5: Verificar que los hooks siguen resolviendo**

`hooks.json` usa `"${CLAUDE_PLUGIN_ROOT}"/scripts/x.sh`, que es relativo al
plugin, así que el contenido **no cambia**. Compruébalo:

```bash
cd /c/proyectos/homework/exo
jq -r '.hooks | to_entries[] | .value[] | .hooks[] | .command' plugins/exo/hooks/hooks.json \
  | sed 's|"${CLAUDE_PLUGIN_ROOT}"/||' | while read -r f; do
      [ -f "plugins/exo/$f" ] && echo "OK   $f" || echo "FALTA $f"
    done
```

Expected: **nueve líneas `OK`, cero `FALTA`**.

- [ ] **Step 6: Correr las suites de script en su ubicación nueva**

```bash
cd /c/proyectos/homework/exo
for t in plugins/exo/scripts/test-*.sh; do
  printf "%s: " "$(basename "$t")"
  bash "$t" >/tmp/$(basename "$t").log 2>&1 && echo OK || { echo FAIL; tail -15 /tmp/$(basename "$t").log; }
done
```

Expected: todas `OK`. Si alguna falla por una ruta que asumía `reflex/` en el
path, arréglala aquí.

- [ ] **Step 7: Commit**

```bash
cd /c/proyectos/homework/exo
git add -A plugins/
git commit -m "refactor(plugins): fusionar process y reflex en el plugin único exo 1.0.0"
```

---

### Task 3: Renombrar `documenta`→`document` y `consolida`→`distill`

**Files:**
- Modify (por `git mv`): `plugins/exo/skills/documenta/` → `plugins/exo/skills/document/`
- Modify (por `git mv`): `plugins/exo/skills/consolida/` → `plugins/exo/skills/distill/`
- Modify: los dos `SKILL.md` (frontmatter `name`)
- Modify: `plugins/exo/scripts/documenta-remind.sh` y su nombre de fichero
- Modify: `plugins/exo/hooks/hooks.json` (la ruta del script renombrado)
- Modify: todo fichero que nombre los skills viejos

**Interfaces:**
- Consumes: el árbol `plugins/exo/` de la Task 2.
- Produces: los nueve skills con nombre definitivo, invocables como
  `exo:document` y `exo:distill`.

- [ ] **Step 1: Inventariar todas las referencias antes de mover**

```bash
cd /c/proyectos/homework/exo
grep -rn '\bdocumenta\b\|\bconsolida\b\|/documenta\|/consolida' plugins/ docs/ README.md \
  | grep -v '^docs/superpowers/consultas/' | grep -v '^docs/superpowers/plans/2026-0[1-8]' \
  > /tmp/refs-rename.txt
wc -l /tmp/refs-rename.txt
awk -F: '{print $1}' /tmp/refs-rename.txt | sort | uniq -c | sort -rn
```

Guarda `/tmp/refs-rename.txt`: es la lista de la compra. Los documentos
históricos (`consultas/`, planes viejos) **no se tocan** — son audit trail y
hablan del pasado.

- [ ] **Step 2: Mover los directorios**

```bash
cd /c/proyectos/homework/exo
git mv plugins/exo/skills/documenta plugins/exo/skills/document
git mv plugins/exo/skills/consolida plugins/exo/skills/distill
git mv plugins/exo/scripts/documenta-remind.sh plugins/exo/scripts/document-remind.sh
```

- [ ] **Step 3: Actualizar el frontmatter de los dos SKILL.md**

En `plugins/exo/skills/document/SKILL.md`, línea 2:

```yaml
name: document
```

En `plugins/exo/skills/distill/SKILL.md`, línea 2:

```yaml
name: distill
```

Y en la `description` de `distill`, sustituir «Consolidación offline de la KB
kb-demo» por «Consolidación offline de la KB» —el nombre de la KB de Paul
no pinta nada en un plugin que va a ser público—. Actualiza también sus
triggers entrecomillados si citan `/consolida`.

- [ ] **Step 4: Actualizar la ruta del script en `hooks.json`**

En `plugins/exo/hooks/hooks.json`, en el bloque `Stop`:

```json
            "command": "\"${CLAUDE_PLUGIN_ROOT}\"/scripts/document-remind.sh"
```

- [ ] **Step 5: Barrer el resto de referencias**

Recorre `/tmp/refs-rename.txt` fichero a fichero. Los sitios conocidos:

- `plugins/exo/scripts/document-remind.sh` — su propio texto de recordatorio
- `plugins/exo/scripts/exo-index.sh`, `exo-recall.sh`, `git-add-all-guard.sh`,
  `verify-before-commit.sh`, `test-subagent-inject.sh`
- `plugins/exo/skills/debug/SKILL.md`, `debug/techniques.md`,
  `orchestrate/SKILL.md`
- `plugins/exo/skills/document/routing.md`
- `plugins/exo/README.md`
- `README.md` de la raíz del repo

En cada uno, `/documenta`→`/document`, `/consolida`→`/distill`,
`process:documenta`→`exo:document`, `reflex:consolida`→`exo:distill`. **Lee el
contexto de cada match**: «documentación» y «documentar» no son el skill.

- [ ] **Step 6: Verificar que no queda ninguna referencia viva**

```bash
cd /c/proyectos/homework/exo
grep -rn 'process:\|reflex:\|/documenta\b\|/consolida\b' plugins/ README.md
```

Expected: **cero líneas**. Y:

```bash
ls plugins/exo/skills/
```

Expected exactamente: `brainstorm  debug  distill  document  orchestrate  plan  recon-first  tdd  verify`

- [ ] **Step 7: Los tests de script siguen verdes**

```bash
cd /c/proyectos/homework/exo
for t in plugins/exo/scripts/test-*.sh; do
  printf "%s: " "$(basename "$t")"
  bash "$t" >/tmp/r-$(basename "$t").log 2>&1 && echo OK || { echo FAIL; tail -15 /tmp/r-$(basename "$t").log; }
done
```

Expected: todas `OK`.

- [ ] **Step 8: Commit**

```bash
cd /c/proyectos/homework/exo
git add -A plugins/ README.md
git commit -m "refactor(plugins)!: documenta->document, consolida->distill

BREAKING CHANGE: los skills cambian de nombre y de prefijo. Invocación nueva:
exo:document, exo:distill. Referencias externas (KB, paul-profile, shim del
pre-commit) se repuntan en los commits siguientes."
```

---

### Task 4: El shim del pre-commit de la KB — el gate que se abre en silencio

**El punto más peligroso de toda la ola.** `kb-demo/.git/hooks/pre-commit`
resuelve `$HOME/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh`.
Con el plugin renombrado a `exo`, el glob no matchea y su rama de fallo hace
`echo … >&2; exit 0`: **el commit pasa y el gate no existe**. No vive en ningún
repo del monorepo, así que ningún `grep` del monorepo lo encuentra.

**Files:**
- Modify: `C:\proyectos\homework\kb-demo\.git\hooks\pre-commit` (fuera de este repo)
- Modify: `plugins/exo/scripts/kb-precommit.sh` (comentarios que citan la ruta vieja)
- Modify: `docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md`

**Interfaces:**
- Consumes: el plugin renombrado de las Tasks 2-3.
- Produces: el gate de la KB apuntando al plugin nuevo.

- [ ] **Step 1: Demostrar el fallo antes de arreglarlo**

```bash
ls -d "$HOME"/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh 2>/dev/null; echo "GLOB_VIEJO_EXIT=$?"
ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh 2>/dev/null;    echo "GLOB_NUEVO_EXIT=$?"
```

Hoy el viejo resuelve y el nuevo no. Tras el cutover será al revés — y el shim
sin tocar se iría por la rama `exit 0`. Anota ambas salidas en el runbook.

- [ ] **Step 2: Reescribir el shim para que falle RUIDOSO, no permisivo**

El shim actual deja pasar el commit cuando no encuentra el script, con el
argumento de que «un plugin ausente no debe convertirse en un repo bloqueado».
Ese criterio es el que convierte un rename en un gate apagado. Sustituir el
contenido de `C:\proyectos\homework\kb-demo\.git\hooks\pre-commit` por:

```bash
#!/usr/bin/env bash
# Gate de la KB: trinquete de techos + presupuestos.
# La lógica vive en el plugin `exo`; esto solo lo localiza y le cede el paso.
#
# Shim en vez de `ln -sf`: en esta máquina core.symlinks=false, así que `ln -s`
# deja una COPIA y el hook se congelaría en la versión del plugin del día del
# cableado. La ruta se resuelve en cada commit y se queda con la versión más
# alta instalada, para que una actualización del marketplace llegue sola.
#
# 2026-08-26: el plugin pasó de `reflex` a `exo`. El glob viejo se conserva
# como fallback SOLO durante el cutover; retirar cuando las dos máquinas estén
# migradas.
set -uo pipefail

script="$(ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh 2>/dev/null | sort -V | tail -1)"
if [ -z "$script" ]; then
  script="$(ls -d "$HOME"/.claude/plugins/cache/exo/reflex/*/scripts/kb-precommit.sh 2>/dev/null | sort -V | tail -1)"
  [ -n "$script" ] && echo "pre-commit: usando el plugin reflex (viejo) — migra a exo" >&2
fi

if [ -z "$script" ]; then
  # ANTES este caso dejaba pasar el commit. Se cambió el 2026-08-26: un gate
  # que se apaga solo cuando cambia un nombre de directorio es peor que un
  # repo bloqueado, porque no avisa. Si de verdad hace falta commitear sin
  # gate, `git commit --no-verify` lo dice en voz alta.
  echo "pre-commit: NO encuentro kb-precommit.sh de ningún plugin (exo ni reflex)." >&2
  echo "pre-commit: el gate de la KB NO ha corrido. Instala el plugin, o usa --no-verify a sabiendas." >&2
  exit 1
fi

exec bash "$script" "$@"
```

- [ ] **Step 3: Verificar el shim en sus tres estados**

```bash
HOOK=/c/proyectos/homework/kb-demo/.git/hooks/pre-commit
# a) con el plugin viejo instalado (estado actual): debe resolver y avisar
bash "$HOOK" </dev/null; echo "EXIT_A=$?"
```

Expected: corre el gate (posiblemente fallando por falta de `kbx`, que es otro
asunto) y avisa por stderr de que usa el plugin viejo.

```bash
# b) sin ningún plugin: debe salir 1 y decirlo
HOME=/tmp/home-vacio bash "$HOOK" </dev/null; echo "EXIT_B=$?"
```

Expected: `EXIT_B=1` y dos líneas en stderr. **Este es el test que importa**:
antes daba 0.

- [ ] **Step 4: Actualizar los comentarios de `kb-precommit.sh`**

En `plugins/exo/scripts/kb-precommit.sh`, actualizar la cabecera y cualquier
comentario que cite `plugin reflex` o la ruta `cache/exo/reflex/`. **No toques
las llamadas a `kbx`**: eso es G4.

- [ ] **Step 5: Documentar en el runbook**

Añadir al runbook de la Task 1 una sección `## El shim del pre-commit` con: el
fallo demostrado en el Step 1, el cambio de criterio (permisivo → ruidoso) y
su razón, y la nota de que el fallback a `reflex` se retira cuando las dos
máquinas estén migradas.

- [ ] **Step 6: Commit**

```bash
cd /c/proyectos/homework/exo
git add plugins/exo/scripts/kb-precommit.sh docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md
git commit -m "fix(kb-gate): el shim del pre-commit apunta a exo y falla ruidoso si no lo encuentra"
```

El hook de `kb-demo` vive en `.git/hooks/`, que no se versiona: queda
anotado en el runbook y hay que repetirlo a mano en la otra máquina (Task 8).

---

### Task 5: Repuntar `paul-profile` y el marketplace

**Files:**
- Modify: `C:\proyectos\homework\exo-plugins\.claude-plugin\marketplace.json`
- Modify: `C:\proyectos\homework\exo-plugins\plugins\paul-profile\.claude-plugin\plugin.json`
- Modify: `C:\proyectos\homework\exo-plugins\plugins\paul-profile\skills\fabrica\SKILL.md`

**Interfaces:**
- Consumes: el plugin `exo` de las Tasks 2-3.
- Produces: marketplace que sirve `exo@exo` y ya no sirve `process` ni `reflex`.

- [ ] **Step 1: Inventariar las referencias de paul-profile**

```bash
cd /c/proyectos/homework/exo-plugins
grep -rn 'process:\|reflex:\|process@\|reflex@' . --include=*.json --include=*.md | grep -v '^./.git/'
```

Expected según la adjudicación: `plugin.json:3` y
`skills/fabrica/SKILL.md:8,43,61`, más las entradas del marketplace.

- [ ] **Step 2: Repuntar paul-profile**

En cada match: `process:orchestrate`→`exo:orchestrate`,
`reflex:executor`→`exo:executor`. **No** metas `paul-profile` dentro del
plugin `exo` (A2): sigue siendo un plugin aparte, solo cambia a quién llama.

- [ ] **Step 3: Reemplazar las dos entradas del marketplace por una**

En `.claude-plugin/marketplace.json`, borrar los objetos `process` y `reflex` y
poner en su lugar:

```json
    {
      "name": "exo",
      "source": {
        "source": "git-subdir",
        "url": "https://github.com/pguerrerolinares/exo.git",
        "path": "plugins/exo"
      },
      "description": "Framework de trabajo agéntico con memoria persistente: nueve skills de proceso, el agente executor y la capa de reflejos (hooks deterministas). Sustituye a los plugins process y reflex.",
      "version": "1.0.0",
      "author": {
        "name": "Paul Guerrero",
        "email": "pguerrerolinares@gmail.com"
      }
    },
```

Subir `metadata.version` del marketplace de `0.2.0` a `0.3.0`.

- [ ] **Step 4: Validar el JSON**

```bash
cd /c/proyectos/homework/exo-plugins
jq -e '.plugins | map(.name) | sort' .claude-plugin/marketplace.json
```

Expected: `["exo","paul-profile","workflow-lint"]`. Si sale `process` o
`reflex`, no se han retirado.

- [ ] **Step 5: Commit en el repo del marketplace**

```bash
cd /c/proyectos/homework/exo-plugins
git add -A
git commit -m "feat(marketplace): servir el plugin exo 1.0.0, retirar process y reflex"
```

---

### Task 6: Repuntar la KB — `core-index` y los skills que la citan

**Files:**
- Modify: `C:\proyectos\homework\kb-demo\core\core-index.md`
- Modify: los `.md` de la KB que citen los skills viejos

**Interfaces:**
- Consumes: los nombres nuevos de la Task 3.
- Produces: la KB coherente con el plugin. **Importa de verdad**: `core-index`
  se inyecta en cada arranque de sesión, así que una línea rancia ahí es
  doctrina falsa servida en cada sesión.

- [ ] **Step 1: Inventariar**

```bash
cd /c/proyectos/homework/kb-demo
grep -rn 'plugin `process`\|process:\|reflex:\|/documenta\|/consolida' \
  --include=*.md . | grep -v '^./archive/' | grep -v '^./log/'
```

`archive/` y `log/` son bitácora e histórico: hablan del pasado y **no se
tocan**. Solo el canon vivo.

- [ ] **Step 2: Actualizar la línea de routing de `core-index.md`**

La línea actual empieza por «ROUTING DE PROCESO (plugin `process`)». Sustituir
por:

```markdown
- ROUTING DE PROCESO (plugin `exo`): brainstorm (diseño antes de código) · plan (spec→plan) · orchestrate (ejecutar plan multi-tarea) · tdd (test primero) · debug (bug o atasco) · verify (antes de declarar hecho) · document (cierre de sesión) · distill (consolidación offline). Si una aplica — aunque sea al 1% — invócala ANTES de responder o actuar, incluidas las preguntas aclaratorias; no lo racionalices. Subagente ejecutando una tarea concreta: exento.
```

- [ ] **Step 3: Verificar el presupuesto de `core-index` (no lo revientes)**

`core-index` tiene techo sellado y se inyecta en un bloque de 6.144 B. Añadir
`· distill (consolidación offline)` lo engorda.

```bash
cd /c/proyectos/homework/kb-demo
wc -c core/core-index.md
grep -n 'kbx_budget_max' core/core-index.md
```

Compara el tamaño con el techo sellado. Si lo rebasa: **no subas el techo y no
mutiles la nota**. Retira una entrada muerta del índice, que es lo que la
doctrina permite para un índice. Si nada cabe, deja el cambio pendiente y dilo.

- [ ] **Step 4: Barrer el resto del canon**

Los demás matches del Step 1, con el mismo criterio de la Task 3: lee el
contexto, «documentación» no es el skill.

- [ ] **Step 5: Commit en la KB (con el gate corriendo)**

```bash
cd /c/proyectos/homework/kb-demo
git add core/core-index.md
git status --short
git commit -m "docs(core-index): routing al plugin exo, document y distill"
```

Expected: el pre-commit de la Task 4 corre y **se ve** en la salida. Si commitea
sin decir nada, el shim no está haciendo su trabajo — vuelve a la Task 4.

---

### Task 7: Actualizar los docs del monorepo

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md`
- Modify: `docs/backlog.md`

**Interfaces:**
- Consumes: todo lo anterior.
- Produces: nada de código.

- [ ] **Step 1: `README.md`**

La sección «## Capa thin: el plugin `process`» describe siete skills y un
plugin que ya no existe. Reescribirla como «## Capa thin: el plugin `exo`» con
los nueve skills, el agente y los hooks, y el id de plugin `exo@exo`. Conserva
íntegra la sección «## Atribución» —es obligación de la licencia MIT—.

- [ ] **Step 2: El runbook de W11**

Sus comandos de instalación instalan `process@exo` y `reflex@exo`, y sus
verificaciones usan rutas `cache/exo/reflex/*`. Añadir al final una sección
`## Actualización 2026-08-26: plugin único exo` con los comandos nuevos, en vez
de reescribir el cuerpo —es audit trail de aquella jornada—.

- [ ] **Step 3: El backlog**

En `docs/backlog.md`, sección Baja, el item «Nombres y ubicaciones» menciona la
incoherencia de nombres. Anota que G2 la resuelve para los plugins y que quedan
vivas `docs/superpowers/` y `reports/` (que son de G5).

- [ ] **Step 4: Commit**

```bash
cd /c/proyectos/homework/exo
git add README.md docs/
git commit -m "docs: actualizar README, runbook de W11 y backlog al plugin exo"
```

---

### Task 8: Cutover en las máquinas y verificación end-to-end

**Files:**
- Modify: `docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md` (secciones «Cutover» y «Rollback»)

**Interfaces:**
- Consumes: todo lo anterior, empujado a `origin/main` de ambos repos.
- Produces: las dos máquinas corriendo el plugin `exo`.

- [ ] **Step 1: Empujar los dos repos**

```bash
cd /c/proyectos/homework/exo && git push
cd /c/proyectos/homework/exo-plugins && git push
```

El plugin se sirve por `git-subdir` desde GitHub: **hasta que esto no esté
empujado, ninguna máquina ve el plugin nuevo**. Es la trampa documentada del
runbook del 24 de agosto.

- [ ] **Step 2: Cutover en esta máquina**

```bash
claude plugin marketplace update exo
claude plugin install exo@exo
claude plugin uninstall process@exo
claude plugin uninstall reflex@exo
ls -d ~/.claude/plugins/cache/exo/*/*/
```

Expected: aparece `exo/1.0.0/` y **desaparecen** `process/` y `reflex/`. Mira
el directorio, no el mensaje del comando.

- [ ] **Step 3: Verificación falsable de los cinco hooks**

```bash
P=~/.claude/plugins/cache/exo/exo/1.0.0/scripts

echo '{"session_id":"t","source":"startup"}' | bash "$P/exo-recall.sh" \
  | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
```

Expected: `1`. Un `0` es el fallback embebido — el arranque mentiría.

```bash
echo '{"prompt":"cómo funciona el recall de exo"}' | bash "$P/recall-inject.sh" | jq -e '.' >/dev/null && echo "recall-inject OK"
echo '{"agent_type":"general-purpose"}' | bash "$P/subagent-inject.sh" | jq -e '.' >/dev/null && echo "subagent-inject OK"
grep recall-fallback ~/.claude/reflex-log.jsonl | tail -3
```

Expected: las dos líneas `OK`, y ninguna entrada nueva de `recall-fallback` con
`reason=no-engine` / `no-index` / `no-contract`.

- [ ] **Step 4: Verificación del gate de la KB con el plugin nuevo**

```bash
ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh
cd /c/proyectos/homework/kb-demo
touch core/core-index.md && git add core/core-index.md
git commit -m "test: verificar el gate tras el cutover" --dry-run 2>&1 | head -5
git reset
```

Expected: el `ls` resuelve al script del plugin nuevo, y el shim ya no emite el
aviso de «usando el plugin reflex (viejo)».

- [ ] **Step 5: Verificar los nueve skills en una sesión real**

Abre una sesión de Claude Code en este repo y comprueba que el listado de
skills disponibles muestra los nueve con prefijo `exo:`, y que no aparece
ninguno con `process:` ni `reflex:`. **Esto no se puede verificar por script**:
es el harness quien resuelve el catálogo. Anota la salida en el runbook.

- [ ] **Step 6: Retirar el fallback del shim**

Cuando **las dos** máquinas estén migradas, borrar del shim de
`kb-demo/.git/hooks/pre-commit` el bloque de fallback a `reflex` que la
Task 4 dejó, y dejar solo el glob de `exo`. Anótalo en el runbook con la fecha.

- [ ] **Step 7: Escribir la sección Rollback del runbook**

Cómo volver: `claude plugin install reflex@exo` y `process@exo` desde el commit
anterior del marketplace, y restaurar el shim viejo. Con los comandos exactos.
Un cutover sin rollback escrito no es un cutover, es una apuesta.

- [ ] **Step 8: Commit**

```bash
cd /c/proyectos/homework/exo
git add docs/superpowers/runbooks/2026-08-26-cutover-plugin-exo.md
git commit -m "docs(runbook): cutover del plugin exo verificado end-to-end"
```

---

## Lo que este plan NO arregla, a propósito

- **`distill` sigue sin funcionar en Windows.** Llama a `kbx`, que aquí no
  compila. Es G4 y la spec lo decide así explícitamente: shippear el rewiring
  a medias sería peor. El skill debe decirlo en voz alta cuando no encuentra
  el binario, no callarse.
- **Las 13 rutas `/home/paul/…` de `distill/SKILL.md`** se parametrizan contra
  la config en G4, cuando se reescriban sus comandos a verbos `exo`.
- **`schema_drift`, `budget_prose_drift` y el resto del linter de la KB** son
  G4.
