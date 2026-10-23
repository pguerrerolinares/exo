# Instalar exo con un agente

> **Para quién:** el agente (Claude Code) al que el usuario le dice *«clona
> `<url-del-repo>` y sigue su `docs/instalar-con-agente.md`»*. Es un runbook:
> síguelo en orden y no improvises pasos. Un humano puede seguirlo igual.
>
> `<url-del-repo>` es la URL desde la que el usuario te pide clonar (en KBI,
> el espejo de `gitlab.norbolsa.com`). El plugin se instala desde ese mismo
> repo; el binario sale de las releases públicas de GitHub, en la versión del
> último tag del clon, y se verifica por SHA256. Referencia completa: [`instalacion.md`](instalacion.md).

## Reglas para el agente

- **No compiles.** Nada de Rust ni Build Tools: hay binario precompilado.
- **Comandos largos en background o con `timeout: 600000`.** `exo init`
  descarga el modelo de embeddings (~0,6 GB, ~6 min en frío).
- **No desinstales nada.** El instalador *desactiva* superpowers; no lo
  cambies por `uninstall`.
- **Un `exit 0` no es evidencia.** Cada paso tiene su check. Si uno falla,
  busca el síntoma en [Problemas conocidos en Windows 11](instalacion.md#8-problemas-conocidos-en-windows-11)
  antes de improvisar.
- **Edita `settings.json` solo con los comandos de esta guía**, nunca
  reescribiéndolo a mano.

## 0. Clonar y comprobar

```bash
git clone <url-del-repo> ~/.exo/repo
git --version && claude --version && uname -s
git config --global user.email
```

- `uname -s` debe empezar por `MINGW64` (Windows) o ser `Linux`. En macOS o
  ARM no hay binario: para y remite a `instalacion.md` §3.
- **Sin `user.email`, para**: `exo init` commitea la KB y sin identidad el
  commit falla en silencio. Pide al usuario que la configure.
- `echo "${HTTPS_PROXY:-<vacío>}"`: el binario `exo` solo usa el proxy si
  está en esa variable (no el de Windows) y no confía en la CA corporativa.
  Si la red lo necesita y está vacía, avisa antes de seguir (en KBI:
  `kbi:doctor`).

## 1. Preguntas al usuario (todas juntas)

1. **Ruta de la KB.** Propón `C:\proyectos\kb` o `~/kb`. Tiene que no existir
   o estar vacía (`exo init` no vuelca sobre contenido), y no estar bajo
   `OneDrive`.
2. **Nombre de la KB**, corto y en kebab-case: `kb-<usuario>`.
3. **¿Cuenta Claude Team o Enterprise?** (paso 4).
4. **Aviso, no pregunta:** superpowers queda desactivado en todos sus
   proyectos. `exo:orchestrate` ocupa el sitio de
   `superpowers:subagent-driven-development`, y `exo:brainstorm`, `plan`,
   `tdd`, `debug` y `verify` los de sus homólogos. Revertir:
   `claude plugin enable <clave>`.

## 2. Binario (`exo` + `jq`)

La versión del binario es la del último tag del clon, no la última de
GitHub. Así binario y plugin salen del mismo punto del repo.

Windows:

```bash
EXO_VERSION="$(git -C ~/.exo/repo describe --tags --abbrev=0 --match 'v[0-9]*')" powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w ~/.exo/repo/install.ps1)"
```

Linux:

```bash
EXO_VERSION="$(git -C ~/.exo/repo describe --tags --abbrev=0 --match 'v[0-9]*')" bash ~/.exo/repo/install.sh
```

Deja `~/.local/bin/exo(.exe)` (SHA256 verificado), un `jq.exe` real si no lo
había, el directorio en el PATH de usuario, superpowers desactivado y un
`exo doctor` con filas `fail`: normal, aún no hay KB ni plugin.

Check: `~/.local/bin/exo --version`.

**El PATH nuevo no llega a esta sesión.** Hasta el reinicio del paso 5,
antepón `PATH="$HOME/.local/bin:$PATH"` a cada comando que use `exo` o `jq`.

## 3. KB

Si el equipo dejó la caché del modelo en un recurso compartido, cópiala antes
a `~/.cache/huggingface/hub/` y te ahorras la descarga. Luego, en background
o con `timeout: 600000`:

```bash
PATH="$HOME/.local/bin:$PATH" exo init --kb "<ruta-kb>" --name "<nombre>"
```

Crea la KB desde la plantilla, `git init` + primer commit, escribe
`~/.exo/config.toml` e indexa. No lances otro `exo index` mientras corre
(`database is locked`).

Check: `git -C "<ruta-kb>" log --oneline | head -1` muestra un commit.

## 4. Plugin

```bash
claude plugin marketplace add "$(git -C ~/.exo/repo remote get-url origin)"
claude plugin install exo@exo
```

Si `marketplace add` falla por política de la organización, **para** y
díselo al usuario: no busques otra vía.

**Actualizaciones automáticas.** Un marketplace añadido así no tiene
`autoUpdate`. Fusión no destructiva:

```bash
export PATH="$HOME/.local/bin:$PATH"; S=~/.claude/settings.json; [ -f "$S" ] || echo '{}' > "$S"
src="$(jq -c '.exo.source' ~/.claude/plugins/known_marketplaces.json)"
jq --argjson src "$src" '.extraKnownMarketplaces.exo = ((.extraKnownMarketplaces.exo // {}) + {source: $src, autoUpdate: true})' "$S" > "$S.tmp" && mv "$S.tmp" "$S"
```

**Solo cuentas Team/Enterprise** (si no, Claude Code avisa en cada turno de
que `prompt.compose` se salta):

```bash
echo '{"composeHook":"false"}' | claude plugin configure exo@exo --values-stdin
```

**Gate de pre-commit de la KB** (presupuestos y trinquete; `exo init` no lo
instala). Shim que delega en el script del plugin y sigue sus
actualizaciones (en Windows `ln -s` copia en vez de enlazar):

```bash
cat > "<ruta-kb>/.git/hooks/pre-commit" <<'EOF'
#!/usr/bin/env bash
# Shim del gate de la KB: delega en el kb-precommit.sh del plugin exo instalado.
s="$(ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh 2>/dev/null | sort -V | tail -1)"
[ -n "$s" ] || { echo "kb-precommit: no encuentro el plugin exo; commit BLOQUEADO (escape: git commit --no-verify)" >&2; exit 1; }
exec bash "$s" "$@"
EOF
```

**Check del arranque, ya en esta sesión** (detecta un modelo o un índice
roto antes de reiniciar):

```bash
export PATH="$HOME/.local/bin:$PATH"
echo '{"session_id":"t","source":"startup"}' | bash "$(ls -d ~/.claude/plugins/cache/exo/exo/*/ | sort -V | tail -1)scripts/exo-recall.sh" | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
```

`1` = el arranque sirve la KB real. `0` = mira la última línea de
`~/.claude/reflex-log.jsonl` con `recall-fallback` (`reason=no-engine`,
`engine-stale`, `no-index`, `no-contract`).

## 5. Reinicio (obligatorio)

Pide al usuario que **cierre Claude Code del todo** (todas las ventanas de la
app, o el proceso `claude` del terminal) y abra una sesión nueva: sin eso ni
el PATH ni los hooks del plugin están cargados, y los hooks fallan en
silencio. La sesión nueva no sabe nada de esta; dale la frase exacta:

> Sigue con el paso 6 de `~/.exo/repo/docs/instalar-con-agente.md` y después
> con `~/.exo/repo/docs/sembrar-kb.md`. KB en `<ruta-kb>`, repos en
> `<carpeta-repos>`.

Pregúntale antes por `<carpeta-repos>`, la carpeta donde tiene sus repos
(normalmente `C:\proyectos`), para ponerla en la frase.

## 6. Verificación (en la sesión nueva)

```bash
exo doctor
```

Sin filas `fail`. Tienen que salir `ok`: `config`, `binary_on_path`,
`kb_readable`, `index_db`, `embeddings_model`, `git_bash`,
`kb_precommit_hook`, `plugin_compat` y `superpowers_disabled`.

```bash
exo search "doctrina" --type fts --limit 3
```

Devuelve notas de la KB. Con eso, la instalación está hecha: sigue con
[`sembrar-kb.md`](sembrar-kb.md).

## Actualizar

- **Plugin:** con `autoUpdate` llega al arrancar Claude Code; a mano,
  `claude plugin marketplace update exo`, `claude plugin update exo@exo` y
  reiniciar.
- **Binario:** `git -C ~/.exo/repo pull --tags` y repetir el paso 2. Si el plugin
  exige un binario más nuevo, lo dicen `exo doctor` (`plugin_compat`) y el
  aviso de arranque.
