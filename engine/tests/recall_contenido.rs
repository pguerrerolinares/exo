//! M6-02: `exo recall --contenido`.
//!
//! El hook de arranque que esto sustituye NO inyecta una lista de ficheros:
//! inyecta el CUERPO del core-index (contrato de memoria + doctrina compacta
//! + mapa de cores) más un digest de actividad reciente. Servir solo rutas
//! sería una regresión funcional silenciosa — el agente perdería la doctrina
//! en todas las sesiones y nadie lo notaría hasta que empezara a comportarse
//! peor.

use exo::indexer::indexa;
use exo::recall::recall_arranque_contenido;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git").args(args).current_dir(dir).output().unwrap();
}

fn escribe(dir: &std::path::Path, nombre: &str, tier: &str, cuerpo: &str) {
    let permalink = nombre.trim_end_matches(".md");
    fs::write(
        dir.join(nombre),
        format!("---\npermalink: kb/{permalink}\ntitle: {permalink}\ntier: {tier}\n---\n\n{cuerpo}\n"),
    )
    .unwrap();
}

fn kb_de_prueba() -> (TempDir, std::path::PathBuf, TempDir) {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    escribe(kb.path(), "indice.md", "core", "DOCTRINA: delega y quédate la conclusión.");
    escribe(kb.path(), "otra.md", "log", "Bitácora de cosas que pasaron.");
    git(kb.path(), &["add", "-A"]);
    git(
        kb.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "kb"],
    );
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    indexa(kb.path(), &db).unwrap();
    (kb, db, dbdir)
}

#[test]
fn contenido_vuelca_el_cuerpo_de_las_notas_core() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        bloque.contains("DOCTRINA: delega y quédate la conclusión."),
        "el cuerpo del core debe ir literal en el bloque, no solo su ruta:\n{bloque}"
    );
}

#[test]
fn contenido_no_vuelca_el_cuerpo_de_las_notas_no_core() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        !bloque.contains("Bitácora de cosas que pasaron."),
        "solo los `tier: core` van con cuerpo; el resto, como mucho, listados"
    );
}

#[test]
fn contenido_lista_las_recientes_por_permalink() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        bloque.contains("kb/otra"),
        "las recientes se listan por permalink (paridad con el digest del hook actual):\n{bloque}"
    );
}

#[test]
fn contenido_respeta_el_cap_de_bytes() {
    let (kb, db, _d) = kb_de_prueba();
    let cap = 120;
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, cap, None).unwrap();

    assert!(
        bloque.len() <= cap,
        "bloque de {} bytes con cap {cap}: el cap del consumidor no es negociable",
        bloque.len()
    );
}

#[test]
fn contenido_falla_si_no_hay_nada_que_servir() {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    indexa(kb.path(), &db).unwrap();

    // KB sin notas: el hook debe poder distinguir "no hay bloque" por exit
    // code y caer a su fallback, en vez de inyectar un bloque vacío.
    assert!(recall_arranque_contenido(&db, kb.path(), 5, 8192, None).is_err());
}
