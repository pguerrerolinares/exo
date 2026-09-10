//! Task 6 — `recolecta`: las declaraciones del árbol.
//!
//! Diferencia con `presupuesto::analiza` (que este fichero no toca ni
//! reutiliza): `recolecta` trae TODA nota con `kbx_budget_max` declarado,
//! incluidas las que están muy por debajo de su tier y que `budget` nunca
//! reporta. El trinquete juzga declaraciones; `budget` juzga tamaños.

use exo::presupuesto::NOMINALES;
use exo::trinquete::{self, Declarada};
use std::fs;

/// Mismo helper que `engine/tests/presupuesto.rs`: `\n` explícito en el
/// frontmatter para que el tamaño del fixture sea el mismo en los tres SO
/// del CI (nunca CRLF).
fn nota(dir: &std::path::Path, rel: &str, tier: &str, extra: &str, relleno: usize) {
    let ruta = dir.join(rel);
    fs::create_dir_all(ruta.parent().unwrap()).unwrap();
    let cabecera = format!("---\ntier: {tier}\n{extra}---\n");
    let cuerpo = "x".repeat(relleno.saturating_sub(cabecera.len()));
    fs::write(&ruta, format!("{cabecera}{cuerpo}")).unwrap();
}

fn encuentra<'a>(declaradas: &'a [Declarada], ruta: &str) -> Option<&'a Declarada> {
    declaradas.iter().find(|d| d.ruta == ruta)
}

#[test]
fn recolecta_trae_toda_declaracion_no_solo_las_infractoras() {
    // 1 KB, muy por debajo del tier `stable` (12.500 nominal): `budget`
    // nunca reportaría esta nota, pero `recolecta` sí, porque declara techo.
    let dir = tempfile::tempdir().unwrap();
    nota(
        dir.path(),
        "s.md",
        "stable",
        "kbx_budget_max: 12000\n",
        1_000,
    );
    let declaradas = trinquete::recolecta(dir.path(), NOMINALES, &[]).unwrap();
    let d = encuentra(&declaradas, "s.md").expect("s.md debe aparecer, aunque no infrinja nada");
    assert_eq!(d.tier, "stable");
    assert_eq!(d.max, 12_000);
    assert_eq!(d.tier_presupuesto, NOMINALES.stable);
}

#[test]
fn recolecta_respeta_los_excluidos() {
    let dir = tempfile::tempdir().unwrap();
    nota(
        dir.path(),
        "archive/viejo.md",
        "log",
        "kbx_budget_max: 5000\n",
        500,
    );
    let declaradas = trinquete::recolecta(dir.path(), NOMINALES, &["archive"]).unwrap();
    assert!(encuentra(&declaradas, "archive/viejo.md").is_none());
}

#[test]
fn recolecta_ignora_las_notas_sin_techo_declarado() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "sin_techo.md", "core", "", 500);
    let declaradas = trinquete::recolecta(dir.path(), NOMINALES, &[]).unwrap();
    assert!(declaradas.is_empty());
}

#[test]
fn el_tamano_es_el_del_fichero_en_bytes() {
    let dir = tempfile::tempdir().unwrap();
    nota(
        dir.path(),
        "core/a.md",
        "core",
        "kbx_budget_max: 8000\n",
        3_000,
    );
    let declaradas = trinquete::recolecta(dir.path(), NOMINALES, &[]).unwrap();
    let d = encuentra(&declaradas, "core/a.md").expect("core/a.md debe aparecer");
    let en_disco = fs::metadata(dir.path().join("core/a.md")).unwrap().len() as i64;
    assert_eq!(d.tamano, en_disco);
}
