use std::path::Path;
use std::process::Command;

/// Recencia de `ruta_rel` (relativa a `kb`) según el último commit de git
/// que la tocó — JAMÁS mtime (§6.2 regla 2: ningún `created:` en frontmatter,
/// un clone fresco resetea mtimes). `None` si el fichero no tiene commits
/// (nuevo, aún no versionado) o si `git` falla por cualquier motivo — la
/// columna `notas.git_epoch` admite NULL para este caso, no es un error de
/// indexado.
pub fn git_epoch_de(kb: &Path, ruta_rel: &Path) -> Option<i64> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(kb)
        .arg("log")
        .arg("-1")
        .arg("--format=%ct")
        .arg("--")
        .arg(ruta_rel)
        .output()
        .ok()?;

    if !salida.status.success() {
        return None;
    }

    let texto = String::from_utf8(salida.stdout).ok()?;
    let texto = texto.trim();
    if texto.is_empty() {
        return None;
    }
    texto.parse::<i64>().ok()
}
