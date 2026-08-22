# Backlog de exo — deuda abierta

> Nota viva: deuda técnica y documental de exo con su siguiente paso accionable.
> **No duplica el plan de cierre** (`plans/2026-08-17-cierre-exo-m2-a-m5b.md`), que
> fija QUÉ campañas quedan y en qué orden. Esto es lo que está suelto: hallazgos
> de gate sin barrer y deuda encontrada fuera de campaña. Editar aquí, no
> duplicar. Cada item cita su evidencia; un item sin evidencia verificable no
> entra.
>
> Última revisión: **2026-08-18** (apertura — valoración del repo completo).

## Estado

| | |
|---|---|
| **Cerradas** | C5 (M2-08+09, cierra E1 read) · C6 (M6, cutover del recall) · C7 (M4, write-path) |
| **Pendientes** | C8 (M3+M1b, cutover de skills) → C9 (M5a, MCP + config propia) → C10 (M5b, desinstalar basic-memory) |
| **Medido** | engine-hybrid **48/55** hit@5 vs bm-hybrid 39/55, mismo día, paridad de corpus ∅, recall <2s (`evals/e1-read/verdict/m2-09-corrida.md`) |
| **Tests** | 111 verdes / 0 rojos en la rama de M4, 98 en main previo (contados por el consultor del gate, no por CI — no hay CI) |

---

## Alta

- [ ] **Adelantar M5a-02: config propia y des-hardcodear la KB.** Hoy el engine
  lee modelo, dims y threshold de `~/.basic-memory/config.json` (RO) y resuelve
  la raíz de la KB con `projects["kb-demo"]` literal (`lib.rs:71`). Es decir:
  **el sustituto depende del sustituido para arrancar.** Es el bloqueante duro
  de C10/M5b, ya escrito como requisito transversal en C11 del plan, y cada
  campaña que pasa sin tocarlo lo encarece. El mismo patrón está en los scripts
  de reflex (`basic-memory-recall.sh`, `a1-freeze-watch.sh` con ruta absoluta),
  asignados a C6/M6-02.
  **Acción:** no esperar a C9 — la config propia con fallback RO a basic-memory
  mientras dure el side-by-side es independiente del MCP.

## Media

- [ ] **CI mínimo — el gate que falta.** No hay `.github/`, ni `rustfmt.toml`, ni
  `clippy.toml`, ni `rust-version` en `Cargo.toml` (el plan afirma MSRV ≥ 1.97 y
  nada lo comprueba), ni `LICENSE` en la raíz. Todo lo verifica un consultor a
  mano, por gate. Verificado 2026-08-18: **el crate no compila en el portátil de
  trabajo** — `cargo check --all-targets` falla con
  `cc-rs: failed to find tool "gcc.exe"` (rusqlite bundled + sqlite-vec exigen
  toolchain C, ausente en el target GNU de esa máquina). Nada verifica el build
  fuera de la máquina de desarrollo.
  **Acción:** workflow con `cargo test` + `fmt --check` + `clippy -D warnings`,
  declarar `rust-version`, LICENSE en raíz. En un proyecto cuya tesis es "gates y
  evidencia", este es el gate más barato de todos.

- [ ] **Barrer los hallazgos vivos del gate M4** (`evals/e1-read/verdict/gate-m4.md`).
  Cerrados en `acb312e`: traversal por `..` en `--dir`/`--titulo`, `--force` sin
  rastro en el envelope, flag muerto `--min-similitud` en `write new`. **Vivos 6,
  por orden de daño:**
  - **#5 [media]** sin fallback walk+parse (la spec §3.2 lo afirma en presente):
    con índice rancio, un `--crea` puede dejar **dos ficheros con el mismo
    permalink**. Riesgo hoy bajo (las 26 bitácoras de `log/` son slug-clean).
    Mínimo: walk de confirmación antes de crear.
  - **#3 [media]** el rechazo exit 3 no emite envelope con `--json`: la spec §3.3
    promete `data.dup_candidatas` y solo hay una línea humana en stderr.
    Implementarlo o corregir la spec.
  - **#8 [baja]** divergencia de slug medida **19/127** frente a basic-memory
    (`_` conservado en 10 bitácoras rotadas, CamelCase separado, `§`→`ss`).
    Autoconsistente, pero conviene decidirlo **por escrito antes de M5b**,
    porque las bitácoras rotadas de `/consolida` usan `_` en el título.
  - **#6 [baja]** `--crea` con permalink de 2 segmentos crea directorio espurio;
    `write_append_cmd` asume 3.
  - **#9 [baja]** el `SKILL.md` de `documenta` omite `--db` en los comandos del
    Paso 3; tomados literales fallan con error de clap.
  - **Disenso abierto del consultor:** el prefijo de proyecto sale de
    `kb.file_name()`, no de la config, contra lo que dice la spec §3.1. Hoy
    coinciden; si en M5a el nombre de proyecto ≠ nombre de directorio, revienta
    en silencio. **Que C9 lo herede como requisito explícito.**

## Baja

- [ ] **Rot documental.** El `README.md` sigue diciendo "M2 (E1 read) al 7/9 —
  falta `exo recall` (M2-08) y la corrida final (M2-09)" con M2, M6 y M4 cerrados
  y mergeados. El propio plan lo listaba como deuda a barrer en C5 y ya van dos
  campañas. **Acción:** actualizar estado y añadir puntero a este backlog.

- [ ] **Nombres y ubicaciones.** `docs/superpowers/` como carpeta de docs del
  proyecto cuyo objetivo declarado es jubilar superpowers, y `reports/` colgando
  de la raíz fuera de toda convención (los verdicts sí viven ordenados en
  `evals/*/verdict/`). **Acción:** decidir de una vez — renombrar o escribir por
  qué se queda. Barato ahora, caro cuando haya más ficheros.

- [ ] **Residuos de entorno del plan** (ya listados allí, se repiten aquí para no
  perderlos): `crontab -r` pendiente de M1a · `reflex-baseline.sh` traga errores
  de `jq` con `2>/dev/null` · cachés huérfanas de reflex 0.6.0/0.8.0.

- [ ] **Decisión abierta: `archive/` en el ranking.** Es el 32%–39% del índice
  (54 de 138 notas). Se decide con la corrida de C5 delante o se cierra
  declarando que se queda indexado. Llevar la decisión abierta indefinidamente es
  peor que cualquiera de las dos opciones.

- [ ] **Privacy-pass a `evals/` antes de cualquier remote público.** Las queries
  del eval son reales y están trackeadas; el snapshot del log está en
  `.gitignore`, el eval set no. Spec §6.6, espíritu clean-room.

---

## Cerrado con evidencia (para no re-proponer)

- [x] **Revisión de HuggingFace pineada: cerrado el 2026-08-22.** `repo_hf`
  (`lib.rs`) resuelve `jinaai/jina-embeddings-v2-base-es` contra el sha
  `8e2d780d…`, el snapshot que generó la línea base del eval; un modelo ajeno
  sigue cayendo a `main` pero el engine lo avisa por stderr. Anotado en la spec
  de fusión §4.6b. 2 tests unitarios vistos fallar primero.

- [x] **Modo mudo de `busca_hybrid`: cerrado el 2026-08-22.** `Busqueda` gana
  `avisos: Vec<String>` (aditivo, omitido cuando está vacío, `search_type`
  intacto porque lo comparan los scripts del eval). `avisos_cobertura_vector`
  compara `vectores` contra `trozos` y distingue arm INERTE (0 vectores) de
  cobertura PARCIAL (con cifras); corpus vacío no avisa, que es el falso rojo
  simétrico. Los avisos salen además por **stderr con y sin `--json`**, así que
  nunca contaminan el envelope y siempre se ven. 4 tests nuevos vistos fallar
  primero (`tests/buscador.rs`), 124 verdes en la suite. Se mantiene el
  contrato de Task 3 (0 vectores ⇒ 0 resultados, no error): avisa, no falla.

- [x] **exo NO degrada a vector-hash como `empirica`** (2026-08-18, lectura de
  `buscador.rs` e `indexer.rs`): un fallo de embed sube por `?` con contexto
  (`indexer.rs:247`, `buscador.rs:241`) y aborta el comando con exit ≠ 0. No hay
  fallback silencioso a hash ni basura *válida* entrando al índice. **La mitad
  mala de la respuesta** es el modo mudo de `busca_hybrid`, promovido a item de
  prioridad alta arriba.

- [x] **Válvula de embed de query vía API de Jina: no se activa.** Era
  condicional a que el hybrid frío no bajara de p95 < 2 s, y la corrida de M2-09
  mide recall < 2 s. Queda anotada por si el corpus crece; nunca OpenAI ni otro
  modelo (rompería la atribución del eval). GPU sigue descartada: no ataca la
  latencia de arranque.

- [x] **Veto AGPL sostenido bajo verificación adversarial**: el consultor del
  gate M4 inspeccionó `escritor.rs` completo — Rust original, diseño replicado
  contra oráculos de la KB de producción, sin copia ni vendorizado posible
  (basic-memory es Python).

- [x] **Permalinks del frontmatter jamás regenerados**, verificado con `xxd` en
  el gate M4 y con paridad de corpus ∅ en M2-09 (138/138, 0 regenerados).
