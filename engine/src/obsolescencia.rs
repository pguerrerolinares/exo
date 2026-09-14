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

/// Días desde 1970-01-01 hasta la fecha civil dada (calendario
/// gregoriano). Algoritmo de Howard Hinnant
/// (howardhinnant.github.io/date_algorithms.html, dominio público);
/// aritmética entera exacta para cualquier año, incluidos los anteriores a
/// 1970 (da negativo). exo no trae ningún crate de fechas —
/// `indexer::git_epoch_de` sortea el problema pidiéndole el epoch a git
/// directamente (`%at`); aquí hace falta además la cadena ISO para
/// `last_commit`, así que se resuelve con esta función pura en vez de
/// añadir una dependencia para dos conversiones de calendario.
fn dias_desde_epoch_civil(anio: i64, mes: i64, dia: i64) -> i64 {
    let y = if mes <= 2 { anio - 1 } else { anio };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (mes + 9) % 12;
    let doy = (153 * mp + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverso de `dias_desde_epoch_civil`: fecha civil para un número de días
/// desde 1970-01-01.
fn civil_desde_dias_epoch(dias: i64) -> (i64, i64, i64) {
    let z = dias + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn parsea_offset_minutos(off: &str) -> Result<i64> {
    let (signo_txt, resto) = off.split_at(1);
    let signo = if signo_txt == "-" { -1 } else { 1 };
    let mut partes = resto.split(':');
    let (Some(Ok(hh)), Some(Ok(mm))) = (partes.next().map(str::parse::<i64>), partes.next().map(str::parse::<i64>)) else {
        anyhow::bail!("huso horario ilegible: {off:?}");
    };
    Ok(signo * (hh * 60 + mm))
}

/// Convierte una fecha-hora en formato `git log --format=%aI` (ISO-8601
/// estricto: `YYYY-MM-DDTHH:MM:SS±HH:MM`, el que produce
/// `gitx::ultimo_commit`, o con sufijo `Z`) a segundos UTC desde epoch.
pub fn epoch_utc_de_iso8601(marca: &str) -> Result<i64> {
    let (fecha, resto) = marca.split_once('T').with_context(|| format!("fecha ISO-8601 sin 'T': {marca:?}"))?;
    let mut pf = fecha.split('-');
    let (Some(Ok(anio)), Some(Ok(mes)), Some(Ok(dia))) =
        (pf.next().map(str::parse::<i64>), pf.next().map(str::parse::<i64>), pf.next().map(str::parse::<i64>))
    else {
        anyhow::bail!("fecha ISO-8601 ilegible: {marca:?}");
    };

    let (hms, offset_min) = if let Some(hms) = resto.strip_suffix('Z') {
        (hms, 0)
    } else if let Some(i) = resto.rfind(['+', '-']) {
        let (hms, off) = resto.split_at(i);
        (hms, parsea_offset_minutos(off)?)
    } else {
        anyhow::bail!("fecha ISO-8601 sin huso horario: {marca:?}");
    };

    let mut ph = hms.split(':');
    let (Some(Ok(h)), Some(Ok(m)), Some(Ok(s))) =
        (ph.next().map(str::parse::<i64>), ph.next().map(str::parse::<i64>), ph.next().map(str::parse::<i64>))
    else {
        anyhow::bail!("hora ISO-8601 ilegible: {marca:?}");
    };

    let dias = dias_desde_epoch_civil(anio, mes, dia);
    Ok(dias * 86_400 + h * 3600 + m * 60 + s - offset_min * 60)
}

/// Formatea segundos-epoch UTC como RFC3339 con sufijo `Z`, sin fracción
/// de segundo — igual que `now.UTC().Format(time.RFC3339)` en Go.
pub fn formatea_rfc3339_utc(epoch: i64) -> String {
    let dias = epoch.div_euclid(86_400);
    let seg_del_dia = epoch.rem_euclid(86_400);
    let (anio, mes, dia) = civil_desde_dias_epoch(dias);
    let (h, m, s) = (seg_del_dia / 3600, (seg_del_dia / 60) % 60, seg_del_dia % 60);
    format!("{anio:04}-{mes:02}-{dia:02}T{h:02}:{m:02}:{s:02}Z")
}

/// `floor((ahora - commit) / 86400s)` sobre instantes UTC absolutos
/// (spec-review M1: "duration-based, never calendar-day arithmetic in any
/// timezone"). `div_euclid` para que un `ahora` anterior al commit
/// redondee hacia abajo, no hacia cero.
fn edad_en_dias(ahora_epoch: i64, commit_epoch: i64) -> i64 {
    (ahora_epoch - commit_epoch).div_euclid(86_400)
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

    #[test]
    fn epoch_de_iso8601_reproduce_el_now_del_golden() {
        // "2026-07-14T22:00:00Z", el `now` del golden de kbx.
        let e = epoch_utc_de_iso8601("2026-07-14T22:00:00Z").unwrap();
        assert_eq!(formatea_rfc3339_utc(e), "2026-07-14T22:00:00Z");
    }

    #[test]
    fn epoch_de_iso8601_respeta_el_huso_horario() {
        // 10:00 +02:00 == 08:00 Z.
        let con_offset = epoch_utc_de_iso8601("2026-06-01T10:00:00+02:00").unwrap();
        let en_z = epoch_utc_de_iso8601("2026-06-01T08:00:00Z").unwrap();
        assert_eq!(con_offset, en_z);
    }

    #[test]
    fn formatea_rfc3339_utc_es_el_inverso_de_epoch_utc_de_iso8601() {
        for marca in ["1970-01-01T00:00:00Z", "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z", "2000-02-29T12:00:00Z"] {
            let e = epoch_utc_de_iso8601(marca).unwrap();
            assert_eq!(formatea_rfc3339_utc(e), marca, "round-trip de {marca}");
        }
    }

    #[test]
    fn edad_en_dias_coincide_con_el_golden() {
        // now=2026-07-14T22:00:00Z, last_commit core/core-index=2026-06-01T10:00:00+02:00 => age_days=43.
        let ahora = epoch_utc_de_iso8601("2026-07-14T22:00:00Z").unwrap();
        let commit = epoch_utc_de_iso8601("2026-06-01T10:00:00+02:00").unwrap();
        assert_eq!(edad_en_dias(ahora, commit), 43);
    }
}
