//! M6-02: `exo recall --contenido`.
//!
//! El hook de arranque que esto sustituye NO inyecta una lista de ficheros:
//! inyecta el CUERPO del core-index (contrato de memoria + doctrina compacta
//! + mapa de cores) más un digest de actividad reciente. Servir solo rutas
//! sería una regresión funcional silenciosa — el agente perdería la doctrina
//! en todas las sesiones y nadie lo notaría hasta que empezara a comportarse
//! peor.

use exo::indexer::indexa;
use exo::recall::recall_arranque_contenido;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git").args(args).current_dir(dir).output().unwrap();
}

fn escribe(dir: &std::path::Path, nombre: &str, tier: &str, cuerpo: &str) {
    let permalink = nombre.trim_end_matches(".md");
    fs::write(
        dir.join(nombre),
        format!("---\npermalink: kb/{permalink}\ntitle: {permalink}\ntier: {tier}\n---\n\n{cuerpo}\n"),
    )
    .unwrap();
}

fn kb_de_prueba() -> (TempDir, std::path::PathBuf, TempDir) {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    escribe(kb.path(), "indice.md", "core", "DOCTRINA: delega y quédate la conclusión.");
    escribe(kb.path(), "otra.md", "log", "Bitácora de cosas que pasaron.");
    git(kb.path(), &["add", "-A"]);
    git(
        kb.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "kb"],
    );
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    indexa(kb.path(), &db).unwrap();
    (kb, db, dbdir)
}

#[test]
fn contenido_vuelca_el_cuerpo_de_las_notas_core() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        bloque.contains("DOCTRINA: delega y quédate la conclusión."),
        "el cuerpo del core debe ir literal en el bloque, no solo su ruta:\n{bloque}"
    );
}

#[test]
fn contenido_no_vuelca_el_cuerpo_de_las_notas_no_core() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        !bloque.contains("Bitácora de cosas que pasaron."),
        "solo los `tier: core` van con cuerpo; el resto, como mucho, listados"
    );
}

#[test]
fn contenido_lista_las_recientes_por_permalink() {
    let (kb, db, _d) = kb_de_prueba();
    let bloque = recall_arranque_contenido(&db, kb.path(), 5, 8192, None).unwrap();

    assert!(
        bloque.contains("kb/otra"),
        "las recientes se listan por permalink (paridad con el digest del hook actual):\n{bloque}"
    );
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
    let (kb, db, _d) = kb_de_prueba();

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

    // Con menos de 3 líneas el bloque es cabecera (+ línea en blanco) y el
    // recall hace `bail!` adrede, así que el primer corte SERVIBLE es k = 3.
    // Exigir que haya más líneas que eso garantiza que este test trunca de
    // verdad y no se queda en un caso degenerado que pasaría siempre.
    assert!(
        costes.len() > 3,
        "la KB de prueba debe dar para al menos un truncado servible:\n{completo}"
    );

    for k in 3..costes.len() {
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
}

#[test]
fn contenido_falla_si_no_hay_nada_que_servir() {
    let kb = TempDir::new().unwrap();
    git(kb.path(), &["init", "-q"]);
    let dbdir = TempDir::new().unwrap();
    let db = dbdir.path().join("i.db");
    indexa(kb.path(), &db).unwrap();

    // KB sin notas: el hook debe poder distinguir "no hay bloque" por exit
    // code y caer a su fallback, en vez de inyectar un bloque vacío.
    assert!(recall_arranque_contenido(&db, kb.path(), 5, 8192, None).is_err());
}
