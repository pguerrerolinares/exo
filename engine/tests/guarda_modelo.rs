//! Ola 1 T1: guarda de modelo de embeddings en `meta`.
//!
//! Dos modelos de la misma dimensión (768d: el jina actual y
//! `multilingual-e5-base`) producen blobs indistinguibles para
//! `vectores::lee` — su chequeo de longitud (`BYTES_ESPERADOS`) no separa
//! por procedencia. Sin esta guarda, un `exo index` incremental tras
//! cambiar el modelo activo mezclaría vectores de ambos en la misma tabla
//! sin una sola queja.
//!
//! Estos tests NUNCA cargan el modelo real: la nota fixture tiene cuerpo
//! vacío, así que `trocea` no produce trozos y `con_embedder_de_proceso`
//! jamás se llama (m2-06). La guarda vive al INICIO de `indexa`, antes de
//! tocar fastembed (decisión del brief T1) — por eso estos tests son
//! rápidos y deterministas, y por eso pueden sembrar `meta` a mano en vez
//! de tocar la config real (`config_embeddings()` lee `$HOME` global y no
//! es inyectable; cambiar `$HOME` en el test redescargaría el modelo).

use exo::indexer::indexa;
use tempfile::TempDir;

/// Nota con frontmatter válido y cuerpo vacío: se indexa (tiene permalink)
/// pero no produce trozos, así que jamás toca el embedder de proceso.
fn kb_vacia() -> TempDir {
    let kb = TempDir::new().unwrap();
    std::fs::write(
        kb.path().join("nota.md"),
        "---\npermalink: kb/nota\ntitle: Nota\n---\n",
    )
    .unwrap();
    kb
}

fn db_temporal() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("i.db");
    (dir, db)
}

#[test]
fn modelo_distinto_aborta_y_cita_rebuild() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    // Primera corrida: DB nueva, meta.modelo_embeddings ausente todavía ->
    // pasa la guarda y escribe el modelo real de la config.
    indexa(kb.path(), &db).unwrap();

    // Se siembra en meta un modelo DISTINTO, como si este índice viniera de
    // otro modelo (sin tocar la config real, ver comentario del módulo).
    let conn = exo::abre_db(&db).unwrap();
    conn.execute(
        "UPDATE meta SET valor = 'multilingual-e5-base' WHERE clave = 'modelo_embeddings'",
        [],
    )
    .unwrap();
    drop(conn);

    let err = indexa(kb.path(), &db).unwrap_err();
    assert!(
        err.to_string().contains("rebuild"),
        "el mensaje debe citar 'exo rebuild', fue: {err}"
    );
}

#[test]
fn clave_ausente_migra_en_silencio() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    // Simula un índice viejo, de antes de que esta guarda existiera.
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM meta WHERE clave = 'modelo_embeddings'", [])
        .unwrap();
    drop(conn);

    // No debe fallar: sin clave que comparar, la guarda deja pasar y
    // `indexa` la reescribe.
    indexa(kb.path(), &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let valor: String = conn
        .query_row(
            "SELECT valor FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .expect("la clave debe quedar reescrita tras la migración silenciosa");
    assert!(!valor.is_empty());
}

/// Fix del finding Critical de la review de T1: `meta.modelo_embeddings`
/// debe quedar escrita aunque la corrida no llegue al final, porque el
/// bucle de `indexa` commitea por nota (transacción por nota) — cuando la
/// corrida aborta a mitad, las notas ya procesadas están en disco con sus
/// vectores, y la clave tiene que reflejarlo desde ya, no solo si la corrida
/// entera termina bien.
///
/// No se simula un `kill -9` real (eso corrompería el propio proceso de
/// test). En su lugar se fuerza un error real y tardío en el pipeline —tras
/// haber commiteado ya una nota anterior— haciendo que `parsea_nota` falle
/// su `std::fs::read_to_string` sobre un fichero `.md` de la KB: el error se
/// propaga con `?` (línea `let Some(n) = parsea_nota(ruta_abs)?`) y aborta
/// `indexa` sin llegar al final. `walk_kb` ordena las rutas
/// (`encontradas.sort()`), así que `a.md` se procesa y commitea ANTES que
/// `b-illeg.md`, que es la que se vuelve ilegible.
///
/// El mecanismo que rompe la lectura es UTF-8 inválido en el fichero, no un
/// `chmod 0o000` como en la primera versión de este test. Razones, por orden
/// de peso:
///
/// 1. `PermissionsExt::from_mode` es `std::os::unix`: no existe en Windows, y
///    un solo target que no compila aborta `cargo test` ENTERO — la suite
///    completa dejaba de correr en Windows por este único test.
/// 2. En Windows no hay equivalente barato: quitar el permiso de LECTURA
///    exige manipular ACLs, no un bit de modo.
/// 3. El 0o000 tampoco era fiable en Unix: bajo root los bits se ignoran, y
///    el test tenía que detectarlo y volverse un `return` silencioso (CI en
///    contenedor = root es lo habitual, o sea que ahí no probaba nada).
/// 4. Marcar el fichero como directorio en vez de fichero NO sirve: `walk_kb`
///    filtra con `tipo.is_file()` y recursa en los directorios, así que
///    `b-illeg.md` ni siquiera llegaría a ser candidato.
///
/// El error sigue siendo un `std::io::Error` genuino devuelto por
/// `read_to_string` (kind `InvalidData`), no uno inventado ni mockeado por el
/// test: la validación UTF-8 vive dentro de la propia llamada, antes de que
/// `parsea_nota` mire el frontmatter, así que el camino de código ejercitado
/// es exactamente el mismo (`?` que cortocircuita) y ahora es determinista en
/// las dos plataformas y con cualquier uid.
///
/// Esto verifica el contrato nuevo (la clave se escribe temprano, no al
/// final) sin necesitar un abort a mitad del bucle: si el upsert siguiera
/// viviendo al final de la función, este test fallaría, porque el `?` de
/// `parsea_nota` en `b-illeg.md` cortocircuita antes de llegar a esas
/// líneas.
#[test]
fn clave_escrita_aunque_la_corrida_no_llegue_al_final() {
    let kb = TempDir::new().unwrap();
    std::fs::write(
        kb.path().join("a.md"),
        "---\npermalink: kb/a\ntitle: A\n---\n",
    )
    .unwrap();
    let ilegible = kb.path().join("b-illeg.md");
    // Bytes 0xFF/0xFE: inválidos en UTF-8 en cualquier posición, así que
    // `read_to_string` falla pase lo que pase con el resto del contenido. El
    // frontmatter delante solo documenta que la nota SERÍA indexable si se
    // pudiera leer: el fallo no viene de que le falte permalink (ese camino
    // es `Ok(None)`, un skip, y no abortaría nada), viene de la lectura.
    std::fs::write(
        &ilegible,
        b"---\npermalink: kb/b\ntitle: B\n---\n\xFF\xFE\xFF",
    )
    .unwrap();

    // El escenario se comprueba directamente en vez de asumirlo: si por lo
    // que sea la lectura tuviera éxito, `indexa` no abortaría y el assert de
    // más abajo fallaría con un mensaje que no explica la causa real.
    assert!(
        std::fs::read_to_string(&ilegible).is_err(),
        "el fixture debe ser ilegible como texto: sin eso este test no prueba nada"
    );

    let (_dbdir, db) = db_temporal();

    let resultado = indexa(kb.path(), &db);

    assert!(
        resultado.is_err(),
        "la corrida debía abortar al toparse con b-illeg.md ilegible"
    );

    let conn = exo::abre_db(&db).unwrap();

    // Confirma que el escenario del finding realmente ocurrió: a.md quedó
    // commiteada (transacción por nota) ANTES del abort en b-illeg.md. Sin
    // esto el test no probaría nada — necesitamos que haya vectores en disco
    // de la corrida abortada para que "meta debe reflejarlos" tenga sentido.
    let a_commiteada: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notas WHERE permalink = 'kb/a'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        a_commiteada, 1,
        "a.md debía quedar commiteada antes del abort en b-illeg.md"
    );

    // A pesar del abort, meta.modelo_embeddings debe estar escrita: es la
    // garantía que exige el fix (upsert junto a kb_root, al principio).
    let valor: String = conn
        .query_row(
            "SELECT valor FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .expect("modelo_embeddings debe quedar escrita aunque la corrida haya abortado a mitad");
    assert!(!valor.is_empty());
}

/// Fix de la review final de T1: un índice viejo (clave ausente) que YA
/// tiene vectores de otro modelo debe seguir migrando sin abortar — el aviso
/// por stderr (Cambio 1) es no-bloqueante, solo declara la deuda en vez de
/// dejarla muda. Este test cubre el camino ("sigue sin error, la clave queda
/// escrita, los vectores no se tocan"), NO la emisión del aviso en sí:
/// capturar stderr desde un test de integración de Rust (proceso separado
/// del binario de test, `eprintln!` no pasa por ningún logger inyectable)
/// no tiene una forma limpia en este repo, y forzarla con
/// `std::process::Command` reinvocando el propio test sería más aparato que
/// señal para una sola línea de `eprintln!`. Queda sin cubrir por test;
/// ver reporte de la tarea.
#[test]
fn clave_ausente_con_vectores_sigue_migrando_sin_error() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();

    // Simula un índice viejo con vectores ya en disco (de "otro modelo",
    // aunque aquí el contenido del vector es indiferente al escenario) y sin
    // la clave que los identifica.
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM meta WHERE clave = 'modelo_embeddings'", [])
        .unwrap();
    exo::vectores::inserta(&conn, 1, &vec![0.0f32; 768]).unwrap();
    drop(conn);

    // No debe fallar: el conteo de vectores solo dispara un aviso por
    // stderr, nunca un abort.
    indexa(kb.path(), &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let valor: String = conn
        .query_row(
            "SELECT valor FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .expect("la clave debe quedar reescrita tras la migración silenciosa");
    assert!(!valor.is_empty());

    let n_vectores: i64 = conn
        .query_row("SELECT COUNT(*) FROM vectores", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        n_vectores, 1,
        "el vector sembrado no debe tocarse por el aviso"
    );
}

#[test]
fn modelo_igual_no_falla_al_reindexar() {
    let kb = kb_vacia();
    let (_dbdir, db) = db_temporal();

    indexa(kb.path(), &db).unwrap();
    // Segunda corrida sin tocar nada: meta.modelo_embeddings ya coincide
    // con la config -> debe seguir sin error.
    indexa(kb.path(), &db).unwrap();

    let conn = exo::abre_db(&db).unwrap();
    let filas: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        filas, 1,
        "modelo_embeddings debe ser upsert, no insert repetido"
    );
}
