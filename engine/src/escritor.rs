//! Write-path de exo (M4/E2). File-first: escribe el markdown de la KB
//! directamente — la KB es un repo git, así que el rollback es `git checkout`
//! (spec madre §4.4-E2). NO commitea y NO indexa: lo primero es del agente
//! (commit scoped por rutas), lo segundo lo absorbe el `--refresh` del recall
//! de la sesión siguiente (M6-01).
//!
//! Reparto de trabajo con el agente (spec M4 §1): aquí viven `new` y `append`
//! —mecánica con línea roja encima, y el append escribe sin leer—; la edición
//! del canon se queda en la tool `Edit`, que opera sobre texto exacto y no
//! parsea headings.

use anyhow::{Context, Result};
use serde::Serialize;
use std::io::{Seek, SeekFrom, Write};
use crate::nota::comilla_simple_yaml;
use std::path::{Path, PathBuf};

/// Gate rechazado: una decisión que se le devuelve al llamador, no un fallo.
/// El CLI lo traduce a **exit 3** (distinto del 1 de error real) para que el
/// consumidor gatee por exit code, jamás por campos de `data`.
#[derive(Debug)]
pub enum Rechazo {
    /// `busca_hybrid` encontró notas parecidas al título de la nueva (M4-02).
    Duplicada { candidatas: Vec<Candidata> },
    /// Append a una nota que no es `tier: log`. Es EL anti-patrón medido de
    /// esta KB: 52 `## Delta AAAA-MM-DD` anexados al canon en la historia
    /// real, causa del 100% de los incidentes caros (spec M4 §7.1).
    AppendACanon { tier: Option<String> },
    /// El bloque a anexar trae cabeceras `## ` que la nota ya tiene: el
    /// reintento tras un fallo parcial duplicaría la entrada de bitácora.
    CabeceraDuplicada { cabeceras: Vec<String> },
    /// El bloque supera `MAX_LINEAS_APPEND` líneas no vacías: una entrada de
    /// bitácora son hechos y wikilinks, no una re-narración (document/SKILL.md).
    CuerpoLargo { lineas: usize },
    /// Cabecera `## ` que no es `## AAAA-MM-DD — <tema>`, o cuyo tema es solo
    /// una etiqueta genérica (`generica`). El puntero del canon cita la
    /// cabecera: sin tema grepeable deja de resolver.
    CabeceraInvalida {
        cabeceras: Vec<String>,
        generica: bool,
    },
}

impl std::fmt::Display for Rechazo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rechazo::Duplicada { candidatas } => {
                write!(
                    f,
                    "ya existe nota parecida ({}). Edita la canónica, o repite con --force si de verdad es tema nuevo",
                    candidatas
                        .iter()
                        .map(|c| format!("{} ~{:.2}", c.permalink, c.score))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Rechazo::AppendACanon { tier } => {
                let tier = tier.as_deref().unwrap_or("(sin tier)");
                write!(
                    f,
                    "append a nota tier '{tier}': el canon se edita como delta, no se anexa. \
                     Usa la bitácora del frente, o --force si es una excepción consciente"
                )
            }
            Rechazo::CabeceraDuplicada { cabeceras } => write!(
                f,
                "la nota ya tiene la cabecera ({}): el bloque parece ya anexado. \
                 Repite con --force si de verdad quieres duplicarla",
                cabeceras.join("; ")
            ),
            Rechazo::CuerpoLargo { lineas } => write!(
                f,
                "el bloque tiene {lineas} líneas (máx. {MAX_LINEAS_APPEND}): una entrada de \
                 bitácora son hechos y wikilinks, sin re-narrar. Acórtala, o --force si es una excepción consciente"
            ),
            Rechazo::CabeceraInvalida {
                cabeceras,
                generica,
            } => write!(
                f,
                "cabecera {} ({}): usa `## AAAA-MM-DD — <tema concreto>`. \
                 Concreta el tema, o --force si es una excepción consciente",
                if *generica {
                    "con etiqueta genérica"
                } else {
                    "fuera de formato"
                },
                cabeceras.join("; ")
            ),
        }
    }
}

impl std::error::Error for Rechazo {}

impl Rechazo {
    /// `data` del envelope de un rechazo (spec de write §3.3). Existe para que
    /// el consumidor tenga el detalle en JSON, no solo una línea de prosa en
    /// stderr — un contrato prometido por escrito y servido por prosa es la
    /// definición de contrato no falsable.
    ///
    /// El exit code sigue siendo el gate: esto es el detalle, no la señal.
    pub fn data(&self) -> serde_json::Value {
        match self {
            Rechazo::Duplicada { candidatas } => serde_json::json!({
                "reason": "duplicate",
                "candidates": candidatas,
            }),
            Rechazo::AppendACanon { tier } => serde_json::json!({
                "reason": "append_to_canon",
                "tier": tier,
            }),
            Rechazo::CabeceraDuplicada { cabeceras } => serde_json::json!({
                "reason": "duplicate_header",
                "cabeceras": cabeceras,
            }),
            Rechazo::CuerpoLargo { lineas } => serde_json::json!({
                "reason": "body_too_long",
                "lines": lineas,
                "max": MAX_LINEAS_APPEND,
            }),
            Rechazo::CabeceraInvalida {
                cabeceras,
                generica,
            } => serde_json::json!({
                "reason": if *generica { "generic_header" } else { "bad_header" },
                "cabeceras": cabeceras,
            }),
        }
    }
}

/// Candidata duplicada devuelta por el dup-gate.
#[derive(Debug, Clone, Serialize)]
pub struct Candidata {
    pub permalink: String,
    pub score: f64,
}

/// `data` del envelope de `exo write`.
#[derive(Debug, Serialize)]
pub struct Escritura {
    pub op: String,
    pub permalink: String,
    #[serde(rename = "relative_path")]
    pub ruta_rel: String,
    #[serde(rename = "absolute_path")]
    pub ruta_abs: String,
    /// `true` si el fichero no existía y esta invocación lo creó.
    #[serde(rename = "created")]
    pub creada: bool,
    /// Claves de frontmatter que exo rellenó porque faltaban (M4-03:
    /// auto-completa y nunca rechaza). Vacío = el autor lo traía todo.
    #[serde(rename = "frontmatter_filled")]
    pub frontmatter_completado: Vec<String>,
    /// Claves obligatorias (`tier`, `tags`) que la nota escrita NO lleva.
    /// exo no las inventa —`tier` decide el presupuesto, `tags` es semántica—
    /// y tampoco rechaza (M4-03), así que la ausencia se declara aquí y por
    /// stderr en vez de salir muda. Vacío = nota completa.
    #[serde(rename = "frontmatter_missing")]
    pub frontmatter_faltante: Vec<String>,
    /// `--force` usado. Se emite SIEMPRE que se fuerza, para que el escape
    /// quede auditable (spec M4 §7.3: un guard sin vía de excepción muere por
    /// ruido; una vía de excepción sin rastro es peor).
    #[serde(rename = "forced")]
    pub forzado: bool,
}

/// Slug de un título, réplica del **diseño** de permalinks de basic-memory
/// verificada contra la KB de producción (jamás su código: veto AGPL).
/// Minúsculas, sin diacríticos, se conservan letras/dígitos/`.`, todo lo demás
/// colapsa a un guion único sin guiones en los extremos.
///
/// Oráculo real: `pguerrero.me — Hub personal / portfolio con Lab explorable
/// de LLMs` → `pguerrero.me-hub-personal-portfolio-con-lab-explorable-de-llms`
/// (el punto sobrevive, el em-dash y la barra colapsan) y `kbx — explorador
/// determinista de la KB (Go)` → `kbx-explorador-determinista-de-la-kb-go`.
pub fn slug(titulo: &str) -> String {
    let mut salida = String::with_capacity(titulo.len());
    let mut pendiente_guion = false;

    for c in titulo.chars() {
        let base = sin_diacritico(c);
        if base.is_ascii_alphanumeric() || base == '.' {
            if pendiente_guion && !salida.is_empty() {
                salida.push('-');
            }
            pendiente_guion = false;
            salida.extend(base.to_lowercase());
        } else {
            pendiente_guion = true;
        }
    }
    salida
}

/// Plegado de diacríticos del subconjunto que aparece en esta KB (castellano,
/// euskera y préstamos). Sin crate de unicode: la tabla es corta, explícita y
/// no añade superficie de dependencia por un puñado de caracteres.
fn sin_diacritico(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'Á' | 'À' | 'Ä' | 'Â' | 'Ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
        'ñ' | 'Ñ' => 'n',
        'ç' | 'Ç' => 'c',
        otro => otro,
    }
}

/// Umbral del dup-gate: solape de tokens (Jaccard) entre el slug de la nota
/// nueva y el de una existente. Calibrado contra el ÚNICO duplicado real de
/// la historia de la KB (1 caso en 153 invocaciones de `/documenta`):
/// `ai-news-bitacora` creada existiendo `ai-news-pipeline-bitacora` → Jaccard
/// 0.75, lo caza. Y contra los falsos positivos que importan: `exo-bitacora`
/// vs `kbx-bitacora` → 0.33, `backlog-diario` vs `backlog-frentes-abiertos` →
/// 0.25, ninguno salta.
const UMBRAL_DUP: f64 = 0.6;

/// Solape de tokens entre dos slugs. **Deliberadamente NO usa retrieval
/// semántico**: el umbral de `busca_hybrid` (0.40 desde D6, Ola 1 G Task 11
/// — antes 0.35-0.40 según viniera de config o del sellado del sweep, hoy
/// los dos extremos son la misma constante `MIN_SIMILARITY_SELLADO`) está
/// calibrado para "tráeme contexto relevante", que es otra pregunta que
/// "esto ya existe" — usarlo como dup-gate produce falsos rojos (verificado:
/// un título sin relación alguna puntuaba 0.36 contra una bitácora
/// cualquiera). Un guard que rebota al cierre de sesión acaba desactivado,
/// que es como murió el primer guard de kbx (spec M4 §7.3).
///
/// Extra: es determinista y no carga el modelo ONNX, así que el gate no le
/// añade segundos al cierre de sesión.
pub fn solape_slug(a: &str, b: &str) -> f64 {
    let tokens = |s: &str| -> std::collections::HashSet<String> {
        s.split(['-', '.'])
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect()
    };
    let (ta, tb) = (tokens(a), tokens(b));
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let interseccion = ta.intersection(&tb).count() as f64;
    let union = ta.union(&tb).count() as f64;
    interseccion / union
}

/// Nombre de fichero de una nota a partir de su título. La barra colapsa a
/// guion, **replicando lo que hace basic-memory en producción**: la nota
/// `pguerrero.me — Hub personal / portfolio con Lab explorable de LLMs` vive
/// en el fichero `…Hub personal - portfolio…md`, con el `title` del
/// frontmatter intacto. Sin esto, un título con barra crearía un
/// subdirectorio accidental — hay un caso real en la KB.
fn nombre_fichero(titulo: &str) -> String {
    titulo.replace(['/', '\\'], "-")
}

/// Rechaza segmentos de ruta que escaparían del árbol de la KB. Es una línea
/// roja dura: el write-path solo escribe DENTRO de la KB, y un `..` en
/// `--dir` o `--title` la atravesaría (verificado en el gate). No es un gate
/// saltable con `--force`: es un error.
fn verifica_segmento(valor: &str, flag: &str) -> Result<()> {
    if valor
        .split(['/', '\\'])
        .any(|seg| seg == ".." || seg == ".")
    {
        anyhow::bail!(
            "{flag} contiene un segmento de ruta relativo ({valor:?}): el write-path \
             solo escribe dentro de la KB"
        );
    }
    if valor.starts_with('/') || valor.starts_with('\\') {
        anyhow::bail!("{flag} no puede ser una ruta absoluta ({valor:?})");
    }
    if valor.trim().is_empty() {
        anyhow::bail!("{flag} no puede estar vacío");
    }
    Ok(())
}

/// Candidatas duplicadas de un slug nuevo contra los permalinks ya indexados.
/// El llamador pasa la lista de permalinks; este módulo no conoce el índice.
pub fn dup_candidatas(slug_nuevo: &str, permalinks: &[String]) -> Vec<(String, f64)> {
    let mut encontradas: Vec<(String, f64)> = permalinks
        .iter()
        .filter_map(|p| {
            let slug_existente = p.rsplit('/').next().unwrap_or(p);
            let score = solape_slug(slug_nuevo, slug_existente);
            (score >= UMBRAL_DUP).then(|| (p.clone(), score))
        })
        .collect();
    encontradas.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    encontradas
}

/// Parámetros de `escribe_nueva` (Ola 1 G Task 6, backlog:637-647): antes
/// eran 8 argumentos posicionales contra el umbral de 7 de clippy, con
/// `#[allow(clippy::too_many_arguments)]` declarado como deuda desde G5a
/// (2026-09-02). `dup_candidatas` lo calcula el llamador (el CLI, con
/// `busca_hybrid`): este módulo no conoce el índice, solo el filesystem.
pub struct NuevaNota<'a> {
    pub kb: &'a Path,
    pub proyecto: &'a str,
    pub dir: &'a str,
    pub titulo: &'a str,
    pub cuerpo: &'a str,
    pub tier: Option<&'a str>,
    pub dup_candidatas: &'a [(String, f64)],
    pub forzado: bool,
}

/// Crea una nota nueva. `cuerpo` puede traer frontmatter propio: se conserva
/// **literal** y solo se le añaden delante las claves que falten (M4-03).
/// Nunca sobrescribe un fichero existente — eso es error duro, no gate: lo
/// correcto ante una colisión es append o edit. Si `dup_candidatas` llega no
/// vacío, `Rechazo::Duplicada` sin tocar el disco.
pub fn escribe_nueva(n: &NuevaNota) -> Result<Escritura> {
    if !n.dup_candidatas.is_empty() {
        return Err(Rechazo::Duplicada {
            candidatas: n
                .dup_candidatas
                .iter()
                .map(|(permalink, score)| Candidata {
                    permalink: permalink.clone(),
                    score: *score,
                })
                .collect(),
        }
        .into());
    }

    verifica_segmento(n.dir, "--dir")?;
    verifica_segmento(n.titulo, "--title")?;

    let permalink = format!("{}/{}/{}", n.proyecto, n.dir, slug(n.titulo));
    let ruta_rel = format!("{}/{}.md", n.dir, nombre_fichero(n.titulo));
    let ruta_abs = n.kb.join(&ruta_rel);

    if ruta_abs.exists() {
        anyhow::bail!(
            "{} ya existe: una nota jamás se sobrescribe (usa append, o edita con Edit)",
            ruta_abs.display()
        );
    }

    let (yaml_previo, cuerpo_limpio) = separa_frontmatter(n.cuerpo);
    let (frontmatter, completado) = compone_frontmatter(&yaml_previo, n.titulo, &permalink, n.tier);

    let frontmatter_faltante = ["tier", "tags"]
        .into_iter()
        .filter(|k| valor_yaml(&frontmatter, k).is_none())
        .map(String::from)
        .collect();

    // `tier` y `type` van sin comillas: el scanner de `frontmatter.rs` los lee
    // crudos y un `'stable'` caería a NOTIER. Por eso se valida lo compuesto.
    if let Err(e) = crate::nota::valida_yaml(&frontmatter) {
        anyhow::bail!("frontmatter YAML inválido, no se escribe la nota: {e}");
    }

    let contenido = format!("---\n{frontmatter}---\n{cuerpo_limpio}");
    escribe_atomico(&ruta_abs, &contenido)?;

    Ok(Escritura {
        op: "new".into(),
        permalink,
        ruta_rel,
        ruta_abs: crate::walker::ruta_portable(&ruta_abs.display().to_string()),
        creada: true,
        frontmatter_completado: completado,
        frontmatter_faltante,
        forzado: n.forzado,
    })
}

/// Anexa al final de una nota existente **sin leer ni reescribir su cuerpo**:
/// un `write` en modo `O_APPEND`. Motivo: las bitácoras son grandes
/// (`log/exo-bitacora.md` ~14 KB, el backlog ~33 KB) y `Edit` obligaría a
/// cargarlas enteras en cada cierre de sesión.
///
/// Rechaza salvo `forzar` si la nota destino no es `tier: log` (§7.1), si el
/// bloque excede el tope de líneas o trae cabeceras mal formadas o genéricas
/// (`valida_formato_append`), o si trae una cabecera `## ` que la nota viva ya
/// tiene (el tier manda).
/// Para leer el `tier` toca el head del fichero; para las cabeceras recorre la
/// nota línea a línea con `BufRead`, sin cargarla entera.
pub fn escribe_append(kb: &Path, ruta_rel: &str, texto: &str, forzar: bool) -> Result<Escritura> {
    let ruta_abs = kb.join(ruta_rel);
    if !ruta_abs.exists() {
        anyhow::bail!(
            "{} no existe: para crear la bitácora usa --create",
            ruta_abs.display()
        );
    }

    let cabecera = lee_cabecera(&ruta_abs)?;
    let (yaml, _) = separa_frontmatter(&cabecera);
    let tier_raw = valor_yaml(&yaml, "tier").unwrap_or_default();
    let permalink = valor_yaml(&yaml, "permalink").unwrap_or_default();

    if tier_raw != "log" && !forzar {
        // Ausencia de tier = `None`, no el centinela de prosa `"(sin tier)"`:
        // ese centinela sigue existiendo, pero solo en el `Display` humano
        // (arriba). Un consumidor de `data()` que lea `data.tier` necesita
        // poder distinguir por TIPO entre "core" (tier real) y ausencia.
        let tier = if tier_raw.is_empty() {
            None
        } else {
            Some(tier_raw)
        };
        return Err(Rechazo::AppendACanon { tier }.into());
    }

    if !forzar {
        valida_formato_append(texto)?;
        let cabeceras = cabeceras_repetidas(&ruta_abs, texto)?;
        if !cabeceras.is_empty() {
            return Err(Rechazo::CabeceraDuplicada { cabeceras }.into());
        }
    }

    anexa(&ruta_abs, texto)?;

    Ok(Escritura {
        op: "append".into(),
        permalink,
        ruta_rel: crate::walker::ruta_portable(ruta_rel),
        ruta_abs: crate::walker::ruta_portable(&ruta_abs.display().to_string()),
        creada: false,
        frontmatter_completado: Vec::new(),
        frontmatter_faltante: Vec::new(),
        forzado: forzar,
    })
}

/// Tope de líneas no vacías por append (document/SKILL.md: «append ≤15 líneas»).
const MAX_LINEAS_APPEND: usize = 15;

/// Etiquetas que por sí solas no identifican una entrada (document/SKILL.md).
const ETIQUETAS_GENERICAS: &[&str] = &[
    "canon",
    "delta",
    "detalle",
    "distill",
    "sesión",
    "sesion",
    "movido del canon",
];

/// Prefijo de los bloques que `distill` y `auto-distill` vuelcan desde el
/// sidecar `-moved.md`: verbatim del canon, de longitud arbitraria.
const PREFIJO_MOVIDO: &str = "movido del canon:";

/// `Some(tema)` si la línea es `## AAAA-MM-DD — <tema>` con tema no vacío.
fn tema_de_cabecera(linea: &str) -> Option<&str> {
    let resto = linea.strip_prefix("## ")?;
    let (fecha, tema) = resto.split_once(" — ")?;
    let b = fecha.as_bytes();
    let fecha_ok = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    let tema = tema.trim();
    (fecha_ok && !tema.is_empty()).then_some(tema)
}

/// Gate de forma del append: tope de líneas y cabeceras `## AAAA-MM-DD — tema`
/// sin etiqueta genérica. Los bloques movidos (todas las cabeceras con
/// `movido del canon: <tema>`) no tienen tope de líneas.
fn valida_formato_append(texto: &str) -> Result<()> {
    let mut malas = Vec::new();
    let mut genericas = Vec::new();
    let mut hay_cabeceras = false;
    let mut todas_movidas = true;
    for l in texto.lines().filter(|l| l.starts_with("## ")) {
        let l = l.trim_end();
        hay_cabeceras = true;
        match tema_de_cabecera(l) {
            None if !malas.iter().any(|m| m == l) => malas.push(l.to_string()),
            None => {}
            Some(tema) => {
                let tema_min = tema.to_lowercase();
                let etiqueta = tema_min.trim_end_matches(['.', ':', ';']).trim();
                if ETIQUETAS_GENERICAS.contains(&etiqueta) {
                    genericas.push(l.to_string());
                }
                if !tema_min.starts_with(PREFIJO_MOVIDO) {
                    todas_movidas = false;
                }
            }
        }
        if tema_de_cabecera(l).is_none() {
            todas_movidas = false;
        }
    }
    if !malas.is_empty() {
        return Err(Rechazo::CabeceraInvalida {
            cabeceras: malas,
            generica: false,
        }
        .into());
    }
    if !genericas.is_empty() {
        return Err(Rechazo::CabeceraInvalida {
            cabeceras: genericas,
            generica: true,
        }
        .into());
    }
    let lineas = texto.lines().filter(|l| !l.trim().is_empty()).count();
    if lineas > MAX_LINEAS_APPEND && !(hay_cabeceras && todas_movidas) {
        return Err(Rechazo::CuerpoLargo { lineas }.into());
    }
    Ok(())
}

/// Cabeceras `## ` del bloque entrante que ya existen en la nota, en orden de
/// aparición y sin repetir. Compara sin espacio ni CR finales.
fn cabeceras_repetidas(ruta: &Path, texto: &str) -> Result<Vec<String>> {
    use std::io::BufRead;
    let norm = |l: &str| l.trim_end().to_string();
    let mut entrantes: Vec<String> = Vec::new();
    for l in texto.lines().filter(|l| l.starts_with("## ")) {
        let l = norm(l);
        if !entrantes.contains(&l) {
            entrantes.push(l);
        }
    }
    if entrantes.is_empty() {
        return Ok(Vec::new());
    }
    let f = std::fs::File::open(ruta).with_context(|| format!("abrir {}", ruta.display()))?;
    let mut vistas = std::collections::HashSet::new();
    for linea in std::io::BufReader::new(f).split(b'\n') {
        let linea = linea.with_context(|| format!("leer {}", ruta.display()))?;
        if linea.starts_with(b"## ") {
            vistas.insert(norm(&String::from_utf8_lossy(&linea)));
        }
    }
    Ok(entrantes
        .into_iter()
        .filter(|c| vistas.contains(c))
        .collect())
}

/// Escribe con `O_APPEND` garantizando exactamente una línea en blanco de
/// separación. Mira solo el final del fichero (seek), nunca lo carga entero.
fn anexa(ruta: &Path, texto: &str) -> Result<()> {
    // `read` además de `append`: hace falta leer los últimos bytes para saber
    // cuántos saltos de línea añadir. Con `O_APPEND` el seek NO afecta a la
    // escritura (el kernel la fuerza al final siempre), solo a la lectura.
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(ruta)
        .with_context(|| format!("abrir {} en modo append", ruta.display()))?;

    let final_previo = cola(&mut f, 2)?;
    let mut a_escribir = String::new();
    if !final_previo.is_empty() {
        // "…texto"   → "\n\n";  "…texto\n" → "\n";  "…texto\n\n" → nada.
        let saltos = final_previo
            .chars()
            .rev()
            .take_while(|c| *c == '\n')
            .count();
        for _ in saltos..2 {
            a_escribir.push('\n');
        }
    }
    a_escribir.push_str(texto);

    f.write_all(a_escribir.as_bytes())
        .with_context(|| format!("anexar a {}", ruta.display()))?;
    Ok(())
}

/// Últimos `n` bytes del fichero como texto (lossy: solo se inspeccionan
/// saltos de línea ASCII, un corte a mitad de UTF-8 es inocuo aquí).
fn cola(f: &mut std::fs::File, n: usize) -> Result<String> {
    let len = f.metadata().context("metadata del fichero")?.len();
    if len == 0 {
        return Ok(String::new());
    }
    let atras = std::cmp::min(len, n as u64);
    f.seek(SeekFrom::End(-(atras as i64)))
        .context("seek al final")?;
    let mut buf = vec![0u8; atras as usize];
    std::io::Read::read_exact(f, &mut buf).context("leer cola del fichero")?;
    f.seek(SeekFrom::End(0)).context("volver al final")?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Primeros 8 KB del fichero: de sobra para el frontmatter más largo de la KB,
/// y evita cargar una bitácora entera solo para leer su `tier`.
fn lee_cabecera(ruta: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(ruta).with_context(|| format!("abrir {}", ruta.display()))?;
    let mut buf = vec![0u8; 8192];
    let leidos = f.read(&mut buf).context("leer cabecera")?;
    buf.truncate(leidos);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Valor escalar de una clave de primer nivel del YAML del frontmatter.
/// Deliberadamente textual: no deserializa (eso perdería las claves que no
/// conoce, que es justo el bug que este módulo no puede permitirse).
fn valor_yaml(yaml: &str, clave: &str) -> Option<String> {
    let prefijo = format!("{clave}:");
    yaml.lines()
        .find(|l| l.starts_with(&prefijo))
        .map(|l| descomilla_simple(l[prefijo.len()..].trim()))
}

/// Inversa de `comilla_simple_yaml`: las notas que escribe `write new` llevan
/// `title`/`permalink` entre comillas simples y los lectores textuales de
/// este módulo (p. ej. el permalink de `append`) necesitan el valor sin ellas.
fn descomilla_simple(v: &str) -> String {
    match v.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        Some(interior) => interior.replace("''", "'"),
        None => v.to_string(),
    }
}

/// Divide en (yaml, cuerpo). Sin frontmatter delimitado ⇒ yaml vacío y el
/// texto entero como cuerpo. El corte lo hace `nota::separa_frontmatter`, el
/// mismo que usa el indexer (H19: antes aquí vivía una copia del algoritmo).
/// Este envoltorio solo añade lo que el write-path necesita y el indexer no:
/// conservar el `\n` final del cuerpo.
fn separa_frontmatter(contenido: &str) -> (String, String) {
    match crate::nota::separa_frontmatter(contenido) {
        Some((yaml, mut cuerpo)) => {
            if contenido.ends_with('\n') && !cuerpo.is_empty() {
                cuerpo.push('\n');
            }
            (yaml, cuerpo)
        }
        None => (String::new(), contenido.to_string()),
    }
}

/// Añade las claves obligatorias que falten, **delante** del YAML del autor y
/// sin tocarlo: preservar literal lo que ya está escrito es la única forma de
/// no perder claves que este módulo no conoce (`tags`, `kbx_budget_max`…).
/// Devuelve el bloque y la lista de claves completadas, en orden estable.
fn compone_frontmatter(
    yaml_previo: &str,
    titulo: &str,
    permalink: &str,
    tier: Option<&str>,
) -> (String, Vec<String>) {
    let mut nuevas = String::new();
    let mut completado = Vec::new();

    let añade = |clave: &str, valor: String, nuevas: &mut String, completado: &mut Vec<String>| {
        if valor_yaml(yaml_previo, clave).is_none() {
            nuevas.push_str(&format!("{clave}: {valor}\n"));
            completado.push(clave.to_string());
        }
    };

    añade("permalink", comilla_simple_yaml(permalink), &mut nuevas, &mut completado);
    añade("title", comilla_simple_yaml(titulo), &mut nuevas, &mut completado);
    añade("type", "note".into(), &mut nuevas, &mut completado);
    if let Some(t) = tier {
        añade("tier", t.into(), &mut nuevas, &mut completado);
    }

    let mut bloque = nuevas;
    if !yaml_previo.is_empty() {
        bloque.push_str(yaml_previo);
        bloque.push('\n');
    }
    (bloque, completado)
}

/// Escritura atómica: fichero temporal en el MISMO directorio (mismo
/// filesystem, condición del rename atómico) y `rename` encima. Un CLI que
/// muera a mitad jamás deja una nota a medias.
fn escribe_atomico(destino: &Path, contenido: &str) -> Result<()> {
    let padre = destino
        .parent()
        .context("la nota destino no tiene directorio padre")?;
    std::fs::create_dir_all(padre).with_context(|| format!("crear {}", padre.display()))?;

    let tmp: PathBuf = padre.join(format!(
        ".{}.exo-tmp",
        destino
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "nota".into())
    ));

    {
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("crear temporal {}", tmp.display()))?;
        f.write_all(contenido.as_bytes())
            .with_context(|| format!("escribir temporal {}", tmp.display()))?;
        f.sync_all().context("sync del temporal")?;
    }

    std::fs::rename(&tmp, destino)
        .with_context(|| format!("rename {} → {}", tmp.display(), destino.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests_separa {
    use super::separa_frontmatter;

    /// Caracterización del write-path ANTES del dedupe (H19): estos casos
    /// pasan con la copia vieja y tienen que seguir pasando con el envoltorio.
    #[test]
    fn separa_conserva_el_salto_final_del_cuerpo_y_normaliza_crlf() {
        assert_eq!(
            separa_frontmatter("---\ntitle: X\n---\ncuerpo\n"),
            ("title: X".to_string(), "cuerpo\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\r\ntitle: X\r\n---\r\ncuerpo\r\n"),
            ("title: X".to_string(), "cuerpo\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\n---\ncuerpo"),
            (String::new(), "cuerpo".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\ntitle: X\n---\n"),
            ("title: X".to_string(), String::new())
        );
    }

    #[test]
    fn sin_frontmatter_o_sin_cierre_todo_es_cuerpo() {
        assert_eq!(
            separa_frontmatter("sin frontmatter\n"),
            (String::new(), "sin frontmatter\n".to_string())
        );
        assert_eq!(
            separa_frontmatter("---\ntitle: X\nsin cierre\n"),
            (String::new(), "---\ntitle: X\nsin cierre\n".to_string())
        );
    }

    /// El write-path y el indexer cortan IGUAL el YAML: si divergieran, una nota
    /// escrita por `exo write` podría indexarse con otro frontmatter.
    #[test]
    fn el_write_path_usa_el_corte_del_indexer() {
        for texto in [
            "---\ntitle: X\n---\ncuerpo\n",
            "---\r\na: 1\r\n---\r\nb\r\n",
        ] {
            let (yaml, _) = separa_frontmatter(texto);
            let (yaml_indexer, _) = crate::nota::separa_frontmatter(texto).unwrap();
            assert_eq!(yaml, yaml_indexer);
        }
    }
}
