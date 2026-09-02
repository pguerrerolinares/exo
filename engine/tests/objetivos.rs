//! `busca_objetivos` contra una KB y un índice reales en un tempdir.

use exo::objetivos::busca_objetivos;
use std::process::Command;

/// KB con cuatro notas committeadas y su índice SQLite poblado a mano.
///
/// La cuarta, `informe.pdf` con `tipo='report'`, no es un descuido: kbx tenía
/// un filtro `note_type='note'` que escondía el 40% de la KB (57 de 138
/// notas reales) y se retiró por eso (ver rustdoc de `CONSULTA_CANDIDATAS`).
/// Sin esta fila, un `AND notas.tipo = 'note'` añadido por error pasaría la
/// suite entera en verde.
///
/// El índice se puebla con SQL directo en vez de con `exo index` porque
/// indexar de verdad descargaría el modelo ONNX de 615 MB, y estos tests no
/// ejercen embeddings.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = Command::new("git")
            .arg("-C")
            .arg(&kb)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    };

    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    std::fs::write(
        kb.join("log/beta.md"),
        "---\ntier: core\n---\n# beta\ncuerpo de beta\n",
    )
    .unwrap();
    std::fs::write(
        kb.join("log/gamma.md"),
        "sin frontmatter\ncuerpo de gamma\n",
    )
    .unwrap();
    std::fs::write(kb.join("informe.pdf"), "contenido binario simulado").unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for (permalink, rel, titulo, tipo, cuerpo) in [
        (
            "kb/log/alpha",
            "log/alpha.md",
            "alpha",
            "note",
            "cuerpo de alpha",
        ),
        (
            "kb/log/beta",
            "log/beta.md",
            "beta",
            "note",
            "cuerpo de beta",
        ),
        (
            "kb/log/gamma",
            "log/gamma.md",
            "gamma",
            "note",
            "cuerpo de gamma",
        ),
        (
            "kb/informe",
            "informe.pdf",
            "informe",
            "report",
            "cuerpo del informe",
        ),
    ] {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, ?4, 0.0, NULL)",
            rusqlite::params![permalink, rel, titulo, tipo],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            rusqlite::params![titulo, cuerpo, permalink],
        )
        .unwrap();
    }
    drop(conn);
    (dir, db)
}

#[test]
fn encuentra_la_candidata_con_tier_size_y_last_commit_de_disco() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();

    assert_eq!(r.tema, "alpha");
    assert_eq!(r.candidatos.len(), 1);
    let c = &r.candidatos[0];
    assert_eq!(c.permalink, "kb/log/alpha");
    assert_eq!(c.tier, "stable");
    assert_eq!(c.headings, vec!["alpha"]);
    assert_eq!(c.ultimo_commit, "2026-07-01T10:00:00+02:00");
    let en_disco = std::fs::metadata(dir.path().join("log/alpha.md"))
        .unwrap()
        .len();
    assert_eq!(c.tamano_bytes as u64, en_disco);
}

#[test]
fn sin_coincidencias_la_lista_esta_vacia_y_no_hay_error() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "zzz-inexistente", 10).unwrap();
    assert!(r.candidatos.is_empty());
}

// El test de `construye_match_query` prueba la FORMA de la cadena; este
// prueba lo único que importa de verdad: que FTS5 la trate como texto. Un
// operador que se ejecutase daría error de sintaxis o traería resultados
// semánticos. Los cuatro casos son las cuatro clases de operador.
//
// Portado de `TestSearch_Injection` de kbx, que mi plan se había dejado.
#[test]
fn ningun_operador_fts5_se_ejecuta_como_operador() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    for tema in [
        "cuerp*",
        "alpha OR beta",
        "NEAR(alpha beta)",
        "titulo:alpha",
    ] {
        let r = busca_objetivos(&conn, dir.path(), tema, 10)
            .unwrap_or_else(|e| panic!("{tema} no debe ser error de sintaxis FTS5: {e:#}"));
        assert!(
            r.candidatos.is_empty(),
            "{tema} se ejecutó como operador y trajo {} candidatas",
            r.candidatos.len()
        );
    }
}

#[test]
fn el_limite_trunca_y_el_orden_es_estable() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let amplio = busca_objetivos(&conn, dir.path(), "cuerpo", 100).unwrap();
    assert!(amplio.candidatos.len() > 2);

    let a = busca_objetivos(&conn, dir.path(), "cuerpo", 2).unwrap();
    let b = busca_objetivos(&conn, dir.path(), "cuerpo", 2).unwrap();
    assert_eq!(a.candidatos.len(), 2);
    let permalinks = |r: &exo::objetivos::Objetivos| {
        r.candidatos
            .iter()
            .map(|c| c.permalink.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(permalinks(&a), permalinks(&b));
}

#[test]
fn un_limite_menor_que_uno_es_error() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    assert!(busca_objetivos(&conn, dir.path(), "alpha", 0).is_err());
}

// El dedup es por `notas.ruta`, no por permalink, y se queda con la PRIMERA
// fila — que por `ORDER BY rank` es la de mejor rank. Hoy `notas_fts` es 1:1
// con `notas` y el dedup es un no-op, pero se mantiene para el día en que el
// FTS indexe trozos.
#[test]
fn deduplica_por_ruta_conservando_la_fila_de_mejor_rank() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES ('alpha', 'relleno relleno alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1, "la ruta duplicada debe colapsar");
}

// Asimetría deliberada: un fallo de DISCO degrada a valores vacíos y la
// candidata sigue apareciendo (el índice la conoce), mientras que un fallo de
// GIT aborta todo el resultado. Este test cubre la mitad best-effort.
#[test]
fn una_candidata_con_fichero_ilegible_sigue_apareciendo() {
    let (dir, db) = kb_con_indice();
    std::fs::remove_file(dir.path().join("log/alpha.md")).unwrap();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "alpha", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1);
    assert_eq!(r.candidatos[0].tier, "");
    assert_eq!(r.candidatos[0].tamano_bytes, 0);
    assert!(r.candidatos[0].headings.is_empty());
}

// Y esta es la otra mitad: git roto NO degrada, aborta. Es el invariante 6 de
// la spec — la trampa concreta de este port.
#[test]
fn un_fallo_de_git_aborta_en_vez_de_dejar_last_commit_vacio() {
    let (dir, db) = kb_con_indice();
    std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
    let conn = exo::abre_db(&db).unwrap();
    assert!(busca_objetivos(&conn, dir.path(), "alpha", 10).is_err());
}

// La nota sin frontmatter no se excluye: aparece con tier vacío. Filtrar por
// tipo o por tier aquí escondería notas reales (M6-04 T3 retiró el filtro
// `note_type='note'` justo por eso).
#[test]
fn una_nota_sin_frontmatter_aparece_con_tier_vacio() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "gamma", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1);
    assert_eq!(r.candidatos[0].tier, "");
}

// El SQL no filtra por `notas.tipo` a propósito: kbx tenía ese filtro y
// escondía 57 de 138 notas reales de la KB. Un `AND notas.tipo = 'note'`
// añadido a `CONSULTA_CANDIDATAS` pondría esta nota en rojo aunque las tres
// notas del resto del fixture sigan pasando. Nombrado como el
// `TestSearchIncluyeTipoNoNote` de kbx.
#[test]
fn incluye_notas_con_tipo_distinto_de_note() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    let r = busca_objetivos(&conn, dir.path(), "informe", 10).unwrap();
    assert_eq!(r.candidatos.len(), 1);
    assert_eq!(r.candidatos[0].permalink, "kb/informe");
}
