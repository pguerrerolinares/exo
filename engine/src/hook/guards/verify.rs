//! Guard verify.

use crate::hook::evento::{Decision, Entorno, Evento};
use crate::hook::journal::{Registro, lee};
use crate::hook::reflex::{payload_truncado, registra};
use regex::Regex;
use std::process::Command;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veredicto {
    Verde,
    Stale,
    Warn,
}

// Literales de bash-guards.sh (rama 3; retirado en plugin 1.32.0).
static VBD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"git[[:space:]]+commit([[:space:]]|$)").unwrap());
static CODIGO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(ts|tsx|js|jsx|py|go|rs|rb|java|kt|c|cc|cpp|h|hpp|cs|php|swift|scala|sh|bash)$")
        .unwrap()
});
static TR_RE1: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^|[&|;[:space:]]+)(pytest|jest|vitest|tox|rspec|phpunit)([[:space:]]|$)").unwrap()
});
static TR_RE2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(^|[&|;[:space:]]+)(npm[[:space:]]+(test|run[[:space:]]+test)|yarn[[:space:]]+test|pnpm[[:space:]]+test|bun[[:space:]]+test|cargo[[:space:]]+test|go[[:space:]]+test|make[[:space:]]+test|mvn[[:space:]]+test|gradle[[:space:]]+test|dotnet[[:space:]]+test)([[:space:]]|$)",
    )
    .unwrap()
});

fn es_test(r: &Registro) -> bool {
    r.clase == "Shell" && (TR_RE1.is_match(&r.cmd) || TR_RE2.is_match(&r.cmd))
}

fn es_edicion(r: &Registro) -> bool {
    r.clase == "FileEdit" || matches!(r.tool.as_str(), "Edit" | "Write" | "NotebookEdit")
}

/// Último test decide (spec §4.2); `ok` desconocido nunca es verde.
pub fn veredicto(regs: &[Registro]) -> Veredicto {
    let regs: Vec<&Registro> = regs.iter().filter(|r| r.kind != "warn").collect();
    let Some(i) = regs.iter().rposition(|r| es_test(r)) else {
        return Veredicto::Warn;
    };
    if regs[i].ok != Some(true) {
        Veredicto::Warn
    } else if regs[i + 1..].iter().any(|r| es_edicion(r)) {
        Veredicto::Stale
    } else {
        Veredicto::Verde
    }
}

fn hay_codigo_staged(cwd: &str) -> bool {
    let mut c = Command::new("git");
    if !cwd.is_empty() {
        c.arg("-C").arg(cwd);
    }
    c.args(["diff", "--cached", "--name-only", "--diff-filter=ACM"]);
    c.output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| CODIGO.is_match(l))
        })
}

/// Solo escribe en el reflex-log (spec §6.1): siempre `Allow`.
pub fn verify(ev: &Evento, en: &Entorno) -> Decision {
    let Some(cmd) = ev.tool.as_ref().and_then(|t| t.command.as_deref()) else {
        return Decision::Allow;
    };
    if !VBD.is_match(cmd) || cmd.contains("--no-verify") || !hay_codigo_staged(&ev.cwd) {
        return Decision::Allow;
    }
    let v = match lee(en, ev.harness, &ev.session_id, &ev.agent_id) {
        Ok(Some(l)) => veredicto(&l.registros),
        _ => Veredicto::Warn,
    };
    if v != Veredicto::Verde {
        let prefijo = if v == Veredicto::Stale { "stale " } else { "" };
        let payload = format!("{prefijo}{}", payload_truncado(cmd, &VBD));
        registra(en, "verify-before-done", ev, &payload);
    }
    Decision::Allow
}
