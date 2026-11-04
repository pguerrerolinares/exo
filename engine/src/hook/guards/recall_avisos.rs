//! Avisos de recall (superpowers duplicado, gate de /document pendiente) para
//! el systemMessage de SessionStart. Port de `exo-recall.sh:186-290`.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// El deadline global de los `git` del gate venció.
#[derive(Debug, PartialEq, Eq)]
pub struct GateTimeout;

/// Aviso si `superpowers@…` sigue habilitado. Precedencia de Claude Code, de
/// menos a más específico: usuario < proyecto < local; gana el último fichero
/// que menciona alguna clave `superpowers@…`, y dentro de él la primera `true`.
/// Falsos positivos aceptados: clave fuera de `enabledPlugins` o en comentario.
pub fn superpowers(home: &Path, proyecto: &Path) -> Option<String> {
    let re = Regex::new(r#""(superpowers@[^"]*)"\s*:\s*(true|false)"#).expect("regex fija");
    let ficheros = [
        home.join(".claude/settings.json"),
        proyecto.join(".claude/settings.json"),
        proyecto.join(".claude/settings.local.json"),
    ];
    let mut clave: Option<String> = None;
    for f in ficheros {
        let Ok(bytes) = std::fs::read(&f) else {
            continue;
        };
        let texto = String::from_utf8_lossy(&bytes);
        let mut hallada = false;
        let mut activa = None;
        for c in re.captures_iter(&texto) {
            hallada = true;
            if &c[2] == "true" && activa.is_none() {
                activa = Some(c[1].to_string());
            }
        }
        if hallada {
            clave = activa;
        }
    }
    clave.map(|k| {
        format!(
            "AVISO exo: {k} sigue habilitado junto a exo (skills y hooks duplicados). Desactívalo: claude plugin disable {k}"
        )
    })
}

/// Corre `cmd` hasta `hasta`; al vencer lo mata y lo recoge (sin zombi) y
/// devuelve `None`. `None` también si no arranca. Solo captura stdout.
pub fn corre_con_deadline(mut cmd: Command, hasta: Instant) -> Option<Output> {
    let mut hijo = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut salida = hijo.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = salida.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    loop {
        match hijo.try_wait() {
            Ok(Some(status)) => {
                // Un nieto puede sujetar el pipe tras salir el hijo: la espera
                // por el stdout también respeta el deadline.
                let stdout = rx.recv_timeout(hasta.saturating_duration_since(Instant::now()));
                let stdout = stdout.ok()?;
                return Some(Output {
                    status,
                    stdout,
                    stderr: Vec::new(),
                });
            }
            Ok(None) if Instant::now() < hasta => std::thread::sleep(Duration::from_millis(5)),
            // Vencido (o try_wait roto): el lector no se une, porque un nieto
            // puede seguir sujetando el pipe; muere solo cuando éste cierre.
            _ => {
                let _ = hijo.kill();
                let _ = hijo.wait();
                return None;
            }
        }
    }
}

/// Avisos de gate de /document pendiente. `/document` deja un trailer
/// `Gate: <ruta> bytes=… limit=… routed=bitacora`; una ruta está pendiente si
/// su Gate más reciente (en `distill/last..HEAD`; sin tag, últimos 7 días) es
/// posterior al auto-distill más reciente que la toca (`--all`) o que la
/// nombra en una línea `Nota: <ruta>`. Resuelta a mano si la nota existe y hoy
/// pesa menos del 85 % del `limit=`. Los 3 `git` comparten un único `deadline`.
/// Cualquier otro fallo de git ⇒ sin aviso; KB sin `.git` ⇒ nada.
pub fn gate_pendiente(
    kb: &Path,
    deadline: Duration,
    git: &OsStr,
) -> Result<Vec<String>, GateTimeout> {
    if !kb.join(".git").exists() {
        return Ok(vec![]);
    }
    let hasta = Instant::now() + deadline;
    let corre = |args: &[&str]| -> Result<Option<String>, GateTimeout> {
        let mut c = Command::new(git);
        c.arg("-C").arg(kb).args(args);
        let out = corre_con_deadline(c, hasta).ok_or(GateTimeout)?;
        Ok(out
            .status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned()))
    };

    let con_tag = corre(&["rev-parse", "-q", "--verify", "refs/tags/distill/last"])?.is_some();
    let rango = if con_tag {
        "refs/tags/distill/last..HEAD"
    } else {
        "--since=7.days"
    };
    // Del más nuevo al más viejo: la primera aparición de cada ruta es la más reciente.
    let Some(gates) = corre(&["log", rango, "--format=@@%ct%n%B"])? else {
        return Ok(vec![]);
    };
    if gates.is_empty() {
        return Ok(vec![]);
    }
    // quotePath=false: sin él git escapa el em dash de las rutas en --name-only.
    let Some(dist) = corre(&[
        "-c",
        "core.quotePath=false",
        "log",
        "--all",
        "--fixed-strings",
        "--grep=docs(kb): auto-distill",
        "--format=@@%ct%n%B%n@@F",
        "--name-only",
    ])?
    else {
        return Ok(vec![]);
    };

    let resueltas = resueltas_por_autodistill(&dist);
    let re_gate = Regex::new(r"^Gate: (.*) bytes=[0-9]+ limit=([0-9]+) routed=bitacora$")
        .expect("regex fija");
    let mut vistas = HashSet::new();
    let mut avisos = Vec::new();
    let mut ct = 0i64;
    for linea in gates.lines() {
        if let Some(resto) = linea.strip_prefix("@@") {
            ct = resto.parse().unwrap_or(0);
            continue;
        }
        let Some(c) = re_gate.captures(linea) else {
            continue;
        };
        let ruta = c[1].to_string();
        if !vistas.insert(ruta.clone()) {
            continue;
        }
        if resueltas.get(&ruta).is_some_and(|&d| ct <= d) {
            continue;
        }
        let limite: u64 = c[2].parse().unwrap_or(0);
        let nota = kb.join(&ruta);
        if nota.is_file()
            && let Ok(m) = std::fs::metadata(&nota)
            && m.len() * 100 < limite * 85
        {
            continue;
        }
        avisos.push(format!(
            "AVISO exo: gate pendiente en {ruta}, sin auto-distill que lo resuelva (corre /distill)"
        ));
    }
    Ok(avisos)
}

/// ruta → `%ct` del auto-distill más reciente que la toca (nombre en
/// `--name-only`) o la nombra (`Nota: <ruta>` en el cuerpo). Entrada de
/// más nuevo a más viejo: gana la primera aparición.
fn resueltas_por_autodistill(dist: &str) -> HashMap<String, i64> {
    let mut d = HashMap::new();
    let mut ct = 0i64;
    let mut cuerpo = false;
    for l in dist.lines() {
        if l == "@@F" {
            cuerpo = false;
        } else if let Some(t) = l
            .strip_prefix("@@")
            .filter(|t| t.starts_with(|c: char| c.is_ascii_digit()))
        {
            ct = t.parse().unwrap_or(0);
            cuerpo = true;
        } else if cuerpo {
            if let Some(ruta) = l.strip_prefix("Nota: ") {
                d.entry(ruta.to_string()).or_insert(ct);
            }
        } else if !l.is_empty() {
            d.entry(l.to_string()).or_insert(ct);
        }
    }
    d
}
