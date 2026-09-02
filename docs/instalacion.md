# Instalación de exo

> Hoy el único camino real es **compilar desde fuente**. No hay releases
> publicadas, ni binarios precompilados, ni `install.sh`, ni CI: el camino
> "desde release" está diseñado (spec de exo genérico, sección G5) pero no
> existe todavía. Este documento describe lo que hay.

## 1. Requisitos

| Camino | Requisitos |
|---|---|
| **Desde fuente** (el único disponible hoy) | Rust estable (ver mínimo abajo) + toolchain C. En Windows, MSVC. |
| **Primera indexación** | Red para descargar el modelo de embeddings (~0,6 GB); en frío tarda unos minutos. |
| **Capa thin (plugin de Claude Code)** | `git` y `jq` ejecutables desde bash. En Windows, Git Bash (Claude Code lo usa para los hooks; no hacen falta wrappers `.cmd`). |
| **Desde release** (planeado) | Cuando exista: `git` y `jq`, sin Rust ni toolchain C. |

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

## 2. Compilar el engine

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

## 3. Crear (o adoptar) una KB

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

## 4. Instalar el plugin de Claude Code (capa thin)

El repo es su propio marketplace (`.claude-plugin/marketplace.json` sirve
`plugins/exo/`, id `exo@exo`):

```bash
claude plugin marketplace add pguerrerolinares/exo
claude plugin install exo@exo
```

Los hooks del plugin necesitan el binario ya instalado (sección 2) y `jq`.
Si el engine no está, el plugin no rompe la sesión: degrada a fallbacks
embebidos y lo deja anotado en `~/.claude/reflex-log.jsonl`. Ese silencio
tiene su deuda: el check de desfase binario↔plugin (`exo doctor`) está
planeado, no implementado — ver `docs/backlog.md`.

## 5. Correr los tests

```bash
cd engine
cargo test            # suite completa: 200 tests en 28 binarios
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

## 6. Lo que NO hay todavía

Para que nadie lo busque: no hay GitHub Releases, ni `install.sh` /
`install.ps1`, ni workflow de CI, ni `exo doctor` (el preflight de entorno) ni
`exo budget`. Todo eso está diseñado en la sección G5 de
`docs/superpowers/specs/2026-08-26-exo-generico-design.md` y pendiente de
ejecutar; el estado real de la deuda vive en `docs/backlog.md`.
