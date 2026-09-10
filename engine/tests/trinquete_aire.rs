//! Task 8 — la guarda de aire: `no-air`, `no-air-debt` y `born-too-big`.
//!
//! La guarda vive DENTRO de `comprueba_contra` (Task 9): no hay un `checkAir`
//! aislado que llamar, así que estos tests pasan por la única puerta pública,
//! `exo::trinquete::comprueba`, igual que el resto de tests de integración de
//! este módulo. Mismo aislamiento de git que `trinquete_abstencion.rs`.

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

fn nota_de(kb: &Path, ruta: &str, bytes: usize) {
    let absoluta = kb.join(ruta);
    if let Some(padre) = absoluta.parent() {
        std::fs::create_dir_all(padre).unwrap();
    }
    std::fs::write(&absoluta, "x".repeat(bytes)).unwrap();
}

#[test]
fn un_sello_fresco_con_aire_suficiente_esta_limpio() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    nota_de(dir.path(), "a.md", 10000);
    escribe_actual(dir.path(), &[("a.md", 11500)]); // techo_minimo(10000) == 11500

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(!informe.hallazgos.iter().any(|h| h.ruta == "a.md"));
    assert!(!informe.fallido());
}

#[test]
fn un_sello_fresco_a_un_byte_del_aire_falla() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    nota_de(dir.path(), "a.md", 10000);
    escribe_actual(dir.path(), &[("a.md", 11499)]); // un byte por debajo del borde

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "a.md")
        .expect("a.md debe tener hallazgo");
    assert_eq!(h.tipo, Tipo::SinAire);
    assert_eq!(h.ahora, 11499);
    assert_eq!(h.limite, 11500);
    assert!(informe.fallido());
}

#[test]
fn un_sello_bajado_sin_aire_falla() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 20000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello alto ya commiteado");

    nota_de(dir.path(), "a.md", 10000);
    escribe_actual(dir.path(), &[("a.md", 11000)]); // baja, y sigue sin aire (< 11500)

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "a.md")
        .expect("a.md debe tener hallazgo");
    assert_eq!(h.tipo, Tipo::SinAire);
    assert_eq!(h.ahora, 11000);
    assert_eq!(h.limite, 11500);
    assert!(informe.fallido());
}

// El test que sostiene la instalabilidad: los 11 sellos reales de la KB no
// tienen 15% de aire, y si esto rompiera el gate el trinquete se
// desinstalaría solo el día que se active.
#[test]
fn un_sello_intacto_sin_aire_es_deuda_no_fallo() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 11000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello sin aire ya commiteado");

    nota_de(dir.path(), "a.md", 10000);
    escribe_actual(dir.path(), &[("a.md", 11000)]); // sin tocar

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "a.md")
        .expect("a.md debe tener hallazgo de deuda");
    assert_eq!(h.tipo, Tipo::DeudaSinAire);
    assert_eq!(h.ahora, 11000);
    assert_eq!(h.limite, 11500);
    assert!(!informe.fallido(), "la deuda no rompe el gate");
}

#[test]
fn un_techo_subido_no_se_etiqueta_como_deuda() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 5000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello bajo ya commiteado");

    nota_de(dir.path(), "a.md", 10000);
    escribe_actual(dir.path(), &[("a.md", 9000)]); // sube, y sigue sin aire

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    let de_a = informe.hallazgos.iter().find(|h| h.ruta == "a.md");
    assert_eq!(
        de_a.map(|h| h.tipo),
        Some(Tipo::SelloSubido),
        "solo SelloSubido, nunca además DeudaSinAire ni SinAire"
    );
    assert!(informe.fallido());
}

#[test]
fn la_corrida_de_activacion_esta_exenta_del_aire() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    // HEAD existe (hay historia) pero SIN sello commiteado: es la corrida de
    // activación, no la abstención.
    std::fs::write(dir.path().join("otro.md"), "contenido\n").unwrap();
    commit_todo(dir.path(), &cfg, "sin sello aun");

    nota_de(dir.path(), "a.md", 44000);
    escribe_actual(dir.path(), &[("a.md", 44500)]); // muy lejos del aire exigido

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(!informe.hallazgos.iter().any(|h| h.ruta == "a.md"));
    assert!(!informe.fallido());
}

#[test]
fn un_sello_renombrado_sin_aire_esta_exento() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(dir.path().join("old.md"), "vieja\n").unwrap();
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("old.md", 20000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "old.md y su sello, commiteados");

    // "Rename": old.md desaparece de los sellos actuales, new.md aparece con
    // el mismo techo (lo conserva) y sin aire (techo_minimo(20000) == 23000).
    nota_de(dir.path(), "new.md", 20000);
    escribe_actual(dir.path(), &[("new.md", 20000)]);

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        informe.hallazgos.is_empty(),
        "el rename absuelve el sello retirado Y exime al fresco de la guarda de aire: {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}

#[test]
fn una_primera_declaracion_en_zona_muerta_dice_parte_la_nota() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    // Fixture literal del bug del sello huérfano (Task 7): 44.000 B en core,
    // muy por encima de lo que el cap de 2x permite (17.000).
    escribe_actual(dir.path(), &[("core/attack.md", 45000)]);
    let declaradas = [Declarada {
        ruta: "core/attack.md".to_string(),
        tier: "core".to_string(),
        max: 45000,
        tier_presupuesto: NOMINALES.core,
        tamano: 44000,
    }];

    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();
    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "core/attack.md")
        .expect("debe salir NaceDemasiadoGrande");
    assert_eq!(h.tipo, Tipo::NaceDemasiadoGrande);
    assert_eq!(h.ahora, 44000);
    assert_eq!(h.limite, NOMINALES.core * 2);
    // Una nota NaceDemasiadoGrande no recibe además los checks de declaración
    // (SobreSello / PrimeraMuyAlta): un solo hallazgo, el que aconseja bien.
    assert_eq!(
        informe
            .hallazgos
            .iter()
            .filter(|h| h.ruta == "core/attack.md")
            .count(),
        1
    );
    assert!(informe.fallido());
}

#[test]
fn el_borde_de_la_zona_muerta_esta_limpio() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    // 14.782 B en core: techo_minimo(14782) == 17000 == tier*2. El único
    // techo legal (17000) satisface las dos guardas a la vez.
    escribe_actual(dir.path(), &[("core/edge.md", 17000)]);
    let declaradas = [Declarada {
        ruta: "core/edge.md".to_string(),
        tier: "core".to_string(),
        max: 17000,
        tier_presupuesto: NOMINALES.core,
        tamano: 14782,
    }];

    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();
    assert!(
        !informe.hallazgos.iter().any(|h| h.ruta == "core/edge.md"),
        "el borde de la zona muerta debe estar limpio: {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}

#[test]
fn un_sello_huerfano_sin_fichero_se_salta() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    // "ghost.md" no existe en el disco: ni se declara, ni tiene fichero.
    escribe_actual(dir.path(), &[("ghost.md", 100)]);

    let informe = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        !informe.hallazgos.iter().any(|h| h.ruta == "ghost.md"),
        "un sello sin fichero no inventa un tamaño, se salta: {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}

#[test]
fn una_nota_log_sellada_sigue_necesitando_aire() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("log/x.md", 5000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello de una nota log, ya commiteado");

    // `log` tiene presupuesto 0 (ilimitado): eso exime a la nota de `budget`,
    // no de la guarda de aire, que es un gate distinto sobre el SELLO.
    escribe_actual(dir.path(), &[("log/x.md", 4000)]); // baja, sin aire
    let declaradas = [Declarada {
        ruta: "log/x.md".to_string(),
        tier: "log".to_string(),
        max: 4000,
        tier_presupuesto: NOMINALES.log,
        tamano: 4000,
    }];

    let informe = trinquete::comprueba(dir.path(), &declaradas, NOMINALES).unwrap();
    let h = informe
        .hallazgos
        .iter()
        .find(|h| h.ruta == "log/x.md")
        .expect("log/x.md sigue necesitando aire");
    assert_eq!(h.tipo, Tipo::SinAire);
    assert_eq!(h.ahora, 4000);
    assert_eq!(h.limite, 4600); // techo_minimo(4000)
    assert!(informe.fallido());
}
