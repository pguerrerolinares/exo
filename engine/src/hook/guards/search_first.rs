//! Guard search-first: avisa una vez por sesión si el primer trabajo sustantivo
//! llega sin un `exo search`/`exo targets` previo en el journal.
use crate::hook::evento::{Decision, Entorno, Evento};
use crate::hook::{journal, reflex};
use regex::Regex;
use std::sync::OnceLock;

const AVISO: &str = "Reflejo search-first: primer trabajo sustantivo de la sesión sin `exo search`/`exo targets` previo. Si el tema puede tener historia en la KB, busca antes (`exo search --type hybrid \"<tema>\"`); si no, dilo en una línea y sigue.";
const CODE: &str = "search-first";

fn busqueda() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(^|[;&|/\s])exo(\.exe)?\s+(search|targets)(\s|$)").expect("regex literal")
    })
}

pub fn search_first(ev: &Evento, en: &Entorno) -> Decision {
    // Por nombre y no por Clase: `Edit` sin ruta se clasifica Other y debe avisar.
    let aplica = ev.tool.as_ref().is_some_and(|t| {
        matches!(
            t.nombre.as_str(),
            "Agent" | "Task" | "Edit" | "Write" | "NotebookEdit"
        )
    });
    if !aplica || !ev.agent_id.is_empty() {
        return Decision::Allow;
    }

    let lectura = journal::lee(en, ev.harness, &ev.session_id, "");
    if let Ok(Some(l)) = &lectura
        && l.registros
            .iter()
            .any(|r| r.kind == "warn" && r.code == CODE)
    {
        return Decision::Allow;
    }
    // Antes de decidir: como mucho un intento por sesión, haya veredicto o no.
    let _ = journal::registra_warn(en, ev, CODE);

    let motivo = match lectura {
        Err(_) => Some("motivo=journal-ilegible"),
        Ok(None) => None,
        Ok(Some(l)) => {
            let hallada = l
                .registros
                .iter()
                .any(|r| r.kind == "tool" && r.tool == "Bash" && busqueda().is_match(&r.cmd));
            if hallada {
                reflex::registra(en, "search-first-ok", ev, "");
                return Decision::Allow;
            }
            None
        }
    };
    reflex::registra(en, CODE, ev, motivo.unwrap_or(""));
    Decision::Steer(AVISO.into())
}
