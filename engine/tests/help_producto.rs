//! La ayuda de `exo` (`--help`) es superficie de producto: la lee alguien que
//! acaba de instalar el binario, no quien lo escribió.
//!
//! Tres invariantes, cada uno falsable contra el binario real:
//! 1. toda opción lleva descripción (un `--json` en blanco no dice nada);
//! 2. el metavar de un flag es el nombre del flag en MAYÚSCULAS (`--limit
//!    <LIMIT>`): los flags largos son ingleses desde la 1.0 y un `<LIMITE>` heredado del
//!    nombre del campo mezcla idiomas en la misma línea;
//! 3. ninguna pantalla lleva jerga interna de campaña (hitos, specs, scripts
//!    del autor, símbolos de Rust) (`la_ayuda_no_lleva_jerga_interna`).
//!
//! Hermético: `EXO_CONFIG` apunta a un fichero inexistente; clap resuelve
//! `--help` antes de leer ninguna config.
use std::process::Command;

fn ayuda(args: &[&str]) -> String {
    let salida = Command::new(env!("CARGO_BIN_EXE_exo"))
        .args(args)
        .arg("--help")
        .env("EXO_CONFIG", "C:/no-existe-jamas/config.toml")
        .output()
        .expect("correr el binario");
    assert!(
        salida.status.success(),
        "{args:?} --help salió con {:?}",
        salida.status
    );
    String::from_utf8(salida.stdout).expect("help en UTF-8")
}

/// Todas las pantallas de ayuda del binario. Si se añade un subcomando y no se
/// añade aquí, `la_lista_cubre_todos_los_subcomandos` falla.
const PANTALLAS: &[&[&str]] = &[
    &[],
    &["init"],
    &["config"],
    &["index"],
    &["rebuild"],
    &["search"],
    &["write"],
    &["write", "new"],
    &["write", "append"],
    &["recall"],
    &["targets"],
    &["budget"],
    &["lint"],
    &["ratchet"],
    &["doctor"],
];

/// Líneas de una sección (`Commands:`, `Options:`, `Arguments:`) hasta la
/// siguiente línea en blanco.
fn seccion<'a>(texto: &'a str, cabecera: &str) -> Vec<&'a str> {
    texto
        .lines()
        .skip_while(|l| *l != cabecera)
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .collect()
}

fn subcomandos(texto: &str) -> Vec<String> {
    seccion(texto, "Commands:")
        .iter()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|n| *n != "help")
        .map(str::to_string)
        .collect()
}

#[test]
fn la_lista_cubre_todos_los_subcomandos() {
    let mut vistas: Vec<Vec<String>> = subcomandos(&ayuda(&[]))
        .into_iter()
        .map(|c| vec![c])
        .collect();
    vistas.extend(
        subcomandos(&ayuda(&["write"]))
            .into_iter()
            .map(|c| vec!["write".to_string(), c]),
    );
    for v in vistas {
        assert!(
            PANTALLAS
                .iter()
                .any(|p| p.iter().copied().eq(v.iter().map(String::as_str))),
            "el subcomando {v:?} no está en PANTALLAS"
        );
    }
}

#[test]
fn toda_opcion_lleva_descripcion() {
    // clap pinta la descripción en la misma línea o, cuando la columna de
    // flags es ancha, en la línea siguiente con sangría profunda. Una opción
    // sin descripción es una línea de flag «pelada» cuya siguiente línea NO es
    // esa sangría.
    let flag_pelado =
        regex::Regex::new(r"^\s+(-[A-Za-z], )?--[a-z0-9-]+( <[A-Z_]+>)?\s*$").unwrap();
    let descripcion_debajo = regex::Regex::new(r"^\s{8,}[^\s-]").unwrap();
    for p in PANTALLAS {
        let texto = ayuda(p);
        let opciones = seccion(&texto, "Options:");
        for (i, linea) in opciones.iter().enumerate() {
            if flag_pelado.is_match(linea) {
                let siguiente = opciones.get(i + 1).copied().unwrap_or("");
                assert!(
                    descripcion_debajo.is_match(siguiente),
                    "exo {} --help: opción sin descripción: {linea:?}",
                    p.join(" ")
                );
            }
        }
    }
}

#[test]
fn el_metavar_de_un_flag_es_el_nombre_del_flag() {
    let flag = regex::Regex::new(r"--([a-z0-9-]+) <([A-Z_]+)>").unwrap();
    for p in PANTALLAS {
        let texto = ayuda(p);
        for linea in seccion(&texto, "Options:") {
            if let Some(c) = flag.captures(linea) {
                let esperado = c[1].to_uppercase().replace('-', "_");
                assert_eq!(
                    &c[2],
                    esperado,
                    "exo {} --help: `--{}` muestra <{}>",
                    p.join(" "),
                    &c[1],
                    &c[2]
                );
            }
        }
    }
}

/// Jerga que delata que la ayuda la escribió el autor para sí mismo. Cada
/// entrada: patrón y por qué no le dice nada a un usuario.
const JERGA: &[(&str, &str)] = &[
    (
        r"\b[MGDE][0-9]+[a-z]?(-[0-9]+)?\b",
        "id de hito interno (M2-07, G4b, D6, E1)",
    ),
    (r"\b[mg][0-9]+-[0-9]+\b", "id de item de campaña (m2-05)"),
    (r"\bD-f[0-9]", "id de decisión de spec"),
    (r"§", "sección de una spec"),
    (r"\b(spec|brief|Task)\b", "documento de proceso"),
    (r"\bsucesor\b", "genealogía del código"),
    (r"kbx [a-z]", "comando de otra herramienta"),
    (r"kb-demo", "nombre de la KB del autor"),
    (
        r"basic-memory-recall|compose-inject|replay-engine|documenta\.md|\breflex\b",
        "script o skill del autor",
    ),
    (
        r"::|busca_hybrid|notas_fts|`notas`|fallido\(\)|_SELLAD",
        "símbolo interno de Rust o SQL",
    ),
];

#[test]
fn la_ayuda_no_lleva_jerga_interna() {
    let patrones: Vec<(regex::Regex, &str)> = JERGA
        .iter()
        .map(|(p, por_que)| (regex::Regex::new(p).unwrap(), *por_que))
        .collect();
    let mut hallazgos = Vec::new();
    for p in PANTALLAS {
        let texto = ayuda(p);
        for linea in texto.lines() {
            for (re, por_que) in &patrones {
                if let Some(m) = re.find(linea) {
                    hallazgos.push(format!(
                        "exo {} --help: {:?} ({por_que}) en {linea:?}",
                        p.join(" "),
                        m.as_str()
                    ));
                }
            }
        }
    }
    assert!(
        hallazgos.is_empty(),
        "jerga en la ayuda:\n{}",
        hallazgos.join("\n")
    );
}
