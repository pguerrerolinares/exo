use exo::lint;
use std::fs;

fn kb_con(ficheros: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

#[test]
fn dirs_duplicados_agrupa_por_basename_bajo_padres_distintos() {
    let dir = kb_con(&[
        ("projects/notas/a.md", "---\ntier: log\n---\n"),
        ("learnings/notas/b.md", "---\ntier: log\n---\n"),
        ("core/unico/c.md", "---\ntier: core\n---\n"),
    ]);
    let (dirs, _) =
        exo::walker::walk_kb_excluyendo(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::dirs_duplicados(&dirs, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].tipo, "duplicate_dir");
    assert_eq!(h[0].ruta, "notas");
    assert_eq!(h[0].detalle, "learnings/notas, projects/notas");
}

#[test]
fn dirs_duplicados_no_mira_dentro_de_lo_excluido() {
    let dir = kb_con(&[
        ("projects/notas/a.md", "---\ntier: log\n---\n"),
        ("archive/notas/b.md", "---\ntier: log\n---\n"),
    ]);
    let (dirs, _) =
        exo::walker::walk_kb_excluyendo(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::dirs_duplicados(&dirs, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "archive/ no cuenta como segundo padre: {h:?}");
}

#[test]
fn frontmatter_malo_distingue_ausencia_de_ilegalidad() {
    let dir = kb_con(&[
        ("sin.md", "# sin frontmatter\n"),
        ("ilegal.md", "---\ntier: inventado\n---\n"),
        ("nbsp.md", "---\ntier: co\u{a0}re\n---\n"),
        ("bien.md", "---\ntier: core\n---\n"),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::frontmatter_malo(dir.path(), &rutas, &exo::presupuesto::EXCLUIDOS).unwrap();
    let por_ruta: std::collections::HashMap<_, _> = h
        .iter()
        .map(|f| (f.ruta.as_str(), f.detalle.as_str()))
        .collect();
    assert_eq!(por_ruta.get("sin.md"), Some(&"NOTIER"));
    assert_eq!(por_ruta.get("ilegal.md"), Some(&"tier ilegal: inventado"));
    // A1: el NBSP no se absuelve, así que la nota sale como tier ilegal.
    assert_eq!(por_ruta.get("nbsp.md"), Some(&"tier ilegal: co\u{a0}re"));
    assert!(!por_ruta.contains_key("bien.md"));
}

#[test]
fn frontmatter_malo_detecta_presencia_con_valor_no_con_tier_vacio() {
    // El fixture crítico: en las cuatro notas de arriba, `valor("tier").is_some()`
    // y `!tier().is_empty()` coinciden siempre, así que ese test NO falsa la
    // distinción que el brief exige. Aquí el valor tras "tier:" es un solo
    // VERTICAL TAB (\u{b}): `valor()` lo ve declarado (`Some("\u{b}")`), pero
    // `tier()` lo filtra como espacio ASCII y devuelve "". Si `frontmatter_malo`
    // decidiera presencia mirando `tier().is_empty()`, esta nota saldría como
    // NOTIER; con `valor().is_some()` sale como tier ilegal (con "" como valor,
    // que sí es la clasificación correcta: la clave se declaró, mal).
    let dir = kb_con(&[("vt.md", "---\ntier: \u{b}\n---\n")]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h = lint::frontmatter_malo(dir.path(), &rutas, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].ruta, "vt.md");
    assert_eq!(h[0].detalle, "tier ilegal: ");
}

#[test]
fn ficheros_en_raiz_solo_mira_profundidad_cero() {
    let dir = kb_con(&[
        ("nota.md", "---\ntier: core\n---\n"),
        ("suelto.txt", "x"),
        ("projects/otro.txt", "x"),
    ]);
    fs::write(dir.path().join(".oculto"), "x").unwrap();
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    let rutas: Vec<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["suelto.txt"]);
    assert_eq!(h[0].tipo, "root_file");
}

#[test]
fn ficheros_en_raiz_ignora_la_exclusion_a_proposito() {
    // A6: la exclusión opera sobre el primer segmento de rutas de profundidad
    // >=1. En la raíz no hay segmento que excluir, así que este check no la
    // consulta — y eso es correcto, no una inconsistencia que arreglar.
    let dir = kb_con(&[("nota.md", "---\ntier: core\n---\n")]);
    fs::write(
        dir.path().join("docs"),
        "un FICHERO llamado docs, no un dir",
    )
    .unwrap();
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].ruta, "docs");
}

#[test]
fn las_claves_del_hallazgo_estan_en_ingles() {
    let dir = kb_con(&[("suelto.txt", "x")]);
    let h = lint::ficheros_en_raiz(dir.path()).unwrap();
    let v = serde_json::to_value(&h[0]).unwrap();
    for k in ["type", "path", "detail"] {
        assert!(v.get(k).is_some(), "falta {k}: {v}");
    }
    for k in ["tipo", "ruta", "detalle"] {
        assert!(v.get(k).is_none(), "sobrevive la clave española {k}");
    }
    // La sola presencia/ausencia de nombres de clave no detecta un swap entre
    // dos campos del mismo tipo (String): si `ruta` se serializara bajo
    // "detail" y `detalle` bajo "path", las dos comprobaciones de arriba
    // seguirían en verde. Comparar el VALOR bajo cada clave contra el campo de
    // origen sí lo pilla.
    assert_eq!(v["type"], h[0].tipo);
    assert_eq!(v["path"], h[0].ruta);
    assert_eq!(v["detail"], h[0].detalle);
}
