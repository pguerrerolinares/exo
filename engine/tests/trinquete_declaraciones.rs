//! Task 9 — `comprueba` y el informe entero: las tres familias sobre
//! declaraciones, la abstención de punta a punta y el orden final.
//!
//! Mismo aislamiento de git que `trinquete_abstencion.rs` y
//! `trinquete_aire.rs`.

use exo::presupuesto::NOMINALES;
use exo::trinquete::{self, Declarada, Sellos, Tipo};
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn sellos(pares: &[(&str, i64)]) -> Sellos {
    pares.iter().map(|(r, t)| (r.to_string(), *t)).collect()
}

/// Instala el ancla (sello vacío commiteado): toda declaración que venga
/// después es una decisión nueva, no la corrida de activación.
fn ancla_instalada(dir: &Path, cfg: &Path) {
    std::fs::write(
        dir.join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&Sellos::new()),
    )
    .unwrap();
    commit_todo(dir, cfg, "instala el trinquete, sin sellos");
}

fn escribe_actual(kb: &Path, pares: &[(&str, i64)]) {
    trinquete::escribe_sellos(kb, &sellos(pares)).unwrap();
}

fn declarada(ruta: &str, tier: &str, max: i64, tier_presupuesto: i64, tamano: i64) -> Declarada {
    Declarada {
        ruta: ruta.to_string(),
        tier: tier.to_string(),
        max,
        tier_presupuesto,
        tamano,
    }
}

#[test]
fn un_waiver_por_encima_de_su_sello_falla() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 5000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "a.md ya sellado");
    escribe_actual(dir.path(), &[("a.md", 5000)]); // sin tocar

    let declaradas = [declarada("a.md", "core", 6000, NOMINALES.core, 4000)];
    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();

    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "a.md")
        .expect("el waiver por encima de su sello debe romper");
    assert_eq!(h.tipo, Tipo::SobreSello);
    assert_eq!(h.era, 5000);
    assert_eq!(h.ahora, 6000);
    assert_eq!(h.limite, 5000);
    assert!(informe.fallido());
}

#[test]
fn un_waiver_igual_o_por_debajo_pasa() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 5000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "a.md ya sellado");
    escribe_actual(dir.path(), &[("a.md", 5000)]);

    // Igual a su sello: no rebasa el trinquete.
    let declaradas = [declarada("a.md", "core", 5000, NOMINALES.core, 4000)];
    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();
    assert!(
        informe.hallazgos.is_empty(),
        "igual a su sello debe pasar: {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}

#[test]
fn una_primera_declaracion_se_capa_a_dos_veces_el_tier() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);
    // Sin sello en absoluto todavía: la declaración no está sellada.

    let declaradas = [declarada(
        "core/new.md",
        "core",
        20000,
        NOMINALES.core,
        5000,
    )];
    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();

    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "core/new.md")
        .expect("la primera declaración debe capar a 2x el tier");
    assert_eq!(h.tipo, Tipo::PrimeraMuyAlta);
    assert_eq!(h.ahora, 20000);
    assert_eq!(h.limite, 17000); // 8500 * 2
    assert!(informe.fallido());
}

#[test]
fn un_waiver_en_tier_log_es_inerte_no_fallo() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);
    // Sin sello: waiver puro en un tier sin presupuesto.

    let declaradas = [declarada("log/x.md", "log", 99999, NOMINALES.log, 5000)];
    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();

    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "log/x.md")
        .expect("el waiver en log debe reportarse, aunque sea inerte");
    assert_eq!(h.tipo, Tipo::WaiverLogInerte);
    assert_eq!(h.ahora, 99999);
    assert!(!informe.fallido(), "un waiver inerte no rompe el gate");
}

#[test]
fn una_nota_sellada_que_escapa_a_log_se_marca() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("x.md", 9000)])),
    )
    .unwrap();
    // La nota real, en tier `log` (sin presupuesto) y sin waiver: se
    // reclasificó para escapar del gate. El sello es la prueba de que tuvo
    // techo.
    std::fs::write(dir.path().join("x.md"), "---\ntier: log\n---\ncontenido\n").unwrap();
    commit_todo(dir.path(), &cfg, "x.md sellado, ya en tier log");
    escribe_actual(dir.path(), &[("x.md", 9000)]); // sin tocar

    // declaradas vacío: ya no declara kbx_budget_max.
    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();

    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "x.md")
        .expect("la nota escapada a log debe marcarse");
    assert_eq!(h.tipo, Tipo::SelladaEscapadaDeTier);
    assert_eq!(h.era, 9000);
    assert!(informe.fallido());
}

#[test]
fn sin_git_el_informe_no_se_aplica() {
    let dir = tempfile::tempdir().unwrap();
    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(!informe.aplicado);
    assert!(informe.razon.is_some());
    assert!(informe.hallazgos.is_empty());
    assert!(
        !informe.fallido(),
        "la abstención es información, no fallo — exit 0 aguas arriba"
    );
}

#[test]
fn los_hallazgos_salen_ordenados_por_ruta() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("z.md", 5000), ("a.md", 5000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "z.md y a.md ya sellados");
    escribe_actual(dir.path(), &[("z.md", 5000), ("a.md", 5000)]);

    let declaradas = [
        declarada("z.md", "core", 6000, NOMINALES.core, 4000),
        declarada("a.md", "core", 6000, NOMINALES.core, 4000),
    ];
    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();

    let rutas: Vec<&str> = informe.hallazgos.iter().map(|h| h.ruta.as_str()).collect();
    assert_eq!(rutas, vec!["a.md", "z.md"]);
}
