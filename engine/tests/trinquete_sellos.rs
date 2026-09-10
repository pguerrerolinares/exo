//! Task 3 — el tipo de los sellos, su parseo y su IO.
//!
//! El fixture `fixtures/kbx-ratchet-real.json` es una copia literal del
//! `.kbx-ratchet.json` de `wisdom-paul` (11 entradas, con em-dash y acentos):
//! un port que se atragante con el Unicode real de la KB no sirve.

use exo::trinquete::{self, Sellos};
use std::collections::BTreeMap;

const FIXTURE_REAL: &str = include_str!("fixtures/kbx-ratchet-real.json");

#[test]
fn parsea_el_sello_real_de_la_kb() {
    let sellos = trinquete::parsea_sellos(FIXTURE_REAL).expect("el fixture real debe parsear");
    assert_eq!(sellos.len(), 11);
    assert_eq!(sellos["core/doctrina-agentes.md"], 20000);
    // La clave con em-dash: es la que rompería un port que asuma ASCII.
    assert_eq!(
        sellos["projects/pguerrero.me — Hub personal - portfolio con Lab explorable de LLMs.md"],
        14000
    );
}

#[test]
fn un_sello_ausente_es_vacio_no_error() {
    // Invariante, no comodidad: es lo que permite que una KB nueva no esté
    // en rojo el día de instalar el trinquete.
    let dir = tempfile::tempdir().expect("tempdir");
    let sellos = trinquete::carga(dir.path()).expect("fichero ausente no es error");
    assert!(sellos.is_empty());
}

#[test]
fn un_ceilings_ausente_es_vacio() {
    // El Go trata `doc.Ceilings == nil` como `Seals{}`; aquí es
    // `#[serde(default)]` sobre el campo.
    let sellos = trinquete::parsea_sellos("{}").expect("sin ceilings no es error");
    assert!(sellos.is_empty());
}

#[test]
fn un_sello_corrupto_es_error() {
    // Sub-invariante 3 del ítem 7 de la spec: "sello corrupto en HEAD es
    // error, no abstención".
    let resultado = trinquete::parsea_sellos("{\"ceilings\":{");
    assert!(resultado.is_err());
}

#[test]
fn serializa_con_el_formato_exacto_de_kbx() {
    let mut sellos: Sellos = BTreeMap::new();
    sellos.insert("a.md".to_string(), 100);
    sellos.insert("b.md".to_string(), 200);

    let esperado = "{\n  \"ceilings\": {\n    \"a.md\": 100,\n    \"b.md\": 200\n  }\n}\n";
    assert_eq!(trinquete::serializa_sellos(&sellos), esperado);
}

#[test]
fn round_trip_serializa_y_parsea() {
    let mut sellos: Sellos = BTreeMap::new();
    sellos.insert("a.md".to_string(), 100);
    sellos.insert("b.md".to_string(), 200);

    let ida_y_vuelta = trinquete::parsea_sellos(&trinquete::serializa_sellos(&sellos))
        .expect("lo que serializamos debe parsear");
    assert_eq!(ida_y_vuelta, sellos);
}

#[test]
fn escribe_sellos_y_carga_los_recupera() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut sellos: Sellos = BTreeMap::new();
    sellos.insert("core/x.md".to_string(), 8500);

    trinquete::escribe_sellos(dir.path(), &sellos).expect("escribir no debe fallar");
    let recuperado = trinquete::carga(dir.path()).expect("cargar no debe fallar");
    assert_eq!(recuperado, sellos);

    // El fichero en disco tiene el nombre exacto que A5 fija.
    assert!(dir.path().join(trinquete::FICHERO_SELLO).exists());
}
