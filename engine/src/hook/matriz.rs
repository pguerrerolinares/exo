//! Matriz guard x harness: qué garantiza cada guard en cada harness.

use super::evento::{EventoKind, Harness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    GitC,
    ZeroResiduo,
    Verify,
    SearchFirst,
    Index,
    RecallContrato,
    RecallCompact,
    RecallSuperpowers,
    RecallGatePendiente,
    ReglasTestigo,
    ReglasInyecta,
    MarcaCompact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Soporte {
    Soportado,
    Degradado(&'static str),
    NoSoportado(&'static str),
}

pub const GUARDS: [Guard; 12] = [
    Guard::GitC,
    Guard::ZeroResiduo,
    Guard::Verify,
    Guard::SearchFirst,
    Guard::Index,
    Guard::RecallContrato,
    Guard::RecallCompact,
    Guard::RecallSuperpowers,
    Guard::RecallGatePendiente,
    Guard::ReglasTestigo,
    Guard::ReglasInyecta,
    Guard::MarcaCompact,
];

impl Guard {
    /// También es el valor de `EXO_HOOK_PANIC` y el payload de `hook-panic`.
    pub fn nombre(&self) -> &'static str {
        match self {
            Guard::GitC => "git-c",
            Guard::ZeroResiduo => "zero-residuo",
            Guard::Verify => "verify",
            Guard::SearchFirst => "search-first",
            Guard::Index => "index",
            Guard::RecallContrato => "recall-contrato",
            Guard::RecallCompact => "recall-compact",
            Guard::RecallSuperpowers => "recall-superpowers",
            Guard::RecallGatePendiente => "recall-gate-pendiente",
            Guard::ReglasTestigo => "reglas-testigo",
            Guard::ReglasInyecta => "reglas-inyecta",
            Guard::MarcaCompact => "marca-compact",
        }
    }

    pub fn evento(&self) -> EventoKind {
        match self {
            Guard::GitC | Guard::ZeroResiduo | Guard::Verify | Guard::SearchFirst => {
                EventoKind::PreToolUse
            }
            Guard::Index => EventoKind::Stop,
            Guard::RecallContrato
            | Guard::RecallCompact
            | Guard::RecallSuperpowers
            | Guard::RecallGatePendiente
            | Guard::ReglasTestigo
            | Guard::ReglasInyecta => EventoKind::SessionStart,
            Guard::MarcaCompact => EventoKind::PostCompact,
        }
    }

    /// Campos que el payload tiene que traer para que el guard actúe.
    pub fn requiere(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Guard::RecallCompact => &[("source", "compact")],
            _ => &[],
        }
    }

    /// Tools que el guard necesita ver; la config de hooks del harness debe cubrirlas.
    /// Los guards que no actúan sobre una tool devuelven vacío.
    pub fn tools(&self, h: Harness) -> &'static [&'static str] {
        match (self, h) {
            (Guard::GitC | Guard::ZeroResiduo | Guard::Verify, Harness::Claude) => {
                &["Bash", "PowerShell"]
            }
            (Guard::GitC | Guard::ZeroResiduo | Guard::Verify, Harness::Codex) => &["Bash"],
            (Guard::SearchFirst, Harness::Claude) => {
                &["Agent", "Task", "Edit", "Write", "NotebookEdit"]
            }
            // `collaborationspawn_agent` en 0.166.0, `spawn_agent` en HEAD.
            (Guard::SearchFirst, Harness::Codex) => {
                &["apply_patch", "spawn_agent", "collaborationspawn_agent"]
            }
            (
                Guard::Index
                | Guard::RecallContrato
                | Guard::RecallCompact
                | Guard::RecallSuperpowers
                | Guard::RecallGatePendiente
                | Guard::ReglasTestigo
                | Guard::ReglasInyecta
                | Guard::MarcaCompact,
                _,
            ) => &[],
        }
    }

    /// Variantes de `Decision` que el guard puede devolver.
    pub fn decisiones(&self) -> &'static [&'static str] {
        match self {
            Guard::GitC => &["Rewrite", "Allow"],
            Guard::ZeroResiduo => &["Deny", "Allow"],
            Guard::Verify => &["Allow"],
            Guard::SearchFirst => &["Steer", "Allow"],
            Guard::Index | Guard::RecallContrato | Guard::RecallCompact | Guard::MarcaCompact => {
                &["Allow"]
            }
            Guard::RecallSuperpowers
            | Guard::RecallGatePendiente
            | Guard::ReglasTestigo
            | Guard::ReglasInyecta => &["Avisa"],
        }
    }
}

pub fn soporte(g: Guard, h: Harness) -> Soporte {
    match (g, h) {
        (Guard::GitC, Harness::Claude) => Soporte::Soportado,
        (Guard::ZeroResiduo, Harness::Claude) => Soporte::Soportado,
        (Guard::Verify, Harness::Claude) => Soporte::Soportado,
        (Guard::SearchFirst, Harness::Claude) => Soporte::Soportado,
        (Guard::GitC, Harness::Codex) => Soporte::Soportado,
        (Guard::ZeroResiduo, Harness::Codex) => Soporte::Soportado,
        (Guard::Verify, Harness::Codex) => Soporte::Degradado("ok: null, solo log"),
        (Guard::SearchFirst, Harness::Codex) => Soporte::Soportado,
        (Guard::Index, Harness::Claude) => Soporte::Soportado,
        (Guard::Index, Harness::Codex) => Soporte::Soportado,
        (Guard::RecallContrato, Harness::Claude) => Soporte::Soportado,
        (Guard::RecallContrato, Harness::Codex) => Soporte::Soportado,
        (Guard::RecallCompact, Harness::Claude) => Soporte::Soportado,
        // La spec §6.2 lo daba NoSoportado; el golden recapturado de 0.166.0 trae SessionStart
        // con source=compact (condición §8.6), así que la reafirmación también llega por ahí.
        (Guard::RecallCompact, Harness::Codex) => Soporte::Soportado,
        (Guard::RecallSuperpowers, Harness::Claude) => Soporte::Soportado,
        (Guard::RecallSuperpowers, Harness::Codex) => {
            Soporte::NoSoportado("lee ~/.claude/settings.json")
        }
        (Guard::RecallGatePendiente, Harness::Claude) => Soporte::Soportado,
        (Guard::RecallGatePendiente, Harness::Codex) => Soporte::Soportado,
        (Guard::ReglasTestigo, Harness::Claude) => Soporte::Soportado,
        (Guard::ReglasTestigo, Harness::Codex) => Soporte::NoSoportado("nadie lee ss-<sid>"),
        (Guard::ReglasInyecta, Harness::Claude) => Soporte::NoSoportado("lo hace el mod"),
        (Guard::ReglasInyecta, Harness::Codex) => Soporte::Soportado,
        (Guard::MarcaCompact, Harness::Claude) => Soporte::NoSoportado("no hace falta"),
        (Guard::MarcaCompact, Harness::Codex) => Soporte::Soportado,
    }
}
