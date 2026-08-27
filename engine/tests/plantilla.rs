use tempfile::TempDir;

#[test]
fn son_doce_ficheros() {
    assert_eq!(exo::plantilla::FICHEROS.len(), 12);
}

#[test]
fn render_sustituye_el_placeholder() {
    let out = exo::plantilla::render("permalink: {{KB_NAME}}/core/core-index", "mi-kb");
    assert_eq!(out, "permalink: mi-kb/core/core-index");
    assert!(!out.contains("{{KB_NAME}}"));
}

#[test]
fn vuelca_escribe_los_doce_y_no_deja_placeholders() {
    let dir = TempDir::new().unwrap();
    let escritos = exo::plantilla::vuelca(dir.path(), "mi-kb").expect("volcar");
    assert_eq!(escritos.len(), 12);
    for f in &escritos {
        assert!(f.exists(), "no existe {}", f.display());
        if f.extension().is_some_and(|e| e == "md") {
            let c = std::fs::read_to_string(f).unwrap();
            assert!(!c.contains("{{"), "placeholder vivo en {}", f.display());
        }
    }
    assert!(dir.path().join("archive/log/.gitkeep").exists());
}
