use exo::lint;
use exo::presupuesto::NOMINALES;
use std::fs;

fn kb_con(ficheros: &[(&str, String)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

fn nota(tier: &str, extra: &str, relleno: usize) -> String {
    let cabecera = format!("---\ntier: {tier}\n{extra}---\n");
    let cuerpo = "x".repeat(relleno.saturating_sub(cabecera.len()));
    format!("{cabecera}{cuerpo}")
}

#[test]
fn el_override_dispara_aunque_el_tier_sea_ilimitado() {
    // LA regresión de paridad de kbx: `tier: log` (nominal 0) con
    // kbx_budget_max: 50 y cuerpo mayor. Un `if tierBudget <= 0 { continue }`
    // prematuro hacía que doctor callase mientras budget disparaba — dos
    // comandos discrepando sobre la misma KB. Aquí no puede pasar: los dos
    // llaman a presupuesto::clasifica.
    let dir = kb_con(&[("l.md", nota("log", "kbx_budget_max: 50\n", 400))]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert_eq!(
        h.len(),
        1,
        "el override sobre un tier ilimitado tiene que disparar"
    );
    assert!(w.is_empty());
    assert_eq!(h[0].tipo, "budget_exceeded");
}

#[test]
fn lint_y_budget_no_pueden_discrepar() {
    // El test de paridad, como lo que es: una sola implementación. Se compara
    // el conjunto de rutas que cada verbo considera infractoras.
    let dir = kb_con(&[
        ("core/grande.md", nota("core", "", 12_000)),
        ("core/ok.md", nota("core", "", 100)),
        ("core/w.md", nota("core", "kbx_budget_max: 20000\n", 12_000)),
        ("log/l.md", nota("log", "kbx_budget_max: 50\n", 400)),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    let informe =
        exo::presupuesto::analiza(dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();

    let de_lint: std::collections::BTreeSet<&str> = h.iter().map(|f| f.ruta.as_str()).collect();
    let de_budget: std::collections::BTreeSet<&str> = informe
        .infractoras
        .iter()
        .map(|o| o.ruta.as_str())
        .collect();
    assert_eq!(
        de_lint, de_budget,
        "lint y budget discrepan sobre quién es infractora"
    );

    let waived_lint: std::collections::BTreeSet<&str> = w.iter().map(|f| f.ruta.as_str()).collect();
    let waived_budget: std::collections::BTreeSet<&str> =
        informe.waived.iter().map(|o| o.ruta.as_str()).collect();
    assert_eq!(
        waived_lint, waived_budget,
        "lint y budget discrepan sobre los waivers"
    );

    // El test de paridad de arriba compara por `path`, no por `type` (BTreeSet
    // de `.ruta`): un typo en el literal de tipo de la rama `Waived` no lo
    // haría caer. `core/w.md` es la única waived de este fixture (determinista
    // por el sort_by), así que este assert sí lo cubre.
    assert_eq!(w[0].tipo, "budget_exceeded");
}

#[test]
fn las_notas_sin_tier_legal_no_disparan_dos_veces() {
    // Una deriva, un tipo de hallazgo: ya las coge bad_frontmatter.
    let dir = kb_con(&[("x.md", nota("inventado", "", 99_000))]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, _) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert!(h.is_empty());
}

#[test]
fn la_deriva_de_prosa_solo_mira_notas_core() {
    // OJO con la frase: la regex exige que la cifra vaya PEGADA al tier (solo
    // `:` o espacios entre medias). "El presupuesto de core es 3.900 B" NO
    // matchea, por el `es`. Esa exigencia es deliberada —es lo que evita que
    // "el presupuesto y las 3 notas core" cuente como cita— pero convierte
    // cualquier frase de prueba mal redactada en un test que pasa sin probar
    // nada.
    let citando = |tier: &str, cifra: &str| {
        format!("---\ntier: {tier}\n---\n\nPresupuesto duro: {tier} {cifra} B.\n")
    };
    let dir = kb_con(&[
        ("core/c.md", citando("core", "3.900")),
        ("stable/s.md", citando("stable", "3.900")),
    ]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].ruta, "core/c.md");
    assert_eq!(h[0].tipo, "budget_prose_drift");
    assert_eq!(h[0].detalle, "cita core 3900B, el tool aplica 8500B");
}

#[test]
fn la_deriva_de_prosa_acepta_el_separador_de_miles_y_la_cifra_correcta() {
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nPresupuesto: core 8.500 bytes, stable 12500 B.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "las cifras correctas no derivan: {h:?}");
}

#[test]
fn la_deriva_de_prosa_parsea_las_cuatro_grafias() {
    // Portado de TestBudgetProseDriftParsesFigureVariants. Es el ÚNICO test que
    // guarda la regex: cuatro grafías con cuatro cifras distintas, para que un
    // miss del parser no pueda pasar por "no había deriva".
    let cuerpo = "---\ntier: core\n---\n\n\
        Presupuesto: core 1.000 B\n\
        Presupuesto: core 2000 bytes\n\
        Presupuesto: core: 3.000\n\
        Presupuesto: core   4000 B\n";
    let dir = kb_con(&[("c.md", cuerpo.to_string())]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    let citadas: Vec<&str> = h.iter().map(|f| f.detalle.as_str()).collect();
    assert_eq!(h.len(), 4, "una grafía no se parseó: {citadas:?}");
    for esperada in [
        "cita core 1000B",
        "cita core 2000B",
        "cita core 3000B",
        "cita core 4000B",
    ] {
        assert!(
            citadas.iter().any(|d| d.starts_with(esperada)),
            "falta {esperada}: {citadas:?}"
        );
    }
}

#[test]
fn la_deriva_de_prosa_calla_ante_una_mencion_vaga() {
    // Falsos positivos son peores que fallos aquí: un gate que grita se ignora.
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nHablemos del presupuesto y de las 3 notas core.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "mención vaga tratada como cita: {h:?}");
}

#[test]
fn la_deriva_de_prosa_no_dispara_sin_la_palabra_presupuesto() {
    // La primera regex de la cascada (`LINEA_DE_PRESUPUESTO`) es la que evita
    // que cualquier "core: 3000" suelto en la prosa cuente como cita: sin
    // "presupuesto"/"budget" en la línea, ni se intenta el parseo de la
    // cifra. La cifra (3000) difiere a propósito del nominal real (8500)
    // para que, si la guarda se quitara, `TIER_Y_CIFRA` SÍ generase un
    // hallazgo — si no difiriera, el test pasaría igual con la guarda rota.
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nTecho declarado: core: 3000 unidades por nota.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        h.is_empty(),
        "una línea sin 'presupuesto'/'budget' no debe citar: {h:?}"
    );
}

#[test]
fn la_deriva_de_prosa_no_matchea_si_la_cifra_no_va_pegada_al_tier() {
    // El caso motivador de la exigencia "pegada" (`[:\s]+`, no `.*`): "El
    // presupuesto de core es 3.900 B" tiene un "es" entre el tier y la cifra
    // y NO debe contar como cita, aunque la línea sí mencione "presupuesto"
    // (así que la guarda que se está falsando aquí es `TIER_Y_CIFRA`, no
    // `LINEA_DE_PRESUPUESTO`).
    let dir = kb_con(&[(
        "c.md",
        "---\ntier: core\n---\n\nEl presupuesto de core es 3.900 B.\n".to_string(),
    )]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(
        h.is_empty(),
        "el 'es' entre tier y cifra no debe contar como cita: {h:?}"
    );
}

#[test]
fn el_informe_de_lint_es_ok_solo_sin_hallazgos() {
    let dir = kb_con(&[("core/ok.md", nota("core", "", 100))]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.ok);
    let v = serde_json::to_value(&informe).unwrap();
    assert!(v["findings"].is_array());
    assert!(v["waived"].is_array());
    assert!(v.get("hallazgos").is_none());
}

#[test]
fn lint_no_emite_schema_drift() {
    // A7: son seis tipos, no siete.
    let dir = kb_con(&[("suelto.txt", "x".to_string())]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!informe.ok);
    assert!(informe.hallazgos.iter().all(|h| h.tipo != "schema_drift"));
}
