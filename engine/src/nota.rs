use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Nota parseada, lista para indexar. `permalink` es SIEMPRE el del
/// frontmatter — el indexer jamás lo regenera (§6.2 regla 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nota {
    pub permalink: String,
    pub titulo: String,
    pub tipo: Option<String>,
    pub cuerpo: String,
}

/// Frontmatter YAML, deserializado laxo: campos ausentes o de más no rompen
/// el parseo (notas reales llevan más claves que estas tres).
#[derive(Debug, Default, Deserialize)]
struct FrontmatterLaxo {
    permalink: Option<String>,
    title: Option<String>,
    #[serde(rename = "type")]
    tipo: Option<String>,
}

/// Parsea una nota `.md`: frontmatter YAML delimitado por `---` al inicio +
/// cuerpo. `Ok(None)` cuando la nota no tiene `permalink:` en el frontmatter
/// (frontmatter ausente, YAML ilegible, o campo ausente) — se salta, JAMÁS
/// se genera uno (§6.2 regla 1; generar-para-nuevas es del write-path M4).
/// El walk nunca aborta por esto: el llamador registra el warning y sigue.
pub fn parsea_nota(ruta: &Path) -> Result<Option<Nota>> {
    let contenido =
        std::fs::read_to_string(ruta).with_context(|| format!("leer {}", ruta.display()))?;

    let Some((yaml, cuerpo)) = separa_frontmatter(&contenido) else {
        return Ok(None);
    };

    // YAML ilegible ⇒ tratar como sin-permalink (skip), nunca abortar el walk.
    let Ok(fm) = yaml_serde::from_str::<FrontmatterLaxo>(&yaml) else {
        return Ok(None);
    };

    let Some(permalink) = fm.permalink else {
        return Ok(None);
    };

    let titulo = fm.title.unwrap_or_else(|| stem_de(ruta));

    Ok(Some(Nota {
        permalink,
        titulo,
        tipo: fm.tipo,
        cuerpo,
    }))
}

fn stem_de(ruta: &Path) -> String {
    ruta.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Divide `contenido` en (yaml_del_frontmatter, cuerpo) si empieza con una
/// línea `---` y existe una línea `---` de cierre más adelante. `None` si no
/// hay frontmatter delimitado (nota sin frontmatter en absoluto). Trabaja
/// línea a línea (no offsets de bytes) para no depender de LF vs CRLF.
fn separa_frontmatter(contenido: &str) -> Option<(String, String)> {
    let lineas: Vec<&str> = contenido.lines().collect();
    if lineas.first().map(|l| l.trim_end_matches('\r')) != Some("---") {
        return None;
    }
    let cierre = lineas[1..]
        .iter()
        .position(|l| l.trim_end_matches('\r') == "---")?
        + 1;

    let yaml = lineas[1..cierre].join("\n");
    let cuerpo = lineas[cierre + 1..].join("\n");
    Some((yaml, cuerpo))
}
