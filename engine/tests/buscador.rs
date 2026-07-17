use exo::buscador::{busca, busca_vector};
use exo::indexer::indexa;
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

fn crea_nota(dir: &Path, nombre: &str, permalink: &str, titulo: &str, cuerpo: &str) {
    let contenido = format!("---\npermalink: {permalink}\ntitle: {titulo}\n---\n{cuerpo}\n");
    std::fs::write(dir.join(nombre), contenido).unwrap();
}

/// KB fixture indexada (mismo patrón que tests/indexer.rs).
fn db_indexada() -> (tempfile::TempDir, tempfile::TempDir, std::path::PathBuf) {
    let kb = tempfile::tempdir().unwrap();
    git(kb.path(), &["init", "-q"]);
    git(kb.path(), &["config", "user.email", "test@exo.local"]);
    git(kb.path(), &["config", "user.name", "exo-test"]);
    crea_nota(
        kb.path(),
        "agent-develop.md",
        "kb-demo/log/agent-develop-bitacora",
        "Bitácora agent-develop",
        "contenido de la bitácora agent-develop, con guiones y acentos: café",
    );
    crea_nota(
        kb.path(),
        "poco.md",
        "kb-demo/poco",
        "Nota con poca relevancia",
        "buscable una vez nada más",
    );
    crea_nota(
        kb.path(),
        "mucho.md",
        "kb-demo/mucho",
        "Nota con mucha relevancia",
        "buscable buscable buscable buscable buscable",
    );
    git(kb.path(), &["add", "."]);
    git(kb.path(), &["commit", "-q", "-m", "fixture de búsqueda"]);

    let db_dir = tempfile::tempdir().unwrap();
    let db = db_dir.path().join("exo.db");
    indexa(kb.path(), &db).unwrap();
    (kb, db_dir, db)
}

#[test]
fn query_con_guiones_y_acentos_no_revienta() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "agent-develop bitácora", 10).expect("no debe reventar FTS5");
    assert_eq!(resultado.search_type, "fts");
    assert!(
        resultado.results.iter().any(|r| r.permalink == "kb-demo/log/agent-develop-bitacora"),
        "{:?}",
        resultado.results
    );
}

#[test]
fn resultados_a_nivel_entidad_con_tipo_fijo() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "bitácora", 10).unwrap();
    assert!(!resultado.results.is_empty());
    for r in &resultado.results {
        assert_eq!(r.tipo, "entity");
    }
}

#[test]
fn resultados_ordenados_por_score_descendente() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "buscable", 10).unwrap();
    assert_eq!(resultado.results.len(), 2, "{:?}", resultado.results);
    assert_eq!(resultado.results[0].permalink, "kb-demo/mucho");
    assert_eq!(resultado.results[1].permalink, "kb-demo/poco");
    assert!(resultado.results[0].score >= resultado.results[1].score);
}

#[test]
fn query_sin_hits_es_exito_con_resultados_vacios() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "palabra-que-no-existe-en-ningun-lado", 10).unwrap();
    assert_eq!(resultado.results, Vec::new());
}

#[test]
fn limite_recorta_resultados() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "bitácora buscable", 1).unwrap();
    assert!(resultado.results.len() <= 1);
}

#[test]
fn envelope_data_serializa_con_las_claves_del_contrato_4_1() {
    let (_kb, _db_dir, db) = db_indexada();
    let resultado = busca(&db, "buscable", 10).unwrap();
    let valor = serde_json::to_value(&resultado).unwrap();
    let obj = valor.as_object().unwrap();
    assert!(obj.contains_key("query"));
    assert!(obj.contains_key("search_type"));
    assert!(obj.contains_key("elapsed_s"));
    assert!(obj.contains_key("results"));
    let primero = &obj["results"][0];
    assert!(primero.get("permalink").is_some());
    assert!(primero.get("type").is_some());
    assert!(primero.get("score").is_some());
}

#[test]
fn db_inexistente_da_error_claro() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("no-existe.db");
    let err = busca(&db, "algo", 10).expect_err("debe fallar, no crear una DB vacía");
    assert!(!db.exists(), "no debe crear el fichero como side-effect del error");
    assert!(format!("{err:#}").contains("no-existe.db"));
}

/// M2-06 Task 3: `busca_vector` sobre una DB poblada devuelve entidades
/// (`type: "entity"`) ordenadas por score descendente, y encuentra la nota
/// cuyo contenido es semánticamente (y aquí también literalmente) más
/// cercano a la query.
#[test]
fn busca_vector_con_db_poblada_devuelve_entidades_ordenadas() {
    let (_kb, _db_dir, db) = db_indexada();

    let resultado = busca_vector(&db, "la bitácora de agent-develop", 10, None).unwrap();

    assert_eq!(resultado.search_type, "vector");
    assert!(!resultado.results.is_empty(), "esperaba al menos un resultado sobre threshold");
    for r in &resultado.results {
        assert_eq!(r.tipo, "entity");
    }
    for ventana in resultado.results.windows(2) {
        assert!(ventana[0].score >= ventana[1].score, "{:?}", resultado.results);
    }
    assert!(
        resultado.results.iter().any(|r| r.permalink == "kb-demo/log/agent-develop-bitacora"),
        "la nota de la bitácora debería aparecer sobre threshold: {:?}",
        resultado.results
    );
}

/// Un threshold inalcanzable (por encima del máximo teórico de similitud
/// coseno, 1.0) filtra todo — verifica que el filtro por
/// `semantic_min_similarity`/`--min-similitud` realmente se aplica.
#[test]
fn busca_vector_threshold_alto_filtra_todo() {
    let (_kb, _db_dir, db) = db_indexada();

    let resultado = busca_vector(&db, "la bitácora de agent-develop", 10, Some(1.5)).unwrap();

    assert_eq!(resultado.results, Vec::new());
}

/// DB con schema creado pero sin ninguna nota indexada (0 filas en
/// `vectores`): éxito con `results: []`, no error (Task 3, declarado —
/// paridad con el contrato de `busca` FTS: "sin hits = éxito").
#[test]
fn busca_vector_sobre_db_sin_vectores_da_cero_resultados() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("vacia.db");
    {
        let conn = exo::abre_db(&db).unwrap();
        exo::schema::crea_schema(&conn).unwrap();
    }

    let resultado = busca_vector(&db, "cualquier cosa", 10, None).unwrap();
    assert_eq!(resultado.search_type, "vector");
    assert_eq!(resultado.results, Vec::new());
}
