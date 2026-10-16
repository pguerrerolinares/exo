# Plan: `rotate` nombra el archivo por las fechas de las cabeceras (#95)

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que el nombre del archivo rotado salga solo de las fechas de las cabeceras `## ` del bloque frío, no de cualquier fecha ISO del cuerpo.

**Architecture:** cambio local en `nombre_de_archivo` (`engine/src/rotacion.rs:209`). Las cabeceras se detectan con la misma lógica que `offsets_de_entradas` (`rotacion.rs:52`, ignora vallas ```), así "qué es una entrada" es idéntico al cortar y al nombrar. El corte sigue siendo posicional; la firma y los dos call sites (`rotacion.rs:511`, `rotacion.rs:539`) no cambian.

**Tech Stack:** Rust (crate `engine`), `regex::bytes` (`PATRON_FECHA_ISO`, `rotacion.rs:197`).

**Plan:** ~10 min, 4 turnos, ~4 KB

**Spec:** sin spec escrita (cambio pequeño, diseño aprobado en chat el 2026-10-16); fuente: issue #95.

**Global Constraints:**
- De cada cabecera `## ` fuera de valla se toma solo la PRIMERA fecha ISO de esa línea.
- El rango es min..max de esas fechas: `{slug}-{min}_{max}.md`.
- Si ninguna cabecera lleva fecha → `{slug}-parte-{NN:02}.md` con NN = `existentes + 1` (como hoy). Sin fallback a fechas del cuerpo.
- Sin filtro de fechas futuras.
- Archivos ya creados no se renombran (no hay migración).
- Fuera de alcance: el conteo de `existentes` para `parte-NN` (`rotacion.rs:332-339`) y las colisiones del mismo día (`desambigua_nombre_de_archivo`).

## Olas

- Ola 1: T1 (única tarea).

### Task 1: `nombre_de_archivo` usa solo fechas de cabeceras

**Files:**
- Modify: `engine/src/rotacion.rs`
- Test: `engine/src/rotacion.rs`

**Interfaces:**
- Produces: `pub fn nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String` (firma sin cambios)

**Tests:** (en `mod tests` de `rotacion.rs`; cada test nuevo se ve fallar antes del código, `exo:tdd`)
- `nombre_de_archivo_ignora_fechas_del_cuerpo`: `frio = b"## 2026-10-13 — x\nrevisión el 2026-10-21, ver gold-2026-09-24.csv\n\n"`, slug `"slug"`, existentes 0 → `"slug-2026-10-13_2026-10-13.md"`. Falla si el rango sigue saliendo del cuerpo (hoy da `slug-2026-09-24_2026-10-21.md`).
- `nombre_de_archivo_toma_solo_la_primera_fecha_de_la_cabecera`: `frio = b"## 2026-10-13 — revisar el 2026-10-21\nx\n\n"` → `"slug-2026-10-13_2026-10-13.md"`. Falla si se usan todas las fechas de la línea de cabecera.
- `nombre_de_archivo_ignora_cabeceras_dentro_de_valla`: `frio = b"## 2026-10-13 — a\n```\n## 2026-01-01 — no soy cabecera\n```\n\n"` → `"slug-2026-10-13_2026-10-13.md"`. Falla si un `## ` dentro de valla aporta fecha.
- `nombre_de_archivo_cae_a_parte_si_solo_el_cuerpo_tiene_fechas`: `frio = b"## sin fecha\nvisto el 2026-10-21\n\n"`, existentes 2 → `"slug-parte-03.md"`. Falla si se reintroduce el fallback a fechas del cuerpo.
- `nombre_de_archivo_mixto_usa_solo_las_cabeceras_fechadas`: `frio = b"## 2026-06-26 — a\nx\n\n## sin fecha\ncita 2026-12-31\n\n## 2026-07-11 — b\ny\n\n"` → `"slug-2026-06-26_2026-07-11.md"`. Falla si una entrada sin fecha en cabecera aporta las fechas de su cuerpo o si el bloque mixto cae a `parte-NN`.
- Existentes sin tocar, siguen verdes: `nombre_de_archivo_usa_el_rango_de_fechas_iso` (cabecera con fecha a mitad: `## Actualizacion 2026-07-11 -- b`) y `nombre_de_archivo_cae_a_numero_de_parte_sin_fechas`.

**Verificación:** desde `engine/`: `cargo fmt --check` → sin salida; `cargo clippy --all-targets --locked -- -D warnings` → sin warnings; `cargo test --locked` → todos los tests en verde (suite entera, incluidos `engine/tests/`).

**Review Focus:**
- Cabecera con la fecha a mitad de línea (`## Delta 2026-08-02 —`): debe contar; cubierto por `nombre_de_archivo_usa_el_rango_de_fechas_iso`.
- Cabecera dentro de valla: cubierto por `nombre_de_archivo_ignora_cabeceras_dentro_de_valla`.
- Dos fechas en la misma cabecera: cubierto por `nombre_de_archivo_toma_solo_la_primera_fecha_de_la_cabecera`.
- Bloque frío mixto (unas cabeceras con fecha y otras sin ella): el rango sale del subconjunto fechado; cubierto por `nombre_de_archivo_mixto_usa_solo_las_cabeceras_fechadas`.

**Notas:** reutiliza `offsets_de_entradas` (o `lineas_con_offset` con el mismo criterio de valla) para localizar las líneas de cabecera; no dupliques el criterio de valla. Actualiza el doc-comment de `nombre_de_archivo` (`rotacion.rs:205-208`) para decir que las fechas salen solo de las cabeceras.
