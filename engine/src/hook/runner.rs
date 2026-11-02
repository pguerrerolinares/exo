//! Orquesta un evento: parsea, corre los guards y renderiza. Nunca falla:
//! todo error degrada a stdout vacío (un exit distinto de 0 bloquearía la herramienta).

use super::claude;
use super::evento::{Decision, Entorno, Evento, EventoKind, Harness, combinar};
use super::guards::{
    git_c::git_c, search_first::search_first, verify::verify, zero_residuo::zero_residuo,
};
use super::journal::registra_tool;
use super::matriz::{GUARDS, Guard, Soporte, soporte};
use super::reflex::registra;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn corre_guard(g: Guard, ev: &Evento, en: &Entorno) -> Decision {
    let r = catch_unwind(AssertUnwindSafe(|| {
        if en.panic_guard.as_deref() == Some(g.nombre()) {
            panic!("EXO_HOOK_PANIC={}", g.nombre());
        }
        match g {
            Guard::GitC => git_c(ev, en, en.cdpath_definido),
            Guard::ZeroResiduo => zero_residuo(ev, en),
            Guard::Verify => verify(ev, en),
            Guard::SearchFirst => search_first(ev, en),
        }
    }));
    r.unwrap_or_else(|_| {
        registra(en, "hook-panic", ev, g.nombre());
        Decision::Allow
    })
}

/// Evento vacío para anotar en el reflex-log un fallo previo al parseo:
/// `session_id` queda en "" porque no hay stdin del que sacarlo.
fn evento_vacio() -> Evento {
    Evento {
        harness: Harness::Claude,
        kind: EventoKind::Otro(String::new()),
        session_id: String::new(),
        cwd: String::new(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: String::new(),
        tool: None,
    }
}

pub fn ejecutar(en: &Entorno, harness: &str, evento: &str, stdin: &[u8]) -> String {
    // Los panics de los guards se registran en el reflex-log, no en stderr.
    std::panic::set_hook(Box::new(|_| {}));
    let h = match harness {
        "claude" => Harness::Claude,
        _ => return String::new(),
    };
    let ev = match claude::parse(evento, stdin) {
        Ok(ev) => ev,
        Err(_) => {
            registra(en, "hook-error", &evento_vacio(), "code=parse");
            return String::new();
        }
    };
    match ev.kind {
        EventoKind::PreToolUse => {
            let ds = GUARDS
                .iter()
                .filter(|g| !matches!(soporte(**g, h), Soporte::NoSoportado(_)))
                .map(|g| corre_guard(*g, &ev, en))
                .collect();
            claude::render(&combinar(ds), &ev)
        }
        EventoKind::PostToolUse => {
            let _ = registra_tool(en, &ev, Some(true));
            String::new()
        }
        EventoKind::PostToolUseFailure => {
            let _ = registra_tool(en, &ev, Some(false));
            String::new()
        }
        EventoKind::Otro(_) => String::new(),
    }
}
