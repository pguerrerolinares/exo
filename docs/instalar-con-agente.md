# Instalar exo con un agente

> **Para quién:** el agente (Claude Code) al que el usuario le dice *«clona
> `<url-del-repo>` y sigue su `docs/instalar-con-agente.md`»*. Es un runbook:
> síguelo en orden y no improvises pasos. Un humano puede seguirlo igual.
>
> `<url-del-repo>` es la URL desde la que el usuario te pide clonar (en KBI,
> el espejo de `gitlab.norbolsa.com`). El plugin se instala desde ese mismo
> repo; el binario sale de las releases públicas de GitHub, en la versión del
> último tag del clon, y se verifica por SHA256. Este runbook da la secuencia;
> los flags, las variables y el porqué de cada paso están en
> [`instalacion.md`](instalacion.md).

## Reglas para el agente

- **No compiles.** Nada de Rust ni Build Tools: hay binario precompilado.
- **Comandos largos en background o con `timeout: 600000`.** `exo init`
  descarga el modelo de embeddings (~0,6 GB, ~6 min en frío).
- **No desinstales nada.** superpowers se *desactiva* (en Windows lo hace el
  instalador, en Linux el paso 4); no lo cambies por `uninstall`.
- **Un `exit 0` no es evidencia.** Cada paso tiene su check. Si uno falla,
  busca el síntoma en [Problemas conocidos en Windows 11](instalacion.md#8-problemas-conocidos-en-windows-11)
  antes de improvisar.
- **Edita `settings.json` solo con los comandos de esta guía**, nunca
  reescribiéndolo a mano.

## 0. Clonar y comprobar

```bash
git clone <url-del-repo> ~/.exo/repo
git --version && claude --version && uname -s && uname -m
git config --global user.email
jq --version
```

- `uname -s` debe empezar por `MINGW64` (Windows) o ser `Linux`, y en Linux
  `uname -m` debe ser `x86_64`. En macOS o en Linux ARM no hay binario: para
  y remite a `instalacion.md` §3.
- **Sin `user.email`, para**: `exo init` commitea la KB y sin identidad el
  commit falla en silencio. Pide al usuario que la configure.
- **`jq`**: en Linux el instalador no lo trae. Si `jq --version` falla, pide
  al usuario que lo instale (`sudo apt install jq` o el equivalente de su
  distro) antes de seguir; sin él los hooks se abstienen. En Windows lo deja
  el instalador del paso 2.
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

## 2. Binario (`exo`)

La versión del binario es la del último tag del clon, no la última de
GitHub. Ojo: eso fija el binario, no el plugin (paso 4).

**Windows:**

```bash
EXO_VERSION="$(git -C ~/.exo/repo describe --tags --abbrev=0 --match 'v[0-9]*')" powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w ~/.exo/repo/install.ps1)"
```

Deja `~/.local/bin/exo.exe` (SHA256 verificado), un `jq.exe` real si no lo
había, el directorio en el PATH de usuario y superpowers desactivado.

**Linux:**

```bash
EXO_VERSION="$(git -C ~/.exo/repo describe --tags --abbrev=0 --match 'v[0-9]*')" bash ~/.exo/repo/install.sh
```

Deja `~/.local/bin/exo` (SHA256 verificado) y nada más: ni instala `jq`, ni
toca el PATH (solo avisa si `~/.local/bin` no está), ni desactiva
superpowers.

En las dos plataformas el instalador cierra con un `exo doctor` con filas
`fail`: normal, aún no hay KB ni plugin.

Check: `~/.local/bin/exo --version`.

**El PATH nuevo no llega a esta sesión.** Hasta el reinicio del paso 5,
antepón `PATH="$HOME/.local/bin:$PATH"` a cada comando que use `exo` o `jq`.

## 3. KB

Si el equipo dejó la caché del modelo en un recurso compartido, cópiala antes
a `$HF_HOME/hub/` si esa variable está definida, y si no a
`~/.cache/huggingface/hub/`, y te ahorras la descarga. Luego, en background
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

**Binario y plugin no avanzan juntos.** El binario está fijado al último tag
del clon; el plugin sigue `main` y, con el `autoUpdate` de abajo, avanza solo.
Si un día el plugin exige un binario más nuevo (`ENGINE_MIN`), lo avisan
`exo doctor` (`plugin_compat`) y el hook de arranque; se resuelve con
«Actualizar», abajo.

**Actualizaciones automáticas.** Un marketplace añadido así no tiene
`autoUpdate`. Fusión no destructiva:

```bash
export PATH="$HOME/.local/bin:$PATH"; S=~/.claude/settings.json; [ -f "$S" ] || echo '{}' > "$S"
src="$(jq -c '.exo.source' ~/.claude/plugins/known_marketplaces.json)"
jq --argjson src "$src" '.extraKnownMarketplaces.exo = ((.extraKnownMarketplaces.exo // {}) + {source: $src, autoUpdate: true})' "$S" > "$S.tmp" && mv "$S.tmp" "$S"
```

**superpowers en Linux** (en Windows ya lo hizo el instalador). Busca la
clave y desactívala en el scope de usuario:

```bash
jq -r '.enabledPlugins // {} | keys[] | select(startswith("superpowers@"))' ~/.claude/settings.json
claude plugin disable <clave-que-salga> --scope user
```

Si no sale ninguna clave, no hay nada que desactivar.

**Solo cuentas Team/Enterprise** (si no, Claude Code avisa en cada turno de
que `prompt.compose` se salta): pide al usuario que ponga a `false` la opción
«Registrar el hook prompt.compose» del plugin en `/config`. La alternativa
`echo '{"composeHook":"false"}' | claude plugin configure exo@exo --values-stdin`
no está verificada (no se sabe si Claude Code convierte `"false"` en
booleano); si la usas, comprueba después que el aviso desaparece. Detalle en
[`instalacion.md` §5](instalacion.md#5-instalar-el-plugin-de-claude-code-capa-thin).

**Gate de pre-commit de la KB** (`exo init` no lo instala). En cada commit
corre los techos de bytes (`budget` y `ratchet`), `rules --check` y la
validación de `tier` y `tags` del frontmatter. Shim que delega en el script
del plugin y sigue sus actualizaciones (en Windows `ln -s` copia en vez de
enlazar). **El `chmod +x` es obligatorio**: sin él git ignora el hook, el
commit pasa sin gate (`exo doctor` lo detecta desde la versión posterior a
0.13.0; hasta 0.13.0 da `ok`).

```bash
cat > "<ruta-kb>/.git/hooks/pre-commit" <<'EOF'
#!/usr/bin/env bash
# Shim del gate de la KB: delega en el kb-precommit.sh del plugin exo instalado.
s="$(ls -d "$HOME"/.claude/plugins/cache/exo/exo/*/scripts/kb-precommit.sh 2>/dev/null | sort -V | tail -1)"
[ -n "$s" ] || { echo "kb-precommit: no encuentro el plugin exo; commit BLOQUEADO (escape: git commit --no-verify)" >&2; exit 1; }
exec bash "$s" "$@"
EOF
chmod +x "<ruta-kb>/.git/hooks/pre-commit"
test -x "<ruta-kb>/.git/hooks/pre-commit" && echo "gate ejecutable"
```

Si `exo doctor` (engine hasta 0.13.0) sugiere en cambio
`ln -sf …/kb-precommit.sh`, ignóralo y usa este shim: es el que esta guía
mantiene y el que proponen las versiones posteriores.

**Check del arranque, ya en esta sesión** (detecta un modelo o un índice
roto antes de reiniciar):

```bash
export PATH="$HOME/.local/bin:$PATH"
echo '{"session_id":"t","source":"startup"}' | bash "$(ls -d ~/.claude/plugins/cache/exo/exo/*/ | sort -V | tail -1)scripts/exo-recall.sh" | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
```

`1` = el arranque sirve la KB real. `0` = mira la última línea de
`~/.claude/reflex-log.jsonl` con `recall-fallback` y su `reason`:

| `reason` | Qué pasó |
|---|---|
| `no-engine` | No hay binario `exo` ejecutable |
| `engine-stale` | El binario es más viejo que `ENGINE_MIN` del plugin: actualízalo |
| `no-index` | No existe el índice (`~/.exo/index.db`, o `$EXO_INDEX`) |
| `no-config` | `exo config` falló: sin config no hay nombre de KB; el payload trae el error |
| `truncated` | El engine avisó de que el bloque de arranque no cupo entero |
| `empty` | `exo recall` no devolvió nada |
| `no-contract` | Devolvió algo, pero no la nota con el «Contrato de memoria» |

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
`kb_readable`, `index_db`, `embeddings_model`, `jq`, `kb_precommit_hook`,
`plugin_compat` y `superpowers_disabled`; más `git_bash` en Windows (en
Linux sale `na`, es lo esperado).

`kb_precommit_hook ok` no prueba que el hook sea ejecutable: confirma con
`test -x "<ruta-kb>/.git/hooks/pre-commit"`.

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
