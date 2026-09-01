//! Superficie CLI v1.0: los flags largos están en inglés y los españoles
//! siguen parseando como alias oculto durante la ventana de migración.
//!
//! Se prueba contra el BINARIO, no contra clap por unidad: lo que se afirma
//! es "esta línea de comandos funciona", y eso solo lo demuestra ejecutarla.
//!
//! Criterio de "el flag existe" sin necesitar índice ni modelo: clap sale con
//! **exit 2** y `unexpected argument` cuando un flag no existe, y con
//! cualquier otro error (config ausente, DB ausente) cuando sí existe y el
//! comando llega a correr. Así el test no depende del estado de la máquina.

use std::process::Command;

fn bin() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    p.push(if cfg!(windows) { "exo.exe" } else { "exo" });
    p
}

/// Corre el binario con un `EXO_CONFIG` inexistente y devuelve
/// `(exit_code, stderr)`. El comando fallará —no hay config— pero el PARSEO
/// de argumentos ocurre antes, que es lo único que aquí se mide.
fn corre(args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(bin())
        .args(args)
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
        .env_remove("EXO_DB")
        .env_remove("EXO_KB")
        .output()
        .expect("correr el binario");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn acepta_el_flag(args: &[&str]) -> bool {
    let (code, err) = corre(args);
    !(code == Some(2)
        || err.contains("unexpected argument")
        || err.contains("argumento inesperado"))
}

#[test]
fn los_flags_ingleses_existen() {
    assert!(
        acepta_el_flag(&["search", "--limit", "3", "q"]),
        "search --limit"
    );
    assert!(
        acepta_el_flag(&["search", "--min-similarity", "0.4", "q"]),
        "search --min-similarity"
    );
    assert!(
        acepta_el_flag(&["search", "--fts-scale", "0.6", "q"]),
        "search --fts-scale"
    );
    assert!(
        acepta_el_flag(&["recall", "--limit", "3"]),
        "recall --limit"
    );
    assert!(acepta_el_flag(&["recall", "--content"]), "recall --content");
    assert!(
        acepta_el_flag(&["recall", "--note", "x/y"]),
        "recall --note"
    );
    assert!(acepta_el_flag(&["recall", "--refresh"]), "recall --refresh");
    assert!(
        acepta_el_flag(&["recall", "--min-similarity", "0.4"]),
        "recall --min-similarity"
    );
    assert!(
        acepta_el_flag(&["write", "new", "--dir", "d", "--title", "T", "--from", "-"]),
        "write new --title"
    );
    assert!(
        acepta_el_flag(&["write", "append", "--from", "-", "--create", "p"]),
        "write append --create"
    );
}

#[test]
fn los_flags_espanoles_siguen_parseando_como_alias() {
    // La ventana de migración: un script viejo cacheado no debe morir con
    // "unexpected argument" a mitad de un hook. Los diez pares, no una
    // muestra: un alias borrado o mal escrito en cualquiera de los diez
    // reproduce ese fallo, y solo se detecta probándolos todos.
    assert!(
        acepta_el_flag(&["write", "new", "--dir", "d", "--titulo", "T", "--from", "-"]),
        "alias --titulo"
    );
    assert!(
        acepta_el_flag(&["write", "append", "--from", "-", "--crea", "p"]),
        "alias --crea"
    );
    assert!(
        acepta_el_flag(&["search", "--limite", "3", "q"]),
        "alias --limite (search)"
    );
    assert!(
        acepta_el_flag(&["search", "--min-similitud", "0.4", "q"]),
        "alias --min-similitud (search)"
    );
    assert!(
        acepta_el_flag(&["search", "--escala-fts", "0.6", "q"]),
        "alias --escala-fts"
    );
    assert!(
        acepta_el_flag(&["recall", "--limite", "3"]),
        "alias --limite (recall)"
    );
    assert!(
        acepta_el_flag(&["recall", "--min-similitud", "0.4"]),
        "alias --min-similitud (recall)"
    );
    assert!(
        acepta_el_flag(&["recall", "--contenido", "--nota", "x/y"]),
        "alias --contenido/--nota"
    );
    assert!(
        acepta_el_flag(&["recall", "--refresca"]),
        "alias --refresca"
    );
}

/// `--help` del subcomando dado, como texto.
fn help_de(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        // Mismo valor literal que `corre()`: clap resuelve `--help` antes de
        // llegar a leer la config, así que hoy pasa incluso sin esto — pero
        // heredar el `EXO_CONFIG` del proceso que corre los tests no está
        // aislado, solo pasa por el orden de evaluación de clap. Fijarlo
        // explícito hace la hermeticidad real, no accidental.
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
        .output()
        .expect("correr --help");
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// `flag` aparece como TOKEN completo en el help, no como substring de otro
/// flag más largo: `--crea` es substring literal de `--create`, así que un
/// `help.contains("--crea")` daría positivo aunque `--crea` no esté — justo
/// la clase de falso positivo/negativo que esta suite existe para atajar.
fn help_contiene_flag(help: &str, flag: &str) -> bool {
    help.split(|c: char| c.is_whitespace() || c == ',')
        .any(|tok| tok == flag)
}

#[test]
fn el_help_solo_documenta_los_ingleses() {
    // Un alias VISIBLE consagraría el nombre español; el objetivo es que
    // desaparezca de la documentación hoy y del código en 1.1. Los diez
    // pares, cada uno en el --help del subcomando que lo declara: un
    // `visible_alias` colado en cualquiera de los diez pasaría inadvertido
    // si solo se mirasen dos.
    let help_write_new = help_de(&["write", "new", "--help"]);
    assert!(
        help_contiene_flag(&help_write_new, "--title"),
        "write new --help no muestra --title:\n{help_write_new}"
    );
    assert!(
        !help_contiene_flag(&help_write_new, "--titulo"),
        "write new --help sigue mostrando --titulo:\n{help_write_new}"
    );

    let help_write_append = help_de(&["write", "append", "--help"]);
    assert!(
        help_contiene_flag(&help_write_append, "--create"),
        "write append --help no muestra --create:\n{help_write_append}"
    );
    assert!(
        !help_contiene_flag(&help_write_append, "--crea"),
        "write append --help sigue mostrando --crea:\n{help_write_append}"
    );

    let help_search = help_de(&["search", "--help"]);
    for (nuevo, viejo) in [
        ("--limit", "--limite"),
        ("--min-similarity", "--min-similitud"),
        ("--fts-scale", "--escala-fts"),
    ] {
        assert!(
            help_contiene_flag(&help_search, nuevo),
            "search --help no muestra {nuevo}:\n{help_search}"
        );
        assert!(
            !help_contiene_flag(&help_search, viejo),
            "search --help sigue mostrando {viejo}:\n{help_search}"
        );
    }

    let help_recall = help_de(&["recall", "--help"]);
    for (nuevo, viejo) in [
        ("--limit", "--limite"),
        ("--min-similarity", "--min-similitud"),
        ("--content", "--contenido"),
        ("--note", "--nota"),
        ("--refresh", "--refresca"),
    ] {
        assert!(
            help_contiene_flag(&help_recall, nuevo),
            "recall --help no muestra {nuevo}:\n{help_recall}"
        );
        assert!(
            !help_contiene_flag(&help_recall, viejo),
            "recall --help sigue mostrando {viejo}:\n{help_recall}"
        );
    }
}

#[test]
fn los_flags_ya_ingleses_no_se_han_movido() {
    // Guardarraíl contra el churn: renombrar de más también rompe. Como el
    // mecanismo es `alias` y no un rename puro, comprobar que el flag viejo
    // sigue PARSEANDO no basta — un `--bonus` renombrado a `--weight` con
    // `alias = "bonus"` seguiría parseando y este test seguiría verde sin
    // detectar el churn. Lo que hay que mirar es el nombre CANÓNICO en el
    // `--help`, el mismo criterio que usa el test del help para los
    // migrados, en sentido inverso.
    let help_search = help_de(&["search", "--help"]);
    assert!(
        help_contiene_flag(&help_search, "--bonus"),
        "search --help ya no muestra --bonus:\n{help_search}"
    );
    assert!(
        help_contiene_flag(&help_search, "--type"),
        "search --help ya no muestra --type:\n{help_search}"
    );

    let help_recall = help_de(&["recall", "--help"]);
    assert!(
        help_contiene_flag(&help_recall, "--cap-bytes"),
        "recall --help ya no muestra --cap-bytes:\n{help_recall}"
    );
}
