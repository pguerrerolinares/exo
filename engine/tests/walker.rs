use exo::walker::walk_kb;
use std::fs;
use std::path::PathBuf;

/// Árbol fixture: `.claude/x.md`, `.superpowers/y.md`, `.omc/z.md`,
/// `archive/a.md`, `b.md`, `c.txt`.
fn arbol_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (sub, nombre) in [
        (".claude", "x.md"),
        (".superpowers", "y.md"),
        (".omc", "z.md"),
        ("archive", "a.md"),
    ] {
        let subdir = dir.path().join(sub);
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join(nombre), "contenido").unwrap();
    }
    fs::write(dir.path().join("b.md"), "contenido").unwrap();
    fs::write(dir.path().join("c.txt"), "contenido").unwrap();
    dir
}

fn nombres(rutas: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = rutas
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn walker_excluye_dotdirs() {
    // nombre FIJADO §1.1 regla 3: los 3 dotdirs fuera
    let dir = arbol_fixture();
    let rutas = walk_kb(dir.path()).unwrap();
    let vistas = nombres(&rutas);
    assert!(!vistas.contains(&"x.md".to_string()), "{vistas:?}");
    assert!(!vistas.contains(&"y.md".to_string()), "{vistas:?}");
    assert!(!vistas.contains(&"z.md".to_string()), "{vistas:?}");
}

#[test]
fn walker_solo_markdown() {
    // nombre FIJADO §1.1 regla 5: c.txt fuera
    let dir = arbol_fixture();
    let rutas = walk_kb(dir.path()).unwrap();
    assert!(!nombres(&rutas).contains(&"c.txt".to_string()));
}

#[test]
fn walker_incluye_archive() {
    // regla 4: archive/a.md dentro
    let dir = arbol_fixture();
    let rutas = walk_kb(dir.path()).unwrap();
    assert!(nombres(&rutas).contains(&"a.md".to_string()));
}

#[test]
fn walker_orden_determinista() {
    let dir = arbol_fixture();
    let primera = walk_kb(dir.path()).unwrap();
    let segunda = walk_kb(dir.path()).unwrap();
    assert_eq!(primera, segunda);
}
