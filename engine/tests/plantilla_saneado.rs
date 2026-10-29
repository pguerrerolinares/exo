//! La semilla que se reparte no debe arrastrar rastro del mantenedor.

use regex::Regex;

const EXCEPCION: &str = ".kbx-ratchet.json";

fn rastros(texto: &str) -> Vec<String> {
    let re = Regex::new(r"(?i)paul|bizkaia|kbx|lighthouses|fable|wisdom|2026-|exo#[0-9]+").unwrap();
    re.find_iter(&texto.replace(EXCEPCION, ""))
        .map(|m| m.as_str().to_string())
        .collect()
}

#[test]
fn rastros_caza_un_positivo_conocido() {
    let r = rastros("lo midió Paul el 2026-10-12 en exo#42");
    assert_eq!(r.len(), 3, "esperaba paul, 2026- y exo#42; hubo {r:?}");
}

#[test]
fn rastros_no_cuenta_el_fichero_del_trinquete() {
    assert_eq!(
        rastros("sellado en .kbx-ratchet.json"),
        Vec::<String>::new()
    );
}

#[test]
fn la_semilla_no_filtra_rastro_del_mantenedor() {
    let mut hallazgos = Vec::new();
    for (ruta, contenido) in exo::plantilla::FICHEROS {
        for m in rastros(contenido) {
            hallazgos.push(format!("{ruta}: {m}"));
        }
    }
    assert!(
        hallazgos.is_empty(),
        "rastro del mantenedor en la semilla:\n{}",
        hallazgos.join("\n")
    );
}
