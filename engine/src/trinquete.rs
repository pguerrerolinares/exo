//! El trinquete de techos declarados: sellos, su formato en disco y su IO.
//!
//! Puerto de `internal/ratchet` de kbx (`fe46443`). Esta primera capa solo
//! trae el tipo de dato (`Sellos`) y su parseo/escritura; la abstención, el
//! ancla, el emparejamiento de renames y la guarda de aire llegan en tareas
//! posteriores del mismo módulo.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// El nombre del fichero de sello **no cambia** al portar a Rust: es el
/// fichero que la KB tiene hoy commiteado con 11 entradas, y renombrarlo a
/// algo `.exo-*` perdería el ancla (ver `anclado_en_head`, Task 4) y con ella
/// todo el histórico del trinquete — precisamente lo que el trinquete existe
/// para conservar (A5).
pub const FICHERO_SELLO: &str = ".kbx-ratchet.json";

/// Ruta de nota (con `/` literal, nunca separador nativo) → techo en bytes.
///
/// Las claves son `String`, no `PathBuf` (A9): son cadenas de identidad que
/// viajan a JSON y se comparan con lo que git devuelve, que siempre usa `/`.
/// Solo se convierten a ruta del sistema en el punto de tocar disco (`carga`,
/// `escribe_sellos`). Y es `BTreeMap`, no `HashMap`: el orden alfabético es
/// una propiedad del tipo, no algo que haya que verificar aparte (A4).
pub type Sellos = std::collections::BTreeMap<String, i64>;

/// El esquema en disco: un objeto con una sola clave, `ceilings`. Si la clave
/// falta (p.ej. `{}`), `serde(default)` la trata como mapa vacío — igual que
/// el Go trata `doc.Ceilings == nil` como `Seals{}` — así que un sello sin
/// declaraciones no es un error, es la KB recién instalada.
#[derive(Deserialize)]
struct DocSello {
    #[serde(default)]
    ceilings: BTreeMap<String, i64>,
}

/// Parsea el contenido de un `.kbx-ratchet.json`. Un JSON sintácticamente
/// corrupto es `Err`, nunca abstención: es el sub-invariante 3 del ítem 7 de
/// la spec ("sello corrupto en HEAD es error, no abstención"), y esta función
/// es el único punto por el que pasa, tanto si el sello viene del disco
/// (`carga`) como de `git show` (`carga_head`, Task 4).
pub fn parsea_sellos(datos: &str) -> Result<Sellos> {
    let doc: DocSello = serde_json::from_str(datos).context("parsear sello de trinquete")?;
    Ok(doc.ceilings)
}

/// Carga el sello desde el disco de la KB. Un fichero **ausente** es
/// `Ok(vacío)`, no error: es lo que permite que una KB nueva —sin trinquete
/// instalado aún— no arranque en rojo.
pub fn carga(kb: &Path) -> Result<Sellos> {
    let ruta = kb.join(FICHERO_SELLO);
    match std::fs::read_to_string(&ruta) {
        Ok(datos) => parsea_sellos(&datos),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Sellos::new()),
        Err(e) => Err(e).with_context(|| format!("leer sello de {}", ruta.display())),
    }
}

/// Serializa a mano el formato exacto de kbx (`writeSeals`,
/// `cmd/kbx/ratchet.go`), byte a byte (A10): claves ordenadas alfabéticamente
/// (gratis, `BTreeMap` ya itera así), indentación de 2 y 4 espacios, valores
/// enteros sin comillas, sin coma tras la última entrada, `\n` final.
///
/// No se usa `serde_json::to_string_pretty` del mapa entero porque su
/// indentación no coincide con la de kbx: el sello está **commiteado en la
/// KB**, y un `--seal` que reescriba con otra indentación produciría un diff
/// de 13 líneas donde debería haber una. Sí se usa `serde_json::to_string`
/// por clave individual, para heredar su escapado correcto sin reimplementarlo.
pub fn serializa_sellos(sellos: &Sellos) -> String {
    let mut salida = String::from("{\n  \"ceilings\": {\n");
    let total = sellos.len();
    for (i, (ruta, techo)) in sellos.iter().enumerate() {
        // `to_string` de un `&String` siempre produce JSON válido (nunca
        // falla): es el único uso de `serde_json` aquí, y solo por el
        // escapado de la clave.
        let clave = serde_json::to_string(ruta).expect("una String siempre serializa a JSON");
        let coma = if i + 1 == total { "" } else { "," };
        salida.push_str(&format!("    {clave}: {techo}{coma}\n"));
    }
    salida.push_str("  }\n}\n");
    salida
}

/// Escribe el sello en el disco de la KB con el formato de `serializa_sellos`.
pub fn escribe_sellos(kb: &Path, sellos: &Sellos) -> Result<()> {
    let ruta = kb.join(FICHERO_SELLO);
    std::fs::write(&ruta, serializa_sellos(sellos))
        .with_context(|| format!("escribir sello en {}", ruta.display()))
}
