use exo::hook::evento::{Clase, Decision, Harness, clasifica, combinar};
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

#[test]
fn combinar_block_gana_a_deny() {
    let d = combinar(vec![
        Decision::Block("b".into()),
        Decision::Deny("d".into()),
    ]);
    assert_eq!(d, Decision::Block("b".into()));
    let d = combinar(vec![
        Decision::Deny("d".into()),
        Decision::Block("b".into()),
    ]);
    assert_eq!(d, Decision::Block("b".into()));
}

#[test]
fn combinar_rewrite_gana_a_steer() {
    let rw = Decision::Rewrite {
        input: json!({"a":1}),
        razon: "r".into(),
    };
    for ds in [
        vec![Decision::Steer("s".into()), rw.clone()],
        vec![rw.clone(), Decision::Steer("s".into())],
    ] {
        assert_eq!(combinar(ds), rw);
    }
}

#[test]
fn combinar_varios_rewrite_gana_el_primero() {
    let a = Decision::Rewrite {
        input: json!({"a":1}),
        razon: "r1".into(),
    };
    let b = Decision::Rewrite {
        input: json!({"b":2}),
        razon: "r2".into(),
    };
    assert_eq!(combinar(vec![a.clone(), b.clone()]), a);
}

#[test]
fn combinar_allow_no_tapa_a_steer() {
    assert_eq!(
        combinar(vec![Decision::Allow, Decision::Steer("s".into())]),
        Decision::Steer("s".into())
    );
}

#[test]
fn avisa_rango_steer() {
    assert_eq!(
        combinar(vec![
            Decision::Steer("a".into()),
            Decision::Avisa("b".into())
        ]),
        Decision::Steer("a\nb".into())
    );
    assert_eq!(
        combinar(vec![
            Decision::Avisa("b".into()),
            Decision::Steer("a".into())
        ]),
        Decision::Steer("a\nb".into())
    );
    assert_eq!(
        combinar(vec![
            Decision::Avisa("b".into()),
            Decision::Avisa("c".into())
        ]),
        Decision::Avisa("b\n\nc".into())
    );
    assert_eq!(
        combinar(vec![
            Decision::Deny("x".into()),
            Decision::Avisa("b".into())
        ]),
        Decision::Deny("x".into())
    );
    let rw = Decision::Rewrite {
        input: json!({"command":"y"}),
        razon: "r".into(),
    };
    assert_eq!(combinar(vec![Decision::Avisa("b".into()), rw.clone()]), rw);
}

#[test]
fn harness_codex_nombre() {
    assert_eq!(Harness::Codex.nombre(), "codex");
}
