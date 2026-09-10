//! Task 10 — `--staged`: el trinquete juzga el índice, no el árbol de trabajo.
//!
//! Mismo aislamiento de git que `trinquete_aire.rs`: `GIT_CONFIG_GLOBAL`/
//! `GIT_CONFIG_SYSTEM` apuntan a un fichero vacío del tempdir, e identidad de
//! autor/committer fija por env var.

use exo::presupuesto::NOMINALES;
use exo::trinquete::{self, Sellos};
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

fn stage_todo(dir: &Path, cfg: &Path) {
    git_aislado(dir, cfg, &["add", "."]);
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

fn escribe_sello_en_disco(kb: &Path, pares: &[(&str, i64)]) {
    trinquete::escribe_sellos(kb, &sellos(pares)).unwrap();
}

fn nota_de(kb: &Path, ruta: &str, bytes: usize) {
    let absoluta = kb.join(ruta);
    if let Some(padre) = absoluta.parent() {
        std::fs::create_dir_all(padre).unwrap();
    }
    std::fs::write(&absoluta, "x".repeat(bytes)).unwrap();
}

// El punto que más importa de la tarea: el índice, no el disco.

#[test]
fn el_modo_staged_ve_el_indice_no_el_working_tree() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 100)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello bajo commiteado");

    // Stagear la subida de techo...
    escribe_sello_en_disco(dir.path(), &[("a.md", 200)]);
    stage_todo(dir.path(), &cfg);
    // ...y luego restaurar el disco al valor commiteado: el working tree ya
    // no muestra la subida, pero el índice sí la lleva.
    escribe_sello_en_disco(dir.path(), &[("a.md", 100)]);

    let disco = trinquete::carga(dir.path()).unwrap();
    let indice = trinquete::carga_staged(dir.path()).unwrap();
    assert_eq!(
        disco.get("a.md"),
        Some(&100),
        "carga miente: ve el disco restaurado"
    );
    assert_eq!(
        indice.get("a.md"),
        Some(&200),
        "carga_staged dice la verdad: ve la subida que sigue en el índice"
    );
}

#[test]
fn staged_caza_la_subida_que_el_working_tree_esconde() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    nota_de(dir.path(), "a.md", 20000);
    std::fs::write(
        dir.path().join(trinquete::FICHERO_SELLO),
        trinquete::serializa_sellos(&sellos(&[("a.md", 21000)])),
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "sello bajo commiteado");

    escribe_sello_en_disco(dir.path(), &[("a.md", 30000)]);
    stage_todo(dir.path(), &cfg);
    escribe_sello_en_disco(dir.path(), &[("a.md", 21000)]); // restaura el disco

    let informe_disco = trinquete::comprueba(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        !informe_disco.fallido(),
        "comprueba (disco) debe pasar limpio: {:?}",
        informe_disco.hallazgos
    );

    let informe_staged = trinquete::comprueba_staged(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        informe_staged.fallido(),
        "comprueba_staged debe cazar la subida escondida en el índice"
    );
    assert!(
        informe_staged
            .hallazgos
            .iter()
            .any(|h| h.ruta == "a.md" && h.tipo == exo::trinquete::Tipo::SelloSubido)
    );
}

#[test]
fn staged_juzga_el_tamano_del_indice_no_el_del_disco() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    // En el índice la nota mide 10.000 B con techo 11.500 (aire exacto al
    // borde). En el disco, tras stagear, la nota crece a 15.000 B, muy por
    // encima de lo que el mismo techo tolera — pero comprueba_staged no debe
    // mirar el disco para el tamaño.
    nota_de(dir.path(), "a.md", 10000);
    escribe_sello_en_disco(dir.path(), &[("a.md", 11500)]);
    stage_todo(dir.path(), &cfg);
    nota_de(dir.path(), "a.md", 15000); // crece en disco DESPUÉS de stagear

    let informe = trinquete::comprueba_staged(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        !informe.hallazgos.iter().any(|h| h.ruta == "a.md"),
        "el tamaño debe salir del índice (10.000 B, con aire), no del disco (15.000 B): {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}

#[test]
fn sin_sello_staged_el_mapa_es_vacio() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    // Repo con historia pero sin nada staged: ni working tree ni índice
    // llevan `.kbx-ratchet.json`.
    std::fs::write(dir.path().join("otro.md"), "contenido\n").unwrap();
    commit_todo(dir.path(), &cfg, "sin sello");

    let indice = trinquete::carga_staged(dir.path()).unwrap();
    assert!(
        indice.is_empty(),
        "sin sello staged, el mapa debe ser vacío"
    );
}

#[test]
fn sin_sello_staged_en_un_directorio_sin_repo_git_el_mapa_es_vacio() {
    // El Go traga CUALQUIER error de `git show` (no solo "objeto ausente"):
    // aquí ni siquiera hay repo, así que git ni resuelve, y aun así
    // carga_staged no debe propagar error.
    let dir = tempfile::tempdir().unwrap();
    let indice = trinquete::carga_staged(dir.path()).unwrap();
    assert!(indice.is_empty());
}

// recolecta_staged: las declaraciones salen del índice, no del disco.

#[test]
fn recolecta_staged_lee_declaraciones_del_indice_no_del_disco() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    std::fs::create_dir_all(dir.path().join("core")).unwrap();
    std::fs::write(
        dir.path().join("core/a.md"),
        "---\ntier: core\nkbx_budget_max: 12000\n---\ncontenido staged\n",
    )
    .unwrap();
    stage_todo(dir.path(), &cfg);
    // Tras stagear, el disco cambia: el override sube. recolecta_staged debe
    // seguir viendo el 12000 del índice.
    std::fs::write(
        dir.path().join("core/a.md"),
        "---\ntier: core\nkbx_budget_max: 99999\n---\ncontenido en disco\n",
    )
    .unwrap();

    let declaradas =
        trinquete::recolecta_staged(dir.path(), NOMINALES, &["archive", ".superpowers"]).unwrap();
    assert_eq!(declaradas.len(), 1);
    assert_eq!(declaradas[0].ruta, "core/a.md");
    assert_eq!(
        declaradas[0].max, 12000,
        "debe leer el max del índice, no del disco"
    );
}

#[test]
fn recolecta_staged_salta_un_fichero_staged_como_borrado() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    std::fs::create_dir_all(dir.path().join("core")).ok();
    std::fs::write(
        dir.path().join("core/a.md"),
        "---\ntier: core\nkbx_budget_max: 9000\n---\nx\n",
    )
    .unwrap();
    commit_todo(dir.path(), &cfg, "a.md commiteado");

    // Borrar y stagear el borrado: `git show :./core/a.md` debe fallar.
    std::fs::remove_file(dir.path().join("core/a.md")).unwrap();
    stage_todo(dir.path(), &cfg);

    let declaradas = trinquete::recolecta_staged(dir.path(), NOMINALES, &[]).unwrap();
    assert!(
        declaradas.is_empty(),
        "un fichero staged como borrado debe saltarse, no romper: {declaradas:?}"
    );
}

#[test]
fn recolecta_staged_excluye_por_primer_segmento() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    std::fs::create_dir_all(dir.path().join("archive")).ok();
    std::fs::write(
        dir.path().join("archive/vieja.md"),
        "---\ntier: core\nkbx_budget_max: 5000\n---\nx\n",
    )
    .unwrap();
    stage_todo(dir.path(), &cfg);

    let declaradas = trinquete::recolecta_staged(dir.path(), NOMINALES, &["archive"]).unwrap();
    assert!(
        declaradas.is_empty(),
        "archive/ debe excluirse: {declaradas:?}"
    );
}

// El tamaño medido en bytes crudos, no en la longitud de un `String` lossy.
//
// Nota de proceso: se intentó montar una nota staged con un byte NO-UTF8
// (p.ej. 0xFF suelto) para falsear que `recolecta_staged`/`comprueba_staged`
// miden bytes crudos y no `from_utf8_lossy().len()`. `std::fs::write` con un
// `&[u8]` arbitrario escribe el byte tal cual en las tres plataformas (no hay
// capa de codificación de por medio en escritura binaria), y `git add` lo
// staged igual — git no valida UTF-8 de blobs. Lo que SÍ varía es si
// `frontmatter::budget_max`/`tier`, que parsean el contenido como YAML sobre
// un `String::from_utf8_lossy`, encuentran el frontmatter con un byte
// inválido intercalado: no se pudo montar un caso donde el frontmatter siga
// parseando y el cuerpo lleve el byte inválido de forma estable entre
// serializadores YAML sin volverse frágil ante cambios en `frontmatter.rs`
// que no son responsabilidad de esta tarea. El test siguiente falsea la
// aritmética de bytes-crudos-vs-lossy de forma determinista y portable, sin
// pasar por el parseo de frontmatter: mide directamente con
// `gitx::muestra_bytes` un blob con un byte inválido staged, y compara con
// `git cat-file -s`, que es la fuente de verdad de git sobre el tamaño del
// blob.
#[test]
fn el_tamano_en_bytes_crudos_coincide_con_git_cat_file_para_un_blob_no_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());

    // Un byte inválido en UTF-8 (0xFF no es un byte de inicio ni continuación
    // válido) intercalado en contenido por lo demás ASCII.
    let mut contenido: Vec<u8> = b"x".repeat(50);
    contenido.push(0xFF);
    contenido.extend(b"y".repeat(50));
    std::fs::write(dir.path().join("bin.md"), &contenido).unwrap();
    stage_todo(dir.path(), &cfg);

    let bytes = exo::gitx::muestra_bytes(dir.path(), ":./bin.md")
        .unwrap()
        .expect("bin.md debe estar en el índice");
    assert_eq!(
        bytes.len(),
        101,
        "muestra_bytes debe medir los bytes reales del blob"
    );

    // La String lossy SUSTITUYE el byte inválido por U+FFFD (3 bytes en
    // UTF-8), así que su longitud diverge del tamaño real del blob: es
    // exactamente el fallo silencioso que `muestra_bytes` existe para evitar.
    let lossy = String::from_utf8_lossy(&bytes).to_string();
    assert_ne!(
        lossy.len(),
        bytes.len(),
        "el lossy debe divergir del tamaño real: es la trampa que muestra_bytes evita"
    );

    let salida = Command::new("git")
        .arg("-C")
        .arg(dir.path())
        .args(["cat-file", "-s", ":./bin.md"])
        .env("GIT_CONFIG_GLOBAL", &cfg)
        .env("GIT_CONFIG_SYSTEM", &cfg)
        .output()
        .unwrap();
    assert!(salida.status.success());
    let tamano_git: usize = String::from_utf8_lossy(&salida.stdout)
        .trim()
        .parse()
        .unwrap();
    assert_eq!(
        bytes.len(),
        tamano_git,
        "muestra_bytes debe coincidir con la fuente de verdad de git (cat-file -s)"
    );
}

// El test anterior falsea `gitx::muestra_bytes` en aislado, pero no ejercita
// el camino real de la guarda de aire (`tamano_de_indice`, dentro de
// `comprueba_staged`): un `tamano_de_indice` implementado por error con la
// `String` lossy de `gitx::muestra` seguía pasando todos los tests de arriba
// en verde (falsado por mutación: se cambió `muestra_bytes` por `muestra` en
// `tamano_de_indice` y los 9 tests de esta suite —incluido el de bytes
// crudos, que no pasa por `comprueba_staged`— seguían en verde). Este test
// cierra ese hueco: monta un sello justo en el borde del aire, con un
// tamaño real que SÍ tiene aire pero cuya versión lossy (con el byte
// inválido inflado a 3 bytes por `from_utf8_lossy`) NO la tiene, y
// comprueba el veredicto a través de `comprueba_staged` de punta a punta.
#[test]
fn comprueba_staged_mide_el_tamano_en_bytes_crudos_no_en_la_string_lossy() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = init_repo(dir.path());
    ancla_instalada(dir.path(), &cfg);

    // 100 bytes reales (99 'x' + 1 byte 0xFF inválido). La versión lossy
    // sustituye el 0xFF por U+FFFD (3 bytes), dando 102.
    let mut contenido: Vec<u8> = b"x".repeat(99);
    contenido.push(0xFF);
    assert_eq!(contenido.len(), 100);
    std::fs::write(dir.path().join("bin.md"), &contenido).unwrap();

    // techo_minimo(100) == 115: con el tamaño real (100) hay aire justo al
    // borde; con el lossy (102) NO lo hay (115*100=11500 < 102*115=11730).
    escribe_sello_en_disco(dir.path(), &[("bin.md", 115)]);
    stage_todo(dir.path(), &cfg);

    let informe = trinquete::comprueba_staged(dir.path(), &[], NOMINALES).unwrap();
    assert!(
        informe.hallazgos.is_empty(),
        "con el tamaño real (100 B) el sello 115 tiene aire justo al borde: {:?}",
        informe.hallazgos
    );
    assert!(!informe.fallido());
}
