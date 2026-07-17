use clap::Parser;

#[derive(Parser)]
#[command(name = "exo", version, about = "engine del framework exo (E1: read)")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    // Subcomandos (index/search/recall/rebuild) entran con sus items (M2-03+).
}
