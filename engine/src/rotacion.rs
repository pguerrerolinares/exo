//! `exo rotate` — divide una bitácora `tier: log` en un prefijo frío
//! (archivado) y una cola caliente (que se queda), portado de
//! `kbx/internal/rotate` (`fe46443`). El corte es posicional, no por
//! fecha: solo 34 de 60 headings de la bitácora más irregular llevan fecha
//! ISO, así que parsear fechas sería frágil justo donde más importa
//! (`rotate.go`, comentario de cabecera del paquete).

use anyhow::{Context, Result};
use serde::Serialize;
use std::borrow::Cow;
use std::path::Path;
use std::sync::LazyLock;

/// El resultado de partir un contenido en preámbulo + frío + caliente.
/// Invariante duro: `preambulo + frio + caliente` reconstruye el original
/// byte a byte — nada se pierde (kbx `rotate.Plan`).
#[derive(Debug, PartialEq, Eq)]
pub struct Plan<'a> {
    pub preambulo: &'a [u8],
    pub frio: &'a [u8],
    pub caliente: &'a [u8],
    pub entradas_frias: usize,
    pub entradas_calientes: usize,
}

/// Cada línea de `contenido` con su offset de inicio, sin el terminador
/// `\n`. Si `contenido` no acaba en `\n`, la última línea (parcial) se
/// incluye igual — mismo contrato que `splitLinesKeepOffsets` en el Go
/// original, del que dependen `offsets_de_entradas`, `bloque_frontmatter` y
/// `frontmatter_sin_terminar`: una sola función de partir en líneas para
/// las tres, en vez de tres copias que puedan derivar.
fn lineas_con_offset(contenido: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off <= contenido.len() {
        match contenido[off..].iter().position(|&b| b == b'\n') {
            None => {
                out.push((off, &contenido[off..]));
                break;
            }
            Some(n) => {
                out.push((off, &contenido[off..off + n]));
                off += n + 1;
            }
        }
    }
    out
}

/// Offset de cada heading `"## "` a inicio de línea, ignorando los que
/// caen dentro de una valla ``` de código.
fn offsets_de_entradas(contenido: &[u8]) -> Vec<usize> {
    let mut offs = Vec::new();
    let mut en_valla = false;
    for (off, linea) in lineas_con_offset(contenido) {
        if linea.starts_with(b"```") {
            en_valla = !en_valla;
            continue;
        }
        if !en_valla && linea.starts_with(b"## ") {
            offs.push(off);
        }
    }
    offs
}

/// Mantiene las entradas más nuevas cuyo tamaño total, junto al preámbulo,
/// entra en `presupuesto_caliente`. Al menos una entrada se queda siempre
/// caliente (kbx `rotate.Split`).
pub fn parte(contenido: &[u8], presupuesto_caliente: i64) -> Plan<'_> {
    let offs = offsets_de_entradas(contenido);
    let Some(&primera) = offs.first() else {
        return Plan {
            preambulo: contenido,
            frio: &contenido[contenido.len()..],
            caliente: &contenido[contenido.len()..],
            entradas_frias: 0,
            entradas_calientes: 0,
        };
    };
    let preambulo = &contenido[..primera];

    let mut corte = offs.len() - 1;
    for i in (0..offs.len() - 1).rev() {
        let tamano = (contenido.len() - offs[i]) as i64;
        if preambulo.len() as i64 + tamano > presupuesto_caliente {
            break;
        }
        corte = i;
    }

    Plan {
        preambulo,
        frio: &contenido[offs[0]..offs[corte]],
        caliente: &contenido[offs[corte]..],
        entradas_frias: corte,
        entradas_calientes: offs.len() - corte,
    }
}

/// Línea delimitadora de frontmatter: `---` seguida solo de espacios/tabs,
/// tolerando también un `\r` final — mismo fix que ya aplica
/// `frontmatter::es_delimitador` para el resto de exo (sin él, un checkout
/// CRLF hace que un cierre `---\r` nunca matchee y el fallback sintético se
/// dispare en silencio sobre frontmatter válido; es el "fallo silencioso
/// canónico del port" que `frontmatter.rs` ya documenta, y esta función
/// replica la misma regla en vez de reintroducir la más estrecha de kbx).
fn es_delimitador_frontmatter(linea: &[u8]) -> bool {
    let mut fin = linea.len();
    while fin > 0 && matches!(linea[fin - 1], b' ' | b'\t' | b'\r') {
        fin -= 1;
    }
    &linea[..fin] == b"---"
}

/// El bloque `---...---` inicial, o un bloque sintético `tier: log` si
/// `contenido` no empieza por un delimitador o nunca cierra.
fn bloque_frontmatter(contenido: &[u8]) -> Cow<'_, [u8]> {
    const SINTETICO: &[u8] = b"---\ntier: log\n---\n";
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return Cow::Borrowed(SINTETICO);
    };
    if !es_delimitador_frontmatter(primera) {
        return Cow::Borrowed(SINTETICO);
    }
    for &(off, linea) in &lineas[1..] {
        if !es_delimitador_frontmatter(linea) {
            continue;
        }
        let mut fin = off + linea.len();
        if fin < contenido.len() && contenido[fin] == b'\n' {
            fin += 1;
        }
        return Cow::Owned(contenido[..fin].to_vec());
    }
    Cow::Borrowed(SINTETICO)
}

/// ¿Empieza `contenido` con un delimitador de frontmatter que nunca cierra?
/// Distinto de "sin frontmatter": `aplica` (Step C) rechaza este caso en vez
/// de archivar entradas frías bajo metadata sintética que borraría en
/// silencio el tier/tags/permalink real de la nota.
fn frontmatter_sin_terminar(contenido: &[u8]) -> bool {
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return false;
    };
    if !es_delimitador_frontmatter(primera) {
        return false;
    }
    !lineas[1..].iter().any(|&(_, l)| es_delimitador_frontmatter(l))
}

fn offset_delimitador_cierre(fm: &[u8]) -> usize {
    let mut cierre = 0usize;
    for (off, linea) in lineas_con_offset(fm) {
        if es_delimitador_frontmatter(linea) {
            cierre = off;
        }
    }
    cierre
}

/// Reescribe `clave: valor` dentro de un bloque de frontmatter,
/// añadiéndola antes del delimitador de cierre si no existía.
fn reemplaza_clave(fm: &[u8], clave: &str, valor: &str) -> Vec<u8> {
    let patron = format!("(?m)^{}:.*$", regex::escape(clave));
    let re = regex::bytes::Regex::new(&patron).expect("patrón de clave válido");
    if re.is_match(fm) {
        return re
            .replace_all(fm, format!("{clave}: {valor}").as_bytes())
            .into_owned();
    }
    let cierre = offset_delimitador_cierre(fm);
    if cierre == 0 {
        return fm.to_vec();
    }
    let mut out = Vec::with_capacity(fm.len() + clave.len() + valor.len() + 4);
    out.extend_from_slice(&fm[..cierre]);
    out.extend_from_slice(format!("{clave}: {valor}\n").as_bytes());
    out.extend_from_slice(&fm[cierre..]);
    out
}

/// Envuelve `s` como escalar YAML de comilla simple: la única regla de
/// escape es doblar una comilla simple embebida, así que es seguro para
/// cualquier contenido — a diferencia del estilo "plain", donde `#`, `&`,
/// `*`, `:` o `[` cambian lo que la línea significa para un parser YAML. El
/// título/slug de una nota los escribe Paul libremente.
fn comilla_simple_yaml(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

static PATRON_FECHA_ISO: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| regex::bytes::Regex::new(r"\d{4}-\d{2}-\d{2}").expect("regex de fecha ISO"));

/// Deriva el nombre del archivo de las fechas ISO del bloque frío. Sin
/// fechas, cae a un número de secuencia (caso real:
/// desarrollo-agentico-bitacora, 26 de 60 headings sin fecha).
/// `existentes` es cuántos archivos ya hay para este slug.
pub fn nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String {
    let mut fechas: Vec<&[u8]> = PATRON_FECHA_ISO.find_iter(frio).map(|m| m.as_bytes()).collect();
    if fechas.is_empty() {
        return format!("{slug}-parte-{:02}.md", existentes + 1);
    }
    fechas.sort_unstable();
    format!(
        "{slug}-{}_{}.md",
        String::from_utf8_lossy(fechas[0]),
        String::from_utf8_lossy(fechas[fechas.len() - 1])
    )
}

/// Construye la nota archivada: el frontmatter original con `title` y
/// `permalink` reescritos, seguido del bloque frío verbatim. Con
/// `archivo_previo` no vacío, escribe un enlace hacia atrás justo después
/// del heading de título — la nota viva solo guarda UN aviso, apuntando al
/// archivo más reciente (Step C), así que cada archivo lleva el enlace
/// hacia atrás y la cadena se recorre archivo a archivo.
pub fn construye_archivo(
    frontmatter_original: &[u8],
    frio: &[u8],
    titulo: &str,
    permalink: &str,
    archivo_previo: &str,
) -> Vec<u8> {
    let mut fm = bloque_frontmatter(frontmatter_original).into_owned();
    fm = reemplaza_clave(&fm, "title", &comilla_simple_yaml(titulo));
    fm = reemplaza_clave(&fm, "permalink", &comilla_simple_yaml(permalink));

    let mut salida = Vec::with_capacity(fm.len() + frio.len() + titulo.len() + 64);
    salida.extend_from_slice(&fm);
    salida.extend_from_slice(b"\n# ");
    salida.extend_from_slice(titulo.as_bytes());
    salida.extend_from_slice(b"\n\n");
    if !archivo_previo.is_empty() {
        salida.extend_from_slice("> Continúa el histórico anterior en [[".as_bytes());
        salida.extend_from_slice(archivo_previo.as_bytes());
        salida.extend_from_slice("]].\n\n".as_bytes());
    }
    salida.extend_from_slice(frio);
    salida
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(entradas: &[&str]) -> Vec<u8> {
        let mut s = String::from("---\ntitle: x-bitacora\ntier: log\n---\n\n# x — bitácora\n\n");
        for e in entradas {
            s.push_str(e);
        }
        s.into_bytes()
    }

    #[test]
    fn mantiene_las_entradas_mas_nuevas_dentro_del_presupuesto() {
        let e1 = format!("## 2026-01-01 — uno\n{}\n\n", "a".repeat(80));
        let e2 = format!("## 2026-02-01 — dos\n{}\n\n", "b".repeat(80));
        let e3 = format!("## 2026-03-01 — tres\n{}\n\n", "c".repeat(80));
        let contenido = doc(&[&e1, &e2, &e3]);

        let p = parte(&contenido, contenido.len() as i64 - 150);

        assert_eq!(p.entradas_calientes, 1);
        assert_eq!(p.entradas_frias, 2);
        assert!(String::from_utf8_lossy(p.caliente).contains("tres"));
        assert!(String::from_utf8_lossy(p.frio).contains("uno"));
        assert!(String::from_utf8_lossy(p.frio).contains("dos"));
        assert!(String::from_utf8_lossy(p.preambulo).starts_with("---\ntitle:"));
        // Invariante duro: nada se pierde.
        let mut reconstruido = Vec::new();
        reconstruido.extend_from_slice(p.preambulo);
        reconstruido.extend_from_slice(p.frio);
        reconstruido.extend_from_slice(p.caliente);
        assert_eq!(reconstruido, contenido);
    }

    #[test]
    fn nada_rota_si_cabe() {
        let contenido = doc(&["## 2026-01-01 — uno\ncorto\n\n"]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_frias, 0);
        assert!(p.frio.is_empty());
        assert_eq!(p.entradas_calientes, 1);
    }

    #[test]
    fn siempre_queda_al_menos_una_entrada_caliente_aunque_no_quepa() {
        let vieja = "## 2026-01-01 — vieja\nx\n\n".to_string();
        let gorda = format!("## 2026-03-01 — gorda\n{}\n", "z".repeat(5000));
        let contenido = doc(&[&vieja, &gorda]);
        let p = parte(&contenido, 100);
        assert_eq!(p.entradas_calientes, 1);
        assert!(String::from_utf8_lossy(p.caliente).contains("gorda"));
    }

    #[test]
    fn sin_entradas_todo_queda_en_preambulo() {
        let contenido = b"---\ntier: log\n---\n\n# solo titulo\n".to_vec();
        let p = parte(&contenido, 10);
        assert!(p.frio.is_empty() && p.caliente.is_empty());
        assert_eq!(p.preambulo, &contenido[..]);
    }

    #[test]
    fn un_heading_dentro_de_una_valla_no_es_una_entrada() {
        let e1 = "## 2026-01-01 — uno\n```\n## no soy un heading\n```\n\n".to_string();
        let e2 = "## 2026-02-01 — dos\nx\n\n".to_string();
        let contenido = doc(&[&e1, &e2]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_calientes, 2);
    }

    #[test]
    fn nombre_de_archivo_usa_el_rango_de_fechas_iso() {
        let frio = b"## 2026-06-26 -- a\nx\n\n## Actualizacion 2026-07-11 -- b\ny\n\n";
        assert_eq!(
            nombre_de_archivo("agent-develop-bitacora", frio, 0),
            "agent-develop-bitacora-2026-06-26_2026-07-11.md"
        );
    }

    #[test]
    fn nombre_de_archivo_cae_a_numero_de_parte_sin_fechas() {
        let frio = b"## sin fecha\nx\n\n";
        assert_eq!(
            nombre_de_archivo("desarrollo-agentico-bitacora", frio, 2),
            "desarrollo-agentico-bitacora-parte-03.md"
        );
    }

    #[test]
    fn construye_archivo_conserva_tier_log_y_reescribe_title_y_permalink() {
        let fm = b"---\ntitle: agent-develop-bitacora\ntype: note\npermalink: wisdom-paul/log/agent-develop-bitacora\ntags:\n- bitacora\ntier: log\n---\n\n# agent-develop -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "agent-develop-bitacora 2026-06-26_2026-07-11", "wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\n"));
        assert!(doc.contains("title: 'agent-develop-bitacora 2026-06-26_2026-07-11'\n"));
        assert!(doc.contains("permalink: 'wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11'\n"));
        assert!(doc.contains("tier: log\n"));
        assert!(doc.contains("## 2026-06-26"));
    }

    // Pin del finding de la review de kbx: un cierre `---  \n` (espacio
    // final) no debe caer al bloque sintético ni al "title/permalink no se
    // escriben" — la misma clase de bug que motivó el `\r`/espacios en
    // `es_delimitador_frontmatter`.
    #[test]
    fn construye_archivo_tolera_espacio_final_en_el_cierre_del_frontmatter() {
        let fm = b"---\ntype: note\ntags:\n- bitacora\ntier: log\n---  \n\n# x -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "nuevo-titulo", "nuevo-permalink", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.contains("title: 'nuevo-titulo'\n"));
        assert!(doc.contains("permalink: 'nuevo-permalink'\n"));
        assert!(doc.contains("type: note\n"), "frontmatter original debe conservarse: {doc}");
    }

    #[test]
    fn construye_archivo_cae_a_sintetico_sin_frontmatter_de_origen() {
        let fm = b"# solo un heading\n\ntexto\n";
        let frio = b"## 2026-01-01 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "titulo-x", "permalink-x", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\ntier: log\n"));
        assert!(doc.contains("title: 'titulo-x'\n"));
    }

    #[test]
    fn construye_archivo_enlaza_al_archivo_previo_cuando_se_pasa() {
        let doc = construye_archivo(b"---\ntier: log\n---\n", b"## x\n", "t", "p", "bitacora-parte-01");
        assert!(String::from_utf8_lossy(&doc).contains("[[bitacora-parte-01]]"));
    }

    #[test]
    fn frontmatter_sin_terminar_detecta_el_delimitador_sin_cierre() {
        let sin_cerrar = b"---\ntitle: x\nsin cierre aqui\n";
        assert!(frontmatter_sin_terminar(sin_cerrar));
        let normal = b"---\ntier: log\n---\ncuerpo\n";
        assert!(!frontmatter_sin_terminar(normal));
        let sin_frontmatter = b"# solo un heading\n";
        assert!(!frontmatter_sin_terminar(sin_frontmatter));
    }
}
