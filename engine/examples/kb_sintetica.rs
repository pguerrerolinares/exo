//! KB sintética para el bench de coste de la campaña A (y el oráculo de
//! coste de la Ola 1 G, Task 1: el arm vector necesita contenido correlado
//! para no ser ciego al umbral, backlog:1046-1057).
//!
//! Pre-registro: `docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md`.
//! La FORMA copia la de la KB real medida el 2026-09-13 (174 notas, 3.290
//! trozos de 617 caracteres de media, 727 aristas con 24 sin resolver, tiers
//! 6 core / 60 stable / 105 log). El TEXTO sigue siendo sintético (24
//! palabras de vocabulario sin significado real) — mide coste, no calidad
//! de retrieval — pero los VECTORES ya no son pseudoaleatorios: cada trozo
//! usa el embedding REAL de su palabra de vocabulario dominante (pool de 24
//! embeddings, calculado UNA VEZ con el modelo real, nunca por trozo) más
//! ruido gaussiano `KB_SINTETICA_SIGMA` (default 0.05, ver abajo),
//! renormalizado a norma unidad. Antes, un vector puramente aleatorio en
//! 768 dims tiene similitud coseno esperada ~0 con cualquier query, así que
//! con el umbral de producción (0.40) el arm vector nunca aportaba nada al
//! bench.
//!
//! **Por qué 0.05 y no un sigma más alto — calibración EMPÍRICA, no una
//! fórmula cerrada (fix de review sobre el fix del orquestador, 2):**
//! recon medido (`pool_de_vocabulario()`, 100.000 muestras de
//! `Xorshift::normal()`, coseno sobre 24 palabras × 20 pares
//! limpio/ruidoso): los vectores del pool son unitarios (norma
//! 0.999999-1.000001), `normal()` es N(0,1) por componente (media
//! muestral -0,0044, varianza 1,0016) y el shrinkage de UN solo trozo
//! ruidoso frente a su propio vector limpio (coseno medio medido 0,5845)
//! coincide casi exacto con la fórmula cerrada para ruido gaussiano
//! ortogonal en expectativa a la señal en alta dimensión,
//! `1/sqrt(1+768·sigma²)` (0,5852 para sigma=0,05). Esa fórmula, sin
//! embargo, **no predice el techo real del bench**: aplicada
//! ingenuamente al techo sin ruido (0,4747 × 0,585 ≈ 0,28) da un valor
//! muy por debajo del máximo medido end-to-end sobre la KB completa
//! (0,4230, N=174) — la búsqueda real agrega por MaxP entre ~19 trozos
//! por nota, y los embeddings de las 24 palabras del vocabulario NO son
//! ortogonales entre sí ni con la query (anisotropía típica de embeddings
//! de frases cortas: coseno medio limpio entre pares de palabras del pool
//! medido en 0,223, lejos de 0) — agregación + geometría real sin forma
//! cerrada simple. Por eso sigma se calibra por SWEEP empírico sobre el
//! pipeline completo (`search --type vector`, N=174, la misma query que
//! usa `bench.sh`), no por la fórmula de shrinkage de un solo trozo:
//!
//! | sigma | max coseno | notas ≥0.40 / 174 |
//! |---|---|---|
//! | 0 (sin ruido) | 0.4747 | 149 |
//! | 0.02 | 0.4668 | 110 |
//! | 0.03 | 0.4514 | 81 |
//! | 0.04 | 0.4361 | 64 |
//! | 0.05 (elegido) | 0.4230 | 25 |
//! | 0.06 | 0.4120 | 8 |
//! | 0.07 | 0.4031 | 1 |
//! | 0.08 | 0.3957 | 0 |
//!
//! 0.05 deja margen sobre 0.40 (0.4230) y discrimina de verdad — 25/174
//! por encima, 149 por debajo — ni "ciego" (0 siempre, como 0.08) ni
//! "trivial" (todo pasa, como sigma=0 con 149/174; ver el check de
//! saturación en `bench.sh`, que falla si el corpus entero cruza 0.40).
//! Si el gate de `bench.sh` empieza a fallar sin que nadie haya tocado
//! este fichero (bump del modelo de embeddings o de la plataforma cambia
//! la geometría del espacio), re-correr este sweep antes de subir o bajar
//! sigma a ciegas.
//!
//! Uso: kb_sintetica <N> <DIR> [SEMILLA]  (env `KB_SINTETICA_SIGMA` opcional)
use anyhow::{Context, Result, bail};
use exo::con_embedder_de_proceso;
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

    /// Normal estándar vía Box-Muller (forma polar simple), determinista con
    /// la misma semilla. `.max(1e-6)` evita `ln(0)` en el caso borde
    /// (probabilidad ~1/2^24 por muestra) sin añadir un segundo generador.
    fn normal(&mut self) -> f32 {
        let u1 = ((self.unidad() + 1.0) / 2.0).max(1e-6);
        let u2 = (self.unidad() + 1.0) / 2.0;
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
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

/// Palabra del vocabulario más frecuente en `texto` — decide qué vector del
/// pool real usa este trozo. La query del bench («trinquete techos indice
/// memoria») también son solo palabras del vocabulario, así que compartir
/// este criterio es lo que hace que query y contenido caigan en la MISMA
/// región del espacio de embeddings real.
fn palabra_dominante(texto: &str) -> usize {
    let mut cuentas = [0usize; VOCABULARIO.len()];
    for palabra in texto.split_whitespace() {
        if let Some(i) = VOCABULARIO.iter().position(|v| *v == palabra) {
            cuentas[i] += 1;
        }
    }
    cuentas
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| **c)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Embeddings REALES de las 24 palabras del vocabulario — una sola pasada
/// de batch (24 textos, no miles). Se calcula UNA VEZ por generación, nunca
/// por trozo: la correlación de contenido que faltaba (backlog:1046-1057)
/// sin pagar el coste de embeber la KB sintética entera.
fn pool_de_vocabulario() -> Result<Vec<Vec<f32>>> {
    let textos: Vec<String> = VOCABULARIO.iter().map(|w| w.to_string()).collect();
    con_embedder_de_proceso(|embedder| embedder.embebe_batch(&textos))
        .context("embed del pool de vocabulario para kb_sintetica")
}

/// Vector "real + ruido": embedding real de la palabra dominante del trozo
/// más ruido gaussiano N(0, sigma²) por componente, renormalizado a norma
/// unidad. `sigma=0` es el embedding real sin ruido; `sigma` alto lo acerca
/// al `vector_unitario()` puramente aleatorio de antes de esta task.
fn vector_desde_pool(pool: &[Vec<f32>], idx: usize, rng: &mut Xorshift, sigma: f32) -> Vec<f32> {
    let base = &pool[idx];
    let mut v: Vec<f32> = base.iter().map(|x| x + sigma * rng.normal()).collect();
    let norma = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norma > 1e-9 {
        for x in &mut v {
            *x /= norma;
        }
    }
    v
}

/// Construye el índice como lo dejaría `exo index`, salvo el TEXTO (sigue
/// siendo sintético). Devuelve (trozos, aristas, aristas sin resolver).
fn construye_indice(
    kb: &Path,
    db: &Path,
    n: usize,
    rng: &mut Xorshift,
    pool: &[Vec<f32>],
    sigma: f32,
) -> Result<(i64, i64, i64)> {
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
            let idx = palabra_dominante(texto);
            exo::vectores::inserta(&tx, id, &vector_desde_pool(pool, idx, rng, sigma))?;
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
    // Default 0.05, no 0.5 (Task 1 original): con sigma=0.5 el ruido
    // domina la señal del pool real y el arm vector vuelve a ser ciego al
    // umbral de producción (0.40) — ver doc-comment del módulo, "fix del
    // orquestador (2)".
    let sigma: f32 = std::env::var("KB_SINTETICA_SIGMA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.05);
    let pool = pool_de_vocabulario().context("pool de embeddings reales del vocabulario")?;
    let (trozos, aristas, rotas) = construye_indice(&kb, &db, n, &mut rng, &pool, sigma)?;
    println!(
        "kb_sintetica: N={n} semilla={semilla} sigma={sigma} trozos={trozos} aristas={aristas} sin_resolver={rotas}"
    );
    Ok(())
}
