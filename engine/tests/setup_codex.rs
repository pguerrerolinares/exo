//! `exo setup codex`: edición del bloque `# >>> exo` de `config.toml` y
//! escritura de los roles. Todo contra tempdirs: nunca `~/.codex` ni `~/.exo`.
use exo::setup_codex::{ROL_CONSULTOR, ROL_EXECUTOR, aplica, ejecuta};
use std::fs;
use std::path::Path;
use std::process::Command;

fn agentes(dir: &Path) -> std::path::PathBuf {
    dir.join("exo").join("codex-agents")
}

fn tabla<'a>(v: &'a toml::Value, nombre: &str) -> &'a toml::Value {
    &v["agents"][nombre]
}

#[test]
fn idempotente() {
    let dir = tempfile::tempdir().unwrap();
    let ag = agentes(dir.path());
    for x in ["", "model = \"o3\"\n\n[tui]\ntheme = \"dark\"\n"] {
        let una = aplica(x, &ag).unwrap();
        // Un `aplica` identidad también sería idempotente: la primera pasada
        // tiene que traer el bloque entero.
        assert!(
            una.contains("# >>> exo") && una.contains("# <<< exo"),
            "{una}"
        );
        assert!(una.contains("[agents.exo-consultor]"), "{una}");
        assert!(una.contains("[agents.exo-executor]"), "{una}");
        let v: toml::Value = una.parse().unwrap();
        for (n, f) in [
            ("exo-consultor", "consultor.toml"),
            ("exo-executor", "executor.toml"),
        ] {
            let cf = Path::new(tabla(&v, n)["config_file"].as_str().unwrap());
            assert!(cf.is_absolute(), "{cf:?}");
            assert_eq!(cf, ag.join(f));
            assert!(!tabla(&v, n)["description"].as_str().unwrap().is_empty());
        }
        assert_eq!(aplica(&una, &ag).unwrap(), una);
    }
}

#[test]
fn respeta_fuera() {
    let dir = tempfile::tempdir().unwrap();
    let ag = agentes(dir.path());
    let antes = "# mío\r\nmodel = 'x'  \n\n[a]\nk = 1\n\n";
    let despues = "\n[b]\nz = \"ñ\"   # sin newline final";
    let viejo = "# >>> exo\n[agents.exo-consultor]\nconfig_file = \"/viejo\"\n# <<< exo\n";
    let r = aplica(&format!("{antes}{viejo}{despues}"), &ag).unwrap();
    assert!(r.starts_with(antes), "{r:?}");
    assert!(r.ends_with(despues), "{r:?}");
    assert!(!r.contains("/viejo"), "el bloque se reescribe en su sitio");
    assert_eq!(r.matches("# >>> exo").count(), 1);

    // Primera vez: se añade al final y lo previo queda intacto, aunque no acabe en newline.
    let orig = "model = 'x'  # sin newline";
    let r = aplica(orig, &ag).unwrap();
    assert!(r.starts_with(orig), "{r:?}");
    assert!(r.contains("[agents.exo-executor]"));
}

#[test]
fn tabla_fuera_falla() {
    let dir = tempfile::tempdir().unwrap();
    let ag = agentes(dir.path());
    for n in ["exo-consultor", "exo-executor"] {
        let cfg = format!("[agents.{n}]\ndescription = \"mío\"\n");
        let e = aplica(&cfg, &ag).unwrap_err().to_string();
        assert!(e.contains(&format!("agents.{n}")), "{e}");
    }

    // `ejecuta` no escribe nada si `aplica` falla.
    let home = dir.path().join("codex");
    fs::create_dir_all(&home).unwrap();
    let cfg = "[agents.exo-consultor]\nx = 1\n";
    fs::write(home.join("config.toml"), cfg).unwrap();
    let raiz = dir.path().join("exo");
    assert!(ejecuta(&home, &raiz).is_err());
    assert_eq!(fs::read_to_string(home.join("config.toml")).unwrap(), cfg);
    assert!(!raiz.exists(), "no se escribe ningún rol");
}

#[test]
fn marcador_huerfano_falla() {
    let dir = tempfile::tempdir().unwrap();
    let ag = agentes(dir.path());
    for cfg in ["a = 1\n# >>> exo\n[x]\n", "a = 1\n# <<< exo\n"] {
        let e = aplica(cfg, &ag).unwrap_err().to_string();
        assert!(e.contains("# >>> exo") || e.contains("# <<< exo"), "{e}");
    }
    assert!(
        aplica("# <<< exo\n# >>> exo\n", &ag).is_err(),
        "cierre antes de la apertura"
    );
}

#[test]
fn no_altera_ajenas() {
    let dir = tempfile::tempdir().unwrap();
    let ag = agentes(dir.path());
    let viejo = "# >>> exo\n[agents.exo-executor]\nconfig_file = \"/v\"\n# <<< exo\n";
    // Sin cabecera detrás del bloque: caería dentro de [agents.exo-executor].
    let e = aplica(&format!("{viejo}ajena = 1\n"), &ag)
        .unwrap_err()
        .to_string();
    assert!(e.contains("ajena"), "{e}");
    // Lo mismo con claves de la raíz del fichero antes de que exista bloque.
    assert!(aplica("a = 1\n", &ag).is_ok());
    // Y una tabla propia detrás del bloque es válida.
    assert!(aplica(&format!("{viejo}\n[otra]\najena = 1\n"), &ag).is_ok());
}

#[test]
fn config_que_no_parsea_falla() {
    let dir = tempfile::tempdir().unwrap();
    assert!(aplica("esto no es = = toml", &agentes(dir.path())).is_err());
}

#[test]
fn toml_valido() {
    let dir = tempfile::tempdir().unwrap();
    let out = aplica("model = \"o3\"\n", &agentes(dir.path())).unwrap();
    let v: toml::Value = out.parse().expect("la salida parsea");
    assert_eq!(v["model"].as_str(), Some("o3"));

    let rol = |t: &str| -> toml::Value { t.parse().expect("el rol parsea") };
    for (t, esfuerzo) in [(ROL_CONSULTOR, "high"), (ROL_EXECUTOR, "medium")] {
        let r = rol(t);
        assert_eq!(r["model"].as_str(), Some("gpt-6.1-sol"));
        assert_eq!(r["model_reasoning_effort"].as_str(), Some(esfuerzo));
        let di = r["developer_instructions"].as_str().unwrap();
        assert!(di.contains("spawn_agent"), "sintaxis de Codex");
        assert!(!di.contains("Agent"), "sin la sintaxis de Claude");
    }
    let di = rol(ROL_CONSULTOR)["developer_instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        di.contains("solo lectura"),
        "la restricción va como texto (spec §10.5)"
    );
}

#[test]
fn roles_derivan_de_los_md() {
    let md = Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugins/exo/agents");
    if !md.is_dir() {
        // El crate puede compilarse aislado (cargo-mutants copia solo engine/).
        eprintln!(
            "roles_derivan_de_los_md: omitido, no existe {}",
            md.display()
        );
        return;
    }
    for (f, rol) in [
        ("consultor.md", ROL_CONSULTOR),
        ("executor.md", ROL_EXECUTOR),
    ] {
        let texto = fs::read_to_string(md.join(f)).unwrap();
        let cuerpo = texto.splitn(3, "---\n").nth(2).unwrap();
        let leads: Vec<&str> = cuerpo
            .lines()
            .filter_map(|l| l.strip_prefix("- **")?.split("**").next())
            .collect();
        assert!(leads.len() >= 6, "{f}: {leads:?}");
        for lead in leads {
            assert!(
                rol.contains(&format!("**{lead}**")),
                "{f}: el rol perdió «{lead}»"
            );
        }
    }
}

#[test]
fn respeta_codex_home() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("mi-codex");
    let raiz = dir.path().join("mi-exo");
    ejecuta(&home, &raiz).unwrap();
    let cfg = fs::read_to_string(home.join("config.toml")).unwrap();
    let v: toml::Value = cfg.parse().unwrap();
    assert_eq!(
        Path::new(tabla(&v, "exo-consultor")["config_file"].as_str().unwrap()),
        raiz.join("codex-agents").join("consultor.toml")
    );
    assert_eq!(
        fs::read_to_string(raiz.join("codex-agents/consultor.toml")).unwrap(),
        ROL_CONSULTOR
    );
    assert_eq!(
        fs::read_to_string(raiz.join("codex-agents/executor.toml")).unwrap(),
        ROL_EXECUTOR
    );
    let sobras: Vec<_> = fs::read_dir(&home).unwrap().flatten().collect();
    assert_eq!(sobras.len(), 1, "sin tmp sobrante: {sobras:?}");
    ejecuta(&home, &raiz).unwrap();
    assert_eq!(fs::read_to_string(home.join("config.toml")).unwrap(), cfg);
}

// Solo unix: `dirs::home_dir()` en Windows ignora HOME/USERPROFILE del proceso
// hijo y el test escribiría en el perfil real.
#[cfg(unix)]
#[test]
fn cli_usa_codex_home_y_home_del_entorno() {
    let dir = tempfile::tempdir().unwrap();
    let (home, codex) = (dir.path().join("h"), dir.path().join("c"));
    let run = |codex_home: &Path| {
        Command::new(env!("CARGO_BIN_EXE_exo"))
            .args(["setup", "codex"])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("CODEX_HOME", codex_home)
            .output()
            .unwrap()
    };
    assert!(run(&codex).status.success());
    assert!(codex.join("config.toml").is_file());
    assert!(home.join(".exo/codex-agents/executor.toml").is_file());

    fs::write(codex.join("config.toml"), "# >>> exo\n").unwrap();
    let o = run(&codex);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("# >>> exo"));
}
