//! `exo rules`: reglas duras de proyecto desde la nota de la KB que casa con
//! el repo git del cwd. Solo lectura. Un skip no es un fallo: el consumidor
//! decide si lo muestra; solo un fallo del propio engine sale con exit ≠ 0.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const ENCABEZADO: &str = "## Reglas duras";
const CAP: usize = 10;

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Razon {
    SinGit,
    SinNota,
    Ambigua,
    SinSeccion,
    SeccionVacia,
    ExcedeCap,
}

impl Razon {
    pub fn como_str(self) -> &'static str {
        match self {
            Razon::SinGit => "sin_git",
            Razon::SinNota => "sin_nota",
            Razon::Ambigua => "ambigua",
            Razon::SinSeccion => "sin_seccion",
            Razon::SeccionVacia => "seccion_vacia",
            Razon::ExcedeCap => "excede_cap",
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

fn casa(clave: &str, contenido: &str, fichero: &Path) -> bool {
    let clave = clave.to_lowercase();
    if let Some(slug) = crate::frontmatter::valor(contenido, "slug")
        && sin_comillas(&slug).to_lowercase() == clave
    {
        return true;
    }
    let stem = fichero
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let base = stem.split(" — ").next().unwrap_or(&stem);
    base.to_lowercase() == clave
}

/// Candidatas ordenadas: `projects/*.md` de primer nivel que casan con `clave`.
fn candidatas(kb: &Path, clave: &str) -> Result<Vec<(String, PathBuf)>> {
    let dir = kb.join("projects");
    // KB mal apuntada ≠ repo sin nota: sin `projects/` legible todo daría
    // `sin_nota` en silencio.
    let lectura = std::fs::read_dir(&dir).with_context(|| format!("leer {}", dir.display()))?;
    let mut halladas = Vec::new();
    for entrada in lectura {
        let entrada = entrada.with_context(|| format!("leer {}", dir.display()))?;
        let ruta = entrada.path();
        if !ruta.is_file() || ruta.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let bytes = std::fs::read(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
        if casa(clave, &String::from_utf8_lossy(&bytes), &ruta) {
            let nombre = entrada.file_name().to_string_lossy().into_owned();
            halladas.push((format!("projects/{nombre}"), ruta));
        }
    }
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

pub fn resuelve(kb: &Path, cwd: &Path) -> Result<Resultado> {
    let Some(repo) = clave_repo(cwd)? else {
        return Ok(skip(None, Razon::SinGit, vec![]));
    };
    let cands = candidatas(kb, &repo)?;
    let (note, ruta) = match cands.as_slice() {
        [] => return Ok(skip(Some(repo), Razon::SinNota, vec![])),
        [uno] => uno.clone(),
        varias => {
            let nombres = varias.iter().map(|(n, _)| n.clone()).collect();
            return Ok(skip(Some(repo), Razon::Ambigua, nombres));
        }
    };
    let bytes = std::fs::read(&ruta).with_context(|| format!("leer {}", ruta.display()))?;
    let Some((rules, ignored_lines)) = lee_seccion(&String::from_utf8_lossy(&bytes)) else {
        return Ok(skip(Some(repo), Razon::SinSeccion, vec![]));
    };
    if rules.is_empty() {
        return Ok(skip(Some(repo), Razon::SeccionVacia, vec![]));
    }
    if rules.len() > CAP {
        return Ok(skip(Some(repo), Razon::ExcedeCap, vec![]));
    }
    Ok(Resultado::Ok {
        repo,
        note,
        rules,
        ignored_lines,
    })
}
