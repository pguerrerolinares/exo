//! `exo ratchet` contra el binario real. Task 12 del plan G4c: el trinquete
//! entero (Tasks 3-11, ya commiteado) cableado como verbo del CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb(ficheros: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(ruta, contenido).unwrap();
    }
    dir
}

/// Mismo aislamiento de git que `trinquete_staged.rs`/`trinquete.rs::tests`:
/// `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM` apuntan a un fichero vacío del
/// tempdir, e identidad de autor/committer fija por env var.
fn git_aislado(raiz: &Path, cfg: &Path, args: &[&str]) {
    let salida = Command::new("git")
        .arg("-C")
        .arg(raiz)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", cfg)
        .env("GIT_CONFIG_SYSTEM", cfg)
        .env("GIT_AUTHOR_NAME", "f")
        .env("GIT_AUTHOR_EMAIL", "f@k.local")
        .env("GIT_COMMITTER_NAME", "f")
        .env("GIT_COMMITTER_EMAIL", "f@k.local")
        .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
        .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "git {args:?} falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

fn init_repo(dir: &Path) -> PathBuf {
    let cfg = dir.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    git_aislado(dir, &cfg, &["init", "-q"]);
    cfg
}

fn commit_todo(dir: &Path, cfg: &Path, mensaje: &str) {
    git_aislado(dir, cfg, &["add", "."]);
    git_aislado(dir, cfg, &["commit", "-q", "-m", mensaje]);
}

/// Sin `.git`: `carga_head` devuelve `Ok(None)` (abstención), y la
/// abstención NUNCA gatea — sale 0, no 3, aunque haya declaraciones.
#[test]
fn ratchet_sin_git_sale_cero() {
    let dir = kb(&[("a.md", "---\ntier: core\nkbx_budget_max: 9000\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["ratchet", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["applied"], false);
}

/// Un waiver (`kbx_budget_max`) que rebasa su propio sello es `over-seal`: el
/// waiver no puede rebasar el trinquete. Sello ya commiteado en HEAD (ancla
/// activa), así que no hay exención de activación de por medio.
#[test]
fn ratchet_con_un_waiver_sobre_su_sello_sale_tres() {
    let sello: exo::trinquete::Sellos = [("a.md".to_string(), 5000i64)].into_iter().collect();
    let dir = kb(&[
        (
            exo::trinquete::FICHERO_SELLO,
            exo::trinquete::serializa_sellos(&sello).as_str(),
        ),
        ("a.md", "---\ntier: core\nkbx_budget_max: 6000\n---\nx\n"),
    ]);
    let cfg = init_repo(dir.path());
    commit_todo(dir.path(), &cfg, "inicial, con el sello ya en HEAD");

    let salida = Command::new(bin())
        .args(["ratchet", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        salida.status.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["applied"], true);
    assert_eq!(v["data"]["findings"][0]["path"], "a.md");
    assert_eq!(v["data"]["findings"][0]["kind"], "over-seal");
    assert!(String::from_utf8_lossy(&salida.stderr).contains("rechazado: ratchet:"));
}

/// `#[arg(conflicts_with = "staged")]`: clap rechaza la combinación en el
/// parseo, antes de que `ejecuta` corra nada — exit 2, no 3 ni 1.
#[test]
fn ratchet_seal_y_staged_juntos_son_error_de_uso() {
    let salida = Command::new(bin())
        .args(["ratchet", "--seal", "--staged"])
        .output()
        .unwrap();
    assert_eq!(
        salida.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
}

#[test]
fn el_envelope_json_lleva_command_ratchet() {
    let dir = kb(&[("a.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["ratchet", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "ratchet");
    assert!(v["data"]["findings"].is_array());
}

/// La atomicidad de `--seal`: "o sella todo o no sella nada". `a.md` tiene
/// aire de sobra tras bajar; `b.md` no. El intento tiene que rechazar los
/// DOS — ni siquiera `a.md`, que sí cumplía, se escribe.
#[test]
fn ratchet_seal_es_atomico_no_escribe_nada_si_falta_aire() {
    let sello_inicial: exo::trinquete::Sellos = [
        ("a.md".to_string(), 20000i64),
        ("b.md".to_string(), 9000i64),
    ]
    .into_iter()
    .collect();
    // a.md: declara 15000 para un cuerpo de ~9992B → techo_minimo ≈ 11491,
    // con aire de sobra bajo 15000.
    let cuerpo_a = format!(
        "---\ntier: core\nkbx_budget_max: 15000\n---\n{}\n",
        "x".repeat(9950)
    );
    // b.md: declara 8000 para un cuerpo de ~7991B → techo_minimo ≈ 9190, sin
    // aire bajo 8000.
    let cuerpo_b = format!(
        "---\ntier: core\nkbx_budget_max: 8000\n---\n{}\n",
        "x".repeat(7950)
    );
    let dir = kb(&[
        (
            exo::trinquete::FICHERO_SELLO,
            exo::trinquete::serializa_sellos(&sello_inicial).as_str(),
        ),
        ("a.md", cuerpo_a.as_str()),
        ("b.md", cuerpo_b.as_str()),
    ]);
    let contenido_antes =
        std::fs::read_to_string(dir.path().join(exo::trinquete::FICHERO_SELLO)).unwrap();

    let salida = Command::new(bin())
        .args(["ratchet", "--seal", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        salida.status.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["findings"][0]["path"], "b.md");
    assert_eq!(v["data"]["findings"][0]["kind"], "no-air");

    let contenido_despues =
        std::fs::read_to_string(dir.path().join(exo::trinquete::FICHERO_SELLO)).unwrap();
    assert_eq!(
        contenido_antes, contenido_despues,
        "ni siquiera a.md (que sí tenía aire) debe escribirse: o sella todo o no sella nada"
    );
    let de_disco = exo::trinquete::carga(dir.path()).unwrap();
    assert_eq!(
        de_disco.get("a.md"),
        Some(&20000),
        "a.md debe seguir en su valor original, no en el 15000 al que hubiera bajado"
    );
    assert_eq!(de_disco.get("b.md"), Some(&9000));
}

/// Portado de `TestTextOutputLeadsWithTheCauseAndSummarisesTheDebt`
/// (`cmd/kbx/ratchet_test.go`, kbx `fe46443`): la causa primero, la deuda
/// resumida en una línea al final — nunca mezcladas.
#[test]
fn la_salida_de_texto_no_mezcla_la_deuda_con_lo_que_rompe() {
    let cuerpo = format!(
        "---\ntier: core\nkbx_budget_max: 11000\n---\n{}\n",
        "x".repeat(9900)
    );
    let dir = kb(&[("a.md", cuerpo.as_str()), ("b.md", cuerpo.as_str())]);
    let cfg = init_repo(dir.path());

    let sello_head: exo::trinquete::Sellos = [
        ("a.md".to_string(), 11000i64),
        ("b.md".to_string(), 11000i64),
    ]
    .into_iter()
    .collect();
    std::fs::write(
        dir.path().join(exo::trinquete::FICHERO_SELLO),
        exo::trinquete::serializa_sellos(&sello_head),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "instala el trinquete, con los dos sellos");

    // a.md baja su techo (queda sin aire): transición que rompe. b.md no se
    // toca: sigue sin aire desde HEAD, así que es deuda preexistente.
    let sello_actual: exo::trinquete::Sellos = [
        ("a.md".to_string(), 10900i64),
        ("b.md".to_string(), 11000i64),
    ]
    .into_iter()
    .collect();
    std::fs::write(
        dir.path().join(exo::trinquete::FICHERO_SELLO),
        exo::trinquete::serializa_sellos(&sello_actual),
    )
    .unwrap();

    let salida = Command::new(bin())
        .args(["ratchet"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        salida.status.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    let texto = String::from_utf8_lossy(&salida.stdout).into_owned();
    assert!(texto.contains("a.md: no-air"), "falta la causa: {texto}");
    assert!(
        !texto.contains("b.md"),
        "la deuda de b.md no debe listarse junto a la causa: {texto}"
    );
    assert!(
        texto.contains("hallazgo(s) más"),
        "la deuda debe resumirse en una línea, no desaparecer: {texto}"
    );
    let pos_causa = texto.find("no-air").expect("la causa debe estar");
    let pos_resumen = texto
        .find("hallazgo(s) más")
        .expect("el resumen debe estar");
    assert!(
        pos_causa < pos_resumen,
        "la causa tiene que ir antes del resumen de deuda: {texto}"
    );
}
