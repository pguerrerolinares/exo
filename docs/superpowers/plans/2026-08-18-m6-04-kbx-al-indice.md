# M6-04 — kbx al índice del engine: plan de tareas

> **For agentic workers:** REQUIRED SUB-SKILL: usa `process:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Un ejecutor fresco por tarea, sin contexto previo: todo lo que
> necesitas está en tu tarea.

**Goal:** que `kbx` lea el índice de exo en vez del de basic-memory, para que
M5b (desinstalar basic-memory) pueda ejecutarse sin que seis comandos mueran.

**Architecture:** dos binarios independientes que comparten un fichero SQLite.
`exo` (Rust, `engine/`) escribe el índice `~/.exo/index.db`; `kbx` (Go, repo
aparte) lo lee en `mode=ro`. El acoplamiento entre ambos es el schema, y el
único puente que lo vigila es el canary de `kbx doctor`. Este plan porta las
seis queries de kbx del schema de basic-memory (`entity`/`relation`/
`search_index`/`project`) al de exo (`notas`/`aristas`/`notas_fts`/`meta`),
añade a exo lo mínimo que falta (una tabla `meta` y WAL), y hace el cutover con
un cambio de default y rollback por binario.

**Tech Stack:** Rust (edition 2024, `rusqlite` bundled, `sqlite-vec` `=0.1.9`)
· Go 1.26.4 (`mattn/go-sqlite3` con build tag `sqlite_fts5`) · SQLite/FTS5 ·
markdown.

**Spec:** `docs/superpowers/specs/2026-08-18-m6-04-kbx-al-indice-design.md`
(commit `08e95d6`). **Verdict del consultor:**
`docs/superpowers/consultas/2026-08-18-m6-04-kbx/consultor-m6-04.md`.

## Global Constraints

Toda tarea los hereda. Valores verbatim de la spec y del régimen §0.

- **Veto AGPL**: jamás código ni vendorizado de basic-memory. Forma de schema sí.
- **Cambios de schema aditivos**: `CREATE TABLE IF NOT EXISTS`. **Ninguna tarea
  fuerza un rebuild del índice** (razón: `engine/src/nota.rs:14`).
- **Permalinks del frontmatter se honran y jamás se regeneran.**
- **Sin métricas nuevas, ventanas ni gates numéricos** (régimen §0). Los gates
  de este plan son de paridad y de conjunto, no de puntuación.
- **Fallo silencioso es el enemigo**: ningún check puede pasar por no-op. Si un
  check no puede fallar, no es un check.
- **kbx abre la DB siempre `mode=ro`.** No hay write path desde kbx, nunca.
- **Comandos de Go SIEMPRE por el Makefile** (`make build|vet|test|check|
  install`): `mattn/go-sqlite3` esconde FTS5 tras el build tag `sqlite_fts5` y
  un `go test ./...` pelado falla en runtime con `no such module: fts5`.
- **`sqlite3` CLI no está instalado** en esta máquina. Para inspeccionar DBs a
  mano: `python3 -c` con el módulo `sqlite3`, abriendo `file:...?mode=ro` (uri=True).
- Commits en castellano con prefijo convencional. Git desde el working dir o
  con `git -C`, **nunca `cd … && git`**.

## Resoluciones de pre-flight

1. **Dos repos, una rama en cada uno**: `m6-04` en
   `/home/paul/Documentos/proyectos/exo` y `m6-04` en
   `/home/paul/Documentos/proyectos/kbx`. Ninguna tarea commitea en `main`/`master`.
2. **Ninguna tarea toca el entorno vivo.** No se instala en `~/.local/bin/`, no
   se corre contra `~/.exo/index.db` en modo escritura, no se cambian call sites
   instalados. Todo eso está en §"Acciones de Paul" al final, como en C9.
   Las tareas verifican contra DBs temporales y contra copias RO.
3. **Task 1 va primero y no toca código.** El pre-registro del gate de `targets`
   caduca en cuanto alguien vea el output del binario nuevo.
4. **El fixture viejo de kbx no se borra hasta Task 9.** Cada tarea deja
   `make check` verde; el fixture nuevo convive con el viejo mientras dura el port.

---

### Task 1: Pre-registro del gate de `targets`

Escribe el criterio de aceptación de `targets` **antes de que exista nada que
mirar**. Una vez visto el output del binario nuevo ya no hay pre-registro
posible. No hay código en esta tarea.

**Files:**
- Create: `/home/paul/Documentos/proyectos/exo/docs/superpowers/plans/2026-08-18-m6-04-preregistro-targets.md`

**Interfaces:**
- Consumes: nada.
- Produces: el fichero de pre-registro que consumen Task 9 (gate de paridad) y
  Task 10 (delta de T3). Ninguna firma de código.

- [ ] **Step 1: Crear la rama en el repo exo**

```bash
git -C /home/paul/Documentos/proyectos/exo checkout -b m6-04
git -C /home/paul/Documentos/proyectos/exo status -sb
```

Expected: `## m6-04`, árbol limpio.

- [ ] **Step 2: Escribir el pre-registro**

Crea el fichero con exactamente este contenido:

```markdown
# Pre-registro del gate de `targets` (M6-04)

> Escrito ANTES de compilar el binario nuevo y ANTES de ver ningún output suyo.
> Spec §5.3. Si esta nota se edita después de la primera corrida, el gate deja
> de ser un pre-registro y hay que declararlo aquí.

## Por qué no hay paridad de ranking

`targets` no puede dar el mismo orden en los dos índices (spec §5.2): las
columnas FTS difieren (`title, content_stems` con stemming del pipeline Python
de basic-memory, `content_snippet`, `permalink` indexado y `prefix='1,2,3,4'`
contra `titulo, cuerpo` crudos en exo) y la multiplicidad de filas también (160
filas `type='entity'` para 143 entities en basic-memory; 1:1 en exo). El
tokenizer NO difiere: ambos usan `unicode61 tokenchars 0x2F`.

## Los topics

Cinco, elegidos para cubrir un término técnico, un nombre propio, una palabra
de dominio, un acrónimo y una consulta multipalabra:

1. `indexer`
2. `reflex`
3. `memoria`
4. `kbx`
5. `recall en el punto de uso`

## Criterio (por topic)

Se compara `kbx targets <topic> --json --limit 5` corrido con el **binario
viejo sobre `~/.basic-memory/memory.db`** contra el **binario nuevo sobre una
copia RO de `~/.exo/index.db`**, con el filtro `tipo='note'` todavía puesto.

- **PASA** si ≥3 de los 5 permalinks del top-5 de basic-memory aparecen en el
  top-5 de exo.
- Cada permalink ausente se explica por stemming, por multiplicidad de filas o
  por `size`/`tier` rancio en basic-memory (spec §7). **Una ausencia sin
  explicación es un FALLO del port**, no una diferencia aceptable.
- El gate global de `targets` pasa si **pasan 4 de los 5 topics**. Con 3 o menos,
  el port se investiga antes de seguir.

## Qué NO mide este gate

No mide orden, ni score, ni `size_bytes` (será distinto por diseño: `entity.size`
está rancio en 17 de 138 notas y el valor nuevo sale de `stat`), ni `snippet`
(los cuerpos indexados son distintos).
```

- [ ] **Step 3: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-08-18-m6-04-preregistro-targets.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(m6-04): pre-registra el gate de targets antes de tocar código"
```

---

### Task 2: exo — `journal_mode=WAL`

El índice de exo está en journal `delete`. `kbx targets` y `kbx doctor`
mantienen el cursor de lectura abierto mientras leen ficheros y shellean git por
fila; en `delete` eso bloquea al escritor y el indexer del hook de Stop ya
registró un `database is locked` real. WAL elimina la clase en ambos sentidos.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/exo/engine/src/lib.rs:40-54` (`abre_db`)
- Test: `/home/paul/Documentos/proyectos/exo/engine/tests/journal_mode.rs` (crear)

**Interfaces:**
- Consumes: `exo::abre_db(ruta: &Path) -> anyhow::Result<rusqlite::Connection>`
  (existente, firma sin cambios).
- Produces: `abre_db` deja la DB en `journal_mode=wal` de forma persistente.
  `abre_db_en_memoria` NO cambia (`:memory:` no admite WAL, devuelve `memory`).

- [ ] **Step 1: Escribir el test que falla**

Crea `engine/tests/journal_mode.rs`:

```rust
use exo::abre_db;

#[test]
fn abre_db_deja_la_db_en_wal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("indice.db");

    let conn = abre_db(&ruta).expect("abre_db");
    let modo: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .expect("leer journal_mode");
    assert_eq!(modo, "wal", "abre_db debe dejar la DB en WAL, no en {modo}");
}

#[test]
fn wal_es_persistente_entre_aperturas() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ruta = dir.path().join("indice.db");

    {
        let conn = abre_db(&ruta).expect("primera apertura");
        conn.execute_batch("CREATE TABLE t (x INTEGER);")
            .expect("crear tabla");
    }

    // Apertura cruda, sin pasar por abre_db: si WAL no fuese persistente en el
    // fichero, aquí saldría 'delete'.
    let conn = rusqlite::Connection::open(&ruta).expect("segunda apertura cruda");
    let modo: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .expect("leer journal_mode");
    assert_eq!(modo, "wal");
}
```

- [ ] **Step 2: Correr el test y verificar que falla**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test --test journal_mode
```

Expected: FAIL. `abre_db_deja_la_db_en_wal` falla con
`assertion \`left == right\` failed: abre_db debe dejar la DB en WAL, no en delete`.

(`tempfile = "3.14"` ya está en `[dev-dependencies]` de `engine/Cargo.toml`:
no hay que añadir nada.)

- [ ] **Step 3: Implementación mínima**

En `engine/src/lib.rs`, dentro de `abre_db`, **después** del `busy_timeout` y
**antes** del `Ok(conn)`, añade:

```rust
    // journal_mode=WAL (M6-04 §2.2). Persistente en el fichero: basta con
    // fijarlo, no hay que repetirlo por conexión, pero fijarlo en cada
    // apertura es idempotente y cubre el bootstrap de una DB nueva.
    //
    // La razón no es el pre-commit de la KB (ese camino no abre la DB): es que
    // kbx mantiene cursores de lectura abiertos mientras lee ficheros y
    // shellea git por fila (`targets.go:118-146`, `doctor.go:170-190`). En
    // journal `delete` un lector así bloquea al escritor, y el busy_timeout de
    // arriba protege al lector, no al indexer. WAL deja convivir a ambos.
    //
    // `PRAGMA journal_mode` devuelve fila, así que va por query_row: un
    // `execute` fallaría con "Execute returned results".
    let modo: String = conn
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .context("fijar journal_mode=WAL")?;
    if modo != "wal" {
        anyhow::bail!("journal_mode quedó en {modo}, se esperaba wal");
    }
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test --test journal_mode
```

Expected: PASS, `test result: ok. 2 passed`.

- [ ] **Step 5: Verificar que no rompe el resto de la suite**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test
```

Expected: PASS en todos los tests. Presta atención a `tests/refresca.rs` y
`tests/indexer.rs`, que abren DBs en disco.

- [ ] **Step 6: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/lib.rs engine/tests/journal_mode.rs engine/Cargo.toml
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(engine): journal_mode=WAL para que kbx pueda leer sin bloquear al indexer"
```

---

### Task 3: exo — tabla `meta` y `kb_root`

kbx resuelve hoy la raíz de la KB con `SELECT path FROM project ORDER BY id
LIMIT 1`. El índice de exo no tiene tabla `project`. Se le añade una tabla
`meta` que registra **procedencia**: de qué KB salió este índice.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/exo/engine/src/schema.rs:9-45` (`crea_schema`)
- Modify: `/home/paul/Documentos/proyectos/exo/engine/src/indexer.rs:76-79` (`indexa`)
- Test: `/home/paul/Documentos/proyectos/exo/engine/tests/schema.rs` (modificar)
- Test: `/home/paul/Documentos/proyectos/exo/engine/tests/indexer.rs` (modificar)

**Interfaces:**
- Consumes: `exo::schema::crea_schema(conn: &Connection) -> anyhow::Result<()>`,
  `exo::indexer::indexa(kb: &Path, db_ruta: &Path) -> anyhow::Result<Resumen>`
  (ambas existentes, firmas sin cambios).
- Produces: tabla `meta (clave TEXT PRIMARY KEY, valor TEXT NOT NULL)` con la
  fila `('kb_root', <ruta absoluta canónica de la KB>)` tras cada `indexa`.
  La consume kbx en Task 5 vía `SELECT valor FROM meta WHERE clave='kb_root'`.

- [ ] **Step 1: Escribir el test de schema que falla**

En `engine/tests/schema.rs`, cambia la lista de tablas esperadas del test
`schema_crea_todas_las_tablas` para incluir `meta`:

```rust
    for esperado in ["notas", "notas_fts", "aristas", "trozos", "vectores", "meta"] {
```

Y añade al final del fichero:

```rust
#[test]
fn meta_tiene_clave_primaria_y_valor_no_nulo() {
    let conn = abre_db_en_memoria().expect("db en memoria");
    crea_schema(&conn).expect("crea_schema");

    conn.execute("INSERT INTO meta (clave, valor) VALUES ('kb_root', '/tmp/kb')", [])
        .expect("primera fila");

    // clave es PK: un segundo INSERT de la misma clave debe fallar.
    let dup = conn.execute("INSERT INTO meta (clave, valor) VALUES ('kb_root', '/otro')", []);
    assert!(dup.is_err(), "clave debe ser PRIMARY KEY");

    // valor es NOT NULL.
    let nulo = conn.execute("INSERT INTO meta (clave, valor) VALUES ('x', NULL)", []);
    assert!(nulo.is_err(), "valor debe ser NOT NULL");
}
```

- [ ] **Step 2: Correr el test y verificar que falla**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test --test schema
```

Expected: FAIL. `schema_crea_todas_las_tablas` falla con
`falta tabla meta en [...]`, y `meta_tiene_clave_primaria_y_valor_no_nulo`
falla con `no such table: meta`.

- [ ] **Step 3: Añadir la tabla al DDL**

En `engine/src/schema.rs`, dentro del `execute_batch`, **después** del bloque
`CREATE TABLE IF NOT EXISTS trozos (...)` y **antes** de la línea de `vectores`,
añade:

```sql
        CREATE TABLE IF NOT EXISTS meta (
          clave TEXT PRIMARY KEY,
          valor TEXT NOT NULL
        );
```

Y añade este comentario justo encima de `pub fn crea_schema`, tras el bloque de
doc existente:

```rust
/// `meta` (M6-04 §2.1) guarda PROCEDENCIA del índice, no config: `kb_root` es
/// "de qué KB salió este índice", no "qué KB debo usar" (eso será la config
/// propia de exo, C10). La consume kbx para resolver la raíz sin `--kb`
/// explícito, sustituyendo al `project.path` de basic-memory.
///
/// kbx consume `notas`, `aristas`, `notas_fts` y `meta`: tocar esas cuatro
/// mira antes el canary de kbx (`internal/index/schema.go`).
```

- [ ] **Step 4: Correr el test de schema y verificar que pasa**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test --test schema
```

Expected: PASS.

- [ ] **Step 5: Escribir el test de población que falla**

Añade al final de `engine/tests/indexer.rs`:

```rust
#[test]
fn indexa_escribe_kb_root_en_meta() {
    let kb = tempfile::tempdir().expect("tempdir kb");
    std::fs::write(
        kb.path().join("nota.md"),
        "---\ntitle: nota\npermalink: kb/nota\n---\n\n# nota\n",
    )
    .expect("escribir nota");

    let dbdir = tempfile::tempdir().expect("tempdir db");
    let db = dbdir.path().join("indice.db");

    exo::indexer::indexa(kb.path(), &db).expect("indexa");

    let conn = exo::abre_db(&db).expect("abrir db");
    let valor: String = conn
        .query_row("SELECT valor FROM meta WHERE clave='kb_root'", [], |r| r.get(0))
        .expect("leer meta.kb_root");

    let esperado = std::fs::canonicalize(kb.path()).expect("canonicalizar kb");
    assert_eq!(valor, esperado.to_string_lossy());
}

#[test]
fn indexa_dos_veces_no_duplica_kb_root() {
    let kb = tempfile::tempdir().expect("tempdir kb");
    std::fs::write(
        kb.path().join("nota.md"),
        "---\ntitle: nota\npermalink: kb/nota\n---\n\n# nota\n",
    )
    .expect("escribir nota");

    let dbdir = tempfile::tempdir().expect("tempdir db");
    let db = dbdir.path().join("indice.db");

    exo::indexer::indexa(kb.path(), &db).expect("primera corrida");
    exo::indexer::indexa(kb.path(), &db).expect("segunda corrida");

    let conn = exo::abre_db(&db).expect("abrir db");
    let filas: i64 = conn
        .query_row("SELECT COUNT(*) FROM meta WHERE clave='kb_root'", [], |r| r.get(0))
        .expect("contar");
    assert_eq!(filas, 1, "kb_root debe ser upsert, no insert repetido");
}
```

- [ ] **Step 6: Correr el test y verificar que falla**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test --test indexer indexa_escribe_kb_root indexa_dos_veces
```

Expected: FAIL con `leer meta.kb_root: QueryReturnedNoRows`.

- [ ] **Step 7: Poblar `meta` en `indexa`**

En `engine/src/indexer.rs`, dentro de `indexa`, **justo después** de
`crea_schema(&conn)?;` (línea 78) y **antes** de `let rutas_absolutas =
walk_kb(kb)?;`, añade:

```rust
    // meta.kb_root: procedencia del índice (M6-04 §2.1). Se escribe en cada
    // corrida por upsert — si la KB se mueve, el índice siguiente lo refleja.
    // Canónica y absoluta: kbx la usa como raíz para abrir ficheros, y una
    // ruta relativa dependería del cwd del proceso que llame a kbx.
    let kb_abs = std::fs::canonicalize(kb)
        .with_context(|| format!("canonicalizar raíz de KB {}", kb.display()))?;
    conn.execute(
        "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)
         ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
        params![kb_abs.to_string_lossy()],
    )
    .context("escribir meta.kb_root")?;
```

- [ ] **Step 8: Correr los tests y verificar que pasan**

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo test
```

Expected: PASS en toda la suite.

- [ ] **Step 9: Verificación manual sobre una copia del índice real**

**No se toca `~/.exo/index.db`.** Se trabaja sobre una copia:

```bash
mkdir -p /tmp/m6-04 && cp ~/.exo/index.db /tmp/m6-04/copia.db
cd /home/paul/Documentos/proyectos/exo/engine && cargo run --release -- index \
  --kb /home/paul/Documentos/proyectos/kb-demo --db /tmp/m6-04/copia.db --json
python3 -c "
import sqlite3
c=sqlite3.connect('file:/tmp/m6-04/copia.db?mode=ro', uri=True)
print('kb_root =', c.execute(\"SELECT valor FROM meta WHERE clave='kb_root'\").fetchone())
print('journal =', c.execute('PRAGMA journal_mode').fetchone()[0])
print('notas   =', c.execute('SELECT COUNT(*) FROM notas').fetchone()[0])
"
```

Expected: `kb_root = ('/home/paul/Documentos/proyectos/kb-demo',)`,
`journal = wal`, `notas = 138` (o el número actual de notas de la KB).

Este paso demuestra que la tabla es **aditiva**: la copia venía de un índice
construido sin `meta` y no ha hecho falta rebuild.

- [ ] **Step 10: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add engine/src/schema.rs engine/src/indexer.rs engine/tests/schema.rs engine/tests/indexer.rs
git -C /home/paul/Documentos/proyectos/exo commit -m "feat(engine): tabla meta con kb_root como procedencia del índice"
```

---

### Task 4: kbx — fixture del índice de exo

Añade un constructor de fixture con el schema de exo **junto al existente**, sin
tocarlo. Las tareas 5-8 van migrando sus tests a él; Task 9 retira el viejo. Así
cada tarea deja `make check` verde.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/fixtures/index.go`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/fixtures/index_test.go`

**Interfaces:**
- Consumes: `fixtures.Notes []Note` (campos `ID int`, `Title`, `Permalink`,
  `FilePath`, `Tier string`, `Tags []string`, `Size int`, `Date string`),
  `fixtures.KBPath(t *testing.T) string`, y el `fixtureRels []rel` del paquete
  (campos `id`, `fromID int`, `toID any`, `toName`, `relType string`).
- Produces:
  - `fixtures.BuildExoIndex(t *testing.T) string` — DB con el schema de exo en
    `t.TempDir()`, devuelve el path.
  - `fixtures.BuildExoIndexMissingColumn(t *testing.T) string` — igual pero sin
    la columna `notas.tipo`, para el test de drift del canary (Task 5).
  - `fixtures.BuildExoIndexSinMeta(t *testing.T) string` — igual pero sin la
    tabla `meta`, para el test de "índice de un exo viejo" (Task 5).

**Inventario de consumidores del fixture viejo** (12 ficheros, medido con
`grep -rln "fixtures.BuildIndex(\|BuildIndex(t)" --include='*_test.go' .`).
Cada uno tiene tarea asignada; Task 9 comprueba que no queda ninguno:

| Fichero | Migra en |
|---|---|
| `internal/index/db_test.go` | Task 5 |
| `internal/index/schema_test.go` | Task 5 |
| `internal/targets/targets_test.go` | Task 6 |
| `cmd/kbx/targets_test.go` | Task 6 |
| `internal/stale/stale_test.go` | Task 7 |
| `internal/doctor/doctor_test.go` | Task 7 |
| `cmd/kbx/stale_test.go` | Task 7 |
| `cmd/kbx/history_test.go` | Task 8 |
| `cmd/kbx/diffsince_test.go` | Task 8 |
| `cmd/kbx/budget_test.go` | Task 8 |
| `cmd/kbx/main_test.go` | Task 8 |
| `internal/fixtures/index_test.go` | Task 9 (se borra) |

- [ ] **Step 1: Crear la rama en el repo kbx**

```bash
git -C /home/paul/Documentos/proyectos/kbx checkout -b m6-04
git -C /home/paul/Documentos/proyectos/kbx status -sb
```

Expected: `## m6-04`, árbol limpio.

- [ ] **Step 2: Escribir el test que falla**

Añade al final de `internal/fixtures/index_test.go`:

```go
func TestBuildExoIndexTieneLasTablasConsumidas(t *testing.T) {
	db, err := sql.Open("sqlite3", "file:"+BuildExoIndex(t)+"?mode=ro")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	for _, tabla := range []string{"notas", "aristas", "notas_fts", "meta"} {
		var n int
		if err := db.QueryRow(
			`SELECT COUNT(*) FROM sqlite_master WHERE name = ?`, tabla,
		).Scan(&n); err != nil {
			t.Fatalf("consultar sqlite_master por %s: %v", tabla, err)
		}
		if n != 1 {
			t.Errorf("falta la tabla %s en el fixture de exo", tabla)
		}
	}
}

func TestBuildExoIndexFilasCoherentesConNotes(t *testing.T) {
	db, err := sql.Open("sqlite3", "file:"+BuildExoIndex(t)+"?mode=ro")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	// len(Notes) + 1: igual que el fixture de basic-memory, el de exo lleva
	// UNA fila cuyo tipo no es 'note', para que los tests del filtro tengan
	// sujeto.
	var n int
	if err := db.QueryRow(`SELECT COUNT(*) FROM notas`).Scan(&n); err != nil {
		t.Fatal(err)
	}
	if n != len(Notes)+1 {
		t.Errorf("filas en notas = %d, want %d", n, len(Notes)+1)
	}

	var noNote int
	if err := db.QueryRow(`SELECT COUNT(*) FROM notas WHERE tipo <> 'note'`).Scan(&noNote); err != nil {
		t.Fatal(err)
	}
	if noNote != 1 {
		t.Errorf("filas con tipo <> 'note' = %d, want 1", noNote)
	}

	var fts int
	if err := db.QueryRow(`SELECT COUNT(*) FROM notas_fts`).Scan(&fts); err != nil {
		t.Fatal(err)
	}
	if fts != n {
		t.Errorf("notas_fts = %d, notas = %d: el fixture de exo es 1:1", fts, n)
	}
}

func TestBuildExoIndexTieneUnaAristaSinResolver(t *testing.T) {
	db, err := sql.Open("sqlite3", "file:"+BuildExoIndex(t)+"?mode=ro")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	// La forward reference del fixture (rel id 6, toID nil) debe llegar al
	// índice de exo como destino_permalink NULL: es lo que arma la trampa
	// NOT IN/NULL que doctor tiene que sortear (spec §3.2).
	var nulos int
	if err := db.QueryRow(
		`SELECT COUNT(*) FROM aristas WHERE destino_permalink IS NULL`,
	).Scan(&nulos); err != nil {
		t.Fatal(err)
	}
	if nulos != 1 {
		t.Errorf("aristas sin resolver = %d, want 1", nulos)
	}
}

func TestBuildExoIndexKbRoot(t *testing.T) {
	db, err := sql.Open("sqlite3", "file:"+BuildExoIndex(t)+"?mode=ro")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	var valor string
	if err := db.QueryRow(
		`SELECT valor FROM meta WHERE clave = 'kb_root'`,
	).Scan(&valor); err != nil {
		t.Fatalf("leer meta.kb_root: %v", err)
	}
	if valor != KBPath(t) {
		t.Errorf("kb_root = %q, want %q", valor, KBPath(t))
	}
}
```

- [ ] **Step 3: Correr el test y verificar que falla**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL de compilación,
`undefined: BuildExoIndex` en `internal/fixtures/index_test.go`.

- [ ] **Step 4: Implementar los constructores**

Añade al final de `internal/fixtures/index.go`:

```go
// DDL del índice de exo, copiado VERBATIM de
// /home/paul/Documentos/proyectos/exo/engine/src/schema.rs (M6-04 Task 3).
// Mismo contrato que el DDL de basic-memory de arriba: estas constantes solo
// derivan cuando deriva el índice real, y quien vigila esa deriva en
// producción es el canary de doctor (CheckSchema), no estos tests.
//
// `vectores` NO se replica: es una virtual table vec0 y el driver
// mattn/go-sqlite3 no trae ese módulo. No hace falta — kbx no la consulta
// nunca, y por eso tampoco está en la lista `consumed` del canary.
const (
	ddlNotas = `CREATE TABLE notas (
  permalink  TEXT PRIMARY KEY,
  ruta       TEXT NOT NULL UNIQUE,
  titulo     TEXT NOT NULL,
  tipo       TEXT,
  mtime      REAL NOT NULL,
  git_epoch  INTEGER
)`

	ddlNotasFTS = `CREATE VIRTUAL TABLE notas_fts USING fts5(
  titulo, cuerpo,
  permalink UNINDEXED,
  tokenize='unicode61 tokenchars 0x2F'
)`

	ddlAristas = `CREATE TABLE aristas (
  origen            TEXT NOT NULL REFERENCES notas(permalink),
  destino_texto     TEXT NOT NULL,
  destino_permalink TEXT,
  UNIQUE (origen, destino_texto)
)`

	ddlTrozos = `CREATE TABLE trozos (
  id        INTEGER PRIMARY KEY,
  permalink TEXT NOT NULL REFERENCES notas(permalink),
  orden     INTEGER NOT NULL,
  texto     TEXT NOT NULL,
  UNIQUE (permalink, orden)
)`

	ddlMeta = `CREATE TABLE meta (
  clave TEXT PRIMARY KEY,
  valor TEXT NOT NULL
)`
)

// exoNoNotePath es la ruta de la única fila del fixture cuyo `tipo` no es
// 'note'. Ocupa el mismo hueco que la entity `note_type='file'` del fixture de
// basic-memory: dar sujeto a los tests del filtro. Apunta a informe.pdf porque
// es el único fichero del git fixture que no tiene ya una Note asociada
// (añadir uno nuevo obligaría a tocar Notes, gitSteps y sus asserts de
// cardinalidad). Artefacto de fixture: exo jamás indexaría un .pdf — lo que
// estos tests prueban es el filtro por `tipo`, no el walker.
const exoNoNotePath = "informe.pdf"

// buildExo construye el índice de exo aplicando `ddls` y poblándolo desde
// Notes/fixtureRels. Si `omitirTipo` es true, la tabla notas se crea sin la
// columna `tipo` (drift para el canary). Si `omitirMeta` es true, no se crea
// la tabla meta (índice producido por un exo anterior a M6-04).
func buildExo(t *testing.T, omitirTipo, omitirMeta bool) string {
	t.Helper()
	path := filepath.Join(t.TempDir(), "index.db")
	db, err := sql.Open("sqlite3", path)
	if err != nil {
		t.Fatalf("create exo fixture db: %v", err)
	}
	defer db.Close()

	notasDDL := ddlNotas
	if omitirTipo {
		notasDDL = `CREATE TABLE notas (
  permalink  TEXT PRIMARY KEY,
  ruta       TEXT NOT NULL UNIQUE,
  titulo     TEXT NOT NULL,
  mtime      REAL NOT NULL,
  git_epoch  INTEGER
)`
	}
	ddls := []string{notasDDL, ddlNotasFTS, ddlAristas, ddlTrozos}
	if !omitirMeta {
		ddls = append(ddls, ddlMeta)
	}
	for _, ddl := range ddls {
		if _, err := db.Exec(ddl); err != nil {
			t.Fatalf("apply DDL: %v\n%s", err, ddl)
		}
	}

	if !omitirMeta {
		if _, err := db.Exec(
			`INSERT INTO meta (clave, valor) VALUES ('kb_root', ?)`, KBPath(t),
		); err != nil {
			t.Fatalf("insert meta.kb_root: %v", err)
		}
	}

	insertaNota := func(permalink, ruta, titulo, tipo string, orden int) {
		t.Helper()
		cols := "permalink, ruta, titulo, tipo, mtime, git_epoch"
		vals := "?, ?, ?, ?, ?, ?"
		args := []any{permalink, ruta, titulo, tipo, float64(1750000000 + orden), 1750000000 + orden}
		if omitirTipo {
			cols = "permalink, ruta, titulo, mtime, git_epoch"
			vals = "?, ?, ?, ?, ?"
			args = []any{permalink, ruta, titulo, float64(1750000000 + orden), 1750000000 + orden}
		}
		if _, err := db.Exec(
			`INSERT INTO notas (`+cols+`) VALUES (`+vals+`)`, args...,
		); err != nil {
			t.Fatalf("insert notas %s: %v", permalink, err)
		}
		// notas_fts es 1:1 con notas en exo (no hay chunking en el FTS: los
		// trozos viven en su propia tabla). El cuerpo lleva el título para
		// que las queries MATCH del fixture encuentren algo determinista.
		if _, err := db.Exec(
			`INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?, ?, ?)`,
			titulo, "cuerpo de "+titulo, permalink,
		); err != nil {
			t.Fatalf("insert notas_fts %s: %v", permalink, err)
		}
	}

	for i, n := range Notes {
		insertaNota(n.Permalink, n.FilePath, n.Title, "note", i)
	}
	// La fila cuyo tipo no es 'note' (ver exoNoNotePath). Sin aristas: si el
	// filtro desapareciera, sería huérfana visible — que es exactamente el
	// delta que mide Task 10.
	insertaNota("fixture-kb/informe", exoNoNotePath, "informe", "report", len(Notes))

	permalinkPorID := make(map[int]string, len(Notes))
	for _, n := range Notes {
		permalinkPorID[n.ID] = n.Permalink
	}
	for _, r := range fixtureRels {
		origen, ok := permalinkPorID[r.fromID]
		if !ok {
			t.Fatalf("fixtureRels: fromID %d sin Note", r.fromID)
		}
		var destino any // NULL para las forward references
		if r.toID != nil {
			id, esInt := r.toID.(int)
			if !esInt {
				t.Fatalf("fixtureRels: toID %v no es int", r.toID)
			}
			p, existe := permalinkPorID[id]
			if !existe {
				t.Fatalf("fixtureRels: toID %d sin Note", id)
			}
			destino = p
		}
		if _, err := db.Exec(
			`INSERT INTO aristas (origen, destino_texto, destino_permalink) VALUES (?, ?, ?)`,
			origen, r.toName, destino,
		); err != nil {
			t.Fatalf("insert aristas %s->%s: %v", origen, r.toName, err)
		}
	}

	return path
}

// BuildExoIndex crea un índice con el schema de exo en t.TempDir(), poblado
// coherentemente con Notes y fixtureRels. Devuelve el path de la DB.
func BuildExoIndex(t *testing.T) string { return buildExo(t, false, false) }

// BuildExoIndexMissingColumn crea el índice sin la columna `notas.tipo`, para
// probar que el canary detecta drift de columna.
func BuildExoIndexMissingColumn(t *testing.T) string { return buildExo(t, true, false) }

// BuildExoIndexSinMeta crea el índice sin la tabla `meta`: es la forma que
// tiene un índice producido por un binario exo anterior a M6-04, el caso de
// disparo inmediato del canary (spec §3.3).
func BuildExoIndexSinMeta(t *testing.T) string { return buildExo(t, false, true) }
```

- [ ] **Step 5: Correr el test y verificar que pasa**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: PASS, incluidos los cuatro tests nuevos y **todos** los existentes (el
fixture viejo sigue intacto).

- [ ] **Step 6: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/fixtures/index.go internal/fixtures/index_test.go
git -C /home/paul/Documentos/proyectos/kbx commit -m "test(fixtures): añade constructor de índice con el schema de exo"
```

---

### Task 5: kbx — `ProjectPath` desde `meta`, default de `ResolvePath`, y canary

Las tres piezas del paquete `internal/index`: de dónde sale la raíz de la KB,
qué DB se abre por defecto, y qué schema vigila el canary.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/index/db.go:28-58`
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/index/schema.go:16-27`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/index/db_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/index/schema_test.go`

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex(t *testing.T) string`,
  `fixtures.BuildExoIndexMissingColumn(t *testing.T) string`,
  `fixtures.BuildExoIndexSinMeta(t *testing.T) string` (Task 4);
  `fixtures.KBPath(t *testing.T) string`.
- Produces: `index.ProjectPath(db *sql.DB) (string, error)` (firma sin cambios,
  fuente nueva), `index.ResolvePath(flagValue string) string` (firma sin
  cambios, default nuevo), `index.CheckSchema(db *sql.DB) ([]Missing, error)`
  (firma sin cambios, lista `consumed` nueva). Las consumen Tasks 6, 7 y 8.

- [ ] **Step 1: Escribir los tests que fallan**

Añade al final de `internal/index/db_test.go`:

```go
func TestProjectPathLeeMetaKbRoot(t *testing.T) {
	db, err := Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	got, err := ProjectPath(db)
	if err != nil {
		t.Fatalf("ProjectPath: %v", err)
	}
	if got != fixtures.KBPath(t) {
		t.Errorf("ProjectPath = %q, want %q", got, fixtures.KBPath(t))
	}
}

func TestProjectPathSinMetaDevuelveVacioSinError(t *testing.T) {
	db, err := Open(fixtures.BuildExoIndexSinMeta(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	// Contrato heredado (kill-criterion K2 de doctor): sin fila, "" y NO error.
	// Quien decide si "" es fatal es el llamador, que exige --kb explícito.
	got, err := ProjectPath(db)
	if err != nil {
		t.Fatalf("una meta ausente no es un error de ProjectPath: %v", err)
	}
	if got != "" {
		t.Errorf("ProjectPath = %q, want \"\"", got)
	}
}

func TestResolvePathDefaultEsElIndiceDeExo(t *testing.T) {
	t.Setenv("KBX_DB", "")
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatal(err)
	}
	want := filepath.Join(home, ".exo", "index.db")
	if got := ResolvePath(""); got != want {
		t.Errorf("ResolvePath(\"\") = %q, want %q", got, want)
	}
}
```

Añade `"os"` y `"path/filepath"` a los imports de `db_test.go` si no están, y
`"github.com/pguerrerolinares/kbx/internal/fixtures"`.

Añade al final de `internal/index/schema_test.go`:

```go
func TestCheckSchemaExoLimpioNoTieneDrift(t *testing.T) {
	db, err := Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	missing, err := CheckSchema(db)
	if err != nil {
		t.Fatalf("CheckSchema: %v", err)
	}
	if len(missing) != 0 {
		t.Errorf("fixture de exo limpio reportó drift: %+v", missing)
	}
}

func TestCheckSchemaExoDetectaColumnaCaida(t *testing.T) {
	db, err := Open(fixtures.BuildExoIndexMissingColumn(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	missing, err := CheckSchema(db)
	if err != nil {
		t.Fatalf("el drift se reporta como findings, no como error: %v", err)
	}
	if len(missing) != 1 || missing[0].Table != "notas" || missing[0].Column != "tipo" {
		t.Errorf("missing = %+v, want [{notas tipo}]", missing)
	}
}

func TestCheckSchemaExoDetectaMetaAusente(t *testing.T) {
	// El caso de disparo real: kbx nuevo contra un índice construido por un
	// binario exo viejo. Debe salir rojo y accionable, no un error críptico
	// de kbRoot más adelante.
	db, err := Open(fixtures.BuildExoIndexSinMeta(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	missing, err := CheckSchema(db)
	if err != nil {
		t.Fatalf("CheckSchema: %v", err)
	}
	if len(missing) != 1 || missing[0].Table != "meta" || missing[0].Column != "" {
		t.Errorf("missing = %+v, want [{meta }]", missing)
	}
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL. `TestProjectPathLeeMetaKbRoot` con `no such table: project`;
`TestResolvePathDefaultEsElIndiceDeExo` con
`= "/home/paul/.basic-memory/memory.db", want "/home/paul/.exo/index.db"`;
los tres de schema con drift de `entity`/`observation`/`relation`/
`search_index`/`project`.

- [ ] **Step 3: Portar `ProjectPath` y `ResolvePath`**

En `internal/index/db.go`, sustituye el cuerpo de `ProjectPath` y su doc por:

```go
// ProjectPath returns the KB root recorded in the index's `meta` table under
// the key `kb_root` — provenance, not config: it answers "which KB produced
// this index", which is the right question for a reader that must open files
// consistent with what it just queried. Returns "" if the row is absent; it
// never errors on that case — callers decide whether "" is fatal (kbx
// doctor's kill-criterion K2: a kb_root that doesn't resolve on disk falls
// back to requiring --kb explicit, it does not silently guess).
//
// Replaces the `SELECT path FROM project` of basic-memory's schema (M6-04).
func ProjectPath(db *sql.DB) (string, error) {
	var path string
	err := db.QueryRow(`SELECT valor FROM meta WHERE clave = 'kb_root'`).Scan(&path)
	if err == sql.ErrNoRows {
		return "", nil
	}
	if err != nil {
		return "", fmt.Errorf("read meta.kb_root: %w", err)
	}
	return path, nil
}
```

Y en `ResolvePath`, sustituye la última línea:

```go
	return filepath.Join(home, ".exo", "index.db")
```

Actualiza también el doc de `ResolvePath`:

```go
// ResolvePath resolves the index path: --db flag > KBX_DB env > default
// ~/.exo/index.db. Changing this default IS the M6-04 cutover: from here on
// kbx reads exo's index, not basic-memory's.
```

Y el doc del paquete, al principio del fichero:

```go
// Package index reads exo's SQLite index. Strictly read-only: every
// connection opens with mode=ro (config hard veto — no write path).
```

- [ ] **Step 4: Portar la lista `consumed` del canary**

En `internal/index/schema.go`, sustituye el bloque `var consumed` completo por:

```go
// consumed is the exact schema subset kbx depends on (design spec: canary
// over the used subset, NOT the full schema — a harmless exo upgrade must
// not raise a false positive).
//
// Since M6-04 this canary no longer watches a foreign schema drifting under
// us: it watches kbx and exo — two binaries, two repos, independent release
// cycles, installed separately in ~/.local/bin — going out of sync. That
// class of failure is demonstrated, not theoretical: exo's binary once ran
// 20h behind its own campaign's fix with nothing warning about it.
//
// `trozos` and `vectores` are deliberately absent: kbx never queries them.
// `vectores` in particular is a vec0 virtual table whose module this driver
// does not carry — no kbx query may ever force a global scan of
// sqlite_master, or reading the index dies with "no such module: vec0".
var consumed = []struct {
	table   string
	columns []string
}{
	{"notas", []string{"permalink", "ruta", "titulo", "tipo", "mtime", "git_epoch"}},
	{"aristas", []string{"origen", "destino_texto", "destino_permalink"}},
	{"notas_fts", []string{"titulo", "cuerpo", "permalink"}},
	{"meta", []string{"clave", "valor"}},
}
```

- [ ] **Step 5: Correr los tests y verificar el estado**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: los seis tests nuevos **PASAN**. Los tests viejos que usan
`fixtures.BuildIndex` (schema de basic-memory) ahora **FALLAN** — es lo esperado
en este punto: el canary ya mira el schema de exo. Anota cuáles fallan; las
Tasks 6-8 los van migrando y Task 9 retira lo que sobre.

Los que deben pasar a fallar aquí son exclusivamente los de
`internal/index/schema_test.go` que usan el fixture viejo:
`TestCheckSchemaCleanIndexHasNoFindings`, `TestCheckSchemaDetectsDroppedColumn`,
`TestCheckSchemaDetectsMissingTable`.

- [ ] **Step 6: Retirar los tres tests del canary viejo**

Ese trío prueba el canary contra un schema que kbx ya no consume. Bórralos de
`internal/index/schema_test.go` (los tres, con sus comentarios) — sus
equivalentes de exo ya están escritos en el Step 1.

Si `buildBareDB` queda sin usar tras el borrado, bórrala también.

- [ ] **Step 7: Correr la suite y verificar**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
```

Expected: `internal/index` en verde. Otros paquetes (`targets`, `stale`,
`doctor`) siguen verdes porque aún usan el fixture viejo y sus propias queries.

- [ ] **Step 8: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/index/
git -C /home/paul/Documentos/proyectos/kbx commit -m "feat(index): kbRoot desde meta, default a ~/.exo/index.db y canary sobre el schema de exo"
```

---

### Task 6: kbx — `targets`

La query caliente. Cambia el JOIN a claves de texto, el `snippet` de columna, y
saca `tier` y `size` del disco en vez del índice. **El shape JSON de salida no
cambia**: `Candidate` conserva sus seis campos.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/targets/targets.go:63-145`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/targets/targets_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/targets_test.go`

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex(t *testing.T) string`,
  `fixtures.BuildGitRepo(t *testing.T) string`, `index.Open(path string) (*sql.DB, error)`,
  `frontmatter.Tier(content []byte) string`.
- Produces: `targets.Search(db *sql.DB, kbRoot, topic string, limit int) (*Result, error)`
  — firma sin cambios. `Candidate{Permalink, Tier string, SizeBytes int,
  Headings []string, LastCommit, Snippet string}` sin cambios.

- [ ] **Step 1: Migrar los tests al fixture de exo**

En `internal/targets/targets_test.go` **y** en `cmd/kbx/targets_test.go`,
sustituye todas las llamadas a `fixtures.BuildIndex(t)` por
`fixtures.BuildExoIndex(t)`:

```bash
cd /home/paul/Documentos/proyectos/kbx
sed -i 's/fixtures\.BuildIndex(t)/fixtures.BuildExoIndex(t)/g' \
  internal/targets/targets_test.go cmd/kbx/targets_test.go
grep -c "BuildExoIndex(t)" internal/targets/targets_test.go cmd/kbx/targets_test.go
```

Y añade al final del fichero:

```go
func TestSearchExcluyeTipoNoNote(t *testing.T) {
	db, err := index.Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	repo := fixtures.BuildGitRepo(t)

	res, err := targets.Search(db, repo, "informe", 10)
	if err != nil {
		t.Fatalf("Search: %v", err)
	}
	for _, c := range res.Candidates {
		if c.Permalink == "fixture-kb/informe" {
			t.Errorf("targets devolvió la fila tipo='report'; el filtro no está aplicado")
		}
	}
}

func TestSearchLeeTierYSizeDelDisco(t *testing.T) {
	db, err := index.Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	repo := fixtures.BuildGitRepo(t)

	res, err := targets.Search(db, repo, "core-index", 5)
	if err != nil {
		t.Fatalf("Search: %v", err)
	}
	if len(res.Candidates) == 0 {
		t.Fatal("sin candidatas para core-index")
	}
	c := res.Candidates[0]
	if c.Tier != "core" {
		t.Errorf("Tier = %q, want \"core\" (leído del frontmatter en disco)", c.Tier)
	}
	// El índice de exo no guarda size: sale del stat del fichero real.
	info, err := os.Stat(filepath.Join(repo, "core/core-index.md"))
	if err != nil {
		t.Fatal(err)
	}
	if c.SizeBytes != int(info.Size()) {
		t.Errorf("SizeBytes = %d, want %d (stat del fichero)", c.SizeBytes, info.Size())
	}
}
```

Añade `"os"` y `"path/filepath"` a los imports del fichero de test si faltan.

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL en `internal/targets` con `no such table: search_index`.

- [ ] **Step 3: Portar la query**

En `internal/targets/targets.go`, sustituye el bloque de comentario y la
constante `candidateQuery` completos por:

```go
// candidateQuery selects note-typed matches only, ranked by FTS5's default
// bm25 rank, joined to notas by permalink — exo's index keys everything by
// the frontmatter permalink, there are no numeric ids to join on.
//
// tier and size are NOT read from the index: exo deliberately does not
// persist tier (engine/src/nota.rs:14 — a new column would force a rebuild of
// existing DBs) and never persisted size. Both come from disk in Search,
// which already opens every candidate to extract headings, so they cost one
// extra stat and no extra open. Same source `budget` and `ratchet` already
// use (internal/frontmatter).
//
// notas.tipo = 'note' mirrors the old note_type filter. NOTE: in exo's index
// this excludes 57 of 138 real markdown notes (report/project/research/guide/
// spec/person), not assets — the walker only indexes .md. Removing it is a
// deliberate, separate change (M6-04 T3).
//
// No LIMIT here: kept from the basic-memory era, where one note could produce
// several search_index rows. exo's notas_fts is 1:1 with notas, so the dedup
// below is a no-op today — it stays because it costs nothing and stays correct
// the day the FTS indexes trozos instead.
const candidateQuery = `
SELECT notas.permalink,
       notas.ruta,
       COALESCE(snippet(notas_fts, 1, '', '', '…', 12), '') AS snip
FROM notas_fts
JOIN notas ON notas.permalink = notas_fts.permalink
WHERE notas_fts MATCH ? AND notas.tipo = 'note'
ORDER BY rank`
```

- [ ] **Step 4: Portar el escaneo y la lectura de disco**

En la función `Search`, sustituye el bucle `for len(result.Candidates) < limit
&& rows.Next() { … }` completo por:

```go
	for len(result.Candidates) < limit && rows.Next() {
		var permalink, filePath, snippet string
		if err := rows.Scan(&permalink, &filePath, &snippet); err != nil {
			return nil, fmt.Errorf("targets: scan candidate: %w", err)
		}
		if seen[filePath] {
			continue
		}
		seen[filePath] = true

		lastCommit, err := LastCommit(kbRoot, filePath)
		if err != nil {
			return nil, fmt.Errorf("targets: %w", err)
		}

		// tier y size del disco (ver candidateQuery). Un fichero ilegible no
		// descarta la candidata: el índice la conoce, así que se devuelve con
		// tier "" y size 0 — misma regla que ExtractHeadings.
		abs := filepath.Join(kbRoot, filePath)
		tier := ""
		size := 0
		if content, err := os.ReadFile(abs); err == nil {
			tier = frontmatter.Tier(content)
			size = len(content)
		}

		result.Candidates = append(result.Candidates, Candidate{
			Permalink:  permalink,
			Tier:       tier,
			SizeBytes:  size,
			Headings:   ExtractHeadings(abs),
			LastCommit: lastCommit,
			Snippet:    snippet,
		})
	}
```

Añade a los imports de `targets.go`:

```go
	"github.com/pguerrerolinares/kbx/internal/frontmatter"
```

(`os` y `path/filepath` ya están importados.)

- [ ] **Step 5: Correr los tests y verificar que pasan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: `internal/targets` en verde, y los tests de `targets` en `cmd/kbx`
también (migrados en el Step 1). Otros tests de `cmd/kbx` (`budget`, `stale`,
`main`, `history`, `diffsince`) siguen usando el fixture viejo y siguen verdes:
sus queries aún no se han portado.

- [ ] **Step 6: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/targets/ cmd/kbx/targets_test.go
git -C /home/paul/Documentos/proyectos/kbx commit -m "feat(targets): porta la query al índice de exo, con tier y size del disco"
```

---

### Task 7: kbx — `stale` y `doctor`

Van juntas porque comparten la semántica de grado y orfandad, y porque la
guardia contra NULL es la misma trampa en las dos.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/stale/stale.go:130-147`
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/doctor/doctor.go:161-170`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/stale/stale_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/internal/doctor/doctor_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/stale_test.go`

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex(t *testing.T) string`,
  `index.Open(path string) (*sql.DB, error)`.
- Produces: sin cambios de firma. `stale` sigue produciendo filas
  `(permalink, filePath, degree)`; `doctor.orphanFindings` sigue produciendo
  `(findings, waived []Finding, err error)`.

- [ ] **Step 1: Escribir el test de la trampa NULL, que falla**

Añade al final de `internal/doctor/doctor_test.go`:

```go
// El fixture de exo lleva UNA arista sin resolver (destino_permalink NULL).
// Sin la guardia IS NOT NULL, `permalink NOT IN (SELECT destino_permalink …)`
// evalúa a NULL para todas las filas y la query devuelve CERO huérfanas:
// verde, silenciosa, y el check apagado. Es el mismo shape que el bug del
// indexer de C9.
func TestOrphanFindingsSobreviveAAristasSinResolver(t *testing.T) {
	db, err := index.Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()

	var nulos int
	if err := db.QueryRow(
		`SELECT COUNT(*) FROM aristas WHERE destino_permalink IS NULL`,
	).Scan(&nulos); err != nil {
		t.Fatal(err)
	}
	if nulos == 0 {
		t.Fatal("el fixture debe tener al menos una arista sin resolver, o este test no prueba nada")
	}

	rep, err := doctor.Run(db, fixtures.BuildGitRepo(t), doctor.DefaultBudgetOptions())
	if err != nil {
		t.Fatalf("doctor.Run: %v", err)
	}
	var orphans int
	for _, f := range append(rep.Findings, rep.Waived...) {
		if f.Type == "orphan" {
			orphans++
		}
	}
	if orphans == 0 {
		t.Error("cero huérfanas con aristas sin resolver: falta la guardia IS NOT NULL")
	}
}
```

Firmas que usa este test, verbatim del paquete:
`doctor.Run(db *sql.DB, kbRoot string, budget BudgetOptions) (Report, error)`,
`doctor.DefaultBudgetOptions() BudgetOptions`,
`Report{OK bool, Findings []Finding, Waived []Finding}`,
`Finding{Type, Path, Detail string}`.

- [ ] **Step 2: Migrar los tests de ambos paquetes al fixture de exo**

```bash
cd /home/paul/Documentos/proyectos/kbx
sed -i 's/fixtures\.BuildIndex(t)/fixtures.BuildExoIndex(t)/g' \
  internal/stale/stale_test.go internal/doctor/doctor_test.go cmd/kbx/stale_test.go
```

- [ ] **Step 3: Correr los tests y verificar que fallan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL en `internal/stale` y `internal/doctor` con
`no such table: entity`.

- [ ] **Step 4: Portar la query de `stale`**

En `internal/stale/stale.go`, sustituye el bloque de comentario y la constante
`degreeQuery` completos por:

```go
// degreeQuery counts, per note, the aristas rows where the note's permalink
// appears as origen OR as destino_permalink — the same definition as before
// (degree 0 = orphan), expressed as a sum of two independent counts. exo keys
// edges by permalink text, not numeric ids, so the join column changes but the
// semantics do not: an unresolved link carries destino_permalink NULL, which
// matches no note, exactly like basic-memory's NULL to_id forward references.
//
// WHERE tipo = 'note' mirrors the old note_type filter. In exo's index this
// excludes real markdown notes (report/project/…), not assets — see M6-04 T3.
const degreeQuery = `
SELECT notas.permalink,
       notas.ruta,
       (SELECT COUNT(*) FROM aristas WHERE aristas.origen = notas.permalink) +
       (SELECT COUNT(*) FROM aristas WHERE aristas.destino_permalink = notas.permalink) AS degree
FROM notas
WHERE notas.tipo = 'note'
ORDER BY notas.ruta`
```

**Nota**: desaparece el `COALESCE(entity.permalink, '')`. En exo `permalink` es
PRIMARY KEY y por tanto NOT NULL: no puede venir NULL. Si el escaneo del
llamador usa `sql.NullString`, simplifícalo a `string`.

- [ ] **Step 5: Portar la query de `doctor`**

En `internal/doctor/doctor.go`, dentro de `orphanFindings`, sustituye el
`db.Query(...)` completo por:

```go
	rows, err := db.Query(`
		SELECT ruta, permalink
		FROM notas
		WHERE permalink NOT IN (SELECT origen FROM aristas)
		  AND permalink NOT IN (SELECT destino_permalink FROM aristas WHERE destino_permalink IS NOT NULL)
		  AND tipo = 'note'
		ORDER BY ruta
	`)
```

Y actualiza el doc de la función, sustituyendo su bloque de comentario por:

```go
// orphanFindings flags every note with zero relations in either direction
// (spec §3.2): not an origen and not a non-NULL destino_permalink in the
// aristas table. path = notas.ruta, detail = the permalink.
//
// The `WHERE destino_permalink IS NOT NULL` guard is LOAD-BEARING, not
// defensive tidiness: with any unresolved link in the index (23 of 573 in the
// live KB), `NOT IN` over a subquery containing NULL evaluates to NULL for
// every row and the query returns ZERO orphans — green, silent, check
// disabled. Measured: 0 without the guard, 7 with it.
//
// AND tipo = 'note' mirrors the old note_type filter; see M6-04 T3 for why it
// excludes real notes rather than assets.
```

- [ ] **Step 6: Correr los tests y verificar que pasan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
```

Expected: `internal/stale` e `internal/doctor` en verde, incluido
`TestOrphanFindingsSobreviveAAristasSinResolver`.

- [ ] **Step 7: Verificar a mano que la guardia es la que sostiene el check**

Quita temporalmente `WHERE destino_permalink IS NOT NULL` de la query, corre
`make test` y confirma que `TestOrphanFindingsSobreviveAAristasSinResolver`
**falla**. Vuelve a ponerla. Esto demuestra que el test puede fallar; sin esta
comprobación no sabes si el test prueba algo.

- [ ] **Step 8: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/stale/ internal/doctor/
git -C /home/paul/Documentos/proyectos/kbx commit -m "feat(stale,doctor): porta grado y orfandad a aristas, con guardia contra NOT IN/NULL"
```

---

### Task 8: kbx — `history` y `diff-since`

Dos lookups 1:1. Verificado que la convención de path y permalink es **idéntica**
entre los dos índices (set-equality sobre las 138 notas de la KB real, cero
diferencias), así que el port es mecánico.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/history.go:139`
- Modify: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/diffsince.go:180`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/history_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/diffsince_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/budget_test.go`
- Test: `/home/paul/Documentos/proyectos/kbx/cmd/kbx/main_test.go`

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex(t *testing.T) string`.
- Produces: sin cambios de firma ni de salida.

- [ ] **Step 1: Migrar los tests al fixture de exo**

Estos cuatro son los últimos consumidores del fixture viejo. `budget_test.go` y
`main_test.go` no tienen queries que portar —`budget` solo abre la DB para el
fallback de kbRoot y `main` prueba el despacho de subcomandos— pero comparten
fixture, así que migran aquí:

```bash
cd /home/paul/Documentos/proyectos/kbx
sed -i 's/fixtures\.BuildIndex(t)/fixtures.BuildExoIndex(t)/g' \
  cmd/kbx/history_test.go cmd/kbx/diffsince_test.go cmd/kbx/budget_test.go cmd/kbx/main_test.go
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL en `cmd/kbx` con `no such table: entity`.

- [ ] **Step 3: Portar los dos lookups**

En `cmd/kbx/history.go`, sustituye la línea 139 por:

```go
	err := db.QueryRow(`SELECT ruta FROM notas WHERE permalink = ?`, arg).Scan(&filePath)
```

En `cmd/kbx/diffsince.go`, sustituye la línea 180 por:

```go
	err := db.QueryRow(`SELECT permalink FROM notas WHERE ruta = ?`, path).Scan(&permalink)
```

**Nota**: desaparecen los `COALESCE(permalink, '')`. En exo `permalink` es
PRIMARY KEY (NOT NULL) y `ruta` es `NOT NULL UNIQUE`: ninguna de las dos puede
venir NULL. El manejo de `sql.ErrNoRows` de ambos llamadores no cambia.

- [ ] **Step 4: Correr la suite y verificar que pasa**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
```

Expected: **toda** la suite en verde. Comprueba que no queda ningún consumidor
del fixture viejo fuera de `internal/fixtures/`:

```bash
cd /home/paul/Documentos/proyectos/kbx && grep -rln "fixtures.BuildIndex(" --include='*_test.go' .
```

Expected: sin salida.

- [ ] **Step 5: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add cmd/kbx/
git -C /home/paul/Documentos/proyectos/kbx commit -m "feat(history,diff-since): porta los lookups de permalink y ruta al índice de exo"
```

---

### Task 9: kbx — retirar el fixture viejo y correr el gate de paridad

Cierra T2 de la spec. Dos mitades: limpiar el fixture de basic-memory (ya sin
consumidores) y **demostrar** que el port es correcto comparando contra el mundo
viejo.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/fixtures/index.go`
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/fixtures/index_test.go`
- Create: `/home/paul/Documentos/proyectos/exo/docs/superpowers/plans/2026-08-18-m6-04-gate-paridad.md`

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex`, y el binario `kbx` viejo (de `master`).
- Produces: el informe de paridad que Task 10 y Task 11 citan.

- [ ] **Step 1: Comprobar que el fixture viejo no tiene consumidores**

```bash
cd /home/paul/Documentos/proyectos/kbx && grep -rn "BuildIndex(\|BuildIndexExtraChunk(\|BuildIndexMissingColumn(" --include='*.go' .
```

Expected: solo hits dentro de `internal/fixtures/index.go` (las propias
definiciones) y de `internal/fixtures/index_test.go`. Si aparece otro paquete,
migra esa llamada a `BuildExoIndex` antes de seguir.

- [ ] **Step 2: Borrar el fixture de basic-memory**

De `internal/fixtures/index.go`, borra: las constantes `ddlProject`,
`ddlEntity`, `ddlObservation`, `ddlRelation`, `ddlSearchIndex`; el tipo
`entityMetadata`; y las funciones `BuildIndex`, `BuildIndexExtraChunk`,
`BuildIndexMissingColumn`.

Conserva `Notes`, `fixtureRels`, el tipo `rel`, `KBPath` y todo lo de
`buildExo`/`BuildExo*`.

De `internal/fixtures/index_test.go`, borra los tests que referencian las
funciones borradas.

- [ ] **Step 3: Correr la suite**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
```

Expected: verde. Si el compilador se queja de imports sin usar (`encoding/json`,
`fmt`), quítalos.

- [ ] **Step 4: Construir los dos binarios para la comparación**

El viejo sale de `master`, sin tocar el árbol de trabajo:

```bash
mkdir -p /tmp/m6-04
git -C /home/paul/Documentos/proyectos/kbx worktree add /tmp/m6-04/kbx-viejo master
cd /tmp/m6-04/kbx-viejo && $HOME/.local/go/bin/go build -tags sqlite_fts5 -o /tmp/m6-04/kbx-viejo-bin ./cmd/kbx
cd /home/paul/Documentos/proyectos/kbx && $HOME/.local/go/bin/go build -tags sqlite_fts5 -o /tmp/m6-04/kbx-nuevo-bin ./cmd/kbx
ls -l /tmp/m6-04/kbx-viejo-bin /tmp/m6-04/kbx-nuevo-bin
```

Expected: los dos binarios existen.

- [ ] **Step 5: Preparar copias RO de los dos índices**

**No se toca ninguno de los índices vivos.**

```bash
cp ~/.basic-memory/memory.db /tmp/m6-04/bm.db
cp ~/.exo/index.db /tmp/m6-04/exo.db
cd /home/paul/Documentos/proyectos/exo/engine && cargo run --release -- index \
  --kb /home/paul/Documentos/proyectos/kb-demo --db /tmp/m6-04/exo.db --json
```

Expected: envelope JSON válido. La copia queda con `meta` poblada y en WAL.

- [ ] **Step 6: Gate de `doctor` — paridad exacta sobre el conjunto crudo**

El gate **no** se mide sobre el JSON final de `doctor`: el report neto es
`0 findings · 2 waived` en ambos lados, así que pasaría igual con un port roto
que devolviera conjuntos vacíos. Se mide sobre la query cruda:

```bash
python3 - <<'PY'
import sqlite3, subprocess
bm = sqlite3.connect('file:/tmp/m6-04/bm.db?mode=ro', uri=True)
exo = sqlite3.connect('file:/tmp/m6-04/exo.db?mode=ro', uri=True)
b = set(r[0] for r in bm.execute(
  "SELECT file_path FROM entity WHERE id NOT IN (SELECT from_id FROM relation) "
  "AND id NOT IN (SELECT to_id FROM relation WHERE to_id IS NOT NULL) AND note_type='note'"))
e = set(r[0] for r in exo.execute(
  "SELECT ruta FROM notas WHERE permalink NOT IN (SELECT origen FROM aristas) "
  "AND permalink NOT IN (SELECT destino_permalink FROM aristas WHERE destino_permalink IS NOT NULL) "
  "AND tipo='note'"))
print("basic-memory:", len(b), "| exo:", len(e))
print("solo bm :", sorted(b - e))
print("solo exo:", sorted(e - b))
print("GATE:", "PASA" if b == e else "FALLA")
PY
```

Expected: `basic-memory: 7 | exo: 7`, ambas diferencias vacías, `GATE: PASA`.

**Si falla**: el port de `doctor` tiene un bug. No sigas; arréglalo y repite.

- [ ] **Step 7: Gate de `stale` — paridad del grado-0**

```bash
/tmp/m6-04/kbx-viejo-bin stale --db /tmp/m6-04/bm.db --kb /home/paul/Documentos/proyectos/kb-demo --json > /tmp/m6-04/stale-viejo.json
/tmp/m6-04/kbx-nuevo-bin stale --db /tmp/m6-04/exo.db --kb /home/paul/Documentos/proyectos/kb-demo --json > /tmp/m6-04/stale-nuevo.json
python3 - <<'PY'
import json
def grado0(p):
    return set(n["path"] for n in json.load(open(p))["notes"] if n["degree"] == 0)
v, n = grado0('/tmp/m6-04/stale-viejo.json'), grado0('/tmp/m6-04/stale-nuevo.json')
print("grado-0 viejo:", len(v), "| nuevo:", len(n))
print("solo viejo:", sorted(v - n))
print("solo nuevo:", sorted(n - v))
print("GATE:", "PASA" if v == n else "FALLA")
PY
```

Expected: `GATE: PASA`, con las dos diferencias vacías. El shape es
`stale.Result{Now string, Notes []Note}` con `Note{Path, Permalink, Tier,
LastCommit string, Uncommitted bool, AgeDays, Degree int, Score float}`.

**El ranking por grado NO se compara**: exo extrae 573 aristas y basic-memory
674, así que los grados difieren por diseño.

- [ ] **Step 8: Gate de `targets` — pre-registro de Task 1**

Lee
`/home/paul/Documentos/proyectos/exo/docs/superpowers/plans/2026-08-18-m6-04-preregistro-targets.md`
y aplícalo **tal como está escrito**. Para cada uno de los cinco topics:

```bash
for t in "indexer" "reflex" "memoria" "kbx" "recall en el punto de uso"; do
  echo "=== $t ==="
  /tmp/m6-04/kbx-viejo-bin targets "$t" --db /tmp/m6-04/bm.db --kb /home/paul/Documentos/proyectos/kb-demo --limit 5 --json \
    | python3 -c "import json,sys; print('\n'.join(c['permalink'] for c in json.load(sys.stdin)['candidates']))"
  echo "--- exo ---"
  /tmp/m6-04/kbx-nuevo-bin targets "$t" --db /tmp/m6-04/exo.db --kb /home/paul/Documentos/proyectos/kb-demo --limit 5 --json \
    | python3 -c "import json,sys; print('\n'.join(c['permalink'] for c in json.load(sys.stdin)['candidates']))"
done
```

Criterio (verbatim del pre-registro): pasa un topic si ≥3 de los 5 permalinks
del top-5 de basic-memory aparecen en el top-5 de exo; el gate global pasa si
pasan 4 de los 5 topics. Cada ausencia se explica o es un fallo.

- [ ] **Step 9: Escribir el informe del gate**

Crea
`/home/paul/Documentos/proyectos/exo/docs/superpowers/plans/2026-08-18-m6-04-gate-paridad.md`
con: la fecha, los tres resultados (doctor con los conjuntos completos, stale
con los conjuntos de grado-0, targets topic a topic con el conteo de overlap y
la explicación de cada ausencia), y el veredicto global PASA/FALLA. Pega el
output real de los comandos, no un resumen.

- [ ] **Step 10: Commit**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/fixtures/
git -C /home/paul/Documentos/proyectos/kbx commit -m "test(fixtures): retira el fixture del índice de basic-memory"
git -C /home/paul/Documentos/proyectos/exo add docs/superpowers/plans/2026-08-18-m6-04-gate-paridad.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(m6-04): informe del gate de paridad del port"
```

- [ ] **Step 11: Limpiar el worktree temporal**

```bash
git -C /home/paul/Documentos/proyectos/kbx worktree remove /tmp/m6-04/kbx-viejo
```

---

### Task 10: kbx — quitar el filtro `tipo='note'`

Commit propio, con el delta esperado escrito **antes** de medirlo. Es T3 de la
spec: aquí el cambio de números **es** el resultado, no una desviación.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/targets/targets.go` (`candidateQuery`)
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/stale/stale.go` (`degreeQuery`)
- Modify: `/home/paul/Documentos/proyectos/kbx/internal/doctor/doctor.go` (`orphanFindings`)
- Test: los tres ficheros `_test.go` correspondientes

**Interfaces:**
- Consumes: `fixtures.BuildExoIndex` (Task 4), las tres queries portadas
  (Tasks 6 y 7).
- Produces: sin cambios de firma. Cambia el corpus visible.

- [ ] **Step 1: Escribir el test que falla**

En `internal/targets/targets_test.go`, sustituye el test
`TestSearchExcluyeTipoNoNote` (Task 6) por su inverso:

```go
func TestSearchIncluyeTipoNoNote(t *testing.T) {
	db, err := index.Open(fixtures.BuildExoIndex(t))
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	repo := fixtures.BuildGitRepo(t)

	res, err := targets.Search(db, repo, "informe", 10)
	if err != nil {
		t.Fatalf("Search: %v", err)
	}
	var visto bool
	for _, c := range res.Candidates {
		if c.Permalink == "fixture-kb/informe" {
			visto = true
		}
	}
	if !visto {
		t.Error("targets debe devolver también las notas cuyo tipo no es 'note'")
	}
}
```

- [ ] **Step 2: Correr el test y verificar que falla**

```bash
cd /home/paul/Documentos/proyectos/kbx && make test
```

Expected: FAIL, `targets debe devolver también las notas cuyo tipo no es 'note'`.

- [ ] **Step 3: Quitar el filtro de las tres queries**

En `internal/targets/targets.go`, `candidateQuery`:

```go
WHERE notas_fts MATCH ?
ORDER BY rank`
```

En `internal/stale/stale.go`, `degreeQuery`: borra la línea
`WHERE notas.tipo = 'note'` entera (queda `FROM notas` seguido de `ORDER BY notas.ruta`).

En `internal/doctor/doctor.go`, `orphanFindings`: borra la línea
`AND tipo = 'note'`.

En los tres, sustituye el párrafo de comentario que empieza por
`notas.tipo = 'note' mirrors the old note_type filter` (o `WHERE tipo = 'note'
mirrors…`) por:

```go
// No type filter: exo's walker only indexes .md files with a permalink in
// frontmatter, so there are no assets to exclude. The old note_type='note'
// filter was documented as excluding pdf/tex/cls/json assets but actually hid
// 57 of 138 real markdown notes — every report/project/research/guide/spec/
// person note, including the project distillates and Paul's own profile.
// Removed in M6-04 T3 as a deliberate scope change.
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
```

Expected: verde. Si algún test de `stale` o `doctor` asertaba cardinalidad con
el fixture (p.ej. `len(fixtures.Notes) - 2`), ahora cuenta una nota más: la fila
`tipo='report'`. Ajusta el literal y **deja un comentario diciendo por qué subió**.

- [ ] **Step 5: Medir el delta contra la KB real, con los dos números**

```bash
/tmp/m6-04/kbx-nuevo-bin targets "memoria" --db /tmp/m6-04/exo.db --kb /home/paul/Documentos/proyectos/kb-demo --limit 200 --json \
  | python3 -c "import json,sys; print('targets ve:', len(json.load(sys.stdin)['candidates']))"
cd /home/paul/Documentos/proyectos/kbx && $HOME/.local/go/bin/go build -tags sqlite_fts5 -o /tmp/m6-04/kbx-sinfiltro ./cmd/kbx
python3 - <<'PY'
import sqlite3
c = sqlite3.connect('file:/tmp/m6-04/exo.db?mode=ro', uri=True)
tot = c.execute("SELECT COUNT(*) FROM notas").fetchone()[0]
note = c.execute("SELECT COUNT(*) FROM notas WHERE tipo='note'").fetchone()[0]
print(f"corpus total {tot} · tipo='note' {note} · delta bruto {tot-note}")
EXCL = ('archive', '.superpowers', 'docs')
def excluida(r): return any(p in EXCL for p in r.split('/')[:-1])
vis = [r for (r,) in c.execute("SELECT ruta FROM notas WHERE tipo<>'note'") if not excluida(r)]
print(f"delta visible para doctor/stale (tras exclusión por dir): {len(vis)}")
for r in sorted(vis): print("   ", r)
PY
```

Expected: `delta bruto 57` y `delta visible … 23`. **Los dos números son
correctos y distintos**: `targets` no excluye por directorio y ve las 57;
`doctor` y `stale` excluyen `archive`/`.superpowers`/`docs` y ven 23.

- [ ] **Step 6: Commit, con los dos deltas en el mensaje**

```bash
git -C /home/paul/Documentos/proyectos/kbx add internal/
git -C /home/paul/Documentos/proyectos/kbx commit -F - <<'EOF'
feat: quita el filtro tipo='note' — kbx pasa a ver la KB entera

El filtro se documentaba como "excluir assets pdf/tex/cls/json", pero en el
índice de exo no hay assets: el walker solo indexa .md con permalink. Lo que
ocultaba eran 57 de 138 notas markdown reales — los 10 destilados de projects/,
el perfil de Paul (type: person) y los 39 report de archive/sesiones.

Delta esperado, y son DOS números distintos:

  targets        +57  (no excluye por directorio)
  doctor, stale  +23  (excluyen archive/.superpowers/docs)

Medir el delta en doctor y esperar 57 lleva a concluir que este commit está
mal. Cierra T3 de la spec de M6-04.
EOF
```

---

### Task 11: cutover — call sites y red de rollback

Última tarea de código. Deja todo listo para que Paul haga el cambio en el
entorno vivo.

**Files:**
- Modify: `/home/paul/Documentos/proyectos/exo/plugins/reflex/skills/consolida/SKILL.md`
- Modify: `/home/paul/Documentos/proyectos/exo/plugins/process/skills/documenta/SKILL.md` (si menciona kbx)
- Create: `/home/paul/Documentos/proyectos/exo/docs/superpowers/runbooks/2026-08-18-m6-04-cutover.md`

**Interfaces:**
- Consumes: el default nuevo de `index.ResolvePath` (Task 5), el informe del
  gate (Task 9).
- Produces: el runbook que ejecuta Paul.

- [ ] **Step 1: Auditar los call sites**

```bash
grep -rn "kbx " /home/paul/Documentos/proyectos/exo/plugins/ --include='*.sh' --include='*.md' | grep -v "^Binary"
```

Expected: hits en `skills/consolida/SKILL.md` (budget, diff-since, doctor,
rotate, stale), `scripts/kb-precommit.sh` (ratchet, rotate) y
`agents/executor.md` (targets).

- [ ] **Step 2: Comprobar qué invocaciones pasan `--db` explícito**

```bash
grep -rn "kbx .*--db" /home/paul/Documentos/proyectos/exo/plugins/ --include='*.sh' --include='*.md'
```

Expected: **ningún hit**. Todas las invocaciones usan el default, así que el
cambio de `ResolvePath` las mueve a la vez y no hay que editar comandos.

Si aparece algún `--db ~/.basic-memory/memory.db` hardcodeado, cámbialo a
`--db ~/.exo/index.db` en el mismo commit.

- [ ] **Step 3: Actualizar la nota de degradación de `consolida`**

En `plugins/reflex/skills/consolida/SKILL.md`, la línea que documenta el modo
degradado dice hoy `kbx no está → make install`, `schema drift → rebuild`.
Sustituye ese fragmento por:

```markdown
`kbx no está → make install`, `schema drift → el binario kbx y el binario exo
están desincronizados: reinstala el que vaya atrasado (`make install` en kbx,
`cargo build --release` + copia en exo) y vuelve a correr`.
```

Razón: desde M6-04 el drift ya no significa "basic-memory cambió de schema",
significa "mis dos binarios no son de la misma época".

- [ ] **Step 4: Escribir el runbook del cutover**

Crea `docs/superpowers/runbooks/2026-08-18-m6-04-cutover.md`:

```markdown
# M6-04 — cutover: lo que ejecuta Paul

> El código está mergeado. Lo que sigue es **entorno vivo**: instalar binarios
> y verificar. Nada de esto lo ejecuta la fábrica.
>
> Spec: `specs/2026-08-18-m6-04-kbx-al-indice-design.md` ·
> Plan: `plans/2026-08-18-m6-04-kbx-al-indice.md` ·
> Gate de paridad: `plans/2026-08-18-m6-04-gate-paridad.md`

## Antes de tocar nada: la red de rollback

```bash
mkdir -p ~/.local/backups/m6-04
cp ~/.local/bin/kbx ~/.local/backups/m6-04/kbx-pre-m6-04
cp ~/.local/bin/exo ~/.local/backups/m6-04/exo-pre-m6-04
```

`kbx --version` no distingue builds, igual que `exo --version` está fijo en
`0.1.0`. La red es la copia del binario, no la versión.

## Paso 1 — Instalar exo (primero, y no es cosmético)

`meta` solo existe cuando el binario **nuevo** de exo corre `exo index`. Si
instalas kbx antes, kbx se queda sin `kb_root` y todos los comandos exigen
`--kb`.

```bash
cd /home/paul/Documentos/proyectos/exo/engine && cargo build --release
cp target/release/exo ~/.local/bin/exo
exo index --kb ~/Documentos/proyectos/kb-demo --db ~/.exo/index.db --json
```

## Paso 2 — Verificación falsable del índice

```bash
python3 -c "
import sqlite3
c = sqlite3.connect('file:$HOME/.exo/index.db?mode=ro', uri=True)
print('kb_root =', c.execute(\"SELECT valor FROM meta WHERE clave='kb_root'\").fetchone())
print('journal =', c.execute('PRAGMA journal_mode').fetchone()[0])
"
```

Esperado: `kb_root = ('/home/paul/Documentos/proyectos/kb-demo',)` y
`journal = wal`. Un `kb_root` vacío o `journal = delete` significa que el
binario instalado es el viejo — es el gotcha de las 20 h de C9, repetido.

## Paso 3 — Instalar kbx

```bash
cd /home/paul/Documentos/proyectos/kbx && make install
```

## Paso 4 — Verificación falsable de kbx

```bash
kbx doctor --json | head -20
kbx targets "memoria" --json --limit 5
```

Esperado: ambos responden **sin `--kb` ni `--db`**. Si `doctor` reporta
`schema_drift`, el canary está haciendo su trabajo: uno de los dos binarios va
atrasado.

Rollback si algo va mal:

```bash
cp ~/.local/backups/m6-04/kbx-pre-m6-04 ~/.local/bin/kbx
cp ~/.local/backups/m6-04/exo-pre-m6-04 ~/.local/bin/exo
```

El rollback de kbx revierte el port entero: no hay grano fino.

## Paso 5 — `/consolida` de humo

Corre `/consolida` una vez. Es el consumidor que falla-fuerte ante
`schema_drift`, así que es la prueba de integración real del canary.

## Lo que este cutover NO cierra

**M6-06 (recall en el punto de uso)** sigue abierto, y **M5b sigue gated** hasta
que esté. M6-04 cierra uno de los dos bloqueadores, no los dos.
```

- [ ] **Step 5: Verificar que la suite sigue verde**

```bash
cd /home/paul/Documentos/proyectos/kbx && make check
cd /home/paul/Documentos/proyectos/exo/engine && cargo test
```

Expected: ambas verdes.

- [ ] **Step 6: Commit**

```bash
git -C /home/paul/Documentos/proyectos/exo add plugins/ docs/superpowers/runbooks/2026-08-18-m6-04-cutover.md
git -C /home/paul/Documentos/proyectos/exo commit -m "docs(m6-04): runbook de cutover y nota de drift en consolida"
```

---

## Acciones de Paul (entorno vivo — ninguna tarea las ejecuta)

1. Revisar y mergear las dos ramas `m6-04` (repo exo y repo kbx).
2. Ejecutar `docs/superpowers/runbooks/2026-08-18-m6-04-cutover.md`.
3. Decidir sobre la deuda que este plan **no** toca (abajo).

## Lo que este plan NO hace

- **No arregla la transaccionalidad del segundo bucle del indexer**
  (`engine/src/indexer.rs:161-179`, deuda anotada en el runbook de C9). Este
  plan le añade un consumidor: con kbx leyendo en concurrencia, un `doctor`
  puede ver una nota borrada a medias y reportar un `orphan` transitorio falso.
  WAL reduce la ventana; no la cierra. Son las mismas 4 líneas presupuestadas.
- **No toca M6-06** (recall en el punto de uso), que tiene su propio ciclo.
  **M5b sigue gated** hasta que M6-04 y M6-06 estén hechos.
- **No corta la config RO de basic-memory** (`lib.rs:kb_desde_config`,
  `config_embeddings`, `min_similitud_de_config`). Eso es C10. `meta.kb_root` es
  procedencia y no la sustituye.
- **No añade métricas ni harness de medición** (régimen §0). El gate de
  `targets` es un pre-registro manual, deliberadamente.
