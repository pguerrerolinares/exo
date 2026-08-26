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
    // "unexpected argument" a mitad de un hook.
    assert!(
        acepta_el_flag(&["search", "--limite", "3", "q"]),
        "alias --limite"
    );
    assert!(
        acepta_el_flag(&["recall", "--contenido", "--nota", "x/y"]),
        "alias --contenido/--nota"
    );
    assert!(
        acepta_el_flag(&["recall", "--refresca"]),
        "alias --refresca"
    );
    assert!(
        acepta_el_flag(&["search", "--min-similitud", "0.4", "q"]),
        "alias --min-similitud"
    );
}

#[test]
fn el_help_solo_documenta_los_ingleses() {
    // Un alias VISIBLE consagraría el nombre español; el objetivo es que
    // desaparezca de la documentación hoy y del código en 1.1.
    let out = Command::new(bin())
        .args(["recall", "--help"])
        .output()
        .expect("correr --help");
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.contains("--limit"),
        "el help no muestra --limit:\n{help}"
    );
    assert!(
        help.contains("--refresh"),
        "el help no muestra --refresh:\n{help}"
    );
    assert!(
        !help.contains("--limite"),
        "el help sigue mostrando --limite:\n{help}"
    );
    assert!(
        !help.contains("--refresca"),
        "el help sigue mostrando --refresca:\n{help}"
    );
}

#[test]
fn los_flags_ya_ingleses_no_se_han_movido() {
    // Guardarraíl contra el churn: renombrar de más también rompe.
    assert!(
        acepta_el_flag(&["recall", "--cap-bytes", "2048"]),
        "--cap-bytes"
    );
    assert!(
        acepta_el_flag(&["search", "--bonus", "0.5", "q"]),
        "--bonus"
    );
    assert!(acepta_el_flag(&["search", "--type", "fts", "q"]), "--type");
}
