use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Directorios excluidos en cualquier nivel del árbol (§6.2 regla 3).
const DOTDIRS_EXCLUIDOS: [&str; 3] = [".claude", ".omc", ".superpowers"];

/// Recorre `raiz` recursivamente y devuelve las rutas absolutas de todos los
/// ficheros `.md`, en orden determinista (ordenado por ruta), excluyendo
/// `.claude/`, `.omc/` y `.superpowers/` en cualquier nivel. `archive/` SE
/// incluye (§6.2 regla 4).
pub fn walk_kb(raiz: &Path) -> Result<Vec<PathBuf>> {
    let mut encontradas = Vec::new();
    visita(raiz, &mut encontradas)?;
    encontradas.sort();
    Ok(encontradas)
}

fn visita(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entradas =
        std::fs::read_dir(dir).with_context(|| format!("leer directorio {}", dir.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", dir.display()))?;
        let ruta = entrada.path();
        let tipo = entrada
            .file_type()
            .with_context(|| format!("file_type de {}", ruta.display()))?;

        if tipo.is_dir() {
            let nombre = entrada.file_name();
            let nombre = nombre.to_string_lossy();
            if DOTDIRS_EXCLUIDOS.contains(&nombre.as_ref()) {
                continue;
            }
            visita(&ruta, out)?;
        } else if tipo.is_file() && ruta.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(ruta);
        }
    }
    Ok(())
}

/// ¿Cae `rel` bajo un directorio excluido? Compara el **primer segmento** de la
/// ruta relativa, no el basename de cada nivel (A4). Ante un
/// `projects/archive/` la semántica de basename lo saltaría en silencio; esta
/// lo chequea. Tolera la barra final en las entradas de la lista, como kbx.
///
/// `pub` y en un solo sitio: `doctor.go:31-37` lleva escrito «do not
/// reintroduce a second copy of it», y una segunda copia con tolerancias que
/// derivan es el modo de fallo que esa nota documenta.
pub fn excluida(rel: &str, excluidos: &[&str]) -> bool {
    let seg = rel.split('/').next().unwrap_or(rel);
    excluidos.iter().any(|e| seg == e.trim_end_matches('/'))
}

/// A5: case-insensitive. Un `NOTA.MD` es una nota.
///
/// `to_ascii_lowercase` y no un slice de los últimos 3 bytes: `&nombre[len-3..]`
/// **panica** si ese corte cae a mitad de un carácter multibyte, y esta KB está
/// llena de nombres con em-dash y acentos. Un `.png` mal nombrado no debe
/// tumbar el verbo.
pub fn es_md(nombre: &str) -> bool {
    // `len() > 3` en bytes: un fichero llamado exactamente `.md` no es una nota
    // (y además es un dotfile, que ya se salta antes).
    nombre.len() > 3 && nombre.to_ascii_lowercase().ends_with(".md")
}

/// `(subdirectorios, notas)`: alias para que clippy no se queje de
/// `type_complexity` en la firma — es la misma tupla que pide el brief, solo
/// nombrada; transparente para quien destructura el resultado.
type SubdirsYNotas = (Vec<(String, String)>, Vec<String>);

/// Recorre `raiz` **una vez** y devuelve `(subdirectorios, notas)`:
///
/// - subdirectorios como `(basename, ruta relativa)`, sin filtrar por
///   exclusión — `duplicate_dir` y `budget` la aplican en momentos distintos.
/// - notas: rutas **relativas** (separador `/`, ordenadas) de los `.md` dentro
///   del scope, ya filtradas por `excluidos`.
///
/// Un solo walk porque `lint` necesita las dos cosas y recorrer el árbol dos
/// veces no lo hace ni más simple ni más general.
///
/// Se salta **todo** lo que empiece por `.`, a cualquier nivel: cierra el
/// agujero de `kbx budget`, que recorre `.git/`, `.claude/` y `.omc/` porque su
/// lista de exclusión no los contiene. Es la divergencia 6 del pre-registro.
pub fn walk_kb_excluyendo(raiz: &Path, excluidos: &[&str]) -> Result<SubdirsYNotas> {
    let mut ficheros = Vec::new();
    let mut dirs = Vec::new();
    recorre(raiz, raiz, &mut dirs, &mut ficheros)?;
    ficheros.retain(|rel| !excluida(rel, excluidos));
    ficheros.sort();
    dirs.sort();
    Ok((dirs, ficheros))
}

/// Azúcar para los llamantes que solo quieren las notas.
pub fn walk_notas(raiz: &Path, excluidos: &[&str]) -> Result<Vec<String>> {
    Ok(walk_kb_excluyendo(raiz, excluidos)?.1)
}

/// Lee una nota como texto, tolerando bytes inválidos.
///
/// `read` + `from_utf8_lossy`, **no `read_to_string`**: este es el idioma de la
/// casa (`objetivos.rs` lo hace igual y por lo mismo) y es lo que hace Go, que
/// trabaja sobre bytes. Con `read_to_string`, un solo byte inválido en una nota
/// abortaría el verbo ENTERO con exit 1 — un gate que se apaga por una nota
/// mal codificada en vez de clasificarla.
pub fn lee_nota(ruta: &Path) -> Result<String> {
    let crudo = std::fs::read(ruta).with_context(|| format!("leer nota {}", ruta.display()))?;
    Ok(String::from_utf8_lossy(&crudo).into_owned())
}

/// Una sola grafía de ruta para todo lo que el binario EMITE: `\` → `/`.
///
/// Incondicional a propósito, no `#[cfg(windows)]`. En Unix `\` es un carácter
/// legal en un nombre de fichero, así que esto tiene ahí una arista — pero es
/// la MISMA arista en todos los puntos que la llaman. Gatearla en unos sí y en
/// otros no haría que `walk_kb` y el indexer discreparan ante un fichero
/// `a\b.md`, y esa nota se vería «borrada» y reinsertada en cada corrida.
pub fn ruta_portable(s: &str) -> String {
    s.replace('\\', "/")
}

fn recorre(
    raiz: &Path,
    dir: &Path,
    dirs: &mut Vec<(String, String)>,
    ficheros: &mut Vec<String>,
) -> Result<()> {
    let entradas =
        std::fs::read_dir(dir).with_context(|| format!("leer directorio {}", dir.display()))?;
    for entrada in entradas {
        let entrada = entrada.with_context(|| format!("entrada de {}", dir.display()))?;
        let ruta = entrada.path();
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if nombre.starts_with('.') {
            continue;
        }
        let rel = ruta_portable(
            &ruta
                .strip_prefix(raiz)
                .with_context(|| {
                    format!("{} fuera de la raíz {}", ruta.display(), raiz.display())
                })?
                .to_string_lossy(),
        );
        let tipo = entrada
            .file_type()
            .with_context(|| format!("file_type de {}", ruta.display()))?;
        if tipo.is_dir() {
            dirs.push((nombre, rel));
            recorre(raiz, &ruta, dirs, ficheros)?;
        } else if tipo.is_file() && es_md(&nombre) {
            ficheros.push(rel);
        }
    }
    Ok(())
}
