//! KB sintética para el bench de coste de la campaña A.
//!
//! Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`.
//! La FORMA copia la de la KB real medida el 2026-09-13 (174 notas, 3.290
//! trozos de 617 caracteres de media, 727 aristas con 24 sin resolver, tiers
//! 6 core / 60 stable / 105 log). El índice se construye SIN el modelo: los
//! vectores son pseudoaleatorios y deterministas. Mide coste, no calidad.
//!
//! Uso: kb_sintetica <N> <DIR> [SEMILLA]
use anyhow::{Context, Result, bail};
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

/// 19 párrafos de ≥580 caracteres: dos juntos pasan de 900 (`trozos::MAX_CHARS`),
/// así que `trocea` da un trozo por párrafo — 19 por nota (3.290/174 = 18,9).
const PARRAFOS_POR_NOTA: usize = 19;
const CHARS_POR_PARRAFO: usize = 580;
/// 727/174 = 4,18 wikilinks por nota.
const ENLACES_POR_NOTA: usize = 4;
/// 24/727 ≈ 1/30 aristas sin resolver.
const UNO_ROTO_CADA: usize = 30;
const EPOCH_COMMIT: i64 = 1_780_000_000;
const NOMBRE_KB: &str = "sint";
const VOCABULARIO: [&str; 24] = [
    "memoria",
    "indice",
    "trinquete",
    "techos",
    "bitacora",
    "canon",
    "recall",
    "prompt",
    "hook",
    "engine",
    "nota",
    "presupuesto",
    "sesion",
    "agente",
    "orquestador",
    "doctrina",
    "permalink",
    "wikilink",
    "trozo",
    "vector",
    "umbral",
    "gate",
    "campana",
    "fabrica",
];

struct Xorshift(u64);

impl Xorshift {
    fn siguiente(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn indice(&mut self, n: usize) -> usize {
        (self.siguiente() % n as u64) as usize
    }

    /// Uniforme en [-1, 1).
    fn unidad(&mut self) -> f32 {
        (self.siguiente() >> 40) as f32 / (1u64 << 23) as f32 - 1.0
    }
}

/// 1/29 core (3,4%), 10/29 stable (34,5%), 18/29 log (62,1%).
fn carpeta_y_tier(i: usize) -> (&'static str, &'static str) {
    match i % 29 {
        0 => ("core", "core"),
        1..=10 => ("projects", "stable"),
        _ => ("log", "log"),
    }
}

fn titulo(i: usize) -> String {
    if i == 0 {
        "core-index".to_string()
    } else {
        format!("nota-{i:05}")
    }
}

fn ruta_rel(i: usize) -> String {
    format!("{}/{}.md", carpeta_y_tier(i).0, titulo(i))
}

fn parrafo(rng: &mut Xorshift) -> String {
    let mut p = String::with_capacity(CHARS_POR_PARRAFO + 16);
    while p.len() < CHARS_POR_PARRAFO {
        if !p.is_empty() {
            p.push(' ');
        }
        p.push_str(VOCABULARIO[rng.indice(VOCABULARIO.len())]);
    }
    p
}

fn cuerpo(i: usize, n: usize, rng: &mut Xorshift) -> String {
    let enlaces: Vec<String> = (0..ENLACES_POR_NOTA)
        .map(|k| {
            if (i * ENLACES_POR_NOTA + k).is_multiple_of(UNO_ROTO_CADA) {
                format!("[[inexistente-{i}-{k}]]")
            } else {
                format!("[[{}]]", titulo(rng.indice(n)))
            }
        })
        .collect();
    let mut texto = format!("# {}\n\n", titulo(i));
    if i == 0 {
        // `exo-recall.sh` exige esta frase para aceptar el bloque de arranque.
        texto.push_str("Contrato de memoria: canon como delta, bitacora como append.\n\n");
    }
    for p in 0..PARRAFOS_POR_NOTA {
        if p > 0 {
            texto.push_str("\n\n");
        }
        texto.push_str(&parrafo(rng));
        if p == 0 {
            texto.push(' ');
            texto.push_str(&enlaces.join(" "));
        }
    }
    texto.push('\n');
    texto
}

fn nota_md(i: usize, n: usize, rng: &mut Xorshift) -> String {
    let (carpeta, tier) = carpeta_y_tier(i);
    let t = titulo(i);
    format!(
        "---\ntitle: {t}\ntype: note\npermalink: {NOMBRE_KB}/{carpeta}/{t}\ntier: {tier}\n---\n\n{}",
        cuerpo(i, n, rng)
    )
}

fn versiona(kb: &Path) -> Result<()> {
    let fecha = format!("{EPOCH_COMMIT} +0000");
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.name=bench",
            "-c",
            "user.email=bench@exo.invalid",
            "commit",
            "-q",
            "-m",
            "kb sintetica",
        ],
    ] {
        let st = Command::new("git")
            .arg("-C")
            .arg(kb)
            .args(&args)
            .env("GIT_AUTHOR_DATE", &fecha)
            .env("GIT_COMMITTER_DATE", &fecha)
            .status()
            .with_context(|| format!("git {args:?}"))?;
        if !st.success() {
            bail!("git {args:?} falló en {}", kb.display());
        }
    }
    Ok(())
}

fn escribe_config(dir: &Path, kb: &Path, db: &Path) -> Result<()> {
    let barras = |p: &Path| p.display().to_string().replace('\\', "/");
    std::fs::write(
        dir.join("config.toml"),
        format!(
            "schema_version = 1\n\n[kb]\npath = \"{}\"\nname = \"{NOMBRE_KB}\"\n\n\
             [index]\ndb = \"{}\"\n\n[embeddings]\nmodel = \"{}\"\ndims = 768\n\
             min_similarity = 0.35\n",
            barras(kb),
            barras(db),
            exo::MODELO_JINA_ES
        ),
    )
    .context("escribir config.toml")
}

fn vector_unitario(rng: &mut Xorshift) -> Vec<f32> {
    let mut v: Vec<f32> = (0..768).map(|_| rng.unidad()).collect();
    let norma = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut v {
        *x /= norma;
    }
    v
}

/// Construye el índice como lo dejaría `exo index`, salvo los vectores.
/// Devuelve (trozos, aristas, aristas sin resolver).
fn construye_indice(kb: &Path, db: &Path, n: usize, rng: &mut Xorshift) -> Result<(i64, i64, i64)> {
    let conn = exo::abre_db(db)?;
    exo::schema::crea_schema(&conn)?;
    let kb_abs = std::fs::canonicalize(kb).context("canonicalizar kb")?;
    for (clave, valor) in [
        ("kb_root", kb_abs.to_string_lossy().into_owned()),
        ("modelo_embeddings", exo::MODELO_JINA_ES.to_string()),
        ("dims_embeddings", "768".to_string()),
    ] {
        conn.execute(
            "INSERT INTO meta (clave, valor) VALUES (?1, ?2)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![clave, valor],
        )?;
    }
    let tx = conn.unchecked_transaction()?;
    for i in 0..n {
        let rel = ruta_rel(i);
        let abs = kb.join(&rel);
        // El MISMO parser que el indexer: título, tipo y cuerpo idénticos a `exo index`.
        let nota = exo::nota::parsea_nota(&abs)?.with_context(|| format!("{rel} sin permalink"))?;
        // Misma fórmula que `indexer::mtime_de`: si no cuadra al bit, `exo index`
        // reindexaría la nota y el bench mediría un indexado.
        let mtime = std::fs::metadata(&abs)?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs_f64();
        tx.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                nota.permalink,
                rel,
                nota.titulo,
                nota.tipo,
                mtime,
                EPOCH_COMMIT
            ],
        )?;
        tx.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            params![nota.titulo, nota.cuerpo, nota.permalink],
        )?;
        exo::aristas::reindexa_aristas_de_nota(&tx, &nota.permalink, &nota.cuerpo)?;
        for (orden, texto) in exo::trozos::trocea(&nota.cuerpo).iter().enumerate() {
            tx.execute(
                "INSERT INTO trozos (permalink, orden, texto) VALUES (?1, ?2, ?3)",
                params![nota.permalink, orden as i64, texto],
            )?;
            let id = tx.last_insert_rowid();
            exo::vectores::inserta(&tx, id, &vector_unitario(rng))?;
        }
    }
    tx.commit()?;
    exo::aristas::resuelve_destinos(&conn)?;
    let cuenta = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0));
    Ok((
        cuenta("SELECT count(*) FROM trozos")?,
        cuenta("SELECT count(*) FROM aristas")?,
        cuenta("SELECT count(*) FROM aristas WHERE destino_permalink IS NULL")?,
    ))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        bail!("uso: kb_sintetica <N> <DIR> [SEMILLA]");
    }
    let n: usize = args[1].parse().context("N debe ser un entero")?;
    if n == 0 {
        bail!("N debe ser > 0");
    }
    let dir = PathBuf::from(&args[2]);
    let semilla: u64 = match args.get(3) {
        Some(s) => s.parse().context("SEMILLA debe ser un entero")?,
        None => 42,
    };
    if dir.exists() {
        bail!("{} ya existe: el generador no pisa nada", dir.display());
    }
    let kb = dir.join("kb");
    let db = dir.join("index.db");
    let mut rng = Xorshift(semilla.max(1));

    for carpeta in ["core", "projects", "log"] {
        std::fs::create_dir_all(kb.join(carpeta))?;
    }
    for i in 0..n {
        std::fs::write(kb.join(ruta_rel(i)), nota_md(i, n, &mut rng))?;
    }
    versiona(&kb)?;
    escribe_config(&dir, &kb, &db)?;
    let (trozos, aristas, rotas) = construye_indice(&kb, &db, n, &mut rng)?;
    println!(
        "kb_sintetica: N={n} semilla={semilla} trozos={trozos} aristas={aristas} sin_resolver={rotas}"
    );
    Ok(())
}
