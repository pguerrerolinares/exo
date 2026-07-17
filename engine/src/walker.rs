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
