use tempfile::TempDir;

#[test]
fn son_diecisiete_ficheros() {
    assert_eq!(exo::plantilla::FICHEROS.len(), 17);
}

#[test]
fn render_sustituye_el_placeholder() {
    let out = exo::plantilla::render("permalink: {{KB_NAME}}/core/core-index", "mi-kb");
    assert_eq!(out, "permalink: mi-kb/core/core-index");
    assert!(!out.contains("{{KB_NAME}}"));
}

#[test]
fn vuelca_escribe_los_diecisiete_y_no_deja_placeholders() {
    let dir = TempDir::new().unwrap();
    let escritos = exo::plantilla::vuelca(dir.path(), "mi-kb").expect("volcar");
    assert_eq!(escritos.len(), 17);
    for f in &escritos {
        assert!(f.exists(), "no existe {}", f.display());
        if f.extension().is_some_and(|e| e == "md") {
            let c = std::fs::read_to_string(f).unwrap();
            assert!(!c.contains("{{"), "placeholder vivo en {}", f.display());
        }
    }
    assert!(dir.path().join("archive/log/.gitkeep").exists());
}

fn campo_frontmatter(c: &str, campo: &str) -> Option<String> {
    let v = c
        .lines()
        .skip(1)
        .take_while(|l| *l != "---")
        .find_map(|l| l.strip_prefix(&format!("{campo}:")))?;
    Some(v.trim().trim_matches('"').to_string())
}

fn md_de(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            md_de(&p, out);
        } else if p.extension().is_some_and(|x| x == "md") {
            out.push(p);
        }
    }
}

/// La semilla debe nacer sin aristas rotas: la resolución es por título o
/// permalink exactos (`aristas::resuelve_destinos`), y un slug o un título
/// desactualizado deja `destino_permalink = NULL` sin que nada avise.
#[test]
fn todos_los_wikilinks_de_la_semilla_resuelven() {
    let dir = TempDir::new().unwrap();
    exo::plantilla::vuelca(dir.path(), "kbtest").expect("volcar");
    let mut mds = Vec::new();
    md_de(dir.path(), &mut mds);

    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let mut origenes = Vec::new();
    for p in &mds {
        let c = std::fs::read_to_string(p).unwrap();
        let (Some(permalink), Some(titulo)) = (
            campo_frontmatter(&c, "permalink"),
            campo_frontmatter(&c, "title"),
        ) else {
            continue;
        };
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch) VALUES (?1, ?2, ?3, NULL, 1.0, NULL)",
            rusqlite::params![permalink, p.to_string_lossy(), titulo],
        )
        .unwrap();
        exo::aristas::reindexa_aristas_de_nota(&conn, &permalink, &c).unwrap();
        origenes.push((permalink, p.clone()));
    }
    assert!(!origenes.is_empty());
    exo::aristas::resuelve_destinos(&conn).unwrap();

    let mut stmt = conn
        .prepare("SELECT origen, destino_texto FROM aristas WHERE destino_permalink IS NULL ORDER BY origen, destino_texto")
        .unwrap();
    let rotos: Vec<String> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(|r| {
            let (o, d) = r.unwrap();
            let f = origenes
                .iter()
                .find(|(p, _)| *p == o)
                .map(|(_, f)| f.display().to_string())
                .unwrap_or(o);
            format!("{f} -> [[{d}]]")
        })
        .collect();
    assert!(
        rotos.is_empty(),
        "wikilinks sin resolver ({}):\n{}",
        rotos.len(),
        rotos.join("\n")
    );
}
