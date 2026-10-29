//! El `core-index.md` de la semilla debe caber bajo el cap de 6.144 B CON EL
//! 15% DE AIRE que exige la propia doctrina que la nota predica (≤ 5.222 B).
//! Sin este gate la semilla nace mordiendo su presupuesto el primer día: un
//! índice que nace pegado a su techo está programado para morderlo.

const CAP: usize = 6_144;
const AIRE: f64 = 0.15;

fn limite() -> usize {
    (CAP as f64 * (1.0 - AIRE)) as usize // 5_222
}

#[test]
fn el_core_index_semilla_cabe_con_15_por_ciento_de_aire() {
    let bytes = include_str!("../kb-template/core/core-index.md").len();
    assert!(
        bytes <= limite(),
        "core-index semilla: {bytes} B > límite {} B (cap {CAP} con {}% de aire). \
         Retira entradas del índice; NO subas el cap ni comprimas las entradas vivas.",
        limite(),
        (AIRE * 100.0) as u32
    );
}

#[test]
fn el_limite_es_el_declarado_en_la_spec() {
    assert_eq!(limite(), 5_222, "el límite de G3 es 5.222 B literal");
}

#[test]
fn ningun_learning_semilla_pasa_su_techo_con_aire() {
    let learnings: Vec<_> = exo::plantilla::FICHEROS
        .iter()
        .filter(|(ruta, _)| ruta.starts_with("learnings/") && ruta.ends_with(".md"))
        .collect();
    assert!(
        learnings.len() >= 10,
        "el filtro solo ve {} learnings; se esperaban >= 10",
        learnings.len()
    );
    for (ruta, contenido) in learnings {
        let bytes = contenido.len();
        assert!(
            exo::presupuesto::tiene_aire(12_500, bytes as i64),
            "{ruta}: {bytes} B sin 15% de aire bajo el techo de 12.500 B"
        );
    }
}
