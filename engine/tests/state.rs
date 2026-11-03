//! `Rutas` y `Seams`: precedencia, defaults y «vacío = ausente». Todo con `var`
//! inyectado: ningún test toca el entorno del proceso.

use exo::state::{Rutas, Seams};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn env(pares: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let m: HashMap<String, OsString> = pares
        .iter()
        .map(|(k, v)| (k.to_string(), OsString::from(v)))
        .collect();
    move |k| m.get(k).cloned()
}

#[test]
fn precedencia_variable_propia() {
    let r = Rutas::resuelve(
        Path::new("/h"),
        env(&[("REFLEX_LOG_FILE", "/x/r.jsonl"), ("EXO_STATE_DIR", "/s")]),
    );
    assert_eq!(r.reflex_log, PathBuf::from("/x/r.jsonl"));
    assert_eq!(r.index_log, PathBuf::from("/s/exo-index.log"));
    assert_eq!(r.state_dir, PathBuf::from("/s"));
    assert_eq!(r.rules_dir, PathBuf::from("/s/exo-rules"));
    assert_eq!(r.canary, PathBuf::from("/s/reflex-inject-canary"));
}

#[test]
fn default_bajo_home() {
    let r = Rutas::resuelve(Path::new("/h"), env(&[]));
    assert_eq!(r.state_dir, PathBuf::from("/h/.exo/state"));
    assert_eq!(
        r.reflex_log,
        PathBuf::from("/h/.exo/state/reflex-log.jsonl")
    );
    assert_eq!(r.rules_dir, PathBuf::from("/h/.exo/state/exo-rules"));
    assert_eq!(r.index_log, PathBuf::from("/h/.exo/state/exo-index.log"));
    assert_eq!(
        r.canary,
        PathBuf::from("/h/.exo/state/reflex-inject-canary")
    );
}

#[test]
fn variable_vacia_es_ausente() {
    let r = Rutas::resuelve(
        Path::new("/h"),
        env(&[
            ("EXO_STATE_DIR", ""),
            ("REFLEX_LOG_FILE", ""),
            ("EXO_INDEX_LOG", ""),
            ("REFLEX_CANARY_FILE", ""),
        ]),
    );
    assert_eq!(r, Rutas::resuelve(Path::new("/h"), env(&[])));
    assert_eq!(r.state_dir, PathBuf::from("/h/.exo/state"));
}

#[test]
fn seams_defaults() {
    let s = Seams::resuelve(env(&[]));
    assert_eq!(s.recall_cap, 6144);
    assert_eq!(s.bloque_cap, 1500);
    assert_eq!(s.recall_limite, 10);
    assert_eq!(s.compact_lineas, 2000);
    assert_eq!(s.git_timeout_s, 3);
    assert_eq!(s.exo_bin, None);
    assert_eq!(s.exo_index, None);
    assert_eq!(s.exo_kb, None);
    assert_eq!(s.exo_kb_name, None);
    assert_eq!(s.recall_nota, None);
    assert_eq!(s.claude_project_dir, None);
    assert_eq!(s.codex_home, None);
}

#[test]
fn seams_override_por_variable() {
    let s = Seams::resuelve(env(&[
        ("EXO_BIN", "/b/exo"),
        ("EXO_INDEX", "/i.db"),
        ("EXO_KB", "/kb"),
        ("EXO_KB_NAME", "demo"),
        ("EXO_RECALL_NOTA", "demo/core/x"),
        ("EXO_RECALL_CAP", "100"),
        ("RECALL_BLOQUE_CAP", "200"),
        ("EXO_RECALL_LIMITE", "7"),
        ("EXO_RECALL_COMPACT_LINEAS", "50"),
        ("EXO_GIT_TIMEOUT", "9"),
        ("CLAUDE_PROJECT_DIR", "/p"),
        ("CODEX_HOME", "/c"),
    ]));
    assert_eq!(s.exo_bin, Some(PathBuf::from("/b/exo")));
    assert_eq!(s.exo_index, Some(PathBuf::from("/i.db")));
    assert_eq!(s.exo_kb, Some(PathBuf::from("/kb")));
    assert_eq!(s.exo_kb_name.as_deref(), Some("demo"));
    assert_eq!(s.recall_nota.as_deref(), Some("demo/core/x"));
    assert_eq!(s.recall_cap, 100);
    assert_eq!(s.bloque_cap, 200);
    assert_eq!(s.recall_limite, 7);
    assert_eq!(s.compact_lineas, 50);
    assert_eq!(s.git_timeout_s, 9);
    assert_eq!(s.claude_project_dir, Some(PathBuf::from("/p")));
    assert_eq!(s.codex_home, Some(PathBuf::from("/c")));
}

#[test]
fn seams_vacio_y_numero_invalido_dan_default() {
    let s = Seams::resuelve(env(&[
        ("EXO_BIN", ""),
        ("EXO_KB_NAME", ""),
        ("EXO_RECALL_CAP", "abc"),
        ("RECALL_BLOQUE_CAP", ""),
        ("EXO_RECALL_LIMITE", "-3"),
        ("EXO_RECALL_COMPACT_LINEAS", "1.5"),
        ("EXO_GIT_TIMEOUT", "x"),
    ]));
    assert_eq!(s.exo_bin, None);
    assert_eq!(s.exo_kb_name, None);
    assert_eq!(s.recall_cap, 6144);
    assert_eq!(s.bloque_cap, 1500);
    assert_eq!(s.recall_limite, 10);
    assert_eq!(s.compact_lineas, 2000);
    assert_eq!(s.git_timeout_s, 3);
}
