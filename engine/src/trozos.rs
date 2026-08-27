//! Chunking propio del engine (spec `2026-07-17-indexer-design.md` §2.1,
//! literal): unidad = bloques markdown (separados por línea en blanco o
//! heading), empaquetado greedy de bloques consecutivos hasta máx 900 chars
//! por trozo; un bloque que solo exceda el máximo se corta duro en 900.
//! Solape 0. Provisional (parámetro del sweep M2-06/07), pero el contrato de
//! ESTE item es implementarlo tal cual.

/// Techo de un trozo, en caracteres Unicode (no bytes) — spec §2.1.
const MAX_CHARS: usize = 900;

/// Trocea el cuerpo de una nota en trozos de texto listos para embeber.
/// Nota vacía (o solo whitespace) → 0 trozos. Determinista: mismo input
/// produce siempre los mismos trozos, en el mismo orden.
pub fn trocea(cuerpo: &str) -> Vec<String> {
    let bloques = bloques_markdown(cuerpo);

    let mut trozos: Vec<String> = Vec::new();
    let mut actual: Vec<String> = Vec::new();
    let mut len_actual = 0usize;

    for bloque in bloques {
        let n = bloque.chars().count();

        if n > MAX_CHARS {
            if !actual.is_empty() {
                trozos.push(actual.join("\n\n"));
                actual = Vec::new();
                len_actual = 0;
            }
            trozos.extend(corta_duro(&bloque));
            continue;
        }

        let separador = if actual.is_empty() { 0 } else { 2 }; // "\n\n"
        if len_actual + separador + n <= MAX_CHARS {
            len_actual += separador + n;
            actual.push(bloque);
        } else {
            trozos.push(actual.join("\n\n"));
            actual = vec![bloque];
            len_actual = n;
        }
    }

    if !actual.is_empty() {
        trozos.push(actual.join("\n\n"));
    }

    trozos
}

/// Divide `cuerpo` en bloques markdown: una línea en blanco cierra el bloque
/// acumulado; una línea heading (ATX `#`..`######` + espacio, o heading
/// "vacío" sin texto tras los `#`) también lo cierra (aunque no venga
/// precedida de línea en blanco) y arranca uno nuevo con la propia línea de
/// heading como primera línea de ese bloque nuevo.
fn bloques_markdown(cuerpo: &str) -> Vec<String> {
    let mut bloques = Vec::new();
    let mut actual: Vec<&str> = Vec::new();

    for linea in cuerpo.lines() {
        if linea.trim().is_empty() {
            if !actual.is_empty() {
                bloques.push(actual.join("\n"));
                actual.clear();
            }
            continue;
        }

        if es_heading(linea) && !actual.is_empty() {
            bloques.push(actual.join("\n"));
            actual.clear();
        }

        actual.push(linea);
    }

    if !actual.is_empty() {
        bloques.push(actual.join("\n"));
    }

    bloques
}

/// `true` si `linea` es un heading ATX (`# `..`###### `, o `#`..`######`
/// solos al final de línea sin texto).
fn es_heading(linea: &str) -> bool {
    let t = linea.trim_start();
    let hashes = t.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return false;
    }
    matches!(t.as_bytes().get(hashes), Some(b' ') | None)
}

/// Corte duro de un bloque que por sí solo excede `MAX_CHARS`: trozos de
/// exactamente `MAX_CHARS` caracteres (el último puede ser más corto), sin
/// solape. Trabaja sobre `Vec<char>` para no partir un carácter UTF-8
/// multibyte a la mitad.
fn corta_duro(bloque: &str) -> Vec<String> {
    let chars: Vec<char> = bloque.chars().collect();
    chars
        .chunks(MAX_CHARS)
        .map(|c| c.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nota_vacia_da_cero_trozos() {
        assert_eq!(trocea(""), Vec::<String>::new());
        assert_eq!(trocea("   \n\n  \n"), Vec::<String>::new());
    }

    #[test]
    fn bloque_simple_da_un_trozo() {
        assert_eq!(
            trocea("un párrafo simple de texto"),
            vec!["un párrafo simple de texto"]
        );
    }

    #[test]
    fn empaquetado_greedy_junta_bloques_bajo_el_techo() {
        let cuerpo = "primer bloque\n\nsegundo bloque\n\ntercer bloque";
        let trozos = trocea(cuerpo);
        assert_eq!(
            trozos,
            vec!["primer bloque\n\nsegundo bloque\n\ntercer bloque"]
        );
    }

    #[test]
    fn empaquetado_respeta_900_y_abre_trozo_nuevo() {
        let bloque_a = "a".repeat(500);
        let bloque_b = "b".repeat(500);
        let cuerpo = format!("{bloque_a}\n\n{bloque_b}");
        let trozos = trocea(&cuerpo);
        // 500 + 2 + 500 = 1002 > 900: no caben juntos en un trozo.
        assert_eq!(trozos, vec![bloque_a, bloque_b]);
    }

    #[test]
    fn empaquetado_junta_cuando_cabe_bajo_900() {
        let bloque_a = "a".repeat(400);
        let bloque_b = "b".repeat(400);
        let cuerpo = format!("{bloque_a}\n\n{bloque_b}");
        let trozos = trocea(&cuerpo);
        // 400 + 2 + 400 = 802 <= 900: caben juntos.
        assert_eq!(trozos, vec![format!("{bloque_a}\n\n{bloque_b}")]);
    }

    #[test]
    fn bloque_que_excede_900_se_corta_duro_sin_solape() {
        let bloque = "x".repeat(1000);
        let trozos = trocea(&bloque);
        assert_eq!(trozos.len(), 2);
        assert_eq!(trozos[0].chars().count(), 900);
        assert_eq!(trozos[1].chars().count(), 100);
        // sin solape: concatenar los trozos reproduce el bloque original
        assert_eq!(format!("{}{}", trozos[0], trozos[1]), bloque);
    }

    #[test]
    fn heading_actua_como_separador_de_bloque() {
        let cuerpo = "texto antes\n# Un heading\ntexto después del heading";
        let bloques = bloques_markdown(cuerpo);
        assert_eq!(
            bloques,
            vec![
                "texto antes".to_string(),
                "# Un heading\ntexto después del heading".to_string(),
            ]
        );
    }

    #[test]
    fn heading_sin_blanco_previo_tambien_separa() {
        // Sin línea en blanco entre el párrafo y el heading: el heading
        // igual cierra el bloque anterior (spec: "separados por línea en
        // blanco O heading").
        let cuerpo = "## Segundo heading\ncontenido";
        let trozos = trocea(cuerpo);
        assert_eq!(trozos, vec!["## Segundo heading\ncontenido"]);
    }

    #[test]
    fn determinismo_mismo_input_mismos_trozos() {
        let cuerpo = "# H1\ntexto 1\n\ntexto 2\n\n## H2\ntexto 3";
        assert_eq!(trocea(cuerpo), trocea(cuerpo));
    }
}
