//! Oráculo del write-path (M4). El slug se verifica contra pares
//! `title`/`permalink` REALES de la KB de producción: son el contrato que ya
//! está en disco, no una convención inventada aquí.

use exo::escritor::{Rechazo, escribe_append, escribe_nueva, slug};
use std::io::Write;

fn kb_falsa() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("projects")).unwrap();
    std::fs::create_dir_all(dir.path().join("log")).unwrap();
    dir
}

fn escribe_nota(dir: &tempfile::TempDir, rel: &str, contenido: &str) -> std::path::PathBuf {
    let ruta = dir.path().join(rel);
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).unwrap();
    }
    let mut f = std::fs::File::create(&ruta).unwrap();
    f.write_all(contenido.as_bytes()).unwrap();
    ruta
}

// ---------------------------------------------------------------- slug

#[test]
fn slug_replica_los_permalinks_reales_de_la_kb() {
    // Pares extraídos de /home/paul/Documentos/proyectos/kb-demo.
    let casos = [
        (
            "exo — framework unificado de trabajo agéntico",
            "exo-framework-unificado-de-trabajo-agentico",
        ),
        (
            "kbx — explorador determinista de la KB (Go)",
            "kbx-explorador-determinista-de-la-kb-go",
        ),
        (
            "pguerrero.me — Hub personal / portfolio con Lab explorable de LLMs",
            "pguerrero.me-hub-personal-portfolio-con-lab-explorable-de-llms",
        ),
        ("core-index", "core-index"),
        ("agent-solve-it-bitacora", "agent-solve-it-bitacora"),
        ("Backlog — frentes abiertos", "backlog-frentes-abiertos"),
    ];
    for (titulo, esperado) in casos {
        assert_eq!(slug(titulo), esperado, "slug de {titulo:?}");
    }
}

#[test]
fn slug_conserva_el_punto_y_colapsa_lo_demas() {
    // El punto sobrevive (`pguerrero.me`); todo lo no alfanumérico colapsa a
    // un solo guion y nunca deja guiones en los extremos.
    assert_eq!(slug("a.b"), "a.b");
    assert_eq!(slug("  ¡hola,  mundo!  "), "hola-mundo");
    assert_eq!(slug("A___B"), "a-b");
    assert_eq!(slug("ñandú Ñ"), "nandu-n");
}

// ------------------------------------------------------- escribe_nueva

#[test]
fn nueva_genera_frontmatter_completo_y_ruta_correcta() {
    let kb = kb_falsa();
    let esc = escribe_nueva(
        kb.path(),
        "kb-demo",
        "projects",
        "Proyecto Nuevo — de prueba",
        "cuerpo de la nota\n",
        Some("stable"),
        &[],
    )
    .unwrap();

    assert_eq!(
        esc.permalink,
        "kb-demo/projects/proyecto-nuevo-de-prueba"
    );
    assert_eq!(esc.ruta_rel, "projects/Proyecto Nuevo — de prueba.md");
    assert!(esc.creada);

    let escrito = std::fs::read_to_string(kb.path().join(&esc.ruta_rel)).unwrap();
    assert!(escrito.starts_with("---\n"), "debe abrir con frontmatter");
    assert!(escrito.contains("title: Proyecto Nuevo — de prueba\n"));
    assert!(escrito.contains("type: note\n"));
    assert!(escrito.contains("permalink: kb-demo/projects/proyecto-nuevo-de-prueba\n"));
    assert!(escrito.contains("tier: stable\n"));
    assert!(escrito.ends_with("cuerpo de la nota\n"));
}

#[test]
fn nueva_respeta_el_frontmatter_que_ya_trae_el_cuerpo() {
    // M4-03: auto-completa lo que falta, JAMÁS pisa lo que el autor puso.
    let kb = kb_falsa();
    let esc = escribe_nueva(
        kb.path(),
        "kb-demo",
        "projects",
        "Con Tags",
        "---\ntags:\n- uno\n- dos\ntype: research\n---\ncuerpo\n",
        Some("log"),
        &[],
    )
    .unwrap();

    let escrito = std::fs::read_to_string(kb.path().join(&esc.ruta_rel)).unwrap();
    assert!(escrito.contains("type: research\n"), "no pisa el type dado");
    assert!(escrito.contains("- uno\n"), "conserva tags del autor");
    assert!(escrito.contains("permalink: kb-demo/projects/con-tags\n"));
    assert!(
        !escrito.contains("type: note"),
        "no debe duplicar la clave type"
    );
    assert_eq!(esc.frontmatter_completado, vec!["permalink", "title", "tier"]);
}

#[test]
fn nueva_jamas_pisa_una_nota_existente() {
    let kb = kb_falsa();
    escribe_nota(&kb, "projects/Ya Existe.md", "---\npermalink: x\n---\nviejo\n");

    let err = escribe_nueva(
        kb.path(),
        "kb-demo",
        "projects",
        "Ya Existe",
        "nuevo\n",
        None,
        &[],
    )
    .unwrap_err();

    // Colisión de ruta = error duro (exit 1), NO un gate saltable: lo correcto
    // ahí es append o edit, nunca overwrite.
    assert!(!err.is::<Rechazo>(), "colisión no es gate, es error");
    let contenido = std::fs::read_to_string(kb.path().join("projects/Ya Existe.md")).unwrap();
    assert!(contenido.contains("viejo"), "el fichero no se toca");
}

#[test]
fn nueva_con_candidatas_duplicadas_rechaza_sin_escribir() {
    let kb = kb_falsa();
    let err = escribe_nueva(
        kb.path(),
        "kb-demo",
        "projects",
        "Tema Repetido",
        "cuerpo\n",
        None,
        &[("kb-demo/projects/tema-repe".into(), 0.9)],
    )
    .unwrap_err();

    let rechazo = err.downcast_ref::<Rechazo>().expect("debe ser gate");
    assert!(matches!(rechazo, Rechazo::Duplicada { .. }));
    assert!(!kb.path().join("projects/Tema Repetido.md").exists());
}

// ------------------------------------------------------ escribe_append

#[test]
fn append_a_bitacora_no_relee_ni_reescribe_el_cuerpo() {
    let kb = kb_falsa();
    escribe_nota(
        &kb,
        "log/x-bitacora.md",
        "---\npermalink: kb-demo/log/x-bitacora\ntier: log\n---\n# X\n\nentrada vieja\n",
    );

    let esc = escribe_append(
        kb.path(),
        "log/x-bitacora.md",
        "## 2026-08-18\n\nentrada nueva\n",
        false,
    )
    .unwrap();

    assert!(!esc.creada);
    let escrito = std::fs::read_to_string(kb.path().join("log/x-bitacora.md")).unwrap();
    assert!(escrito.contains("entrada vieja"), "no pierde lo anterior");
    assert!(escrito.ends_with("## 2026-08-18\n\nentrada nueva\n"));
    assert!(
        escrito.contains("entrada vieja\n\n## 2026-08-18"),
        "separa con línea en blanco: {escrito:?}"
    );
}

#[test]
fn append_a_canon_se_rechaza_por_defecto() {
    // La defensa principal (§7.1): 52 ocurrencias del anti-patrón Delta-append
    // al canon en la historia real de la KB.
    let kb = kb_falsa();
    escribe_nota(
        &kb,
        "projects/canon.md",
        "---\npermalink: kb-demo/projects/canon\ntier: stable\n---\ncuerpo\n",
    );

    let err = escribe_append(
        kb.path(),
        "projects/canon.md",
        "## Delta 2026-08-18\n\nalgo\n",
        false,
    )
    .unwrap_err();

    let rechazo = err.downcast_ref::<Rechazo>().expect("debe ser gate");
    assert!(matches!(rechazo, Rechazo::AppendACanon { .. }));

    let intacto = std::fs::read_to_string(kb.path().join("projects/canon.md")).unwrap();
    assert!(!intacto.contains("Delta"), "no escribió nada");
}

#[test]
fn append_a_canon_forzado_escribe_y_queda_registrado() {
    let kb = kb_falsa();
    escribe_nota(
        &kb,
        "projects/canon.md",
        "---\npermalink: kb-demo/projects/canon\ntier: core\n---\ncuerpo\n",
    );

    let esc = escribe_append(kb.path(), "projects/canon.md", "excepcion\n", true).unwrap();

    assert!(esc.forzado, "el escape queda auditable en el envelope");
    let escrito = std::fs::read_to_string(kb.path().join("projects/canon.md")).unwrap();
    assert!(escrito.contains("excepcion"));
}

#[test]
fn append_a_nota_inexistente_falla_salvo_con_crea() {
    let kb = kb_falsa();
    assert!(escribe_append(kb.path(), "log/nueva-bitacora.md", "x\n", false).is_err());
}

// ------------------------------------------------------------ dup-gate

#[test]
fn dup_gate_caza_el_unico_duplicado_real_de_la_historia() {
    // 2026-07-11: se creó `log/ai-news-bitacora.md` existiendo ya la canónica
    // `ai-news-pipeline-bitacora`. Único duplicado en 153 invocaciones.
    let indexados = vec![
        "kb-demo/log/ai-news-pipeline-bitacora".to_string(),
        "kb-demo/log/exo-bitacora".to_string(),
        "kb-demo/projects/cge".to_string(),
    ];
    let candidatas = exo::escritor::dup_candidatas("ai-news-bitacora", &indexados);
    assert_eq!(candidatas.len(), 1, "debe cazar exactamente la canónica");
    assert_eq!(candidatas[0].0, "kb-demo/log/ai-news-pipeline-bitacora");
}

#[test]
fn dup_gate_no_dispara_con_bitacoras_de_frentes_distintos() {
    // El falso positivo mata al guard: bitácoras y notas que comparten UNA
    // palabra son la norma en esta KB, no una señal de duplicado.
    let indexados = vec![
        "kb-demo/log/exo-bitacora".to_string(),
        "kb-demo/log/kbx-bitacora".to_string(),
        "kb-demo/log/backlog-diario".to_string(),
        "kb-demo/Backlog — frentes abiertos".to_string(),
    ];
    for nuevo in [
        "cge-bitacora",
        "backlog-frentes-abiertos",
        "zumaia-pruebas-de-ambito-v2",
    ] {
        assert!(
            exo::escritor::dup_candidatas(nuevo, &indexados).is_empty(),
            "falso positivo con {nuevo}"
        );
    }
}

#[test]
fn solape_es_simetrico_y_acotado() {
    use exo::escritor::solape_slug;
    assert_eq!(solape_slug("a-b", "b-a"), 1.0);
    assert_eq!(solape_slug("a-b", "a-b"), 1.0);
    assert_eq!(solape_slug("a", "b"), 0.0);
    assert_eq!(solape_slug("", "a"), 0.0);
    assert_eq!(solape_slug("a-b-c", "c-b-a"), solape_slug("c-b-a", "a-b-c"));
}
