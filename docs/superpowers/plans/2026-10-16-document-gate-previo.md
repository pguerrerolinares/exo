# Plan: /document con gate previo, auto-distill sin pérdida y distill semanal

> For agentic workers: ejecución con `exo:orchestrate`.

**Goal:** que `/document` decida por número antes de escribir qué va al canon, que los punteros a la bitácora sobrevivan a la rotación y que `/distill` vuelva a ser semanal y de juicio.

**Architecture:** tres cambios de engine pequeños e independientes (nombre de archive por cabeceras, `hot_bytes` 40960, `write append` rechaza cabecera duplicada), reescritura de las skills `document` y `distill` con el gate, la cabecera única, el trailer `Gate:` y el auto-distill síncrono en subagente, y un aviso de una línea en `exo-recall.sh`. Sin estado nuevo: el trailer del commit es la única fuente.

**Tech Stack:** Rust (engine, `cargo test`), bash + jq (hooks del plugin, suites `test-*.sh`), Markdown (skills).

**Spec:** `docs/superpowers/specs/2026-10-16-document-gate-previo-design.md`

**Plan:** 25 min, 14 turnos, 16 KB

**Global Constraints:**
- Techo de una nota = `kbx_budget_max` del frontmatter si existe; si no, nominal del tier: core 8.500 B, stable 12.500 B.
- Gate de `/document`: tamaño actual (`wc -c <ruta>`) ≥ 85 % del techo ⇒ al canon solo la línea de estado, sustituyendo a la anterior; todo el detalle a la bitácora. Nunca evicción en caliente.
- Objetivo de poda de distill y auto-distill: ≤60 % del techo. Si no se alcanza sin tocar reglas (`## For agents`, `## Reglas duras`) ni decisiones vivas ⇒ partición (solo el distill manual crea notas; el auto-distill solo la propone).
- `--hot-bytes` default: `40960`.
- Cabecera de entrada de bitácora: `## AAAA-MM-DD — <tema concreto>` (separador ` — `, em dash con espacios). Puntero del canon: `[[<slug>-bitacora]], AAAA-MM-DD «<tema concreto>»`. Etiquetas prohibidas sin tema detrás: «movido del canon», «delta», «detalle», «distill».
- Cabecera de bloque movido: `## AAAA-MM-DD — movido del canon: <tema>`.
- Trailer del commit `documenta` cuando muerde el gate, una línea por nota: `Gate: <ruta> bytes=<N> limit=<techo> routed=bitacora`.
- Búsqueda del trailer siempre con `--fixed-strings` (las rutas llevan ` — ` y paréntesis).
- Auto-distill: subagente `model: sonnet`, contexto limpio, síncrono, tope 10 min; rama `distill/auto/<slug>` (slug = último segmento del permalink de la nota) en worktree de la KB; `--kb <worktree>` obligatorio en todo comando `exo`; sidecar `.distill/<slug>-moved.md`; nunca toca la bitácora, nunca crea notas, nunca merge ni push; `git log -p -n 10` de la nota en el brief. Commit en su rama: `docs(kb): auto-distill <slug> — <evicción|propuesta de partición>`.
- Rechazos del engine salen con exit 3; `--force` los deja pasar.
- Los archives existentes no se renombran.

## Olas

- Ola 1, en paralelo: T1, T2, T3, T4, T5. `Files` disjuntos; ninguna consume código de otra (el trailer `Gate:` y el subject de auto-distill son Global Constraints, no salidas de T3).
- Ola 2: T6 (wiring: bump del plugin), consume T3-T5.

### Task 1: rotate — nombre de archive por cabeceras y hot_bytes 40960

**Files:**
- Modify: `engine/src/rotacion.rs`
- Modify: `engine/src/main.rs`
- Test: `engine/tests/rotar_cli.rs`

**Interfaces:**
- Produces: `exo::rotacion::nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String` (misma firma; cambia la semántica: fechas solo de líneas `## ` fuera de vallas, y solo fechas válidas)
- Produces: `exo rotate` con `--hot-bytes` default `40960`

**Tests:**
- `nombre_de_archivo_ignora_fechas_del_cuerpo` (unit, `rotacion.rs`): frío `"## 2026-10-14 — a\n- caduca el 2026-02-31; informe el 2026-10-21\n\n## 2026-10-15 — b\nx\n\n"` da `"bizkaia-now-bitacora-2026-10-14_2026-10-15.md"`. Falla si el rango sale del cuerpo (bug medido: 19/53 archives).
- `nombre_de_archivo_descarta_fechas_invalidas`: cabeceras `## 2026-02-31 — x` y `## 2026-10-14 — y` dan `<slug>-2026-10-14_2026-10-14.md`. Falla si una fecha imposible entra en el nombre.
- `nombre_de_archivo_cae_a_parte_si_solo_el_cuerpo_tiene_fechas`: cabeceras sin fecha y cuerpo con `2026-10-14`, `existentes = 2`, da `<slug>-parte-03.md`. Falla si el cuerpo sigue aportando fechas.
- `nombre_de_archivo_cuenta_fecha_en_cualquier_posicion_de_la_cabecera`: `## Ola 3 (2026-10-05)` y `## 2026-10-07 — z` dan `<slug>-2026-10-05_2026-10-07.md`. Falla si solo se mira el prefijo de la cabecera.
- `nombre_de_archivo_ignora_cabeceras_en_valla_y_subcabeceras`: una línea `## 2026-01-01 — v` dentro de un bloque ```` ``` ```` y una `### 2026-12-31 — w` fuera, más `## 2026-10-14 — a`, dan `<slug>-2026-10-14_2026-10-14.md`. Falla si cuenta vallas o `###`.
- `hot_bytes_por_defecto_es_40960` (`rotar_cli.rs`): `exo rotate --json` sin `--hot-bytes` sobre una KB con una bitácora da `data.hot_bytes == 40960`. Falla si el default sigue en 20480.
- Los tests existentes `nombre_de_archivo_usa_el_rango_de_fechas_iso` y `nombre_de_archivo_cae_a_numero_de_parte_sin_fechas` siguen en verde sin cambiar sus aserciones.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked` → todo verde; `cargo clippy --manifest-path engine/Cargo.toml --all-targets --locked -- -D warnings` y `cargo fmt --manifest-path engine/Cargo.toml --check` → sin salida de error.

**Review Focus:**
- Cabecera con fecha válida y cuerpo con fecha futura (caso real `2026-10-21`): `nombre_de_archivo_ignora_fechas_del_cuerpo`.
- `###` y cabeceras dentro de vallas: `nombre_de_archivo_ignora_cabeceras_en_valla_y_subcabeceras`.
- CRLF al final de la cabecera: la fecha se extrae igual (añadir el caso `\r\n` al primer test).
- Validación de día por mes no exigida: `2026-02-30` pasa (la spec pide mes 1-12, día 1-31); no añadir calendario.

**Notas:** reusa `lineas_con_offset` y el tratamiento de vallas de `offsets_de_entradas` para recorrer solo las líneas `## `. Actualiza el doc-comment de `nombre_de_archivo` (dice «fechas ISO del bloque frío»).

### Task 2: write append rechaza cabecera duplicada

**Files:**
- Modify: `engine/src/escritor.rs`
- Test: `engine/tests/escritor.rs`
- Test: `engine/tests/rechazo_envelope.rs`

**Interfaces:**
- Produces: variante `Rechazo::CabeceraDuplicada { cabeceras: Vec<String> }` (exit 3 por el `downcast_ref::<Rechazo>` existente en `main.rs:466`; `main.rs` no se toca)
- Produces: `escribe_append(kb: &Path, ruta_rel: &str, texto: &str, forzar: bool) -> Result<Escritura>` (misma firma; nuevo rechazo)

**Tests:**
- `append_rechaza_cabecera_que_ya_existe`: nota `tier: log` con `## 2026-10-16 — foo`; append de `"## 2026-10-16 — foo\n- x\n"` da `Err` que hace downcast a `Rechazo::CabeceraDuplicada` con `cabeceras == ["## 2026-10-16 — foo"]`, y el fichero queda byte a byte igual. Falla si escribe antes de comprobar.
- `append_con_force_admite_cabecera_duplicada`: mismo caso con `forzar = true` anexa y `Escritura.forzado == true`. Falla si `--force` no la deja pasar.
- `append_con_cabecera_nueva_anexa`: `## 2026-10-16 — bar` sobre la misma nota anexa. Falla si el rechazo es por fecha y no por línea literal.
- `append_a_canon_sigue_rechazando_por_tier_primero`: nota `tier: stable` que contiene la cabecera; el rechazo es `AppendACanon`, no `CabeceraDuplicada`. Falla si cambia la precedencia.
- `append_compara_cabecera_sin_espacio_final_ni_cr`: nota con `## 2026-10-16 — foo \r` y bloque con `## 2026-10-16 — foo` rechaza. Falla si un espacio o CR final cuela un duplicado.
- `rechazo_cabecera_duplicada_data_y_display` (`rechazo_envelope.rs`): `data()` trae `cabeceras` como array de strings, con el mismo esquema que las otras variantes; el `Display` nombra la cabecera y menciona `--force`. Falla si el consumidor no puede ver qué cabecera chocó.

**Verificación:** `cargo test --manifest-path engine/Cargo.toml --locked` → todo verde; clippy y fmt como en T1.

**Review Focus:**
- Precedencia tier > duplicado: `append_a_canon_sigue_rechazando_por_tier_primero`.
- Espacios y CR finales: `append_compara_cabecera_sin_espacio_final_ni_cr`.
- Bloque entrante con varias cabeceras, una duplicada: rechaza y lista solo la duplicada (añadir a `append_rechaza_cabecera_que_ya_existe`).
- Una cabecera repetida dentro del propio bloque entrante no es rechazo de esta regla (solo se compara contra la nota).

**Notas:** solo mira la nota viva, no `archive/` (spec). Lee la nota línea a línea con `BufRead` comparando solo líneas que empiezan por `## `; no la cargues entera en un `String` (el doc de `escribe_append` justifica no cargar bitácoras enteras). Actualiza ese doc-comment.

### Task 3: skill document — gate previo, cabecera única, trailer y auto-distill

**Files:**
- Modify: `plugins/exo/skills/document/SKILL.md`
- Modify: `plugins/exo/skills/document/routing.md`
- Create: `plugins/exo/skills/document/auto-distill.md`

**Interfaces:**
- Produces: trailer `Gate: <ruta> bytes=<N> limit=<techo> routed=bitacora` en commits `docs(kb): documenta …`
- Produces: subject `docs(kb): auto-distill <slug> — <evicción|propuesta de partición>` en la rama `distill/auto/<slug>`

**Tests:** n/a — prosa de skill; la cubren `check-skill-refs.sh` y la revisión.

**Verificación:** `bash scripts/check-skill-refs.sh` → exit 0; `bash scripts/test-docs-vivos.sh` → exit 0.

**Review Focus:**
- Paso 2 de `SKILL.md`: el gate usa `wc -c` y la regla de techo de Global Constraints, no `exo budget` (que no lista notas por debajo del 85 %).
- La regla vieja «si el pre-commit rechaza… parte la nota» se reescribe coherente con el gate (el pre-commit queda como red), sin contradecirlo.
- `auto-distill.md` es el brief literal para el subagente: incluye la pregunta de `consolidacion.md` («¿qué párrafo ya no paga su sitio?»), `git log -p -n 10`, `--kb <worktree>`, sidecar, verificación de conservación (todo bloque que sale del canon aparece literal en el sidecar), `exo budget --kb` sin offenders, `exo lint --kb` limpio, «rama ya existe = ya lanzado», y el tope de 10 min con abstención ruidosa (no commitea, deja informe).
- El paso de lanzamiento solo corre si el gate mordió y no existe ya `distill/auto/<slug>`; el resumen final dice en qué rama quedó o que venció el tope.
- `routing.md`: la fila de bitácora pasa a exigir la cabecera única; quitar cualquier ejemplo con etiqueta genérica.

**Notas:** contenido obligatorio, de la spec §«Plugin — `document/SKILL.md`» puntos 1-5 y §«Auto-distill (subagente)» entera. El merge de la rama y la aplicación del sidecar (`exo write append --from .distill/<slug>-moved.md`, borrar el sidecar después) se describen en `auto-distill.md` como pasos del mantenedor o del distill manual, no del subagente.

### Task 4: skill distill — semanal, poda a 60 %, ramas auto y nombres de archive

**Files:**
- Modify: `plugins/exo/skills/distill/SKILL.md`
- Modify: `plugins/exo/skills/distill/chequeos.md`
- Modify: `plugins/exo/skills/distill/rotacion.md`
- Modify: `plugins/exo/skills/distill/consolidacion.md`

**Interfaces:**
- Produces: skill `distill` con el flujo de ramas `distill/auto/*`, poda a 60 % y chequeo nombre/rango

**Tests:** n/a — prosa de skill; la cubren `check-skill-refs.sh` y la revisión.

**Verificación:** `bash scripts/check-skill-refs.sh` → exit 0; `bash scripts/test-docs-vivos.sh` → exit 0.

**Review Focus:**
- La precondición de árbol limpio de `rotacion.md` (y de `SKILL.md` paso 0) pasa a `git -C $KB_ROOT status --porcelain -- log/ archive/log/`; la reversión sigue siendo por ruta.
- Chequeo nuevo en `rotacion.md`: el rango del nombre del archive nuevo coincide con min/max de las fechas de sus cabeceras `## `.
- El disparador de partición usa `git log --since=7.days --fixed-strings --grep='Gate: <ruta>'`; sin `--fixed-strings` no casa nunca.
- Primer paso de cada pasada: revisar ramas `distill/auto/*` (merge + aplicar sidecar, o descartar); no rehacerlas automáticamente.
- Cola `no_air` cruzada con `exo stale` (`age_days`): sin commits en >30 días no es trabajo de la pasada.

**Notas:** contenido obligatorio, de la spec §«Plugin — `distill/SKILL.md`, …» puntos 1-7 (incluido 2b). Promoción de doctrina y barrido por calor obligatorios en cada pasada; si no hay nada, se dice. Notas nuevas solo por partición, con test del título y `## Relations` en ambos sentidos.

### Task 5: exo-recall — aviso de gate pendiente al arrancar

**Files:**
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Test: `plugins/exo/scripts/test-exo-recall.sh`

**Interfaces:**
- Produces: línea en `additionalContext`: `AVISO exo: gate pendiente en <ruta>, sin rama distill/auto/ (corre /distill)`
- Produces: seam `EXO_KB` (raíz de la KB; sin él, `exo config --json | jq -r '.data.kb.path'`)

**Tests:** (casos nuevos en `test-exo-recall.sh`, KB fixture = repo git temporal)
- `gate_pendiente_avisa`: commit con trailer `Gate: projects/a — b (c).md bytes=11000 limit=12500 routed=bitacora` y sin auto-distill ⇒ el `additionalContext` contiene `AVISO exo: gate pendiente en projects/a — b (c).md, sin rama distill/auto/ (corre /distill)`. Falla si la ruta con ` — ` y paréntesis no casa (`--fixed-strings`).
- `gate_resuelto_por_rama_no_avisa`: después, un commit en otra rama con subject `docs(kb): auto-distill a — evicción` que toca esa ruta ⇒ sin aviso. Falla si solo mira la rama actual (debe ser `git log --all`).
- `gate_anterior_a_distill_last_no_avisa`: trailer en un commit anterior al tag `distill/last` ⇒ sin aviso. Sin el tag, la ventana es `--since=7.days`.
- `kb_sin_git_no_rompe`: `EXO_KB` apunta a un directorio sin `.git` ⇒ exit 0, JSON válido, sin aviso.
- El caso feliz existente sin trailers sigue sin aviso y con la salida golden intacta.

**Verificación:** `bash plugins/exo/scripts/test-exo-recall.sh` → todo PASS; `bash scripts/test-plugin.sh` → verde; `SHELLCHECK=<binario 0.11.0> bash scripts/test-shellcheck.sh` → sin hallazgos.

**Review Focus:**
- Ruta con espacios, em dash y paréntesis: `gate_pendiente_avisa`.
- Rama descartada sin merge: el auto-distill ya no aparece en `--all` ⇒ vuelve a avisar (correcto; cubierto por la regla, mencionarlo en el comentario).
- Varias notas con gate: una línea por ruta, sin duplicados aunque haya varios trailers de la misma ruta.
- Latencia: los `git` van bajo `con_timeout` (`_timeout.sh`); timeout ⇒ sin aviso, nunca bloquea (exit 0 siempre).
- `source=compact`: no repetir el aviso (mismo criterio que el bloque de reglas de proyecto).

**Notas:** regla de «pendiente», una sola: para cada ruta con trailer `Gate:` en `distill/last..HEAD`, está pendiente si el commit más reciente con ese trailer es posterior al commit más reciente de `git log --all --fixed-strings --grep='docs(kb): auto-distill' -- <ruta>` (o no hay ninguno). La línea va al `TEXTO`, como el aviso de superpowers, no a `systemMessage`.

### Task 6: wiring — bump del plugin

**Files:**
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`

**Interfaces:**
- Consumes: `plugins/exo/skills/document/auto-distill.md` @Task 3
- Consumes: skill `distill` con el flujo de ramas `distill/auto/*`, poda a 60 % y chequeo nombre/rango @Task 4
- Consumes: `AVISO exo: gate pendiente en <ruta>, sin rama distill/auto/ (corre /distill)` @Task 5

**Tests:** n/a — el gate `test-plugin-bump.sh` lo cubre.

**Verificación:** `bash scripts/test-versiones.sh` → exit 0; `bash scripts/test-plugin.sh` → verde; `cargo test --manifest-path engine/Cargo.toml --locked` → verde (suite entera tras la ola 1).

**Review Focus:**
- `version` `1.8.2` → `1.9.0` en los dos ficheros, idénticas.
- `ENGINE_MIN` no cambia (las skills no dependen del engine nuevo para funcionar; el rechazo de cabecera duplicada es red, no requisito).

**Notas:** el bump del engine a `0.7.0` y el tag van en un PR de release aparte, como #97; no en este plan.
