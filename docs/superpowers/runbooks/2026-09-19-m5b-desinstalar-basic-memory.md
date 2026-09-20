# 2026-09-19 — M5b: desinstalar basic-memory (checklist C10)

> Checklist de cierre de `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md`
> §Campaña 10 (M5b). Decisión de Paul del 2026-09-19 (#6,
> `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §7): *"sí,
> tras el checklist C10. La fábrica prepara y corre en seco las
> comprobaciones automatizables (este runbook, campaña L); la desinstalación
> la ejecuta Paul (línea roja)."* Incluye también la decisión #15 de la
> misma sesión (slug canónico), que cierra el punto #8 del gate M4
> (`docs/backlog.md`, ítem «Barrer los hallazgos vivos del gate M4»,
> ~`backlog:857-861`).
>
> **Qué NO hace este runbook:** no desinstala basic-memory, no borra
> `~/.basic-memory/`, no toca `~/.claude.json`. Todo eso es de Paul,
> ejecutado a mano, después de leer las comprobaciones en seco de abajo y
> decidir que el checklist está satisfecho.

## Checklist C10 (del plan de cierre, con el estado real de hoy)

1. El hook de recall no llama a basic-memory por ninguna vía (ni en el
   FALLBACK). → **Comprobación A** abajo.
2. Ningún hook conserva matchers `mcp__basic-memory__*` vivos. →
   **Comprobación D** abajo.
3. ~~kbx apunta al índice del engine y `consumed` está actualizado.~~
   **CADUCADO**: kbx dejó de ser dependencia de nada desde la campaña D
   (`docs/backlog.md`, "cutover kbx→exo", commits `0051638`/`aa82f95`;
   ver también `docs/backlog.md:1143`, "kbx dejó de ser dependencia de
   nada"). No hay nada que verificar aquí.
4. `/document` y `/distill` corren end-to-end sin el MCP (nombres
   actuales de las skills que el plan de cierre llamaba `/documenta` y
   `/consolida`). → **Comprobación C** abajo.
5. exo tiene config propia (M5a-02). → **YA CERRADO** (`docs/backlog.md`,
   ítem "M5a-02 config propia: cerrado el 2026-08-26" — `~/.exo/config.toml`,
   precedencia `flag > env > config > error accionable`, sin fallback a
   basic-memory salvo `exo init --from-basic-memory`, explícito).
6. La KB está commiteada y pusheada (el plan de cierre dice "en kb-demo";
   hoy el nombre real es `wisdom-paul`). → **Comprobación E** abajo.
   **Estado real al ejecutar esta comprobación (2026-09-20): NO
   satisfecho** — hay un commit sin pushear. Ver Comprobación E.

## Comprobación A — el hook de recall no llama a basic-memory (ni en fallback)

```bash
cd /home/paul/Documentos/proyectos/exo
git grep -in "mcp__basic" -- plugins/ engine/
echo "exit=$?"
```

Output real (verificado 2026-09-20): sin líneas, `exit=1` (grep sin
matches). Ninguna mención viva de un tool MCP de basic-memory en plugin ni
engine.

## Comprobación B — la única lectura de basic-memory que sobrevive es la explícita

```bash
cd /home/paul/Documentos/proyectos/exo
grep -rn "basic-memory/config.json" engine/src/ | grep -v inicia.rs
```

Output real (verificado 2026-09-20, dos líneas, ambas COMENTARIOS de
diseño, no código que se ejecute):
```
engine/src/config.rs:3://! Sustituye la lectura RO de `~/.basic-memory/config.json` que hacían
engine/src/main.rs:129:    /// Toma raíz, nombre y embeddings de `~/.basic-memory/config.json`.
```
Las dos son doc-comments (`//!`/`///`) que EXPLICAN el diseño (uno describe
qué sustituyó `config.rs`, el otro documenta el flag `--from-basic-memory`
de `exo init`) — ninguna es una llamada real. La única lectura de código que
toca esa ruta sigue siendo `engine/src/inicia.rs`, exclusiva de
`exo init --from-basic-memory` (explícita, opt-in, nunca automática).

## Comprobación C — `/document` y `/distill` no invocan tools MCP de basic-memory

```bash
cd /home/paul/Documentos/proyectos/exo
grep -in "mcp__basic\|basic-memory" \
  plugins/exo/skills/document/SKILL.md \
  plugins/exo/skills/distill/SKILL.md \
  plugins/exo/skills/distill/chequeos.md
echo "exit=$?"
```

Output real (verificado 2026-09-20): sin líneas, `exit=1`. Ninguna de las
tres skills menciona basic-memory ni sus tools MCP — las dos ya migraron
por completo al engine `exo` (`exo write`/`exo search`/`exo recall`, vía el
binario, no vía MCP).

## Comprobación D — `hooks.json` sin matchers `mcp__basic-memory__*`

```bash
cd /home/paul/Documentos/proyectos/exo
grep -n "basic-memory\|mcp__basic" plugins/exo/hooks/hooks.json
echo "exit=$?"
```

Output real (verificado 2026-09-20): sin líneas, `exit=1`. Los diez hooks
de producción (`jq '[.hooks[]?[]?.hooks[]?] | length'` → `10`, verificado
hoy) citan solo scripts de `${CLAUDE_PLUGIN_ROOT}/scripts/`, ninguno un
matcher de tool MCP.

## Comprobación E — la KB está commiteada y pusheada (snapshot de hoy)

```bash
git -C /home/paul/Documentos/proyectos/wisdom-paul status --porcelain
git -C /home/paul/Documentos/proyectos/wisdom-paul rev-parse --abbrev-ref --symbolic-full-name @{u}
git -C /home/paul/Documentos/proyectos/wisdom-paul log @{u}..HEAD --oneline
```

Output real (verificado 2026-09-20): `status --porcelain` vacío (árbol
limpio), upstream `origin/main`, **`log @{u}..HEAD` NO vacío**:
```
6bf57d5 docs(kb): documenta frente de tasa de re-explicación de exo
```
Hay **un commit sin pushear** en `wisdom-paul`. Esto difiere del snapshot
citado en el plan (2026-09-19, donde salía vacío) — confirma que este dato
cambia día a día y que el checklist C10 punto 6 **NO está satisfecho ahora
mismo**. **Paul: repite esta comprobación justo antes de desinstalar** y
haz `git -C /home/paul/Documentos/proyectos/wisdom-paul push` (o
equivalente) si vuelve a salir un commit pendiente — no es una garantía
permanente, es un snapshot del momento de ejecutar la comprobación.

## Decisión #15 (slug canónico) — cierra el #8 del gate M4

`docs/backlog.md`, ítem «Barrer los hallazgos vivos del gate M4»,
sub-ítem **#8 [baja]**: divergencia de slug medida **19/127** entre el
generador de permalinks de `exo` y el de basic-memory (`_` conservado en
bitácoras rotadas, CamelCase separado, `§`→`ss`). Decisión de Paul,
2026-09-19 (#15, `propuesta.md` §7): **el slug de exo es canónico** — la
divergencia 19/127 con basic-memory queda **aceptada por escrito**, no se
persigue paridad. Con esto el #8 del gate M4 queda cerrado por esta misma
decisión — marcado en `docs/backlog.md`, sub-ítem **#8 [baja]** del ítem
«Barrer los hallazgos vivos del gate M4» (la Task 6 de campaña L barrió
otros cinco ítems de bookkeeping, no este).

## Pasos manuales de Paul — desinstalación

Ejecutar SOLO cuando las cinco comprobaciones de arriba estén en el estado
esperado el día de la desinstalación (repetirlas, no fiarse de este
runbook si pasó tiempo — hoy mismo, 2026-09-20, la Comprobación E ya dio
distinto que el 2026-09-19):

1. **Quitar el servidor MCP** de la config de Claude Code:
   `claude mcp remove basic-memory` (o edición manual de `~/.claude.json`).
   Medido hoy (2026-09-20, solo lectura): `mcpServers.basic-memory` no
   lleva ninguna clave `disabled` (es la config de conexión `stdio` sin
   más) — lo que existe de verdad es
   `projects["/home/paul/Documentos/proyectos/exo"].disabledMcpServers`,
   un array que incluye `"basic-memory"` (junto con otros servidores
   desactivados para ese proyecto). O sea: desactivado para el proyecto
   `exo` vía esa lista, pero sigue registrado globalmente en
   `mcpServers`.
2. **Confirmar que nada más lo referencia**: `claude mcp list` no debe
   listar `basic-memory` tras el paso 1.
3. **Liberar el caché/índice de basic-memory** (~1 GB medido hoy en
   `~/.basic-memory/`, independiente de la KB en sí):
   `rm -rf ~/.basic-memory` (revisar antes con `du -sh ~/.basic-memory` que
   la cifra sigue siendo del orden de lo esperado, no un false-negative de
   este runbook).
4. **Limpiar el residuo vacío**: `rmdir ~/basic-memory` si sigue vacío
   (`ls -la ~/basic-memory` antes de borrar, por si alguna vez se pobló).
5. **NO tocar** `/home/paul/Documentos/proyectos/wisdom-paul` — esa es la
   KB, vive fuera de basic-memory y la sigue usando `exo`.
6. **Verificación post-desinstalación**: `claude mcp list` sin
   `basic-memory`; sesión nueva de Claude Code en el repo `exo` arranca
   igual (el hook `exo-recall.sh` no lo usa, ya lo confirmó la
   Comprobación A); `exo doctor` sigue en verde (no depende de
   basic-memory).

## Rollback

Si algo falla tras el paso 3 (p. ej. se necesita releer una nota vieja que
solo basic-memory tenía indexada — no debería pasar, la KB en markdown
sigue intacta en `wisdom-paul/`, pero por si el índice de basic-memory
guardaba algo que el markdown no):

1. Los datos NUNCA vivieron solo en `~/.basic-memory/` — es un índice
   derivado de `wisdom-paul/` (markdown, en git). Nada se pierde al borrar
   el índice.
2. Para volver a tener el MCP disponible: `uvx basic-memory mcp` sigue
   funcionando sin reinstalar nada persistente (`uvx` descarga/cachea bajo
   `uv` en caliente); solo hace falta re-añadirlo con
   `claude mcp add basic-memory -- uvx basic-memory mcp` (o restaurar la
   entrada en `~/.claude.json`) y dejar que regenere su índice desde
   `wisdom-paul/` (`basic-memory sync` o equivalente, fuera de alcance de
   `exo`).
3. Si se borró `~/.basic-memory/config.json` por error: recrearlo con
   `default_project: "wisdom-paul"` y el proyecto `wisdom-paul` apuntando a
   `path: "/home/paul/Documentos/proyectos/wisdom-paul"` (verificado hoy,
   2026-09-20, leyendo `~/.basic-memory/config.json` en vivo — claves
   `default_project` y `projects.wisdom-paul.path`) es suficiente para que
   vuelva a apuntar a `wisdom-paul/`.
