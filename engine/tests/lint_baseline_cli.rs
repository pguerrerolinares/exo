//! `exo lint --baseline`: solo gatean los hallazgos del `--kb` que la baseline
//! no tiene. Ejercita el binario: el contrato es el exit code y el stdout.
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn nota(permalink: &str, cuerpo: &str) -> String {
    format!("---\npermalink: {permalink}\ntier: stable\n---\n{cuerpo}\n")
}

/// KB base: `a` y `b` se enlazan entre sí (ninguna huérfana) y `a` lleva un
/// link roto previo, ajeno a lo que el subagente toque.
fn escribe_base(dir: &Path) {
    fs::write(dir.join("a.md"), nota("kb/a", "[[kb/b]] [[no-existe]]")).unwrap();
    fs::write(dir.join("b.md"), nota("kb/b", "[[kb/a]]")).unwrap();
}

fn copia(origen: &Path, destino: &Path) {
    for e in fs::read_dir(origen).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), destino.join(e.file_name())).unwrap();
    }
}

fn lint(kb: &Path, baseline: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["lint", "--kb"])
        .arg(kb)
        .arg("--baseline")
        .arg(baseline)
        .output()
        .unwrap()
}

fn texto(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn hallazgo_previo_en_la_baseline_no_bloquea() {
    let base = tempfile::tempdir().unwrap();
    let wt = tempfile::tempdir().unwrap();
    escribe_base(base.path());
    copia(base.path(), wt.path());
    // Edita una nota sin tocar el link roto ajeno.
    fs::write(wt.path().join("b.md"), nota("kb/b", "[[kb/a]] más texto")).unwrap();

    let o = lint(wt.path(), base.path());
    assert_eq!(o.status.code(), Some(0), "stdout: {}", texto(&o));
    assert!(texto(&o).contains("ok"), "{}", texto(&o));
}

#[test]
fn broken_link_nuevo_en_la_nota_tocada_sale_tres_y_solo_lista_el_nuevo() {
    let base = tempfile::tempdir().unwrap();
    let wt = tempfile::tempdir().unwrap();
    escribe_base(base.path());
    copia(base.path(), wt.path());
    fs::write(
        wt.path().join("b.md"),
        nota("kb/b", "[[kb/a]] [[nuevo-roto]]"),
    )
    .unwrap();

    let o = lint(wt.path(), base.path());
    assert_eq!(o.status.code(), Some(3), "stdout: {}", texto(&o));
    let out = texto(&o);
    assert!(out.contains("[[nuevo-roto]]"), "{out}");
    assert!(
        !out.contains("[[no-existe]]"),
        "el previo no se reporta: {out}"
    );
}

#[test]
fn mover_un_bloque_que_rompe_un_link_en_otra_nota_se_detecta() {
    // Lintar solo la nota tocada no basta: renombrar el título de `b` rompe el
    // `[[Titulo B]]` de `a`, que no se tocó.
    let base = tempfile::tempdir().unwrap();
    let wt = tempfile::tempdir().unwrap();
    fs::write(base.path().join("a.md"), nota("kb/a", "[[Titulo B]]")).unwrap();
    fs::write(
        base.path().join("b.md"),
        "---\npermalink: kb/b\ntitle: Titulo B\ntier: stable\n---\n[[kb/a]]\n",
    )
    .unwrap();
    copia(base.path(), wt.path());
    fs::write(
        wt.path().join("b.md"),
        "---\npermalink: kb/b\ntitle: Otro\ntier: stable\n---\n[[kb/a]]\n",
    )
    .unwrap();

    let o = lint(wt.path(), base.path());
    assert_eq!(o.status.code(), Some(3), "stdout: {}", texto(&o));
    assert!(
        texto(&o).contains("broken_link\ta.md\t[[Titulo B]]"),
        "{}",
        texto(&o)
    );
}

#[test]
fn baseline_y_db_son_excluyentes() {
    let base = tempfile::tempdir().unwrap();
    escribe_base(base.path());
    let o = Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(["lint", "--db", "/x.db", "--kb"])
        .arg(base.path())
        .arg("--baseline")
        .arg(base.path())
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
