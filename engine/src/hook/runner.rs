//! Orquesta un evento: parsea, corre los guards y renderiza. Nunca falla:
//! todo error degrada a stdout vacío (un exit distinto de 0 bloquearía la herramienta).

use super::evento::{Decision, Entorno, Evento, EventoKind, Harness, combinar};
use super::guards::{
    git_c::git_c,
    index::index,
    recall::{FALLBACK, Fragmentos, compone_claude, reafirmacion, recall_contrato},
    recall_avisos::{gate_pendiente, superpowers},
    reglas::{engine_stale, plazo_git, registra_skip, reglas_testigo, resuelve_reglas},
    search_first::search_first,
    verify::verify,
    zero_residuo::zero_residuo,
};
use super::journal::{marca_pendiente, registra_evento, registra_marca, registra_tool};
use super::matriz::{GUARDS, Guard, Soporte, soporte};
use super::reflex::registra;
use super::{claude, codex};
use crate::obsolescencia::formatea_rfc3339_utc;
use crate::state::{Migracion, migra_todo};
use serde_json::{Value, json};
use std::ffi::OsStr;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// 4 × `additionalContextLimit` (2500 tokens): por encima Codex trunca el contexto por el medio.
const SPILL_BYTES: usize = 10_000;

fn harness_de(nombre: &str) -> Option<Harness> {
    match nombre {
        "claude" => Some(Harness::Claude),
        "codex" => Some(Harness::Codex),
        _ => None,
    }
}

fn render(h: Harness, d: &Decision, ev: &Evento) -> String {
    match h {
        Harness::Claude => claude::render(d, ev),
        Harness::Codex => codex::render(d, ev),
    }
}

/// Corre `f` aislando sus panics: ante uno anota `hook-panic` y devuelve `si_panic`.
fn guarda<T>(nombre: &str, ev: &Evento, en: &Entorno, f: impl FnOnce() -> T, si_panic: T) -> T {
    catch_unwind(AssertUnwindSafe(|| {
        if en.panic_guard.as_deref() == Some(nombre) {
            panic!("EXO_HOOK_PANIC={nombre}");
        }
        f()
    }))
    .unwrap_or_else(|_| {
        registra(en, "hook-panic", ev, nombre);
        si_panic
    })
}

fn corre_guard(g: Guard, ev: &Evento, en: &Entorno) -> Decision {
    guarda(
        g.nombre(),
        ev,
        en,
        || match g {
            Guard::GitC => git_c(ev, en, en.cdpath_definido),
            Guard::ZeroResiduo => zero_residuo(ev, en),
            Guard::Verify => verify(ev, en),
            Guard::SearchFirst => search_first(ev, en),
            _ => Decision::Allow,
        },
        Decision::Allow,
    )
}

/// Evento vacío para anotar en el reflex-log un fallo previo al parseo:
/// `session_id` queda en "" porque no hay stdin del que sacarlo.
fn evento_vacio(h: Harness) -> Evento {
    Evento {
        harness: h,
        kind: EventoKind::Otro(String::new()),
        session_id: String::new(),
        cwd: String::new(),
        agent_id: String::new(),
        agent_type: String::new(),
        crudo: String::new(),
        tool: None,
        source: String::new(),
        prompt: String::new(),
    }
}

/// Red final: un panic fuera de los guards (parse, render, journal) también
/// degrada a stdout vacío; un exit 101 bloquearía la herramienta.
pub fn ejecutar(
    en: &Entorno,
    harness: &str,
    evento: &str,
    min: Option<&str>,
    stdin: &[u8],
) -> String {
    catch_unwind(AssertUnwindSafe(|| {
        interno(en, harness, evento, min, stdin)
    }))
    .unwrap_or_else(|_| {
        let h = harness_de(harness).unwrap_or(Harness::Claude);
        registra(en, "hook-panic", &evento_vacio(h), "runner");
        String::new()
    })
}

fn interno(en: &Entorno, harness: &str, evento: &str, min: Option<&str>, stdin: &[u8]) -> String {
    if en.panic_guard.as_deref() == Some("runner") {
        panic!("EXO_HOOK_PANIC=runner");
    }
    let Some(h) = harness_de(harness) else {
        return String::new();
    };
    let parseado = match h {
        Harness::Claude => claude::parse(evento, stdin),
        Harness::Codex => codex::parse(evento, stdin),
    };
    let ev = match parseado {
        Ok(ev) => ev,
        Err(_) => {
            registra(en, "hook-error", &evento_vacio(h), "code=parse");
            return String::new();
        }
    };
    let vacio = || render(h, &Decision::Allow, &ev);
    match ev.kind {
        EventoKind::PreToolUse => {
            let ds = GUARDS
                .iter()
                .filter(|g| g.evento() == EventoKind::PreToolUse)
                .filter(|g| !matches!(soporte(**g, h), Soporte::NoSoportado(_)))
                .map(|g| corre_guard(*g, &ev, en))
                .collect();
            render(h, &combinar(ds), &ev)
        }
        EventoKind::PostToolUse => {
            // Codex no informa del resultado: `ok: null`.
            let ok = (h == Harness::Claude).then_some(true);
            let _ = registra_tool(en, &ev, ok);
            vacio()
        }
        EventoKind::PostToolUseFailure => {
            let _ = registra_tool(en, &ev, Some(false));
            vacio()
        }
        EventoKind::SessionStart => session_start(en, h, &ev, min),
        EventoKind::Stop => {
            if activo(Guard::Index, h, &ev) {
                guarda(
                    Guard::Index.nombre(),
                    &ev,
                    en,
                    || index(&ev, en),
                    Decision::Allow,
                );
            }
            vacio()
        }
        EventoKind::UserPromptSubmit => user_prompt_submit(en, h, &ev),
        EventoKind::PostCompact => {
            if activo(Guard::MarcaCompact, h, &ev) {
                let _ = registra_evento(en, &ev);
                let _ = registra_marca(en, &ev, "post-compact");
            }
            vacio()
        }
        EventoKind::Otro(_) => vacio(),
    }
}

/// El guard está soportado en `h` y el payload trae los campos de `requiere()`.
fn activo(g: Guard, h: Harness, ev: &Evento) -> bool {
    !matches!(soporte(g, h), Soporte::NoSoportado(_))
        && g.requiere().iter().all(|(campo, valor)| match *campo {
            "source" => ev.source == *valor,
            _ => false,
        })
}

fn kb_de(en: &Entorno) -> Option<PathBuf> {
    en.seams
        .exo_kb
        .clone()
        .or_else(|| crate::kb_desde_config().ok())
}

pub(crate) fn cwd_de(ev: &Evento) -> PathBuf {
    if ev.cwd.is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        PathBuf::from(&ev.cwd)
    }
}

/// `a < b` por semver; si alguno no parsea, no se declara obsoleto.
fn menor(a: &str, b: &str) -> bool {
    match (
        crate::doctor::parse_semver(a),
        crate::doctor::parse_semver(b),
    ) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

/// Piezas del contexto -> texto. Claude: contrato + avisos con cap y la reafirmación detrás
/// (las reglas las inyecta el mod). Codex: contrato (cap) -> reglas -> reafirmación -> avisos,
/// porque Codex trunca por el medio. Devuelve además los bytes previos al recorte, si hubo.
pub fn ensambla(h: Harness, f: &Fragmentos, cap: usize) -> (String, Option<usize>) {
    if h == Harness::Claude {
        return compone_claude(f, cap);
    }
    let solo_contrato = Fragmentos {
        contrato: f.contrato.clone(),
        ..Fragmentos::default()
    };
    let (mut texto, recortado) = compone_claude(&solo_contrato, cap);
    let mut une = |parte: &str| {
        if parte.is_empty() {
            return;
        }
        if !texto.is_empty() {
            texto.push_str("\n\n");
        }
        texto.push_str(parte);
    };
    une(&f.reglas);
    une(f.reafirmacion.trim_start_matches('\n'));
    for a in &f.avisos {
        une(a);
    }
    (texto, recortado)
}

/// `additionalContext` (+ `systemMessage` con los avisos visibles) de SessionStart/UPS.
fn salida_contexto(h: Harness, ev: &Evento, texto: &str, visibles: &[String]) -> String {
    let base = render(h, &Decision::Steer(texto.into()), ev);
    if visibles.is_empty() {
        return base;
    }
    let mut v: Value = serde_json::from_str(&base).unwrap_or_else(|_| json!({}));
    v["systemMessage"] = json!(visibles.join("; "));
    v.to_string()
}

/// Mueve `~/.claude/{reflex-log.jsonl,exo-index.log,reflex-inject-canary}` a la ruta nueva.
fn migra(en: &Entorno, ev: &Evento) {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    // Sin `:` en el sufijo del nombre de fichero (Windows).
    let ts: String = formatea_rfc3339_utc(epoch)
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    for (nombre, m) in migra_todo(&en.rutas.home, &en.rutas, &ts) {
        if let Migracion::Fallo(e) = m {
            registra(en, "state-migrate-failed", ev, &format!("{nombre}: {e}"));
        }
    }
}

fn session_start(en: &Entorno, h: Harness, ev: &Evento, min: Option<&str>) -> String {
    if let Some(min) = min {
        if crate::doctor::parse_semver(min).is_none() {
            registra(en, "hook-error", ev, "code=min");
        } else if menor(VERSION, min) {
            return engine_obsoleto(en, h, ev, min);
        }
    }
    guarda("migracion", ev, en, || migra(en, ev), ());

    let compact = ev.source == "compact";
    let kb = kb_de(en);
    let mut fr = Fragmentos::default();
    // Avisos para el humano (`systemMessage`); en Codex además van al contexto.
    let mut visibles: Vec<String> = vec![];

    if activo(Guard::RecallContrato, h, ev) {
        let (contrato, razones) = guarda(
            Guard::RecallContrato.nombre(),
            ev,
            en,
            || recall_contrato(ev, en),
            (FALLBACK.to_string(), vec![]),
        );
        for r in razones {
            registra(en, "recall-fallback", ev, &format!("reason={r}"));
        }
        fr.contrato = contrato;
    }
    if activo(Guard::RecallCompact, h, ev) {
        if !ev.session_id.is_empty() {
            registra(en, "compact", ev, "compact");
        }
        fr.reafirmacion = guarda(
            Guard::RecallCompact.nombre(),
            ev,
            en,
            || reafirmacion(&ev.session_id, en),
            String::new(),
        );
    }
    if activo(Guard::RecallSuperpowers, h, ev) {
        let proyecto = en
            .seams
            .claude_project_dir
            .clone()
            .unwrap_or_else(|| cwd_de(ev));
        let aviso = guarda(
            Guard::RecallSuperpowers.nombre(),
            ev,
            en,
            || superpowers(&en.rutas.home, &proyecto),
            None,
        );
        fr.avisos.extend(aviso);
    }
    if activo(Guard::RecallGatePendiente, h, ev)
        && !compact
        && let Some(kb) = &kb
    {
        let plazo = Duration::from_secs(en.seams.git_timeout_s);
        let r = guarda(
            Guard::RecallGatePendiente.nombre(),
            ev,
            en,
            || gate_pendiente(kb, plazo, OsStr::new("git")),
            Ok(vec![]),
        );
        match r {
            Ok(avisos) => fr.avisos.extend(avisos),
            Err(_) => registra(
                en,
                "gate-timeout",
                ev,
                &format!("deadline={}s", en.seams.git_timeout_s),
            ),
        }
    }
    if activo(Guard::ReglasTestigo, h, ev)
        && let Some(kb) = &kb
        && let Some(r) = guarda(
            Guard::ReglasTestigo.nombre(),
            ev,
            en,
            || reglas_testigo(en, kb, ev),
            None,
        )
    {
        visibles.extend(r.avisos);
    }
    if activo(Guard::ReglasInyecta, h, ev) {
        // Inyecta también con compact: consume la marca si PostCompact la dejó antes.
        if compact && marca_pendiente(en, ev) {
            let _ = registra_marca(en, ev, "post-compact-consumed");
        }
        let (bloque, avisos) = guarda(
            Guard::ReglasInyecta.nombre(),
            ev,
            en,
            || reglas_inyecta(en, ev, kb.as_deref()),
            (String::new(), vec![]),
        );
        if bloque.is_empty() && fr.contrato == FALLBACK {
            visibles.push(
                "⚠ sin reglas duras: contrato de memoria en modo fallback y ninguna regla resuelta"
                    .into(),
            );
        }
        fr.reglas = bloque;
        visibles.extend(avisos);
    }

    if h == Harness::Codex {
        fr.avisos.extend(visibles.iter().cloned());
    }
    let (texto, recortado) = ensambla(h, &fr, en.seams.bloque_cap);
    if let Some(antes) = recortado {
        registra(
            en,
            "recall-truncated",
            ev,
            &format!("bytes={antes} cap={}", en.seams.bloque_cap),
        );
    }
    if h == Harness::Codex && texto.len() > SPILL_BYTES {
        registra(
            en,
            "context-spill-risk",
            ev,
            &format!("bytes={}", texto.len()),
        );
    }
    salida_contexto(h, ev, &texto, &visibles)
}

/// Reglas duras para el contexto (solo Codex): bloque y avisos visibles.
fn reglas_inyecta(en: &Entorno, ev: &Evento, kb: Option<&Path>) -> (String, Vec<String>) {
    let Some(kb) = kb else {
        return (String::new(), vec![]);
    };
    let r = resuelve_reglas(kb, &cwd_de(ev), plazo_git(en));
    registra_skip(en, ev, &r);
    if !r.bloque.is_empty() {
        registra(
            en,
            "project-rules-emitted",
            ev,
            &format!("bytes={}", r.bloque.len()),
        );
    }
    (r.bloque, r.avisos)
}

/// Engine más viejo que `--min`: aviso + fallback; sin migración ni testigo (spec §4.7).
fn engine_obsoleto(en: &Entorno, h: Harness, ev: &Evento, min: &str) -> String {
    registra(
        en,
        "recall-fallback",
        ev,
        &format!("reason=engine-stale engine={VERSION} min={min}"),
    );
    let r = engine_stale(&cwd_de(ev));
    registra_skip(en, ev, &r);
    let mut aviso =
        format!("engine desactualizado ({VERSION} < {min}) — actualiza el binario instalado");
    if h == Harness::Codex {
        aviso.push_str("; sin reglas duras");
    }
    let mut visibles = vec![aviso];
    visibles.extend(r.avisos);
    let fr = Fragmentos {
        contrato: FALLBACK.into(),
        avisos: if h == Harness::Codex {
            visibles.clone()
        } else {
            vec![]
        },
        ..Fragmentos::default()
    };
    let (texto, _) = ensambla(h, &fr, en.seams.bloque_cap);
    salida_contexto(h, ev, &texto, &visibles)
}

/// Codex: la primera `UserPromptSubmit` tras un `PostCompact` inyecta reglas + reafirmación una vez.
fn user_prompt_submit(en: &Entorno, h: Harness, ev: &Evento) -> String {
    let _ = registra_evento(en, ev);
    let vacio = render(h, &Decision::Allow, ev);
    if !activo(Guard::ReglasInyecta, h, ev) || !marca_pendiente(en, ev) {
        return vacio;
    }
    // Consumir antes de resolver: si algo tarda o falla, el siguiente UPS no repite.
    let _ = registra_marca(en, ev, "post-compact-consumed");
    let kb = kb_de(en);
    let (bloque, avisos) = guarda(
        Guard::ReglasInyecta.nombre(),
        ev,
        en,
        || reglas_inyecta(en, ev, kb.as_deref()),
        (String::new(), vec![]),
    );
    let fr = Fragmentos {
        reglas: bloque,
        reafirmacion: guarda(
            Guard::RecallCompact.nombre(),
            ev,
            en,
            || reafirmacion(&ev.session_id, en),
            String::new(),
        ),
        avisos: avisos.clone(),
        ..Fragmentos::default()
    };
    let (texto, _) = ensambla(h, &fr, en.seams.bloque_cap);
    if texto.is_empty() {
        return vacio;
    }
    if h == Harness::Codex && texto.len() > SPILL_BYTES {
        registra(
            en,
            "context-spill-risk",
            ev,
            &format!("bytes={}", texto.len()),
        );
    }
    salida_contexto(h, ev, &texto, &avisos)
}
