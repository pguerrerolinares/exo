//! El trinquete de techos declarados: sellos, su formato en disco y su IO.
//!
//! Puerto de `internal/ratchet` de kbx (`fe46443`). Esta primera capa solo
//! trae el tipo de dato (`Sellos`) y su parseo/escritura; la abstención, el
//! ancla, el emparejamiento de renames y la guarda de aire llegan en tareas
//! posteriores del mismo módulo.

use crate::gitx;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

/// Carga el sello desde `HEAD`, no del disco. Puerto de `LoadHEAD`
/// (`internal/ratchet/load.go` de kbx, `fe46443`); esta es la primera prosa
/// escrita en el repo para estos dos mecanismos (Task 4, A2) — hasta ahora
/// solo vivían en ese código Go.
///
/// `Ok(None)` es **abstención**: no hay historia utilizable de la que
/// fiarse, y el trinquete prefiere no opinar a opinar sobre datos que no
/// tiene. Dos causas, las dos abstención:
/// - no hay repo, o HEAD no resuelve (KB sin commits todavía);
/// - **shallow-clone**: en un clon truncado (`--depth`) el objeto de HEAD
///   puede faltar, y leer un sello vacío de un HEAD incompleto haría pasar
///   en verde cualquier subida de techo.
///
/// `Ok(Some(sellos))` significa que SÍ hay historia utilizable, y el mapa
/// puede venir vacío porque `.kbx-ratchet.json` **todavía no está
/// commiteado en HEAD** — y eso NO es abstención. Es el **ancla de
/// activación**: la corrida en la que el fichero no existe en HEAD es la
/// corrida que instala el trinquete, y en ella los techos se sellan sin
/// juzgarlos. Sin esta exención, el día de la instalación toda la KB saldría
/// en rojo por sellos que nadie ha tenido ocasión de podar. Una vez el
/// fichero está en HEAD, el ancla existe y la exención se acaba para
/// siempre. Confundir este caso con la abstención de arriba es el bug: el
/// trinquete quedaría desactivado en toda KB nueva sin que nadie se entere.
///
/// Un sello **corrupto** en HEAD (JSON que no parsea) es `Err`, nunca
/// ninguna de las dos ramas anteriores: sub-invariante 3 del ítem 7 de la
/// spec.
pub fn carga_head(kb: &Path) -> Result<Option<Sellos>> {
    if !gitx::es_work_tree(kb)? {
        return Ok(None);
    }
    if gitx::es_shallow(kb)? {
        return Ok(None);
    }
    // El `./` explícito es obligatorio (A8): sin él, git resuelve la ruta
    // contra la raíz del repo y no contra `kb` (el `-C`), y si la KB vive en
    // un subdirectorio los sellos de HEAD se leerían vacíos en silencio.
    let objeto = format!("HEAD:./{FICHERO_SELLO}");
    match gitx::muestra(kb, &objeto)? {
        Some(datos) => parsea_sellos(&datos).map(Some),
        // HEAD resuelve pero el objeto no está: sello aún no commiteado,
        // ancla de activación, no abstención.
        None if gitx::head_resuelve(kb) => Ok(Some(Sellos::new())),
        // HEAD no resuelve: repo sin historia (o sin repo en absoluto, ya
        // cubierto arriba). Abstención.
        None => Ok(None),
    }
}

/// ¿Existe ya `.kbx-ratchet.json` en `HEAD`? El ancla de activación (ver
/// `carga_head`): mientras el fichero no esté en HEAD, la corrida en curso
/// es la que instala el trinquete.
pub fn anclado_en_head(kb: &Path) -> bool {
    let objeto = format!("HEAD:./{FICHERO_SELLO}");
    matches!(gitx::muestra(kb, &objeto), Ok(Some(_)))
}

/// Los nueve tipos de hallazgo que el trinquete puede producir. Las claves
/// JSON (inglés, D7/D8) son las de kbx: cambiarlas rompería a quien ya
/// parsea el envelope de `exo lint`.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tipo {
    #[serde(rename = "seal-raised")]
    SelloSubido,
    #[serde(rename = "seal-removed")]
    SelloRetirado,
    #[serde(rename = "over-seal")]
    SobreSello,
    #[serde(rename = "first-declaration-too-high")]
    PrimeraMuyAlta,
    #[serde(rename = "inert-log-waiver")]
    WaiverLogInerte,
    #[serde(rename = "sealed-escaped-tier")]
    SelladaEscapadaDeTier,
    #[serde(rename = "no-air")]
    SinAire,
    #[serde(rename = "born-too-big")]
    NaceDemasiadoGrande,
    #[serde(rename = "no-air-debt")]
    DeudaSinAire,
}

impl Tipo {
    /// Los siete que rompen el gate (exit 3). `WaiverLogInerte` y
    /// `DeudaSinAire` son información: existen para verse, no para
    /// bloquear. Exhaustivo y sin `_ =>` a propósito (ver brief de Task 5):
    /// una variante nueva obliga a decidir aquí, no a colarse como
    /// rompiente (o no) por accidente.
    pub fn rompe(self) -> bool {
        match self {
            Tipo::SelloSubido => true,
            Tipo::SelloRetirado => true,
            Tipo::SobreSello => true,
            Tipo::PrimeraMuyAlta => true,
            Tipo::WaiverLogInerte => false,
            Tipo::SelladaEscapadaDeTier => true,
            Tipo::SinAire => true,
            Tipo::NaceDemasiadoGrande => true,
            Tipo::DeudaSinAire => false,
        }
    }
}

/// `true` si el valor es el `i64` por defecto: usado solo como predicado de
/// `skip_serializing_if` en `Hallazgo`, para que un campo no aplicable
/// (p.ej. `limite` en una subida de techo, que no tiene tope de tier) no
/// aparezca en el JSON en vez de aparecer como `0` engañoso.
fn es_cero(v: &i64) -> bool {
    *v == 0
}

/// Un hallazgo del trinquete: una ruta, su tipo, y hasta tres cifras de
/// contexto que solo tienen sentido según el tipo (de ahí que las tres sean
/// opcionales en el JSON). Claves JSON `path`/`kind`/`was`/`now`/`limit`
/// (inglés, D7/D8).
#[derive(Serialize, Clone, Debug)]
pub struct Hallazgo {
    #[serde(rename = "path")]
    pub ruta: String,
    #[serde(rename = "kind")]
    pub tipo: Tipo,
    #[serde(rename = "was", skip_serializing_if = "es_cero")]
    pub era: i64,
    #[serde(rename = "now", skip_serializing_if = "es_cero")]
    pub ahora: i64,
    #[serde(rename = "limit", skip_serializing_if = "es_cero")]
    pub limite: i64,
}

/// Compara los sellos de `HEAD` contra los del árbol actual y devuelve todo
/// lo que cambió para peor: techos que suben (`SelloSubido`) y sellos que
/// desaparecen (`SelloRetirado`, sub-invariante 1 del ítem 7 de la spec —
/// borrar un sello es indistinguible de subirlo a infinito, así que sale
/// con el mismo tipo que una subida). Un sello nuevo, o uno que baja, no es
/// violación: el trinquete solo mira hacia arriba.
pub fn violaciones(head: &Sellos, actual: &Sellos) -> Vec<Hallazgo> {
    let mut hallazgos = Vec::new();
    for (ruta, &ahora) in actual {
        if let Some(&era) = head.get(ruta)
            && ahora > era
        {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelloSubido,
                era,
                ahora,
                limite: 0,
            });
        }
    }
    for (ruta, &era) in head {
        if !actual.contains_key(ruta) {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelloRetirado,
                era,
                ahora: 0,
                limite: 0,
            });
        }
    }
    // El `BTreeMap` de origen ya itera en orden alfabético (A4), pero aquí
    // se combinan dos pasadas sobre dos mapas distintos (subidas desde
    // `actual`, bajas desde `head`), así que ese orden no sobrevive gratis
    // al resultado combinado. El contrato es el orden final, no cómo se
    // llega a él.
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    hallazgos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sellos(pares: &[(&str, i64)]) -> Sellos {
        pares.iter().map(|(r, t)| (r.to_string(), *t)).collect()
    }

    #[test]
    fn un_techo_que_sube_es_violacion() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 200)]);
        let hallazgos = violaciones(&head, &actual);
        assert_eq!(hallazgos.len(), 1);
        assert_eq!(hallazgos[0].ruta, "a.md");
        assert_eq!(hallazgos[0].tipo, Tipo::SelloSubido);
        assert_eq!(hallazgos[0].era, 100);
        assert_eq!(hallazgos[0].ahora, 200);
    }

    #[test]
    fn un_techo_que_baja_no_lo_es() {
        let head = sellos(&[("a.md", 200)]);
        let actual = sellos(&[("a.md", 100)]);
        assert!(violaciones(&head, &actual).is_empty());
    }

    #[test]
    fn un_sello_nuevo_no_es_violacion() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        assert!(violaciones(&head, &actual).is_empty());
    }

    #[test]
    fn borrar_un_sello_equivale_a_subirlo_a_infinito() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[]);
        let hallazgos = violaciones(&head, &actual);
        assert_eq!(hallazgos.len(), 1);
        assert_eq!(hallazgos[0].ruta, "a.md");
        assert_eq!(hallazgos[0].tipo, Tipo::SelloRetirado);
        assert_eq!(hallazgos[0].era, 100);
        assert_eq!(hallazgos[0].ahora, 0);
    }

    #[test]
    fn las_violaciones_salen_ordenadas_por_ruta() {
        // `a.md` se borra (SelloRetirado) y `z.md` sube (SelloSubido): dos
        // tipos distintos, producidos por dos pasadas distintas sobre el
        // mapa. El orden alfabético del resultado es el contrato, no un
        // efecto secundario de qué pasada corre primero.
        let head = sellos(&[("a.md", 100), ("z.md", 50)]);
        let actual = sellos(&[("z.md", 999)]);
        let hallazgos = violaciones(&head, &actual);
        let rutas: Vec<&str> = hallazgos.iter().map(|h| h.ruta.as_str()).collect();
        assert_eq!(rutas, vec!["a.md", "z.md"]);
    }
}
