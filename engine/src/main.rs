use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use exo::{
    buscador::{busca, busca_hybrid, busca_vector},
    envelope,
    escritor::{escribe_append, escribe_nueva},
    indexer::indexa,
    kb_desde_config,
    recall::{recall_arranque, recall_consulta, renderiza, resuelve_rutas_absolutas},
};
use std::path::PathBuf;

/// Defaults SELLADOS del arm hybrid (M2-07, §5.2.6 de la spec de fusión):
/// ganadores del sweep 15+1 corridas (grid bonus{0,0.1,0.2,0.3,0.5}×
/// β{0.6,0.8,1.0} + diagnóstica A, `reports/m2-07-impl-report.md`) —
/// selección pre-registrada §5.2.4 (max hit@5=49/55 → 4 celdas empatadas en
/// β=0.6 → menor bonus=0.0), confirmada nativa (§5.2.5, `--min-similitud
/// 0.40` da 49/55 idéntico al post-hoc). Cubren SOLO el uso de `exo search
/// --type hybrid` sin `--bonus`/`--escala-fts` explícitos; el sweep siempre
/// pasó ambos flags, así que estos valores no afectaron su resultado. El
/// threshold ganador (0.40) NO se sella aquí como constante — D-f3/§4.6: el
/// valor difiere del 0.35 de config y config es RO hasta M5a, así que se
/// pasa por `--min-similitud 0.40` explícito en corridas/consumidores hasta
/// entonces (documentado en el verdict, no hardcodeado en el binario).
const BONUS_SELLADO: f64 = 0.0;
const ESCALA_FTS_SELLADA: f64 = 0.6;

#[derive(Parser)]
#[command(name = "exo", version, about = "engine del framework exo (E1: read)")]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Crea `~/.exo/config.toml`. Con `--from-basic-memory`, migra los valores
    /// de `~/.basic-memory/config.json` una sola vez.
    Init(ArgsInit),
    /// Indexa la KB de forma incremental (mtime al invocar, sin daemon).
    Index(ArgsIndex),
    /// Borra la DB y reconstruye desde cero (primera clase, no cirugía —
    /// spec §3: "corrupción de índice = borrar y rebuild").
    Rebuild(ArgsIndex),
    /// Búsqueda FTS5 mínima sobre `notas_fts` (spec §4.1, m2-05).
    Search(ArgsSearch),
    /// Escribe en la KB (M4/E2): nota nueva o append a bitácora. File-first,
    /// sin commit y sin indexar — eso es del agente y del recall siguiente.
    #[command(subcommand)]
    Write(ComandoWrite),
    /// Sirve contenido de la KB para arranque (`tier: core` + recientes) o
    /// consulta (`busca_hybrid`) — sucesor de `basic-memory-recall.sh` y
    /// `compose-inject.sh` de reflex (M2-08, M6). NO conoce reflex ni
    /// perfiles de agentes: eso lo compone el consumidor.
    Recall(ArgsRecall),
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
    #[arg(long)]
    from_basic_memory: bool,
    /// Sobreescribe una config existente.
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsWriteNew {
    /// Fichero SQLite del índice (mismo contrato D6 que el resto). Lo usa el
    /// dup-gate; con `--force` no se consulta.
    #[arg(long)]
    db: PathBuf,
    /// Raíz de la KB. Default: `[kb] path` de `~/.exo/config.toml`.
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Directorio destino dentro de la KB (`projects`, `log`, `research`…).
    #[arg(long)]
    dir: String,
    /// Título de la nota. De él salen el nombre de fichero y el slug del
    /// permalink.
    #[arg(long)]
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
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ArgsWriteAppend {
    #[arg(long)]
    db: PathBuf,
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Fichero con el texto a anexar (`-` = stdin).
    #[arg(long)]
    from: String,
    /// Crea la bitácora si no existe (documenta.md la pide con `tier: log`).
    #[arg(long)]
    crea: bool,
    /// Anexa aunque el destino no sea `tier: log`. Queda registrado en el
    /// envelope (`forzado: true`) para que la excepción sea auditable.
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
    /// Permalink de la nota destino (p.ej. `kb-demo/log/exo-bitacora`).
    permalink: String,
}

#[derive(clap::Args)]
struct ArgsIndex {
    /// Fichero SQLite del índice. Obligatorio, sin default (D6).
    #[arg(long)]
    db: PathBuf,
    /// Raíz de la KB. Por defecto, `[kb] path` de `~/.exo/config.toml`.
    /// Precedencia: flag > config (D6).
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON (spec §4) en stdout.
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
    /// Fichero SQLite del índice. Obligatorio, sin default (D6: un default
    /// sería config encubierta).
    #[arg(long)]
    db: PathBuf,
    /// Máximo de resultados. Default 10 (replay-engine pasa el suyo
    /// explícito; flags > config).
    #[arg(long, default_value_t = 10)]
    limite: usize,
    /// Tipo de búsqueda (fts|vector|hybrid, M2-07). Default `fts`:
    /// comportamiento actual intacto si no se pasa el flag.
    #[arg(long, value_enum, default_value_t = TipoBusqueda::Fts)]
    r#type: TipoBusqueda,
    /// Umbral de similitud coseno del arm vector/hybrid. Opcional: si se
    /// omite, cae a `[embeddings] min_similarity` de `~/.exo/config.toml`
    /// (D6, precedencia flags > config). Sin efecto en `--type fts`.
    #[arg(long)]
    min_similitud: Option<f64>,
    /// Peso del canal débil en la fórmula de fusión (`bonus·min(v,f)`,
    /// spec fusión §4.4). Solo para `--type hybrid`: override puntual del
    /// sellado (M2-07, §5.2.6); si se omite, cae al default sellado
    /// `BONUS_SELLADO`.
    #[arg(long)]
    bonus: Option<f64>,
    /// Anclaje β de la normalización BM25 por-query (spec fusión §4.3,
    /// D-f1). Solo para `--type hybrid`: override puntual del sellado
    /// (M2-07, §5.2.6); si se omite, cae al default sellado
    /// `ESCALA_FTS_SELLADA`.
    #[arg(long)]
    escala_fts: Option<f64>,
    /// Emite el resultado como envelope JSON (spec §4) en stdout.
    #[arg(long)]
    json: bool,
    /// Texto de la consulta.
    query: String,
}

#[derive(clap::Args)]
struct ArgsRecall {
    /// Fichero SQLite del índice. Obligatorio, sin default (D6, mismo
    /// contrato que `index`/`search`).
    #[arg(long)]
    db: PathBuf,
    /// Raíz de la KB. Por defecto, `[kb] path` de `~/.exo/config.toml`.
    /// `exo recall` la necesita aunque
    /// solo lea del índice: `notas.ruta` es relativa, y modo arranque
    /// también relee `tier` del `.md` en disco (no está en el índice).
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Texto de la consulta. Ausente ⇒ modo arranque (`tier: core` +
    /// recientes por git); presente ⇒ modo consulta (`busca_hybrid`).
    #[arg(long)]
    query: Option<String>,
    /// Máximo de notas. En modo arranque, tope del bloque de "recientes"
    /// (los `tier: core` siempre entran todos); en modo consulta, tope de
    /// `busca_hybrid`. Default 5 (contrato del brief para modo consulta;
    /// mismo flag, mismo default en ambos modos).
    #[arg(long, default_value_t = 5)]
    limite: usize,
    /// Presupuesto de bytes del bloque de salida (texto o `--json`), trunca
    /// por líneas ENTERAS. Default 2048 (brief).
    #[arg(long, default_value_t = 2048)]
    cap_bytes: usize,
    /// Umbral de similitud coseno del arm vector de `busca_hybrid` (modo
    /// consulta). Sin efecto en modo arranque. Default de config si se
    /// omite (D6, mismo contrato que `search`).
    #[arg(long)]
    min_similitud: Option<f64>,
    /// Modo arranque en versión CONTENIDO: vuelca el cuerpo de las notas
    /// `tier: core` + lista de recientes, en vez de una línea por nota. Es
    /// lo que consume el hook de SessionStart (paridad con el
    /// `basic-memory-recall.sh` que sustituye, que inyectaba el cuerpo del
    /// core-index, no sus rutas). Incompatible con `--query`.
    #[arg(long)]
    contenido: bool,
    /// Permalink de la nota cuyo cuerpo se quiere en `--contenido` (p.ej.
    /// `core/core-index`). Sin este flag, `--contenido` vuelca TODAS las
    /// `tier: core` — que en una KB con un core grande agota el presupuesto
    /// con la primera. Qué nota es "la de arranque" lo decide el consumidor,
    /// no el engine.
    #[arg(long)]
    nota: Option<String>,
    /// Refresca el índice (indexado incremental) ANTES de servir, para no
    /// devolver un bloque de una KB rancia (M6-01, "índice fresco sin
    /// daemon"). Barato cuando nada cambió: un `stat` por fichero y ninguna
    /// carga del modelo. Si la DB no existe, la construye (bootstrap).
    #[arg(long)]
    refresca: bool,
    /// Emite el resultado como envelope JSON (spec §4) en stdout. Sin este
    /// flag, imprime un bloque de texto plano (el que consumirá el hook).
    #[arg(long)]
    json: bool,
}

fn main() {
    if let Err(e) = ejecuta() {
        // Un gate rechazado NO es un error del sistema: es una decisión que se
        // le devuelve al llamador (nota duplicada, append al canon). Sale con
        // 3 para que el consumidor lo distinga de un fallo real por exit code
        // —jamás parseando `data`— y pueda reintentar con `--force`.
        if let Some(rechazo) = e.downcast_ref::<exo::escritor::Rechazo>() {
            eprintln!("rechazado: {rechazo}");
            std::process::exit(3);
        }
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn ejecuta() -> Result<()> {
    let cli = Cli::parse();
    match cli.comando {
        Comando::Init(args) => init_cmd(args),
        Comando::Index(args) => corre("index", args, false),
        Comando::Rebuild(args) => corre("rebuild", args, true),
        Comando::Search(args) => busca_cmd(args),
        Comando::Recall(args) => recall_cmd(args),
        Comando::Write(sub) => match sub {
            ComandoWrite::New(args) => write_new_cmd(args),
            ComandoWrite::Append(args) => write_append_cmd(args),
        },
    }
}

/// `exo init`: escribe la config propia. El volcado de la KB semilla llega en
/// G3 y se engancha aquí sin cambiar esta firma.
fn init_cmd(args: ArgsInit) -> Result<()> {
    let destino = exo::config::ruta_config()?;
    let db_default = dirs::home_dir().context("sin HOME")?.join(".exo/index.db");

    let (kb, nombre, emb) = if args.from_basic_memory {
        let ruta = exo::inicia::ruta_basic_memory()?;
        let json =
            std::fs::read_to_string(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
        exo::inicia::desde_basic_memory(&json)?
    } else {
        let kb = args
            .kb
            .context("--kb es obligatorio sin --from-basic-memory")?;
        let nombre = args
            .name
            .context("--name es obligatorio sin --from-basic-memory")?;
        // Defaults del modelo de producción: los mismos que la línea base del
        // eval, declarados en la spec como posicionamiento (producto en español).
        let emb = exo::config::Embeddings {
            model: "jinaai/jina-embeddings-v2-base-es".to_string(),
            dims: 768,
            min_similarity: 0.35,
        };
        (kb, nombre, emb)
    };

    exo::inicia::escribe_config(&destino, &kb, &nombre, &emb, &db_default, args.force)?;

    if args.json {
        exo::envelope::emite(
            "init",
            serde_json::json!({
                "config": destino.display().to_string(),
                "kb": kb.display().to_string(),
                "name": nombre,
                "from_basic_memory": args.from_basic_memory,
            }),
        );
    } else {
        println!("config escrita en {}", destino.display());
        println!("KB: {} (name: {nombre})", kb.display());
        println!("siguiente: exo index --json");
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
    let kb = match args.kb {
        Some(p) => p,
        None => kb_desde_config().context("resolver raíz de la KB (--kb ausente)")?,
    };
    let cuerpo = lee_from(&args.from)?;

    // El nombre del proyecto es el primer segmento del permalink de la KB
    // (`kb-demo/projects/…`). Sale del directorio raíz, NO hardcodeado:
    // requisito C11 del plan (no cerrar la puerta a otras instancias).
    let proyecto = kb
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .context("la raíz de la KB no tiene nombre de directorio")?;

    // Dup-gate por solape de slug, NO por retrieval: ver `solape_slug`.
    // Barato y determinista — no carga el modelo de embeddings, así que el
    // cierre de sesión no paga segundos por esta comprobación.
    let candidatas: Vec<(String, f64)> = if args.force {
        Vec::new()
    } else {
        let indexados =
            exo::buscador::permalinks(&args.db).context("dup-gate: leer permalinks del índice")?;
        exo::escritor::dup_candidatas(&exo::escritor::slug(&args.titulo), &indexados)
    };

    let esc = escribe_nueva(
        &kb,
        &proyecto,
        &args.dir,
        &args.titulo,
        &cuerpo,
        args.tier.as_deref(),
        &candidatas,
        args.force,
    )?;

    emite_escritura(esc, args.json);
    Ok(())
}

/// `exo write append`: resuelve permalink→ruta contra el índice y anexa. Con
/// `--crea`, una bitácora que no existe se crea en vez de fallar.
fn write_append_cmd(args: ArgsWriteAppend) -> Result<()> {
    let kb = match args.kb {
        Some(p) => p,
        None => kb_desde_config().context("resolver raíz de la KB (--kb ausente)")?,
    };
    let texto = lee_from(&args.from)?;

    let ruta_rel = match exo::buscador::ruta_de(&args.db, &args.permalink)? {
        Some(r) => r,
        None if args.crea => {
            // Sin fila en el índice: o la bitácora no existe, o el índice está
            // rancio. Derivar la ruta del permalink es correcto SOLO para
            // crearla (`log/<slug>.md`); para una nota ya existente el slug no
            // es invertible y por eso jamás se adivina.
            let (dir, slug_nota) = args
                .permalink
                .rsplit_once('/')
                .map(|(izq, slug)| (izq.rsplit_once('/').map_or(izq, |(_, d)| d), slug))
                .context("permalink sin forma <proyecto>/<dir>/<slug>")?;
            let rel = format!("{dir}/{slug_nota}.md");

            if !kb.join(&rel).exists() {
                let proyecto = kb
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .context("la raíz de la KB no tiene nombre de directorio")?;
                escribe_nueva(&kb, &proyecto, dir, slug_nota, "", Some("log"), &[], false)
                    .context("crear la bitácora con --crea")?;
                eprintln!("write: bitácora creada en {rel}");
            }
            rel
        }
        None => anyhow::bail!(
            "{} no está en el índice: comprueba el permalink, o usa --crea si la bitácora aún no existe",
            args.permalink
        ),
    };

    let esc = escribe_append(&kb, &ruta_rel, &texto, args.force)?;
    emite_escritura(esc, args.json);
    Ok(())
}

/// Salida común de `write`: envelope v1 por stdout con `--json`, o una línea
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
    let kb = match args.kb {
        Some(p) => p,
        None => kb_desde_config().context("resolver raíz de la KB (--kb ausente)")?,
    };

    if args.refresca {
        // El resumen va a stderr: stdout es exclusivo del envelope/bloque
        // (contrato §4), y el hook consume stdout tal cual.
        let resumen = exo::refresca_indice(&kb, &args.db)
            .context("refrescar el índice antes del recall (--refresca)")?;
        if resumen.indexadas > 0 || resumen.borradas > 0 {
            eprintln!(
                "refresca: indexadas={} borradas={} saltadas={}",
                resumen.indexadas, resumen.borradas, resumen.saltadas
            );
        }
    }

    if args.contenido {
        if args.query.is_some() {
            anyhow::bail!("--contenido es del modo arranque: no se combina con --query");
        }
        // Camino del hook: bloque de texto a stdout y fuera. No pasa por el
        // envelope ni por `aplica_cap` (trae su propio truncado por líneas).
        let bloque = exo::recall::recall_arranque_contenido(
            &args.db,
            &kb,
            args.limite,
            args.cap_bytes,
            args.nota.as_deref(),
        )?;
        print!("{bloque}");
        return Ok(());
    }

    let bruto = match &args.query {
        None => recall_arranque(&args.db, &kb, args.limite)?,
        Some(q) => {
            let mut bruto = recall_consulta(
                &args.db,
                q,
                args.limite,
                args.min_similitud,
                BONUS_SELLADO,
                ESCALA_FTS_SELLADA,
            )?;
            resuelve_rutas_absolutas(&mut bruto, &kb);
            bruto
        }
    };

    let resultado = renderiza(bruto, args.cap_bytes);

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
    let resultado = match args.r#type {
        TipoBusqueda::Fts => busca(&args.db, &args.query, args.limite)?,
        TipoBusqueda::Vector => {
            busca_vector(&args.db, &args.query, args.limite, args.min_similitud)?
        }
        TipoBusqueda::Hybrid => busca_hybrid(
            &args.db,
            &args.query,
            args.limite,
            args.min_similitud,
            args.bonus.unwrap_or(BONUS_SELLADO),
            args.escala_fts.unwrap_or(ESCALA_FTS_SELLADA),
        )?,
    };

    // Los avisos van a stderr SIEMPRE, con o sin `--json`: nunca contaminan el
    // envelope de stdout, y quien mira la terminal ve la degradación sin
    // tener que parsear nada. Un instrumento degradado que no grita es el
    // modo de fallo caro que este campo existe para matar.
    for aviso in &resultado.avisos {
        eprintln!("aviso: {aviso}");
    }

    if args.json {
        envelope::emite("search", serde_json::to_value(&resultado)?);
    } else {
        for r in &resultado.results {
            println!("{}\t{}\t{:.4}", r.permalink, r.tipo, r.score);
        }
    }
    Ok(())
}

fn corre(nombre: &str, args: ArgsIndex, borra_antes: bool) -> Result<()> {
    let kb = match args.kb {
        Some(p) => p,
        None => kb_desde_config().context("resolver raíz de la KB (--kb ausente)")?,
    };

    if borra_antes && args.db.exists() {
        std::fs::remove_file(&args.db)
            .with_context(|| format!("borrar {} antes de rebuild", args.db.display()))?;
    }

    let resumen = indexa(&kb, &args.db)?;

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
