//! Guard git -C: espejo de la rama 1 (1a rewrite, 1b warn) de `bash-guards.sh` (retirado en plugin 1.32.0).

use crate::hook::evento::{Clase, Decision, Entorno, Evento};
use crate::hook::reflex::registra;
use regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

const ALLOWLIST: &str =
    "status|log|diff|show|rev-parse|describe|ls-files|blame|shortlog|reflog|grep";

// `.` y `$` sin (?s)/(?m): el .sh nunca reescribe multilínea, y aquí tampoco llega.
static REWRITE_BASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^cd[[:space:]]+([A-Za-z0-9._/~-]+)[[:space:]]*&&[[:space:]]*git[[:space:]]+(.+)$")
        .unwrap()
});
static REWRITE_PS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:[Cc]d|[Ss]et-[Ll]ocation)[[:space:]]+([A-Za-z0-9._/~-]+)[[:space:]]*&&[[:space:]]*git[[:space:]]+(.+)$",
    )
    .unwrap()
});
static REST_PERMITIDO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^(?:{ALLOWLIST})(?:[[:space:]]|$)")).unwrap());
// grep opera por línea: `[^&;|]` no cruza `\n` allí, y aquí se evalúa línea a línea.
static WARN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"cd[[:space:]]+[^&;|]+(?:&&|;)[[:space:]]*git(?:[[:space:]]|$)").unwrap()
});
static WARN_PS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:^|[^[:alnum:]_-])(?:set-location|sl|push-location|pushd)[[:space:]]+[^&;|]+(?:&&|;)[[:space:]]*git(?:[[:space:]]|$)",
    )
    .unwrap()
});

const META: &[char] = &['&', ';', '|', '<', '>', '`', '$', '(', ')', '*', '?', '['];

fn path_reescribible(path: &str, ps: bool, cdpath: bool) -> bool {
    if path.starts_with('-') {
        return false;
    }
    if path == ".." || path.starts_with("../") || path.ends_with("/..") || path.contains("/../") {
        return false;
    }
    match path.chars().next() {
        Some('/') => true,
        Some('~') => !ps,
        // Relativo: CDPATH cambia a qué directorio va `cd`.
        _ => !cdpath,
    }
}

fn reescribe(cmd: &str, ps: bool, cdpath: bool) -> Option<String> {
    if cmd.contains('\n') {
        return None;
    }
    let re = if ps { &*REWRITE_PS } else { &*REWRITE_BASH };
    let c = re.captures(cmd)?;
    let (path, rest) = (&c[1], &c[2]);
    if path_reescribible(path, ps, cdpath) && !rest.contains(META) && REST_PERMITIDO.is_match(rest)
    {
        Some(format!("git -C {path} {rest}"))
    } else {
        None
    }
}

fn avisa(cmd: &str, ps: bool) -> bool {
    cmd.split('\n').any(|l| WARN.is_match(l))
        || (ps && cmd.split('\n').any(|l| WARN_PS.is_match(l)))
}

pub fn git_c(ev: &Evento, en: &Entorno, cdpath: bool) -> Decision {
    if !ev.crudo.contains("git") {
        return Decision::Allow;
    }
    let Some(tool) = ev.tool.as_ref().filter(|t| t.clase == Clase::Shell) else {
        return Decision::Allow;
    };
    let Some(cmd) = tool.command.as_deref().filter(|c| !c.is_empty()) else {
        return Decision::Allow;
    };
    let ps = tool.nombre == "PowerShell";
    if let Some(nuevo) = reescribe(cmd, ps, cdpath) {
        registra(en, "git-c-rewrite", ev, &format!("{cmd} -> {nuevo}"));
        return Decision::Rewrite {
            razon: format!(
                "Reflejo git-c: reescrito a `{nuevo}` (equivalente; evita el cd encadenado)"
            ),
            input: json!({ "command": nuevo }),
        };
    }
    if avisa(cmd, ps) {
        registra(en, "git-c", ev, cmd);
    }
    Decision::Allow
}
