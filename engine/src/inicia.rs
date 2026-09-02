//! `exo init`: creación de la config propia y migración de una sola vez desde
//! basic-memory.
//!
//! Es la ÚNICA lectura de `~/.basic-memory/config.json` que sobrevive en el
//! engine, y es explícita y borrable: una migración se puede eliminar en tres
//! meses, un fallback permanente no lo quita nadie nunca.

use crate::config::Embeddings;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Lee el JSON de basic-memory y devuelve `(ruta_kb, nombre_kb, embeddings)`.
///
/// El nombre sale de `default_project`, **no** de un literal `"kb-demo"`:
/// ese literal (`lib.rs:93` en la versión anterior) era el acoplamiento que
/// esta ola viene a matar; reintroducirlo aquí sería cambiar de sitio el bug.
pub fn desde_basic_memory(json: &str) -> Result<(PathBuf, String, Embeddings)> {
    let v: serde_json::Value =
        serde_json::from_str(json).context("la config de basic-memory no es JSON válido")?;
    let nombre = v
        .get("default_project")
        .and_then(|x| x.as_str())
        .context("default_project ausente en la config de basic-memory")?
        .to_string();
    let path = v
        .get("projects")
        .and_then(|p| p.get(&nombre))
        .and_then(|p| p.get("path"))
        .and_then(|p| p.as_str())
        .with_context(|| format!("projects.{nombre}.path ausente en la config de basic-memory"))?;
    let emb = Embeddings {
        model: v["semantic_embedding_model"]
            .as_str()
            .context("semantic_embedding_model ausente")?
            .to_string(),
        dims: v["semantic_embedding_dimensions"]
            .as_u64()
            .context("semantic_embedding_dimensions ausente")? as usize,
        min_similarity: v["semantic_min_similarity"]
            .as_f64()
            .context("semantic_min_similarity ausente")?,
    };
    Ok((PathBuf::from(path), nombre, emb))
}

/// Valida que `nombre` es apto como `[kb] name` — no prosa libre, es el
/// prefijo de permalink de TODAS las notas de la KB. `plantilla::render`
/// vuelca ese nombre con un `String::replace` crudo dentro de
/// `permalink: "{{KB_NAME}}/..."` en el frontmatter YAML de las once notas
/// de la semilla, sin escapar nada: una comilla, un espacio o un salto de
/// línea rompe ese YAML, y la KB nace con `indexed: 0` — `exo init` ya salió
/// con exit 0 antes de que nadie lo note. `cadena_toml` (arriba) protege el
/// mismo nombre del lado TOML; esta función protege el lado plantilla, que
/// no tenía guardia.
///
/// Se rechaza aquí, en la frontera de `exo init`, en vez de escapar YAML en
/// `plantilla::render`: el placeholder aparece en frontmatter, en títulos y
/// en prosa dentro de la plantilla, y cada sitio querría un escapado
/// distinto. Más simple: el nombre nunca llega a ser ambiguo.
///
/// Whitelist, no blacklist — una blacklist de "caracteres peligrosos" siempre
/// se deja alguno fuera (¿y `:`? ¿y los caracteres de control?). Se admite
/// ASCII alfanumérico + `-_.`: alcanza para nombres reales (`kb-demo`,
/// `mi-kb.v2`) y no dice nada sobre cómo se escapa en TOML, en YAML o en una
/// ruta — no hace falta, porque nunca lleva un carácter que necesite escape.
pub fn valida_nombre(nombre: &str) -> Result<()> {
    if nombre.is_empty() {
        anyhow::bail!(
            "el nombre de la KB está vacío — [kb] name es el prefijo de permalink de todas las notas, no puede quedar en blanco"
        );
    }
    if let Some(c) = nombre
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')))
    {
        anyhow::bail!(
            "nombre de KB inválido: {nombre:?} contiene el carácter {c:?}, no permitido — [kb] name es el prefijo de permalink de todas las notas de la KB (permalink: \"NOMBRE/...\" en cada frontmatter), solo se admite ASCII alfanumérico y - _ ."
        );
    }
    Ok(())
}

/// Renderiza un valor de cadena como literal TOML **escapado**. La estructura
/// del fichero se sigue escribiendo a mano —los comentarios son la mitad del
/// valor de un TOML editable, y el serializador los pierde—, pero los valores
/// no: una comilla en un nombre o en una ruta (legal en Linux y macOS)
/// produciría un fichero que no se puede releer, y el fallo aparecería lejos
/// de aquí.
fn cadena_toml(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

/// Comprueba que `destino` admite escribir la config: inexistente, o
/// `force`. Pensada para llamarse ANTES de tocar el disco de la KB (I4,
/// review de rama): `init_cmd` volcaba la plantilla de 12 ficheros y hacía
/// `git init` + commit ANTES de que `escribe_config` (más abajo) descubriera
/// que ya había una config y abortara sin `--force` — residuo en disco tras
/// un exit 1, y el reintento fallaba ya por otra vía (`prepara_kb`, "no está
/// vacía"). `escribe_config` sigue haciendo la misma comprobación al escribir
/// de verdad (defensa en profundidad, es barata), pero quien orquesta varios
/// pasos debe llamar a esta función primero.
pub fn valida_config_escribible(destino: &Path, force: bool) -> Result<()> {
    if destino.exists() && !force {
        anyhow::bail!(
            "ya existe una config en {} — repite con --force si de verdad quieres pisarla",
            destino.display()
        );
    }
    Ok(())
}

/// Escribe `config.toml`. Se niega si el destino existe y no hay `--force`:
/// pisar la config de alguien sin avisar es exactamente el tipo de efecto
/// silencioso que este proyecto persigue.
pub fn escribe_config(
    destino: &Path,
    kb: &Path,
    nombre: &str,
    emb: &Embeddings,
    db: &Path,
    force: bool,
) -> Result<()> {
    valida_config_escribible(destino, force)?;
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre).with_context(|| format!("crear {}", padre.display()))?;
    }
    // Se serializa a mano en vez de con `toml::to_string`: los comentarios son
    // la mitad del valor de un TOML editable a mano, y el serializador los
    // pierde. El runbook de W11 documenta que este fichero se edita a mano.
    let contenido = format!(
        r#"schema_version = 1

[kb]
# Raíz de la KB markdown. Barras normales funcionan también en Windows.
path = {}
# Prefijo de permalink. Explícito: NO se deriva del nombre del directorio.
name = {}

[index]
db = {}

[embeddings]
model = {}
dims = {}
min_similarity = {}
"#,
        cadena_toml(&kb.display().to_string().replace('\\', "/")),
        cadena_toml(nombre),
        cadena_toml(&db.display().to_string().replace('\\', "/")),
        cadena_toml(&emb.model),
        emb.dims,
        emb.min_similarity,
    );
    std::fs::write(destino, contenido)
        .with_context(|| format!("escribir {}", destino.display()))?;
    Ok(())
}

/// Ruta por defecto del JSON de basic-memory, para `--from-basic-memory`.
///
/// `EXO_BASIC_MEMORY_JSON` la overridea cuando está puesta (y no vacía) —
/// mismo patrón que `resuelve_db`/`resuelve_kb` en `main.rs`: sin este seam,
/// los tests de adopción leerían el `~/.basic-memory/config.json` real de la
/// máquina y dejarían de ser herméticos.
pub fn ruta_basic_memory() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("EXO_BASIC_MEMORY_JSON")
        && !v.is_empty()
    {
        return Ok(PathBuf::from(v));
    }
    Ok(dirs::home_dir()
        .context("sin HOME")?
        .join(".basic-memory/config.json"))
}

/// Comprueba que `kb` es un destino legítimo: inexistente, o existente y
/// vacío. Con `force`, cualquiera. Volcar sobre una KB con contenido pisaría
/// notas de alguien sin avisar — el efecto silencioso que este proyecto
/// persigue.
pub fn prepara_kb(kb: &Path, force: bool) -> Result<()> {
    if force || !kb.exists() {
        return Ok(());
    }
    let vacia = std::fs::read_dir(kb)
        .with_context(|| format!("leer {}", kb.display()))?
        .next()
        .is_none();
    if !vacia {
        anyhow::bail!(
            "{} existe y no está vacía — repite con --force si de verdad quieres volcar encima",
            kb.display()
        );
    }
    Ok(())
}

/// Exige que en modo CREACIÓN lo indexado cuadre con lo volcado (C2, review
/// de rama). Antes, `init_cmd` descartaba el `indexer::Resumen` de `indexa` —
/// una semilla que vuelca 12 ficheros e indexa CERO notas (frontmatter
/// ilegible, cualquier causa) salía exit 0 con mensaje de éxito. Solo aplica
/// en creación: en modo ADOPCIÓN la KB es del usuario, `init` no decide su
/// contenido, y una nota suya sin permalink no es un bug de `init`.
///
/// `escritos` son las rutas que devuelve `plantilla::vuelca` — incluye
/// ficheros no-`.md` (`archive/log/.gitkeep`), que `walk_kb`/`indexa` ni
/// siquiera ven. Se cuentan solo las `.md` en vez de hardcodear "11": si la
/// plantilla gana o pierde una nota, la cuenta esperada se mueve con ella en
/// vez de divergir en silencio.
pub fn verifica_indexado_completo(escritos: &[PathBuf], indexadas: usize) -> Result<()> {
    let esperadas = escritos
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .count();
    if indexadas != esperadas {
        anyhow::bail!(
            "la KB semilla volcó {esperadas} notas (.md) pero el índice solo indexó {indexadas} \
             — alguna se saltó en silencio (frontmatter ilegible o similar); revisa las notas de \
             la plantilla antes de confiar en esta KB"
        );
    }
    Ok(())
}
