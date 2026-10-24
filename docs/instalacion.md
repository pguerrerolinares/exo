# Instalación de exo

> Dos caminos: **desde release** (binario precompilado, sin Rust ni toolchain
> C — el recomendado) y **desde fuente**. Este documento describe los dos y
> declara al final lo que sigue sin existir.

## 1. Requisitos

| Camino | Requisitos |
|---|---|
| **Desde fuente** | Rust estable (ver mínimo abajo) + toolchain C. En Windows, MSVC. |
| **Primera indexación** | Red para descargar el modelo de embeddings (~0,6 GB); en frío tarda unos minutos. |
| **Capa thin (plugin de Claude Code)** | `git` y `jq` ejecutables desde bash. En Windows, Git Bash (Claude Code lo usa para los hooks; no hacen falta wrappers `.cmd`). |
| **Desde release** (recomendado) | `git` y `jq`. **Ni Rust ni toolchain C.** |

- **Toolchain C obligatorio**: `rusqlite` (SQLite bundled) y `sqlite-vec`
  compilan C durante el build. Sin compilador C, `cargo build` falla — está
  medido: en una máquina con target GNU sin `gcc.exe`, `cargo check` muere en
  `cc-rs: failed to find tool "gcc.exe"`.
- **Windows, con detalle ganado a pulso**: instala las Visual Studio Build
  Tools **con `--includeRecommended`**. El workload `VCTools` sin ese flag
  **no trae el compilador** (MSVC es componente *recomendado*, no requerido), y
  `winget` devuelve éxito igualmente — reproducido tres veces, documentado en
  `docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md`.
- **Versión mínima de Rust**: `rust-version = "1.95"`, declarada en
  `engine/Cargo.toml` y verificada empíricamente con el lockfile actual
  (1.94 falla — `libsqlite3-sys` usa `cfg_select`, estable desde 1.95 —
  y 1.95 compila el crate con todos sus targets).
- **Precondición: la KB debe ser la raíz de un repo git.** No todos los
  subcomandos la exigen igual, y sin ella cada uno se comporta distinto:
  `targets` falla con un mensaje que nombra la condición y el remedio
  (`git init`; `exo::gitx::es_repo_git`); `stale` falla con el error crudo
  de git (necesita el último commit de cada nota); `ratchet` se abstiene
  con exit 0 (no hay historia contra la que medir); `budget`, `lint` y
  `rotate` no miran git y funcionan igual con o sin él.

## 2. Instalar desde release (recomendado)

```bash
curl -fsSL https://raw.githubusercontent.com/pguerrerolinares/exo/main/install.sh | bash
```

En Windows, desde PowerShell:

```powershell
irm https://raw.githubusercontent.com/pguerrerolinares/exo/main/install.ps1 | iex
```

Los dos hacen lo mismo: detectan la plataforma, bajan el binario de la última
release, **verifican su SHA256 antes de copiar nada**, lo dejan en
`~/.local/bin/exo` (`exo.exe` en Windows) y cierran corriendo `exo doctor`.

**Por qué `~/.local/bin` y no otro sitio del PATH:** es donde caen los dos
instaladores, y como `~/.local/bin` suele estar en el `PATH`, el pre-commit
de la KB (`plugins/exo/scripts/kb-precommit.sh:20-21`) lo resuelve por
`command -v exo` — mismo orden que usan los hooks — antes de mirar el
literal `$HOME/.local/bin/exo(.exe)` como fallback. Si ninguno de los dos
resuelve un `exo` ejecutable, ese hook sale 1 y BLOQUEA el commit
(fail-closed) — el escape consciente es `git commit --no-verify`. `exo
doctor` tiene un check dedicado al literal (`hook_fallback_binary`).

`install.ps1` además deja un `jq.exe` oficial (jq 1.7.1, SHA256 verificado)
junto a `exo.exe` si no hay ya un jq real en el PATH, y antepone el
directorio al PATH de **usuario** (el alias de WindowsApps no es un jq).
El SHA256 de jq 1.7.1 va fijado en el script. El PATH de usuario se reescribe
conservando los `%VAR%` (REG_EXPAND_SZ).
Si superpowers está habilitado en el scope de usuario, lo desactiva
(`claude plugin disable <clave> --scope user`, nunca uninstall) y te dice cómo
revertirlo (`claude plugin enable <clave>`). Opt-out: `-KeepSuperpowers` o
`EXO_DISABLE_SUPERPOWERS=0` (el entorno es lo que llega por `irm | iex`). Sin
`claude` en el PATH, imprime el comando y sigue.

Variables reconocidas: `EXO_DIR` (destino), `EXO_VERSION` (un tag concreto en
vez de `latest`), `EXO_INIT_KB` + `EXO_INIT_NAME` (encadenan `exo init`; sin
ellas el instalador imprime el comando, no inventa una KB), `EXO_FORCE=1`
(`install.sh` reemplaza un wrapper de build con `cuda`, §3; sin ella aborta
sin tocarlo).

### Verificar la instalación

```bash
exo doctor
```

Trece checks de entorno; cada uno dice **el artefacto que miró**. `warn`
informa, `fail` sale con código 3. Con `--json` emite el envelope v2.

## 3. Compilar el engine

```bash
git clone https://github.com/pguerrerolinares/exo
cd exo/engine
cargo build --release
# binario resultante: target/release/exo  (exo.exe en Windows)
```

Colócalo en el `PATH`. Los hooks del plugin lo resuelven con
`command -v exo` y, si no está en `PATH`, caen al literal
`$HOME/.local/bin/exo` — en Windows/Git Bash ese literal sin extensión
falla el test de ejecutable, así que allí lo fiable es copiar `exo.exe` a un
directorio que esté en `PATH`:

```bash
mkdir -p ~/.local/bin
cp target/release/exo ~/.local/bin/       # Linux (o macOS, desde fuente)
cp target/release/exo.exe ~/.local/bin/   # Windows (Git Bash); ~/.local/bin en PATH
```

### Embeddings en GPU (opcional, NVIDIA)

Con la feature `cuda`, `exo index` y `exo rebuild` embeben en la GPU.
`exo search` y el refresco previo a un recall siguen en CPU a propósito:
corren en hooks, embeben pocos textos y arrancar CUDA les costaría más de lo
que ahorra. Las releases no la llevan.

Requisitos en el sistema: el runtime CUDA (cudart, cuBLAS, cuFFT, cuRAND) y
cuDNN 9 de la misma rama mayor que el binario de onnxruntime que baja ort-sys
(12 o 13; se elige con `ORT_CUDA_VERSION`, y por defecto es 12). En WSL2 el
driver es el de Windows: instala solo librerías (repo apt `wsl-ubuntu` de
NVIDIA, nunca `cuda`, `cuda-drivers` ni `nvidia-driver-*`). cuDNN no está en
ese repo: tarball del redist de NVIDIA a `/opt` y su `lib/` en `ld.so.conf.d`.

Medido el 2026-10-15 (RTX 4090, WSL2, CUDA 13.2, cuDNN 9.27): `exo rebuild`
de 4394 trozos pasa de 9 min 11 s en CPU (16 hilos) a 23 s. Los resultados
de `search` coinciden (40 queries: mismo top-1, |Δscore| ≤ 7e-7).

```bash
ORT_CUDA_VERSION=13 cargo build --release --features cuda
# El provider CUDA es una .so aparte que onnxruntime busca en el directorio
# de argv[0]. Invocado como `exo` desde el PATH, eso es el cwd y no lo
# encuentra: binario y .so a un directorio propio, y en el PATH un wrapper
# que lo lanza por ruta absoluta.
P=$(dirname "$(find ~/.cache/ort.pyke.io -name libonnxruntime_providers_cuda.so | head -1)")
mkdir -p ~/.local/lib/exo
cp target/release/exo "$P"/libonnxruntime_providers_{shared,cuda}.so ~/.local/lib/exo/
printf '#!/bin/sh\nexec "$HOME/.local/lib/exo/exo" "$@"\n' > ~/.local/bin/exo
chmod +x ~/.local/bin/exo
```

Si CUDA no arranca, el indexado sigue en CPU, lo dice en stderr y lo deja
registrado: `exo doctor` lo muestra en el check `embeddings_device`.

Para actualizar, recompila el tag nuevo y copia solo el binario a
`~/.local/lib/exo/`. `install.sh` ve el wrapper y aborta: copiar la release
encima dejaría la máquina indexando en CPU sin aviso.

## 4. Crear (o adoptar) una KB

El engine arranca con `~/.exo/config.toml`; sin config no hay defaults
inventados, solo un error que nombra el comando que la crea:

```bash
# KB nueva: vuelca la semilla de engine/kb-template/, la versiona con git
# (best-effort) y la indexa
exo init --kb ~/mi-kb --name mi-kb

# KB ya existente gestionada por basic-memory: adopción de una sola vez,
# sin tocar un byte dentro de la KB
exo init --from-basic-memory
```

Una DB sirve a una sola KB: si el índice de destino (`$EXO_DB` o el default
de config) ya tiene guardada la ruta de otra KB en disco, `exo init` (y
`exo index`/`exo rebuild`) lo rechazan antes de tocar nada. El remedio que
sugiere el error depende del comando, porque `exo init` no tiene `--db`
(resuelve por `$EXO_DB`):

```
# exo index / exo rebuild (tienen --db)
error: este índice es de otra KB que sigue en disco: <ruta previa> (pediste <ruta nueva>). Una DB sirve a UNA KB: usa otra --db para esta, o `exo rebuild --kb <kb> --db <esta db>` si de verdad quieres reemplazar el índice

# exo init (no tiene --db)
error: este índice es de otra KB que sigue en disco: <ruta previa> (pediste <ruta nueva>). Una DB sirve a UNA KB: usa otro $EXO_DB para esta KB (`EXO_DB=<ruta> exo init …`), o borra/reemplaza la DB actual si de verdad quieres reutilizarla
```

`exo search` y `exo recall` (solo lectura) no rechazan nada, pero avisan por
stderr con el mismo criterio si la DB que resuelven trae la ruta de otra KB
que sigue en disco: una config con `[index] db` mal apuntado (o un `$EXO_DB`
suelto) responde igual, pero deja de hacerlo en silencio. La KB contra la
que se compara es la KB ACTIVA de cada comando — `$EXO_KB` > `[kb] path` de
la config en `exo search` (no tiene `--kb`); `--kb` > `$EXO_KB` > config en
`exo recall` — nunca una lectura aparte del disco.

Una segunda KB en la misma máquina necesita, además, su propio fichero de
**config** — `exo init` no tiene flag `--db`, así que la forma de indexar
esta segunda KB en su propia DB (y no en la de la primera) es `$EXO_DB`:

```bash
# 1. Config Y db propios para la segunda KB. `escribe_config` graba en
#    `[index] db` la DB efectiva de ESTE init ($EXO_DB), no el default.
EXO_CONFIG=~/.exo/otra-kb.toml EXO_DB=~/.exo/otra-kb.db \
  exo init --kb ~/otra-kb --name otra-kb

# 2. De aquí en adelante, EXO_CONFIG basta — ya no hace falta EXO_DB.
EXO_CONFIG=~/.exo/otra-kb.toml exo search "…"
EXO_CONFIG=~/.exo/otra-kb.toml exo index
```

Una config sirve a una KB; para varias, una config (con su propio
`EXO_DB` en el `init` que la crea) por KB.

La primera indexación descarga el modelo de embeddings
(`jinaai/jina-embeddings-v2-base-es`, ~0,6 GB, pineado a una revisión
concreta de HuggingFace) a la caché local. En frío son varios minutos
(~6 medidos en la máquina de referencia); las corridas siguientes no
vuelven a pagarlo.

**La caché respeta `$HF_HOME`** (Task 13, G, 2026-09-16): si la variable
está definida, el modelo se busca y se descarga bajo `$HF_HOME/hub`; si no,
bajo `~/.cache/huggingface/hub` (el default de siempre). `exo doctor` mira
la misma ruta que usa el engine para descargar, así que su check
`embeddings_model` no puede quedarse mirando un directorio distinto. Si ya
tenías el modelo cacheado en el default y ahora defines `HF_HOME` por
primera vez, la próxima indexación vuelve a pagar la descarga completa una
vez, porque busca en la ruta nueva. Dos efectos que vienen de leer el
entorno como lo hace `hf-hub` y conviene conocer: el token de HuggingFace se
busca junto a la caché (`$HF_HOME/token` en vez de
`~/.cache/huggingface/token`), y si defines **`$HF_ENDPOINT`** el modelo se
descarga de esa URL en vez de la de HuggingFace — antes esa variable se
ignoraba.

Comprobación rápida:

```bash
exo config --json    # config efectiva con rutas expandidas
exo search "doctrina" --type hybrid --min-similarity 0.40 --limit 5  # hybrid (el default) carga el modelo de embeddings; --type fts es el modo barato
exo recall --limit 5
```

## 5. Instalar el plugin de Claude Code (capa thin)

El repo es su propio marketplace (`.claude-plugin/marketplace.json` sirve
`plugins/exo/`, id `exo@exo`):

```bash
claude plugin marketplace add pguerrerolinares/exo
claude plugin install exo@exo
```

Los hooks del plugin necesitan el binario ya instalado (sección 3) y `jq`.
Si el engine no está, el plugin no rompe la sesión: degrada a fallbacks
embebidos y lo deja anotado en `~/.claude/reflex-log.jsonl`. El desfase
binario↔plugin (un binario más viejo que lo que el plugin instalado declara
necesitar) sí tiene check dedicado en `exo doctor` (`plugin_compat`,
campaña H) y en el hook `exo-recall.sh` (SessionStart) — ver
`docs/backlog.md`.

**Versiones.** El engine y el plugin versionan por separado: `exo --version`
es la del binario (= tag de la release); el plugin lleva la suya en
`plugins/exo/.claude-plugin/plugin.json`. `scripts/test-versiones.sh` impide
que los ficheros se contradigan.

## 6. Correr los tests

```bash
cd engine
cargo test            # suite completa
scripts/test-hermetico.sh   # gate: la suite entera sin ~/.exo/config.toml
```

Dos avisos honestos, ambos anotados en `docs/backlog.md`:

- Las suites que indexan contenido necesitan el modelo de embeddings en la
  caché local de HuggingFace. En una máquina sin esa caché, la primera
  corrida lo descarga (~0,6 GB); en un runner sin red ni caché, esas suites
  fallan. El gate de hermeticidad cubre la config, no esta segunda
  dependencia.
- Dos tests van marcados `#[ignore]` precisamente por dependencias de
  entorno (descarga del modelo, índice real); se corren explícitos con
  `--ignored`.

## 7. Lo que NO hay todavía

- **Binario para macOS (retirado el 2026-10-04) ni para Linux ARM.** Solo se
  publican `x86_64-unknown-linux-gnu` y `x86_64-pc-windows-msvc`; en macOS o
  en un Linux aarch64 toca compilar
  desde fuente. Los instaladores lo detectan y abortan diciéndolo, en vez de
  dejar un binario que no arranca.
- **`exo diff-since` y `exo history`.** No se portan por decisión: se usan
  `git diff`/`git log` directamente (ver skill `distill`).

## 8. Problemas conocidos en Windows 11

Encontrados en máquinas reales. El camino guiado está en
[`instalar-con-agente.md`](instalar-con-agente.md).

| Síntoma | Causa | Remedio |
|---|---|---|
| Los hooks no hacen nada, sin error | `jq` es el alias de la Microsoft Store (`WindowsApps\jq.exe`): Git Bash no lo resuelve y sin `jq` los hooks se abstienen | `install.ps1` deja un `jq.exe` real delante en el PATH. Check: `exo doctor` (fila `jq`) |
| `exo`/`jq` "command not found" tras instalar | La sesión de Claude Code arrancó antes del cambio de PATH | Cerrar Claude Code del todo y abrir sesión nueva |
| Hooks `.sh` no arrancan o corren en WSL | El `bash` del PATH es `System32\bash.exe` (WSL), no Git Bash | Git for Windows antes en el PATH, o `CLAUDE_CODE_GIT_BASH_PATH` a `Git\bin\bash.exe`. Check: `exo doctor` (fila `git_bash`) |
| `exo init` falla al descargar el modelo | `exo` (ureq + rustls) solo ve el proxy por `HTTPS_PROXY`/`ALL_PROXY`, no el de Windows, y confía solo en `webpki-roots`, no en la CA corporativa | `HTTPS_PROXY` definido; o `HF_ENDPOINT` a un espejo interno; o copiar `~/.cache/huggingface/hub/models--jinaai--jina-embeddings-v2-base-es` de otra máquina (respeta `HF_HOME`) |
| `exo init` muere a los 2 min | Timeout por defecto del Bash tool; la descarga en frío tarda ~6 min | Background o `timeout: 600000` |
| `exo init`: «existe y no está vacía» | Por diseño: no pisa notas | Carpeta nueva o vacía; adoptar una KB existente solo se soporta desde basic-memory |
| La KB queda sin commits | `exo init` commitea best-effort y sin `user.email` falla en silencio | `git config --global user.email …` y commitear a mano |
| `database is locked` | Dos indexados a la vez (`init` y el hook `Stop`) | Esperar a que termine uno; `ps -W` lista el binario como `exo`, sin `.exe` |
| Commit de la KB bloqueado | El pre-commit no encuentra `exo` o el plugin (fail-closed) | Revisar PATH / plugin; escape consciente: `git commit --no-verify` |
| `exo budget` o el pre-commit: «sin tier» / «falta tags» | Nota creada con `exo write new` sin `--tier` o sin `tags` en el frontmatter del `--from` | Añadir `tier` y `tags` al frontmatter |
| Aviso «prompt.compose hook was skipped» en cada turno | Cuenta Team/Enterprise: el guard `cc-plugin-sec-default` lo intercepta | `echo '{"composeHook":"false"}' \| claude plugin configure exo@exo --values-stdin` |
| Skills o hooks duplicados | superpowers sigue habilitado | `claude plugin disable superpowers@<marketplace> --scope user`. Check: `exo doctor` (fila `superpowers_disabled`) |
| Cada prompt tarda ~1,5 s más | Coste de los hooks bajo Git Bash (≈20 procesos por hook; W11 p50 ≈ 1,7 s) | Esperado |
| PowerShell: `&&` no es válido | Windows PowerShell 5.1 no lo admite (sí `pwsh` 7) | Ejecutar desde Git Bash, o encadenar con `;` |
| KB con cambios perdidos o `.git` corrupto | KB en una carpeta sincronizada por OneDrive | Mover la KB fuera (`C:\proyectos\…`) |
