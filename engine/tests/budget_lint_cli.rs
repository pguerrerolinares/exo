use std::fs;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

fn kb(ficheros: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (rel, contenido) in ficheros {
        let ruta = dir.path().join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, contenido).unwrap();
    }
    dir
}

/// DB con el schema y las notas dadas ya indexadas. **Indexadas de verdad**:
/// desde la Task 9, una KB con notas en disco y el índice vacío emite
/// `index_stale`, así que un fixture con la DB vacía no puede aseverar "limpio".
///
/// **`dir` NO es la raíz de la KB**: corrección sobre el brief, que colocaba
/// `index.db` (y sus `-shm`/`-wal` de WAL) dentro del árbol lintado. `lint`
/// gatea por `ficheros_en_raiz`, que a propósito NO consulta la lista de
/// exclusión (doc-comment de `lint.rs`: "es correcto" que no lo haga) — un
/// `index.db` en la raíz de la KB se reporta como `root_file` igual que
/// cualquier otro fichero suelto, y el caso "limpio" del test de abajo no
/// podía salir jamás limpio tal como estaba escrito.
fn db_con(dir: &std::path::Path, notas: &[&str]) -> std::path::PathBuf {
    let db = dir.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    for rel in notas {
        conn.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?1, 'note', 0.0, NULL)",
            rusqlite::params![format!("kb/{rel}"), rel],
        )
        .unwrap();
    }
    db
}

#[test]
fn budget_limpio_sale_cero_con_el_envelope_completo() {
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
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
    assert_eq!(v["command"], "budget");
    assert!(v["data"]["tiers"].is_array());
    assert_eq!(v["data"]["offenders"].as_array().unwrap().len(), 0);
}

#[test]
fn budget_con_infractoras_sale_tres_y_emite_igual() {
    // El primer ejercicio de la decisión de exit 3: hallazgos != error. El
    // informe se emite ENTERO antes de gatear, porque quien lo consume necesita
    // saber QUÉ falló, no solo que falló.
    let grande = format!("---\ntier: core\n---\n{}", "x".repeat(9000));
    let dir = kb(&[("core/big.md", grande.as_str())]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(3), "hallazgos son 3, no 1");
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["offenders"][0]["path"], "core/big.md");
    // `.contains("budget")` a secas dejaba pasar un `GateFallido.comando`
    // mutado a "budgetz" (sigue conteniendo la subcadena "budget"): el
    // prefijo con los dos puntos de `Display` ("budget: ...") es lo que
    // ata el aserto al comando exacto, no a una subcadena suya.
    assert!(String::from_utf8_lossy(&salida.stderr).contains("rechazado: budget:"));
}

#[test]
fn el_aire_solo_nunca_saca_del_cero() {
    // stable a ras (12.000 de 12.500, objetivo de poda 10.869): aviso, no gate.
    let ras = format!("---\ntier: stable\n---\n{}", "x".repeat(11_950));
    let dir = kb(&[("s.md", ras.as_str())]);
    let salida = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success(), "el aire NUNCA gatea");
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["data"]["no_air"].as_array().unwrap().len(), 1);
}

#[test]
fn lint_con_hallazgos_sale_tres_y_lint_limpio_sale_cero() {
    // Dos notas y una arista entre ellas: si no, ambas serían huérfanas y el
    // caso "limpio" no existiría. Y las dos indexadas, o salta `index_stale`.
    let dir = kb(&[
        ("core/hub.md", "---\ntier: core\n---\n[[ok]]\n"),
        ("core/ok.md", "---\ntier: core\n---\nx\n"),
    ]);
    // La DB vive FUERA de la KB (ver el comentario de `db_con`): dentro,
    // `ficheros_en_raiz` la marcaría a ella misma como `root_file` y el caso
    // "limpio" de abajo jamás saldría limpio.
    let dir_db = tempfile::tempdir().unwrap();
    let db = db_con(dir_db.path(), &["core/hub.md", "core/ok.md"]);
    {
        let conn = exo::abre_db(&db).unwrap();
        conn.execute(
            "INSERT INTO aristas (origen, destino_texto, destino_permalink)
             VALUES ('kb/core/hub.md', 'ok', 'kb/core/ok.md')",
            [],
        )
        .unwrap();
    }
    let limpio = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        limpio.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&limpio.stdout),
        String::from_utf8_lossy(&limpio.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&limpio.stdout).unwrap();
    assert_eq!(v["command"], "lint");
    assert_eq!(v["data"]["ok"], true);

    fs::write(dir.path().join("suelto.txt"), "x").unwrap();
    let sucio = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert_eq!(sucio.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&sucio.stdout).unwrap();
    assert_eq!(v["data"]["ok"], false);
    assert_eq!(v["data"]["findings"][0]["type"], "root_file");
    // El brief no aseveraba el stderr de `lint` (a diferencia del de
    // `budget`, arriba): un `GateFallido.comando` mutado a "lintz" o a
    // "budget" pasaba en verde. Mismo motivo que el assert de `budget`.
    assert!(String::from_utf8_lossy(&sucio.stderr).contains("rechazado: lint:"));
}

#[test]
fn una_db_inexistente_es_error_uno_no_gate_tres() {
    // La distinción que justifica los dos códigos: "la KB está mal" (3) frente
    // a "el binario no pudo trabajar" (1).
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["lint", "--json"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(dir.path().join("no-existe.db"))
        .output()
        .unwrap();
    // Un índice que no existe no es "la KB está mal": es que el binario no
    // puede trabajar. Distinguirlo de `index_stale` —que sí es exit 3— es la
    // razón de tener dos códigos.
    assert_eq!(salida.status.code(), Some(1));
    assert!(salida.stdout.is_empty(), "un error no ensucia stdout");
}

#[test]
fn la_salida_humana_no_lleva_json() {
    let dir = kb(&[("core/ok.md", "---\ntier: core\n---\nx\n")]);
    let salida = Command::new(bin())
        .args(["budget"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(texto.contains("core"));
    assert!(!texto.trim_start().starts_with('{'));
}
