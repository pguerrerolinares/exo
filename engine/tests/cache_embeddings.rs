//! M6-01b: cache de embeddings por contenido del trozo.
//!
//! Al reindexar una nota, los trozos cuyo TEXTO no cambió reutilizan el
//! embedding ya almacenado en vez de volver a pasar por el modelo. Patrón
//! estándar del ecosistema RAG (LlamaIndex `IngestionPipeline` con hash en el
//! docstore y `DocstoreStrategy.UPSERTS`; LanceDB documenta lo mismo para
//! contextual retrieval) y, en la medición de esta máquina, la diferencia
//! entre pagar ~0,25 s por trozo de la nota entera o solo por el que se tocó.
//!
//! Estos tests NO cargan el modelo real: comprueban la CONTABILIDAD
//! (`embebidos` / `reusados`) del indexer, que es lo que distingue haber
//! llamado al modelo de haberlo evitado.

use exo::indexer::indexa;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
}

fn kb_con(dir: &std::path::Path, cuerpo: &str) {
    fs::write(
        dir.join("nota.md"),
        format!("---\npermalink: kb/nota\ntitle: Nota\n---\n\n{cuerpo}\n"),
    )
    .unwrap();
}

fn commitea(dir: &std::path::Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            msg,
        ],
    );
}

/// Dos párrafos largos separados por línea en blanco → el troceador los deja
/// en trozos distintos, que es lo que este test necesita para tocar uno solo.
fn dos_parrafos(segundo: &str) -> String {
    format!(
        "{}\n\n{}",
        "Primer párrafo que no se toca. ".repeat(40),
        segundo
    )
}

#[test]
fn reindexar_sin_cambios_de_texto_no_vuelve_a_embeber() {
    let kb = TempDir::new().unwrap();
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    git(kb.path(), &["init", "-q"]);
    kb_con(kb.path(), &dos_parrafos("Segundo párrafo original."));
    commitea(kb.path(), "uno");

    let primero = indexa(kb.path(), &db).unwrap();
    assert!(
        primero.trozos_embebidos >= 2,
        "el primer indexado sí embebe"
    );
    assert_eq!(primero.trozos_reusados, 0, "no había nada que reutilizar");

    // Cambia el frontmatter (el título), NO el cuerpo: el mtime cambia, así
    // que la nota se reindexa entera... pero ni un solo trozo debe pasar por
    // el modelo.
    fs::write(
        kb.path().join("nota.md"),
        format!(
            "---\npermalink: kb/nota\ntitle: Otro título\n---\n\n{}\n",
            dos_parrafos("Segundo párrafo original.")
        ),
    )
    .unwrap();
    commitea(kb.path(), "dos");

    let segundo = indexa(kb.path(), &db).unwrap();
    assert_eq!(
        segundo.indexadas, 1,
        "la nota sí se reindexa (mtime cambió)"
    );
    assert_eq!(
        segundo.trozos_embebidos, 0,
        "ningún trozo cambió de texto: cero llamadas al modelo"
    );
    assert!(segundo.trozos_reusados >= 2, "todos reutilizados");
}

#[test]
fn editar_un_trozo_solo_reembebe_ese_trozo() {
    let kb = TempDir::new().unwrap();
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    git(kb.path(), &["init", "-q"]);
    kb_con(kb.path(), &dos_parrafos("Segundo párrafo original."));
    commitea(kb.path(), "uno");

    let primero = indexa(kb.path(), &db).unwrap();
    let total = primero.trozos_embebidos;
    assert!(total >= 2);

    // Solo cambia el segundo párrafo.
    kb_con(
        kb.path(),
        &dos_parrafos("Segundo párrafo REESCRITO por completo."),
    );
    commitea(kb.path(), "dos");

    let segundo = indexa(kb.path(), &db).unwrap();
    assert_eq!(
        segundo.trozos_embebidos, 1,
        "solo el trozo tocado pasa por el modelo"
    );
    assert_eq!(
        segundo.trozos_reusados,
        total - 1,
        "el resto se reutiliza tal cual"
    );
}

#[test]
fn el_embedding_reutilizado_es_el_mismo_byte_a_byte() {
    let kb = TempDir::new().unwrap();
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    git(kb.path(), &["init", "-q"]);
    kb_con(kb.path(), &dos_parrafos("Segundo párrafo original."));
    commitea(kb.path(), "uno");
    indexa(kb.path(), &db).unwrap();

    let antes = embedding_del_primer_trozo(&db);

    kb_con(kb.path(), &dos_parrafos("Segundo párrafo REESCRITO."));
    commitea(kb.path(), "dos");
    indexa(kb.path(), &db).unwrap();

    let despues = embedding_del_primer_trozo(&db);
    assert_eq!(
        antes, despues,
        "reutilizar no puede corromper ni desplazar el vector guardado"
    );
}

/// Lee el embedding del trozo `orden = 0` de la única nota, como blob crudo.
fn embedding_del_primer_trozo(db: &std::path::Path) -> Vec<u8> {
    let conn = exo::abre_db(db).unwrap();
    let id: i64 = conn
        .query_row(
            "SELECT id FROM trozos WHERE permalink = 'kb/nota' AND orden = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    conn.query_row(
        "SELECT embedding FROM vectores WHERE rowid = ?1",
        [id],
        |r| r.get(0),
    )
    .unwrap()
}
