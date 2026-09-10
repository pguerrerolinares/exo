//! `exo targets` contra el binario real.
//!
//! No usa `tests/common/mod.rs`: estos tests pasan `--db` y `--kb` explícitos,
//! así que `resuelve_db`/`resuelve_kb` cortan en el flag y nunca llegan a
//! cargar config. Declarar el módulo sin usarlo sería un warning, y clippy es
//! gate duro.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// Reusa el montaje de `tests/objetivos.rs` a través de un helper local: KB
/// con git y su índice poblado a mano.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    let cfg = kb.join("gitconfig-vacio");
    std::fs::write(&cfg, "").unwrap();
    let git = |args: &[&str]| {
        let s = Command::new("git")
            .arg("-C")
            .arg(&kb)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", &cfg)
            .env("GIT_CONFIG_SYSTEM", &cfg)
            .env("GIT_AUTHOR_NAME", "f")
            .env("GIT_AUTHOR_EMAIL", "f@k.local")
            .env("GIT_COMMITTER_NAME", "f")
            .env("GIT_COMMITTER_EMAIL", "f@k.local")
            .env("GIT_AUTHOR_DATE", "2026-07-01T10:00:00+02:00")
            .env("GIT_COMMITTER_DATE", "2026-07-01T10:00:00+02:00")
            .output()
            .unwrap();
        assert!(s.status.success(), "git {args:?}");
    };
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "inicial"]);

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

/// Copia de `kb_con_indice()` sin `git init`/`add`/`commit`: el caso real de
/// `exo init --from-basic-memory`, que crea KBs sin versionar. Sin closure de
/// git ni las variables de entorno que solo servían para él — si se quedan,
/// son variables sin usar y `clippy -D warnings` deja la tarea en rojo.
fn kb_con_indice_sin_git() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

// `exo init --from-basic-memory` crea KBs sin git. Ahí `search`, `recall` e
// `index` funcionan y `targets` reventaba en la primera candidata con un
// mensaje de fallo de git por fichero, que no dice qué hacer. Ahora la
// condición se detecta una vez, antes del bucle (A2).
#[test]
fn una_kb_sin_git_da_un_error_accionable_y_no_un_fallo_por_fichero() {
    let (dir, db) = kb_con_indice_sin_git();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();

    assert_eq!(salida.status.code(), Some(1), "es un error, no un gate");
    let stderr = String::from_utf8_lossy(&salida.stderr);
    assert!(
        stderr.contains("no está versionada") || stderr.contains("git init"),
        "el error tiene que nombrar la condición y la salida: {stderr}"
    );
    // Y una sola vez, no una por candidata.
    assert_eq!(stderr.matches("git init").count(), 1);
    assert!(salida.stdout.is_empty());
}

// Regresión real de la review de la Task 11: un fallo de git que NO es
// "esto no es un repo" —aquí, dubious ownership de git >= 2.35.2— tiene que
// llegar al usuario con el mensaje real de git, nunca disfrazado del genérico
// de A2 ("corre git init"). GIT_TEST_ASSUME_DIFFERENT_OWNER=1 fuerza el
// chequeo real sin montar el repo con otra cuenta de sistema; se pasa sobre
// el Command que lanza el binario `exo`, no sobre el proceso de test, para no
// contaminar otros tests que corren en paralelo en el mismo binario.
#[test]
fn un_fallo_de_git_que_no_es_falta_de_repo_no_se_disfraza_de_git_init() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .env("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1")
        .output()
        .unwrap();

    // El stderr se captura ANTES de la primera aserción a propósito: si el
    // exit code no es el esperado, lo único que permite diagnosticar es lo que
    // git dijo, y un `assert_eq!` de códigos a secas lo tira. Medido el
    // 2026-09-10: este test llevaba días en rojo en el CI de ubuntu/macOS con
    // `left: Some(0), right: Some(1)` y ni una pista del porqué, porque el
    // mensaje no traía el stderr. Un assert que no dice qué pasó obliga a
    // reproducir en una plataforma que quizá no tienes.
    let stderr = String::from_utf8_lossy(&salida.stderr);
    let stdout_diag = String::from_utf8_lossy(&salida.stdout);
    assert_eq!(
        salida.status.code(),
        Some(1),
        "es un error, no un gate.\n--- stderr: {stderr}\n--- stdout: {stdout_diag}"
    );
    assert!(
        stderr.contains("dubious ownership") || stderr.contains("safe.directory"),
        "el mensaje tiene que traer el problema y el remedio reales de git: {stderr}"
    );
    assert!(
        !stderr.contains("git init"),
        "no puede disfrazarse del mensaje generico de A2: {stderr}"
    );
    assert!(salida.stdout.is_empty());
}

#[test]
fn es_repo_git_distingue_las_dos_condiciones() {
    let sin = tempfile::tempdir().unwrap();
    assert!(!exo::gitx::es_repo_git(sin.path()).unwrap());
    let (con, _db) = kb_con_indice();
    assert!(exo::gitx::es_repo_git(con.path()).unwrap());
}

// `--is-inside-work-tree` diría true aquí, `git log` resolvería sin error y
// last_commit saldría vacío para TODAS las notas, en silencio. Es el caso
// real de un $HOME con los dotfiles versionados.
#[test]
fn una_kb_anidada_en_un_repo_ajeno_no_cuenta_como_versionada() {
    let (repo, _db) = kb_con_indice();
    let anidada = repo.path().join("kb-dentro");
    std::fs::create_dir_all(&anidada).unwrap();
    assert!(!exo::gitx::es_repo_git(&anidada).unwrap());
}

#[test]
fn el_envelope_lleva_command_targets_y_schema_version_2() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "5"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();

    assert!(
        salida.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "targets");
    assert_eq!(v["data"]["topic"], "alpha");
    let c = &v["data"]["candidates"][0];
    assert_eq!(c["permalink"], "kb/log/alpha");
    assert_eq!(c["tier"], "stable");
    assert_eq!(c["last_commit"], "2026-07-01T10:00:00+02:00");
    assert!(c["size_bytes"].as_i64().unwrap() > 0);
}

// Las claves de data van en inglés (D8) y las colecciones vacías serializan
// como `[]`, nunca como `null`: un consumidor que haga `.candidates[]` con jq
// no puede encontrarse un null.
#[test]
fn sin_candidatas_el_array_es_vacio_no_nulo() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("zzz-inexistente")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert!(v["data"]["candidates"].is_array());
    assert_eq!(v["data"]["candidates"].as_array().unwrap().len(), 0);
}

#[test]
fn un_limite_cero_falla_sin_ensuciar_stdout() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json", "--limit", "0"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(
        salida.stdout.is_empty(),
        "stdout tiene que quedar limpio ante error"
    );
}

#[test]
fn un_tema_vacio_falla() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("   ")
        .output()
        .unwrap();
    assert!(!salida.status.success());
}

#[test]
fn la_salida_humana_nombra_el_permalink() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["targets"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("kb/log/alpha"), "salida: {texto}");
}

// Una DB inexistente no debe crearse como efecto colateral de un typo en
// --db: los comandos de solo lectura comprueban antes de abrir.
#[test]
fn una_db_inexistente_falla_y_no_se_crea() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("no-existe.db");
    let salida = Command::new(bin())
        .args(["targets", "--json"])
        .arg("--db")
        .arg(&db)
        .arg("--kb")
        .arg(dir.path())
        .arg("alpha")
        .output()
        .unwrap();
    assert!(!salida.status.success());
    assert!(!db.exists(), "un typo en --db no puede crear el fichero");
}
