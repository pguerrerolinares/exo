# Verdict GATE — rama `o-deuda` (ola 3, campaña O)

**Veredicto: MERGED** (consultor fable fresco, 2026-10-05T01:18+02:00).

**Condición de ejecución, no de contenido**: el merge va **por PR**, no por
`merge --no-ff` local, salvo override de Paul registrado en el ledger. Ver §4.1.

Régimen: `.superpowers/fabrica/config.md` §"Ejecución de gates" — cuatro
condiciones. (1) Fresco: este dispatch no participó en ninguna fase de la rama;
el brief traía el deliverable, el criterio y la disposición del orquestador
sobre la reserva 1, que aquí se verifica contra el código, no se hereda.
(2) Verificación primaria propia: §2. (3) Mandato de disenso: §4. (4)
Verdict-artifact en path versionado: este fichero, que commitea el orquestador
antes del `GATE-EXEC`.

Deliverable: worktree `.worktrees/o-deuda`, 7 commits sobre `main` `beecead`
(`789ac19` → `cbdd221`), 12 ficheros, +107/−38. Working tree limpio.
`git merge-tree --write-tree main o-deuda` → árbol sin conflictos (rc 0).

---

## 1. Criterios citados y cumplimiento

Fuente de criterio: propuesta ola 3 §3 O (`propuesta-ola3:docs/superpowers/consultas/2026-10-05-campanas/propuesta.md`)
y `docs/backlog.md` en las líneas que la propuesta cita.

| Ítem | Criterio literal | Cumple | Evidencia |
|---|---|---|---|
| `recall --json` sin `trozos` (backlog:2211) | propuesta: «test que reproduce `no such table: trozos` y pasa a degradar con aviso. `recall.rs:613-621` `primer_trozo` sin `tabla_existe`». backlog: «aplicar a `primer_trozo` (y de paso `fila_notas`, que asume `notas` existe) el mismo patrón de `tabla_existe` + degradación con aviso que `avisos_cobertura_vector` ya usa para `vectores`» | Sí, con la matización de §3 | `recall.rs:616` `if !crate::buscador::tabla_existe(conn, "trozos")? { return Ok(None); }`; test nuevo `recall_json_query_degrada_si_falta_tabla_trozos` (`kb_root_lectura_cli.rs:689`) en verde; repro manual §2.3: binario de `main` → `error: leer primer trozo de kb/z: no such table: trozos`, rc 1; binario de la rama → rc 0 con `warnings` de cobertura en el JSON |
| Prosa «coseno» y «0.35 por defecto» (backlog:1846) | propuesta: «`main.rs:1023` y `tests/buscador.rs:217,347,382` dicen 0.40 en escala L2 sellada (H28)» | Sí, en el alcance que la propuesta fija | `main.rs:1032-1033`: «0.40 por defecto: escala propia de `similitud_desde_l2`, ≈ coseno 0,28 — H28»; `grep -n coseno engine/tests/buscador.rs` → vacío. Lo que el backlog añade (el `--help` de `main.rs:258,304` nombrando `similitud_desde_l2`, y la remisión de `inicia.rs:177`) **no** está en el criterio de la propuesta y sigue abierto; el package lo declara y lo pasa a N. Correcto: el ítem del backlog no se marca cerrado |
| `kb_sintetica.rs`, escala mezclada (backlog:1687) | propuesta: «el doc-comment dice ≈0,28 donde hoy dice ≈0,393». backlog: «declare qué escala mide cada número (propia vs. cosine real) y recalcular el ≈0,28 como ≈0,393» | Sí | `kb_sintetica.rs:29-36`: «0,4747 → cos₀ ≈ 0,4481 → × 0,585 ≈ 0,2622 → sim ≈ 0,393, a 0,030 del medido»; cabecera de la tabla pasa de «max coseno» a «max sim (escala propia)». Aritmética recomputada por mí con `sim = 1 − L2/2` (`buscador.rs:314-316`) y `L2 = sqrt(2−2cos)`: cos₀ = 0,4481, shrunk 0,2622, sim 0,3926, brecha 0,030. Cuadra. También 0,40 ↔ coseno 0,28 cuadra |
| Suite Python de J fuera de CI (backlog:1723) | propuesta: «job de CI verde que la corre. **Toca CI ⇒ va por PR** (lección 1 de la ola 2)». backlog: «un job de CI que instale Python y corra `python3 -m unittest discover` sobre el harness» | Contenido sí; **vía de merge no** (§4.1) | `ci.yml:116-121`, step «Harness de retrieval-heldout (suite Python, offline)» en `static-checks` (`runs-on: ubuntu-latest`), `if: always()`; YAML parsea (`yaml.safe_load`), el `run:` es idéntico al comando del backlog. 82 tests OK en local, rc 0 |
| `MAX_CHARS` no cubre `paquete()→texto` (backlog:1709) | propuesta: «una aserción nueva en `test_gold_j.py`». backlog: «añadir una aserción sobre `paq["texto"]` … para que una regresión en `paquete()` la cace» | Sí, falsable | `test_gold_j.py:64-65` extrae el bloque de cuerpo de `paq["texto"]` y afirma `== jz.MAX_CHARS`. Mutación propia (§2.4): `paquete()` releyendo el fichero crudo en vez de `nota()` → `AssertionError: 17061 != 12000`. La aserción vieja (`len(candidata_grande["cuerpo"])`) no la cazaba, exactamente lo que el backlog denunciaba |
| Restos de macOS (`_timeout.sh`, `test-shellcheck.sh`) | propuesta: «**recon previo**: comprobar que Git Bash en W11 trae `timeout` antes de quitar el fallback. Si no lo trae, se queda y se re-comenta como de Windows» | Rama conservadora correcta; premisa del comentario sin verificar (§4.5) | `git diff main -- plugins/exo/scripts/_timeout.sh` sin líneas de código, solo comentarios; el fallback a perl se conserva. Plugin 1.5.13 → 1.5.14, `plugin-bump-gate` OK |
| «No toca el ranking: ningún cambio en `buscador.rs` fuera de comentarios» | propuesta §3 O | Espíritu sí; letra no (§4.2) | Único cambio: `fn tabla_existe` → `pub(crate) fn tabla_existe` (`buscador.rs:67`). Sin efecto en comportamiento; declarado en el package |

---

## 2. Verificación primaria propia

Todo con cwd en el worktree, sobre `cbdd221`. Los rc son los del comando,
capturados con `$?` sin pipes.

### 2.1 Oráculos

| Comando | rc | Resumen |
|---|---|---|
| `cd engine && cargo test --release --locked` | **0** | 57 bloques `test result`, **624 passed, 0 failed, 12 ignored**; `test recall_json_query_degrada_si_falta_tabla_trozos ... ok` |
| `cargo clippy --all-targets --locked -- -D warnings` | **0** | sin avisos |
| `cargo fmt --check` | **0** | — |
| `python3 -m unittest discover -s evals/retrieval-heldout/harness -p "test_*.py"` | **0** | `Ran 82 tests in 6.285s — OK` |
| `bash scripts/test-plugin.sh` | **0** | `test-plugin: OK — 17/17 suites del plugin en verde` |
| `bash scripts/test-docs-vivos.sh` | **0** | `[OK] test-docs-vivos: README.md/docs/{arquitectura,instalacion}.md sin frases muertas…` |
| `BASE=main bash scripts/plugin-bump-gate.sh` | **0** | `plugin-bump-gate: OK — plugins/exo/ cambia y version también: 1.5.13 → 1.5.14` |
| `bash scripts/test-rutas-personales.sh` | **0** | — |
| `bash scripts/test-versiones.sh` | **0** | — |
| `scripts/test-shellcheck.sh` | no corrido | `shellcheck` no está instalado en esta máquina (confirmado: `which shellcheck` vacío). Coincide con el package. La rama solo toca comentarios de bash; CI lo cubre |

### 2.2 Dónde nacen `trozos` y `vectores` (reserva 1)

`engine/src/schema.rs:17-58`, `crea_schema`: un único `execute_batch` crea
`notas`, `notas_fts`, `aristas`, **`trozos`** (`:42`), `meta` y **`vectores`**
(`:55`, `CREATE VIRTUAL TABLE IF NOT EXISTS vectores USING vec0(...)`). Es
idempotente y es la única DDL del engine (`grep -rn 'CREATE.*TABLE' engine/src`
→ solo `schema.rs`). Por tanto una DB que pasó por `exo index`/`rebuild` tiene
las dos; una que no pasó, ninguna. La premisa del orquestador es cierta.

Cadena del aviso: `buscador.rs:94-104` `avisos_cobertura_vector` →
`if !tabla_existe(conn, "vectores")? { return Ok(vec!["arm vector INERTE: la
tabla `vectores` no existe (la DB nunca pasó por `exo index`/`exo rebuild`)…"]) }`;
`busca_hybrid` lo arrastra (`buscador.rs:703` `let avisos = vector.avisos;`);
`recall_consulta` lo recoge (`recall.rs:551` `let mut avisos = resultado.avisos;`)
y lo emite en `RecallBruto.avisos`.

### 2.3 Repro con el binario

DB hecha a mano (python `sqlite3`): `notas` + `notas_fts`, una nota `kb/z`
«zafiro» con cuerpo vacío (fuerza el camino `fragmento_que_casa → None →
primer_trozo`). Sin `trozos` ni `vectores`.

- Binario del checkout `main` (`engine/target/release/exo`, `beecead`):
  `error: leer primer trozo de kb/z: no such table: trozos: Error code 1: SQL logic error` — **rc 1**. El bug del backlog, reproducido.
- Binario de la rama: **rc 0**. stderr: `aviso: arm vector INERTE: la tabla
  `vectores` no existe…`. stdout JSON: `"notes":[{"permalink":"kb/z","score":0.6,"snippet":null,…}]`,
  `"warnings":["arm vector INERTE: la tabla `vectores` no existe (la DB nunca
  pasó por `exo index`/`exo rebuild`). …"]`. La nota sale, el snippet degrada a
  `null`, el aviso de cobertura viaja en el envelope.

Caso agujero que el orquestador no probó — `vectores` presente, `trozos`
ausente (tabla plana `vectores` creada a mano): **rc 1**,
`error: contar filas de trozos: no such table: trozos`. Muere en
`avisos_cobertura_vector` (`buscador.rs:106` `SELECT count(*) FROM trozos`),
**antes** de llegar a `primer_trozo`. Consecuencia: no existe ningún camino
alcanzable hasta el nuevo `Ok(None)` sin que el aviso de `vectores` ya se haya
emitido. La disposición se sostiene, y por una razón más fuerte que la que dio
el orquestador.

### 2.4 Mutación de `paquete()`

Copia del harness en scratchpad; en `paquete()` tras `cands = [nota(...)]`,
`c["cuerpo"] = open(rutas[...]).read()` (bypass del truncado de `nota()`).
Resultado: `FAIL: test_paquete_y_parsea — AssertionError: 17061 != 12000`.
La aserción nueva es falsable y caza la regresión que el backlog describía.

---

## 3. Reserva 1 — adjudicación

Backlog:2211 pedía «degradación con aviso». El fix de `primer_trozo` degrada
**sin aviso propio**. Verificado en el código y en el binario (§2.2-2.3): en
toda DB real sin `trozos` tampoco hay `vectores` (nacen en el mismo
`execute_batch`), y entonces `busca_hybrid` ya ha emitido el aviso de cobertura
que `recall` propaga al JSON. Un aviso adicional «sin snippet porque falta
`trozos`» sería redundante con el que ya dice «la DB nunca pasó por `exo
index`». **Se acepta la disposición.**

Dos matices que el package no recoge y van al backlog, no bloquean:

- `avisos_cobertura_vector` cuenta `trozos` sin `tabla_existe` (`buscador.rs:106`).
  Con `vectores` presente y `trozos` ausente, `busca_hybrid` revienta con error
  duro. Es preexistente en `main`, no lo introduce esta rama, y el schema es
  ajeno/artificial (ninguna ruta de exo lo produce). Residuo para N, misma
  clase que backlog:2211.
- «`fila_notas` lo cubre el test existente `db_sin_tabla_meta`» es impreciso:
  esa fixture **crea** `notas` (`kb_root_lectura_cli.rs:249`), así que ningún
  test ejercita `fila_notas` con `notas` ausente. El backlog lo pedía «de paso»
  y una DB sin `notas` no es un índice de exo por ninguna definición
  (`enriquece_rutas` en `buscador.rs:138` también la asume). No bloquea, pero el
  package no debería presentarlo como cubierto.

---

## 4. Qué busqué para objetar

Busqué activamente motivos de RECHAZADA. Lo que encontré, en orden de peso:

### 4.1 La vía de merge contradice el criterio escrito (lo más serio; condición)

La propuesta lo dice dos veces: §2 tabla, fila O: «ninguno; **el job de CI va
por PR**»; §3 O: «**Toca CI ⇒ va por PR** (lección 1 de la ola 2)». El package
§"Instrucción de merge" prescribe `git merge --no-ff o-deuda` local y argumenta
«el job tocado (`static-checks`) corre en ubuntu y su comando se ejecutó aquí,
así que la lección 1 no aplica». El razonamiento es plausible (yo mismo
verifiqué que el YAML parsea y que el comando corre en local), pero es el
orquestador reinterpretando una instrucción literal de la propuesta. Config
§"Overrides de Paul": «Las prohibiciones y caps … ceden SOLO ante pedido directo
de Paul en sesión. Todo override se registra en el ledger ANTES de ejecutarlo».
Un consultor no puede concederlo. Por eso el veredicto es MERGED sobre el
contenido con **condición de ejecución: por PR**, como dice la propuesta, salvo
override de Paul en el ledger. Con branch protection activa en `main` (config,
bloque 2026-09-19: «12 required checks — un merge ya no pasa con CI rojo») el PR
es además el único camino que ejecuta el step nuevo antes de que toque `main`.

### 4.2 `buscador.rs` cambia algo que no es un comentario

Propuesta §3 O: «ningún cambio en `buscador.rs` fuera de comentarios». El diff
cambia la visibilidad de `tabla_existe` (`fn` → `pub(crate) fn`). No es
comentario. El espíritu de la frase es «no toca el ranking», y una visibilidad
no cambia ni una línea ejecutable; la alternativa (duplicar la consulta a
`sqlite_master` en `recall.rs`) sería peor ingeniería. El package lo declara
en su primera línea. Lo registro como desviación declarada y sin efecto, no
como motivo de rechazo.

### 4.3 ¿El test nuevo prueba lo que dice?

Comprobé que la ruta del test pasa de verdad por `primer_trozo`: cuerpo vacío en
`notas_fts` ⇒ `fragmento_que_casa` no tiene nada que recortar ⇒ cae a
`primer_trozo`. El repro con el binario de `main` (§2.3) muere exactamente ahí
con el mensaje del backlog. El package dice que la review lo vio fallar con el
fix revertido; yo lo vi fallar con el binario de `main`, que es equivalente.

### 4.4 ¿La aritmética de H28 está bien o el brief del orquestador (0,28 ↔ 0,393 invertidos) contaminó el código?

Recomputé desde la fórmula real (`similitud_desde_l2 = 1 − L2/2`,
`buscador.rs:314-316`), no desde el backlog: cuadra a tres decimales (§1). El
executor siguió el backlog, no el brief invertido. Bien resuelto.

### 4.5 El comentario nuevo de `_timeout.sh` afirma una premisa que nadie verificó

El criterio exigía recon en W11 (¿Git Bash trae `timeout`?). No se hizo (no se
puede desde aquí: es PAUL-STEP de facto). El executor tomó la rama conservadora
—conservar el fallback—, que es la correcta ante incertidumbre. Pero el
comentario nuevo dice «en un entorno sin él (BSD, Git Bash sin coreutils)»:
«Git Bash sin coreutils» describe una configuración concreta cuya existencia o
frecuencia nadie ha comprobado. No afirmo ni lo uno ni lo otro: no lo he medido
en W11 y no tengo fuente que lo establezca en ningún sentido. Señalo que el
comentario fija una premisa que el recon exigido nunca estableció. Nit
documental; va con los de N.

### 4.6 Nits que ya estaban escalados y confirmo

`kb_sintetica.rs:15-17` «similitud coseno esperada ~0» frente al umbral 0.40
(escala propia; coseno 0 ≡ sim ≈ 0,293, sigue por debajo de 0,40, así que la
conclusión es válida aunque mezcle escalas). `tests/buscador.rs:383` «sin
importar el signo real de la similitud»: en escala propia la similitud de
vectores unitarios no baja de 0,0, el «signo» ya no aplica. Ambos declarados en
el package como escalados.

### 4.7 Lo que no encontré

- Cambio de ranking: ninguno (`buscador.rs` sin código ejecutable tocado;
  624 tests en verde incluidos los de desempate y umbral).
- Conflictos con `q-w11`/`p-upstream`: `merge-tree` limpio contra `main`
  (que ya contiene `p-upstream`); el package dice que con `q-w11` solo
  coincide el bump de versión.
- Tocar `docs/backlog.md`: la rama no lo toca (`git diff main --stat -- docs/`
  vacío). Coherente con «los cierres los absorbe N».
- Residuo en el working tree: limpio.
- Scripts del plugin con cambio de comportamiento: ninguno; 17/17 suites.

---

## 5. Resumen

Los seis ítems cumplen el criterio que la propuesta les asigna, con evidencia
propia y dos falsaciones (test de `recall` contra el binario de `main`;
mutación de `paquete()`). La reserva 1 se sostiene con una razón más fuerte que
la del orquestador. Las desviaciones de letra (§4.2) están declaradas y no
tienen efecto. La única discrepancia con fuente citable es la vía de merge
(§4.1), y es de ejecución, no de contenido: **MERGED, por PR**.

Residuos para la campaña N (backlog): `avisos_cobertura_vector` sin
`tabla_existe` sobre `trozos`; `fila_notas` sin test de `notas` ausente;
`--help` de `main.rs:258,304` e `inicia.rs:177` (ya declarados); nits de
prosa §4.5-4.6.

Líneas para el package (la primera con la forma exacta de config §Mecánica; la
condición va en la línea siguiente para no romper la forma canónica):

```
GATE: MERGED (consultor fable, 2026-10-05T01:18+02:00, verdict=docs/superpowers/consultas/2026-10-05-campanas/verdict-gate-o.md)
Condición de ejecución (verdict §4.1): merge por PR, como manda la propuesta §2 fila O y §3 O; un merge local exige override de Paul registrado en el ledger.
```
