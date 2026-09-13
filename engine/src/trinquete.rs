//! El trinquete de techos declarados: sellos, su formato en disco y su IO.
//!
//! Puerto de `internal/ratchet` de kbx (`fe46443`). Esta primera capa solo
//! trae el tipo de dato (`Sellos`) y su parseo/escritura; la abstención, el
//! ancla, el emparejamiento de renames y la guarda de aire llegan en tareas
//! posteriores del mismo módulo.

use crate::gitx;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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

/// Carga el sello desde el índice de git (`:./<fichero>`), no de HEAD ni del
/// disco. Puerto de `LoadStaged` (`internal/ratchet/staged.go` de kbx,
/// `fe46443`): es lo que hace `--staged` juzgar lo que el commit se va a
/// llevar y no lo que hay en el árbol de trabajo — stagear una subida de
/// techo y luego restaurar el disco no basta para escapar del gate.
///
/// Cualquier fallo al leer el índice (fichero no staged, no hay repo, sello
/// nunca stageado) se traga en silencio y devuelve mapa vacío: así lo hace
/// el Go, que descarta el error de `gitx.Run` entero (`err != nil => return
/// Seals{}, nil`). Un JSON staged que SÍ existe pero está corrupto sigue
/// siendo `Err`: la única puerta de parseo es `parsea_sellos`, y el
/// sub-invariante 3 del ítem 7 de la spec no distingue de dónde vino el
/// sello.
pub fn carga_staged(kb: &Path) -> Result<Sellos> {
    let objeto = format!(":./{FICHERO_SELLO}");
    match gitx::muestra(kb, &objeto) {
        Ok(Some(datos)) => parsea_sellos(&datos),
        Ok(None) => Ok(Sellos::new()),
        Err(_) => Ok(Sellos::new()),
    }
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

/// Una nota que declara techo (`kbx_budget_max`), tal cual está en el árbol
/// actual — no un hallazgo, no una comparación con `HEAD`. Es la materia
/// prima de la que salen las nueve variantes de `Tipo` en tareas
/// posteriores del mismo módulo.
#[derive(Clone, Debug)]
pub struct Declarada {
    pub ruta: String,
    pub tier: String,
    pub max: i64,
    pub tier_presupuesto: i64,
    pub tamano: i64,
}

/// Recorre la KB y trae toda nota que declare `kbx_budget_max`, sin filtrar
/// por si infringe nada: a diferencia de `presupuesto::analiza`, el trinquete
/// juzga declaraciones, no tamaños, así que una nota muy por debajo de su
/// tier igual aparece aquí (y `budget` nunca la reportaría).
///
/// Las notas sin techo declarado se saltan enteras — no se mira si tienen
/// aire o no, eso es cosa de `budget`. `tier_presupuesto` es
/// `presupuestos.para_tier(tier).unwrap_or(0)`: un tier ilegal se trata como
/// 0 (sin techo), igual que `log`; la distinción `None`/`Some(0)` que
/// `budget` necesita no aporta aquí porque lo único que el trinquete hace
/// con un tier sin presupuesto es marcar el waiver como inerte, y eso vale
/// para los dos casos.
pub fn recolecta(
    kb: &Path,
    presupuestos: crate::presupuesto::Presupuestos,
    excluidos: &[&str],
) -> Result<Vec<Declarada>> {
    let rutas = crate::walker::walk_notas(kb, excluidos)?;

    let mut declaradas = Vec::new();
    for rel in rutas {
        let absoluta = kb.join(&rel);
        let contenido = crate::walker::lee_nota(&absoluta)?;
        let Some(max) = crate::frontmatter::budget_max(&contenido) else {
            continue;
        };
        let tamano = std::fs::metadata(&absoluta)
            .with_context(|| format!("stat de {}", absoluta.display()))?
            .len() as i64;
        let tier = crate::frontmatter::tier(&contenido);
        let tier_presupuesto = presupuestos.para_tier(&tier).unwrap_or(0);

        declaradas.push(Declarada {
            ruta: rel,
            tier,
            max,
            tier_presupuesto,
            tamano,
        });
    }

    declaradas.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(declaradas)
}

/// `recolecta`, pero sobre el índice de git en vez del árbol de trabajo: las
/// rutas salen de `gitx::md_staged` (el diff cacheado, A3) y el contenido de
/// cada una sale de `git show :./<ruta>`, no de disco. Puerto de
/// `CollectStaged` (`internal/ratchet/staged.go`, kbx `fe46443`).
///
/// El tamaño se mide en bytes crudos del índice (`gitx::muestra_bytes`), no
/// con la `String` lossy de `gitx::muestra`: es el motivo que documenta
/// `gitx::muestra_bytes`, y aquí importa igual que allí — una `Declarada`
/// con un tamaño equivocado hace que la guarda de aire compare contra un
/// número que no es el que el commit se va a llevar.
///
/// Un fichero staged como borrado (`git show :./x` falla) se salta, no
/// rompe: la misma exención que documenta `gitx::muestra_bytes` para
/// `sizeFromIndex`.
pub fn recolecta_staged(
    kb: &Path,
    presupuestos: crate::presupuesto::Presupuestos,
    excluidos: &[&str],
) -> Result<Vec<Declarada>> {
    let rutas = gitx::md_staged(kb)?;

    let mut declaradas = Vec::new();
    for rel in rutas {
        if crate::walker::excluida(&rel, excluidos) {
            continue;
        }
        let Some(bytes) = gitx::muestra_bytes(kb, &format!(":./{rel}"))? else {
            continue; // staged como borrado: el objeto no está en el índice.
        };
        let contenido = String::from_utf8_lossy(&bytes);
        let Some(max) = crate::frontmatter::budget_max(&contenido) else {
            continue;
        };
        let tier = crate::frontmatter::tier(&contenido);
        let tier_presupuesto = presupuestos.para_tier(&tier).unwrap_or(0);

        declaradas.push(Declarada {
            ruta: rel,
            tier,
            max,
            tier_presupuesto,
            tamano: bytes.len() as i64,
        });
    }

    declaradas.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(declaradas)
}

/// ¿Existió de verdad `ruta` en `HEAD`? Sin esta comprobación, un sello
/// huérfano podría hacer de rename — ver `empareja_renames`, que es quien la
/// usa. El Go (`existedInHEAD`, `internal/ratchet/check.go`) resuelve con
/// `git rev-parse HEAD:./<ruta>`, que trae la entrada del árbol sin el blob;
/// aquí se reutiliza `gitx::muestra` (que sí trae el blob) en vez de añadir un
/// verbo nuevo a `gitx` para una sola llamada — más caro, semánticamente
/// equivalente para este uso: solo importa si el objeto existe, no su
/// contenido.
fn existio_en_head(kb: &Path, ruta: &str) -> bool {
    gitx::muestra(kb, &format!("HEAD:./{ruta}"))
        .ok()
        .flatten()
        .is_some()
}

/// El emparejamiento de renames: absuelve un sello retirado y uno fresco que
/// nacen en el mismo commit por culpa de `git mv`, y solo por eso. Puerto del
/// bloque de renames de `checkAgainst` (`internal/ratchet/check.go`, kbx
/// `fe46443`).
///
/// **Absolución de renames** (prosa escrita aquí por primera vez, A2): `git
/// mv nota-vieja.md nota-nueva.md` retira un sello y declara otro en el mismo
/// commit. Sin tratamiento, eso es a la vez un `SelloRetirado` (rojo: borrar
/// un sello sube el techo a infinito) y una primera declaración sujeta al cap
/// de 2× tier (rojo otra vez, Task 8) — un rename legítimo saldría en rojo
/// doble. El trinquete empareja cada sello retirado con un sello fresco de
/// techo **menor o igual**, y absuelve a los dos. Es heurístico por
/// construcción: sin mirar contenido, un rename y un borrar+crear son
/// indistinguibles.
///
/// **El sello huérfano que no puede lavar** (`0ae126d`): un sello "retirado"
/// solo cuenta como candidato a rename si la nota **existió de verdad en
/// HEAD** (`existio_en_head`). Sin esa comprobación, un sello huérfano —de
/// una nota que nunca se commiteó, dejado atrás por un borrado antiguo—
/// serviría de coartada: retirarlo en el mismo commit que declara cualquier
/// techo nuevo absolvería esa declaración de la guarda de aire (Task 8) y del
/// cap de 2×, sin un solo hallazgo. No hace falta mala intención: limpiar un
/// sello obsoleto a la vez que se añade una nota lo dispara por accidente.
///
/// **Tie-break**, portado literal de `checkAgainst`: los retirados y los
/// frescos se ordenan por `(techo, ruta)` ascendente; para cada retirado, en
/// ese orden, se elige entre los frescos aún libres con techo `<=` el suyo
/// **el mayor que quepa**. No es una cuestión de empaquetado sino de
/// identidad: un rename **conserva** su techo, así que el candidato correcto
/// es el más cercano (por abajo) al retirado, y "el mayor que quepa" siempre
/// elige la coincidencia exacta cuando existe. Emparejar por el menor
/// absolvería una declaración nueva del mismo día y dejaría el rename
/// legítimo contra el cap.
///
/// Devuelve `(sellos_frescos_absueltos, sellos_retirados_emparejados)`.
fn empareja_renames(
    kb: &Path,
    head: &Sellos,
    actual: &Sellos,
) -> (BTreeSet<String>, BTreeSet<String>) {
    struct Ref {
        ruta: String,
        techo: i64,
    }
    let ordena = |v: &mut Vec<Ref>| {
        v.sort_by(|a, b| a.techo.cmp(&b.techo).then_with(|| a.ruta.cmp(&b.ruta)))
    };

    let mut retirados: Vec<Ref> = head
        .iter()
        .filter(|(ruta, _)| !actual.contains_key(*ruta) && existio_en_head(kb, ruta))
        .map(|(ruta, &techo)| Ref {
            ruta: ruta.clone(),
            techo,
        })
        .collect();
    let mut frescos: Vec<Ref> = actual
        .iter()
        .filter(|(ruta, _)| !head.contains_key(*ruta))
        .map(|(ruta, &techo)| Ref {
            ruta: ruta.clone(),
            techo,
        })
        .collect();
    ordena(&mut retirados);
    ordena(&mut frescos);

    let mut frescos_absueltos = BTreeSet::new();
    let mut retirados_emparejados = BTreeSet::new();
    for rem in &retirados {
        // `frescos` está ordenado ascendente: no se corta el bucle al
        // encontrar el primero que encaja porque el último que encaje es el
        // mayor — "el mayor que quepa".
        let mut mejor: Option<usize> = None;
        for (i, fr) in frescos.iter().enumerate() {
            if frescos_absueltos.contains(&fr.ruta) || fr.techo > rem.techo {
                continue;
            }
            mejor = Some(i);
        }
        if let Some(i) = mejor {
            frescos_absueltos.insert(frescos[i].ruta.clone());
            retirados_emparejados.insert(rem.ruta.clone());
        }
    }
    (frescos_absueltos, retirados_emparejados)
}

/// Una primera declaración no puede sellar más de 2× el nominal de su tier.
/// Con el 15% de aire (`presupuesto::tiene_aire`), el techo legal de una nota
/// nueva sale en `tier*2/1,15 = 1,739×` su tier: 14.782 B en core, 21.739 B
/// en stable. Por encima no hay techo legal que valga, y el remedio no es un
/// techo más alto: es partir la nota (`Tipo::NaceDemasiadoGrande`).
const FACTOR_PRIMERA_DECLARACION: i64 = 2;

/// Tamaño de una nota en el árbol de trabajo, en bytes. `None` si no se
/// puede leer (nota borrada, permisos): la guarda de aire se salta el sello
/// en vez de inventar un tamaño (`sizeFromDisk`, `internal/ratchet/check.go`).
fn tamano_de_disco(kb: &Path, ruta: &str) -> Option<i64> {
    std::fs::metadata(kb.join(ruta))
        .ok()
        .map(|meta| meta.len() as i64)
}

/// El informe del trinquete: `aplicado: false` significa que se abstuvo y
/// sus hallazgos no tienen autoridad ninguna — ni siquiera si la lista viene
/// vacía. Claves JSON `applied`/`reason`/`findings` (D7/D8, las de kbx).
#[derive(Serialize, Debug)]
pub struct Informe {
    #[serde(rename = "applied")]
    pub aplicado: bool,
    #[serde(rename = "reason", skip_serializing_if = "Option::is_none")]
    pub razon: Option<String>,
    #[serde(rename = "findings")]
    pub hallazgos: Vec<Hallazgo>,
}

impl Informe {
    /// `true` si el gate debe salir con exit ≠ 0. Un informe abstenido nunca
    /// falla (la abstención es información, no fallo), y los hallazgos
    /// informativos (`WaiverLogInerte`, `DeudaSinAire`) tampoco cuentan:
    /// `Tipo::rompe` es la única fuente de verdad sobre qué rompe.
    pub fn fallido(&self) -> bool {
        self.aplicado && self.hallazgos.iter().any(|h| h.tipo.rompe())
    }
}

/// Corre el trinquete sobre el árbol de trabajo: los sellos actuales salen
/// de `.kbx-ratchet.json` en disco, y el tamaño de cada nota sale de
/// `metadata().len()`. Puerto de `Check` (`internal/ratchet/check.go`).
pub fn comprueba(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga, tamano_de_disco)
}

/// Tamaño de una nota en el índice de git, en bytes crudos. `None` si el
/// objeto no está en el índice (fichero no staged, staged como borrado): la
/// guarda de aire se salta el sello en vez de inventar un tamaño
/// (`sizeFromIndex`, `internal/ratchet/staged.go`).
fn tamano_de_indice(kb: &Path, ruta: &str) -> Option<i64> {
    gitx::muestra_bytes(kb, &format!(":./{ruta}"))
        .ok()
        .flatten()
        .map(|b| b.len() as i64)
}

/// Corre el trinquete sobre el índice de git: los sellos actuales salen de
/// `carga_staged` y el tamaño de cada nota de `tamano_de_indice` — las dos
/// leen la misma revisión (el índice), que es la condición que
/// `comprueba_contra` exige (mezclar un techo del índice con un tamaño del
/// disco compararía dos mundos distintos). Puerto de `CheckStaged`
/// (`internal/ratchet/staged.go`).
pub fn comprueba_staged(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Result<Informe> {
    comprueba_contra(kb, declaradas, presupuestos, carga_staged, tamano_de_indice)
}

/// El núcleo del trinquete, parametrizado por de dónde salen los sellos
/// actuales y el tamaño de una nota — exactamente como `checkAgainst` en kbx.
/// `comprueba` pasa el árbol de trabajo (`carga`, `tamano_de_disco`); la
/// variante `--staged` (Task 10) pasará el índice de git con las mismas dos
/// funciones parametrizadas, y por eso la firma ya las lleva aunque hoy solo
/// exista un llamador.
///
/// Los sellos actuales y los tamaños **tienen que venir de la misma
/// revisión** — mezclar un techo del índice con un tamaño del disco compara
/// dos mundos distintos y la guarda de aire deja de significar nada.
fn comprueba_contra(
    kb: &Path,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
    carga_actual: impl Fn(&Path) -> Result<Sellos>,
    tamano_de: impl Fn(&Path, &str) -> Option<i64>,
) -> Result<Informe> {
    let Some(head) = carga_head(kb)? else {
        return Ok(Informe {
            aplicado: false,
            razon: Some(
                "no git history available (missing repo, unreadable history, or shallow clone)"
                    .to_string(),
            ),
            hallazgos: Vec::new(),
        });
    };
    let actual = carga_actual(kb)?;

    // Renames (Task 7): cada retirado emparejado se resta de `violaciones`
    // (el techo viajó, no desapareció) y cada fresco emparejado queda exento
    // del cap de primera declaración más abajo — carga su techo, no lo
    // declara de cero.
    let (frescos_absueltos, retirados_emparejados) = empareja_renames(kb, &head, &actual);

    let mut hallazgos: Vec<Hallazgo> = violaciones(&head, &actual)
        .into_iter()
        .filter(|v| !(v.tipo == Tipo::SelloRetirado && retirados_emparejados.contains(&v.ruta)))
        .collect();

    // El ancla de activación (Task 4): sin sello commiteado en HEAD, esta
    // corrida es la que instala el trinquete y consagra lo que ya existía.
    let activacion = !anclado_en_head(kb);

    // Las tres familias, en el mismo orden en que se empujaban antes de
    // partir la función: `sort_by` es estable, así que dos hallazgos de la
    // misma ruta conservan ese orden relativo en el informe.
    let (aire, nacio_demasiado_grande) = guarda_de_aire(
        &head,
        &actual,
        declaradas,
        activacion,
        &frescos_absueltos,
        |ruta| tamano_de(kb, ruta),
    );
    hallazgos.extend(aire);
    hallazgos.extend(checks_de_declaracion(
        &head,
        &actual,
        declaradas,
        activacion,
        &frescos_absueltos,
        &nacio_demasiado_grande,
    ));
    hallazgos.extend(sellos_escapados_de_tier(
        kb,
        &actual,
        declaradas,
        presupuestos,
    ));

    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    Ok(Informe {
        aplicado: true,
        razon: None,
        hallazgos,
    })
}

/// La guarda de aire (Task 8): juzga TRANSICIONES, no estado. Un sello que
/// nadie toca no se juzga aunque no tenga aire — se reporta como deuda, que
/// no rompe. Sin eso, los 11 sellos reales de la KB (ninguno con 15% de
/// aire, medido) dejarían el repo en rojo permanente el día de la
/// instalación.
///
/// Devuelve los hallazgos y el conjunto de rutas marcadas
/// `NaceDemasiadoGrande`, que `checks_de_declaracion` necesita para no
/// apilarles más hallazgos.
fn guarda_de_aire(
    head: &Sellos,
    actual: &Sellos,
    declaradas: &[Declarada],
    activacion: bool,
    frescos_absueltos: &BTreeSet<String>,
    tamano_de: impl Fn(&str) -> Option<i64>,
) -> (Vec<Hallazgo>, BTreeSet<String>) {
    let declaradas_por_ruta: BTreeMap<&str, &Declarada> =
        declaradas.iter().map(|d| (d.ruta.as_str(), d)).collect();

    let mut hallazgos = Vec::new();
    let mut nacio_demasiado_grande: BTreeSet<String> = BTreeSet::new();
    for (ruta, &techo) in actual {
        let declarada = declaradas_por_ruta.get(ruta.as_str()).copied();
        let tamano = match declarada {
            Some(d) => d.tamano,
            // Sin declaración: el tamaño sale de la misma revisión que los
            // sellos. Si no se puede leer (nota borrada, sello huérfano sin
            // fichero), se salta — no se inventa un tamaño.
            None => match tamano_de(ruta) {
                Some(t) => t,
                None => continue,
            },
        };
        if crate::presupuesto::tiene_aire(techo, tamano) {
            continue; // rama 1: tiene aire, nada que reportar.
        }
        let era = head.get(ruta).copied();
        if let Some(era) = era
            && techo > era
        {
            // rama 2: ya rompió como SelloSubido; etiquetarlo además como
            // deuda mal-clasificaría una decisión de hoy como preexistente.
            continue;
        }
        let cambio = match era {
            None => true,
            Some(era) => techo < era,
        };
        if !cambio {
            // rama 3: intacto desde HEAD. Deuda, no fallo.
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::DeudaSinAire,
                era: 0,
                ahora: techo,
                limite: crate::presupuesto::techo_minimo(tamano),
            });
            continue;
        }
        // rama 4: cambió (fresco o bajado). La activación consagra lo que
        // ya existía; un rename carga su techo — ninguno es una decisión
        // nueva sobre el margen.
        if activacion || frescos_absueltos.contains(ruta) {
            continue;
        }
        let fresco = era.is_none();
        if fresco
            && let Some(d) = declarada
            && d.tier_presupuesto > 0
            && crate::presupuesto::techo_minimo(tamano)
                > d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION
        {
            // Zona muerta: ninguna nota de este tamaño puede tener a la vez
            // aire y respetar el cap de 2×. El remedio es partir la nota, no
            // un techo más alto.
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::NaceDemasiadoGrande,
                era: 0,
                ahora: tamano,
                limite: d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION,
            });
            nacio_demasiado_grande.insert(ruta.clone());
            continue;
        }
        hallazgos.push(Hallazgo {
            ruta: ruta.clone(),
            tipo: Tipo::SinAire,
            era: 0,
            ahora: techo,
            limite: crate::presupuesto::techo_minimo(tamano),
        });
    }
    (hallazgos, nacio_demasiado_grande)
}

/// Las tres familias sobre declaraciones: cap de 2× en la primera
/// declaración (sellada o no), waiver por encima del sello y waiver inerte
/// en un tier sin presupuesto. Una nota ya marcada `NaceDemasiadoGrande` no
/// recibe además estos checks: ya tiene el único hallazgo que aconseja bien,
/// y el Go la salta con el mismo set.
fn checks_de_declaracion(
    head: &Sellos,
    actual: &Sellos,
    declaradas: &[Declarada],
    activacion: bool,
    frescos_absueltos: &BTreeSet<String>,
    nacio_demasiado_grande: &BTreeSet<String>,
) -> Vec<Hallazgo> {
    let mut hallazgos = Vec::new();
    for d in declaradas {
        if nacio_demasiado_grande.contains(&d.ruta) {
            continue;
        }
        match actual.get(&d.ruta) {
            Some(&sello) => {
                // El cap de 2× sobre una nota ya sellada: no aplica en la
                // corrida de activación, si ya estaba en HEAD, o si es un
                // fresco absuelto por rename (carga su techo, no lo declara).
                if !activacion
                    && !head.contains_key(&d.ruta)
                    && !frescos_absueltos.contains(&d.ruta)
                    && d.tier_presupuesto > 0
                {
                    let limite = d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION;
                    if sello > limite {
                        hallazgos.push(Hallazgo {
                            ruta: d.ruta.clone(),
                            tipo: Tipo::PrimeraMuyAlta,
                            era: 0,
                            ahora: sello,
                            limite,
                        });
                    }
                }
                // El waiver no puede rebasar el trinquete.
                if d.max > sello {
                    hallazgos.push(Hallazgo {
                        ruta: d.ruta.clone(),
                        tipo: Tipo::SobreSello,
                        era: sello,
                        ahora: d.max,
                        limite: sello,
                    });
                }
            }
            None if d.tier_presupuesto <= 0 => {
                // Waiver en un tier sin presupuesto (p.ej. log): inerte, no
                // hay techo que rebasar. Información, no fallo.
                hallazgos.push(Hallazgo {
                    ruta: d.ruta.clone(),
                    tipo: Tipo::WaiverLogInerte,
                    era: 0,
                    ahora: d.max,
                    limite: 0,
                });
            }
            None => {
                let limite = d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION;
                if d.max > limite {
                    hallazgos.push(Hallazgo {
                        ruta: d.ruta.clone(),
                        tipo: Tipo::PrimeraMuyAlta,
                        era: 0,
                        ahora: d.max,
                        limite,
                    });
                }
            }
        }
    }
    hallazgos
}

/// Una nota sellada que ya no declara waiver y cuyo tier ACTUAL no tiene
/// presupuesto se reclasificó a `log` para escapar del gate: el sello es la
/// prueba de que tuvo techo. Solo mira los sellos SIN `Declarada` — con
/// `Declarada` ya pasó por `checks_de_declaracion`.
///
/// Lee el tier del **disco** también en `--staged`: es el comportamiento
/// heredado de antes de partir `comprueba_contra`, y este refactor no lo
/// cambia.
fn sellos_escapados_de_tier(
    kb: &Path,
    actual: &Sellos,
    declaradas: &[Declarada],
    presupuestos: crate::presupuesto::Presupuestos,
) -> Vec<Hallazgo> {
    let rutas_declaradas: BTreeSet<&str> = declaradas.iter().map(|d| d.ruta.as_str()).collect();
    let mut hallazgos = Vec::new();
    for (ruta, &sello) in actual {
        if rutas_declaradas.contains(ruta.as_str()) {
            continue;
        }
        let Ok(contenido) = std::fs::read_to_string(kb.join(ruta)) else {
            continue; // nota borrada: el sello huérfano se queda, nada que mirar.
        };
        let tier = crate::frontmatter::tier(&contenido);
        if !tier.is_empty() && presupuestos.para_tier(&tier).unwrap_or(0) <= 0 {
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::SelladaEscapadaDeTier,
                era: sello,
                ahora: 0,
                limite: 0,
            });
        }
    }
    hallazgos
}

/// `min(sello_actual, declarado)`: el techo que `--seal` escribiría. Para
/// cada `Declarada`, el resultado se queda con el menor entre el sello que
/// ya había y lo que la nota declara — un techo existente **nunca sube**,
/// ni aunque la nota declare uno más alto. Un sello sin `Declarada` (un
/// huérfano, o cualquier ruta que nadie declara en esta pasada) se
/// **conserva tal cual**: retirarlo devolvería el margen que el trinquete
/// existe para retener. Puerto de `Seal` (`internal/ratchet/collect.go`,
/// kbx `fe46443`).
pub fn sella(actual: &Sellos, declaradas: &[Declarada]) -> Sellos {
    let mut siguiente = actual.clone();
    for d in declaradas {
        match siguiente.get(&d.ruta) {
            Some(&existente) if d.max < existente => {
                siguiente.insert(d.ruta.clone(), d.max);
            }
            None => {
                siguiente.insert(d.ruta.clone(), d.max);
            }
            // Ya existe y `d.max >= existente`: no se toca. Es la mitad del
            // contrato que un `min` ingenuo (sobrescribir siempre) rompería.
            _ => {}
        }
    }
    siguiente
}

/// Los sellos de `siguiente` que la transición dejaría sin aire —
/// restringido a los que **cambian** respecto a `actual`: un techo que
/// nadie toca no es una transición, y esta guarda no lo juzga. Es lo que
/// deja que un `--seal` que parte dos notas selle esas dos limpio mientras
/// otras nueve, intactas, siguen en deuda sin bloquear nada. Puerto de
/// `AirViolations` (`internal/ratchet/collect.go`, kbx `fe46443`).
///
/// La atomicidad de `--seal` no vive aquí: esta función es pura y no toca
/// disco. Es el llamador (Task 12, `exo ratchet --seal`) quien tiene que
/// escribir `siguiente` con `escribe_sellos` únicamente si el resultado de
/// esta función viene vacío — "o sella todo o no sella nada".
///
/// Los sellos huérfanos que `sella` conserva no llevan `Declarada`, así que
/// nunca cambian de valor (siguen exactamente en `actual`) y nunca llegan a
/// esta guarda.
pub fn violaciones_de_aire(
    actual: &Sellos,
    siguiente: &Sellos,
    declaradas: &[Declarada],
) -> Vec<Hallazgo> {
    let por_ruta: BTreeMap<&str, &Declarada> =
        declaradas.iter().map(|d| (d.ruta.as_str(), d)).collect();

    let mut hallazgos = Vec::new();
    for (ruta, &techo) in siguiente {
        let existia = actual.get(ruta);
        if let Some(&era) = existia
            && techo == era
        {
            continue; // no es una transición.
        }
        let Some(&d) = por_ruta.get(ruta.as_str()) else {
            continue; // sin declaración, no hay tamaño que juzgar.
        };
        if crate::presupuesto::tiene_aire(techo, d.tamano) {
            continue;
        }
        if existia.is_none()
            && d.tier_presupuesto > 0
            && crate::presupuesto::techo_minimo(d.tamano)
                > d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION
        {
            // Zona muerta, igual que en `comprueba_contra`: ninguna nota de
            // este tamaño puede tener a la vez aire y respetar el cap de 2×.
            hallazgos.push(Hallazgo {
                ruta: ruta.clone(),
                tipo: Tipo::NaceDemasiadoGrande,
                era: 0,
                ahora: d.tamano,
                limite: d.tier_presupuesto * FACTOR_PRIMERA_DECLARACION,
            });
            continue;
        }
        hallazgos.push(Hallazgo {
            ruta: ruta.clone(),
            tipo: Tipo::SinAire,
            era: 0,
            ahora: techo,
            limite: crate::presupuesto::techo_minimo(d.tamano),
        });
    }
    hallazgos.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    hallazgos
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sellos(pares: &[(&str, i64)]) -> Sellos {
        pares.iter().map(|(r, t)| (r.to_string(), *t)).collect()
    }

    fn conjunto(rutas: &[&str]) -> BTreeSet<String> {
        rutas.iter().map(|r| r.to_string()).collect()
    }

    /// Repo git real y aislado, con `commiteadas` ya commiteadas en HEAD.
    /// Mismo aislamiento que `gitx::tests::repo`: `GIT_CONFIG_GLOBAL` apunta
    /// a un fichero vacío real (no `/dev/null`, que en Windows no vale para
    /// esta variable) e identidad de autor/committer fija por env var.
    fn repo_con(commiteadas: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let raiz = dir.path();
        let cfg = raiz.join("gitconfig-vacio");
        std::fs::write(&cfg, "").unwrap();
        let corre = |args: &[&str]| {
            let salida = std::process::Command::new("git")
                .arg("-C")
                .arg(raiz)
                .args(args)
                .env("GIT_CONFIG_GLOBAL", &cfg)
                .env("GIT_CONFIG_SYSTEM", &cfg)
                .env("GIT_AUTHOR_NAME", "f")
                .env("GIT_AUTHOR_EMAIL", "f@k.local")
                .env("GIT_COMMITTER_NAME", "f")
                .env("GIT_COMMITTER_EMAIL", "f@k.local")
                .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
                .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
                .output()
                .unwrap();
            assert!(salida.status.success(), "git {args:?} falló");
        };
        corre(&["init", "-q"]);
        // Marcador siempre presente: si `commiteadas` viene vacío, sigue
        // habiendo algo que commitear y HEAD resuelve igualmente.
        std::fs::write(raiz.join(".gitkeep"), "").unwrap();
        for ruta in commiteadas {
            let absoluta = raiz.join(ruta);
            if let Some(padre) = absoluta.parent() {
                std::fs::create_dir_all(padre).unwrap();
            }
            std::fs::write(&absoluta, "contenido\n").unwrap();
        }
        corre(&["add", "."]);
        corre(&["commit", "-q", "-m", "inicial"]);
        dir
    }

    // Task 7 — el emparejamiento de renames.

    #[test]
    fn un_sello_retirado_absuelve_a_uno_solo_no_a_dos() {
        let repo = repo_con(&["old.md"]);
        let head = sellos(&[("old.md", 100)]);
        let actual = sellos(&[("new1.md", 90), ("new2.md", 90)]);
        let (frescos, retirados) = empareja_renames(repo.path(), &head, &actual);
        assert_eq!(frescos.len(), 1, "un retirado no puede absolver a dos");
        assert_eq!(retirados, conjunto(&["old.md"]));
    }

    // El corazón de la tarea (0ae126d / f0d0564): fixture literal del plan —
    // `core/junk.md` NUNCA se commitea, así que es un sello huérfano en
    // `head`. Si la guarda no comprueba `existio_en_head`, este sello
    // absolvería `core/attack.md` (45.000 B de techo para una nota real de
    // 44.000 B, 1,3% de aire) y el gate saldría verde sin un solo hallazgo.
    // Aquí solo se comprueba el emparejamiento en sí: que el huérfano no
    // pueda absolver nada. El "cero hallazgos" completo del bug se falsa en
    // `trinquete_declaraciones.rs`/`trinquete_aire.rs`, una vez existe
    // `comprueba` (Task 9).
    #[test]
    fn un_sello_huerfano_no_puede_hacer_de_rename() {
        let repo = repo_con(&["otro.md"]); // core/junk.md no está aquí.
        let head = sellos(&[("core/junk.md", 45000)]);
        let actual = sellos(&[("core/attack.md", 45000)]);
        let (frescos, retirados) = empareja_renames(repo.path(), &head, &actual);
        assert!(frescos.is_empty(), "el huérfano no debe absolver nada");
        assert!(retirados.is_empty());
    }

    // Misma forma que el test anterior, pero `core/junk.md` SÍ está
    // commiteado: sin este test, la guarda de `existio_en_head` podría
    // implementarse "rechazando siempre" y el test del huérfano pasaría
    // igual de verde.
    #[test]
    fn un_rename_real_sigue_absuelto_tras_la_guarda() {
        let repo = repo_con(&["core/junk.md"]);
        let head = sellos(&[("core/junk.md", 45000)]);
        let actual = sellos(&[("core/attack.md", 45000)]);
        let (frescos, retirados) = empareja_renames(repo.path(), &head, &actual);
        assert_eq!(frescos, conjunto(&["core/attack.md"]));
        assert_eq!(retirados, conjunto(&["core/junk.md"]));
    }

    #[test]
    fn el_emparejamiento_prefiere_el_techo_que_conservo() {
        let repo = repo_con(&["old.md"]);
        let head = sellos(&[("old.md", 100)]);
        // `exact.md` conserva el mismo techo que `old.md` tenía; `low.md` es
        // una declaración distinta y más pequeña. El emparejamiento debe
        // preferir el que coincide, no el que más "cabe holgado".
        let actual = sellos(&[("low.md", 30), ("exact.md", 100)]);
        let (frescos, retirados) = empareja_renames(repo.path(), &head, &actual);
        assert_eq!(frescos, conjunto(&["exact.md"]));
        assert_eq!(retirados, conjunto(&["old.md"]));
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

    // H15 — cada familia de `comprueba_contra`, por separado y sin git.

    fn declarada(ruta: &str, max: i64, tier_presupuesto: i64, tamano: i64) -> Declarada {
        Declarada {
            ruta: ruta.to_string(),
            tier: "core".to_string(),
            max,
            tier_presupuesto,
            tamano,
        }
    }

    fn sin_tamano(_: &str) -> Option<i64> {
        None
    }

    #[test]
    fn aire_un_sello_intacto_sin_aire_es_deuda_no_fallo() {
        // 100 B bajo techo 100: 10.000 < 11.500, sin aire. Intacto desde HEAD.
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, grandes) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::DeudaSinAire);
        assert_eq!(h[0].ahora, 100);
        assert_eq!(h[0].limite, 115, "techo_minimo(100) = ceil(115)");
        assert!(grandes.is_empty());
    }

    #[test]
    fn aire_un_sello_fresco_sin_aire_es_sin_aire() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SinAire);
        assert_eq!(h[0].limite, 115);
    }

    #[test]
    fn aire_la_activacion_consagra_el_sello_fresco() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 100, 8500, 100)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, true, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty(), "activación: {h:?}");
    }

    #[test]
    fn aire_una_nota_en_zona_muerta_nace_demasiado_grande_y_se_marca() {
        // 18.000 B: techo_minimo = 20.700 > 2 × 8.500.
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 18000)];
        let (h, grandes) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::NaceDemasiadoGrande);
        assert_eq!(h[0].ahora, 18000);
        assert_eq!(h[0].limite, 17000);
        assert_eq!(grandes, conjunto(&["a.md"]));
    }

    #[test]
    fn aire_una_subida_no_se_etiqueta_ademas_como_deuda() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 200)]);
        let decl = [declarada("a.md", 200, 8500, 190)];
        let (h, _) = guarda_de_aire(&head, &actual, &decl, false, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty(), "la subida ya es SelloSubido: {h:?}");
    }

    #[test]
    fn aire_sin_declaracion_ni_tamano_legible_se_salta() {
        let head = sellos(&[]);
        let actual = sellos(&[("huerfano.md", 100)]);
        let (h, _) = guarda_de_aire(&head, &actual, &[], false, &conjunto(&[]), sin_tamano);
        assert!(h.is_empty());
    }

    #[test]
    fn aire_sin_declaracion_usa_el_tamano_de_la_revision() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 100)]);
        let (h, _) = guarda_de_aire(&head, &actual, &[], false, &conjunto(&[]), |_| Some(100));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SinAire);
    }

    #[test]
    fn declaracion_un_waiver_por_encima_del_sello_es_sobre_sello() {
        let head = sellos(&[("a.md", 100)]);
        let actual = sellos(&[("a.md", 100)]);
        let decl = [declarada("a.md", 150, 8500, 50)];
        let h = checks_de_declaracion(&head, &actual, &decl, false, &conjunto(&[]), &conjunto(&[]));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::SobreSello);
        assert_eq!((h[0].era, h[0].ahora, h[0].limite), (100, 150, 100));
    }

    #[test]
    fn declaracion_un_sello_fresco_por_encima_de_2x_es_primera_muy_alta() {
        let head = sellos(&[]);
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 100)];
        let h = checks_de_declaracion(&head, &actual, &decl, false, &conjunto(&[]), &conjunto(&[]));
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::PrimeraMuyAlta);
        assert_eq!((h[0].ahora, h[0].limite), (20000, 17000));
    }

    #[test]
    fn declaracion_sin_sello_por_encima_de_2x_es_primera_muy_alta() {
        let decl = [declarada("a.md", 20000, 8500, 100)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &sellos(&[]),
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&[]),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::PrimeraMuyAlta);
        assert_eq!(h[0].ahora, 20000);
    }

    #[test]
    fn declaracion_un_waiver_en_tier_sin_presupuesto_es_inerte() {
        let decl = [declarada("a.md", 500, 0, 100)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &sellos(&[]),
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&[]),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].tipo, Tipo::WaiverLogInerte);
        assert!(!h[0].tipo.rompe());
    }

    #[test]
    fn declaracion_no_apila_hallazgos_sobre_una_nota_demasiado_grande() {
        let actual = sellos(&[("a.md", 20000)]);
        let decl = [declarada("a.md", 20000, 8500, 18000)];
        let h = checks_de_declaracion(
            &sellos(&[]),
            &actual,
            &decl,
            false,
            &conjunto(&[]),
            &conjunto(&["a.md"]),
        );
        assert!(h.is_empty(), "{h:?}");
    }

    #[test]
    fn escapados_un_sello_sin_waiver_en_tier_log_escapo_del_gate() {
        let kb = tempfile::tempdir().unwrap();
        std::fs::write(kb.path().join("a.md"), "---\ntier: log\n---\nx\n").unwrap();
        std::fs::write(kb.path().join("b.md"), "---\ntier: core\n---\nx\n").unwrap();
        let actual = sellos(&[("a.md", 9000), ("b.md", 9000), ("borrada.md", 9000)]);
        let h = sellos_escapados_de_tier(kb.path(), &actual, &[], crate::presupuesto::NOMINALES);
        assert_eq!(
            h.len(),
            1,
            "solo a.md: b.md tiene presupuesto, borrada.md no se lee"
        );
        assert_eq!(h[0].ruta, "a.md");
        assert_eq!(h[0].tipo, Tipo::SelladaEscapadaDeTier);
        assert_eq!(h[0].era, 9000);
    }

    #[test]
    fn escapados_ignora_los_sellos_con_declaracion() {
        let kb = tempfile::tempdir().unwrap();
        std::fs::write(kb.path().join("a.md"), "---\ntier: log\n---\nx\n").unwrap();
        let actual = sellos(&[("a.md", 9000)]);
        let decl = [declarada("a.md", 9000, 0, 10)];
        let h = sellos_escapados_de_tier(kb.path(), &actual, &decl, crate::presupuesto::NOMINALES);
        assert!(h.is_empty());
    }
}
