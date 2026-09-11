# Ruta portable y 4ª columna en `search` — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** que toda ruta que el binario `exo` emite lleve una sola grafía (`/`) y
que `exo search` en modo humano dé la ruta del fichero, para que un agente no
necesite `jq` ni adivine la forma del envelope.

**Architecture:** hay una sola fuente de rutas relativas (`notas.ruta`) y dos
funciones que la construían distinto: `walker.rs` normaliza a `/`, `indexer.rs`
no. Se unifica en un helper (`walker::ruta_portable`), se migra la columna ya
escrita desde el camino de escritura, se arreglan las emisiones de ruta absoluta
que usaban `Path::join` (que en Windows reintroduce `\`), y se añade la 4ª
columna a la salida humana de `search`. El envelope `--json` no cambia.

**Tech Stack:** Rust 2024 (crate `exo`, `engine/`), `rusqlite`, `clap`,
`anyhow`, `tempfile`; tests de integración en `engine/tests/`; gate de shell en
`plugins/exo/scripts/`.

## Global Constraints

Valores exactos, copiados de `docs/superpowers/specs/2026-09-11-ruta-portable-y-columna-humana-design.md`.
Toda tarea los hereda.

- La normalización es **incondicional**, nunca bajo `#[cfg(windows)]`: si el
  indexer la gateara y el walker no, discreparían en Linux ante un fichero
  `a\b.md` y esa nota se vería «borrada» y reinsertada en cada corrida.
- **El envelope `--json` no cambia**: `schema_version` sigue en `2` y las claves
  de `.data` siguen siendo `elapsed_s`, `query`, `results`, `search_type`,
  `warnings` (omitida si vacía). Solo cambian VALORES de `path`.
- Marcador de ruta ausente, literal exacto: `(sin-ruta:rebuild)` — **un token,
  sin espacios ni tabs**.
- Aviso de ruta ausente, literal exacto:
  `aviso: N de M resultados sin ruta (índice inconsistente): exo rebuild`
  (con `N`/`M` sustituidos). Va a **stderr**, y **NO** entra en
  `resultado.avisos` / `warnings`.
- Toda **emisión** de ruta absoluta concatena con `/`. Si vas a `display()` una
  ruta que sale por stdout o por el envelope, ese `display()` es el bug. El
  `PathBuf` sigue usándose para E/S sin tocar.
- La migración corre **solo en el camino de escritura** (`indexa_incremental`).
  **Nunca en `abre_db`**: los comandos de solo lectura de este repo no escriben.
- `clippy` es gate duro (`-D warnings`): nada de imports, variables o helpers
  sin usar.
- Comando de verificación de la suite (el mismo que corre el CI):
  `cd engine && EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast`

---

### Task 1: `ruta_portable` y `ruta_relativa`, más la migración de `notas.ruta`

**Files:**
- Modify: `engine/src/walker.rs:118-131` (extraer el helper que ya vive inline)
- Modify: `engine/src/indexer.rs:461-472` (`ruta_relativa`)
- Modify: `engine/src/indexer.rs:158-164` (migración antes de leer `existentes`)
- Test: `engine/tests/indexer.rs`

**Interfaces:**
- Produces: `pub fn exo::walker::ruta_portable(s: &str) -> String` — sustituye
  `\` por `/` en toda la cadena.
- Produces: `pub fn exo::indexer::ruta_relativa(kb: &Path, ruta_abs: &Path) -> anyhow::Result<String>`
  — pasa de privada a pública para que el gate pueda aseverar exactamente la
  cadena que compara el incremental.
- Produces: `pub fn exo::indexer::migra_rutas_portables(conn: &rusqlite::Connection) -> anyhow::Result<usize>`
  — devuelve el número de filas migradas; 0 si no había ninguna con `\`.

- [ ] **Step 1: Escribir los tests que fallan**

Al final de `engine/tests/indexer.rs`:

```rust
#[test]
fn ruta_portable_sustituye_todas_las_barras_invertidas() {
    assert_eq!(exo::walker::ruta_portable("a\\b\\c.md"), "a/b/c.md");
    assert_eq!(exo::walker::ruta_portable("a/b.md"), "a/b.md");
    assert_eq!(exo::walker::ruta_portable(""), "");
}

#[test]
fn ruta_relativa_nunca_devuelve_barra_invertida() {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path();
    let abs = kb.join("log").join("alpha.md");
    let rel = exo::indexer::ruta_relativa(kb, &abs).unwrap();
    assert!(!rel.contains('\\'), "rel: {rel}");
    assert_eq!(rel, "log/alpha.md");
}

#[test]
fn la_migracion_normaliza_y_es_idempotente() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log\\alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();

    let migradas = exo::indexer::migra_rutas_portables(&conn).unwrap();
    assert_eq!(migradas, 1, "la primera corrida migra la fila con backslash");

    let ruta: String = conn
        .query_row("SELECT ruta FROM notas WHERE permalink = 'kb/log/alpha'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(ruta, "log/alpha.md");

    let otra_vez = exo::indexer::migra_rutas_portables(&conn).unwrap();
    assert_eq!(otra_vez, 0, "la segunda corrida no toca ninguna fila");
}

#[test]
fn tras_migrar_la_fila_casa_con_lo_que_calcula_el_incremental() {
    // El defecto que este test existe para impedir: `indexa_incremental`
    // compara por cadena exacta (`indexer.rs`, `existentes` vs `vistas`). Si
    // la migración y `ruta_relativa` no producen LA MISMA cadena, cada nota se
    // ve nueva y cada fila vieja se ve borrada → reindex completo con
    // re-embedding.
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path();
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(kb.join("log/alpha.md"), "---\ntier: stable\n---\n# alpha\n").unwrap();

    let db = kb.join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log\\alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    exo::indexer::migra_rutas_portables(&conn).unwrap();

    let en_db: String = conn
        .query_row("SELECT ruta FROM notas WHERE permalink = 'kb/log/alpha'", [], |r| r.get(0))
        .unwrap();
    let calculada = exo::indexer::ruta_relativa(kb, &kb.join("log").join("alpha.md")).unwrap();
    assert_eq!(en_db, calculada, "la fila migrada debe casar con el incremental");
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run:
```bash
cd engine && cargo test --test indexer ruta_portable_sustituye -- --nocapture
```
Expected: FAIL en compilación — `function `ruta_portable` not found in `exo::walker`` (y lo mismo para `ruta_relativa` / `migra_rutas_portables`, que hoy no son públicas ni existen).

- [ ] **Step 3: Implementación mínima**

En `engine/src/walker.rs`, justo antes de `fn recorre` (la función que hoy hace
el `.replace` inline):

```rust
/// Una sola grafía de ruta para todo lo que el binario EMITE: `\` → `/`.
///
/// Incondicional a propósito, no `#[cfg(windows)]`. En Unix `\` es un carácter
/// legal en un nombre de fichero, así que esto tiene ahí una arista — pero es
/// la MISMA arista en todos los puntos que la llaman. Gatearla en unos sí y en
/// otros no haría que `walk_kb` y el indexer discreparan ante un fichero
/// `a\b.md`, y esa nota se vería «borrada» y reinsertada en cada corrida.
pub fn ruta_portable(s: &str) -> String {
    s.replace('\\', "/")
}
```

Y en `recorre`, sustituir el `.replace('\\', "/")` inline por la llamada:

```rust
        let rel = ruta_portable(
            &ruta
                .strip_prefix(raiz)
                .with_context(|| format!("{} fuera de la raíz {}", ruta.display(), raiz.display()))?
                .to_string_lossy(),
        );
```

En `engine/src/indexer.rs`, `ruta_relativa` pasa a pública y normaliza:

```rust
/// Ruta de `ruta_abs` relativa a la raíz de la KB, **siempre con `/`**.
///
/// Pública porque es la cadena exacta que `indexa_incremental` compara contra
/// `notas.ruta`: el gate necesita poder aseverar esa igualdad sin reimplementarla.
pub fn ruta_relativa(kb: &Path, ruta_abs: &Path) -> Result<String> {
    Ok(crate::walker::ruta_portable(
        &ruta_abs
            .strip_prefix(kb)
            .with_context(|| {
                format!(
                    "{} no está bajo la raíz {}",
                    ruta_abs.display(),
                    kb.display()
                )
            })?
            .to_string_lossy(),
    ))
}
```

Y la migración, en el mismo fichero:

```rust
/// Pone `notas.ruta` en grafía portable. Idempotente: la segunda corrida
/// afecta 0 filas. Devuelve cuántas migró.
///
/// **Solo desde el camino de escritura.** Un índice escrito antes de que
/// `ruta_relativa` normalizara tiene filas con `\`, y esas filas no casan con
/// lo que calcula el incremental: sin esto, la primera corrida tras el cambio
/// ve TODAS las notas como nuevas y todas las filas como borradas, y vuelve a
/// embeber la KB entera.
///
/// `UNIQUE(ruta)` no puede colisionar: una fila tiene un solo separador, no dos
/// variantes de sí misma.
pub fn migra_rutas_portables(conn: &rusqlite::Connection) -> Result<usize> {
    let filas = conn
        .execute(
            r"UPDATE notas SET ruta = replace(ruta, '\', '/') WHERE ruta LIKE '%\%'",
            [],
        )
        .context("migrar notas.ruta a grafía portable")?;
    Ok(filas)
}
```

Y la llamada, en `indexa_incremental`, **antes** de leer `existentes`
(hoy `engine/src/indexer.rs:160`, justo tras los `INSERT INTO meta`):

```rust
    // Antes de comparar nada: las filas escritas por una versión anterior
    // llevan el separador nativo, y la comparación es por cadena exacta.
    migra_rutas_portables(&conn)?;

    let existentes: HashMap<String, f64> = {
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run:
```bash
cd engine && cargo test --test indexer ruta_portable_sustituye
cd engine && cargo test --test indexer ruta_relativa_nunca
cd engine && cargo test --test indexer la_migracion_normaliza
cd engine && cargo test --test indexer tras_migrar_la_fila_casa
cd engine && cargo clippy --all-targets -- -D warnings
```
Expected: los cuatro PASS; clippy sin salida.

- [ ] **Step 5: Commit**

```bash
git add engine/src/walker.rs engine/src/indexer.rs engine/tests/indexer.rs
git commit -m "fix(indexer): ruta_relativa normaliza el separador, como ya hacia el walker"
```

---

### Task 2: las rutas absolutas se emiten con `/`, no con `Path::join`

**Files:**
- Modify: `engine/src/recall.rs:549-553` (`resuelve_rutas_absolutas`)
- Modify: `engine/src/escritor.rs:281` y `:300` (`write new`), `:340-347` (`write append`)
- Test: `engine/tests/recall.rs`, `engine/tests/escritor.rs`

**Interfaces:**
- Consumes: `exo::walker::ruta_portable(s: &str) -> String` (Task 1).
- Produces: ningún símbolo nuevo. `RecallBruto.notas[].ruta` y
  `Escritura.ruta_abs` (`absolute_path` en el envelope) pasan a no contener `\`.

- [ ] **Step 1: Escribir los tests que fallan**

Al final de `engine/tests/recall.rs`:

```rust
#[test]
fn la_ruta_absoluta_de_recall_no_lleva_barra_invertida() {
    // El bug real: `kb.join(&nota.ruta)` con `kb` en "/" (viene de config) y
    // `ruta` relativa produce, en Windows, un separador `\` INTERIOR. Por eso
    // se asevera la cadena entera, no el prefijo: un test que mirara solo el
    // principio bendice exactamente el defecto que existe para cazar.
    let mut bruto = exo::recall::RecallBruto {
        modo: "consulta".into(),
        query: Some("alpha".into()),
        notas: vec![exo::recall::NotaRecall {
            permalink: "kb/log/alpha".into(),
            ruta: "log/alpha.md".into(),
            titulo: "alpha".into(),
            tier: Some("stable".into()),
            score: Some(1.0),
            snippet: None,
        }],
    };
    exo::recall::resuelve_rutas_absolutas(&mut bruto, std::path::Path::new("C:/kb"));
    assert!(
        !bruto.notas[0].ruta.contains('\\'),
        "ruta: {}",
        bruto.notas[0].ruta
    );
    assert_eq!(bruto.notas[0].ruta, "C:/kb/log/alpha.md");
}
```

Al final de `engine/tests/escritor.rs`:

```rust
#[test]
fn el_absolute_path_de_write_no_lleva_barra_invertida() {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path();
    std::fs::create_dir_all(kb.join("log")).unwrap();

    let e = exo::escritor::escribe_nueva(
        kb,
        "kb",
        "log",
        "alpha",
        "cuerpo\n",
        Some("stable"),
        &[],
        false,
    )
    .unwrap();

    assert!(!e.ruta_abs.contains('\\'), "absolute_path: {}", e.ruta_abs);
    assert!(!e.ruta_rel.contains('\\'), "relative_path: {}", e.ruta_rel);

    // `write append` llega a `ruta_abs` por otro camino (la ruta relativa sale
    // del índice, no se compone), así que se ejerce aparte.
    let a = exo::escritor::escribe_append(kb, "log/alpha.md", "\nmás cuerpo\n", false).unwrap();
    assert!(!a.ruta_abs.contains('\\'), "absolute_path: {}", a.ruta_abs);
    assert!(!a.ruta_rel.contains('\\'), "relative_path: {}", a.ruta_rel);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run:
```bash
cd engine && cargo test --test recall la_ruta_absoluta_de_recall
cd engine && cargo test --test escritor el_absolute_path_de_write
```
Expected en Windows: FAIL con `ruta: C:/kb\log/alpha.md` (el `\` lo mete
`Path::join`). En Linux el test pasa ya: el defecto es específico de Windows,
y el CI corre los tres SO — el verde de Linux no es evidencia.

- [ ] **Step 3: Implementación mínima**

`engine/src/recall.rs`, `resuelve_rutas_absolutas`:

```rust
pub fn resuelve_rutas_absolutas(bruto: &mut RecallBruto, kb: &Path) {
    // `kb.join(...)` NO: en Windows empuja con `\` cuando lo añadido no empieza
    // por separador, y eso es justo el separador interior que rompía el bloque
    // que `recall-inject.sh` inyecta en cada prompt. La ruta que se EMITE se
    // concatena a mano; el `PathBuf` sigue siendo para E/S.
    let raiz = crate::walker::ruta_portable(&kb.display().to_string());
    let raiz = raiz.trim_end_matches('/');
    for nota in &mut bruto.notas {
        nota.ruta = format!("{raiz}/{}", crate::walker::ruta_portable(&nota.ruta));
    }
}
```

`engine/src/escritor.rs`: donde hoy se construye el campo emitido
`ruta_abs: ruta_abs.display().to_string()` (dos sitios: `write new` ~`:300` y
`write append` ~`:347`), sustituir por:

```rust
        ruta_abs: crate::walker::ruta_portable(&ruta_abs.display().to_string()),
```

y en `write append`, donde `ruta_rel` sale del índice:

```rust
        ruta_rel: crate::walker::ruta_portable(ruta_rel),
```

El `PathBuf` `ruta_abs` que se pasa a `escribe_atomico` / `anexa` **no se
toca**: esto es sobre lo que se emite, no sobre cómo se abre un fichero.

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run:
```bash
cd engine && cargo test --test recall la_ruta_absoluta_de_recall
cd engine && cargo test --test escritor el_absolute_path_de_write
cd engine && cargo clippy --all-targets -- -D warnings
```
Expected: PASS los dos; clippy sin salida.

- [ ] **Step 5: Commit**

```bash
git add engine/src/recall.rs engine/src/escritor.rs engine/tests/recall.rs engine/tests/escritor.rs
git commit -m "fix(recall,escritor): emitir rutas absolutas con / en vez de Path::join"
```

---

### Task 3: `exo doctor` delata un índice con rutas nativas

**Files:**
- Modify: `engine/src/doctor.rs:157-173` (`analiza`, añadir el check a la lista)
- Modify: `engine/src/doctor.rs` (nueva `fn check_rutas_portables`)
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Consumes: `exo::doctor::Check::nuevo(id: &'static str, estado: Estado, artefacto: impl Into<String>, detalle: impl Into<String>) -> Check`
  y los helpers ya existentes `db_efectiva(entorno, cfg) -> Option<PathBuf>`.
- Produces: un `Check` con `id: "index_paths_portable"`. `Estado::Warn` cuando
  hay filas con `\`, `Estado::Ok` cuando no, `Estado::Na` si no hay índice.

**Por qué existe:** la migración de Task 1 corre solo al indexar, así que entre
el upgrade del binario y el siguiente `exo index` hay una ventana en la que el
índice sigue sirviendo `\`. Una ventana que no se puede observar es «ausencia ≠
evidencia».

- [ ] **Step 1: Escribir el test que falla**

Al final de `engine/tests/doctor.rs`:

```rust
#[test]
fn el_check_de_rutas_portables_avisa_cuando_el_indice_trae_backslash() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log\\alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    drop(conn);

    let check = exo::doctor::check_rutas_portables_de(&db);
    assert_eq!(check.id, "index_paths_portable");
    assert_eq!(check.estado, exo::doctor::Estado::Warn);
    assert!(
        check.detalle.contains("exo index"),
        "el detalle debe nombrar el remedio: {}",
        check.detalle
    );
}

#[test]
fn el_check_de_rutas_portables_pasa_con_el_indice_limpio() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    drop(conn);

    let check = exo::doctor::check_rutas_portables_de(&db);
    assert_eq!(check.estado, exo::doctor::Estado::Ok);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run:
```bash
cd engine && cargo test --test doctor el_check_de_rutas_portables
```
Expected: FAIL en compilación — `function `check_rutas_portables_de` not found in `exo::doctor``.

- [ ] **Step 3: Implementación mínima**

En `engine/src/doctor.rs`:

```rust
/// Núcleo testeable del check: toma la DB ya resuelta, para que un test no
/// tenga que montar un `Entorno` entero.
pub fn check_rutas_portables_de(db: &Path) -> Check {
    if !db.is_file() {
        return Check::nuevo(
            "index_paths_portable",
            Estado::Na,
            db.display().to_string(),
            "no hay índice todavía",
        );
    }
    let nativas: i64 = match crate::abre_db(db).and_then(|c| {
        Ok(c.query_row(
            r"SELECT count(*) FROM notas WHERE ruta LIKE '%\%'",
            [],
            |r| r.get(0),
        )?)
    }) {
        Ok(n) => n,
        Err(e) => {
            return Check::nuevo(
                "index_paths_portable",
                Estado::Fail,
                db.display().to_string(),
                format!("{e:#}"),
            );
        }
    };
    if nativas > 0 {
        Check::nuevo(
            "index_paths_portable",
            Estado::Warn,
            db.display().to_string(),
            format!(
                "{nativas} ruta(s) con separador nativo: este índice se escribió \
                 con una versión anterior y sirve rutas que no se pueden pegar \
                 en un comando — corre `exo index`"
            ),
        )
    } else {
        Check::nuevo(
            "index_paths_portable",
            Estado::Ok,
            db.display().to_string(),
            "todas las rutas del índice usan `/`",
        )
    }
}

fn check_rutas_portables(entorno: &Entorno, cfg: Option<&crate::config::Config>) -> Check {
    match db_efectiva(entorno, cfg) {
        Some(db) => check_rutas_portables_de(&db),
        None => Check::nuevo(
            "index_paths_portable",
            Estado::Na,
            "(sin config)",
            "no hay config legible, así que no se sabe qué DB mirar",
        ),
    }
}
```

Y en `analiza`, añadir la línea tras `check_indice(entorno, cfg.as_ref()),`:

```rust
        check_rutas_portables(entorno, cfg.as_ref()),
```

`Estado` ya deriva `PartialEq` (`engine/src/doctor.rs:15`), así que el
`assert_eq!` del test compila sin tocar el enum.

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run:
```bash
cd engine && cargo test --test doctor el_check_de_rutas_portables
cd engine && cargo clippy --all-targets -- -D warnings
```
Expected: los dos PASS; clippy sin salida.

- [ ] **Step 5: Commit**

```bash
git add engine/src/doctor.rs engine/tests/doctor.rs
git commit -m "feat(doctor): check que delata un indice con rutas de separador nativo"
```

---

### Task 4: la 4ª columna de `exo search` en modo humano

**Files:**
- Modify: `engine/src/main.rs:204-240` (`ArgsSearch`: flag `--kb`)
- Modify: `engine/src/main.rs:890-920` (`busca_cmd`)
- Create: `engine/tests/buscador_cli.rs`

**Interfaces:**
- Consumes: `exo::walker::ruta_portable(s: &str) -> String` (Task 1);
  `resuelve_kb(flag: Option<PathBuf>) -> Result<PathBuf>` (ya existe en
  `engine/src/main.rs:457`).
- Produces: salida humana de `exo search` con **cuatro** columnas separadas por
  tab: `permalink \t type \t score(4 decimales) \t ruta absoluta`.

**Helper local:** esta suite monta su propia KB+índice. **No** se mueve
`kb_con_indice()` a `tests/common/mod.rs`: las tres copias existentes montan KBs
distintas (`objetivos.rs` con beta/gamma/pdf, las otras dos con solo alpha), así
que unificarlas es trabajo de fixtures, no un `git mv` — está en el backlog de
la spec.

- [ ] **Step 1: Escribir los tests que fallan**

Crear `engine/tests/buscador_cli.rs`:

```rust
//! `exo search` contra el binario real, modo humano y `--json`.
//!
//! No usa `tests/common/mod.rs`: estos tests pasan `--db` y `--kb` explícitos,
//! así que `resuelve_db`/`resuelve_kb` cortan en el flag y nunca cargan config.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_exo")
}

/// KB con una nota en un subdirectorio (el subdirectorio importa: el defecto
/// original era un separador INTERIOR) y su índice poblado a mano.
fn kb_con_indice() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let kb = dir.path().to_path_buf();
    std::fs::create_dir_all(kb.join("log")).unwrap();
    std::fs::write(
        kb.join("log/alpha.md"),
        "---\ntier: stable\n---\n# alpha\ncuerpo de alpha\n",
    )
    .unwrap();

    let db = dir.path().join("index.db");
    let conn = exo::abre_db(&db).unwrap();
    exo::schema::crea_schema(&conn).unwrap();
    conn.execute(
        "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
         VALUES ('kb/log/alpha', 'log/alpha.md', 'alpha', 'note', 0.0, NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notas_fts (titulo, cuerpo, permalink)
         VALUES ('alpha', 'cuerpo de alpha', 'kb/log/alpha')",
        [],
    )
    .unwrap();
    drop(conn);
    (dir, db)
}

fn busca(kb: &std::path::Path, db: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(["search"])
        .arg("--db")
        .arg(db)
        .arg("--kb")
        .arg(kb)
        .args(args)
        .arg("alpha")
        .output()
        .unwrap()
}

#[test]
fn la_salida_humana_lleva_cuatro_columnas_y_la_cuarta_es_la_ruta() {
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &[]);
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    let cols: Vec<&str> = linea.split('\t').collect();
    assert_eq!(cols.len(), 4, "linea: {linea:?}");
    assert_eq!(cols[0], "kb/log/alpha");
    assert!(
        std::path::Path::new(cols[3]).is_file(),
        "la 4a columna debe existir en disco: {}",
        cols[3]
    );
}

#[test]
fn la_ruta_humana_no_lleva_barra_invertida() {
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &[]);
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    let ruta = linea.split('\t').nth(3).unwrap();
    // La cadena ENTERA, no el prefijo: el defecto era un separador interior.
    assert!(!ruta.contains('\\'), "ruta: {ruta}");
}

#[test]
fn sin_ruta_la_columna_lo_dice_y_stderr_avisa() {
    // Índice inconsistente a propósito: el permalink está en notas_fts pero no
    // en notas, que es el único caso en que `path` es None.
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &[]);
    assert!(salida.status.success());
    let texto = String::from_utf8_lossy(&salida.stdout);
    let linea = texto.lines().next().expect("al menos un resultado");
    assert_eq!(linea.split('\t').nth(3).unwrap(), "(sin-ruta:rebuild)");

    // Media señal es la que no grita: marcador Y aviso, en el mismo test.
    let err = String::from_utf8_lossy(&salida.stderr);
    assert!(
        err.contains("sin ruta (índice inconsistente)"),
        "stderr: {err}"
    );
    assert!(err.contains("exo rebuild"), "stderr: {err}");
}

#[test]
fn el_marcador_no_lleva_whitespace() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &[]);
    let texto = String::from_utf8_lossy(&salida.stdout);
    let marcador = texto.lines().next().unwrap().split('\t').nth(3).unwrap();
    // Sin esto, `awk '{print $4}'` (sin -F, el habito mas comun) imprime
    // solo el primer trozo del marcador.
    assert!(
        !marcador.chars().any(char::is_whitespace),
        "marcador: {marcador:?}"
    );
}

#[test]
fn el_aviso_no_entra_en_el_envelope() {
    let (dir, db) = kb_con_indice();
    let conn = exo::abre_db(&db).unwrap();
    conn.execute("DELETE FROM notas WHERE permalink = 'kb/log/alpha'", [])
        .unwrap();
    drop(conn);

    let salida = busca(dir.path(), &db, &["--json"]);
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    // El aviso de ruta ausente va SOLO a stderr: `path: null` ya es inferible
    // desde `results`, y `warnings` es para lo que NO se puede inferir.
    let avisos = v["data"]["warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        !avisos.iter().any(|a| a.as_str().unwrap_or("").contains("sin ruta")),
        "warnings: {avisos:?}"
    );
}

#[test]
fn el_json_no_cambia() {
    // Ejerce el BINARIO, no el struct: `contrato_envelope.rs` ya serializa el
    // struct a mano y por eso no habria atrapado nada de este incidente.
    let (dir, db) = kb_con_indice();
    let salida = busca(dir.path(), &db, &["--json"]);
    assert!(salida.status.success());
    let v: serde_json::Value = serde_json::from_slice(&salida.stdout).unwrap();
    assert_eq!(v["schema_version"], 2);
    assert_eq!(v["command"], "search");
    for clave in ["elapsed_s", "query", "results", "search_type"] {
        assert!(v["data"].get(clave).is_some(), "falta .data.{clave}: {v}");
    }
    let r0 = &v["data"]["results"][0];
    assert_eq!(r0["permalink"], "kb/log/alpha");
    assert_eq!(r0["type"], "entity");
    // El JSON sigue dando la ruta RELATIVA (la absoluta es solo de la humana).
    assert_eq!(r0["path"], "log/alpha.md");
}

#[test]
fn sin_kb_resoluble_el_modo_humano_falla_con_remedio() {
    let (dir, db) = kb_con_indice();
    let salida = Command::new(bin())
        .args(["search"])
        .arg("--db")
        .arg(&db)
        .arg("alpha")
        .env("EXO_CONFIG", dir.path().join("no-existe.toml"))
        .env_remove("EXO_KB")
        .output()
        .unwrap();
    assert!(!salida.status.success(), "debe fallar, no dar ruta relativa");
    let err = String::from_utf8_lossy(&salida.stderr);
    assert!(err.contains("--kb"), "el error debe nombrar el remedio: {err}");
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run:
```bash
cd engine && cargo test --test buscador_cli
```
Expected: FAIL. `la_salida_humana_lleva_cuatro_columnas…` con
`assertion `left == right` failed: left: 3, right: 4`, y
`sin_kb_resoluble…` porque hoy `search` no acepta `--kb` (error de clap:
`unexpected argument '--kb'`).

- [ ] **Step 3: Implementación mínima**

En `engine/src/main.rs`, dentro de `struct ArgsSearch`, junto a `db`:

```rust
    /// Raíz de la KB. Solo la usa la salida humana, que emite rutas absolutas
    /// (el `--json` sigue dando la relativa). Precedencia: flag > $EXO_KB >
    /// config, igual que en `recall` y `targets`.
    #[arg(long)]
    kb: Option<PathBuf>,
```

Y `busca_cmd`, sustituyendo el bloque `if args.json { … } else { … }`:

```rust
    if args.json {
        envelope::emite("search", serde_json::to_value(&resultado)?);
    } else {
        // La humana da la ruta ABSOLUTA: el cwd del agente es el repo en el que
        // trabaja, no la KB, así que una relativa no se le puede pasar a `Edit`
        // sin resolver antes la raíz — y eso devolvería el jq que esta columna
        // existe para quitar. Mismo criterio que `exo write`.
        let kb = resuelve_kb(args.kb).context(
            "la salida humana de `exo search` emite rutas absolutas y necesita la \
             raíz de la KB: pasa --kb, o corre `exo init`",
        )?;
        let raiz = exo::walker::ruta_portable(&kb.display().to_string());
        let raiz = raiz.trim_end_matches('/');

        let mut sin_ruta = 0usize;
        for r in &resultado.results {
            let ruta = match &r.ruta {
                Some(rel) => format!("{raiz}/{}", exo::walker::ruta_portable(rel)),
                None => {
                    sin_ruta += 1;
                    // Un token, sin espacios: `awk '{print $4}'` sin -F es el
                    // hábito más común y partiría un marcador con espacios.
                    "(sin-ruta:rebuild)".to_string()
                }
            };
            println!("{}\t{}\t{:.4}\t{}", r.permalink, r.tipo, r.score, ruta);
        }
        if sin_ruta > 0 {
            // A stderr y FUERA de `warnings`: `path: null` ya es inferible
            // desde `results`, y el envelope no cambia por esto.
            eprintln!(
                "aviso: {sin_ruta} de {} resultados sin ruta (índice inconsistente): exo rebuild",
                resultado.results.len()
            );
        }
    }
```

Si `anyhow::Context` no está ya importado en `main.rs`, añádelo al `use`.

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run:
```bash
cd engine && cargo test --test buscador_cli
cd engine && cargo clippy --all-targets -- -D warnings
```
Expected: 7 passed; clippy sin salida.

- [ ] **Step 5: Verificación real (no solo tests)**

Run, contra la KB de verdad:
```bash
exo index
exo search --type hybrid --limit 3 "fallo silencioso"
exo doctor --json | jq -c '.data.checks[] | select(.id=="index_paths_portable")'
exo recall --query "fallo silencioso" --limit 2
```
Expected: cuatro columnas, la 4ª una ruta absoluta **sin ningún `\`**; el check
de doctor en `ok`; `recall` sin `\` en sus rutas.

- [ ] **Step 6: Commit**

```bash
git add engine/src/main.rs engine/tests/buscador_cli.rs
git commit -m "feat(search): la salida humana da la ruta absoluta en una 4a columna"
```

---

### Task 5: la prosa deja de llevar jq y deja de mentir

**Files:**
- Modify: `plugins/exo/agents/executor.md:13`
- Modify: `plugins/exo/skills/document/SKILL.md:20-24`
- Modify: `docs/arquitectura.md` (§3.5, ~L192, y la fila de `exo search` de la
  tabla de comandos, ~L289)

**Interfaces:**
- Consumes: la salida de Task 4 (cuatro columnas) y el envelope intacto.
- Produces: prosa. Ningún símbolo.

**El defecto que corrige:** las dos primeras dicen que `search --json` devuelve
`ruta`. El JSON emite `path` (`buscador.rs:30`, `#[serde(rename = "path")]`), y
`.data.results[] | .ruta` devuelve `null` por fila **sin error de jq**.

- [ ] **Step 1: `plugins/exo/agents/executor.md:13`**

Sustituir la línea entera por:

```markdown
- **Usa la memoria si aplica (degradable).** Si tu brief referencia notas de memoria (permalinks / memory packet), léelas antes de empezar con `exo search --type hybrid --limit 5 "<query>"` — cuatro columnas separadas por tab: `permalink`, `type`, `score`, **ruta absoluta** (pégala tal cual en `Read`/`Edit`; el permalink NO es invertible). `exo targets <topic>` da headings sin body. Si el engine no responde, sigue sin bloquearte.
```

- [ ] **Step 2: `plugins/exo/skills/document/SKILL.md`, párrafo de las líneas 20-24**

Sustituir desde `Probe del engine antes de rutear:` hasta
`mientras exista.` por:

```markdown
Probe del engine antes de rutear: `exo search --type hybrid --limit 5 "<topic>"`
da por candidata `permalink`, `type`, `score` y **la ruta absoluta**, en cuatro
columnas separadas por tab y sin jq de por medio. Elige "nota X, sección Y" y lee
SOLO la ganadora antes de escribir. La ruta es imprescindible: el permalink NO es
invertible (el slug come acentos, espacios y em-dashes), así que sin ella no
puedes localizar el fichero. Alternativa con snippet y misma búsqueda híbrida:
`exo recall --query "<topic>" --limit 5`; `search` es la que además da el score.
Si necesitas el envelope para un script: `exo search --type hybrid --json
"<topic>" | jq -r '.data.results[] | "\(.score)  \(.permalink)  \(.path)"'` — los
resultados cuelgan de `.data.results[]` (`.data` es un objeto, no un array) y el
campo es `path`, nunca `ruta`. `exo targets` sigue sirviendo para ver headings sin
body mientras exista.
```

- [ ] **Step 3: `docs/arquitectura.md`**

En §3.5, tras el párrafo que empieza `exo search` tiene tres modos, añadir:

```markdown
Dos salidas, dos formas. La humana son cuatro columnas separadas por tab —
`permalink`, `type`, `score` (4 decimales), **ruta absoluta** — y necesita la
raíz de la KB (`--kb`, `$EXO_KB` o config). El `--json` emite el envelope §4:
los resultados cuelgan de `.data.results[]` (`.data` es un objeto, no un array)
y cada uno trae `permalink`, `type`, `score` y `path`, esta **relativa** a la
raíz de la KB. Cuando el índice no tiene la ruta de un permalink, la humana
imprime `(sin-ruta:rebuild)` y avisa por stderr; el envelope pone `path: null`.
```

Y en la fila de `exo search` de la tabla de comandos, añadir `--kb` a la lista
de flags.

- [ ] **Step 4: Verificar que la prosa no miente**

Run:
```bash
exo search --type hybrid --limit 2 "doctrina"
exo search --type hybrid --json --limit 2 "doctrina" | jq -r '.data.results[] | "\(.score)  \(.permalink)  \(.path)"'
grep -rn "ruta\`" plugins/exo/agents/executor.md plugins/exo/skills/document/SKILL.md
```
Expected: los dos comandos dan salida no vacía y sin `null`; el `grep` no
encuentra ninguna mención a un campo `ruta` del JSON.

- [ ] **Step 5: Commit**

```bash
git add plugins/exo/agents/executor.md plugins/exo/skills/document/SKILL.md docs/arquitectura.md
git commit -m "docs: la prosa de search deja de llevar jq y de llamar 'ruta' a 'path'"
```

---

### Task 6: el gate de contrato cubre `search` y declara que es local

**Files:**
- Modify: `plugins/exo/scripts/test-contrato-engine.sh` (cabecera y bloque de
  predicados)

**Interfaces:**
- Consumes: el binario del repo (`$EXO_BIN`, por defecto
  `engine/target/release/exo.exe`) y `$EXO_INDEX` / `$EXO_KB` resueltos por
  `exo config --json` — todo ya montado en el script.
- Produces: nada importable. Un gate más en la misma suite de `pass`/`fail`.

- [ ] **Step 1: Declarar en la cabecera que NO corre en CI**

Sustituir el párrafo que hoy empieza `# G5 (cuando haya CI):` por:

```bash
# GATE LOCAL, NO DE CI (verificado 2026-09-11): `.github/workflows/ci.yml:99`
# solo lanza `engine/scripts/test-hermetico.sh`, que verifica otra cosa (que la
# suite corra sin ~/.exo/config.toml). Este script depende del estado de ESTA
# máquina —índice y KB reales— y no hay fixture reproducible todavía. Corrérlo
# es responsabilidad del que toca el contrato del engine; el CI no lo hará por ti.
```

- [ ] **Step 2: Añadir los predicados de `search`**

Antes de la línea `printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"`, insertar:

```bash
# --- Los predicados de los que vive la prosa de `search --json` --------------
# La receta que documentan document/SKILL.md y arquitectura.md es
# `.data.results[] | .permalink, .path`. Si el envelope deriva, esa prosa pasa a
# mentir en silencio (un `.ruta` devolvía `null` sin error de jq durante meses).
SALIDA_S="$(timeout "${EXO_CONTRATO_TIMEOUT:-15}" "$EXO_BIN" search --type hybrid \
              --json --limit 3 --db "$EXO_INDEX" --kb "$EXO_KB" "doctrina" 2>/dev/null)"
RC_S=$?
if [ "$RC_S" -ne 0 ] || [ -z "$SALIDA_S" ]; then
  abstenerse "search --json salió con rc=$RC_S o sin salida"
fi

if printf '%s' "$SALIDA_S" | jq -e '.data.results | type == "array"' >/dev/null 2>&1; then
  pass "contrato search: .data.results es un array"
else fail "contrato search: .data.results es un array" "$(printf '%s' "$SALIDA_S" | jq -c '.data | keys' 2>/dev/null)"; fi

if printf '%s' "$SALIDA_S" | jq -e '
      .data.results[0] as $r
      | ($r.permalink|type) == "string" and ($r.permalink|length) > 0
      and ($r.path|type) == "string" and ($r.path|length) > 0
    ' >/dev/null 2>&1; then
  pass "contrato search: el primer resultado trae permalink y path no vacíos"
else
  fail "contrato search: el primer resultado trae permalink y path no vacíos" \
    "$(printf '%s' "$SALIDA_S" | jq -c '.data.results[0]' 2>/dev/null)"
fi

# `ruta` es el nombre del campo RUST; el envelope emite `path`. Si algún día
# reaparece, la prosa que lo citaba vuelve a ser correcta y este gate debe caer.
if printf '%s' "$SALIDA_S" | jq -e '.data.results[0] | has("ruta") | not' >/dev/null 2>&1; then
  pass "contrato search: el envelope NO trae 'ruta' (es 'path')"
else fail "contrato search: el envelope NO trae 'ruta'" "$(printf '%s' "$SALIDA_S" | jq -c '.data.results[0]' 2>/dev/null)"; fi

# La ruta del envelope no lleva separador nativo.
if printf '%s' "$SALIDA_S" | jq -e '[.data.results[].path | select(. != null) | contains("\\")] | any | not' >/dev/null 2>&1; then
  pass "contrato search: ninguna path del envelope lleva barra invertida"
else fail "contrato search: ninguna path lleva barra invertida" "$(printf '%s' "$SALIDA_S" | jq -c '[.data.results[].path]' 2>/dev/null)"; fi
```

- [ ] **Step 3: Correr el gate y verificar que pasa**

Run:
```bash
cd engine && cargo build --release
bash plugins/exo/scripts/test-contrato-engine.sh
```
Expected: todas las líneas `[PASS]`, `0 failed`, exit 0. Si sale
`[ABSTENCION]` con exit 2, el índice o la KB de esta máquina no están
disponibles — eso **no** es un verde, es que no se pudo verificar nada.

- [ ] **Step 4: Commit**

```bash
git add plugins/exo/scripts/test-contrato-engine.sh
git commit -m "test(contrato): cubrir search --json y declarar el gate como local"
```

---

## Cierre del plan

- [ ] **Suite completa, como la corre el CI**

```bash
cd engine && EXO_CONFIG=/tmp/no-existe.toml cargo test --release --no-fail-fast
cd engine && cargo clippy --all-targets -- -D warnings
```
Expected: 0 failed; clippy sin salida.

- [ ] **Verificación real de extremo a extremo**

```bash
exo index
exo doctor --json | jq -c '.data.checks[] | select(.id=="index_paths_portable")'
exo search --type hybrid --limit 3 "fallo silencioso"
exo recall --query "fallo silencioso" --limit 2
```
Expected: el check en `ok`; `search` con cuatro columnas y ruta absoluta sin
`\`; `recall` sin `\`. Y en la sesión siguiente, el bloque que inyecta
`recall-inject.sh` en cada prompt debe traer rutas con `/` y la raíz común bien
calculada (hoy dice `C:/proyectos/homework` en vez de `…/wisdom-paul`).
