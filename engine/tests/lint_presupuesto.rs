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

/// Indexa una nota en `notas`. Desde la Task 9, `analiza` compara el índice
/// contra el disco (`index_stale`): los fixtures que esperan `ok` ya no pueden
/// dejar el índice vacío con notas en disco, hay que indexarlas de verdad.
fn indexa(conn: &rusqlite::Connection, ruta: &str) {
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
        rusqlite::params![format!("kb/{ruta}"), ruta],
    )
    .unwrap();
}

/// Le da a una nota ya indexada una arista real, para que `huerfanas` no la
/// marque huérfana. Alternativa a `kbx_orphan_ok`: ese waiver es una segunda
/// fuente de `waived` independiente de la que el fixture quiere probar, y un
/// test que solo le interesa el budget (o el índice) no debe acoplarse al
/// mecanismo de waiver de huérfanas.
fn da_arista_real(conn: &rusqlite::Connection, ruta: &str) {
    conn.execute(
        "INSERT INTO aristas (origen, destino_texto, destino_permalink)
         VALUES (?1, 'destino', NULL)",
        rusqlite::params![format!("kb/{ruta}")],
    )
    .unwrap();
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
    // Minor: sus tests solo comprobaban `tipo`. Un typo en el formato del
    // detalle (que el gate de paridad compara contra el de kbx) no lo cogía
    // nadie.
    assert_eq!(h[0].detalle, "400B > 50B (log)");
}

#[test]
fn presupuesto_excedido_respeta_la_exclusion_con_rutas_sin_filtrar() {
    // La guarda `if excluida(rel, excluidos) { continue; }` es muda con
    // `rutas` real: `walk_notas`/`walk_kb_excluyendo` ya filtran, así que
    // ningún test que use esas rutas puede falsarla (mismo hallazgo ya
    // cerrado para `frontmatter_malo`). Aquí `rutas` se construye a mano, sin
    // filtrar, con una nota bajo un top-level excluido que SÍ se reportaría
    // (12.000B > 8.500B nominal core) si la guarda desapareciera.
    let dir = kb_con(&[("archive/grande.md", nota("core", "", 12_000))]);
    let rutas = vec!["archive/grande.md".to_string()];
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert!(h.is_empty(), "archive/ no debe reportarse: {h:?}");
    assert!(w.is_empty());
}

#[test]
fn presupuesto_excedido_no_reporta_sin_aire() {
    // `Clase::SinAire { .. } | Clase::Ok => {}` fusiona dos ramas a
    // propósito: el aviso de aire es de `budget`, no un gate de `lint`. Si
    // `SinAire` se moviera a la rama de `Infractora`, `exo lint` empezaría a
    // gatear sobre notas que solo están a ras de su nominal sin excederlo
    // (del orden de 19 notas en la KB real) — el falso positivo que la doc
    // del módulo dice evitar. Ninguno de los fixtures de arriba cae en la
    // banda sin-aire, y `lint_y_budget_no_pueden_discrepar` compara por
    // `path` así que tampoco la pillaría.
    //
    // objetivo_poda(8500) = 7.391, así que 7.392..=8.500 es la banda sin aire
    // para una nota `core` sin override. 8.000 cae dentro.
    let dir = kb_con(&[("core/al_limite.md", nota("core", "", 8_000))]);
    let rutas = exo::walker::walk_notas(dir.path(), &exo::presupuesto::EXCLUIDOS).unwrap();
    let (h, w) =
        lint::presupuesto_excedido(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS)
            .unwrap();
    assert!(h.is_empty(), "sin aire no debe gatear: {h:?}");
    assert!(w.is_empty());
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
    // Minor: mismo razonamiento que en el test de infractora de arriba.
    assert_eq!(w[0].detalle, "12000B ≤ 20000B (waived: kbx_budget_max)");
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
fn deriva_de_prosa_respeta_la_exclusion_con_rutas_sin_filtrar() {
    // Misma guarda muda que en `presupuesto_excedido`, mismo remedio: `rutas`
    // a mano, sin filtrar, con una nota `core` bajo un top-level excluido que
    // SÍ citaría una deriva (1000 != 8500) si la guarda desapareciera.
    let dir = kb_con(&[(
        "docs/c.md",
        "---\ntier: core\n---\n\nPresupuesto: core 1.000 B.\n".to_string(),
    )]);
    let rutas = vec!["docs/c.md".to_string()];
    let h =
        lint::deriva_de_prosa(dir.path(), &rutas, NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(h.is_empty(), "docs/ no debe reportarse: {h:?}");
}

#[test]
fn el_informe_de_lint_es_ok_con_solo_waived() {
    // `ok: hallazgos.is_empty() && waived.is_empty()` es la mutación que
    // rompería esto: un waiver es una declaración aceptada, no debe mover el
    // gate. Los dos tests existentes de `analiza` tienen `waived` vacío en su
    // fixture y no pillarían esa mutación.
    let dir = kb_con(&[("core/w.md", nota("core", "kbx_budget_max: 20000\n", 12_000))]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    indexa(&conn, "core/w.md");
    // Arista real, no `kbx_orphan_ok`: ese waiver metería una segunda fuente
    // de `waived` independiente del budget, y el fixture dejaría de falsar la
    // mutación de `Clase::Waived` que silencia el hallazgo de presupuesto —
    // el waiver de huérfana lo taparía sin que este test se diera cuenta.
    da_arista_real(&conn, "core/w.md");
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(informe.hallazgos.is_empty());
    assert!(
        !informe.waived.is_empty(),
        "el fixture debe producir un waived"
    );
    assert!(informe.ok, "un waiver no debe apagar ok");
}

#[test]
fn el_informe_de_lint_es_ok_solo_sin_hallazgos() {
    let dir = kb_con(&[("core/ok.md", nota("core", "", 100))]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    // Task 9: un índice vacío con notas en disco ya no puede salir `ok`
    // (`index_stale`), así que este fixture tiene que indexar de verdad.
    indexa(&conn, "core/ok.md");
    // Arista real, no `kbx_orphan_ok`: ya indexada y sin ninguna arista, la
    // nota saldría huérfana por un motivo que este test no quiere probar.
    da_arista_real(&conn, "core/ok.md");
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
    // A7: `schema_drift` murió y no vuelve; `index_stale` ocupa su hueco.
    // Siete tipos otra vez, pero un siete distinto.
    let dir = kb_con(&[("suelto.txt", "x".to_string())]);
    let conn = exo::abre_db_en_memoria().unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    let informe =
        lint::analiza(&conn, dir.path(), NOMINALES, &exo::presupuesto::EXCLUIDOS).unwrap();
    assert!(!informe.ok);
    assert!(informe.hallazgos.iter().all(|h| h.tipo != "schema_drift"));
}
