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
fn budget_gatea_por_notier_sin_infractoras_y_el_detalle_no_cruza_numeros() {
    // Important 1: `informe.excedido()` mira infractoras Y notier. Sin este
    // test, cambiar el gate por `!informe.infractoras.is_empty()` (que borra
    // el gate de notier en silencio) dejaba los seis tests del CLI en verde:
    // una KB con solo notas sin tier legal salía 0 en vez de 3.
    let dir_notier = kb(&[("inventado.md", "---\ntier: inventado\n---\nx\n")]);
    let solo_notier = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir_notier.path())
        .output()
        .unwrap();
    assert_eq!(
        solo_notier.status.code(),
        Some(3),
        "notier gatea sin infractoras"
    );
    let v: serde_json::Value = serde_json::from_slice(&solo_notier.stdout).unwrap();
    assert_eq!(v["data"]["offenders"].as_array().unwrap().len(), 0);
    assert_eq!(v["data"]["notier"][0], "inventado.md");

    // Important 2: el detalle del mensaje ("N sobre presupuesto, M sin tier
    // legal") no estaba falsado — cruzar `infractoras.len()` y `notier.len()`
    // en el `format!` sobrevivía. Conteos asimétricos (1 infractora, 2
    // notier) a propósito: con conteos iguales el cruce produce el mismo
    // texto y el test no lo vería.
    let grande = format!("---\ntier: core\n---\n{}", "x".repeat(9000));
    let dir_mixto = kb(&[
        ("core/big.md", grande.as_str()),
        ("inventado.md", "---\ntier: inventado\n---\nx\n"),
        ("otro.md", "---\ntier: otro\n---\nx\n"),
    ]);
    let mixto = Command::new(bin())
        .args(["budget", "--json"])
        .arg("--kb")
        .arg(dir_mixto.path())
        .output()
        .unwrap();
    assert_eq!(mixto.status.code(), Some(3));
    assert_eq!(
        String::from_utf8_lossy(&mixto.stderr).trim(),
        "rechazado: budget: 1 nota(s) sobre presupuesto, 2 sin tier legal"
    );
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
    //
    // El detalle completo, no solo el prefijo: `InformeLint` tiene un
    // `waived` además de `hallazgos`, y poner `waived.len()` donde va
    // `hallazgos.len()` en el `format!` sobrevivía si solo se miraba el
    // prefijo. Aquí `waived` está vacío y `hallazgos` tiene uno: el número
    // exacto ata el mensaje al campo correcto.
    assert_eq!(
        String::from_utf8_lossy(&sucio.stderr).trim(),
        "rechazado: lint: 1 hallazgo(s)"
    );
}

#[test]
fn lint_limpio_igual_imprime_los_waived_en_modo_humano() {
    // Important 2 de la code review: `lint_cmd` en modo humano imprimía "ok" y
    // se callaba `informe.waived` — a diferencia de `budget_cmd`, que sí los
    // imprime incluso en verde. Ese comportamiento es a propósito, no un
    // accidente: `emitDoctorReport` en el `kbx` original (cmd/kbx/main.go)
    // dice literalmente "Waived items surface even on a clean (ok:true) run:
    // they are the human-facing audit surface for recognized exceptions
    // (spec §10)". Este test fija que `lint` en Rust respeta el mismo
    // contrato: el caso limpio (hallazgos vacíos, exit 0, "ok") tiene que
    // llevar la línea `waived` igualmente.
    //
    // `core/grande.md` rebasa el nominal del tier core (8500B) pero declara
    // `kbx_budget_max: 20000` por encima de su tamaño real: cae en
    // `Clase::Waived` (presupuesto.rs::clasifica), no en infractora. Enlazada
    // desde `core/hub.md` para que `orphan` no la marque — el mismo patrón de
    // arista real que usa `lint_con_hallazgos_sale_tres_y_lint_limpio_sale_cero`
    // arriba, para no mezclar ruido de `orphan`/`index_stale` en un test que
    // solo le interesa el waiver de budget.
    let grande = format!(
        "---\ntier: core\nkbx_budget_max: 20000\n---\n{}",
        "x".repeat(9000)
    );
    let dir = kb(&[
        ("core/hub.md", "---\ntier: core\n---\n[[grande]]\n"),
        ("core/grande.md", grande.as_str()),
    ]);
    let dir_db = tempfile::tempdir().unwrap();
    let db = db_con(dir_db.path(), &["core/hub.md", "core/grande.md"]);
    {
        let conn = exo::abre_db(&db).unwrap();
        conn.execute(
            "INSERT INTO aristas (origen, destino_texto, destino_permalink)
             VALUES ('kb/core/hub.md', 'grande', 'kb/core/grande.md')",
            [],
        )
        .unwrap();
    }

    let salida = Command::new(bin())
        .args(["lint"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    let texto = String::from_utf8_lossy(&salida.stdout);
    let lineas: Vec<&str> = texto.trim_end().lines().collect();
    assert_eq!(
        lineas,
        vec![
            "ok",
            "waived\tbudget_exceeded\tcore/grande.md\t9041B ≤ 20000B (waived: kbx_budget_max)",
        ],
        "el caso limpio tiene que imprimir 'ok' Y la línea waived, no callarla"
    );
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
fn la_salida_humana_de_budget_no_lleva_json() {
    // Important 3b: `texto.contains("core")` a secas sobrevivía a reordenar
    // los campos del `println!` de la fila de tier (siguen conteniendo la
    // subcadena "core"). Cada assert de abajo ata TODAS las columnas a sus
    // valores, en el orden exacto que imprime `budget_cmd`: reordenar
    // cualquier campo de cualquiera de las cuatro líneas lo tumba.
    let grande = format!("---\ntier: core\n---\n{}", "x".repeat(9000));
    let ras = format!("---\ntier: stable\n---\n{}", "x".repeat(11_950));
    let dir = kb(&[
        ("core/big.md", grande.as_str()),
        ("s.md", ras.as_str()),
        ("inventado.md", "---\ntier: inventado\n---\nx\n"),
    ]);
    let salida = Command::new(bin())
        .args(["budget"])
        .arg("--kb")
        .arg(dir.path())
        .output()
        .unwrap();
    assert_eq!(salida.status.code(), Some(3));
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(!texto.trim_start().starts_with('{'));
    assert!(texto.contains("core\tnotas=1\tbytes=9019\tpresupuesto=8500\tdelta=519"));
    assert!(texto.contains("offender: core/big.md (core) 9019/8500 bytes"));
    assert!(texto.contains(
        "no-air: s.md (stable) 11971/12500 bytes a ras — poda a 10869 para el 15% de aire"
    ));
    assert!(texto.contains("notier: inventado.md"));
}

#[test]
fn la_salida_humana_de_lint_no_lleva_json() {
    // Important 3a: `la_salida_humana_no_lleva_json` solo cubría `budget`.
    // Sin este test, reordenar el `println!("{}\t{}\t{}", h.tipo, h.ruta,
    // h.detalle)` de `lint_cmd`, o cambiar el `"ok"` del caso limpio, no lo
    // detecta nadie.
    let dir = kb(&[
        ("core/hub.md", "---\ntier: core\n---\n[[ok]]\n"),
        ("core/ok.md", "---\ntier: core\n---\nx\n"),
    ]);
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
        .args(["lint"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert!(limpio.status.success());
    assert_eq!(String::from_utf8_lossy(&limpio.stdout).trim(), "ok");

    fs::write(dir.path().join("suelto.txt"), "x").unwrap();
    let sucio = Command::new(bin())
        .args(["lint"])
        .arg("--kb")
        .arg(dir.path())
        .arg("--db")
        .arg(&db)
        .output()
        .unwrap();
    assert_eq!(sucio.status.code(), Some(3));
    assert_eq!(
        String::from_utf8_lossy(&sucio.stdout).trim(),
        "root_file\tsuelto.txt\tfichero no-nota en la raíz"
    );
}
