//! Matriz guard x harness: qué garantiza cada guard en cada harness.

use super::evento::{EventoKind, Harness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    GitC,
    ZeroResiduo,
    Verify,
    SearchFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Soporte {
    Soportado,
    Degradado(&'static str),
    NoSoportado(&'static str),
}

pub const GUARDS: [Guard; 4] = [
    Guard::GitC,
    Guard::ZeroResiduo,
    Guard::Verify,
    Guard::SearchFirst,
];

impl Guard {
    pub fn nombre(&self) -> &'static str {
        match self {
            Guard::GitC => "git-c",
            Guard::ZeroResiduo => "zero-residuo",
            Guard::Verify => "verify",
            Guard::SearchFirst => "search-first",
        }
    }

    pub fn evento(&self) -> EventoKind {
        EventoKind::PreToolUse
    }

    /// Variantes de `Decision` que el guard puede devolver.
    pub fn decisiones(&self) -> &'static [&'static str] {
        match self {
            Guard::GitC => &["Rewrite", "Allow"],
            Guard::ZeroResiduo => &["Deny", "Allow"],
            Guard::Verify => &["Allow"],
            Guard::SearchFirst => &["Steer", "Allow"],
        }
    }
}

pub fn soporte(g: Guard, h: Harness) -> Soporte {
    match (g, h) {
        (Guard::GitC, Harness::Claude) => Soporte::Soportado,
        (Guard::ZeroResiduo, Harness::Claude) => Soporte::Soportado,
        (Guard::Verify, Harness::Claude) => Soporte::Soportado,
        (Guard::SearchFirst, Harness::Claude) => Soporte::Soportado,
    }
}
