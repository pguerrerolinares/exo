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

/// Árbol que ejercita las tres decisiones a la vez.
fn arbol_excluible() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    for sub in [
        "archive",
        "docs",
        "projects",
        "projects/archive",
        ".git",
        ".superpowers",
    ] {
        fs::create_dir_all(p.join(sub)).unwrap();
    }
    fs::write(p.join("raiz.md"), "x").unwrap();
    fs::write(p.join("MAYUS.MD"), "x").unwrap();
    fs::write(p.join("archive/vieja.md"), "x").unwrap();
    fs::write(p.join("docs/doc.md"), "x").unwrap();
    fs::write(p.join(".superpowers/sp.md"), "x").unwrap();
    fs::write(p.join(".git/hook.md"), "x").unwrap();
    fs::write(p.join("projects/vivo.md"), "x").unwrap();
    fs::write(p.join("projects/archive/anidada.md"), "x").unwrap();
    dir
}

#[test]
fn excluye_por_primer_segmento_y_no_por_basename() {
    // A4: `archive/` de raíz se excluye; `projects/archive/` NO. La semántica
    // de basename-en-cada-nivel de kbx budget la saltaría en silencio; se elige
    // la que chequea (degradación hacia rojo). Medido el 2026-09-04: en la KB
    // real no hay ningún dir de estos anidado, así que hoy no cambia números.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(rutas.contains(&"projects/archive/anidada.md".to_string()));
    assert!(!rutas.iter().any(|r| r.starts_with("archive/")));
    assert!(!rutas.iter().any(|r| r.starts_with("docs/")));
}

#[test]
fn el_filtro_md_es_case_insensitive() {
    // A5: un NOTA.MD es una nota. kbx doctor la trataba como no-nota y la
    // sacaba del gate en silencio.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(rutas.contains(&"MAYUS.MD".to_string()));
}

#[test]
fn ningun_dotdir_entra_aunque_no_este_en_la_lista() {
    // .git no está en EXCLUIDOS y kbx budget lo recorre. Aquí no.
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!rutas.iter().any(|r| r.starts_with(".git/")));
    assert!(!rutas.iter().any(|r| r.starts_with(".superpowers/")));
}

#[test]
fn las_rutas_son_relativas_con_barra_y_ordenadas() {
    let dir = arbol_excluible();
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let mut ordenadas = rutas.clone();
    ordenadas.sort();
    assert_eq!(rutas, ordenadas);
    assert!(rutas.iter().all(|r| !r.contains('\\')));
    assert!(rutas.iter().all(|r| !r.starts_with('/')));
}

#[test]
fn el_walk_da_los_subdirectorios_sin_filtrar_por_exclusion() {
    let dir = arbol_excluible();
    let (dirs, _) =
        exo::walker::walk_kb_excluyendo(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(dirs.contains(&("archive".to_string(), "archive".to_string())));
    assert!(dirs.contains(&("archive".to_string(), "projects/archive".to_string())));
    // Los dotdirs no se reportan ni se descienden.
    assert!(!dirs.iter().any(|(b, _)| b.starts_with('.')));
}

#[test]
fn es_md_no_panica_con_nombres_multibyte() {
    // `&nombre[len-3..]` reventaría aquí: esta KB está llena de em-dashes.
    assert!(!exo::walker::es_md("añá"));
    assert!(!exo::walker::es_md("—"));
    assert!(exo::walker::es_md("nota—larga.md"));
    assert!(exo::walker::es_md("NOTA.MD"));
    assert!(!exo::walker::es_md(".md"));
}
