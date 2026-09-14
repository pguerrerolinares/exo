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
}
