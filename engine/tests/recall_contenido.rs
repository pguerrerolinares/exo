//! M6-02: `exo recall --content`.
//!
//! El hook de arranque que esto sustituye NO inyecta una lista de ficheros:
//! inyecta el CUERPO del core-index (contrato de memoria + doctrina
//! compacta + mapa de cores) más un digest de actividad reciente. Servir
//! solo rutas sería una regresión funcional silenciosa — el agente perdería
//! la doctrina en todas las sesiones y nadie lo notaría hasta que empezara
//! a comportarse peor.

mod common;

use exo::indexer::indexa;
use exo::recall::recall_arranque_contenido;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
}

fn escribe(dir: &std::path::Path, nombre: &str, tier: &str, cuerpo: &str) {
    let permalink = nombre.trim_end_matches(".md");
    fs::write(
        dir.join(nombre),
        format!(
            "---\npermalink: kb/{permalink}\ntitle: {permalink}\ntier: {tier}\n---\n\n{cuerpo}\n"
        ),
    )
    .unwrap();
}

fn kb_de_prueba() -> TempDir {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    escribe(
        kb.path(),
        "indice.md",
        "core",
        "DOCTRINA: delega y quédate la conclusión.",
    );
    escribe(
        kb.path(),
        "otra.md",
        "log",
        "Bitácora de cosas que pasaron.",
    );
    git(kb.path(), &["add", "-A"]);
    git(
        kb.path(),
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "kb",
        ],
    );
    kb
}

fn db_temporal() -> (TempDir, std::path::PathBuf) {
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    (dbdir, db)
}

#[test]
fn contenido_vuelca_el_cuerpo_de_las_notas_core() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

        assert!(
            bloque.contains("DOCTRINA: delega y quédate la conclusión."),
            "el cuerpo del core debe ir literal en el bloque, no solo su ruta:\n{bloque}"
        );
    });
}

#[test]
fn contenido_no_vuelca_el_cuerpo_de_las_notas_no_core() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

        assert!(
            !bloque.contains("Bitácora de cosas que pasaron."),
            "solo los `tier: core` van con cuerpo; el resto, como mucho, listados"
        );
    });
}

/// M6-07: las recientes se listan por RUTA relativa a la raíz declarada en
/// cabecera, no por permalink. El permalink llevaba el nombre del proyecto
/// (`kb-demo/`, `kb/`) delante de CADA línea: sobre la KB real son 12
/// bytes por línea, diez líneas, que no dicen nada que la cabecera no diga ya
/// una vez. La ruta relativa cuesta eso menos, y además es accionable: raíz +
/// ruta es un `Read` directo, cosa que el permalink no da.
#[test]
fn contenido_lista_las_recientes_por_ruta_relativa() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

        assert!(
            bloque.contains("otra.md"),
            "las recientes se listan por ruta relativa a la raíz:\n{bloque}"
        );
        assert!(
            !bloque.contains("kb/otra"),
            "el prefijo de proyecto del permalink no debe repetirse por línea:\n{bloque}"
        );
    });
}

/// La raíz aparece UNA vez, en cabecera, y ninguna línea la repite. Es lo que
/// hace que el bloque quepa, y de paso lo vuelve estable entre máquinas: con
/// la ruta absoluta en cada core, el mismo bloque cabía en Linux
/// (`/home/paul/...`) y se truncaba en Windows (`C:\Users\<user>\...`), que es
/// la clase de bug "pasa allí, falla aquí" que este formato elimina de raíz.
#[test]
fn contenido_declara_la_raiz_una_sola_vez() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

        let raiz = kb.path().display().to_string();
        assert_eq!(
            bloque.matches(&raiz).count(),
            1,
            "la raíz debe aparecer exactamente una vez, en cabecera:\n{bloque}"
        );
        assert!(
            bloque.lines().nth(1).unwrap_or("").starts_with("KB: "),
            "la segunda línea del bloque declara la raíz:\n{bloque}"
        );
        assert!(
            bloque.contains("# indice (indice.md)"),
            "el core se titula con su ruta RELATIVA:\n{bloque}"
        );
    });
}

/// Casi toda bitácora archivada se titula igual que su fichero
/// (`exo-bitacora-2026-07-17_2026-08-22`), así que la línea escribía el mismo
/// texto dos veces separado por un guión. Se omite el título cuando no añade
/// nada — y solo entonces.
#[test]
fn contenido_omite_el_titulo_solo_cuando_repite_el_nombre_del_fichero() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();
        let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

        assert!(
            !bloque.contains("otra.md — otra"),
            "título redundante: repite el nombre del fichero:\n{bloque}"
        );
    });

    // Una nota cuyo título NO es su nombre de fichero sí lo conserva.
    let kb2 = TempDir::new().unwrap();
    git(kb2.path(), &["init", "-q"]);
    escribe(kb2.path(), "indice.md", "core", "DOCTRINA.");
    fs::write(
        kb2.path().join("slug-feo.md"),
        "---\npermalink: kb/slug-feo\ntitle: Un título de verdad\ntier: log\n---\n\nx\n",
    )
    .unwrap();
    git(kb2.path(), &["add", "-A"]);
    git(
        kb2.path(),
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "kb",
        ],
    );
    let d2 = TempDir::new().unwrap();
    let db2 = d2.path().join("i.db");
    common::con_config(kb2.path(), "kb-test", &db2, || {
        indexa(kb2.path(), &db2).unwrap();
        let bloque2 = recall_arranque_contenido(&db2, kb2.path(), 5, 8192, None).unwrap();

        assert!(
            bloque2.contains("slug-feo.md — Un título de verdad"),
            "un título que aporta información se conserva:\n{bloque2}"
        );
    });
}

/// El cap es en BYTES y el bloque mete la ruta ABSOLUTA de cada core en su
/// línea de título (`# titulo (ruta)`), así que el `120` mágico que había aquí
/// no medía la propiedad: medía cuánto ocupa el tempdir del sistema operativo.
/// La cabecera son 55 bytes y la línea en blanco 1, o sea que del cap solo
/// quedaban 64 para esa línea de título. En Linux la ruta de la nota es
/// `/tmp/.tmpXXXXXX/indice.md` (25 bytes), la línea sale a 37 y cabía: el
/// bloque se truncaba, el assert pasaba y el test parecía bueno. En Windows es
/// `C:\Users\<user>\AppData\Local\Temp\.tmpXXXXXX\indice.md` (55 bytes), la
/// línea sale a 67 y NO cabía: el bloque se quedaba en la cabecera pelada y
/// `recall_arranque_contenido` hacía `bail!` a propósito — test rojo por el
/// entorno, no por una regresión.
///
/// Los caps se derivan del bloque REALMENTE renderizado en esta máquina, así
/// que el test es portable y además más estricto que antes: ya no comprueba
/// solo `<= cap`, sino que el truncado corta por líneas enteras, en el borde
/// exacto, y que lo que sobrevive es prefijo literal del bloque completo.
#[test]
fn contenido_respeta_el_cap_de_bytes() {
    let kb = kb_de_prueba();
    let (_d, db) = db_temporal();

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();

        // Referencia sin truncar: la KB de prueba cabe de sobra en 8192 (mismo
        // cap holgado que usan los demás tests de este fichero).
        let completo = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();
        assert!(
            completo.len() < 8192,
            "la referencia debe ir SIN truncar o los caps derivados no significan nada: \
             {} bytes",
            completo.len()
        );

        // `split_inclusive` conserva el '\n' que el recall añade a cada línea, así
        // que el coste medido es el mismo que cuenta el truncado (`linea.len() + 1`)
        // sin suponer nada sobre finales de línea. `cumulativo[i]` = bytes que
        // ocupan las `i + 1` primeras líneas.
        let costes: Vec<usize> = completo.split_inclusive('\n').map(str::len).collect();
        let cumulativo: Vec<usize> = costes
            .iter()
            .scan(0usize, |acc, c| {
                *acc += c;
                Some(*acc)
            })
            .collect();
        assert_eq!(
            cumulativo.last().copied(),
            Some(completo.len()),
            "el desglose por líneas debe sumar el bloque entero:\n{completo}"
        );

        // El primer corte SERVIBLE es el que ya incluye la primera línea de
        // CONTENIDO: por debajo de eso el bloque es cabecera (+ la línea en blanco
        // que la separa del cuerpo) y el recall hace `bail!` adrede.
        //
        // Se DERIVA del bloque en vez de escribirse a mano. Aquí había un `3`
        // fijo, correcto mientras la cabecera fue una sola línea; M6-07 le añadió
        // la línea de la raíz y ese 3 pasó a apuntar dentro de la cabecera, con lo
        // que el test empezó a exigir que `bail!` no ocurriera justo donde debe
        // ocurrir. Derivarlo hace que la próxima línea de cabecera no rompa nada.
        let primer_contenido = completo
            .lines()
            .position(|l| l.starts_with("# "))
            .expect("el bloque debe traer al menos un core con cuerpo");
        let primer_servible = primer_contenido + 1;
        assert!(
            costes.len() > primer_servible,
            "la KB de prueba debe dar para al menos un truncado servible:\n{completo}"
        );

        for k in primer_servible..costes.len() {
            // Cap clavado en el borde de la línea k: deben entrar k líneas, ni
            // una menos (no se malgasta presupuesto) ni una más (el cap manda).
            let cap = cumulativo[k - 1];
            let bloque = recall_arranque_contenido(&db, kb.path(), 5, cap, None).unwrap();
            assert!(
                bloque.len() <= cap,
                "bloque de {} bytes con cap {cap}: el cap del consumidor no es negociable",
                bloque.len()
            );
            assert_eq!(
                bloque.len(),
                cap,
                "con el cap justo en el borde de la línea {k} deben caber esas {k} líneas enteras:\n\
                 {bloque}"
            );
            assert!(
                bloque.len() < completo.len(),
                "si el cap no recorta nada, este test no está probando el truncado"
            );
            assert_eq!(
                bloque,
                completo[..cap],
                "lo que sobrevive al cap debe ser prefijo LITERAL del bloque completo"
            );

            // Un byte menos de lo que costaría la línea k + 1: no debe colarse
            // media línea para llenar el hueco. El truncado es por líneas enteras.
            let cap_sobrante = cumulativo[k] - 1;
            let bloque_sobrante =
                recall_arranque_contenido(&db, kb.path(), 5, cap_sobrante, None).unwrap();
            assert!(
                bloque_sobrante.len() <= cap_sobrante,
                "bloque de {} bytes con cap {cap_sobrante}: el cap del consumidor no es negociable",
                bloque_sobrante.len()
            );
            assert_eq!(
                bloque_sobrante,
                bloque,
                "con {cap_sobrante} bytes la línea {} no cabe entera, así que no entra NADA de ella",
                k + 1
            );
        }
    });
}

#[test]
fn contenido_falla_si_no_hay_nada_que_servir() {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");

    common::con_config(kb.path(), "kb-test", &db, || {
        indexa(kb.path(), &db).unwrap();

        // KB sin notas: el hook debe poder distinguir "no hay bloque" por exit
        // code y caer a su fallback, en vez de inyectar un bloque vacío.
        assert!(recall_arranque_contenido(&db, kb.path(), 5, 8192, None).is_err());
    });
}
