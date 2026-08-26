//! Write-path de exo (M4/E2). File-first: escribe el markdown de la KB
//! directamente — la KB es un repo git, así que el rollback es `git checkout`
//! (spec madre §4.4-E2). NO commitea y NO indexa: lo primero es del agente
//! (commit scoped por rutas), lo segundo lo absorbe el `--refresca` del recall
//! de la sesión siguiente (M6-01).
//!
//! Reparto de trabajo con el agente (spec M4 §1): aquí viven `new` y `append`
//! —mecánica con línea roja encima, y el append escribe sin leer—; la edición
//! del canon se queda en la tool `Edit`, que opera sobre texto exacto y no
//! parsea headings.

use anyhow::{Context, Result};
use serde::Serialize;
use std::io::{Seek, SeekFrom, Write};
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
    pub ruta_rel: String,
    pub ruta_abs: String,
    /// `true` si el fichero no existía y esta invocación lo creó.
    pub creada: bool,
    /// Claves de frontmatter que exo rellenó porque faltaban (M4-03:
    /// auto-completa y nunca rechaza). Vacío = el autor lo traía todo.
    pub frontmatter_completado: Vec<String>,
    /// `--force` usado. Se emite SIEMPRE que se fuerza, para que el escape
    /// quede auditable (spec M4 §7.3: un guard sin vía de excepción muere por
    /// ruido; una vía de excepción sin rastro es peor).
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
/// semántico**: el umbral de `busca_hybrid` (0.35-0.40) está calibrado para
/// "tráeme contexto relevante", que es otra pregunta que "esto ya existe" —
/// usarlo como dup-gate produce falsos rojos (verificado: un título sin
/// relación alguna puntuaba 0.36 contra una bitácora cualquiera). Un guard que
/// rebota al cierre de sesión acaba desactivado, que es como murió el primer
/// guard de kbx (spec M4 §7.3).
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
/// `--dir` o `--titulo` la atravesaría (verificado en el gate). No es un gate
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

/// Crea una nota nueva. `cuerpo` puede traer frontmatter propio: se conserva
/// **literal** y solo se le añaden delante las claves que falten (M4-03).
/// Nunca sobrescribe un fichero existente — eso es error duro, no gate: lo
/// correcto ante una colisión es append o edit.
///
/// `dup_candidatas` lo calcula el llamador (el CLI, con `busca_hybrid`): este
/// módulo no conoce el índice, solo el filesystem. Si llega no vacío,
/// `Rechazo::Duplicada` sin tocar el disco.
pub fn escribe_nueva(
    kb: &Path,
    proyecto: &str,
    dir: &str,
    titulo: &str,
    cuerpo: &str,
    tier: Option<&str>,
    dup_candidatas: &[(String, f64)],
    forzado: bool,
) -> Result<Escritura> {
    if !dup_candidatas.is_empty() {
        return Err(Rechazo::Duplicada {
            candidatas: dup_candidatas
                .iter()
                .map(|(permalink, score)| Candidata {
                    permalink: permalink.clone(),
                    score: *score,
                })
                .collect(),
        }
        .into());
    }

    verifica_segmento(dir, "--dir")?;
    verifica_segmento(titulo, "--titulo")?;

    let permalink = format!("{proyecto}/{dir}/{}", slug(titulo));
    let ruta_rel = format!("{dir}/{}.md", nombre_fichero(titulo));
    let ruta_abs = kb.join(&ruta_rel);

    if ruta_abs.exists() {
        anyhow::bail!(
            "{} ya existe: una nota jamás se sobrescribe (usa append, o edita con Edit)",
            ruta_abs.display()
        );
    }

    let (yaml_previo, cuerpo_limpio) = separa_frontmatter(cuerpo);
    let (frontmatter, completado) = compone_frontmatter(&yaml_previo, titulo, &permalink, tier);

    let contenido = format!("---\n{frontmatter}---\n{cuerpo_limpio}");
    escribe_atomico(&ruta_abs, &contenido)?;

    Ok(Escritura {
        op: "new".into(),
        permalink,
        ruta_rel,
        ruta_abs: ruta_abs.display().to_string(),
        creada: true,
        frontmatter_completado: completado,
        forzado,
    })
}

/// Anexa al final de una nota existente **sin leer ni reescribir su cuerpo**:
/// un `write` en modo `O_APPEND`. Motivo: las bitácoras son grandes
/// (`log/exo-bitacora.md` ~14 KB, el backlog ~33 KB) y `Edit` obligaría a
/// cargarlas enteras en cada cierre de sesión.
///
/// Rechaza si la nota destino no es `tier: log` (§7.1) salvo `forzar`.
/// Para leer el `tier` toca el head del fichero, no el cuerpo entero.
pub fn escribe_append(kb: &Path, ruta_rel: &str, texto: &str, forzar: bool) -> Result<Escritura> {
    let ruta_abs = kb.join(ruta_rel);
    if !ruta_abs.exists() {
        anyhow::bail!(
            "{} no existe: para crear la bitácora usa --crea",
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

    anexa(&ruta_abs, texto)?;

    Ok(Escritura {
        op: "append".into(),
        permalink,
        ruta_rel: ruta_rel.to_string(),
        ruta_abs: ruta_abs.display().to_string(),
        creada: false,
        frontmatter_completado: Vec::new(),
        forzado: forzar,
    })
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
        .map(|l| l[prefijo.len()..].trim().to_string())
}

/// Divide en (yaml, cuerpo). Sin frontmatter ⇒ yaml vacío y el texto entero
/// como cuerpo. Trabaja por líneas (indiferente a LF/CRLF), igual que
/// `nota::separa_frontmatter`.
fn separa_frontmatter(contenido: &str) -> (String, String) {
    let lineas: Vec<&str> = contenido.lines().collect();
    if lineas.first().map(|l| l.trim_end_matches('\r')) != Some("---") {
        return (String::new(), contenido.to_string());
    }
    let Some(cierre) = lineas[1..]
        .iter()
        .position(|l| l.trim_end_matches('\r') == "---")
        .map(|i| i + 1)
    else {
        return (String::new(), contenido.to_string());
    };

    let yaml = lineas[1..cierre].join("\n");
    let mut cuerpo = lineas[cierre + 1..].join("\n");
    if contenido.ends_with('\n') && !cuerpo.is_empty() {
        cuerpo.push('\n');
    }
    (yaml, cuerpo)
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

    let añade = |clave: &str, valor: &str, nuevas: &mut String, completado: &mut Vec<String>| {
        if valor_yaml(yaml_previo, clave).is_none() {
            nuevas.push_str(&format!("{clave}: {valor}\n"));
            completado.push(clave.to_string());
        }
    };

    añade("permalink", permalink, &mut nuevas, &mut completado);
    añade("title", titulo, &mut nuevas, &mut completado);
    añade("type", "note", &mut nuevas, &mut completado);
    if let Some(t) = tier {
        añade("tier", t, &mut nuevas, &mut completado);
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
