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

**Por qué `~/.local/bin` y no otro sitio del PATH:** es la ruta literal que
mira el pre-commit de la KB (`plugins/exo/scripts/kb-precommit.sh:18`). Si el
binario no está ahí, ese hook sale 0 —commit permitido, sin gate— y no rompe
nada al hacerlo. `exo doctor` tiene un check dedicado a eso
(`hook_fallback_binary`).

Variables reconocidas: `EXO_DIR` (destino), `EXO_VERSION` (un tag concreto en
vez de `latest`), `EXO_INIT_KB` + `EXO_INIT_NAME` (encadenan `exo init`).

### Verificar la instalación

```bash
exo doctor
```

Diez checks de entorno; cada uno dice **el artefacto que miró**. `warn`
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
cp target/release/exo ~/.local/bin/       # Linux / macOS
cp target/release/exo.exe ~/.local/bin/   # Windows (Git Bash); ~/.local/bin en PATH
```

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
`exo index`) lo rechazan antes de tocar nada:

```
error: este índice es de otra KB que sigue en disco: <ruta previa> (pediste <ruta nueva>). Una DB sirve a UNA KB: usa otra --db para esta, o `exo rebuild --kb <kb> --db <esta db>` si de verdad quieres reemplazar el índice
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

Comprobación rápida:

```bash
exo config --json    # config efectiva con rutas expandidas
exo search "doctrina" --type hybrid --min-similarity 0.40 --limit 5
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
embebidos y lo deja anotado en `~/.claude/reflex-log.jsonl`. Ese silencio
tiene su deuda: el check de desfase binario↔plugin sigue sin existir en
`exo doctor` — ver `docs/backlog.md`.

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

- **Binario para macOS Intel, ni para Linux ARM.** Solo se publican
  `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` y
  `aarch64-apple-darwin`; en un Mac Intel o en un Linux aarch64 toca compilar
  desde fuente. Los instaladores lo detectan y abortan diciéndolo, en vez de
  dejar un binario que no arranca.
- **`exo rotate` y `exo stale`.** Siguen viviendo en `kbx` (Go). El remedio
  que la doctrina manda aplicar cuando el gate de presupuestos muerde
  —rotar la bitácora— exige por tanto `kbx` instalado. `exo:distill` lo
  detecta y lo dice en una línea visible en vez de callarse.
- **`exo diff-since` y `exo history`.** No portados y sin fecha.
