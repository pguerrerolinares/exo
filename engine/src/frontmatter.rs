//! Frontmatter con el contrato exacto de `kbx/internal/frontmatter`, que NO es
//! el de `nota.rs`.
//!
//! `nota::parsea_nota` parsea YAML de verdad y devuelve `None` si falta
//! `permalink`: correcto para indexar, inservible para un gate, porque
//! descartaría justo las notas rotas que el gate existe para encontrar. kbx
//! renunció a la librería YAML a propósito ("verdict v5: no YAML library"):
//! escanea líneas, nunca devuelve error y degrada todo a ausente. Los
//! comandos portados en G4 dependen de esa semántica, así que se replica aquí
//! en vez de reusar la de casa.
//!
//! Los marcadores conservan el prefijo histórico `kbx_` (`kbx_budget_max`,
//! `kbx_orphan_ok`): están escritos en 11 notas vivas y renombrarlos sería una
//! migración de datos a cambio de nada (spec G4, "Nombres que NO se tocan").

/// Recorre el bloque de frontmatter llamando a `f(clave, valor_crudo)` hasta
/// que `f` devuelve `true` o el bloque se cierra.
///
/// El fichero **debe empezar** por la línea delimitadora: un BOM o una línea
/// en blanco delante y el bloque se considera ausente. Es el comportamiento de
/// kbx y degrada hacia rojo (la nota cae en NOTIER), nunca hacia verde.
fn escanea(contenido: &str, mut f: impl FnMut(&str, &str) -> bool) {
    let mut lineas = contenido.lines();
    match lineas.next() {
        Some(primera) if es_delimitador(primera) => {}
        _ => return,
    }
    for linea in lineas {
        if es_delimitador(linea) {
            return;
        }
        let Some((clave, crudo)) = linea.split_once(':') else {
            continue;
        };
        if f(clave.trim(), crudo.trim_start_matches([' ', '\t'])) {
            return;
        }
    }
}

/// Una línea que es `---` seguida solo de whitespace, igual que el
/// `/^---[[:space:]]*$/` del awk original.
///
/// El `\r` es parte de esa clase y hay que recortarlo: `str::lines()` ya quita
/// el `\r` de un `\r\n`, pero no el de un `---\r\r` ni el de una línea que
/// llegue por otra vía. Sin esto, en un checkout CRLF el delimitador de
/// apertura no matchea nunca, el escaneo devuelve **nada** —ni tier, ni
/// waivers— y todos los comandos siguen saliendo con exit 0. Es el fallo
/// silencioso canónico del port (kbx `5c7eb3d`).
fn es_delimitador(linea: &str) -> bool {
    linea.trim_end_matches([' ', '\t', '\r']) == "---"
}

/// Los seis caracteres que kbx borra del `tier`
/// (`internal/frontmatter/frontmatter.go`, `stripWhitespace`). ASCII a
/// propósito, no `char::is_whitespace()`: un NBSP dentro de `tier: co<NBSP>re`
/// debe dejar el tier ILEGAL (⇒ `notier`, gate visible) en vez de repararlo en
/// silencio y colarlo como `core`. Degradación hacia rojo, como el resto del
/// módulo (A1 del plan de G4b).
const ESPACIOS_ASCII: [char; 6] = [' ', '\t', '\n', '\r', '\u{b}', '\u{c}'];

/// El `tier` declarado, o `""` si no hay clave.
///
/// Quita **todo** el whitespace, no solo los extremos: mimetiza el
/// `tr -d '[:space:]'` del `kb-budget-check.sh` retirado, así que `co re` se
/// normaliza a `core` en vez de fallar. La legalidad del valor no se juzga
/// aquí — eso es trabajo del llamante.
pub fn tier(contenido: &str) -> String {
    let mut salida = String::new();
    escanea(contenido, |clave, crudo| {
        if clave == "tier" {
            salida = crudo
                .chars()
                .filter(|c| !ESPACIOS_ASCII.contains(c))
                .collect();
            return true;
        }
        false
    });
    salida
}

/// El valor crudo de `clave`, recortado solo por los extremos.
pub fn valor(contenido: &str, clave: &str) -> Option<String> {
    let mut salida = None;
    escanea(contenido, |k, crudo| {
        if k == clave {
            salida = Some(crudo.trim_end_matches([' ', '\t', '\r']).to_string());
            return true;
        }
        false
    });
    salida
}

/// El techo declarado en `kbx_budget_max`.
///
/// `Some` **solo** con un entero positivo: `0`, negativo, vacío o no numérico
/// se ignoran en silencio y manda el nominal del tier. Ojo con la trampa: `0`
/// no significa "sin límite" (esa es la semántica del nominal de tier `log`),
/// significa "no hay declaración".
pub fn budget_max(contenido: &str) -> Option<i64> {
    let crudo = valor(contenido, "kbx_budget_max")?;
    match crudo.trim().parse::<i64>() {
        Ok(n) if n > 0 => Some(n),
        _ => None,
    }
}

/// Si la nota declara el marcador `kbx_orphan_ok: true`.
///
/// Solo el literal exacto `true` waiva: `True`, `yes` y `1` no.
pub fn orphan_ok(contenido: &str) -> bool {
    valor(contenido, "kbx_orphan_ok").is_some_and(|v| v.trim() == "true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_lee_el_valor_del_frontmatter() {
        let c = "---\ntier: core\ntitle: x\n---\ncuerpo\n";
        assert_eq!(tier(c), "core");
    }

    // El invariante más letal del port (kbx commit 5c7eb3d): en un checkout
    // CRLF el delimitador llega como "---\r". Sin tolerarlo, el frontmatter
    // entero se lee como AUSENTE y todos los comandos siguen saliendo 0.
    // Medido en kbx sobre la KB real: 88 notas NOTIER, waivers ignorados,
    // `budget` respondiendo ok con notes=0 en los tres tiers.
    #[test]
    fn tier_sobrevive_a_un_fichero_con_crlf() {
        let c = "---\r\ntier: core\r\ntitle: x\r\n---\r\ncuerpo\r\n";
        assert_eq!(tier(c), "core");
    }

    // El test de arriba NO falsa la guarda: `str::lines()` ya se come el `\r`
    // de un `\r\n`, así que pasaría igual con `es_delimitador` comparando
    // contra `"---"` a pelo. Este sí la falsa. Un `\r` que `lines()` no
    // absorbe —el primero de un `\r\r\n`— deja la línea como `"---\r"`, que
    // es exactamente lo que Go ve al partir por `"\n"` y lo que motivó el
    // commit `5c7eb3d`. Quita el `trim_end_matches` de `es_delimitador` y
    // este test se pone rojo; el otro no.
    #[test]
    fn el_trim_del_delimitador_es_lo_que_sostiene_el_caso_crlf() {
        let c = "---\r\r\ntier: core\r\r\n---\r\r\n";
        assert_eq!(tier(c), "core");
    }

    // Go hace stripWhitespace (tr -d '[:space:]'), no un trim: replica el awk
    // original de kb-budget-check.sh. Un tier con espacios internos se
    // normaliza en vez de fallar.
    #[test]
    fn tier_quita_todo_el_whitespace_incluido_el_interno() {
        let c = "---\ntier:   co re \n---\n";
        assert_eq!(tier(c), "core");
    }

    // La legalidad NO es responsabilidad de este módulo ("Tier() only
    // extracts; legality is the caller's job").
    #[test]
    fn tier_deja_pasar_un_valor_ilegal_tal_cual() {
        let c = "---\ntier: banana\n---\n";
        assert_eq!(tier(c), "banana");
    }

    #[test]
    fn el_tier_no_absuelve_el_whitespace_unicode() {
        // A1: el NBSP NO se filtra, así que el tier queda ilegal y la nota cae en
        // `notier` — el gate salta y se ve. Filtrarlo (char::is_whitespace) la
        // colaría como `core` en silencio: degradación hacia verde, justo lo que
        // el resto de este módulo evita a propósito.
        let contenido = "---\ntier: co\u{a0}re\n---\n";
        assert_eq!(tier(contenido), "co\u{a0}re");
    }

    // El predecesor de este test ("los seis, en cualquier posición") metía
    // los seis caracteres en una sola cadena con el '\r' pegado al '\n' de
    // cierre. `str::lines()` ya absorbe ese '\r' como parte del terminador
    // `\r\n` antes de que `escanea` vea la línea, así que nunca llegaba al
    // filtro de `tier()`: borra '\r' de `ESPACIOS_ASCII` y este test seguía
    // en verde. Una cadena única tampoco habría delatado cuál faltaba —falla
    // en bloque—, así que aquí cada carácter se falsa por separado.
    //
    // De los seis de `ESPACIOS_ASCII` solo cinco son alcanzables: `escanea`
    // extrae el crudo de una única línea (`contenido.lines()`), y una línea
    // por definición no puede contener un '\n' interno —es el propio
    // separador—. Insertarlo en "tier: c\nore" no cuela un '\n' hasta el
    // filtro: parte la clave en dos líneas ("tier: c" y "ore"), y `escanea`
    // ve `crudo == "c"`, nunca "c\nore". No hay fixture que haga llegar un
    // '\n' al filtro, así que no es falsable y no entra en el bucle. Se queda
    // en `ESPACIOS_ASCII` solo por paridad literal con el `stripWhitespace`
    // de Go (switch sobre ' ', '\t', '\n', '\r', '\v', '\f'), no porque haya
    // un hueco de cobertura ni un candidato a que alguien lo "limpie".
    #[test]
    fn el_tier_filtra_cada_espacio_ascii_alcanzable_por_separado() {
        let alcanzables = [' ', '\t', '\r', '\u{b}', '\u{c}'];
        for c in alcanzables {
            // El carácter va en medio de "c<c>ore", nunca al final de la
            // línea: un '\r' como último carácter de la línea sí lo absorbe
            // `lines()` (es la trampa del test anterior), pero uno seguido de
            // más contenido antes del '\n' sobrevive intacto hasta `crudo`.
            let contenido = format!("---\ntier: c{c}ore\n---\n");
            assert_eq!(
                tier(&contenido),
                "core",
                "el caracter {c:?} deberia haber sido filtrado de tier()"
            );
        }
    }

    #[test]
    fn el_tier_limpio_no_cambia() {
        assert_eq!(tier("---\ntier: stable\n---\n"), "stable");
    }

    #[test]
    fn sin_frontmatter_no_hay_nada() {
        assert_eq!(tier("cuerpo suelto\ntier: core\n"), "");
        assert_eq!(valor("cuerpo suelto\ntier: core\n", "tier"), None);
    }

    // El fichero DEBE empezar por la línea delimitadora. Un BOM UTF-8 delante
    // hace que no se detecte y el frontmatter se lea como ausente. Es el
    // comportamiento de kbx y se replica a propósito: degrada hacia rojo (la
    // nota cae en NOTIER y el gate grita), no hacia verde.
    #[test]
    fn un_bom_delante_del_delimitador_anula_el_bloque() {
        let c = "\u{feff}---\ntier: core\n---\n";
        assert_eq!(tier(c), "");
    }

    #[test]
    fn el_bloque_se_cierra_y_lo_de_despues_no_cuenta() {
        let c = "---\ntitle: x\n---\ntier: log\n";
        assert_eq!(tier(c), "");
    }

    // Refleja el awk original: no exige el delimitador de cierre para
    // encontrar una clave que ya ha visto.
    #[test]
    fn encuentra_la_clave_aunque_el_bloque_no_se_cierre() {
        let c = "---\ntier: stable\n";
        assert_eq!(tier(c), "stable");
    }

    #[test]
    fn valor_recorta_solo_los_extremos_y_preserva_el_interior() {
        let c = "---\ntitle:   hola   mundo  \n---\n";
        assert_eq!(valor(c, "title").as_deref(), Some("hola   mundo"));
    }

    #[test]
    fn valor_corta_por_el_primer_dos_puntos() {
        let c = "---\npermalink: kb/a:b\n---\n";
        assert_eq!(valor(c, "permalink").as_deref(), Some("kb/a:b"));
    }

    // Solo entero positivo activa el techo — fail-toward-red. Ojo: 0 NO es
    // "sin límite", es "ignora el override y usa el nominal del tier".
    #[test]
    fn budget_max_solo_acepta_entero_positivo() {
        assert_eq!(budget_max("---\nkbx_budget_max: 19000\n---\n"), Some(19000));
        assert_eq!(budget_max("---\nkbx_budget_max: 0\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max: -5\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max: basura\n---\n"), None);
        assert_eq!(budget_max("---\nkbx_budget_max:\n---\n"), None);
        assert_eq!(budget_max("---\ntier: core\n---\n"), None);
    }

    // Solo el literal exacto "true" waiva. "True", "yes", "1" no.
    #[test]
    fn orphan_ok_solo_con_el_literal_true() {
        assert!(orphan_ok("---\nkbx_orphan_ok: true\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: True\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: yes\n---\n"));
        assert!(!orphan_ok("---\nkbx_orphan_ok: 1\n---\n"));
        assert!(!orphan_ok("---\ntier: core\n---\n"));
    }
}
