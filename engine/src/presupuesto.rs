//! La fuente ÚNICA de los presupuestos por tier y de la aritmética de aire.
//!
//! En kbx esto vivía repartido: los nominales en dos sitios de producción sin
//! nada que los enlazara (`cmd/kbx/budget.go` y `doctor.DefaultBudgetOptions`),
//! y la clasificación de una nota reimplementada en `doctor` además de en
//! `budget`, con una regresión dirigida vigilando que no divergieran. Aquí hay
//! una sola de cada, y `budget` y `lint` las consumen las dos.

/// Los tres tiers legales, en orden de reporte.
pub const TIERS: [&str; 3] = ["core", "stable", "log"];

/// Directorios excluidos del scope de la KB, verbatim de kbx
/// (`budget.DefaultExclude` y `doctor.DefaultBudgetOptions`, que coinciden).
pub const EXCLUIDOS: [&str; 3] = [".superpowers", "archive", "docs"];

/// Sellar o bajar un techo exige que quede un 15% por encima del tamaño. Un
/// techo a ras es un mordisco programado para mañana: medido el 2026-08-17, un
/// sello con 1,1% de aire mordió al día siguiente, a mitad de cierre.
const FACTOR_AIRE_PCT: i64 = 115;

#[derive(Clone, Copy, Debug)]
pub struct Presupuestos {
    pub core: i64,
    pub stable: i64,
    pub log: i64,
}

/// Los nominales citables. `log` es 0 = ilimitado.
pub const NOMINALES: Presupuestos = Presupuestos {
    core: 8500,
    stable: 12500,
    log: 0,
};

impl Presupuestos {
    /// `None` significa "no es un tier legal" (⇒ la nota va a `notier`).
    /// `Some(0)` significa "tier legal, sin techo". Confundirlos es lo que
    /// hacía que `doctor` se saltara las notas `log` con override declarado.
    pub fn para_tier(&self, tier: &str) -> Option<i64> {
        match tier {
            "core" => Some(self.core),
            "stable" => Some(self.stable),
            "log" => Some(self.log),
            _ => None,
        }
    }
}

/// ¿Deja `techo` el margen exigido sobre `tamano`? Aritmética entera a
/// propósito: sin redondeo de float en un gate.
///
/// **`checked_mul`, no `saturating_mul`.** Rust panica en overflow en debug
/// donde Go hace wraparound silencioso, así que hay que elegir; y saturar es la
/// elección equivocada: los dos lados colapsarían a `i64::MAX` y la comparación
/// diría **que hay aire**, o sea degradación hacia verde en la única función
/// del módulo que existe para gatear. Con `checked_mul`, lo que no cabe no
/// tiene aire.
pub fn tiene_aire(techo: i64, tamano: i64) -> bool {
    match (techo.checked_mul(100), tamano.checked_mul(FACTOR_AIRE_PCT)) {
        (Some(izq), Some(der)) => izq >= der,
        _ => false,
    }
}

/// El techo más bajo que pasaría la guarda para una nota de este tamaño:
/// ceil(tamano * 1,15). Si no cabe en un `i64`, no hay techo legal: se devuelve
/// `i64::MAX`, que `tiene_aire` seguirá rechazando.
pub fn techo_minimo(tamano: i64) -> i64 {
    tamano
        .checked_mul(FACTOR_AIRE_PCT)
        .and_then(|v| v.checked_add(99))
        .map(|v| v / 100)
        .unwrap_or(i64::MAX)
}

/// El mayor tamaño admisible bajo este techo. Es el número accionable del
/// mensaje: "poda a N".
///
/// Aquí `saturating_mul` sí vale, y la diferencia con `tiene_aire` es la que
/// importa: esto **no gatea**, es una cifra para un mensaje humano. Saturar
/// produce un número absurdo en un caso imposible; saturar en la guarda
/// produciría un verde.
pub fn objetivo_poda(techo: i64) -> i64 {
    techo.saturating_mul(100) / FACTOR_AIRE_PCT
}

/// En qué cae una nota. El orden de las ramas importa: una infractora nunca se
/// reporta además como falta de aire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clase {
    /// Rebasa su presupuesto efectivo (el override si lo declaró, si no el
    /// nominal del tier). `presupuesto` es el efectivo.
    Infractora {
        presupuesto: i64,
    },
    /// Rebasaría el nominal del tier, pero su override la salva.
    /// `presupuesto` es el override.
    Waived {
        presupuesto: i64,
    },
    /// Sin override declarado y a menos del 15% de aire de su nominal.
    /// Informativo: nunca gatea. `presupuesto` es el nominal del tier.
    SinAire {
        presupuesto: i64,
    },
    Ok,
}

/// La clasificación de UNA nota. Pura a propósito: es el punto donde `budget` y
/// el check `budget_exceeded` de `lint` se encuentran, de modo que no puedan
/// divergir aunque alguien edite solo uno de los dos.
pub fn clasifica(tier_presupuesto: i64, override_max: Option<i64>, tamano: i64) -> Clase {
    let efectivo = override_max.unwrap_or(tier_presupuesto);
    if efectivo > 0 && tamano > efectivo {
        return Clase::Infractora {
            presupuesto: efectivo,
        };
    }
    if let Some(max) = override_max {
        if tier_presupuesto > 0 && tamano > tier_presupuesto {
            return Clase::Waived { presupuesto: max };
        }
        // Con techo declarado, el aire lo vigila el ratchet, no budget.
        return Clase::Ok;
    }
    if tier_presupuesto > 0 && !tiene_aire(tier_presupuesto, tamano) {
        return Clase::SinAire {
            presupuesto: tier_presupuesto,
        };
    }
    Clase::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_nominales_son_los_citables() {
        assert_eq!(NOMINALES.core, 8500);
        assert_eq!(NOMINALES.stable, 12500);
        assert_eq!(NOMINALES.log, 0);
    }

    #[test]
    fn para_tier_devuelve_none_solo_en_tier_ilegal() {
        assert_eq!(NOMINALES.para_tier("core"), Some(8500));
        assert_eq!(NOMINALES.para_tier("stable"), Some(12500));
        // log es LEGAL con presupuesto 0 (= ilimitado). None es "no es un
        // tier", que es otra cosa: la distinción decide si la nota va a
        // `notier` o si simplemente no tiene techo.
        assert_eq!(NOMINALES.para_tier("log"), Some(0));
        assert_eq!(NOMINALES.para_tier(""), None);
        assert_eq!(NOMINALES.para_tier("Core"), None);
        assert_eq!(NOMINALES.para_tier("co\u{a0}re"), None);
    }

    #[test]
    fn el_aire_es_el_115_por_ciento_en_aritmetica_entera() {
        // fe46443, internal/ratchet/check.go: techo*100 >= tamaño*115.
        assert!(tiene_aire(12500, 10869));
        assert!(!tiene_aire(12500, 10870));
        assert!(tiene_aire(8500, 7391));
        assert!(!tiene_aire(8500, 7392));
    }

    #[test]
    fn el_objetivo_de_poda_es_el_mayor_tamano_que_pasa_la_guarda() {
        assert_eq!(objetivo_poda(12500), 10869);
        assert_eq!(objetivo_poda(8500), 7391);
        // El invariante que une las dos funciones: el objetivo siempre pasa,
        // y un byte más nunca. Es lo que hace accionable el mensaje.
        for techo in [1000, 8500, 12500, 99999] {
            assert!(tiene_aire(techo, objetivo_poda(techo)));
            assert!(!tiene_aire(techo, objetivo_poda(techo) + 1));
        }
    }

    #[test]
    fn el_techo_minimo_redondea_hacia_arriba() {
        // ceil(tamaño*1.15), fe46443 minCeilingFor.
        assert_eq!(techo_minimo(10000), 11500);
        assert_eq!(techo_minimo(1), 2);
        for tamano in [1, 999, 10000, 12345] {
            assert!(tiene_aire(techo_minimo(tamano), tamano));
        }
    }

    #[test]
    fn el_overflow_no_panica_y_degrada_hacia_rojo() {
        // Rust panica en overflow en debug donde Go hace wraparound silencioso,
        // así que hay que elegir explícitamente. Y la elección importa: con
        // `saturating_mul` los dos lados colapsan a i64::MAX, la comparación
        // sale `true` y el gate diría QUE HAY AIRE. Con `checked_mul`, no.
        assert!(!tiene_aire(i64::MAX, i64::MAX));
        assert!(!tiene_aire(i64::MAX, i64::MAX / 2));
        assert!(!tiene_aire(i64::MAX / 50, i64::MAX / 50));
        // Y no revienta: devuelve el techo imposible, que `tiene_aire` rechaza.
        assert_eq!(techo_minimo(i64::MAX), i64::MAX);
        assert!(!tiene_aire(techo_minimo(i64::MAX), i64::MAX));
    }

    #[test]
    fn clasifica_prefiere_infractora_sobre_falta_de_aire() {
        // fe46443, TestRunPrefersOffenderOverAirWarning: el switch es
        // ordenado, no un conjunto de condiciones independientes.
        assert!(matches!(
            clasifica(12500, None, 13000),
            Clase::Infractora { presupuesto: 12500 }
        ));
    }

    #[test]
    fn clasifica_usa_el_override_como_presupuesto_efectivo() {
        // Sobre el override: infractora contra el override, no contra el tier.
        assert!(matches!(
            clasifica(12500, Some(9000), 9500),
            Clase::Infractora { presupuesto: 9000 }
        ));
        // Bajo el override pero sobre el nominal: waived, y el presupuesto que
        // se reporta es el OVERRIDE (kbx: Budget: override).
        assert!(matches!(
            clasifica(12500, Some(20000), 13000),
            Clase::Waived { presupuesto: 20000 }
        ));
    }

    #[test]
    fn clasifica_no_avisa_de_aire_a_quien_declaro_techo() {
        // fe46443, TestRunSkipsNotesThatDeclareAWaiver: con override
        // declarado, el aire lo vigila el ratchet, no budget.
        assert!(matches!(clasifica(12500, Some(20000), 12000), Clase::Ok));
        // Sin override y a ras: SinAire, reportando el NOMINAL del tier.
        assert!(matches!(
            clasifica(12500, None, 12000),
            Clase::SinAire { presupuesto: 12500 }
        ));
    }

    #[test]
    fn clasifica_no_toca_los_tiers_ilimitados() {
        // fe46443, TestRunSkipsUnlimitedTiers: presupuesto 0 = ilimitado, ni
        // infractora ni aviso de aire. Pero un override SÍ dispara sobre log,
        // que es la regresión que doctor se había dejado (Corrección 4).
        assert!(matches!(clasifica(0, None, 999_999), Clase::Ok));
        assert!(matches!(
            clasifica(0, Some(50), 100),
            Clase::Infractora { presupuesto: 50 }
        ));
    }

    #[test]
    fn los_presupuestos_son_de_verdad_un_parametro() {
        // `analiza` y los checks toman `Presupuestos` en vez de leer NOMINALES
        // directamente. Si ningún test usa otros valores, el parámetro es
        // config especulativa y sobra: este lo ejerce, como hace el Go.
        let apretados = Presupuestos {
            core: 100,
            stable: 200,
            log: 50,
        };
        assert_eq!(apretados.para_tier("core"), Some(100));
        assert!(matches!(
            clasifica(apretados.para_tier("log").unwrap(), None, 60),
            Clase::Infractora { presupuesto: 50 }
        ));
        // Con el nominal real, esa misma nota de 60 bytes no es nada.
        assert!(matches!(clasifica(NOMINALES.log, None, 60), Clase::Ok));
    }

    #[test]
    fn el_borde_del_presupuesto_es_estrictamente_mayor() {
        // fe46443, TestRunBudgetCoreBoundaryStrictGreaterThan: 8500 exacto NO
        // es infractora.
        assert!(!matches!(
            clasifica(8500, None, 8500),
            Clase::Infractora { .. }
        ));
        assert!(matches!(
            clasifica(8500, None, 8501),
            Clase::Infractora { presupuesto: 8500 }
        ));
    }
}
