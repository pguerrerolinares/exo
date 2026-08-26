use exo::indexer::indexa;
use exo::recall::{recall_arranque, recall_consulta, renderiza, resuelve_rutas_absolutas};
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("ejecutar git");
    assert!(status.success(), "git {args:?} falló en {dir:?}");
}

/// Commitea `nombre` con contenido `contenido` en un epoch de autor/committer
/// FIJO (mismo patrón que `tests/git_epoch.rs`), para poder controlar el
/// orden de recencia sin depender del reloj real.
fn commitea_con_epoch(dir: &Path, nombre: &str, contenido: &str, epoch: i64) {
    std::fs::write(dir.join(nombre), contenido).unwrap();
    git(dir, &["add", nombre]);
    let fecha = format!("{epoch} +0000");
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["commit", "-q", "-m", &format!("añade {nombre}")])
        .env("GIT_AUTHOR_DATE", &fecha)
        .env("GIT_COMMITTER_DATE", &fecha)
        .status()
        .expect("git commit");
    assert!(status.success());
}

fn nota_md(permalink: &str, titulo: &str, tier: Option<&str>, cuerpo: &str) -> String {
    match tier {
        Some(t) => {
            format!("---\npermalink: {permalink}\ntitle: {titulo}\ntier: {t}\n---\n{cuerpo}\n")
        }
        None => format!("---\npermalink: {permalink}\ntitle: {titulo}\n---\n{cuerpo}\n"),
    }
}

/// Fixture: 2 notas `tier: core` (nombradas para que el orden alfabético de
/// ruta NO coincida con el orden de creación/commit — verifica "orden de
/// ruta estable", no orden de inserción) + 3 notas sin tier con epochs de
/// git crecientes (verifica recencia = git, orden y truncado a `limite`).
fn kb_arranque() -> (tempfile::TempDir, tempfile::TempDir, std::path::PathBuf) {
    let kb = tempfile::tempdir().unwrap();
    git(kb.path(), &["init", "-q"]);
    git(kb.path(), &["config", "user.email", "test@exo.local"]);
    git(kb.path(), &["config", "user.name", "exo-test"]);

    // core-b.md se commitea ANTES que core-a.md: si el orden de salida
    // fuera por tiempo de commit en vez de por ruta, saldría [b, a].
    commitea_con_epoch(
        kb.path(),
        "core-b.md",
        &nota_md(
            "kb-demo/core-b",
            "Core B",
            Some("core"),
            "contenido core b",
        ),
        1_700_000_000,
    );
    commitea_con_epoch(
        kb.path(),
        "core-a.md",
        &nota_md(
            "kb-demo/core-a",
            "Core A",
            Some("core"),
            "contenido core a",
        ),
        1_700_000_001,
    );
    commitea_con_epoch(
        kb.path(),
        "r-viejo.md",
        &nota_md(
            "kb-demo/r-viejo",
            "Reciente viejo",
            None,
            "contenido viejo",
        ),
        1_700_000_002,
    );
    commitea_con_epoch(
        kb.path(),
        "r-medio.md",
        &nota_md(
            "kb-demo/r-medio",
            "Reciente medio",
            None,
            "contenido medio",
        ),
        1_700_000_003,
    );
    commitea_con_epoch(
        kb.path(),
        "r-nuevo.md",
        &nota_md(
            "kb-demo/r-nuevo",
            "Reciente nuevo",
            None,
            "contenido nuevo",
        ),
        1_700_000_004,
    );

    let db_dir = tempfile::tempdir().unwrap();
    let db = db_dir.path().join("exo.db");
    indexa(kb.path(), &db).unwrap();
    (kb, db_dir, db)
}

#[test]
fn recall_arranque_cores_en_orden_de_ruta_y_recientes_por_git_hasta_limite() {
    let (kb, _db_dir, db) = kb_arranque();

    let bruto = recall_arranque(&db, kb.path(), 2).unwrap();
    assert_eq!(bruto.modo, "arranque");
    assert_eq!(bruto.query, None);

    let permalinks: Vec<&str> = bruto.notas.iter().map(|n| n.permalink.as_str()).collect();
    assert_eq!(
        permalinks,
        vec![
            "kb-demo/core-a", // orden de RUTA ("core-a.md" < "core-b.md"), no de commit
            "kb-demo/core-b",
            "kb-demo/r-nuevo", // recientes: 2 más nuevos por git_epoch, viejo excluido por limite=2
            "kb-demo/r-medio",
        ],
        "{:?}",
        bruto.notas
    );

    // cores: tier presente, score/snippet null (modo arranque)
    assert_eq!(bruto.notas[0].tier.as_deref(), Some("core"));
    assert_eq!(bruto.notas[0].score, None);
    assert_eq!(bruto.notas[0].snippet, None);
    // recientes: tier null (no relevante fuera de la sección core)
    assert_eq!(bruto.notas[2].score, None);

    // ruta absoluta, no relativa
    assert!(
        bruto.notas[0]
            .ruta
            .starts_with(&kb.path().display().to_string()),
        "{}",
        bruto.notas[0].ruta
    );
    assert!(bruto.notas[0].ruta.ends_with("core-a.md"));
}

#[test]
fn recall_arranque_no_duplica_core_en_recientes() {
    let (kb, _db_dir, db) = kb_arranque();

    // limite alto: si no se excluyeran los cores del bloque de recientes,
    // core-a/core-b (los commits MÁS RECIENTES del fixture cronológicamente
    // hablando... en realidad core-b/core-a son los primeros commits, así
    // que no son los más recientes; usamos limite=10 para traer todo lo que
    // no sea core y verificar que ninguna de las 2 notas core aparece dos
    // veces en el resultado completo).
    let bruto = recall_arranque(&db, kb.path(), 10).unwrap();
    let apariciones_core_a = bruto
        .notas
        .iter()
        .filter(|n| n.permalink == "kb-demo/core-a")
        .count();
    assert_eq!(apariciones_core_a, 1, "{:?}", bruto.notas);
}

#[test]
fn recall_consulta_devuelve_score_y_snippet_no_nulos() {
    let (kb, _db_dir, db) = kb_arranque();

    let mut bruto = recall_consulta(&db, "contenido nuevo", 5, Some(0.0), 0.0, 0.6).unwrap();
    assert_eq!(bruto.modo, "consulta");
    assert_eq!(bruto.query.as_deref(), Some("contenido nuevo"));
    assert!(!bruto.notas.is_empty(), "esperaba al menos un resultado");

    for n in &bruto.notas {
        assert!(n.score.is_some(), "{:?}", n);
        assert!(n.snippet.is_some(), "{:?}", n);
    }

    resuelve_rutas_absolutas(&mut bruto, kb.path());
    assert!(
        bruto.notas[0]
            .ruta
            .starts_with(&kb.path().display().to_string()),
        "{}",
        bruto.notas[0].ruta
    );
}

#[test]
fn recall_consulta_sin_hits_da_notas_vacias_no_error() {
    let (kb, _db_dir, db) = kb_arranque();

    let bruto = recall_consulta(
        &db,
        "palabra-que-no-existe-en-ningun-lado-de-verdad",
        5,
        Some(1.5),
        0.0,
        0.6,
    )
    .unwrap();
    assert!(bruto.notas.is_empty(), "{:?}", bruto.notas);
    let _ = kb; // la ausencia de hits no es error a este nivel; el CLI decide exit 1
}

#[test]
fn renderiza_produce_bloque_de_texto_con_cabecera_y_notas() {
    let (kb, _db_dir, db) = kb_arranque();
    let bruto = recall_arranque(&db, kb.path(), 1).unwrap();

    let resultado = renderiza(bruto, 2048);
    assert!(!resultado.recall.truncado);
    assert!(resultado.texto.starts_with("=== Recall exo"));
    assert!(resultado.texto.contains("core-a.md"));
}
