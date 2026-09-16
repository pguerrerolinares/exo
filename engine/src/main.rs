use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use exo::{
    buscador::{busca, busca_hybrid, busca_vector},
    envelope,
    escritor::{NuevaNota, escribe_append, escribe_nueva},
    indexer::indexa,
    recall::{recall_arranque, recall_consulta, renderiza, resuelve_rutas_absolutas},
};
use std::path::{Path, PathBuf};

/// Defaults SELLADOS del arm hybrid (M2-07, §5.2.6 de la spec de fusión):
/// ganadores del sweep 15+1 corridas (grid bonus{0,0.1,0.2,0.3,0.5}×
/// β{0.6,0.8,1.0} + diagnóstica A, `evals/e1-read/reports/m2-07-impl-report.md`) —
/// selección pre-registrada §5.2.4 (max hit@5=49/55 → 4 celdas empatadas en
/// β=0.6 → menor bonus=0.0), confirmada nativa (§5.2.5, `--min-similarity
/// 0.40` da 49/55 idéntico al post-hoc). Cubren SOLO el uso de `exo search
/// --type hybrid` sin `--bonus`/`--fts-scale` explícitos; el sweep siempre
/// pasó ambos flags, así que estos valores no afectaron su resultado. El
/// threshold ganador (0.40) NO se sella aquí como constante — D-f3/§4.6: el
/// valor difiere del 0.35 de config y config es RO hasta M5a, así que se
/// pasa por `--min-similarity 0.40` explícito en corridas/consumidores hasta
/// entonces (documentado en el verdict, no hardcodeado en el binario).
const BONUS_SELLADO: f64 = 0.0;
const ESCALA_FTS_SELLADA: f64 = 0.6;

#[derive(Parser)]
#[command(
    name = "exo",
    version,
    about = "Memoria persistente para agentes: indexa una KB de notas markdown y la sirve por búsqueda y recall."
)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Crea la config (`~/.exo/config.toml`) y una KB nueva desde la
    /// plantilla, ya indexada. Con `--from-basic-memory`, adopta una KB
    /// existente de basic-memory.
    Init(ArgsInit),
    /// Muestra la config efectiva, con las rutas ya expandidas.
    Config(ArgsConfig),
    /// Indexa la KB de forma incremental: solo lo que cambió desde la última
    /// vez.
    Index(ArgsIndex),
    /// Borra el índice y lo reconstruye desde cero. Es el remedio ante un
    /// índice corrupto.
    Rebuild(ArgsIndex),
    /// Busca en la KB: texto completo (`fts`), semántica (`vector`) o las dos
    /// fusionadas (`hybrid`).
    Search(ArgsSearch),
    /// Escribe en la KB: nota nueva o entrada de bitácora. No commitea ni
    /// indexa.
    #[command(subcommand)]
    Write(ComandoWrite),
    /// Sirve memoria de la KB a un agente. Sin `--query`, el bloque de
    /// arranque (notas `tier: core` y recientes); con `--query`, las notas
    /// relevantes para esa consulta.
    Recall(ArgsRecall),
    /// Lista las notas candidatas a tocar para un tema, con tier, tamaño,
    /// cabeceras y último commit. Solo lectura.
    Targets(ArgsTargets),
    /// Comprueba el presupuesto de bytes por tier. Imprime el informe entero y
    /// sale con 3 si alguna nota lo rebasa o no declara un tier válido.
    Budget(ArgsBudget),
    /// Comprueba la salud de la KB: notas huérfanas, frontmatter roto, índice
    /// desfasado y más. Imprime el informe entero y sale con 3 si hay
    /// hallazgos.
    Lint(ArgsLint),
    /// Comprueba que ningún techo de tamaño declarado suba respecto al último
    /// commit. Imprime el informe entero y sale con 3 si algún hallazgo
    /// rompe el trinquete (un techo que sube o se retira, un sobre-sello,
    /// una primera declaración muy alta, una nota sellada que se escapa de
    /// su tier, o una nota nueva sin aire o que nace demasiado grande); las
    /// deudas informativas no lo rompen. Sin historia de git se abstiene y
    /// sale con 0.
    Ratchet(ArgsRatchet),
    /// Divide una bitácora `tier: log` en frío (a `archive/log/`) y
    /// caliente (que se queda). Sin `--apply` es un dry-run: no toca disco.
    /// Solo barre el nivel superior de `log/` — igual que kbx, sin recursión.
    Rotate(ArgsRotate),
    /// Diagnostica esta máquina (binario, config, KB, índice, modelo de
    /// embeddings y dependencias de los hooks). Cada check dice qué artefacto
    /// miró; sale con 3 si alguno falla.
    Doctor(ArgsDoctor),
    /// Urgencia de actualización de cada nota: edad de su último commit,
    /// grado en el grafo de relaciones y tier, combinados en una
    /// puntuación. Solo lectura.
    Stale(ArgsStale),
}

#[derive(Subcommand)]
enum ComandoWrite {
    /// Crea una nota nueva con frontmatter completo y permalink derivado del
    /// título. Rechaza (exit 3) si hay candidatas duplicadas, salvo `--force`.
    New(ArgsWriteNew),
    /// Anexa al final de una bitácora sin releerla. Rechaza (exit 3) si el
    /// destino no es `tier: log`, salvo `--force`.
    Append(ArgsWriteAppend),
}

#[derive(clap::Args)]
struct ArgsInit {
    /// Raíz de la KB. Obligatorio salvo con `--from-basic-memory`.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Nombre de la KB (prefijo de permalink). Obligatorio salvo con
    /// `--from-basic-memory`.
    #[arg(long)]
    name: Option<String>,
    /// Toma raíz, nombre y embeddings de `~/.basic-memory/config.json`.
    /// Incompatible con `--kb`/`--name`: mezclarlos sería descartar uno de
    /// los dos orígenes en silencio.
    #[arg(long, conflicts_with_all = ["kb", "name"])]
    from_basic_memory: bool,
    /// Sobreescribe una config existente. En modo creación (sin
    /// `--from-basic-memory`) también autoriza volcar la semilla sobre una
    /// `--kb` no vacía, pisando lo que hubiera dentro.
    #[arg(long)]
    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsConfig {
    /// Emite la config como envelope JSON en stdout (para scripts: jq no lee
    /// TOML).
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsWriteNew {
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config. Lo usa
    /// el dup-gate; con `--force` no se consulta.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Default: `[kb] path` de `~/.exo/config.toml`.
    /// Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Directorio destino dentro de la KB (`projects`, `log`, `research`…).
    #[arg(long)]
    dir: String,
    /// Título de la nota. De él salen el nombre de fichero y el slug del
    /// permalink.
    #[arg(long = "title", value_name = "TITLE")]
    titulo: String,
    /// Fichero con el cuerpo (`-` = stdin). El contenido NO viaja por argv:
    /// el agente lo escribe con su tool `Write` y aquí solo se referencia,
    /// que es lo que evita el escaping frágil de heredocs.
    #[arg(long)]
    from: String,
    /// `tier` del frontmatter (core|stable|log).
    #[arg(long)]
    tier: Option<String>,
    /// Salta el dup-gate de similitud. JAMÁS salta una colisión de fichero.
    #[arg(long)]
    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsWriteAppend {
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Default: `[kb] path` de `~/.exo/config.toml`.
    /// Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Fichero con el texto a anexar (`-` = stdin).
    #[arg(long)]
    from: String,
    /// Crea la bitácora (`tier: log`) si no existe.
    #[arg(long = "create")]
    crea: bool,
    /// Anexa aunque el destino no sea `tier: log`. Queda registrado en el
    /// envelope (`forced: true`) para que la excepción sea auditable.
    #[arg(long)]
    force: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
    /// Permalink de la nota destino (p.ej. `mi-kb/log/proyecto-bitacora`).
    permalink: String,
}

#[derive(clap::Args)]
struct ArgsIndex {
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Por defecto, `[kb] path` de `~/.exo/config.toml`.
    /// Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

/// `search_type` del contrato §4.1 ("fts | vector | hybrid").
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum TipoBusqueda {
    Fts,
    Vector,
    Hybrid,
}

#[derive(clap::Args)]
struct ArgsSearch {
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Solo la usa la salida humana, que emite rutas absolutas
    /// (el `--json` sigue dando la relativa). Precedencia: flag > $EXO_KB >
    /// config, igual que en `recall` y `targets`.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Máximo de resultados.
    #[arg(long = "limit", value_name = "LIMIT", default_value_t = 10)]
    limite: usize,
    /// Tipo de búsqueda.
    #[arg(long, value_enum, default_value_t = TipoBusqueda::Fts)]
    r#type: TipoBusqueda,
    /// Umbral de similitud coseno de la búsqueda semántica. Si se omite,
    /// `[embeddings] min_similarity` de la config. Sin efecto en `--type fts`.
    #[arg(long = "min-similarity", value_name = "MIN_SIMILARITY")]
    min_similitud: Option<f64>,
    /// Peso del canal más débil al fusionar (`max + bonus·min`). Solo
    /// `--type hybrid`; si se omite, el default del engine.
    #[arg(long)]
    bonus: Option<f64>,
    /// Escala de normalización del score de texto completo antes de
    /// fusionar. Solo `--type hybrid`; si se omite, el default del engine.
    #[arg(long = "fts-scale", value_name = "FTS_SCALE")]
    escala_fts: Option<f64>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
    /// Texto de la consulta.
    query: String,
}

#[derive(clap::Args)]
struct ArgsRecall {
    /// Fichero SQLite del índice. Default: `[index] db` de la config
    /// (`~/.exo/config.toml`). Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Por defecto, `[kb] path` de `~/.exo/config.toml`.
    /// Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Texto de la consulta. Sin él, modo arranque (`tier: core` + recientes
    /// por git); con él, modo consulta (búsqueda híbrida).
    #[arg(long)]
    query: Option<String>,
    /// Máximo de notas: en modo arranque, cuántas recientes (las `tier: core`
    /// entran siempre); en modo consulta, cuántos resultados.
    #[arg(long = "limit", value_name = "LIMIT", default_value_t = 5)]
    limite: usize,
    /// Presupuesto de bytes del bloque de salida (texto o `--json`); trunca
    /// por líneas enteras.
    #[arg(long, default_value_t = 2048)]
    cap_bytes: usize,
    /// Umbral de similitud coseno en modo consulta. Si se omite, el de la
    /// config. Sin efecto en modo arranque.
    #[arg(long = "min-similarity", value_name = "MIN_SIMILARITY")]
    min_similitud: Option<f64>,
    /// Modo arranque con el CUERPO de las notas `tier: core` y la lista de
    /// recientes, en vez de una línea por nota. Es lo que inyecta el hook de
    /// inicio de sesión. Incompatible con `--query`.
    #[arg(long = "content")]
    contenido: bool,
    /// Permalink de la nota cuyo cuerpo se quiere en `--content` (p.ej.
    /// `core/core-index`). Sin este flag, `--content` vuelca TODAS las
    /// `tier: core` — que en una KB con un core grande agota el presupuesto
    /// con la primera. Qué nota es "la de arranque" lo decide el consumidor,
    /// no el engine.
    #[arg(long = "note", value_name = "NOTE")]
    nota: Option<String>,
    /// Refresca el índice (incremental) ANTES de servir, para no devolver una
    /// KB rancia. Barato si nada cambió; si el índice no existe, lo construye.
    #[arg(long = "refresh")]
    refresca: bool,
    /// Emite el resultado como envelope JSON en stdout. Sin él, un bloque de
    /// texto plano (el que inyectan los hooks).
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsTargets {
    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB en disco. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Máximo de candidatas.
    #[arg(long = "limit", value_name = "LIMIT", default_value_t = 10)]
    limite: usize,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
    /// Tema a buscar.
    #[arg(value_name = "TOPIC")]
    tema: String,
}

#[derive(clap::Args)]
struct ArgsBudget {
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsLint {
    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    /// `lint` lo necesita para detectar huérfanas e índice desfasado.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsDoctor {
    /// Fichero SQLite del índice que juzga `index_db`. Precedencia: flag >
    /// $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB que juzgan `kb_readable` y `kb_precommit_hook`.
    /// Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsStale {
    /// Fichero SQLite del índice. Precedencia: flag > $EXO_DB > config.
    #[arg(long)]
    db: Option<PathBuf>,
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Override del reloj, RFC3339 (por defecto: la hora real; los tests
    /// deterministas siempre lo pasan).
    #[arg(long)]
    now: Option<String>,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsRatchet {
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Escribe `.kbx-ratchet.json` con `min(sello, declarado)`. Atómico: o
    /// sella todo, o el fichero no cambia. Incompatible con `--staged`: sellar
    /// es una decisión sobre el árbol de trabajo, no sobre lo que se va a
    /// commitear.
    #[arg(long, conflicts_with = "staged")]
    seal: bool,
    /// Juzga el índice de git en vez del working tree (para el pre-commit).
    #[arg(long)]
    staged: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsRotate {
    /// Raíz de la KB. Precedencia: flag > $EXO_KB > config.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Presupuesto en bytes para la cola caliente que se queda en la nota.
    #[arg(long = "hot-bytes", value_name = "HOT_BYTES", default_value_t = 20480)]
    presupuesto_caliente: i64,
    /// Escribe de verdad. Sin este flag es un dry-run: nada toca disco.
    #[arg(long)]
    apply: bool,
    /// Emite el resultado como envelope JSON en stdout.
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    // El flag sale del parseo de clap, no de un escaneo de argv: un valor de
    // otro flag que fuese literalmente "--json" (p.ej. `--title "--json"`)
    // engañaría al escaneo y filtraría el envelope a stdout sin que nadie lo
    // hubiera pedido. Se consulta ANTES de ejecutar porque en la rama de
    // error el comando ya se ha consumido dentro de `ejecuta`.
    let json = quiere_json(&cli.comando);
    match ejecuta(cli.comando) {
        Ok(()) => {}
        Err(e) => {
            // Un gate rechazado NO es un error del sistema: es una decisión que
            // se le devuelve al llamador (nota duplicada, append al canon).
            // Sale con 3 para que el consumidor lo distinga de un fallo real
            // por exit code —jamás parseando `data`— y pueda reintentar con
            // `--force`. Con `--json` además emite el envelope: el exit code
            // sigue siendo el gate, el envelope es el detalle para quien lo
            // quiera parsear (spec write §3.3).
            if let Some(rechazo) = e.downcast_ref::<exo::escritor::Rechazo>() {
                eprintln!("rechazado: {rechazo}");
                if json {
                    exo::envelope::emite("write", rechazo.data());
                }
                std::process::exit(3);
            }
            // `budget`/`lint`: "la KB está mal" tampoco es un error del
            // sistema, pero aquí no hay un segundo envelope que emitir — el
            // informe entero ya salió por stdout ANTES de gatear (`gate.rs`).
            // Con `--json` esto solo añade la línea a stderr; el envelope no
            // se repite.
            if let Some(gate) = e.downcast_ref::<exo::gate::GateFallido>() {
                eprintln!("rechazado: {gate}");
                std::process::exit(3);
            }
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }
}

/// Si el comando invocado pidió `--json`. EXHAUSTIVO a propósito, sin `_ =>`
/// comodín: si mañana se añade un subcomando nuevo, el compilador debe
/// obligar a decidir explícitamente si emite JSON, no caer en un default
/// silencioso.
fn quiere_json(c: &Comando) -> bool {
    match c {
        Comando::Init(a) => a.json,
        Comando::Config(a) => a.json,
        Comando::Index(a) | Comando::Rebuild(a) => a.json,
        Comando::Search(a) => a.json,
        Comando::Recall(a) => a.json,
        Comando::Targets(a) => a.json,
        Comando::Budget(a) => a.json,
        Comando::Lint(a) => a.json,
        Comando::Ratchet(a) => a.json,
        Comando::Rotate(a) => a.json,
        Comando::Doctor(a) => a.json,
        Comando::Stale(a) => a.json,
        Comando::Write(w) => match w {
            ComandoWrite::New(a) => a.json,
            ComandoWrite::Append(a) => a.json,
        },
    }
}

/// Precedencia de la DB del índice: `--db` > `$EXO_DB` > `[index] db`.
///
/// `--db` era obligatorio «sin default (D6: un default sería config
/// encubierta)». Con config propia deja de serlo: un valor declarado en un
/// fichero del usuario no es config encubierta, es config.
fn resuelve_db(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    if let Ok(v) = std::env::var("EXO_DB")
        && !v.is_empty()
    {
        return Ok(exo::config::expande_tilde(Path::new(&v)));
    }
    let cfg = exo::config::carga()?;
    Ok(exo::config::expande_tilde(&cfg.index.db))
}

/// Precedencia de la raíz de la KB: `--kb` > `$EXO_KB` > `[kb] path`.
fn resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    if let Ok(v) = std::env::var("EXO_KB")
        && !v.is_empty()
    {
        return Ok(exo::config::expande_tilde(Path::new(&v)));
    }
    exo::kb_desde_config()
}

/// Ejecuta el comando ya parseado. El flag `--json` no se extrae aquí: lo
/// resuelve `quiere_json` en `main`, antes de llamar, porque en la rama de
/// error el `comando` ya se ha movido dentro de esta función.
fn ejecuta(comando: Comando) -> Result<()> {
    match comando {
        Comando::Init(args) => init_cmd(args),
        Comando::Config(args) => config_cmd(args),
        Comando::Index(args) => corre("index", args, false),
        Comando::Rebuild(args) => corre("rebuild", args, true),
        Comando::Search(args) => busca_cmd(args),
        Comando::Recall(args) => recall_cmd(args),
        Comando::Targets(args) => targets_cmd(args),
        Comando::Budget(args) => budget_cmd(args),
        Comando::Lint(args) => lint_cmd(args),
        Comando::Ratchet(args) => ratchet_cmd(args),
        Comando::Rotate(args) => rotate_cmd(args),
        Comando::Doctor(args) => doctor_cmd(args),
        Comando::Stale(args) => stale_cmd(args),
        Comando::Write(sub) => match sub {
            ComandoWrite::New(args) => write_new_cmd(args),
            ComandoWrite::Append(args) => write_append_cmd(args),
        },
    }
}

/// `git init` + primer commit sobre la KB recién volcada.
/// `std::process::Command`, nunca `git2`: añadir una dependencia con
/// toolchain C contradice D4, que existe para sacar el toolchain C del
/// camino del usuario. Un fallo en CUALQUIER paso (git ausente del PATH, o
/// presente pero sin `user.name`/`user.email` en una máquina fresca de
/// tercero — el caso más probable del público de G3) avisa por stderr y NO
/// aborta: una KB sin git funciona, abortar por eso sería peor.
fn versiona_kb(kb: &Path) -> bool {
    let paso = |args: &[&str]| -> bool {
        match std::process::Command::new("git")
            .arg("-C")
            .arg(kb)
            .args(args)
            .output()
        {
            Ok(salida) if salida.status.success() => true,
            Ok(salida) => {
                eprintln!(
                    "aviso: git {} falló: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&salida.stderr).trim()
                );
                false
            }
            Err(e) => {
                eprintln!("aviso: no se pudo ejecutar git {}: {e}", args.join(" "));
                false
            }
        }
    };

    paso(&["init"])
        && paso(&["add", "."])
        && paso(&["commit", "-m", "kb: semilla inicial de exo init"])
}

/// `exo init`: dos modos. ADOPCIÓN (`--from-basic-memory`) apunta a una KB
/// **ya existente y poblada** — no se toca ni un byte dentro de ella: nada
/// de `prepara_kb`, nada de plantilla, nada de `git init`. CREACIÓN
/// (`--kb` + `--name`) es la que nace aquí: valida el destino, vuelca la
/// semilla, la versiona con git (best-effort) y la indexa.
fn init_cmd(args: ArgsInit) -> Result<()> {
    let destino = exo::config::ruta_config()?;
    let home = dirs::home_dir().context("sin HOME")?;

    // I4 (review de rama): se comprueba ANTES de tocar nada en disco. Antes
    // esta guarda solo vivía dentro de `escribe_config`, llamada después de
    // volcar la plantilla (12 ficheros) y de `git init` + commit en modo
    // creación — sin `--force` contra una config existente, el aborto llegaba
    // tarde: KB a medio escribir + repo git en disco + exit 1, y el reintento
    // fallaba ya por otra vía (`prepara_kb`: "no está vacía").
    exo::inicia::valida_config_escribible(&destino, args.force)?;

    // H1: la DB que este `init` valida, indexa y graba en config.toml
    // (`$EXO_DB` > `~/.exo/index.db`; la regla vive en `db_de_init`).
    let exo_db = std::env::var("EXO_DB").ok();
    let db_objetivo = exo::inicia::db_de_init(exo_db.as_deref(), &home);

    let (kb, nombre, emb, modo, escritos, git_ok) = if args.from_basic_memory {
        let ruta = exo::inicia::ruta_basic_memory()?;
        let json =
            std::fs::read_to_string(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
        let (kb, nombre, emb) = exo::inicia::desde_basic_memory(&json)
            .with_context(|| format!("leyendo {}", ruta.display()))?;
        // El nombre no lo escribió el usuario en esta rama: salió de
        // `default_project` en el JSON de basic-memory. El mensaje de
        // `valida_nombre` lo deja claro para no confundir a quien lea el
        // error pensando que tecleó `--name` él mismo.
        exo::inicia::valida_nombre(&nombre).with_context(|| {
            format!(
                "el nombre adoptado ({nombre:?}) viene de `default_project` en {} \
                 (JSON de basic-memory), no lo escribiste tú con --name — corrígelo ahí",
                ruta.display()
            )
        })?;
        exo::inicia::valida_db_para_kb(&db_objetivo, &kb)?;
        (kb, nombre, emb, "adopt", Vec::new(), false)
    } else {
        // `expande_tilde`: sin esto, `exo init --kb ~/mi-kb` en Windows crea
        // un directorio LITERAL llamado `~` (el shell no expande `~` como en
        // POSIX), sale 0, y la config queda apuntando a un sitio que no es el
        // que el usuario escribió — falla ruidoso en el comando siguiente,
        // pero ya con basura en disco (Minor, review de rama).
        let kb = exo::config::expande_tilde(
            &args
                .kb
                .context("--kb es obligatorio sin --from-basic-memory")?,
        );
        let nombre = args
            .name
            .context("--name es obligatorio sin --from-basic-memory")?;
        // Defaults del modelo de producción: los mismos que la línea base del
        // eval, declarados en la spec como posicionamiento (producto en español).
        // `MODELO_JINA_ES` (no un literal duplicado): `repo_hf` en lib.rs solo
        // pinea la revisión si la cadena coincide EXACTAMENTE con esa
        // constante; un literal que divergiera silenciosamente destino a
        // `main` (móvil) y perdería la comparabilidad del índice y del eval.
        let emb = exo::config::Embeddings {
            model: exo::MODELO_JINA_ES.to_string(),
            // 768 es la dimensionalidad DE ESTE modelo (MODELO_JINA_ES): si
            // se cambia uno, el otro tiene que cambiar con él.
            dims: 768,
            min_similarity: 0.35,
        };

        exo::inicia::valida_nombre(&nombre)?;
        exo::inicia::valida_db_para_kb(&db_objetivo, &kb)?;
        exo::inicia::prepara_kb(&kb, args.force)?;
        std::fs::create_dir_all(&kb).with_context(|| format!("crear {}", kb.display()))?;
        let escritos = exo::plantilla::vuelca(&kb, &nombre)?;
        let git_ok = versiona_kb(&kb);

        (kb, nombre, emb, "create", escritos, git_ok)
    };

    // Se graba `db_objetivo`, NO el default `~/.exo/index.db`: `db_objetivo`
    // es la DB que este `init` acaba de validar (H1) e indexa más abajo — es
    // la efectiva. Grabar siempre el default era el bug: con `$EXO_DB` puesto,
    // la config quedaba mintiendo sobre qué DB usa este `init` (indexaba una
    // DB y apuntaba a otra), y un `exo config`/`exo search` posterior que
    // solo pusiera `$EXO_CONFIG` resolvía silenciosamente al índice
    // equivocado.
    exo::inicia::escribe_config(&destino, &kb, &nombre, &emb, &db_objetivo, args.force)?;

    // Índice inicial. `resuelve_db(None)` (precedencia `$EXO_DB` > `[index]
    // db`) en vez de usar `db_objetivo` directamente: son la misma ruta ahora
    // que la config también la graba, pero pasar por `resuelve_db` mantiene
    // este `init_cmd` en el mismo camino de resolución que cualquier otro
    // comando — si `resuelve_db` cambiara de precedencia, `init` la sigue
    // sin tener que tocarse.
    let db = resuelve_db(None)?;
    let resumen = indexa(&kb, &db)?;

    // C2 (review de rama): solo en modo CREACIÓN — en ADOPCIÓN la KB es del
    // usuario y su contenido no lo decide `init`. La lógica vive en
    // `inicia::verifica_indexado_completo` (testable sin pasar por el
    // binario); aquí solo se orquesta.
    if modo == "create" {
        exo::inicia::verifica_indexado_completo(&escritos, resumen.indexadas).with_context(
            || {
                format!(
                    "la config y la KB ya están en disco en {} — revísala antes de reintentar",
                    kb.display()
                )
            },
        )?;
    }

    if args.json {
        exo::envelope::emite(
            "init",
            serde_json::json!({
                "config": destino.display().to_string(),
                "kb": kb.display().to_string(),
                "name": nombre,
                "from_basic_memory": args.from_basic_memory,
                "mode": modo,
                "files": escritos.len(),
                "git": git_ok,
                "index": resumen,
            }),
        );
    } else {
        match modo {
            "adopt" => {
                println!("KB existente adoptada: {} (name: {nombre})", kb.display());
                println!("config escrita en {}", destino.display());
            }
            _ => {
                println!(
                    "KB semilla volcada en {} ({} ficheros)",
                    kb.display(),
                    escritos.len()
                );
                if !git_ok {
                    eprintln!("aviso: la KB no quedó versionada con git (ver avisos arriba)");
                }
                println!("config escrita en {}", destino.display());
            }
        }
        println!(
            "índice: indexadas={} saltadas={} sin_permalink={} borradas={}",
            resumen.indexadas, resumen.saltadas, resumen.sin_permalink, resumen.borradas
        );
        println!("KB: {} (name: {nombre})", kb.display());
        println!("siguiente: exo recall --json");
    }
    Ok(())
}

/// Ruta lista para cruzar la frontera hacia shell: barras normales siempre.
/// `expande_tilde` hace `home.join(resto)` con `home` en formato nativo de
/// Windows (`\`) y `resto` con el separador literal del TOML (`/`); el
/// `PathBuf` resultante mezcla los dos y a Rust no le importa (así abre
/// ficheros en todo el engine), pero un script de shell sí que lo nota si
/// hace `dirname`, recorta un prefijo o compara contra otra ruta. Se
/// normaliza aquí, en la salida, no en `expande_tilde`: ese `PathBuf` es de
/// uso interno del engine, y el problema nace justo al cruzar a shell.
fn ruta_shell(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// `exo config`: la config efectiva, con rutas expandidas.
fn config_cmd(args: ArgsConfig) -> Result<()> {
    let cfg = exo::config::carga()?;
    let kb = exo::config::expande_tilde(&cfg.kb.path);
    let db = exo::config::expande_tilde(&cfg.index.db);
    let data = serde_json::json!({
        "kb": { "path": ruta_shell(&kb), "name": cfg.kb.name },
        "index": { "db": ruta_shell(&db) },
        "embeddings": {
            "model": cfg.embeddings.model,
            "dims": cfg.embeddings.dims,
            "min_similarity": cfg.embeddings.min_similarity,
        },
    });
    if args.json {
        exo::envelope::emite("config", data);
    } else {
        println!("kb.path   {}", kb.display());
        println!("kb.name   {}", cfg.kb.name);
        println!("index.db  {}", db.display());
        println!("model     {}", cfg.embeddings.model);
    }
    Ok(())
}

/// Lee el contenido de `--from`: ruta de fichero, o stdin si es `-`.
fn lee_from(from: &str) -> Result<String> {
    if from == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .context("leer el cuerpo desde stdin")?;
        return Ok(buf);
    }
    std::fs::read_to_string(from).with_context(|| format!("leer el cuerpo desde {from}"))
}

/// `exo write new`: dup-gate con `busca_hybrid` (salvo `--force`) y creación
/// de la nota. El gate corre ANTES de tocar el disco.
fn write_new_cmd(args: ArgsWriteNew) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let db = resuelve_db(args.db)?;
    let cuerpo = lee_from(&args.from)?;

    // El nombre del proyecto sale de `[kb] name` de la config, EXPLÍCITO.
    // Antes se derivaba de `kb.file_name()` contra lo que decía la spec §3.1;
    // coincidían por suerte y el día que no, reventaba en silencio.
    let proyecto = exo::nombre_kb()?;

    // Dup-gate por solape de slug, NO por retrieval: ver `solape_slug`.
    // Barato y determinista — no carga el modelo de embeddings, así que el
    // cierre de sesión no paga segundos por esta comprobación.
    let candidatas: Vec<(String, f64)> = if args.force {
        Vec::new()
    } else {
        let indexados =
            exo::buscador::permalinks(&db).context("dup-gate: leer permalinks del índice")?;
        exo::escritor::dup_candidatas(&exo::escritor::slug(&args.titulo), &indexados)
    };

    let esc = escribe_nueva(&NuevaNota {
        kb: &kb,
        proyecto: &proyecto,
        dir: &args.dir,
        titulo: &args.titulo,
        cuerpo: &cuerpo,
        tier: args.tier.as_deref(),
        dup_candidatas: &candidatas,
        forzado: args.force,
    })?;

    emite_escritura(esc, args.json);
    Ok(())
}

/// Walk de confirmación del gate M4 #5 (Ola 1 G Task 7): busca, DENTRO de
/// `dir` (no recursivo — las bitácoras de `--create` viven a un nivel), un
/// `.md` cuyo frontmatter declare exactamente `permalink`. `None` si `dir`
/// no existe todavía (KB nueva) o si ningún fichero lo declara.
fn busca_permalink_en_dir(kb: &Path, dir: &str, permalink: &str) -> Result<Option<PathBuf>> {
    let carpeta = kb.join(dir);
    if !carpeta.is_dir() {
        return Ok(None);
    }
    for entrada in std::fs::read_dir(&carpeta)
        .with_context(|| format!("leer directorio {}", carpeta.display()))?
    {
        let entrada = entrada.with_context(|| format!("entrada de {}", carpeta.display()))?;
        let ruta = entrada.path();
        if !exo::walker::es_md(&entrada.file_name().to_string_lossy()) {
            continue;
        }
        if let Some(nota) = exo::nota::parsea_nota(&ruta)?
            && nota.permalink == permalink
        {
            return Ok(Some(ruta));
        }
    }
    Ok(None)
}

/// `exo write append`: resuelve permalink→ruta contra el índice y anexa. Con
/// `--create`, una bitácora que no existe se crea en vez de fallar.
fn write_append_cmd(args: ArgsWriteAppend) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let db = resuelve_db(args.db)?;
    let texto = lee_from(&args.from)?;

    let ruta_rel = match exo::buscador::ruta_de(&db, &args.permalink)? {
        Some(r) => r,
        None if args.crea => {
            // Sin fila en el índice: o la bitácora no existe, o el índice está
            // rancio. Derivar la ruta del permalink es correcto SOLO para
            // crearla (`log/<slug>.md`); para una nota ya existente el slug no
            // es invertible y por eso jamás se adivina.
            //
            // M4 #6 (Ola 1 G Task 7, backlog:708-730): exige EXACTAMENTE 3
            // segmentos. Antes, un permalink de 2 segmentos
            // (<proyecto>/<slug>) colaba el primer segmento como <dir> vía
            // `map_or(izq, ...)`, creando un directorio espurio
            // (`<proyecto>/<slug>.md`) que no es ni el proyecto real ni un
            // dir pedido por nadie.
            let (izq, slug_nota) = args
                .permalink
                .rsplit_once('/')
                .context("permalink sin forma <proyecto>/<dir>/<slug>")?;
            let (_, dir) = izq.rsplit_once('/').with_context(|| {
                format!(
                    "{:?} tiene menos de 3 segmentos (<proyecto>/<dir>/<slug>): \
                     --create no puede inferir un directorio",
                    args.permalink
                )
            })?;
            let rel = format!("{dir}/{slug_nota}.md");

            if !kb.join(&rel).exists() {
                // M4 #5 (Ola 1 G Task 7, backlog:708-730): walk de
                // confirmación antes de crear. El check de arriba solo mira
                // la ruta CANÓNICA; con índice rancio, la bitácora real
                // puede vivir bajo OTRO nombre de fichero con el mismo
                // permalink (renombrada a mano tras `exo index`). Crear
                // otra encima dejaría DOS ficheros con el mismo permalink.
                if let Some(existente) = busca_permalink_en_dir(&kb, dir, &args.permalink)? {
                    anyhow::bail!(
                        "{} ya existe con este permalink bajo otro nombre: {} \
                         (índice rancio: corre `exo rebuild` o usa ese fichero directamente)",
                        args.permalink,
                        existente.display()
                    );
                }
                let proyecto = exo::nombre_kb()?;
                escribe_nueva(&NuevaNota {
                    kb: &kb,
                    proyecto: &proyecto,
                    dir,
                    titulo: slug_nota,
                    cuerpo: "",
                    tier: Some("log"),
                    dup_candidatas: &[],
                    forzado: false,
                })
                .context("crear la bitácora con --create")?;
                eprintln!("write: bitácora creada en {rel}");
            }
            rel
        }
        None => anyhow::bail!(
            "{} no está en el índice: comprueba el permalink, o usa --create si la bitácora aún no existe",
            args.permalink
        ),
    };

    let esc = escribe_append(&kb, &ruta_rel, &texto, args.force)?;
    emite_escritura(esc, args.json);
    Ok(())
}

/// Salida común de `write`: envelope v2 por stdout con `--json`, o una línea
/// humana con la ruta absoluta —lo que `/documenta` necesita para su commit
/// scoped— por stdout sin él.
fn emite_escritura(esc: exo::escritor::Escritura, json: bool) {
    if esc.forzado {
        eprintln!("aviso: escritura forzada (--force); queda registrada en el envelope");
    }
    if json {
        envelope::emite(
            "write",
            serde_json::to_value(&esc).expect("Escritura es serializable"),
        );
    } else {
        println!("{}", esc.ruta_abs);
    }
}

/// `exo recall`: resuelve `--kb`, delega en `recall_arranque`/`recall_consulta`
/// según haya o no `--query`, aplica el cap de bytes (`renderiza`, único
/// punto que decide qué notas entran) y emite. Exit codes (brief, "el
/// consumidor gatea por exit code, jamás por campos de `data`"): 0 = hay
/// bloque (aunque venga truncado); recall vacío (cero notas tras el cap) =
/// `bail!` = exit 1, sin tabla de códigos nueva.
fn recall_cmd(args: ArgsRecall) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let db = resuelve_db(args.db)?;

    let mut refresh_s = None;
    if args.refresca {
        // El resumen va a stderr: stdout es exclusivo del envelope/bloque
        // (contrato §4), y el hook consume stdout tal cual.
        let inicio = std::time::Instant::now();
        let resumen = exo::refresca_indice(&kb, &db)
            .context("refrescar el índice antes del recall (--refresh)")?;
        refresh_s = Some(inicio.elapsed().as_secs_f64());
        if resumen.indexadas > 0 || resumen.borradas > 0 {
            eprintln!(
                "refresca: indexadas={} borradas={} saltadas={}",
                resumen.indexadas, resumen.borradas, resumen.saltadas
            );
        }
    }

    if args.contenido {
        if args.query.is_some() {
            anyhow::bail!("--content es del modo arranque: no se combina con --query");
        }
        // El aviso de kb_root (si lo hay) sale por stderr desde DENTRO de
        // `recall_arranque_contenido` — este camino no tiene envelope ni
        // `Recall.avisos`, así que no hay nada que reenviar aquí.
        // Camino del hook: bloque de texto a stdout y fuera. No pasa por el
        // envelope ni por `aplica_cap` (trae su propio truncado por líneas).
        let bloque = exo::recall::recall_arranque_contenido(
            &db,
            &kb,
            args.limite,
            args.cap_bytes,
            args.nota.as_deref(),
        )?;
        print!("{bloque}");
        return Ok(());
    }

    let bruto = match &args.query {
        None => recall_arranque(&db, &kb, args.limite)?,
        Some(q) => {
            let mut bruto = recall_consulta(
                &db,
                q,
                args.limite,
                args.min_similitud,
                BONUS_SELLADO,
                ESCALA_FTS_SELLADA,
                &kb,
            )?;
            resuelve_rutas_absolutas(&mut bruto, &kb);
            bruto
        }
    };

    let mut resultado = renderiza(bruto, args.cap_bytes);
    resultado.recall.refresh_s = refresh_s;

    // H2: los avisos van a stderr SIEMPRE, igual que en `busca_cmd`, y ANTES
    // del bail de «recall vacío»: un arm vector INERTE sin hits FTS es justo
    // el caso en que más importa verlo, y el que antes se perdía entero.
    for aviso in &resultado.recall.avisos {
        eprintln!("aviso: {aviso}");
    }

    if resultado.recall.notas.is_empty() {
        anyhow::bail!(
            "recall vacío (modo {}): sin notas para el bloque, no se emite nada",
            resultado.recall.modo
        );
    }

    if resultado.recall.truncado {
        eprintln!(
            "aviso: recall truncado por --cap-bytes={} ({} líneas descartadas)",
            args.cap_bytes, resultado.lineas_perdidas
        );
    }

    if args.json {
        envelope::emite("recall", serde_json::to_value(&resultado.recall)?);
    } else {
        print!("{}", resultado.texto);
    }
    Ok(())
}

fn busca_cmd(args: ArgsSearch) -> Result<()> {
    let db = resuelve_db(args.db)?;

    // `search` no tiene `--kb`: mismo resolvedor que el resto del binario
    // (`resuelve_kb`, precedencia `$EXO_KB` > `[kb] path` de la config, ya
    // que no hay flag). `.ok()`: sin KB resoluble (p.ej. `search --db` sin
    // config) el aviso es `None` — nunca un error que tumbe `search`.
    let kb = resuelve_kb(None).ok();

    let resultado = match args.r#type {
        TipoBusqueda::Fts => busca(&db, &args.query, args.limite, kb.as_deref())?,
        TipoBusqueda::Vector => busca_vector(
            &db,
            &args.query,
            args.limite,
            args.min_similitud,
            kb.as_deref(),
        )?,
        TipoBusqueda::Hybrid => busca_hybrid(
            &db,
            &args.query,
            args.limite,
            args.min_similitud,
            args.bonus.unwrap_or(BONUS_SELLADO),
            args.escala_fts.unwrap_or(ESCALA_FTS_SELLADA),
            kb.as_deref(),
        )?,
    };

    // El aviso de kb_root es SOLO stderr, nunca el envelope: `Busqueda`
    // trae el campo `aviso_kb_root` con `#[serde(skip)]` justo para eso —
    // no es una clave nueva de `data`, es un canal que no se serializa.
    if let Some(aviso) = &resultado.aviso_kb_root {
        eprintln!("aviso: {aviso}");
    }

    // Los avisos van a stderr SIEMPRE, con o sin `--json`: nunca contaminan el
    // envelope de stdout, y quien mira la terminal ve la degradación sin
    // tener que parsear nada. Un instrumento degradado que no grita es el
    // modo de fallo caro que este campo existe para matar.
    for aviso in &resultado.avisos {
        eprintln!("aviso: {aviso}");
    }

    if args.json {
        envelope::emite("search", serde_json::to_value(&resultado)?);
    } else if resultado.results.is_empty() {
        // Contrato alineado con `targets_cmd` (más abajo, `no candidates`):
        // una terminal en blanco no distingue "sin resultados" de "no filtré
        // la salida". El envelope JSON no cambia — `results: []` ya lo
        // distinguía ahí.
        println!("no results");
    } else {
        // La humana da la ruta ABSOLUTA: el cwd del agente es el repo en el que
        // trabaja, no la KB, así que una relativa no se le puede pasar a `Edit`
        // sin resolver antes la raíz — y eso devolvería el jq que esta columna
        // existe para quitar. Mismo criterio que `exo write`.
        let kb = resuelve_kb(args.kb).context(
            "la salida humana de `exo search` emite rutas absolutas y necesita la \
             raíz de la KB: pasa --kb, o corre `exo init`",
        )?;
        let raiz = exo::walker::ruta_portable(&kb.display().to_string());
        let raiz = raiz.trim_end_matches('/');

        let mut sin_ruta = 0usize;
        for r in &resultado.results {
            let ruta = match &r.ruta {
                Some(rel) => format!("{raiz}/{}", exo::walker::ruta_portable(rel)),
                None => {
                    sin_ruta += 1;
                    // Un token, sin espacios: `awk '{print $4}'` sin -F es el
                    // hábito más común y partiría un marcador con espacios.
                    "(sin-ruta:rebuild)".to_string()
                }
            };
            println!("{}\t{}\t{:.4}\t{}", r.permalink, r.tipo, r.score, ruta);
        }
        if sin_ruta > 0 {
            // A stderr y FUERA de `warnings`: `path: null` ya es inferible
            // desde `results`, y el envelope no cambia por esto.
            eprintln!(
                "aviso: {sin_ruta} de {} resultados sin ruta (índice inconsistente): exo rebuild",
                resultado.results.len()
            );
        }
    }
    Ok(())
}

/// Candidatas de la KB para un tema (`exo::objetivos::busca_objetivos`).
/// Solo lectura: la DB inexistente se comprueba ANTES de abrir para que un
/// typo en `--db` no la cree como efecto colateral (mismo contrato que
/// `recall`/`search`).
fn targets_cmd(args: ArgsTargets) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!(
            "DB no encontrada: {} — corre `exo index` primero",
            db_ruta.display()
        );
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;
    let resultado = exo::objetivos::busca_objetivos(&conn, &kb, &args.tema, args.limite)?;

    if args.json {
        envelope::emite("targets", serde_json::to_value(&resultado)?);
    } else if resultado.candidatos.is_empty() {
        println!("no candidates");
    } else {
        for c in &resultado.candidatos {
            println!(
                "{}\ttier={}\tsize={}\tlast_commit={}\theadings={:?}",
                c.permalink, c.tier, c.tamano_bytes, c.ultimo_commit, c.headings
            );
            println!("\t{}", c.snippet);
        }
    }
    Ok(())
}

/// `exo budget`: presupuestos por tier sobre el árbol de ficheros. Solo lee
/// disco (sin `--db`: `budget` no toca el índice, a diferencia de `lint`).
///
/// El informe se emite ENTERO antes de gatear: quien lo consume necesita
/// saber QUÉ falló, no solo que falló.
fn budget_cmd(args: ArgsBudget) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let informe = exo::presupuesto::analiza(
        &kb,
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )?;

    if args.json {
        envelope::emite("budget", serde_json::to_value(&informe)?);
    } else {
        for fila in &informe.tiers {
            println!(
                "{}\tnotas={}\tbytes={}\tpresupuesto={}\tdelta={}",
                fila.tier, fila.notas, fila.bytes, fila.presupuesto, fila.delta
            );
        }
        for o in &informe.infractoras {
            println!(
                "offender: {} ({}) {}/{} bytes",
                o.ruta, o.tier, o.tamano_bytes, o.presupuesto
            );
        }
        for w in &informe.waived {
            println!(
                "waived: {} ({}) {}/{} bytes",
                w.ruta, w.tier, w.tamano_bytes, w.presupuesto
            );
        }
        for na in &informe.sin_aire {
            println!(
                "no-air: {} ({}) {}/{} bytes a ras — poda a {} para el 15% de aire",
                na.ruta,
                na.tier,
                na.tamano_bytes,
                na.presupuesto,
                exo::presupuesto::objetivo_poda(na.presupuesto)
            );
        }
        for n in &informe.notier {
            println!("notier: {n}");
        }
    }

    if informe.excedido() {
        return Err(exo::gate::GateFallido {
            comando: "budget",
            detalle: format!(
                "{} nota(s) sobre presupuesto, {} sin tier legal",
                informe.infractoras.len(),
                informe.notier.len()
            ),
        }
        .into());
    }
    Ok(())
}

/// `exo lint`: los siete checks de deriva de la KB. A diferencia de `budget`
/// sí necesita `--db`: `orphan` e `index_stale` leen `notas`. Misma DB
/// inexistente = error real (exit 1, no exit 3): el binario no puede
/// trabajar, que es distinto de "la KB está mal" (`index_stale` sí es exit 3).
fn lint_cmd(args: ArgsLint) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!(
            "DB no encontrada: {} — corre `exo index` primero",
            db_ruta.display()
        );
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;
    let informe = exo::lint::analiza(
        &conn,
        &kb,
        exo::presupuesto::NOMINALES,
        &exo::presupuesto::EXCLUIDOS,
    )?;

    if args.json {
        envelope::emite("lint", serde_json::to_value(&informe)?);
    } else {
        if informe.hallazgos.is_empty() {
            println!("ok");
        } else {
            for h in &informe.hallazgos {
                println!("{}\t{}\t{}", h.tipo, h.ruta, h.detalle);
            }
        }
        // Los waived son la superficie de auditoría humana de las excepciones
        // reconocidas y se imprimen SIEMPRE, incluso en un run limpio
        // (informe.hallazgos vacío / "ok"): mismo contrato que budget_cmd
        // arriba, que a su vez porta el de emitDoctorReport en el kbx
        // original (cmd/kbx/main.go): "Waived items surface even on a clean
        // (ok:true) run: they are the human-facing audit surface for
        // recognized exceptions (spec §10)".
        for w in &informe.waived {
            println!("waived\t{}\t{}\t{}", w.tipo, w.ruta, w.detalle);
        }
    }

    if !informe.ok {
        return Err(exo::gate::GateFallido {
            comando: "lint",
            detalle: format!("{} hallazgo(s)", informe.hallazgos.len()),
        }
        .into());
    }
    Ok(())
}

/// `exo doctor`: preflight de la máquina, no de la KB. Emite el informe
/// entero SIEMPRE y solo después gatea: exit 3 si algún check sale `fail`.
fn doctor_cmd(args: ArgsDoctor) -> Result<()> {
    let mut entorno = exo::doctor::Entorno::del_proceso();
    // Misma precedencia que el resto de verbos (`resuelve_kb`/`resuelve_db`:
    // flag > $EXO_KB/$EXO_DB > config). A diferencia de `search`/`index`, un
    // fallo de resolución aquí NO aborta el comando con `?`: `analiza` no
    // puede devolver `Result` —un check que no sabe qué artefacto mirar es
    // una fila `fail`, nunca un error que mate `doctor`—, así que se queda
    // en `None` y los checks que dependen de `kb`/`db` caen a la `Config`
    // que `analiza` carga por su cuenta, reportando `fail` si tampoco hay
    // eso. Antes de este arreglo `doctor` ni siquiera intentaba `$EXO_KB`/
    // `$EXO_DB`: con la env puesta, dictaminaba sobre una KB que ningún otro
    // verbo estaba tocando (review final de rama, 2026-09-11).
    entorno.kb = resuelve_kb(args.kb).ok();
    entorno.db = resuelve_db(args.db).ok();
    let informe = exo::doctor::analiza(&entorno);

    // El informe entero sale SIEMPRE, pase lo que pase con el gate: un
    // preflight que se calla justo cuando algo va mal no sirve de nada.
    if args.json {
        envelope::emite("doctor", serde_json::to_value(&informe)?);
    } else {
        for c in &informe.checks {
            println!("{}\t{}\t{}\t{}", c.estado, c.id, c.artefacto, c.detalle);
        }
    }

    if !informe.ok {
        return Err(exo::gate::GateFallido {
            comando: "doctor",
            detalle: format!("{} check(s) en fail", informe.fallidos()),
        }
        .into());
    }
    Ok(())
}

/// Etiqueta JSON del tipo (kebab-case, D7/D8). Se reutiliza el `#[serde(rename)]`
/// que ya lleva `trinquete::Tipo` en vez de duplicar las nueve cadenas a mano
/// aquí: un rename en el tipo no puede divergir en silencio de lo que el texto
/// humano imprime.
fn tipo_str(t: exo::trinquete::Tipo) -> String {
    serde_json::to_value(t)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Una línea por hallazgo, con la cifra sobre la que hay que actuar — no solo
/// las que el check comparó: para un fallo de aire es el tamaño al que hay que
/// podar la nota, que es el objeto entero de la guarda. Puerto de
/// `formatFinding` (`cmd/kbx/ratchet.go`, kbx `fe46443`).
fn formatea_hallazgo(h: &exo::trinquete::Hallazgo) -> String {
    use exo::trinquete::Tipo;
    let tipo = tipo_str(h.tipo);
    match h.tipo {
        Tipo::SinAire => format!(
            "{}: {tipo} — techo {}, poda la nota a ≤ {} B (o sella ≥ {})",
            h.ruta,
            h.ahora,
            exo::presupuesto::objetivo_poda(h.ahora),
            h.limite
        ),
        Tipo::DeudaSinAire => format!(
            "{}: {tipo} — techo {} a ras; con 15% de aire sería {} (deuda, no bloquea)",
            h.ruta, h.ahora, h.limite
        ),
        Tipo::NaceDemasiadoGrande => format!(
            "{}: {tipo} — mide {} B y el cap de su tier es {}: pártela, o adelgázala a ≤ {} B",
            h.ruta,
            h.ahora,
            h.limite,
            exo::presupuesto::objetivo_poda(h.limite)
        ),
        // Los tipos del propio trinquete (subida/retirada de sello, waiver
        // sobre el sello, primera declaración, sello escapado de tier)
        // comparan un antes y un después; los de aire no tienen "era", y
        // hubiera impreso "era 0" como si fuera un cero real.
        _ if h.era != 0 => format!(
            "{}: {tipo} (era {}, ahora {}, límite {})",
            h.ruta, h.era, h.ahora, h.limite
        ),
        _ => format!(
            "{}: {tipo} (ahora {}, límite {})",
            h.ruta, h.ahora, h.limite
        ),
    }
}

/// Salida de texto de `exo ratchet`: la causa primero, la deuda resumida al
/// final. Portado de `TestTextOutputLeadsWithTheCauseAndSummarisesTheDebt`
/// (`cmd/kbx/ratchet_test.go`, kbx `fe46443`): los hallazgos que rompen el
/// gate (`Tipo::rompe`) se listan enteros arriba; los informativos
/// (`DeudaSinAire`, `WaiverLogInerte`) NO se mezclan con ellos — si hay algo
/// que rompe, la deuda se resume en una sola línea, porque en la KB real hay
/// once sellos con deuda y mezclarla con la única causa real enterraría la
/// línea que importa. Si nada rompe, la deuda sí se lista entera: es un
/// informe, no un gate, y es la cola de trabajo de la siguiente pasada.
fn imprime_informe_ratchet(informe: &exo::trinquete::Informe) {
    if !informe.aplicado {
        println!(
            "ratchet: abstención — {}",
            informe.razon.as_deref().unwrap_or("sin razón")
        );
        return;
    }
    if informe.hallazgos.is_empty() {
        println!("ratchet: limpio");
        return;
    }
    let (rompe, informativos): (Vec<_>, Vec<_>) =
        informe.hallazgos.iter().partition(|h| h.tipo.rompe());
    for h in &rompe {
        println!("{}", formatea_hallazgo(h));
    }
    if rompe.is_empty() {
        for h in &informativos {
            println!("{}", formatea_hallazgo(h));
        }
    } else if !informativos.is_empty() {
        println!(
            "\n({} hallazgo(s) más sin romper el gate — `exo ratchet --kb <kb> --json` los lista)",
            informativos.len()
        );
    }
}

/// `exo stale`: urgencia de actualización por nota (`obsolescencia::calcula`).
/// Solo lectura — el único exit no-cero es 1, un error de IO/parseo; la
/// obsolescencia en sí es información, no un veredicto de gate (kbx: "the
/// only non-zero exit is 2 (IO/usage)" — misma idea, exit distinto porque
/// en exo 2 es de clap).
fn stale_cmd(args: ArgsStale) -> Result<()> {
    let db_ruta = resuelve_db(args.db)?;
    if !db_ruta.exists() {
        anyhow::bail!(
            "DB no encontrada: {} — corre `exo index` primero",
            db_ruta.display()
        );
    }
    let kb = resuelve_kb(args.kb)?;
    let conn = exo::abre_db(&db_ruta)?;

    let ahora_epoch = match args.now {
        Some(marca) => exo::obsolescencia::epoch_utc_de_iso8601(&marca)
            .with_context(|| format!("stale: --now inválido: {marca:?}"))?,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("stale: reloj del sistema anterior a 1970")?
            .as_secs() as i64,
    };

    let informe =
        exo::obsolescencia::calcula(&conn, &kb, &exo::presupuesto::EXCLUIDOS, ahora_epoch)?;

    if args.json {
        envelope::emite("stale", serde_json::to_value(&informe)?);
    } else {
        println!("now: {}", informe.now);
        for n in &informe.notes {
            let commit = if n.sin_commit {
                "(uncommitted)".to_string()
            } else {
                n.ultimo_commit.clone()
            };
            println!(
                "{:<40} tier={:<6} age_days={:<6} degree={:<3} last_commit={} score={:.2}",
                n.path,
                n.tier,
                n.edad_dias,
                n.degree,
                commit,
                n.score.valor()
            );
        }
    }
    Ok(())
}

/// `exo ratchet`: el trinquete de techos declarados. Solo lee disco (`--kb`),
/// sin `--db`: igual que `budget`, el trinquete no toca el índice.
///
/// El informe se emite ENTERO antes de gatear (mismo contrato que
/// `budget_cmd`/`lint_cmd`): quien lo consume necesita saber QUÉ rompió, no
/// solo que rompió. La abstención (`!informe.aplicado`) nunca gatea —
/// `Informe::fallido` ya lo garantiza — así que sale 0 igual que un informe
/// limpio.
fn ratchet_cmd(args: ArgsRatchet) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let presupuestos = exo::presupuesto::NOMINALES;
    let excluidos = exo::presupuesto::EXCLUIDOS;

    if args.seal {
        return ratchet_seal_cmd(&kb, presupuestos, &excluidos, args.json);
    }

    let declaradas = if args.staged {
        exo::trinquete::recolecta_staged(&kb, presupuestos, &excluidos)?
    } else {
        exo::trinquete::recolecta(&kb, presupuestos, &excluidos)?
    };
    let informe = if args.staged {
        exo::trinquete::comprueba_staged(&kb, &declaradas, presupuestos)?
    } else {
        exo::trinquete::comprueba(&kb, &declaradas, presupuestos)?
    };

    if args.json {
        envelope::emite("ratchet", serde_json::to_value(&informe)?);
    } else {
        imprime_informe_ratchet(&informe);
    }

    if informe.fallido() {
        return Err(exo::gate::GateFallido {
            comando: "ratchet",
            detalle: format!(
                "{} hallazgo(s) rompen el gate",
                informe.hallazgos.iter().filter(|h| h.tipo.rompe()).count()
            ),
        }
        .into());
    }
    Ok(())
}

/// `exo ratchet --seal`: escribe `.kbx-ratchet.json` con `min(sello,
/// declarado)`. Atómico (A contrato de la Task 11): se calcula el siguiente
/// estado y SUS violaciones de aire ANTES de tocar disco, y solo se escribe
/// si la lista viene vacía — "o sella todo o no sella nada", porque quien
/// está podando necesita la lista entera de infractores para hacer una
/// pasada, no N pasadas reintentando `--seal` una nota a la vez.
///
/// Las violaciones de `violaciones_de_aire` son siempre de un tipo que rompe
/// (`SinAire`/`NaceDemasiadoGrande`, nunca `DeudaSinAire`: esa función solo
/// juzga transiciones), así que se reutiliza `imprime_informe_ratchet` sin
/// tener que distinguir causa de deuda aquí — no hay deuda que mezclar.
fn ratchet_seal_cmd(
    kb: &Path,
    presupuestos: exo::presupuesto::Presupuestos,
    excluidos: &[&str],
    json: bool,
) -> Result<()> {
    let declaradas = exo::trinquete::recolecta(kb, presupuestos, excluidos)?;
    let actual = exo::trinquete::carga(kb)?;
    let siguiente = exo::trinquete::sella(&actual, &declaradas);
    let violaciones = exo::trinquete::violaciones_de_aire(&actual, &siguiente, &declaradas);

    if !violaciones.is_empty() {
        let informe = exo::trinquete::Informe {
            aplicado: true,
            razon: None,
            hallazgos: violaciones,
        };
        if json {
            envelope::emite("ratchet", serde_json::to_value(&informe)?);
        } else {
            imprime_informe_ratchet(&informe);
        }
        return Err(exo::gate::GateFallido {
            comando: "ratchet",
            detalle: format!(
                "{} techo(s) sin aire, no se selló nada",
                informe.hallazgos.len()
            ),
        }
        .into());
    }

    exo::trinquete::escribe_sellos(kb, &siguiente)?;

    if json {
        envelope::emite(
            "ratchet",
            serde_json::json!({ "applied": true, "sealed": siguiente.len() }),
        );
    } else {
        println!(
            "ratchet: sellados {} techo(s) en {}",
            siguiente.len(),
            exo::trinquete::FICHERO_SELLO
        );
    }
    Ok(())
}

/// `exo rotate`: barre `log/` (solo el nivel superior — igual que kbx, sin
/// recursión ni el resto de la KB) y rota cada nota `tier: log` cuya cola
/// fría exceda el presupuesto. Un fallo en una nota no aborta la barrida:
/// se acumula y el exit code final lo refleja con `bail!` (exit 1 — D-3:
/// no es un `GateFallido`, es un fichero que no se pudo procesar).
fn rotate_cmd(args: ArgsRotate) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    if args.presupuesto_caliente <= 0 {
        anyhow::bail!(
            "rotate: --hot-bytes tiene que ser > 0, se recibió {}",
            args.presupuesto_caliente
        );
    }
    // D-4: el prefijo del `permalink` del archivo es `[kb] name`. Solo
    // `--apply` lo escribe; el dry-run no necesita config (sirve sobre una
    // KB ajena sin `~/.exo`). Sin nombre resoluble, `--apply` falla ANTES de
    // tocar disco: "sin defaults inventados" (config.rs), igual que `write new`.
    let nombre_kb = if args.apply {
        exo::nombre_kb().context(
            "rotate --apply necesita `[kb] name` en la config para el permalink del archivo",
        )?
    } else {
        String::new()
    };

    let dir_log = kb.join("log");
    // Un directorio `x.md/` dentro de `log/` no es una nota — se salta en
    // silencio, igual que kbx (`if e.IsDir() || filepath.Ext(...) != ".md"
    // { continue }`), no cuenta como fallo de la barrida.
    let mut rutas: Vec<PathBuf> = match std::fs::read_dir(&dir_log) {
        Ok(e) => e
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| !p.is_dir() && p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e).with_context(|| format!("leer {}", dir_log.display())),
    };
    rutas.sort();

    let mut resultados = Vec::new();
    let mut fallidas = Vec::new();
    for ruta_abs in rutas {
        let rel = format!("log/{}", ruta_abs.file_name().unwrap().to_string_lossy());
        let contenido = match std::fs::read(&ruta_abs) {
            Ok(c) => c,
            Err(e) => {
                fallidas.push(format!("{rel}: {e}"));
                continue;
            }
        };
        if exo::frontmatter::tier(&String::from_utf8_lossy(&contenido)) != "log" {
            continue;
        }
        match exo::rotacion::aplica(&kb, &rel, args.presupuesto_caliente, args.apply, &nombre_kb) {
            Ok(r) => {
                if r.rotado {
                    resultados.push(r);
                }
            }
            Err(e) => fallidas.push(format!("{rel}: {e}")),
        }
    }

    if args.json {
        envelope::emite(
            "rotate",
            serde_json::json!({ "applied": args.apply, "hot_bytes": args.presupuesto_caliente, "rotations": resultados }),
        );
    } else if resultados.is_empty() {
        println!("rotate: nothing to rotate");
    } else {
        let verbo = if args.apply { "moved" } else { "would move" };
        for r in &resultados {
            println!(
                "{}: {verbo} {} B ({} entries) -> {}",
                r.nota,
                r.bytes_movidos,
                r.entradas_frias,
                r.archivo.as_deref().unwrap_or("")
            );
        }
    }

    if !fallidas.is_empty() {
        for f in &fallidas {
            eprintln!("rotate: {f}");
        }
        anyhow::bail!("{} nota(s) fallaron durante la barrida", fallidas.len());
    }
    Ok(())
}

fn corre(nombre: &str, args: ArgsIndex, borra_antes: bool) -> Result<()> {
    let kb = resuelve_kb(args.kb)?;
    let db = resuelve_db(args.db)?;

    if borra_antes && db.exists() {
        std::fs::remove_file(&db)
            .with_context(|| format!("borrar {} antes de rebuild", db.display()))?;
    }

    let resumen = indexa(&kb, &db)?;

    if args.json {
        envelope::emite(nombre, serde_json::to_value(&resumen)?);
    } else {
        eprintln!(
            "{nombre}: indexadas={} saltadas={} borradas={}",
            resumen.indexadas, resumen.saltadas, resumen.borradas
        );
    }
    Ok(())
}
