use exo::presupuesto::{self, NOMINALES};
use std::fs;

fn nota(dir: &std::path::Path, rel: &str, tier: &str, extra: &str, relleno: usize) {
    let ruta = dir.join(rel);
    fs::create_dir_all(ruta.parent().unwrap()).unwrap();
    let cabecera = format!("---\ntier: {tier}\n{extra}---\n");
    let cuerpo = "x".repeat(relleno.saturating_sub(cabecera.len()));
    fs::write(&ruta, format!("{cabecera}{cuerpo}")).unwrap();
}

#[test]
fn las_filas_de_tier_van_en_orden_y_log_es_ilimitado() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/a.md", "core", "", 100);
    nota(dir.path(), "log/b.md", "log", "", 99_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let tiers: Vec<&str> = informe.tiers.iter().map(|f| f.tier.as_str()).collect();
    assert_eq!(tiers, vec!["core", "stable", "log"]);
    let log = &informe.tiers[2];
    assert_eq!(log.presupuesto, 0);
    // delta solo tiene sentido con presupuesto: con 0 se reporta 0, no
    // "bytes - 0", que sería el tamaño entero disfrazado de exceso.
    assert_eq!(log.delta, 0);
    assert!(!log.excedido);
}

#[test]
fn una_nota_sin_tier_legal_va_a_notier_y_gatea() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "x.md", "inventado", "", 50);
    nota(dir.path(), "y.md", "co\u{a0}re", "", 50); // A1: el NBSP la deja ilegal
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(informe.notier, vec!["x.md".to_string(), "y.md".to_string()]);
    assert!(informe.excedido());
}

#[test]
fn las_infractoras_van_por_exceso_descendente_y_desempatan_por_ruta() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/mucho.md", "core", "", 12_000);
    nota(dir.path(), "core/poco.md", "core", "", 9_000);
    nota(dir.path(), "core/b.md", "core", "", 9_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let rutas: Vec<&str> = informe
        .infractoras
        .iter()
        .map(|o| o.ruta.as_str())
        .collect();
    assert_eq!(rutas, vec!["core/mucho.md", "core/b.md", "core/poco.md"]);
}

#[test]
fn el_waiver_saca_de_infractoras_y_reporta_el_override() {
    let dir = tempfile::tempdir().unwrap();
    nota(
        dir.path(),
        "core/w.md",
        "core",
        "kbx_budget_max: 20000\n",
        12_000,
    );
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.infractoras.is_empty());
    assert_eq!(informe.waived.len(), 1);
    assert_eq!(informe.waived[0].presupuesto, 20_000);
    assert!(!informe.excedido());
}

#[test]
fn el_aire_es_informativo_y_no_gatea() {
    let dir = tempfile::tempdir().unwrap();
    // stable: objetivo de poda 10.869. 12.000 está a ras pero no excede.
    nota(dir.path(), "s.md", "stable", "", 12_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(informe.sin_aire.len(), 1);
    assert_eq!(informe.sin_aire[0].presupuesto, 12_500);
    assert!(informe.infractoras.is_empty());
    assert!(!informe.excedido(), "el aire NUNCA mueve el exit code");
}

#[test]
fn las_listas_vacias_serializan_como_array_y_nunca_como_null() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/ok.md", "core", "", 100);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let v = serde_json::to_value(&informe).unwrap();
    for k in ["offenders", "waived", "no_air", "notier"] {
        assert!(v[k].is_array(), "{k} no es array: {}", v[k]);
    }
}

#[test]
fn las_claves_del_informe_estan_en_ingles() {
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/big.md", "core", "", 12_000);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let v = serde_json::to_value(&informe).unwrap();
    let o = &v["offenders"][0];
    for k in ["path", "tier", "size_bytes", "budget"] {
        assert!(o.get(k).is_some(), "falta {k}: {o}");
    }
    for k in ["ruta", "tamano_bytes", "presupuesto"] {
        assert!(o.get(k).is_none(), "sobrevive la clave española {k}");
    }
    assert_eq!(o["path"], "core/big.md");
    assert_eq!(o["budget"], 8500);
    let f = &v["tiers"][0];
    for k in ["tier", "notes", "bytes", "budget", "delta", "exceeded"] {
        assert!(f.get(k).is_some(), "falta {k} en tiers[]: {f}");
    }
}

#[test]
fn el_tamano_sale_del_disco_y_no_del_indice() {
    // El invariante que hace comparable el informe con el de kbx: bytes del
    // fichero entero en disco, frontmatter incluido.
    let dir = tempfile::tempdir().unwrap();
    nota(dir.path(), "core/a.md", "core", "", 500);
    let informe = presupuesto::analiza(dir.path(), NOMINALES, &presupuesto::EXCLUIDOS).unwrap();
    let en_disco = fs::metadata(dir.path().join("core/a.md")).unwrap().len() as i64;
    assert_eq!(informe.tiers[0].bytes, en_disco);
}
