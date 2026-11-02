use exo::hook::evento::{Clase, Decision, clasifica, combinar};
use serde_json::json;

#[test]
fn clasifica_bash_y_powershell_son_shell() {
    for n in ["Bash", "PowerShell"] {
        let (c, cmd, p) = clasifica(n, &json!({"command":"ls"}));
        assert_eq!(c, Clase::Shell, "{n}");
        assert_eq!(cmd.as_deref(), Some("ls"), "{n}");
        assert!(p.is_empty());
    }
}

#[test]
fn clasifica_edit_write_notebook_son_fileedit_con_path() {
    for n in ["Edit", "Write"] {
        let (c, cmd, p) = clasifica(n, &json!({"file_path":"a.rs"}));
        assert_eq!(c, Clase::FileEdit, "{n}");
        assert_eq!(p, vec!["a.rs"], "{n}");
        assert_eq!(cmd, None);
    }
    let (c, _, p) = clasifica("NotebookEdit", &json!({"notebook_path":"n.ipynb"}));
    assert_eq!(c, Clase::FileEdit);
    assert_eq!(p, vec!["n.ipynb"]);
}

#[test]
fn clasifica_fileedit_sin_path_se_abstiene() {
    let (c, _, p) = clasifica("Edit", &json!({}));
    assert_eq!(c, Clase::Other);
    assert!(p.is_empty());
}

#[test]
fn clasifica_agent_y_task_son_agent() {
    for n in ["Agent", "Task"] {
        let (c, _, p) = clasifica(n, &json!({}));
        assert_eq!(c, Clase::Agent, "{n}");
        assert!(p.is_empty(), "{n}");
    }
}

#[test]
fn clasifica_command_no_string_es_none() {
    for v in [json!({"command":null}), json!({"command":3})] {
        let (c, cmd, _) = clasifica("Bash", &v);
        assert_eq!(c, Clase::Shell);
        assert_eq!(cmd, None);
    }
    assert_eq!(
        clasifica("Bash", &json!({"command":"ls"})).1.as_deref(),
        Some("ls")
    );
}

#[test]
fn combinar_deny_gana_a_rewrite() {
    let d = combinar(vec![
        Decision::Rewrite {
            input: json!({}),
            razon: "r".into(),
        },
        Decision::Deny("x".into()),
    ]);
    assert_eq!(d, Decision::Deny("x".into()));
}

#[test]
fn combinar_concatena_razones_y_steers() {
    assert_eq!(
        combinar(vec![Decision::Deny("a".into()), Decision::Deny("b".into())]),
        Decision::Deny("a\n\nb".into())
    );
    assert_eq!(
        combinar(vec![
            Decision::Steer("a".into()),
            Decision::Steer("b".into())
        ]),
        Decision::Steer("a\n\nb".into())
    );
}

#[test]
fn combinar_vacio_es_allow() {
    assert_eq!(combinar(vec![]), Decision::Allow);
}
