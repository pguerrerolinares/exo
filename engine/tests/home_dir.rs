//! `exo::home_dir` en un hijo con el entorno acotado (los tests no usan `set_var`).

#[cfg(windows)]
#[test]
fn userprofile_manda_en_windows() {
    if let Some(esperado) = std::env::var_os("EXO_HOME_ESPERADO") {
        assert_eq!(exo::home_dir(), Some(std::path::PathBuf::from(esperado)));
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let st = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "userprofile_manda_en_windows",
            "--test-threads=1",
        ])
        .env("USERPROFILE", d.path())
        .env("HOME", "C:\\otro-home")
        .env("EXO_HOME_ESPERADO", d.path())
        .status()
        .unwrap();
    assert!(st.success());
}

#[cfg(windows)]
#[test]
fn userprofile_vacio_o_relativo_cae_a_dirs() {
    if std::env::var_os("EXO_HOME_HIJO").is_some() {
        assert_eq!(exo::home_dir(), dirs::home_dir());
        return;
    }
    for v in ["", "relativa\\ruta"] {
        let st = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "userprofile_vacio_o_relativo_cae_a_dirs",
                "--test-threads=1",
            ])
            .env("USERPROFILE", v)
            .env("EXO_HOME_HIJO", "1")
            .status()
            .unwrap();
        assert!(st.success(), "USERPROFILE={v:?}");
    }
}
