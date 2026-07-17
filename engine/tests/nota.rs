use exo::nota::parsea_nota;
use std::io::Write;

fn escribe(dir: &tempfile::TempDir, nombre: &str, contenido: &str) -> std::path::PathBuf {
    let ruta = dir.path().join(nombre);
    let mut f = std::fs::File::create(&ruta).unwrap();
    f.write_all(contenido.as_bytes()).unwrap();
    ruta
}

#[test]
fn permalink_del_frontmatter_se_honra() {
    // nombre FIJADO en spec §1.1 regla 1
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(
        &dir,
        "x.md",
        "---\npermalink: kb-demo/x/y\ntitle: X\n---\ncuerpo de la nota\n",
    );
    let nota = parsea_nota(&ruta).unwrap().expect("debe parsear");
    assert_eq!(nota.permalink, "kb-demo/x/y");
}

#[test]
fn nota_sin_permalink_se_salta() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(&dir, "sin.md", "---\ntitle: Sin permalink\n---\ncuerpo\n");
    assert_eq!(parsea_nota(&ruta).unwrap(), None);
}

#[test]
fn nota_sin_frontmatter_se_salta() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(&dir, "plano.md", "solo texto, sin frontmatter\n");
    assert_eq!(parsea_nota(&ruta).unwrap(), None);
}

#[test]
fn frontmatter_ilegible_se_salta_sin_panic() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(&dir, "roto.md", "---\npermalink: [x: y\n---\ncuerpo\n");
    assert_eq!(parsea_nota(&ruta).unwrap(), None);
}

#[test]
fn titulo_de_frontmatter_o_stem() {
    let dir = tempfile::tempdir().unwrap();

    let con_titulo = escribe(
        &dir,
        "a.md",
        "---\npermalink: kb-demo/a\ntitle: Título explícito\n---\ncuerpo\n",
    );
    let nota = parsea_nota(&con_titulo).unwrap().unwrap();
    assert_eq!(nota.titulo, "Título explícito");

    let sin_titulo = escribe(&dir, "mi-nota-b.md", "---\npermalink: kb-demo/b\n---\ncuerpo\n");
    let nota = parsea_nota(&sin_titulo).unwrap().unwrap();
    assert_eq!(nota.titulo, "mi-nota-b");
}

#[test]
fn tipo_del_frontmatter_se_lee() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(
        &dir,
        "c.md",
        "---\npermalink: kb-demo/c\ntype: nota\n---\ncuerpo\n",
    );
    let nota = parsea_nota(&ruta).unwrap().unwrap();
    assert_eq!(nota.tipo.as_deref(), Some("nota"));
}

#[test]
fn cuerpo_es_lo_que_sigue_al_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = escribe(
        &dir,
        "d.md",
        "---\npermalink: kb-demo/d\n---\nlínea 1\nlínea 2\n",
    );
    let nota = parsea_nota(&ruta).unwrap().unwrap();
    assert_eq!(nota.cuerpo, "línea 1\nlínea 2");
}
