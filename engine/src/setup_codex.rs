//! `exo setup codex`: registra los roles `exo-consultor` y `exo-executor` en
//! el `config.toml` de Codex (spec §4.9). Se edita el TEXTO, no el árbol TOML:
//! reserializar reordenaría y reformatearía lo que es del usuario, y Codex
//! también reescribe ese fichero. `toml` solo valida.
use anyhow::{Context, Result, bail};
use std::path::Path;

pub const ROL_CONSULTOR: &str = include_str!("../../plugins/exo/agents/codex/consultor.toml");
pub const ROL_EXECUTOR: &str = include_str!("../../plugins/exo/agents/codex/executor.toml");

const ABRE: &str = "# >>> exo";
const CIERRA: &str = "# <<< exo";

const ROLES: [(&str, &str, &str); 2] = [
    (
        "exo-consultor",
        "consultor.toml",
        "Consultor de solo lectura para decisiones de diseño y review adversarial de premisas: recomendación, condición de invalidación medible y evidencia etiquetada.",
    ),
    (
        "exo-executor",
        "executor.toml",
        "Ejecutor de tareas de implementación acotadas bajo doctrina de buena ingeniería: verifica antes de declarar hecho y commitea limpio.",
    ),
];

fn cadena(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

fn bloque(agentes_dir: &Path) -> String {
    let mut b = format!("{ABRE}\n# Lo gestiona `exo setup codex`: edítalo ahí, no aquí.\n");
    for (nombre, fichero, descripcion) in ROLES {
        b.push_str(&format!(
            "[agents.{nombre}]\ndescription = {}\nconfig_file = {}\n",
            cadena(descripcion),
            cadena(&agentes_dir.join(fichero).to_string_lossy()),
        ));
    }
    b.push_str(CIERRA);
    b.push('\n');
    b
}

/// Rango en bytes `[inicio, fin)` del bloque, con su salto de línea final.
fn localiza(config: &str) -> Result<Option<(usize, usize)>> {
    let (mut abre, mut cierra) = (Vec::new(), Vec::new());
    let mut pos = 0;
    for linea in config.split_inclusive('\n') {
        match linea.trim() {
            ABRE => abre.push((pos, pos + linea.len())),
            CIERRA => cierra.push((pos, pos + linea.len())),
            _ => {}
        }
        pos += linea.len();
    }
    match (abre.as_slice(), cierra.as_slice()) {
        ([], []) => Ok(None),
        ([(ini, _)], [(c, fin)]) if ini < c => Ok(Some((*ini, *fin))),
        ([], _) => bail!("marcador `{CIERRA}` sin su `{ABRE}`"),
        (_, []) => bail!("marcador `{ABRE}` sin su `{CIERRA}`"),
        _ => bail!("los marcadores `{ABRE}` / `{CIERRA}` no forman un único bloque en orden"),
    }
}

/// Primera clave (ruta con puntos) cuyo valor difiere entre `a` y `b`.
fn difiere(a: &toml::Table, b: &toml::Table, prefijo: &str) -> Option<String> {
    for k in a.keys().chain(b.keys()) {
        let ruta = format!("{prefijo}{k}");
        match (a.get(k), b.get(k)) {
            (Some(toml::Value::Table(x)), Some(toml::Value::Table(y))) => {
                if let Some(r) = difiere(x, y, &format!("{ruta}.")) {
                    return Some(r);
                }
            }
            (x, y) if x != y => return Some(ruta),
            _ => {}
        }
    }
    None
}

/// Texto de entrada → texto de salida, sin tocar disco. Lo de fuera de los
/// marcadores sale byte a byte igual.
pub fn aplica(config: &str, agentes_dir: &Path) -> Result<String> {
    if !agentes_dir.is_absolute() {
        bail!(
            "el directorio de roles no es absoluto: {}",
            agentes_dir.display()
        );
    }
    config
        .parse::<toml::Table>()
        .context("config.toml de Codex no parsea; no toco nada")?;
    let existente = localiza(config)?;
    let (antes, despues) = match existente {
        Some((ini, fin)) => (&config[..ini], &config[fin..]),
        None => (config, ""),
    };
    let fuera = format!("{antes}{despues}");
    let tabla_fuera = fuera
        .parse::<toml::Table>()
        .context("config.toml de Codex no parsea sin el bloque de exo")?;
    for (nombre, _, _) in ROLES {
        let existe = tabla_fuera
            .get("agents")
            .and_then(|a| a.get(nombre))
            .is_some();
        if existe {
            bail!(
                "[agents.{nombre}] está definido fuera de `{ABRE}`/`{CIERRA}`: bórralo o muévelo al bloque"
            );
        }
    }

    let mut salida = String::from(antes);
    if existente.is_none() && !antes.is_empty() {
        if !salida.ends_with('\n') {
            salida.push('\n');
        }
        salida.push('\n');
    }
    salida.push_str(&bloque(agentes_dir));
    salida.push_str(despues);

    let mut resultado = salida
        .parse::<toml::Table>()
        .context("el bloque de exo no encaja en config.toml")?;
    if let Some(toml::Value::Table(a)) = resultado.get_mut("agents") {
        for (nombre, _, _) in ROLES {
            a.remove(nombre);
        }
        if a.is_empty() && !tabla_fuera.contains_key("agents") {
            resultado.remove("agents");
        }
    }
    if let Some(k) = difiere(&tabla_fuera, &resultado, "") {
        bail!(
            "el bloque alteraría `{k}`, que no es de exo (¿contenido sin cabecera tras `{CIERRA}`? ponle una tabla)"
        );
    }
    Ok(salida)
}

/// tmp + rename en el mismo directorio: Codex también reescribe `config.toml`
/// de forma atómica y no debe ver un fichero a medias.
fn escribe_atomico(destino: &Path, contenido: &str) -> Result<()> {
    let padre = destino.parent().context("ruta sin directorio padre")?;
    std::fs::create_dir_all(padre).with_context(|| format!("creando {}", padre.display()))?;
    let nombre = destino
        .file_name()
        .context("ruta sin nombre")?
        .to_string_lossy();
    let tmp = padre.join(format!(".{nombre}.exo-tmp-{}", std::process::id()));
    let paso = || -> std::io::Result<()> {
        std::fs::write(&tmp, contenido)?;
        if let Ok(m) = std::fs::metadata(destino) {
            std::fs::set_permissions(&tmp, m.permissions())?;
        }
        std::fs::rename(&tmp, destino)
    };
    paso().map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        anyhow::Error::new(e).context(format!("escribiendo {}", destino.display()))
    })
}

/// Escribe los roles en `<exo_root>/codex-agents/` y el bloque en
/// `<codex_home>/config.toml`. Nada se escribe si `aplica` falla.
pub fn ejecuta(codex_home: &Path, exo_root: &Path) -> Result<()> {
    let agentes = exo_root.join("codex-agents");
    let config = codex_home.join("config.toml");
    let actual = match std::fs::read_to_string(&config) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("leyendo {}", config.display())),
    };
    let nuevo = aplica(&actual, &agentes).with_context(|| config.display().to_string())?;
    escribe_atomico(&agentes.join("consultor.toml"), ROL_CONSULTOR)?;
    escribe_atomico(&agentes.join("executor.toml"), ROL_EXECUTOR)?;
    escribe_atomico(&config, &nuevo)
}
