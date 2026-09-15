//! Preflight de **entorno**: `lint` juzga la KB, `doctor` juzga la máquina.
//!
//! Contrato, y es el punto entero del comando: **cada check reporta el
//! artefacto que miró** —la ruta, la versión, los bytes— y **ninguno
//! desaparece del informe**. Lo que no aplica a esta plataforma sale como
//! `na` con lo que miró, porque una fila ausente no se distingue de un check
//! que nunca existió. Es la lección literal de los seis casos del runbook de
//! W11 (`docs/superpowers/runbooks/2026-08-24-integracion-equipo-trabajo-windows.md`):
//! «el código de salida no es evidencia; lo que valió fue mirar el artefacto
//! real».
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Estado de un check. `Na` NO es `Ok`: es «aquí esto no se mide».
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Estado {
    Ok,
    Warn,
    Fail,
    Na,
}

impl std::fmt::Display for Estado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Estado::Ok => "ok",
            Estado::Warn => "warn",
            Estado::Fail => "fail",
            Estado::Na => "na",
        })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Check {
    pub id: &'static str,
    #[serde(rename = "status")]
    pub estado: Estado,
    /// Lo que se miró: ruta absoluta, comando resuelto, versión, tamaño.
    #[serde(rename = "artifact")]
    pub artefacto: String,
    #[serde(rename = "detail")]
    pub detalle: String,
}

impl Check {
    fn nuevo(
        id: &'static str,
        estado: Estado,
        artefacto: impl Into<String>,
        detalle: impl Into<String>,
    ) -> Self {
        Self {
            id,
            estado,
            artefacto: artefacto.into(),
            detalle: detalle.into(),
        }
    }
}

#[derive(Serialize)]
pub struct InformeDoctor {
    pub ok: bool,
    #[serde(rename = "platform")]
    pub plataforma: &'static str,
    pub checks: Vec<Check>,
}

impl InformeDoctor {
    fn nuevo(checks: Vec<Check>) -> Self {
        // `warn` informa y NO gatea: una DB rancia o el modelo sin cachear son
        // deuda con arreglo conocido, no una máquina rota.
        let ok = !checks.iter().any(|c| c.estado == Estado::Fail);
        Self {
            ok,
            plataforma: std::env::consts::OS,
            checks,
        }
    }

    pub fn fallidos(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.estado == Estado::Fail)
            .count()
    }
}

/// El entorno que doctor juzga, inyectable entero: un test no puede mover el
/// HOME ni el PATH del proceso, y sin inyección estos checks solo se podrían
/// probar en la máquina del que los escribió.
pub struct Entorno {
    pub home: PathBuf,
    pub config: PathBuf,
    pub cache_hf: PathBuf,
    pub path: String,
    /// KB ya resuelta por la cadena `flag > $EXO_KB > config` de `main.rs`
    /// (`resuelve_kb`). `None` si esa cadena no resolvió nada —sin flag, sin
    /// env, sin config legible—: en ese caso los checks caen a lo que diga
    /// la `Config` que `analiza` carga por su cuenta, el mismo camino que
    /// existía antes de esta tarea. Doctor resolvía la KB SOLO desde la
    /// config e ignoraba `$EXO_KB`: con la env puesta, `index`/`search`
    /// trabajaban sobre una KB y `doctor` dictaminaba sobre otra (review
    /// final de rama, 2026-09-11).
    pub kb: Option<PathBuf>,
    /// DB ya resuelta por `flag > $EXO_DB > config` (`resuelve_db`). Mismo
    /// contrato que `kb`.
    pub db: Option<PathBuf>,
}

impl Entorno {
    /// El entorno real del proceso. La caché replica lo que hace `hf_hub`
    /// (`Cache::from_env`): `$HF_HOME/hub`, y si no `~/.cache/huggingface/hub`.
    ///
    /// `kb`/`db` salen en `None`: este constructor no conoce flags de CLI, así
    /// que quien lo invoque (`doctor_cmd`) los rellena aplicando la misma
    /// precedencia que el resto de verbos.
    pub fn del_proceso() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_hf = match std::env::var_os("HF_HOME") {
            Some(h) => PathBuf::from(h).join("hub"),
            None => home.join(".cache").join("huggingface").join("hub"),
        };
        Self {
            config: crate::config::ruta_config()
                .unwrap_or_else(|_| home.join(".exo").join("config.toml")),
            home,
            cache_hf,
            path: std::env::var("PATH").unwrap_or_default(),
            kb: None,
            db: None,
        }
    }
}

/// La KB que juzgan `kb_readable` y `kb_precommit_hook`: la ya resuelta en
/// `entorno.kb` (flag > $EXO_KB > config) si la hay, y si no, lo que diga la
/// `cfg` cargada por `analiza` — el mismo fallback que corría antes de esta
/// tarea cuando ni flag ni env aplican.
fn kb_efectiva(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Option<PathBuf> {
    entorno
        .kb
        .clone()
        .or_else(|| cfg.map(|c| crate::config::expande_tilde(&c.kb.path)))
}

/// La DB que juzga `index_db`. Mismo contrato que `kb_efectiva`.
fn db_efectiva(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Option<PathBuf> {
    entorno
        .db
        .clone()
        .or_else(|| cfg.map(|c| crate::config::expande_tilde(&c.index.db)))
}

pub fn analiza(entorno: &Entorno) -> InformeDoctor {
    // La config se carga UNA vez y se pasa a los checks que dependen de ella:
    // releerla por check daría informes internamente incoherentes si alguien
    // la edita a mitad de corrida.
    let cfg = crate::config::carga_desde(&entorno.config).ok();
    InformeDoctor::nuevo(vec![
        check_config(entorno),
        check_binario_en_path(entorno),
        check_fallback_del_hook(entorno),
        check_kb(entorno, cfg.as_ref()),
        check_indice(entorno, cfg.as_ref()),
        check_rutas_portables(entorno, cfg.as_ref()),
        check_modelo(entorno, cfg.as_ref()),
        check_jq(entorno),
        check_git_bash(entorno),
        check_detach(entorno),
        check_hook_precommit(entorno, cfg.as_ref()),
        check_plugin_compat(entorno),
    ])
}

/// Parsea `"X.Y.Z"` a una tupla comparable por orden natural. `None` si no
/// tiene esa forma exacta (tres componentes numéricos separados por punto) —
/// un directorio que no es una versión (basura, `.DS_Store`) se descarta en
/// vez de reventar el sort. Sin dependencia nueva: `semver` es una crate más
/// para lo mismo que tres `parse::<u32>()`.
pub fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let mut partes = s.trim().split('.');
    let mayor: u32 = partes.next()?.parse().ok()?;
    let menor: u32 = partes.next()?.parse().ok()?;
    let parche: u32 = partes.next()?.parse().ok()?;
    if partes.next().is_some() {
        return None;
    }
    Some((mayor, menor, parche))
}

/// El subdirectorio de versión MÁS ALTA bajo `base` (cada entrada es un
/// directorio `X.Y.Z`, el layout de `~/.claude/plugins/cache/exo/<familia>/`).
/// Compara semver real, no la cadena: `"1.10.0"` < `"1.9.0"` como texto,
/// pero es la versión MAYOR — el bug que tenía `script_del_plugin` antes de
/// esta campaña (Task 5 lo reutiliza para corregirlo). `None` si `base` no
/// existe o no contiene ningún directorio con nombre de versión válido.
pub fn version_dir_mas_alta(base: &Path) -> Option<(PathBuf, (u32, u32, u32))> {
    let entradas = std::fs::read_dir(base).ok()?;
    entradas
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let nombre = e.file_name().to_string_lossy().into_owned();
            parse_semver(&nombre).map(|v| (e.path(), v))
        })
        .max_by_key(|(_, v)| *v)
}

/// `ENGINE_MIN` es el fichero de una línea (`plugins/exo/ENGINE_MIN` en el
/// repo, copiado tal cual al instalar) donde el PLUGIN declara la versión
/// mínima de engine con la que fue probado. Este check compara ESE número
/// contra `env!("CARGO_PKG_VERSION")` — la versión de ESTE binario, fijada
/// en compilación — para detectar el caso que motiva la campaña H: un
/// plugin actualizado (que ya no lleva los alias españoles retirados en
/// 0.2.0, por ejemplo) corriendo contra un binario que se quedó atrás.
fn check_plugin_compat(entorno: &Entorno) -> Check {
    let base = entorno
        .home
        .join(".claude")
        .join("plugins")
        .join("cache")
        .join("exo")
        .join("exo");
    let Some((dir, version)) = version_dir_mas_alta(&base) else {
        return Check::nuevo(
            "plugin_compat",
            Estado::Warn,
            base.display().to_string(),
            "no encuentro el plugin exo instalado — sin plugin no hay hooks \
             que puedan degradar, pero tampoco recall automático",
        );
    };
    let ruta_min = dir.join("ENGINE_MIN");
    let declarado = std::fs::read_to_string(&ruta_min).unwrap_or_default();
    let declarado = declarado.trim();
    let Some(min) = parse_semver(declarado) else {
        return Check::nuevo(
            "plugin_compat",
            Estado::Warn,
            ruta_min.display().to_string(),
            "el plugin instalado no lleva un ENGINE_MIN legible (versión \
             anterior a esta campaña) — no puedo comparar",
        );
    };
    let (va, vb, vc) = version;
    let propia_str = env!("CARGO_PKG_VERSION");
    let propia = parse_semver(propia_str)
        .expect("CARGO_PKG_VERSION de este crate siempre es X.Y.Z (engine/Cargo.toml)");
    let artefacto = format!(
        "binario {propia_str} · plugin {va}.{vb}.{vc} exige >= {declarado} ({})",
        ruta_min.display()
    );
    if propia < min {
        Check::nuevo(
            "plugin_compat",
            Estado::Fail,
            artefacto,
            format!(
                "este binario ({propia_str}) es más viejo que lo que el \
                 plugin instalado declara necesitar ({declarado}) — los \
                 hooks pueden degradar con forma válida. Actualiza el \
                 binario a >= {declarado}"
            ),
        )
    } else {
        Check::nuevo(
            "plugin_compat",
            Estado::Ok,
            artefacto,
            "el binario cumple el ENGINE_MIN que declara el plugin instalado",
        )
    }
}

/// Primera coincidencia de `nombre` (o `nombre.exe` en Windows) en un PATH
/// dado. No usa la crate `which`: el PATH es inyectable a propósito, y una
/// dependencia nueva para quince líneas no se paga.
fn busca_en_path(path: &str, nombre: &str) -> Option<PathBuf> {
    let candidatos: Vec<String> = if cfg!(windows) {
        vec![format!("{nombre}.exe"), nombre.to_string()]
    } else {
        vec![nombre.to_string()]
    };
    // `split_paths("")` devuelve UN componente vacío, no cero, y `"".join(x)`
    // es una ruta relativa: sin este filtro, un PATH vacío hace que el check
    // busque en el **cwd del proceso**. Medido el 2026-09-10: plantando un
    // `exo.exe` en `engine/`, el test del PATH vacío pasa de verde a rojo — es
    // decir, estaba midiendo el directorio actual, no el PATH.
    for dir in std::env::split_paths(path).filter(|d| !d.as_os_str().is_empty()) {
        for c in &candidatos {
            let ruta = dir.join(c);
            if ruta.is_file() {
                return Some(ruta);
            }
        }
    }
    None
}

fn check_binario_en_path(entorno: &Entorno) -> Check {
    match busca_en_path(&entorno.path, "exo") {
        Some(ruta) => Check::nuevo(
            "binary_on_path",
            Estado::Ok,
            ruta.display().to_string(),
            "los hooks lo resuelven con `command -v exo`",
        ),
        None => Check::nuevo(
            "binary_on_path",
            Estado::Warn,
            format!("PATH={}", entorno.path),
            "`command -v exo` no lo encuentra; los hooks caerán al fallback \
             $HOME/.local/bin/exo",
        ),
    }
}

/// El fallback literal de `plugins/exo/scripts/kb-precommit.sh:18`
/// (`EXO="${EXO_BIN:-$HOME/.local/bin/exo}"`). Si ese fichero no está, la
/// línea 20 del hook sale **0**: commit permitido, gate apagado, sin romper
/// nada. Por eso esto es `fail` y no `warn`.
///
/// Se reporta QUÉ fichero existe: medido el 2026-09-10 en el Git Bash de W11,
/// msys resuelve `exo` → `exo.exe` en `stat()` y el test `-x` sobre la ruta
/// sin extensión da verdadero, al revés de lo que afirma la spec. Un check
/// que diera por buena cualquiera de las dos versiones estaría adivinando;
/// este mira.
/// `[ -x ruta ]` tal y como lo evalua el hook: existe, es fichero y —en unix—
/// lleva bit de ejecucion. En Windows no hay tal bit: ahi `-x` de msys mira la
/// extension, asi que `is_file()` es la equivalencia correcta.
///
/// El bit va en una función aparte por plataforma en vez de en un bloque
/// `#[cfg]` dentro de esta: el `return` que ese bloque exigía es
/// `clippy::needless_return` al compilarse en unix, y **el clippy de una
/// máquina Windows no lo ve**, porque no compila ese `cfg`. Lo cazó el CI en
/// ubuntu (run 34581824820) — mismo punto ciego que impide correr aquí los
/// tests `#[cfg(unix)]`.
fn es_ejecutable(ruta: &std::path::Path) -> bool {
    ruta.is_file() && tiene_bit_de_ejecucion(ruta)
}

#[cfg(unix)]
fn tiene_bit_de_ejecucion(ruta: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(ruta)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// En Windows no hay bit de ejecución: el `-x` de msys mira la extensión, así
/// que `is_file()` ya es la equivalencia completa.
#[cfg(not(unix))]
fn tiene_bit_de_ejecucion(_ruta: &std::path::Path) -> bool {
    true
}

fn check_fallback_del_hook(entorno: &Entorno) -> Check {
    let base = entorno.home.join(".local").join("bin");
    let literal = base.join("exo");
    let con_exe = base.join("exo.exe");
    // `is_file()` no es lo que mira el hook: su linea 20 evalua `[ -x ]`. En
    // unix un fichero sin bit de ejecucion daba `ok` aqui mientras el gate
    // estaba apagado — justo el escenario para el que existe este check.
    // `install.sh` hace `chmod 0755` precisamente porque ese bit se pierde.
    let existe_literal = es_ejecutable(&literal);
    let existe_exe = es_ejecutable(&con_exe);
    let artefacto = format!(
        "{} (existe={}) · {} (existe={})",
        literal.display(),
        existe_literal,
        con_exe.display(),
        existe_exe
    );
    if existe_literal || existe_exe {
        Check::nuevo(
            "hook_fallback_binary",
            Estado::Ok,
            artefacto,
            "kb-precommit.sh encuentra el binario y el gate de la KB muerde",
        )
    } else {
        Check::nuevo(
            "hook_fallback_binary",
            Estado::Fail,
            artefacto,
            "kb-precommit.sh:20 sale 0 sin gate — COMMIT PERMITIDO en silencio. \
             Instala con install.sh/install.ps1 o copia el binario ahí",
        )
    }
}

fn check_kb(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    let Some(kb) = kb_efectiva(entorno, cfg) else {
        return Check::nuevo(
            "kb_readable",
            Estado::Fail,
            "(sin config)",
            "no hay config legible, así que no se sabe qué KB mirar",
        );
    };
    let artefacto = kb.display().to_string();
    if !kb.is_dir() {
        return Check::nuevo(
            "kb_readable",
            Estado::Fail,
            artefacto,
            "la raíz de la KB no existe o no es un directorio",
        );
    }
    match crate::walker::walk_kb(&kb) {
        Ok(notas) => Check::nuevo(
            "kb_readable",
            Estado::Ok,
            artefacto,
            format!("{} nota(s) .md bajo la raíz", notas.len()),
        ),
        Err(e) => Check::nuevo("kb_readable", Estado::Fail, artefacto, format!("{e:#}")),
    }
}

fn check_indice(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    let Some(db) = db_efectiva(entorno, cfg) else {
        return Check::nuevo(
            "index_db",
            Estado::Fail,
            "(sin config)",
            "no hay config legible, así que no se sabe qué DB mirar",
        );
    };
    if !db.is_file() {
        return Check::nuevo(
            "index_db",
            Estado::Warn,
            db.display().to_string(),
            "no hay índice todavía — córrelo con `exo index`",
        );
    }
    let bytes = std::fs::metadata(&db).map(|m| m.len()).unwrap_or(0);
    let artefacto = format!("{} ({bytes} bytes)", db.display());
    let notas: i64 = match crate::abre_db(&db)
        .and_then(|c| Ok(c.query_row("SELECT count(*) FROM notas", [], |r| r.get(0))?))
    {
        Ok(n) => n,
        Err(e) => return Check::nuevo("index_db", Estado::Fail, artefacto, format!("{e:#}")),
    };
    // «No rancia»: la DB es al menos tan nueva como la nota más reciente.
    // Heurística de mtime, la misma que usa el indexer incremental; no
    // pretende detectar un borrado, sino el caso medido en W11 —el hook de
    // reindexado muerto durante meses sin un solo rastro—.
    // Sin KB resuelta (ni entorno ni config) no se puede afirmar ranciedad:
    // se salta la comprobación en vez de adivinar sobre una ruta que no hay.
    if let Some(kb) = kb_efectiva(entorno, cfg)
        && let Some(nota) = mtime_mas_reciente(&kb)
        && let Ok(indice) = std::fs::metadata(&db).and_then(|m| m.modified())
        && nota > indice
    {
        return Check::nuevo(
            "index_db",
            Estado::Warn,
            artefacto,
            format!(
                "{notas} nota(s) indexadas, pero la KB tiene cambios más nuevos \
                 que el índice — corre `exo index`"
            ),
        );
    }
    if notas == 0 {
        Check::nuevo(
            "index_db",
            Estado::Warn,
            artefacto,
            "el índice existe pero está vacío — corre `exo index`",
        )
    } else {
        Check::nuevo(
            "index_db",
            Estado::Ok,
            artefacto,
            format!("{notas} nota(s) indexadas"),
        )
    }
}

/// Núcleo testeable del check: toma la DB ya resuelta, para que un test no
/// tenga que montar un `Entorno` entero.
pub fn check_rutas_portables_de(db: &Path) -> Check {
    if !db.is_file() {
        return Check::nuevo(
            "index_paths_portable",
            Estado::Na,
            db.display().to_string(),
            "no hay índice todavía",
        );
    }
    let nativas: i64 = match crate::abre_db(db).and_then(|c| {
        Ok(c.query_row(
            r"SELECT count(*) FROM notas WHERE ruta LIKE '%\%'",
            [],
            |r| r.get(0),
        )?)
    }) {
        Ok(n) => n,
        Err(e) => {
            return Check::nuevo(
                "index_paths_portable",
                Estado::Fail,
                db.display().to_string(),
                format!("{e:#}"),
            );
        }
    };
    if nativas > 0 {
        Check::nuevo(
            "index_paths_portable",
            Estado::Warn,
            db.display().to_string(),
            format!(
                "{nativas} ruta(s) con separador nativo: este índice se escribió \
                 con una versión anterior y sirve rutas que no se pueden pegar \
                 en un comando — corre `exo index`"
            ),
        )
    } else {
        Check::nuevo(
            "index_paths_portable",
            Estado::Ok,
            db.display().to_string(),
            "todas las rutas del índice usan `/`",
        )
    }
}

fn check_rutas_portables(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    match db_efectiva(entorno, cfg) {
        Some(db) => check_rutas_portables_de(&db),
        None => Check::nuevo(
            "index_paths_portable",
            Estado::Na,
            "(sin config)",
            "no hay config legible, así que no se sabe qué DB mirar",
        ),
    }
}

/// El mtime de la nota más reciente de la KB. `None` si la KB no se puede
/// recorrer: la ranciedad no se puede afirmar, y afirmarla a ciegas sería
/// justo el tipo de veredicto sin artefacto que este comando evita.
fn mtime_mas_reciente(kb: &std::path::Path) -> Option<std::time::SystemTime> {
    let notas = crate::walker::walk_kb(kb).ok()?;
    notas
        .iter()
        .filter_map(|n| std::fs::metadata(n).ok())
        .filter_map(|m| m.modified().ok())
        .max()
}

/// El `model.onnx` bajo cualquier snapshot del modelo. `metadata` sigue el
/// symlink —en la caché de HF los ficheros del snapshot apuntan a `blobs/`—,
/// así que la longitud es la real, no la del enlace.
fn onnx_en_cache(dir_modelo: &std::path::Path) -> Option<(PathBuf, u64)> {
    let entradas = std::fs::read_dir(dir_modelo.join("snapshots")).ok()?;
    for e in entradas.flatten() {
        let cand = e.path().join("onnx").join("model.onnx");
        if let Ok(m) = std::fs::metadata(&cand)
            && m.len() > 0
        {
            return Some((cand, m.len()));
        }
    }
    None
}

/// Presencia del modelo de embeddings en la caché de `hf_hub`. `warn`, no
/// `fail`: sin él la primera indexación baja 615 MB y tarda unos minutos —es
/// deuda de tiempo, no una máquina rota—, pero decirlo AQUÍ es la diferencia
/// entre un `exo index` que parece colgado y uno que se sabe descargando.
fn check_modelo(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    let modelo = cfg
        .map(|c| c.embeddings.model.as_str())
        .unwrap_or(crate::MODELO_JINA_ES);
    let dir = entorno
        .cache_hf
        .join(format!("models--{}", modelo.replace('/', "--")));
    match onnx_en_cache(&dir) {
        Some((ruta, bytes)) => Check::nuevo(
            "embeddings_model",
            Estado::Ok,
            ruta.display().to_string(),
            format!("{modelo} · {bytes} bytes en caché"),
        ),
        None => Check::nuevo(
            "embeddings_model",
            Estado::Warn,
            dir.display().to_string(),
            format!(
                "{modelo} no está en la caché: la primera indexación se baja \
                 ~615 MB (unos minutos en frío)"
            ),
        ),
    }
}

/// `jq` es requisito declarado del camino «desde release». Los hooks del
/// plugin lo usan para componer y leer el envelope
/// (`plugins/exo/scripts/recall-inject.sh`, `exo-recall.sh`), así que sin él
/// el bloque de recall no se inyecta: `fail`, no `warn`.
///
/// Dos trampas medidas, ambas del runbook de W11: el `jq` de la Store es un
/// **alias de ejecución** bajo `WindowsApps` que no es un jq, y el `jq`
/// nativo de winget emite **CRLF**, que solo muerde leyendo con `while read`
/// desde process substitution (`a1-gate.sh:201-202`, 22 checks caídos).
fn check_jq(entorno: &Entorno) -> Check {
    let Some(ruta) = busca_en_path(&entorno.path, "jq") else {
        return Check::nuevo(
            "jq",
            Estado::Fail,
            format!("PATH={}", entorno.path),
            "sin jq: recall-inject.sh y exo-recall.sh no pueden leer el envelope",
        );
    };
    let artefacto = ruta.display().to_string();
    if artefacto.contains("WindowsApps") {
        return Check::nuevo(
            "jq",
            Estado::Fail,
            artefacto,
            "es el alias de ejecución de WindowsApps, no un jq real — instala \
             uno de verdad (winget install jqlang.jq) y ponlo antes en el PATH",
        );
    }
    match std::process::Command::new(&ruta).arg("--version").output() {
        Ok(o) if o.stdout.contains(&b'\r') => Check::nuevo(
            "jq",
            Estado::Warn,
            artefacto,
            "jq nativo: emite CRLF. Muerde en `while read` desde process \
             substitution (a1-gate.sh:201-202); con $(...) bash come el \\r",
        ),
        Ok(o) => Check::nuevo(
            "jq",
            Estado::Ok,
            artefacto,
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
        ),
        Err(e) => Check::nuevo(
            "jq",
            Estado::Warn,
            artefacto,
            format!("está en el PATH pero no se pudo ejecutar: {e}"),
        ),
    }
}

/// Claude Code usa Git Bash como shell de hooks en Windows: sin él los
/// `.sh` del plugin no corren. Fuera de Windows sale `na` —no desaparece—
/// porque una fila ausente no se distingue de un check que nunca existió.
fn check_git_bash(entorno: &Entorno) -> Check {
    if !cfg!(windows) {
        return Check::nuevo(
            "git_bash",
            Estado::Na,
            format!("plataforma={}", std::env::consts::OS),
            "solo se mide en Windows: fuera de ahí el shell de los hooks ya es bash",
        );
    }
    match busca_en_path(&entorno.path, "bash") {
        Some(ruta) => Check::nuevo(
            "git_bash",
            Estado::Ok,
            ruta.display().to_string(),
            "Claude Code puede correr los hooks .sh del plugin",
        ),
        None => Check::nuevo(
            "git_bash",
            Estado::Fail,
            format!("PATH={}", entorno.path),
            "sin Git Bash los hooks .sh del plugin no corren",
        ),
    }
}

/// La vía de detach del reindexado (`plugins/exo/scripts/exo-index.sh`):
/// `setsid` donde lo haya, y en msys `cmd //c start //b`. Sin ninguna de las
/// dos el hook deja evento `index-fallback / reason=no-detach` — que es
/// justamente lo que se añadió después de meses de sesiones en Windows sin
/// refrescar el índice y sin un solo rastro.
fn check_detach(entorno: &Entorno) -> Check {
    if let Some(ruta) = busca_en_path(&entorno.path, "setsid") {
        return Check::nuevo(
            "detach",
            Estado::Ok,
            ruta.display().to_string(),
            "exo-index.sh reindexa en segundo plano vía setsid",
        );
    }
    if cfg!(windows)
        && let Some(ruta) = busca_en_path(&entorno.path, "cmd")
    {
        return Check::nuevo(
            "detach",
            Estado::Ok,
            ruta.display().to_string(),
            "sin setsid, exo-index.sh detacha vía `cmd //c start //b`",
        );
    }
    Check::nuevo(
        "detach",
        Estado::Warn,
        format!("PATH={}", entorno.path),
        "ni setsid ni cmd: exo-index.sh dejará evento \
         index-fallback / reason=no-detach y el índice no se refrescará solo",
    )
}

/// El shim `pre-commit` de la KB. Cinco estados con consecuencias distintas,
/// y por eso no se colapsan: **no instalado** es deuda (`warn`), **symlink
/// colgando** es el fallo de V6 —git lo ejecuta, no encuentra el destino y el
/// commit pasa— (`fail`), **shim que no resuelve a ningún script** es el
/// mismo fallo en máquinas con `core.symlinks=false` (`fail`), **shim que
/// solo resuelve al plugin `reflex` viejo** es deuda de migración (`warn`), y
/// **un `pre-commit` que no menciona `kb-precommit.sh`** es de otro dueño, no
/// el gate de la KB (`warn`): atribuirse un hook ajeno sería el mismo
/// veredicto-sin-artefacto que este comando existe para no dar.
fn check_hook_precommit(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    let Some(kb) = kb_efectiva(entorno, cfg) else {
        return Check::nuevo(
            "kb_precommit_hook",
            Estado::Fail,
            "(sin config)",
            "no hay config legible, así que no se sabe en qué KB mirar el hook",
        );
    };
    let git = kb.join(".git");
    if !git.exists() {
        return Check::nuevo(
            "kb_precommit_hook",
            Estado::Warn,
            git.display().to_string(),
            "la KB no está versionada: sin git no hay gate de pre-commit",
        );
    }
    let hook = git.join("hooks").join("pre-commit");
    if !hook.exists() {
        // `exists()` sigue el symlink: un shim colgante da false aquí, así que
        // se distingue antes de dar el veredicto.
        if let Ok(destino) = std::fs::read_link(&hook) {
            return Check::nuevo(
                "kb_precommit_hook",
                Estado::Fail,
                format!("{} -> {}", hook.display(), destino.display()),
                "el shim existe y apunta a un script que NO está: git lo \
                 ejecuta, falla al resolverlo y el commit pasa sin gate",
            );
        }
        return Check::nuevo(
            "kb_precommit_hook",
            Estado::Warn,
            hook.display().to_string(),
            "gate no instalado — instálalo con: \
             ln -sf <repo>/plugins/exo/scripts/kb-precommit.sh <kb>/.git/hooks/pre-commit",
        );
    }
    // El hook existe. Un symlink se juzga por su destino; un fichero regular
    // es un shim —el caso real en máquinas con `core.symlinks=false`— y hay
    // que resolver a dónde lleva. Decir `ok` aquí sin mirarlo sería el
    // veredicto-sin-artefacto que este comando existe para no dar.
    if let Ok(destino) = std::fs::read_link(&hook) {
        // El destino existe (el colgante ya salio arriba), pero eso no basta:
        // un symlink a CUALQUIER script daba `ok` y afirmaba que el gate
        // corre. La rama de shim si comprobaba el nombre; esta no. Mismo
        // estado de maquina, dos veredictos opuestos segun `core.symlinks`.
        let es_el_gate = destino
            .file_name()
            .map(|n| n == "kb-precommit.sh")
            .unwrap_or(false);
        let artefacto = format!("{} -> {}", hook.display(), destino.display());
        if !es_el_gate {
            return Check::nuevo(
                "kb_precommit_hook",
                Estado::Warn,
                artefacto,
                "hay un pre-commit instalado, pero apunta a otro script: no es \
                 el gate de la KB",
            );
        }
        let viejo = destino.components().any(|c| c.as_os_str() == "reflex");
        if viejo {
            return Check::nuevo(
                "kb_precommit_hook",
                Estado::Warn,
                artefacto,
                "el symlink apunta al kb-precommit.sh del plugin reflex \
                 (viejo) — migra a exo",
            );
        }
        return Check::nuevo(
            "kb_precommit_hook",
            Estado::Ok,
            artefacto,
            "el gate de presupuestos y trinquete corre en cada commit de la KB",
        );
    }
    let contenido = std::fs::read_to_string(&hook).unwrap_or_default();
    if !contenido.contains("kb-precommit.sh") {
        return Check::nuevo(
            "kb_precommit_hook",
            Estado::Warn,
            hook.display().to_string(),
            "hay un pre-commit instalado, pero no menciona kb-precommit.sh: no es el gate de la KB",
        );
    }
    match script_del_plugin(&entorno.home) {
        Some((script, true)) => Check::nuevo(
            "kb_precommit_hook",
            Estado::Ok,
            format!("{} -> {}", hook.display(), script.display()),
            "el shim resuelve al kb-precommit.sh del plugin exo",
        ),
        Some((script, false)) => Check::nuevo(
            "kb_precommit_hook",
            Estado::Warn,
            format!("{} -> {}", hook.display(), script.display()),
            "el shim solo encuentra el plugin reflex (viejo) — migra a exo",
        ),
        None => Check::nuevo(
            "kb_precommit_hook",
            Estado::Fail,
            hook.display().to_string(),
            "el shim está instalado pero no resuelve a ningún kb-precommit.sh: el gate de la KB no puede correr",
        ),
    }
}

/// ¿A qué `kb-precommit.sh` resolvería el shim? Replica el glob del shim real
/// instalado en la KB: el plugin `exo` primero y el `reflex` viejo como
/// fallback declarado del cutover. Devuelve la ruta y si viene de `exo`.
///
/// El shim se queda con la versión más alta por `sort -V`; aquí basta con que
/// **alguna** resuelva, así que se ordena lexicográficamente y se toma la
/// última. La diferencia importaría para decir QUÉ versión corre, no para
/// decir si el gate puede correr, que es lo que este check afirma.
///
/// Sin la crate `glob`: dos `read_dir` no pagan una dependencia.
fn script_del_plugin(home: &std::path::Path) -> Option<(PathBuf, bool)> {
    for (familia, es_exo) in [("exo", true), ("reflex", false)] {
        let base = home
            .join(".claude")
            .join("plugins")
            .join("cache")
            .join("exo")
            .join(familia);
        let Ok(entradas) = std::fs::read_dir(&base) else {
            continue;
        };
        let mut candidatos: Vec<PathBuf> = entradas
            .flatten()
            .map(|e| e.path().join("scripts").join("kb-precommit.sh"))
            .filter(|p| p.is_file())
            .collect();
        candidatos.sort();
        if let Some(ultimo) = candidatos.pop() {
            return Some((ultimo, es_exo));
        }
    }
    None
}

fn check_config(entorno: &Entorno) -> Check {
    let ruta = entorno.config.display().to_string();
    match crate::config::carga_desde(&entorno.config) {
        Ok(c) => Check::nuevo(
            "config",
            Estado::Ok,
            ruta,
            format!(
                "schema_version={} kb={} db={}",
                c.schema_version,
                c.kb.path.display(),
                c.index.db.display()
            ),
        ),
        Err(e) => Check::nuevo("config", Estado::Fail, ruta, format!("{e:#}")),
    }
}
