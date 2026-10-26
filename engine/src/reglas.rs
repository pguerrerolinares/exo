//! `exo rules`: reglas duras de proyecto desde la nota de la KB que casa con
//! el repo git del cwd. Solo lectura. Un skip no es un fallo: el consumidor
//! decide si lo muestra; solo un fallo del propio engine sale con exit ≠ 0.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const ENCABEZADO: &str = "## Reglas duras";
pub const CAP: usize = 10;

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Razon {
    SinGit,
    SinNota,
    Ambigua,
    SinSeccion,
    SeccionVacia,
}

impl Razon {
    pub fn como_str(self) -> &'static str {
        match self {
            Razon::SinGit => "sin_git",
            Razon::SinNota => "sin_nota",
            Razon::Ambigua => "ambigua",
            Razon::SinSeccion => "sin_seccion",
            Razon::SeccionVacia => "seccion_vacia",
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Resultado {
    Ok {
        repo: String,
        note: String,
        rules: Vec<String>,
        ignored_lines: Vec<String>,
    },
    Skip {
        repo: Option<String>,
        reason: Razon,
        candidates: Vec<String>,
    },
}

fn skip(repo: Option<String>, reason: Razon, candidates: Vec<String>) -> Resultado {
    Resultado::Skip {
        repo,
        reason,
        candidates,
    }
}

/// ¿Hay una entrada `.git` (fichero o directorio) en `cwd` o sus ancestros?
/// Respeta `GIT_CEILING_DIRECTORIES` como git: el techo y lo de encima no se miran.
fn hay_entrada_git(cwd: &Path) -> bool {
    let techos: Vec<PathBuf> = std::env::var_os("GIT_CEILING_DIRECTORIES")
        .map(|v| {
            std::env::split_paths(&v)
                .filter_map(|p| std::fs::canonicalize(p).ok())
                .collect()
        })
        .unwrap_or_default();
    let inicio = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    inicio
        .ancestors()
        .take_while(|a| !techos.iter().any(|t| t == a))
        .any(|a| a.join(".git").symlink_metadata().is_ok())
}

/// Nombre del repo principal (también desde un worktree). Si git sale con ≠ 0
/// hay dos casos: sin ningún `.git` en los ancestros del cwd es `Ok(None)`
/// (cwd fuera de un repo); con `.git` es un repo roto y `Err` (error del
/// engine, visible). `Err` también si git no se puede lanzar.
/// No se parsea stderr: con `--path-format=absolute` basta el código de salida.
fn clave_repo(cwd: &Path) -> Result<Option<String>> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .with_context(|| format!("invocar git en {}", cwd.display()))?;
    if !salida.status.success() {
        if hay_entrada_git(cwd) {
            anyhow::bail!("git falla en {} pese a haber un .git", cwd.display());
        }
        return Ok(None);
    }
    let comun = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    let comun = std::fs::canonicalize(&comun).with_context(|| format!("canonicalizar {comun}"))?;
    Ok(comun
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned()))
}

fn sin_comillas(s: &str) -> &str {
    let s = s.trim();
    for q in ['"', '\''] {
        if let Some(i) = s.strip_prefix(q).and_then(|r| r.strip_suffix(q)) {
            return i;
        }
    }
    s
}

/// Cómo casa una nota con la clave del repo: `Slug` > `Repos` > `Stem`. El slug
/// y `repos:` se declaran a propósito (`repos:` cubre las notas de varios
/// repos, que solo pueden llevar un slug); el stem también lo lleva la nota
/// hermana que nace al partir el canon (`foo — reglas android.md`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Casa {
    Stem,
    Repos,
    Slug,
}

fn casa(clave: &str, contenido: &str, fichero: &Path) -> Option<Casa> {
    let clave = clave.to_lowercase();
    if let Some(slug) = crate::frontmatter::valor(contenido, "slug")
        && sin_comillas(&slug).to_lowercase() == clave
    {
        return Some(Casa::Slug);
    }
    let en_repos = crate::frontmatter::lista(contenido, "repos")
        .iter()
        .any(|r| {
            // Las URLs de clonado traen `.git` o `/` final.
            let r = r.trim_end_matches('/');
            let r = r.strip_suffix(".git").unwrap_or(r);
            r.rsplit('/').next().unwrap_or(r).to_lowercase() == clave
        });
    if en_repos {
        return Some(Casa::Repos);
    }
    let stem = fichero
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let base = stem.split(" — ").next().unwrap_or(&stem);
    (base.to_lowercase() == clave).then_some(Casa::Stem)
}

/// Ruta relativa a la KB con `/` también en Windows: se une por componentes,
/// nunca con el separador del sistema.
fn une_con_barra(componentes: &[&str]) -> String {
    componentes.join("/")
}

/// Notas `projects/*.md` y `projects/*/*.md` (un solo nivel de subdirectorio),
/// con la ruta relativa a la KB y con `/`. Sin `projects/` legible es `Err`.
fn notas_de_proyecto(kb: &Path) -> Result<Vec<(String, PathBuf)>> {
    let dir = kb.join("projects");
    let lectura = std::fs::read_dir(&dir).with_context(|| format!("leer {}", dir.display()))?;
    let mut notas = Vec::new();
    let mut anade = |ruta: PathBuf, rel: String| {
        if ruta.is_file() && ruta.extension().is_some_and(|e| e == "md") {
            notas.push((rel, ruta));
        }
    };
    for entrada in lectura {
        let entrada = entrada.with_context(|| format!("leer {}", dir.display()))?;
        let ruta = entrada.path();
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if ruta.is_dir() {
            let sub =
                std::fs::read_dir(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
            for e in sub {
                let e = e.with_context(|| format!("leer {}", ruta.display()))?;
                let n = e.file_name().to_string_lossy().into_owned();
                anade(e.path(), une_con_barra(&["projects", &nombre, &n]));
            }
        } else {
            anade(ruta, une_con_barra(&["projects", &nombre]));
        }
    }
    Ok(notas)
}

/// Candidatas ordenadas de `notas_de_proyecto` que casan con `clave`. Solo
/// cuentan las del mejor rango presente: slug > `repos:` > stem.
fn candidatas(kb: &Path, clave: &str) -> Result<Vec<(String, PathBuf)>> {
    // KB mal apuntada ≠ repo sin nota: sin `projects/` legible todo daría
    // `sin_nota` en silencio.
    let mut halladas = Vec::new();
    for (rel, ruta) in notas_de_proyecto(kb)? {
        let bytes = std::fs::read(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
        if let Some(como) = casa(clave, &String::from_utf8_lossy(&bytes), &ruta) {
            halladas.push((como, rel, ruta));
        }
    }
    let mejor = halladas.iter().map(|(c, ..)| *c).max();
    let mut halladas: Vec<_> = halladas
        .into_iter()
        .filter(|(c, ..)| Some(*c) == mejor)
        .map(|(_, n, r)| (n, r))
        .collect();
    halladas.sort();
    Ok(halladas)
}

/// `None` si no hay encabezado; si no, (reglas, líneas ignoradas).
fn lee_seccion(contenido: &str) -> Option<(Vec<String>, Vec<String>)> {
    let mut lineas = contenido.lines();
    lineas.find(|l| *l == ENCABEZADO)?;
    let (mut reglas, mut ignoradas) = (Vec::new(), Vec::new());
    for l in lineas {
        if l.starts_with("## ") {
            break;
        }
        if let Some(texto) = l.strip_prefix("- ")
            && !texto.trim().is_empty()
        {
            reglas.push(texto.to_string());
        } else if !l.trim().is_empty() {
            ignoradas.push(l.to_string());
        }
    }
    Some((reglas, ignoradas))
}

/// Lo que `exo rules` entrega: el resultado del proyecto más las reglas
/// globales y cuántas reglas (de ambas listas) se recortaron al `CAP`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Salida {
    #[serde(flatten)]
    pub resultado: Resultado,
    pub global: Vec<String>,
    pub truncated: usize,
}

fn recorta(mut reglas: Vec<String>) -> (Vec<String>, usize) {
    let cortadas = reglas.len().saturating_sub(CAP);
    reglas.truncate(CAP);
    (reglas, cortadas)
}

/// Reglas de `core/reglas-duras.md` recortadas al `CAP` y cuántas se cortaron;
/// fichero o sección ausentes dan `(vec![], 0)`.
fn lee_global(kb: &Path) -> Result<(Vec<String>, usize)> {
    let ruta = kb.join("core/reglas-duras.md");
    let bytes = match std::fs::read(&ruta) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], 0)),
        Err(e) => return Err(e).with_context(|| format!("leer {}", ruta.display())),
    };
    Ok(lee_seccion(&String::from_utf8_lossy(&bytes))
        .map(|(reglas, _)| recorta(reglas))
        .unwrap_or_default())
}

pub fn resuelve(kb: &Path, cwd: &Path) -> Result<Salida> {
    let (global, cortadas_global) = lee_global(kb)?;
    let (resultado, cortadas) = resuelve_proyecto(kb, cwd)?;
    Ok(Salida {
        resultado,
        global,
        truncated: cortadas + cortadas_global,
    })
}

fn resuelve_proyecto(kb: &Path, cwd: &Path) -> Result<(Resultado, usize)> {
    let Some(repo) = clave_repo(cwd)? else {
        return Ok((skip(None, Razon::SinGit, vec![]), 0));
    };
    let cands = candidatas(kb, &repo)?;
    let (note, ruta) = match cands.as_slice() {
        [] => return Ok((skip(Some(repo), Razon::SinNota, vec![]), 0)),
        [uno] => uno.clone(),
        varias => {
            let nombres = varias.iter().map(|(n, _)| n.clone()).collect();
            return Ok((skip(Some(repo), Razon::Ambigua, nombres), 0));
        }
    };
    let bytes = std::fs::read(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
    let Some((rules, ignored_lines)) = lee_seccion(&String::from_utf8_lossy(&bytes)) else {
        return Ok((skip(Some(repo), Razon::SinSeccion, vec![]), 0));
    };
    if rules.is_empty() {
        return Ok((skip(Some(repo), Razon::SeccionVacia, vec![]), 0));
    }
    let (rules, cortadas) = recorta(rules);
    Ok((
        Resultado::Ok {
            repo,
            note,
            rules,
            ignored_lines,
        },
        cortadas,
    ))
}

/// Nota cuya sección `## Reglas duras` supera el `CAP`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Exceso {
    pub nota: String,
    pub reglas: usize,
}

/// Estado de la sección `## Reglas duras` de una nota frente al `CAP`.
/// `AlCap`: lo siguiente que entre desplaza a otra regla.
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Estado {
    Ok,
    AlCap,
    Excede,
    SinSeccion,
    SeccionVacia,
}

impl Estado {
    pub fn como_str(self) -> &'static str {
        match self {
            Estado::Ok => "ok",
            Estado::AlCap => "al_cap",
            Estado::Excede => "excede",
            Estado::SinSeccion => "sin_seccion",
            Estado::SeccionVacia => "seccion_vacia",
        }
    }
}

/// Cobertura de una nota: cuántas reglas duras declara y su estado.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Cobertura {
    pub nota: String,
    pub reglas: usize,
    pub estado: Estado,
}

/// Notas de `projects/*.md`, `projects/*/*.md` y `core/reglas-duras.md` con
/// su número de reglas duras y su estado, ordenadas por ruta. Informativo:
/// incluye las notas sin sección. Una KB sin `projects/` ni global da vacío.
/// No toca git.
pub fn cobertura(kb: &Path) -> Result<Vec<Cobertura>> {
    let mut notas = if kb.join("projects").is_dir() {
        notas_de_proyecto(kb)?
    } else {
        vec![]
    };
    let global = kb.join("core/reglas-duras.md");
    if global.is_file() {
        notas.push((une_con_barra(&["core", "reglas-duras.md"]), global));
    }
    let mut filas = Vec::new();
    for (nota, ruta) in notas {
        let bytes = std::fs::read(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
        let reglas = lee_seccion(&String::from_utf8_lossy(&bytes)).map(|(r, _)| r.len());
        let estado = match reglas {
            None => Estado::SinSeccion,
            Some(0) => Estado::SeccionVacia,
            Some(n) if n > CAP => Estado::Excede,
            Some(CAP) => Estado::AlCap,
            Some(_) => Estado::Ok,
        };
        filas.push(Cobertura {
            nota,
            reglas: reglas.unwrap_or(0),
            estado,
        });
    }
    filas.sort_by(|a, b| a.nota.cmp(&b.nota));
    Ok(filas)
}

/// Notas de `cobertura` con más de `CAP` reglas. Sin sección o vacía no es
/// exceso; una KB sin `projects/` ni global tampoco. No toca git.
pub fn check(kb: &Path) -> Result<Vec<Exceso>> {
    Ok(cobertura(kb)?
        .into_iter()
        .filter(|c| c.estado == Estado::Excede)
        .map(|c| Exceso {
            nota: c.nota,
            reglas: c.reglas,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_con_barra_nunca_usa_el_separador_del_sistema() {
        assert_eq!(une_con_barra(&["projects", "x", "y.md"]), "projects/x/y.md");
    }
}
