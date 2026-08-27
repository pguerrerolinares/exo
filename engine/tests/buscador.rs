use exo::buscador::{busca, busca_hybrid, busca_vector};
use exo::indexer::indexa;
use rusqlite::params;
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
        resultado
            .results
            .iter()
            .any(|r| r.permalink == "kb-demo/log/agent-develop-bitacora"),
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
    assert!(
        !db.exists(),
        "no debe crear el fichero como side-effect del error"
    );
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
    assert!(
        !resultado.results.is_empty(),
        "esperaba al menos un resultado sobre threshold"
    );
    for r in &resultado.results {
        assert_eq!(r.tipo, "entity");
    }
    for ventana in resultado.results.windows(2) {
        assert!(
            ventana[0].score >= ventana[1].score,
            "{:?}",
            resultado.results
        );
    }
    assert!(
        resultado
            .results
            .iter()
            .any(|r| r.permalink == "kb-demo/log/agent-develop-bitacora"),
        "la nota de la bitácora debería aparecer sobre threshold: {:?}",
        resultado.results
    );
}

/// Un threshold inalcanzable (por encima del máximo teórico de similitud
/// coseno, 1.0) filtra todo — verifica que el filtro por
/// `semantic_min_similarity`/`--min-similarity` realmente se aplica.
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

/// Test contractual 4 (spec fusión §7): admisión por unión (D-f2) — una
/// query sin NINGÚN resultado FTS ("palabra-que-no-existe-en-ningun-lado",
/// mismo string que `query_sin_hits_es_exito_con_resultados_vacios`) no debe
/// perder los candidatos vectoriales; con `min_similitud: Some(0.0)`
/// explícito (B3 — jamás `None` en un test) se garantiza que el canal
/// vector exhaustivo admita candidatos incluso para una query sin relación
/// semántica clara con el fixture. Verifica que `busca_hybrid` no gatea la
/// entrada por FTS.
#[test]
fn fusion_gate_fts_no_pierde_hit_semantico() {
    let (_kb, _db_dir, db) = db_indexada();

    let fts = busca(&db, "palabra-que-no-existe-en-ningun-lado", 50).unwrap();
    assert_eq!(
        fts.results,
        Vec::new(),
        "precondición: FTS vacío para esta query"
    );

    let hybrid = busca_hybrid(
        &db,
        "palabra-que-no-existe-en-ningun-lado",
        10,
        Some(0.0),
        0.2,
        0.8,
    )
    .unwrap();

    assert!(
        !hybrid.results.is_empty(),
        "el canal vector no debería perderse por FTS vacío: {:?}",
        hybrid.results
    );
}

/// Test contractual 9: `v < umbral` ⇒ la entidad pierde el candidato vector
/// pero conserva el FTS si lo tiene (D-f3, threshold pre-fusión sobre v).
/// Umbral inalcanzable (1.5 > 1.0 teórico, mismo patrón que
/// `busca_vector_threshold_alto_filtra_todo`) vacía el canal vector entero;
/// la query "bitácora" sí tiene candidato FTS, así que debe sobrevivir con
/// `score == f` (canal vector ausente = 0).
#[test]
fn threshold_filtra_vector_pre_fusion() {
    let (_kb, _db_dir, db) = db_indexada();

    let hybrid = busca_hybrid(&db, "bitácora", 10, Some(1.5), 0.2, 0.8).unwrap();

    assert!(
        hybrid
            .results
            .iter()
            .any(|r| r.permalink == "kb-demo/log/agent-develop-bitacora"),
        "el candidato FTS debe sobrevivir aunque el vector quede filtrado: {:?}",
        hybrid.results
    );
}

/// Test contractual 11: envelope de `busca_hybrid` — `search_type: "hybrid"`
/// literal, forma del contrato §4.1 intacta (`{permalink, type: "entity",
/// score}`).
#[test]
fn busqueda_hybrid_envelope() {
    let (_kb, _db_dir, db) = db_indexada();

    let resultado = busca_hybrid(&db, "buscable", 10, Some(0.0), 0.2, 0.8).unwrap();
    assert_eq!(resultado.search_type, "hybrid");
    assert!(!resultado.results.is_empty());

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

/// DB con 3 entidades que comparten el MISMO embedding (unitario, componentes
/// 0/1 iguales para las tres) — garantiza empate EXACTO de similitud coseno
/// contra cualquier query, sin depender de azares del modelo real. Inserta
/// las notas en el orden dado (M2-09a: el desempate debe ser independiente
/// del orden de llegada de las filas).
fn db_con_entidades_empatadas(orden: [&str; 3]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("empate.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();

    let mut vector_unitario = vec![0.0f32; 768];
    let raiz_media = std::f32::consts::FRAC_1_SQRT_2;
    vector_unitario[0] = raiz_media;
    vector_unitario[1] = raiz_media;

    for (i, permalink) in orden.into_iter().enumerate() {
        let id = i as i64 + 1;
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?1, ?1, NULL, 0.0, NULL)",
            params![permalink],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO trozos (id, permalink, orden, texto) VALUES (?1, ?2, 0, 'trozo')",
            params![id, permalink],
        )
        .unwrap();
        exo::vectores::inserta(&conn, id, &vector_unitario).unwrap();
    }
    (dir, db)
}

/// M2-09a: `busca_vector` desempata por permalink ascendente cuando el score
/// empata exactamente. `--min-similarity -2.0` (por debajo del mínimo teórico
/// de coseno, -1.0) garantiza que el filtro de umbral nunca descarte las
/// tres entidades empatadas, sin importar el signo real de la similitud
/// contra la query embebida.
#[test]
fn busca_vector_desempate_determinista_por_permalink() {
    let (_d1, db1) = db_con_entidades_empatadas(["z", "x", "y"]);
    let (_d2, db2) = db_con_entidades_empatadas(["y", "z", "x"]);

    let r1 = busca_vector(&db1, "cualquier query", 10, Some(-2.0)).unwrap();
    let r2 = busca_vector(&db2, "cualquier query", 10, Some(-2.0)).unwrap();

    for r in [&r1, &r2] {
        let permalinks: Vec<&str> = r.results.iter().map(|res| res.permalink.as_str()).collect();
        assert_eq!(
            permalinks,
            vec!["x", "y", "z"],
            "empate triple debe desempatar por permalink ascendente: {:?}",
            r.results
        );
    }
}

// --- Modo mudo del arm vector (backlog "alta", 2026-08-22) ---
//
// `busca_hybrid` fusionaba un arm vector vacío sin distinguir "no encontró
// nada" de "la tabla `vectores` está vacía o a medio poblar", y devolvía el
// resultado etiquetado `hybrid` siendo FTS puro. Devolver resultados
// plausibles estando roto es el modo de fallo más caro de un instrumento de
// retrieval; estos tests fijan que la degradación sea VISIBLE.

/// DB fixture indexada a la que se le vacía la tabla `vectores`: trozos sí,
/// vectores no. Es el estado real que produce un embed abortado a medias o un
/// índice heredado de antes de M2-06.
fn db_sin_vectores() -> (tempfile::TempDir, tempfile::TempDir, std::path::PathBuf) {
    let (kb, db_dir, db) = db_indexada();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM vectores", []).unwrap();
    let quedan: i64 = conn
        .query_row("SELECT count(*) FROM vectores", [], |f| f.get(0))
        .unwrap();
    assert_eq!(quedan, 0, "precondición: la tabla vectores queda vacía");
    (kb, db_dir, db)
}

#[test]
fn hybrid_sin_vectores_avisa_de_que_es_fts_puro() {
    let (_kb, _db_dir, db) = db_sin_vectores();

    let hybrid = busca_hybrid(&db, "buscable", 10, Some(0.0), 0.2, 0.8).unwrap();

    assert!(
        !hybrid.results.is_empty(),
        "precondición: FTS sí tiene hits para esta query"
    );
    assert!(
        hybrid.avisos.iter().any(|a| a.contains("vector")),
        "un hybrid servido por FTS puro debe avisarlo: {:?}",
        hybrid.avisos
    );
}

#[test]
fn hybrid_con_cobertura_parcial_avisa_con_las_cifras() {
    let (_kb, _db_dir, db) = db_indexada();
    let conn = exo::abre_db(&db).unwrap();
    let trozos: i64 = conn
        .query_row("SELECT count(*) FROM trozos", [], |f| f.get(0))
        .unwrap();
    assert!(trozos >= 2, "precondición: el fixture tiene varios trozos");
    let victima: i64 = conn
        .query_row("SELECT rowid FROM vectores LIMIT 1", [], |f| f.get(0))
        .unwrap();
    conn.execute("DELETE FROM vectores WHERE rowid = ?1", params![victima])
        .unwrap();

    let hybrid = busca_hybrid(&db, "buscable", 10, Some(0.0), 0.2, 0.8).unwrap();

    let aviso = hybrid.avisos.join(" | ");
    assert!(
        aviso.contains(&(trozos - 1).to_string()) && aviso.contains(&trozos.to_string()),
        "el aviso de cobertura parcial debe llevar las cifras ({} de {}): {aviso:?}",
        trozos - 1,
        trozos
    );
}

#[test]
fn hybrid_con_cobertura_completa_no_ensucia_el_envelope() {
    let (_kb, _db_dir, db) = db_indexada();

    let hybrid = busca_hybrid(&db, "buscable", 10, Some(0.0), 0.2, 0.8).unwrap();

    assert!(
        hybrid.avisos.is_empty(),
        "sin degradación no hay avisos: {:?}",
        hybrid.avisos
    );
    let valor = serde_json::to_value(&hybrid).unwrap();
    assert!(
        !valor.as_object().unwrap().contains_key("warnings"),
        "la clave `warnings` no debe aparecer cuando está vacía (envelope v2 §4.1)"
    );
}

#[test]
fn vector_puro_sin_vectores_tambien_avisa() {
    let (_kb, _db_dir, db) = db_sin_vectores();

    let vector = busca_vector(&db, "buscable", 10, Some(0.0)).unwrap();

    assert_eq!(
        vector.results,
        Vec::new(),
        "contrato Task 3: 0 resultados, no error"
    );
    assert!(
        !vector.avisos.is_empty(),
        "0 resultados por tabla vacía no es lo mismo que 0 resultados por query: {:?}",
        vector.avisos
    );
}
