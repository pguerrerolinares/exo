//! Task 11 — `--seal`: `min(actual, declarado)`, nunca sube, y atómico: o
//! sella todo o no sella nada.
//!
//! `sella` y `violaciones_de_aire` son puras (no tocan disco). La
//! atomicidad se ejercita aquí igual que lo hará `exo ratchet --seal`
//! (Task 12, todavía sin cablear): computa `siguiente`, comprueba las
//! violaciones, y escribe con `escribe_sellos` solo si vienen vacías —
//! `intenta_sellar` es ese mismo protocolo, para que los tests de esta
//! suite lo ejerzan de punta a punta contra disco real.

use exo::presupuesto::NOMINALES;
use exo::trinquete::{self, Declarada, Hallazgo, Sellos, Tipo};
use std::path::Path;

fn sellos(pares: &[(&str, i64)]) -> Sellos {
    pares.iter().map(|(r, t)| (r.to_string(), *t)).collect()
}

fn declarada(ruta: &str, tier: &str, max: i64, tier_presupuesto: i64, tamano: i64) -> Declarada {
    Declarada {
        ruta: ruta.to_string(),
        tier: tier.to_string(),
        max,
        tier_presupuesto,
        tamano,
    }
}

/// El protocolo de `--seal`: computa el siguiente estado, comprueba las
/// violaciones de aire, y escribe solo si no hay ninguna. Devuelve las
/// violaciones (vacío si selló).
fn intenta_sellar(kb: &Path, actual: &Sellos, declaradas: &[Declarada]) -> Vec<Hallazgo> {
    let siguiente = trinquete::sella(actual, declaradas);
    let violaciones = trinquete::violaciones_de_aire(actual, &siguiente, declaradas);
    if violaciones.is_empty() {
        trinquete::escribe_sellos(kb, &siguiente).unwrap();
    }
    violaciones
}

// `sella`: el valor menor, nunca sube.

#[test]
fn sellar_toma_el_valor_menor_y_nunca_sube() {
    let actual = sellos(&[("a.md", 12000)]);
    let declaradas = [declarada("a.md", "core", 15000, NOMINALES.core, 10000)];
    let siguiente = trinquete::sella(&actual, &declaradas);
    assert_eq!(
        siguiente.get("a.md"),
        Some(&12000),
        "el techo nunca sube: se queda en el sello existente, no en la declaración más alta"
    );
}

#[test]
fn sella_baja_el_techo_cuando_la_declaracion_es_menor() {
    // El contraste del test anterior: `sella` no es "siempre conserva lo
    // que había", es de verdad un mínimo — si la nueva declaración es MÁS
    // BAJA, el techo baja.
    let actual = sellos(&[("a.md", 20000)]);
    let declaradas = [declarada("a.md", "core", 9000, NOMINALES.core, 8000)];
    let siguiente = trinquete::sella(&actual, &declaradas);
    assert_eq!(siguiente.get("a.md"), Some(&9000));
}

#[test]
fn sella_declara_un_sello_nuevo_para_una_ruta_sin_techo_previo() {
    let actual = Sellos::new();
    let declaradas = [declarada("a.md", "core", 9000, NOMINALES.core, 8000)];
    let siguiente = trinquete::sella(&actual, &declaradas);
    assert_eq!(siguiente.get("a.md"), Some(&9000));
}

#[test]
fn sella_conserva_un_sello_huerfano_sin_declarada() {
    // Un sello sin declaración correspondiente (huérfano, o simplemente una
    // ruta que esta pasada no toca) se conserva tal cual: retirarlo
    // devolvería el margen que el trinquete existe para retener.
    let actual = sellos(&[("huerfano.md", 5000)]);
    let siguiente = trinquete::sella(&actual, &[]);
    assert_eq!(siguiente.get("huerfano.md"), Some(&5000));
}

// La atomicidad: o sella todo o no sella nada.

#[test]
fn sellar_se_niega_y_no_escribe_nada_si_falta_aire() {
    let dir = tempfile::tempdir().unwrap();
    let actual = sellos(&[("a.md", 20000), ("b.md", 9000)]);
    trinquete::escribe_sellos(dir.path(), &actual).unwrap();
    let contenido_antes =
        std::fs::read_to_string(dir.path().join(trinquete::FICHERO_SELLO)).unwrap();

    let declaradas = [
        // a.md: con aire de sobra. Si el intento fuera parcial, esta nota
        // se escribiría igual.
        declarada("a.md", "core", 15000, NOMINALES.core, 13000),
        // b.md: sin aire (techo_minimo(8000) == 9200 > 8000).
        declarada("b.md", "core", 8000, NOMINALES.core, 8000),
    ];

    let violaciones = intenta_sellar(dir.path(), &actual, &declaradas);
    assert!(
        !violaciones.is_empty(),
        "b.md no tiene aire, el intento debe reportar violaciones"
    );

    let contenido_despues =
        std::fs::read_to_string(dir.path().join(trinquete::FICHERO_SELLO)).unwrap();
    assert_eq!(
        contenido_antes, contenido_despues,
        "ni siquiera a.md (que sí tenía aire) debe escribirse: o sella todo o no sella nada"
    );
    let de_disco = trinquete::carga(dir.path()).unwrap();
    assert_eq!(
        de_disco.get("a.md"),
        Some(&20000),
        "a.md debe seguir en su valor original, no en el 15000 que hubiera bajado"
    );
}

#[test]
fn sellar_lista_todos_los_infractores_no_solo_el_primero() {
    let dir = tempfile::tempdir().unwrap();
    let actual = sellos(&[("a.md", 9000), ("b.md", 9000), ("c.md", 20000)]);

    let declaradas = [
        declarada("a.md", "core", 8000, NOMINALES.core, 8000), // sin aire
        declarada("b.md", "core", 8500, NOMINALES.core, 8500), // sin aire
        declarada("c.md", "core", 15000, NOMINALES.core, 13000), // con aire
    ];

    let violaciones = intenta_sellar(dir.path(), &actual, &declaradas);
    let rutas: Vec<&str> = violaciones.iter().map(|h| h.ruta.as_str()).collect();
    assert_eq!(
        rutas,
        vec!["a.md", "b.md"],
        "las dos infractoras deben salir, no solo la primera: {violaciones:?}"
    );
}

#[test]
fn sellar_escribe_cuando_todo_tiene_aire() {
    let dir = tempfile::tempdir().unwrap();
    // Un sello huérfano (sin Declarada) que debe sobrevivir intacto, y una
    // nota declarada que baja su techo con aire de sobra.
    let actual = sellos(&[("huerfano.md", 5000), ("a.md", 20000)]);
    trinquete::escribe_sellos(dir.path(), &actual).unwrap();

    let declaradas = [declarada("a.md", "core", 15000, NOMINALES.core, 13000)];

    let violaciones = intenta_sellar(dir.path(), &actual, &declaradas);
    assert!(
        violaciones.is_empty(),
        "todo tiene aire, no debe haber violaciones: {violaciones:?}"
    );

    let de_disco = trinquete::carga(dir.path()).unwrap();
    assert_eq!(de_disco.get("a.md"), Some(&15000), "a.md debe haber bajado");
    assert_eq!(
        de_disco.get("huerfano.md"),
        Some(&5000),
        "el sello huérfano, sin tocar, debe sobrevivir el --seal"
    );
    assert_eq!(
        de_disco.len(),
        2,
        "el fichero resultante debe parsear con las dos entradas"
    );
}

#[test]
fn las_violaciones_de_aire_solo_juzgan_techos_que_cambian() {
    // Un sello intacto sin aire: re-declarar el MISMO techo no es una
    // transición, así que `violaciones_de_aire` no debe verlo, aunque le
    // falte aire de sobra (misma regla que la deuda de la guarda de aire de
    // `comprueba`, Task 8, del otro lado).
    let actual = sellos(&[("a.md", 5000)]);
    let declaradas = [declarada("a.md", "core", 5000, NOMINALES.core, 10000)];
    let siguiente = trinquete::sella(&actual, &declaradas);
    assert_eq!(
        siguiente.get("a.md"),
        Some(&5000),
        "no cambia: 5000 no es menor que 5000"
    );

    let violaciones = trinquete::violaciones_de_aire(&actual, &siguiente, &declaradas);
    assert!(
        violaciones.is_empty(),
        "un techo que no cambia no es una transición, así que no se juzga: {violaciones:?}"
    );
}

#[test]
fn violaciones_de_aire_marca_una_declaracion_nueva_en_zona_muerta() {
    // El mismo fixture de zona muerta que `trinquete_aire.rs`, del lado de
    // `--seal`: una primera declaración de 44.000 B en core no puede tener
    // a la vez aire y respetar el cap de 2×. El remedio es partir la nota.
    let actual = Sellos::new();
    let declaradas = [declarada(
        "core/attack.md",
        "core",
        45000,
        NOMINALES.core,
        44000,
    )];
    let siguiente = trinquete::sella(&actual, &declaradas);
    let violaciones = trinquete::violaciones_de_aire(&actual, &siguiente, &declaradas);
    assert_eq!(violaciones.len(), 1);
    assert_eq!(violaciones[0].ruta, "core/attack.md");
    assert_eq!(violaciones[0].tipo, Tipo::NaceDemasiadoGrande);
    assert_eq!(violaciones[0].ahora, 44000);
    assert_eq!(violaciones[0].limite, NOMINALES.core * 2);
}
