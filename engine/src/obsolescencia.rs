//! `exo stale` — urgencia de actualización de cada nota: edad de su
//! último commit, grado en el grafo de relaciones y tier, combinados en
//! una puntuación. Puerto de `kbx/internal/stale` (`fe46443`). La fórmula
//! y los pesos están FIRMADOS por Paul
//! (`.superpowers/fabrica/PENDIENTE-PAUL-m4-stale-formula.md`,
//! "Adjudicación de Paul", 2026-07-11) — no son una decisión de esta
//! campaña, se copian verbatim.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

/// Edad fija para una nota sin commits (spec-review H1: constante, nunca
/// relativa a la corrida ni a la composición del fixture).
pub const EDAD_SIN_COMMIT_DIAS: i64 = 36_500;

/// Pesos de la fórmula (bloque agrupado, un re-peso es un edit de una
/// línea — mismo contrato que el `const` block de kbx). `PESO_TIER_CORE`/
/// `STABLE`/`LOG` escalan la edad cruda por tier; `PESO_TIER_SIN_TIER`
/// (tier ausente o ilegal) puntúa como `log`, la urgencia más baja —
/// decisión ya tomada: `exo budget` ya expone `notier` como su propia
/// alarma de higiene, y catapultar NOTIER al tope del ranking de `stale`
/// diluiría la señal que `stale` existe para dar. `PESO_DECAIMIENTO_DEGREE`
/// descuenta la urgencia por conectividad como `1/(1+degree*peso)`: degree
/// 0 no divide por cero, y el descuento crece monótono con degree.
pub const PESO_TIER_CORE: f64 = 1.5;
pub const PESO_TIER_STABLE: f64 = 1.0;
pub const PESO_TIER_LOG: f64 = 0.5;
pub const PESO_TIER_SIN_TIER: f64 = PESO_TIER_LOG;
pub const PESO_DECAIMIENTO_DEGREE: f64 = 0.2;

fn peso_de_tier(tier: &str) -> f64 {
    match tier {
        "core" => PESO_TIER_CORE,
        "stable" => PESO_TIER_STABLE,
        "log" => PESO_TIER_LOG,
        _ => PESO_TIER_SIN_TIER,
    }
}

/// Urgencia de una nota, redondeada a 2 decimales antes de envolver.
/// Serializa como un `f64` normal (formato más corto que redondea exacto,
/// p. ej. `29.0` en vez de `29.00`). kbx usa `Score.MarshalJSON` para fijar
/// el texto en 2 decimales; el port **no** replica ese formateo — el
/// feature de `serde_json` que haría falta rompe `#[serde(untagged)]` con
/// floats en todo el grafo de dependencias (`tokenizers` lo usa en 12
/// sitios, ver Global Constraints). El valor es idéntico, solo el texto
/// difiere — divergencia 5 del pre-registro.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
pub struct Puntuacion(f64);

impl Puntuacion {
    pub fn valor(self) -> f64 {
        self.0
    }
}

/// Combina edad, degree y tier en una urgencia (M4 spec §3, "Formula"):
/// edad escalada por el peso del tier, descontada por conectividad.
/// Redondeada a 2 decimales antes de envolver.
pub fn puntua(edad_dias: i64, degree: i64, tier: &str) -> Puntuacion {
    let cruda = edad_dias as f64 * peso_de_tier(tier) / (1.0 + degree as f64 * PESO_DECAIMIENTO_DEGREE);
    Puntuacion((cruda * 100.0).round() / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puntua_reproduce_el_golden_de_kbx() {
        // Nueve pares únicos (edad, degree, tier) => score, verbatim de
        // cmd/kbx/testdata/stale_golden.json (fe46443) — dos de las 10
        // notas del golden repiten el mismo par (log/alpha-bitacora y
        // sesiones/2026-07-01, age_days=13/degree=2/tier=log => 4.64), así
        // que hay 9 pares únicos, no 10.
        let casos = [
            (43, 2, "core", 46.07),
            (43, 1, "stable", 35.83),
            (29, 0, "stable", 29.00),
            (29, 0, "", 14.50),
            (29, 5, "stable", 14.50),
            (29, 1, "", 12.08),
            (13, 1, "stable", 10.83),
            (29, 2, "", 10.36),
            (13, 2, "log", 4.64),
        ];
        for (edad, degree, tier, esperado) in casos {
            let got = puntua(edad, degree, tier).valor();
            assert!(
                (got - esperado).abs() < 1e-9,
                "puntua({edad}, {degree}, {tier:?}) = {got}, want {esperado}"
            );
        }
    }

    // --- Axiomas de la spec M4 §3 (mismos 5 que TestScore_Axiom{1..5} en kbx) ---

    const TIERS: [&str; 4] = ["core", "stable", "log", ""];

    #[test]
    fn axioma_1_determinismo() {
        for tier in TIERS {
            for edad in [0, 1, 13, 29, 43, EDAD_SIN_COMMIT_DIAS] {
                for degree in [0, 1, 2, 4, 10] {
                    let primero = puntua(edad, degree, tier).valor();
                    for _ in 0..5 {
                        assert_eq!(puntua(edad, degree, tier).valor(), primero);
                    }
                }
            }
        }
    }

    #[test]
    fn axioma_2_monotonia_de_edad() {
        let edades = [0, 1, 5, 13, 29, 43, 100, 1000, EDAD_SIN_COMMIT_DIAS];
        for tier in TIERS {
            for degree in [0, 1, 2, 4, 10] {
                for w in edades.windows(2) {
                    let (joven, vieja) = (puntua(w[0], degree, tier).valor(), puntua(w[1], degree, tier).valor());
                    assert!(vieja >= joven, "tier={tier} degree={degree}: {vieja} < {joven}");
                }
            }
        }
    }

    #[test]
    fn axioma_3_monotonia_de_degree() {
        let degrees = [0, 1, 2, 3, 4, 10, 50];
        for tier in TIERS {
            for edad in [1, 13, 29, 43, 1000] {
                for w in degrees.windows(2) {
                    let (menos, mas) = (puntua(edad, w[0], tier).valor(), puntua(edad, w[1], tier).valor());
                    assert!(mas <= menos, "tier={tier} edad={edad}: {mas} > {menos}");
                }
            }
        }
    }

    #[test]
    fn axioma_4_orden_de_tier() {
        for edad in [1, 13, 29, 43, 1000] {
            for degree in [0, 1, 2, 4, 10] {
                let core = puntua(edad, degree, "core").valor();
                let stable = puntua(edad, degree, "stable").valor();
                let log = puntua(edad, degree, "log").valor();
                assert!(core >= stable, "edad={edad} degree={degree}: core {core} < stable {stable}");
                assert!(stable >= log, "edad={edad} degree={degree}: stable {stable} < log {log}");
            }
        }
    }

    #[test]
    fn axioma_5_forma_finita_no_negativa_y_dos_decimales() {
        for tier in TIERS {
            for edad in [0, 1, 13, 1000, EDAD_SIN_COMMIT_DIAS] {
                for degree in [0, 1, 4, 50] {
                    let s = puntua(edad, degree, tier).valor();
                    assert!(s.is_finite() && s >= 0.0, "puntua({edad},{degree},{tier:?}) = {s}");
                }
            }
        }
    }
}
