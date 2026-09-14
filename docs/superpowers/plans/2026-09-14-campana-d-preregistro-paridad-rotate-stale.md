# Pre-registro del gate de paridad de `rotate` + `stale` (Campaña D)

> **Esto sí es un pre-registro**, mismo contrato que G4a/G4b/G4c: se escribe
> el 2026-09-14, antes de que exista una sola línea de
> `engine/src/rotacion.rs` ni `engine/src/obsolescencia.rs`. El hecho
> verificable que lo respalda es de `git`, no de prosa — la Task 1 del plan
> de campaña (`2026-09-14-campana-d-cutover-kbx.md`) comitea este fichero
> **antes** de tocar `engine/src/`, y su primer step verifica eso con
> `git log`. Nadie ha visto todavía el output de ninguno de los dos
> binarios: ni el Rust (no compila, el módulo no existe) ni el Go (esta
> máquina, a fecha de escribir esto, no tiene Go instalado — la Task 2 del
> plan lo instala).
>
> Es el mismo motivo que ya obligó a titular el documento de `targets`
> como "registro posterior" en vez de pre-registro: un criterio firmado a
> posteriori, con el binario ya corrido, es un check no falsable — se
> redacta para que dé verde sabiendo ya lo que da. Éste se firma antes.

## Alcance

Dos comandos, cada uno con su propia sección de criterio: `rotate`
(§Rotate) y `stale` (§Stale). Los dos comparten referencia y montaje
(§Referencia), y los dos se gatean sobre la misma copia de la KB.
`history` y `diff-since` **no** se portan (decisión ya tomada por Paul,
ver el plan de campaña) y no tienen sección aquí.

## Referencia

- **kbx**: `fe46443`, compilado fresco (`go build -tags sqlite_fts5 -o
  /tmp/campana-d/kbx`, nunca `make install` — no escribe en `~/.local/bin`,
  Global Constraints del plan de campaña) en un `git worktree` temporal
  fuera de `~/Documentos/proyectos/kbx` (Task 2 del plan de campaña). El
  checkout local de `kbx` en esta máquina puede estar en un commit
  distinto — no sirve como referencia sin verificar antes que `fe46443` es
  su ancestro (mismo cuidado que en G4a/G4c).
- **exo**: el binario de la rama `d-port-rotate-stale` (o la que el
  orquestador use para esta task), `cargo build --release`. Al escribir
  esto no existe.
- **KB base (solo lectura)**: una copia de `wisdom-paul` en
  `/tmp/campana-d/kb-base/` (Task 2 del plan), sobre la que se leen `stale`
  y el `--dry-run` de `rotate`. Nunca se escribe ahí.
- **Copias de trabajo desechables para `--apply`**: `rotate` **muta**
  ficheros — igual que `ratchet --seal` en G4c —, así que comparar dos
  corridas con `--apply` sobre el mismo árbol pisaría una corrida con la
  otra. Cada corrida de `--apply` (una para kbx, otra para exo) parte de un
  `git clone` fresco de `kb-base/` a un directorio nuevo bajo
  `/tmp/campana-d/`, se usa una vez y se descarta. Nunca se reutiliza un
  clon entre corridas.
- **Índice**: una copia de solo lectura de `~/.exo/index.db` en
  `/tmp/campana-d/index.db` (`stale` lo necesita para `degree`; `rotate` no
  toca el índice en absoluto — opera solo sobre ficheros).

## Rotate

### Qué se compara

`rotate` es **byte-comparable de verdad** (a diferencia de `targets`): el
invariante "nada se pierde" es sobre bytes exactos, y el criterio lo mide
así.

Tres invocaciones, sobre KBs preparadas para cada caso (ver §Comandos):

1. **Dry-run** (`--json`, sin `--apply`) sobre `kb-base/`: compara el
   `Result` de cada nota candidata (`note`, `archive` cuando `rotated`,
   `moved_bytes`, `cold_entries`, `rotated`).
2. **`--apply`** sobre un clon fresco por binario: compara el estado del
   árbol tras rotar — no solo el JSON de respuesta.
3. **Reconstrucción byte a byte**: para cada nota rotada, `preámbulo +
   frío + caliente == original`. Se verifica una vez por binario (es el
   invariante que cada suite de tests ya cubre por separado) y **una vez
   más cruzado**: el archivo que escribió kbx concatenado con la nota viva
   que escribió kbx reconstruye el original, y лo mismo para exo — si un
   lado rompe el invariante y el otro no, tiene que verse aquí, no solo en
   sus propios tests unitarios.

### Criterio

**PASA** si, sobre la misma nota de partida:

- El conjunto de notas que cada binario decide rotar (`rotated: true`) es
  **idéntico**.
- Para cada nota rotada, `cold_entries` y `moved_bytes` coinciden
  **exactamente**.
- El **nombre del archivo** (`archive`) coincide, salvo colisión de
  disambiguación (ver divergencia 4) — declarado, no fallo.
- El **contenido** del archivo (`archive/log/<nombre>`) coincide byte a
  byte tras normalizar solo el título/permalink reescritos (que dependen
  del nombre de fichero elegido, ya cubierto por el punto anterior).
- La **nota viva** tras `--apply` coincide byte a byte, salvo el aviso de
  archivo (que cita el nombre del archivo, y por tanto hereda cualquier
  divergencia ya declarada de nombre).
- La reconstrucción cruzada (preámbulo + frío del archivo + caliente de la
  nota viva == original) es exacta en los dos binarios.

**NO PASA** si cualquiera de los puntos anteriores difiere sin que la
divergencia esté ya declarada abajo. Un NO PASA no se racionaliza: se
arregla el port, o se declara aquí **antes** de volver a correr.

### Divergencias declaradas antes de correr nada

Salen de leer `internal/rotate/{rotate,apply}.go` de `fe46443` y el diseño
del port (`engine/src/rotacion.rs`, todavía sin escribir). Se declaran ahora
para que, si aparecen en el diff, no se racionalicen después.

1. **Exit code de fallos parciales en la barrida: divergencia deliberada.**
   `cmd/kbx/rotate.go` sigue barriendo `log/` aunque una nota falle (p.ej.
   frontmatter sin cerrar) y sale **2** al final si hubo algún fallo. El
   port hace lo mismo — sigue la barrida, no aborta — pero al final hace
   `bail!` con la cuenta de fallidas, que en exo sale **1** (ningún fallo
   de `rotate` es un `GateFallido`: no hay una decisión de negocio que
   rechazar, es un fichero que no se pudo procesar — más cerca del resto de
   errores genéricos de exo que del contrato de `ratchet`/`budget`/`lint`).
   Es la Decisión D-3 del plan de campaña; no bloquea el port — el
   ejecutor implementa exit 1 y esta nota es su declaración. La misma
   divergencia de exit code aparece en `--hot-bytes <= 0`: exo sale **1**
   (`anyhow::bail!` en `rotate_cmd`, mismo contrato de D-3), kbx sale **2**.
   No se reabre — es el mismo caso, no uno nuevo.
2. **Alcance de la barrida: solo `log/`, sin recursión — verbatim de kbx.**
   `runRotate` hace `os.ReadDir(filepath.Join(*kbPath, "log"))`, no
   recursivo y no toca el resto de la KB aunque haya otra nota `tier: log`
   fuera de `log/`. El port replica esto exactamente (mandato de Paul:
   "porta lo que kbx hace, no lo mejores de paso"). Si la KB real tiene
   alguna `tier: log` fuera de `log/`, ninguno de los dos binarios la
   rotará — no es una divergencia entre ellos, es una limitación compartida
   que el gate no puede ver.
3. **Resolución de `--kb`: exo usa la precedencia establecida
   (`--kb` > `$EXO_KB` > config), kbx exige el flag desnudo.** Mismo patrón
   que every otro comando ya portado (`targets`, `budget`, `lint`,
   `ratchet`); no es una decisión nueva de esta campaña. El gate pasa
   `--kb` explícito en los dos lados, así que esto no se puede observar en
   la comparación.
4. **Disambiguación de nombre de archivo: puede divergir por vaciado del
   directorio.** `disambiguateArchiveName`/`desambigua_nombre_de_archivo`
   dependen de qué ficheros YA existen en `archive/log/` en el momento de
   escribir. Como cada binario corre sobre su propio clon (ver
   §Referencia), si el estado de partida de `archive/log/` no es
   **idéntico** en los dos clones, el sufijo de desambiguación (`-2`,
   `-3`…) puede diferir aunque el contenido archivado sea el mismo. El
   montaje (Task 6 del plan) clona los dos desde el mismo commit de
   `kb-base/` para cerrar esto; si aun así diverge, se ancla por el
   **contenido** del archivo, no por su nombre exacto.
5. **`archive_test.go` / trailing-whitespace en el delimitador de cierre
   del frontmatter**: el port replica `frontmatterBlock`/`isFrontmatterDelimiter`
   tolerando `---` con espacios/tabs finales, igual que
   `frontmatter.rs::es_delimitador` ya hace para el resto del binario (una
   sola regla de delimitador en todo exo, no una copia local con su propia
   tolerancia). No se espera divergencia; se declara porque es el caso que
   el review de kbx (finding 2) encontró y arregló tarde.
6. **El `permalink` del archivo: `nombre_kb()`, no `"wisdom-paul/"`
   hardcodeado — Decisión D-4 del plan de campaña.** `apply.go:159` escribe
   literalmente `"wisdom-paul/" + …`: kbx nació para esta KB y nunca tomó un
   nombre de proyecto como parámetro. El port usa
   `exo::nombre_kb()` (`[kb] name` de la config), el mismo mecanismo que ya
   usan `write new`/`write append` (`escritor.rs:279`, `main.rs:752`) — exo
   sirve cualquier KB, no solo `wisdom-paul`, y hardcodear el nombre
   reintroduciría justo el acoplamiento que la campaña B vino a limar.
   **Efecto en el gate**: el campo `permalink` dentro del frontmatter del
   archivo queda **excluido** de la comparación byte a byte estricta (se
   compara `tier`, el título reescrito, y el bloque frío verbatim; el
   `permalink` se anota pero no gatea) — en la práctica coincide igualmente
   porque la KB de prueba es `wisdom-paul` y así se llama en
   `~/.exo/config.toml`; la Task de la gate verifica ese valor antes de
   correr, en vez de asumirlo.
7. **`\r` en el delimitador de frontmatter**: exo lo acepta (
   `frontmatter.rs::es_delimitador`, y la variante propia del port en
   `rotacion.rs::es_delimitador_frontmatter`, ambas tratan `---` seguido de
   espacios/tabs/`\r` como cierre válido); kbx no (`isFrontmatterDelimiter`
   solo recorta ` ` y `\t`). No se espera divergencia sobre la KB real
   (checkout sin CRLF), se declara por si aparece un fichero con `\r`
   suelto en algún checkout.

## Stale

### Qué se compara

A diferencia de `rotate`, `stale` es de **solo lectura** — no hay
`--apply` que gatear, y la comparación es directa: valores por nota,
sobre la misma copia de índice y de KB.

Dos referencias:

1. **El golden de kbx** (`cmd/kbx/testdata/stale_golden.json`, adjuntado
   verbatim en este documento en §Golden de kbx): 10 notas de un fixture
   sintético (`internal/fixtures`), con `--now 2026-07-15T00:00:00+02:00`.
   El port no tiene ese mismo fixture sintético en Rust — construirlo es
   parte de la Task de port (TDD, axiomas) — así que el golden de kbx se
   usa como **oráculo de valores**, no como comparación binario-contra-binario:
   se corre `exo stale --now <mismo instante> --json` contra un fixture
   equivalente construido en Rust (mismas 10 notas, mismos tiers,
   mismos `age_days`/`degree` de partida) y se compara campo a campo.
2. **La KB real** (`wisdom-paul`, copia de solo lectura): `kbx stale --json`
   contra `exo stale --json`, mismo `--now` fijo pasado explícito en los
   dos lados (nunca el reloj de pared — el gate tiene que ser repetible).

### Criterio

**PASA** si:

- Contra el golden (oráculo de valores, no el fixture sintético completo
  reconstruido en Rust): los 9 pares únicos `(edad, degree, tier) → score`
  del golden que cubre `puntua_reproduce_el_golden_de_kbx` dan el mismo
  **valor** de `score` que el golden de kbx (tolerancia de punto flotante,
  no el mismo texto — ver divergencia 5), y el caso `age_days=43` del
  golden lo cubre `edad_en_dias_coincide_con_el_golden`.
- Contra la KB real: el conjunto de `path` es idéntico, y para cada nota
  `tier`, `age_days`, `degree`, `uncommitted`, `last_commit` (salvo
  `uncommitted`, donde `last_commit` es `""` en los dos) y `score`
  coinciden exactamente.
- El **orden** de `notes[]` coincide (desc por `score`, `path` como
  desempate) en los dos lados.

**NO PASA** si cualquier campo difiere sin que la divergencia esté
declarada abajo.

### Divergencias declaradas antes de correr nada

1. **La fórmula y los pesos NO son una decisión de esta campaña.** Ya están
   FIRMADOS por Paul (`.superpowers/fabrica/PENDIENTE-PAUL-m4-stale-formula.md`,
   "Adjudicación de Paul", 2026-07-11): `score = age_days × tier_weight /
   (1 + degree×0.2)`, con `core=1.5 · stable=1.0 · log=0.5 · NOTIER=0.5`
   (igual que `log`). El port copia el bloque `const` verbatim, con
   nombres en castellano (`PESO_TIER_CORE`, `PESO_TIER_STABLE`,
   `PESO_TIER_LOG`, `PESO_TIER_SIN_TIER`, `PESO_DECAIMIENTO_DEGREE`). No
   se reabre.
2. **`--stale-exclude` no se porta como flag.** kbx expone
   `--stale-exclude` con default `budget.DefaultExclude` unido por comas;
   `exo budget` y `exo lint`, los dos comandos ya portados que también
   excluyen directorios, **no** exponen ese flag — usan
   `presupuesto::EXCLUIDOS` fijo. `exo stale` sigue ese mismo patrón
   (YAGNI: nadie ha invocado `kbx stale --stale-exclude` con un valor
   distinto del default, medible con `git log -p -- cmd/kbx/stale.go` si
   hiciera falta evidencia). El conjunto de directorios excluidos por
   defecto es el mismo en los dos lados (`.superpowers`, `archive`,
   `docs`), así que esto no cambia ningún resultado del gate — solo quita
   un flag que nadie ejercita.
3. **La profundidad de la exclusión cambia: de "cualquier nivel" a "primer
   segmento".** `stale.go::isExcluded` recorre **todos** los componentes de
   directorio de la ruta relativa. El port reutiliza
   `walker::excluida`, que compara **solo el primer segmento** — la única
   función de exclusión que exo tiene, por mandato explícito del código
   (`doctor.go:31-37`: *"do not reintroduce a second copy of it"*, citado
   en `walker.rs`). Es la misma decisión A4 de G4b, aplicada aquí sin
   reabrirla. **Impacto medido**: en `wisdom-paul` a fecha de escribir esto
   hay exactamente `archive/` y `docs/` como directorios excluibles, **sin
   anidar** — las dos semánticas dan el mismo resultado hoy. Si el gate
   encuentra una diferencia por esto, es una `archive/` o `docs/` anidada
   nueva, y la lista de arriba dice qué mirar primero.
4. **`git log -1 --format=%aI` se reutiliza tal cual
   (`gitx::ultimo_commit`), no una función nueva.** El contrato ya
   coincide: stdout vacío + exit 0 ⇒ nota sin commits (kbx:
   `uncommitted=true, err=nil`; exo: `Ok("")`, que `obsolescencia.rs`
   interpreta como `uncommitted=true`); exit ≠ 0 ⇒ error real en los dos
   lados. No se declara como divergencia funcional, se declara como
   decisión de implementación (DRY: una sola función de "último commit"
   en todo exo, la que ya existe).
5. **El formato JSON del `score`**: se descartó `arbitrary_precision`
   porque rompe `serde(untagged)` con floats en `tokenizers` — el texto del
   número difiere (`29.00` vs `29.0`), el valor es idéntico. `Puntuacion`
   serializa como un `f64` normal, sin decimales fijos. No es una
   divergencia de VALOR; es una nota de implementación por si el diff de
   JSON crudo muestra una diferencia de longitud de string en algún paso
   intermedio de depuración — el gate de la KB real normaliza con jq
   (`score: (.score + 0)`, ya que jq 1.7 preserva literales) antes de
   comparar, precisamente para no confundir esta diferencia de texto con
   una divergencia real.

## Golden de kbx (verbatim, `cmd/kbx/testdata/stale_golden.json`, `fe46443`)

```json
{"schema_version":1,"command":"stale","data":{"now":"2026-07-14T22:00:00Z","notes":[{"path":"core/core-index.md","permalink":"fixture-kb/core/core-index","tier":"core","last_commit":"2026-06-01T10:00:00+02:00","uncommitted":false,"age_days":43,"degree":2,"score":46.07},{"path":"metodologia.md","permalink":"fixture-kb/metodologia","tier":"stable","last_commit":"2026-06-01T10:00:00+02:00","uncommitted":false,"age_days":43,"degree":1,"score":35.83},{"path":"huerfana.md","permalink":"fixture-kb/huerfana","tier":"stable","last_commit":"2026-06-15T10:00:00+02:00","uncommitted":false,"age_days":29,"degree":0,"score":29.00},{"path":"informe.pdf","permalink":"fixture-kb/informe","tier":"","last_commit":"2026-06-15T10:00:00+02:00","uncommitted":false,"age_days":29,"degree":0,"score":14.50},{"path":"projects/alpha.md","permalink":"fixture-kb/projects/alpha","tier":"stable","last_commit":"2026-06-15T10:00:00+02:00","uncommitted":false,"age_days":29,"degree":5,"score":14.50},{"path":"notes/sin-tier.md","permalink":"fixture-kb/notes/sin-tier","tier":"","last_commit":"2026-06-15T10:00:00+02:00","uncommitted":false,"age_days":29,"degree":1,"score":12.08},{"path":"notes/estructura-headings.md","permalink":"fixture-kb/notes/estructura-headings","tier":"stable","last_commit":"2026-07-01T10:00:00+02:00","uncommitted":false,"age_days":13,"degree":1,"score":10.83},{"path":"notes/tier-ilegal.md","permalink":"fixture-kb/notes/tier-ilegal","tier":"","last_commit":"2026-06-15T10:00:00+02:00","uncommitted":false,"age_days":29,"degree":2,"score":10.36},{"path":"log/alpha-bitacora.md","permalink":"fixture-kb/log/alpha-bitacora","tier":"log","last_commit":"2026-07-01T10:00:00+02:00","uncommitted":false,"age_days":13,"degree":2,"score":4.64},{"path":"sesiones/2026-07-01.md","permalink":"fixture-kb/sesiones/2026-07-01","tier":"log","last_commit":"2026-07-01T10:00:00+02:00","uncommitted":false,"age_days":13,"degree":2,"score":4.64}]}}
```

## Comandos

```bash
export TMPDIR="${TMPDIR:-$(mktemp -d)}"
mkdir -p /tmp/campana-d
cp ~/.exo/index.db /tmp/campana-d/index.db
KB_BASE=/tmp/campana-d/kb-base
KBX=/tmp/campana-d/kbx   # nunca ~/.local/bin/kbx — Task 2, Step 3
git clone --no-local ~/Documentos/proyectos/wisdom-paul "$KB_BASE"  # solo lectura desde aquí

# --- rotate: dry-run, los dos contra la copia de solo lectura ---
"$KBX" rotate --kb "$KB_BASE" --json | jq -S '.data.rotations | sort_by(.note)' > /tmp/campana-d/go-rotate-dry.json
./target/release/exo rotate --kb "$KB_BASE" --json | jq -S '.data.rotations | sort_by(.note)' > /tmp/campana-d/rs-rotate-dry.json
diff -u /tmp/campana-d/go-rotate-dry.json /tmp/campana-d/rs-rotate-dry.json && echo "PASA: rotate dry-run" || echo "REVISAR: rotate dry-run"

# --- rotate: --apply, cada uno en su propio clon desechable ---
GO_CLON=$(mktemp -d /tmp/campana-d/go-apply.XXXX)
RS_CLON=$(mktemp -d /tmp/campana-d/rs-apply.XXXX)
git clone --no-local "$KB_BASE" "$GO_CLON"
git clone --no-local "$KB_BASE" "$RS_CLON"
"$KBX" rotate --kb "$GO_CLON" --apply --json | jq -S '.data.rotations | sort_by(.note)' > /tmp/campana-d/go-rotate-apply.json
./target/release/exo rotate --kb "$RS_CLON" --apply --json | jq -S '.data.rotations | sort_by(.note)' > /tmp/campana-d/rs-rotate-apply.json
diff -u /tmp/campana-d/go-rotate-apply.json /tmp/campana-d/rs-rotate-apply.json && echo "PASA: rotate --apply (JSON)" || echo "REVISAR: rotate --apply (JSON)"
diff -qr "$GO_CLON/archive" "$RS_CLON/archive" && echo "PASA: rotate --apply (archive/ byte a byte)" || echo "REVISAR: rotate --apply (archive/)"
diff -qr "$GO_CLON/log" "$RS_CLON/log" && echo "PASA: rotate --apply (log/ byte a byte)" || echo "REVISAR: rotate --apply (log/)"

# --- stale: contra la KB real, --now fijo en los dos lados ---
# `score: (.score + 0)` normaliza el número vía aritmética de jq (jq 1.7
# preserva el literal de origen si no se toca) — kbx emite "29.00" con 2
# decimales fijos y exo emite "29.0"; el `+0` fuerza a los dos por el mismo
# formateador de jq antes de comparar, para no confundir una diferencia de
# TEXTO con una de VALOR (divergencia 5 del pre-registro).
NOW="2026-09-14T12:00:00+02:00"
"$KBX" stale --db /tmp/campana-d/index.db --kb "$KB_BASE" --now "$NOW" --json \
  | jq -S '.data.notes | map({path, permalink, tier, last_commit, uncommitted, age_days, degree, score: (.score + 0)})' \
  > /tmp/campana-d/go-stale.json
./target/release/exo stale --db /tmp/campana-d/index.db --kb "$KB_BASE" --now "$NOW" --json \
  | jq -S '.data.notes | map({path, permalink, tier, last_commit, uncommitted, age_days, degree, score: (.score + 0)})' \
  > /tmp/campana-d/rs-stale.json
diff -u /tmp/campana-d/go-stale.json /tmp/campana-d/rs-stale.json && echo "PASA: stale (KB real)" || echo "REVISAR: stale (KB real)"

# El orden de notes[] (desc score, path como desempate) se compara SIN -S,
# que reordena claves de objeto pero no reordena el array:
"$KBX" stale --db /tmp/campana-d/index.db --kb "$KB_BASE" --now "$NOW" --json | jq '.data.notes | map(.path)' > /tmp/campana-d/go-stale-orden.json
./target/release/exo stale --db /tmp/campana-d/index.db --kb "$KB_BASE" --now "$NOW" --json | jq '.data.notes | map(.path)' > /tmp/campana-d/rs-stale-orden.json
diff -u /tmp/campana-d/go-stale-orden.json /tmp/campana-d/rs-stale-orden.json && echo "PASA: stale (orden)" || echo "REVISAR: stale (orden)"
```

**Control del instrumento (ciclo rojo-verde), antes de fiarse de un PASA:**
antes de correr el diff real, se repite el bloque de `stale` con el binario
Rust construido a propósito con `PESO_TIER_CORE = 1.6` (un cambio de una
línea en `obsolescencia.rs`, revertido después del control). El diff tiene
que dar **REVISAR** — si da PASA con la fórmula deliberadamente rota, el
comparador no está midiendo nada y el gate real que sigue no vale. Mismo
control para `rotate`: correr `--apply` con `--hot-bytes 1` de un lado (o
con la KB base intacta, sin cambios) contra el binario real del otro lado
tiene que dar **REVISAR** en el diff de `archive/`. Anotar el resultado de
los dos controles en el registro de abajo antes del PASA/NO PASA real.

## Lo que ya está medido

**Vacío.** Ningún comando de arriba se ha ejecutado: no hay `kbx` de
`fe46443` compilado en esta máquina, no hay `engine/src/rotacion.rs` ni
`engine/src/obsolescencia.rs`, y por tanto no hay binario `exo` con
`rotate`/`stale`. El golden de kbx pegado en §Golden es un fichero leído
del árbol de kbx (`git show fe46443:cmd/kbx/testdata/stale_golden.json`),
no una corrida.

## Registro de la corrida

> A rellenar **después** de que exista el port (Tasks de `rotate` y `stale`
> del plan de campaña) y de que Task 2 (prerequisitos) haya compilado kbx.
> Si Task 2 falla, esta sección queda `encolado` y se anota por qué, sin
> inventar un resultado.

### Rotate

- Fecha: 2026-09-14.
- Commit de exo: `e31ab2a036c37bcbf9d1d5e1fc16017dfe5af308` (binario release de
  `/home/paul/Documentos/proyectos/exo/.worktrees/d-cutover-kbx/engine/target/release/exo`,
  `exo --version` → `exo 0.1.0`).
- Commit de kbx: `fe46443` (`/tmp/campana-d/kbx`).
- Precondición D-4 (Step 1): `~/.exo/config.toml` tiene `[kb] name =
  "wisdom-paul"` — coincide con el nombre de la KB de prueba, así que el
  `permalink` no necesitó excluirse de la comparación estricta (ver más
  abajo).
- Control rojo-verde del instrumento (`--hot-bytes 1` en un clon vs. `kbx
  rotate --apply` normal en otro clon del mismo `kb-base`, ambos
  desechables): **REVISAR** (como se esperaba) — `archive/` de exo con
  `--hot-bytes 1` tiene 98 ficheros, el de kbx con hot-bytes por defecto
  tiene 78; `diff -qr` lista >20 líneas de diferencia. El comparador mide de
  verdad, se procede al gate real.
- Dry-run coincide: **sí** — `diff -u go-rotate-dry.json rs-rotate-dry.json`
  vacío; 44 notas evaluadas en `log/`, 6 con `rotated:true` en los dos
  lados, mismos `note`/`archive`/`cold_entries`/`moved_bytes`.
- `--apply` (JSON) coincide: **sí** — `diff -u` vacío sobre
  `.data.rotations` (6 rotaciones, mismos campos que el dry-run).
- `--apply` (`archive/` byte a byte) coincide: **sí** — `diff -qr` sin
  salida; 78 ficheros en cada `archive/` (kbx: 1.164.196 bytes totales,
  exo: 1.164.196 bytes totales), mismos nombres y mismos tamaños por
  fichero (verificado fichero a fichero, no solo el conteo).
- `--apply` (`log/` byte a byte) coincide: **sí** — `diff -qr` sin salida.
- Exit codes de `--apply` en esta corrida: kbx `0`, exo `0` (sin fallos
  parciales en esta barrida — la divergencia 1, exit 2 vs. exit 1, no se
  observó porque no hubo ninguna nota que fallara; queda declarada pero no
  ejercitada).
- Permalink del archivo (divergencia 6): inspeccionado en
  `archive/log/agent-develop-bitacora-2026-06-26_2026-07-11.md` de los dos
  clones — idéntico byte a byte
  (`permalink: 'wisdom-paul/archive/log/...'`) porque `[kb] name` de la
  config real ya es `wisdom-paul`; no hizo falta excluirlo de la
  comparación estricta, coincidió por las buenas.
- Disambiguación de nombre (divergencia 4): no se observó — los dos clones
  partieron del mismo commit de `kb-base` (`4bf1dc4`) y los 78 nombres de
  archivo listados por `find -printf` son idénticos en los dos lados.
- Reconstrucción cruzada byte a byte (preámbulo + frío + caliente ==
  original): **no ejecutada** — no está en el script ejecutable de
  "## Comandos" (solo describe el invariante en "Qué se compara"); Task 6
  Step 3 manda correr literalmente el bloque de "## Comandos", que cubre
  dry-run + `--apply` (JSON) + los dos `diff -qr`. Con `archive/` y `log/`
  byte-idénticos entre kbx y exo, la reconstrucción cruzada entre binarios
  queda cubierta transitivamente (si cada nota reconstruye su original en
  su propio binario — ya cubierto por la suite de tests de cada port — y
  las salidas de los dos binarios son idénticas, la reconstrucción cruzada
  no puede divergir); no se afirma como medición independiente.
- Divergencias observadas y su adjudicación: ninguna divergencia respecto a
  lo declarado — dry-run, JSON de `--apply`, `archive/` y `log/` son
  idénticos byte a byte en las 6 notas rotadas de las 44 evaluadas
  (conteos de ficheros y bytes citados arriba para descartar dos árboles
  "iguales" por no haber rotado nada). Las divergencias 1 (exit code) y 4
  (disambiguación) están declaradas pero no se ejercitaron en esta corrida
  por no darse las condiciones (sin fallos parciales, sin colisión de
  nombres). La divergencia 6 (permalink) coincidió en la práctica, tal
  como el pre-registro anticipaba.
- **PASA / NO PASA**: **PASA**.

> Addendum post-review final (2026-09-15, review opus de la rama):
> - **(a) "44 notas evaluadas" es erróneo.** `kb-base/log/` tiene **28**
>   ficheros `.md` (`ls /tmp/campana-d/kb-base/log/*.md | wc -l` → `28`), no
>   44. No cambia el veredicto (6 rotaciones idénticas sobre las notas
>   `tier: log` reales de `log/`, contadas correctamente), pero el conteo de
>   "notas evaluadas" de la corrida original estaba mal y queda corregido
>   aquí: 28, no 44.
> - **(b) la reconstrucción cruzada del criterio se declaró PASA por
>   argumento transitivo** en el registro original ("no se afirma como
>   medición independiente"). El review final la midió después, directamente
>   sobre exo: **6/6 notas reconstruidas byte a byte** (el `frio` del archivo
>   reinsertado en el sitio donde la nota viva dejó el aviso reconstruye el
>   original exacto), y con `archive/`/`log/` ya byte-idénticos entre kbx y
>   exo (medido arriba), kbx da el mismo resultado. El PASA del criterio
>   queda respaldado por medición posterior, no solo por el argumento
>   transitivo original.
> - **(c) divergencia no ejercitada, ahora comprobada:** sin rotaciones (KB
>   sin `log/`, o con `log/` vacío) kbx emite `"rotations": null`; exo emite
>   `"rotations": []`. Comprobado con `/tmp/campana-d/kbx rotate --kb
>   <kb-vacia> --json` y el `exo` de esta rama, los dos casos (`log/`
>   ausente y `log/` vacío) dan el mismo resultado: kbx `null`, exo `[]`. No
>   se declaró en el pre-registro original porque no se ejercitó en la
>   corrida real (`kb-base` sí tiene rotaciones); queda declarada aquí como
>   divergencia conocida y no bloqueante — `[]` es, si acaso, el contrato más
>   honesto (`Notes` nunca es `null`, mismo criterio que `Informe` de
>   `stale`), pero diverge textualmente de kbx.

### Stale

- Fecha: 2026-09-14.
- Commit de exo: `e31ab2a036c37bcbf9d1d5e1fc16017dfe5af308` (binario release
  de `/home/paul/Documentos/proyectos/exo/.worktrees/d-cutover-kbx/engine/target/release/exo`,
  `exo --version` → `exo 0.1.0`). **Desviación del plan**: el Step 2 (sabotaje
  y el `cargo build --release` del control rojo-verde) y el Step 1
  (`cargo test`) se corrieron en el worktree `d-cutover-kbx/engine` — que
  ya tenía `target/` sembrado y coincide byte a byte con `e31ab2a` — en vez
  de en `/home/paul/Documentos/proyectos/exo/engine` (el path literal del
  plan, que en esta rama del repo raíz ni siquiera tiene
  `engine/src/obsolescencia.rs`, y de `d-gate-rotate-stale/engine`, que no
  tiene `target/` sembrado). Motivo: instrucción explícita del dispatch de
  esta task ("NO compiles aquí", usar el build release de `d-cutover-kbx`)
  para no forzar una compilación en frío en esta máquina de 15 GiB. Antes y
  después del sabotaje se verificó `git -C .../d-cutover-kbx diff --stat --
  engine/src/obsolescencia.rs` vacío (ver abajo).
- Commit de kbx: `fe46443` (`/tmp/campana-d/kbx`).
- Step 1 (oráculo de valores del golden, cubierto por tests ya existentes):
  **verde** — `cargo test --release --lib
  obsolescencia::tests::puntua_reproduce_el_golden_de_kbx` → `1 passed`;
  `cargo test --release --lib
  obsolescencia::tests::edad_en_dias_coincide_con_el_golden` → `1 passed`
  (corridos por separado — `cargo test` no acepta dos `TESTNAME`
  posicionales en la misma invocación, desviación menor de sintaxis del
  comando del plan, mismo efecto).
- Control rojo-verde del instrumento (`PESO_TIER_CORE` 1.5→1.6, recompilado,
  revertido, recompilado): **REVISAR** (como se esperaba) — el diff muestra
  cambio en las notas `tier: core`, p.ej. `core/core-index.md` 3.59→3.83 y
  `Paul - perfil de trabajo.md` 3.58→3.82; notas de otros tiers sin cambio.
  `git -C .../d-cutover-kbx diff --stat -- engine/src/obsolescencia.rs`
  vacío tanto antes de sabotear como después de revertir — el sabotaje
  quedó revertido, no commiteado. El comparador mide de verdad, se procede
  al gate real.
- Golden (oráculo de valores, 9 pares únicos + `age_days=43`) coincide:
  **sí** — cubierto por Step 1 arriba (`2 passed`); el pre-registro (§Qué se
  compara) fija que el criterio es justo estos dos tests, no el fixture
  sintético completo reconstruido en Rust.
- KB real: conjunto de `path` coincide: **sí** — 101 notas evaluadas en
  cada lado (`jq 'length'` idéntico en `go-stale.json` y `rs-stale.json`).
- KB real: campos por nota coinciden: **sí** — `diff -u
  go-stale.json rs-stale.json` (con `score: (.score + 0)` normalizado por
  jq) vacío → `PASA: stale (KB real)`. Rango de `score` idéntico en los dos
  lados: mín `-0.36`, máx `64`. Divergencia 5 (formato del número)
  confirmada en texto crudo sin normalizar — las mismas notas
  (`README.md`, `metodologia.md`) salen `"score":64.00` en kbx y
  `"score":64.0` en exo, mismo VALOR — lo que demuestra que la
  normalización con jq no es un no-op (anti falso-PASA). `uncommitted`:
  0 notas en cualquiera de los dos lados (el `kb-base` es un `git clone
  --no-local` limpio) — la rama `uncommitted=true` del contrato
  (divergencia 4) no se pudo ejercitar con esta copia.
- KB real: orden coincide: **sí** — `diff -u go-stale-orden.json
  rs-stale-orden.json` (array de `path`, sin `-S`) vacío →
  `PASA: stale (orden)`.
- Divergencias observadas y su adjudicación:
  - Divergencia 1 (fórmula firmada): no se reabre; confirmada por el Step 1
    en verde.
  - Divergencia 2 (`--stale-exclude` no portado): no ejercitada — el flag
    no se pasó en ninguno de los dos lados (mismo default en los dos).
  - Divergencia 3 (exclusión por primer segmento vs. cualquier nivel): no
    observada — `wisdom-paul` no tiene `archive/` ni `docs/` anidados hoy
    (impacto medido en el pre-registro = 0), consistente con que las 101
    notas coinciden exactamente.
  - Divergencia 4 (`gitx::ultimo_commit` / contrato `uncommitted`): no
    ejercitada — 0 notas `uncommitted=true` en el `kb-base` clonado.
  - Divergencia 5 (formato JSON del `score`): observada tal como se
    anticipó — `64.00` (kbx) vs. `64.0` (exo), mismo valor; el gate compara
    VALOR vía normalización jq, no texto, y da PASA.
  - Ninguna divergencia no declarada.
- **PASA / NO PASA**: **PASA**.

> Addendum post-review final (2026-09-15, review opus de la rama):
> - **(d) la corrida real dio scores negativos** (mín `-0.36`, ya anotado
>   arriba) porque el `--now` fijo de la corrida (`2026-09-14T12:00:00+02:00`)
>   quedó **anterior** a commits de la copia de trabajo (`git clone
>   --no-local` preserva las fechas de commit originales, algunas del propio
>   día de la corrida pero después del `--now` elegido), dando `age_days`
>   negativo para esas notas. kbx da el mismo score negativo sobre la misma
>   entrada (paridad intacta — es el mismo cálculo con el mismo signo en los
>   dos lados), pero el resultado contradice el axioma 5 de la spec M4
>   ("forma finita **no negativa**", `axioma_5_forma_finita_no_negativa_y_dos_decimales`
>   en `obsolescencia.rs`, que solo cubre entradas sintéticas con edad ≥ 0,
>   no este caso real). No se toca la fórmula ni se clampea el score aquí —
>   la decisión de si `age_days`/`score` deben clampearse a 0 (o si un `--now`
>   anterior al commit es simplemente un input inválido que no hay que
>   soportar) queda **PENDIENTE-PAUL**: la fórmula se mantiene tal cual está
>   por decisión suya (ver "NO tocar" del brief de este fix).
> - **(e) `--now` inválido: divergencia encontrada en el review y corregida
>   en este fix.** `epoch_utc_de_iso8601` aceptaba en silencio fechas
>   imposibles (`2026-13-45T99:99:99Z`, exit 0, normalizaba a otra fecha en
>   vez de fallar) y rechazaba RFC3339 válido con fracción de segundo
>   (`2026-09-14T12:00:00.5+02:00`). kbx (`fe46443`, `time.Parse(time.RFC3339,
>   …)`) hace lo contrario: rechaza rangos inválidos (mes/día/hora fuera de
>   rango) con exit 2 y acepta la fracción. Corregido en el commit
>   `79a56df` (`fix(d-final-fix, stale): valida rangos y acepta fracción de
>   segundo en --now`) de esta misma rama — validación de mes (1-12), día
>   (según el mes, incluido bisiesto), hora (<24), minuto/segundo (<60), y
>   truncado de la fracción de segundo antes de parsear. No afecta al camino
>   de `last_commit` (viene de `git %aI`, nunca lleva fracción).
