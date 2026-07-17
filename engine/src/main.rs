use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use exo::{buscador::busca, envelope, indexer::indexa, kb_desde_config};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "exo", version, about = "engine del framework exo (E1: read)")]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Indexa la KB de forma incremental (mtime al invocar, sin daemon).
    Index(ArgsIndex),
    /// Borra la DB y reconstruye desde cero (primera clase, no cirugía —
    /// spec §3: "corrupción de índice = borrar y rebuild").
    Rebuild(ArgsIndex),
    /// Búsqueda FTS5 mínima sobre `notas_fts` (spec §4.1, m2-05).
    Search(ArgsSearch),
}

#[derive(clap::Args)]
struct ArgsIndex {
    /// Fichero SQLite del índice. Obligatorio, sin default (D6).
    #[arg(long)]
    db: PathBuf,
    /// Raíz de la KB. Por defecto, `projects.kb-demo.path` de
    /// `~/.basic-memory/config.json` (RO). Precedencia: flag > config (D6).
    #[arg(long)]
    kb: Option<PathBuf>,
    /// Emite el resultado como envelope JSON (spec §4) en stdout.
    #[arg(long)]
    json: bool,
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
    /// Emite el resultado como envelope JSON (spec §4) en stdout.
    #[arg(long)]
    json: bool,
    /// Texto de la consulta.
    query: String,
}

fn main() {
    if let Err(e) = ejecuta() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn ejecuta() -> Result<()> {
    let cli = Cli::parse();
    match cli.comando {
        Comando::Index(args) => corre("index", args, false),
        Comando::Rebuild(args) => corre("rebuild", args, true),
        Comando::Search(args) => busca_cmd(args),
    }
}

fn busca_cmd(args: ArgsSearch) -> Result<()> {
    let resultado = busca(&args.db, &args.query, args.limite)?;

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
