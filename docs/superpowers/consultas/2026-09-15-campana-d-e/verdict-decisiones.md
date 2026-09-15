# Verdict — decisiones abiertas de las campañas D y E

- **Fecha**: 2026-09-15
- **Consultor**: Fable, fresco (no participó en ninguna fase de D ni de E; brifeado con deliverable + criterio, sin el razonamiento del orquestador)
- **Alcance**: las 5 decisiones del brief — (1) gate de rutas personales, (2) `exo stale` con score negativo, (3) D-4 fallback sin `[kb] name`, (4) D-3 exit code de `exo rotate`, (5) duplicación del bucle de descubrimiento de bash versionado. Nada más: no es un gate de merge de las ramas.
- **Régimen**: `.superpowers/fabrica/config.md` §Ejecución de gates (4 condiciones: fresco, verificación primaria propia, mandato de disenso, verdict-artifact en path versionado). Este fichero es el artifact; lo commitea el orquestador.
- **Ramas juzgadas**: E `e-hooks-honestos-y-ci` @ `55ef5e1` · D `d-cutover-kbx` @ `812ef22` (apila E). Los dos worktrees quedaron sin tocar (`git status --porcelain` vacío al terminar).

## Fuentes leídas (no solo citadas)

- `.superpowers/fabrica/config.md` (bloque ACTUALIZACIÓN 2026-09-14, §Ejecución de gates) · `pendiente-paul.md` · `packages/campana-{e,d}-package.md` · `ledger.md` (filas E T8, review E T8 :1199-1207).
- Planes: `docs/superpowers/plans/2026-09-14-campana-d-cutover-kbx.md` (D-3/D-4 :102-143, Global Constraints, Residuo declarado :3195-3218) · `2026-09-14-campana-e-hooks-honestos-y-ci.md` (Global Constraints :109-164, Task 8 :1360-1545) · pre-registro `2026-09-14-campana-d-preregistro-paridad-rotate-stale.md` (completo, incl. registros y addenda).
- Backlog (rama E y D): ítem «"exo genérico" sigue siendo el plugin de Paul» (:268-305), «Rutas personales y `hooks.json` sin validar en CI» (E :609-638), cierre de `test-git-c-bash.sh:74-75` (:1615-1628).
- Código exo (rama D): `engine/src/main.rs` (`main` :428-466, `resuelve_kb` :512-522, `write_new_cmd` :789-823, `stale_cmd` :1318-1345, `rotate_cmd` :1472-1556) · `envelope.rs` · `config.rs` :1-105 · `lib.rs::nombre_kb` :112-118 · `walker.rs` :1-53 · `rotacion.rs::aplica` :455-574 · `obsolescencia.rs` (:1-65, :210-240, :451-464) · `tests/rotar_cli.rs` · `tests/common/mod.rs::render_config` :36-49.
- Scripts (rama E): `scripts/test-rutas-personales.sh`, `scripts/test-shellcheck.sh`, `plugins/exo/scripts/_truncate-payload.sh`, `git-add-all-guard.sh:44-52`, `.github/workflows/ci.yml` (job `lint`).
- Consumidores: `plugins/exo/skills/distill/{SKILL.md,rotacion.md}`, `docs/arquitectura.md` (§3.7-3.8).
- kbx `fe46443` vía `git -C ~/Documentos/proyectos/kbx show fe46443:<path>` (sin checkout): `cmd/kbx/rotate.go`, `internal/rotate/apply.go:140-175`, `internal/stale/stale.go` (:1-60, :80-90, :204-210), `internal/stale/stale_test.go:329-346`, `cmd/kbx/stale.go:24-38`, `docs/superpowers/specs/2026-07-11-m4-budget-stale-spec.md:60-130`, `.superpowers/fabrica/PENDIENTE-PAUL-m4-stale-formula.md:100-110`.

## Comandos de verificación corridos (literal → resultado)

Binarios: `EXO=.worktrees/d-cutover-kbx/engine/target/release/exo` (D+E, `812ef22`), `KBX=/tmp/campana-d/kbx` (fe46443). KBs desechables en el scratchpad de sesión; `kb-base` e `index.db` de `/tmp/campana-d/` solo en lectura. Sin `cargo build/test`.

1. **stale con `--now` anterior a commits** (kb-base RO, `NOW=2026-09-01T00:00:00Z`, `--db /tmp/campana-d/index.db`):
   `"$EXO" stale … --now $NOW --json | jq '.data.notes | map(select(.age_days<0))'` → **35 notas** con edad negativa, `min_age=-14`, `min_score=-9.29`, ejemplo `log/agent-develop-bitacora.md age_days=-2 score=-0.25 tier=log degree=15`. exit 0, stderr vacío.
   Mismo comando con `$KBX` → **idéntico** (35, -14, -9.29, mismo ejemplo). exit 0, stderr 0 bytes (kbx tampoco avisa).
   `diff <(jq -S '.data.notes|map({path,age_days,score:(.score+0)})' exo) <(… kbx)` → vacío: **paridad exacta incluidos los negativos**.
2. **rotate con fallos parciales** (KB con `log/buena.md` tier log grande, `log/rota.md` frontmatter sin cerrar, `log/ilegible.md` chmod 000; `--hot-bytes 8000 --apply --json`):
   exo → **exit 1**, envelope emitido con `rotations` de 1 nota aplicada, stderr: `rotate: log/ilegible.md: Permission denied`, `rotate: log/rota.md: … no se encontró el cierre`, `error: 2 nota(s) fallaron durante la barrida`.
   kbx (clon fresco idéntico) → **exit 2**, envelope con 1 rotación, dos líneas `kbx rotate: …` equivalentes. Misma semántica de barrida, solo difiere el código.
3. **D-4 fallback**: `EXO_CONFIG=/nonexistent "$EXO" rotate --kb <kb> --hot-bytes 8000 --apply` → **exit 0**, stderr `aviso: rotate usa prefijo 'kb' — sin [kb] name: no encuentro la config…`, y en `archive/log/*.md`: `permalink: 'kb/archive/log/buena-2026-01-01_2026-04-01'`.
   Con `EXO_CONFIG` apuntando a una config válida con `name = "otra-kb"` y `path = "/otra/ruta"` (≠ `--kb`) → exit 0, **stderr 0 bytes**, `permalink: 'otra-kb/archive/log/…'` (usa el nombre de la config para una KB que no es la de la config, sin aviso — mismo patrón que `write new`, `main.rs:794-797`).
   Config TOML válida pero sin `name` → `config incompleta o mal tipada … missing field 'name'` ⇒ cae al mismo fallback `kb`. Conclusión: "sin `[kb] name`" ≡ "sin config válida" (`config.rs:26-33`, `name: String` obligatorio).
4. **Gate de rutas por forma**: `printf … | grep -En "$PATRON"` con el `PATRON` literal del script → casan `/home/runner/work/exo`, `/Users/Shared/x`, `C:/Users/Public/y`, `C:\Users\Default\z`, `/home/user/.local/bin`; NO casan `$HOME/.local/bin`, `/opt/proyectos/x`, `/home/$USER/x`.
   `git grep -n -E '/home/runner|/Users/Shared|C:[\\/]Users[\\/]Public'` en el árbol completo de D (incluye `docs/` y `evals/`, que el gate excluye) → **0 líneas**.
5. **Precedente de helper**: `grep -rn "_truncate-payload.sh"` → sourceado en `git-add-all-guard.sh:49` y `verify-before-commit.sh` (2 consumidores, extraído por "llevaban este bloque copiado línea a línea", `_truncate-payload.sh:2-4`). `diff` de `scripts/test-rutas-personales.sh` entre E y D → idéntico. Los dos gates hoy: bucle de descubrimiento byte-idéntico (`test-shellcheck.sh:51-64` vs `test-rutas-personales.sh:24-37`).

---

## Decisión 1 — Gate de rutas personales: `/home/runner`, `/Users/Shared`, `C:/Users/Public`

**Decisión: (a) dejarlo tal cual.** Sin allowlist de directorios de sistema.

**Qué cambia exactamente**: nada en código ni en CI. Opcional (coste 1 línea, no bloquea): en el ítem del backlog «Rutas personales y `hooks.json` sin validar en CI» (E `docs/backlog.md:609-638`), añadir una frase: *"Falso positivo conocido por diseño: el gate caza por forma, así que `/home/runner`, `/Users/Shared`, `C:/Users/Public` disparan; el arreglo es el que imprime el propio gate (variable: `$HOME`, `$RUNNER_TEMP`, `$GITHUB_WORKSPACE`), no una allowlist."*

**Cita**:
- Plan E `:1369-1371`: *"**Decisión de Paul, ya tomada, no reabrir**: el gate mira SOLO rutas (`/home/<user>`, `C:\Users\<user>`, `/Users/<user>`), no nombres propios."*
- Plan E `:157-158` (Fuera de alcance): *"detección de nombres propios en el gate de rutas personales (solo rutas, decisión de Paul)"*.
- Plan E `:1489-1491`: *"sustituir solo el nombre de usuario por uno genérico NO basta — el gate mira la FORMA `/home/<lo-que-sea>`, no si el nombre es real. Por eso la ruta sale por completo de `/home`."* — el plan ya asume y acepta que la forma manda sobre el nombre.
- `scripts/test-rutas-personales.sh:66`: *"Arreglo: sustituye por una ruta genérica fuera de /home, /Users o C:\Users (p.ej. /opt/proyectos/...) o por una variable."*

**Evidencia propia**: comando 4. Cero apariciones en todo el repo (ni siquiera en `docs/`/`evals/`, que el gate ni mira). El único sitio plausible donde alguien escribiría `/home/runner` es un script de CI, y ahí la forma correcta es `$HOME`/`$GITHUB_WORKSPACE` — el gate fallando ahí es señal útil, no ruido.

**Trade-offs**:
- (a) Pro: fiel a la decisión literal ("solo rutas"; una lista de nombres que NO son personales sigue siendo lógica de nombres, por la puerta de atrás). Cero mantenimiento. Contra: un día un comentario que mencione `/home/runner` pone CI en rojo y hay que reescribirlo — coste 1 minuto, con el arreglo ya impreso.
- (b) Pro: 3 líneas, evita ese rojo. Contra: introduce una lista que hay que mantener (¿`/Users/Guest`? ¿`C:/Users/Default`? ¿`/home/ubuntu` de un VPS?) y dos falsaciones más; cambia el alcance de una decisión cerrada por un beneficio hoy nulo.

**Qué busqué para objetar**: el argumento más fuerte a favor de (b) es que un falso positivo en CI se decide "bajo presión" y a deshora (Paul fuera del critical path). No gana porque (i) hoy hay 0 casos en 46 scripts, (ii) el gate solo mira bash versionado fuera de `docs/`/`evals/` — el lugar donde uno escribe prosa sobre runners está excluido —, y (iii) el mensaje de error ya dicta el arreglo correcto, que además es mejor código que la ruta literal. También comprobé si `C:\Users\Default` o `/home/user` (rutas "genéricas" que alguien podría usar como ejemplo) disparan: sí, y el plan E :1489-1491 ya lo declara como deseado.

**Rama afectada**: E (solo la línea opcional de backlog) — o ninguna.

---

## Decisión 2 — `exo stale` con score negativo (`--now` anterior al último commit)

**Decisión: (a) declarar y dejar.** Ni clamp ni rechazo de `--now`.

Respuesta a la pregunta del brief: **la "fórmula tal cual" es el cálculo del score sobre `age_days`, y `age_days` es un input del pre-registro con su propia definición cerrada** — `floor((now − commit)/24h)` (spec M4 kbx :63-66; `stale.go:207-209`; `obsolescencia.rs:214-216`). El axioma 5 **no** afirma que el score sea ≥ 0 para cualquier `--now`: kbx lo acota explícitamente al dominio `ageDays >= 0` (`stale.go:82-83`: *"Finite and >= 0 for any ageDays >= 0 and degree >= 0 (spec axiom 5)"*) y su test `TestScore_Axiom5_ShapeAndOrdering` (`stale_test.go:332-346`) solo recorre `age ∈ {0,1,13,1000,36500}` — exactamente los mismos valores que `axioma_5_forma_finita_no_negativa_y_dos_decimales` (`obsolescencia.rs:452-464`). No hay contradicción con el axioma: hay un input fuera de dominio (un reloj anterior al commit), que en uso real (`--now` ausente ⇒ reloj de pared, `main.rs:1332-1335`) solo ocurre con commits fechados en el futuro. Un score negativo ahí manda la nota al fondo del ranking — que es donde debe estar una nota "más fresca que ahora".

- **(b) clamp de `age_days` a 0 SÍ contradice** la decisión — no la fórmula en sí, sino el gate pre-registrado que la certifica: el criterio de §Stale exige que `age_days` y `score` *"coincidan exactamente"* con kbx (pre-registro :205-208), y la corrida firmada PASA ya contenía negativos (mín −0.36, registro :479). Con clamp, esa misma corrida habría dado NO PASA o habría exigido declarar una divergencia nueva **después** de ver el número — justo lo que el pre-registro prohíbe (:95-97: *"Un NO PASA no se racionaliza"*).
- **(c) rechazar/avisar** no contradice nada, pero añade superficie (un aviso por stderr que ningún consumidor lee: `distill` invoca `stale --json` sin `--now`, `SKILL.md:65`) para un caso que solo se da en gates y tests, donde `--now` es deliberado.

**Qué cambia exactamente**: código, nada. Documentación mínima (coste 5 min, rama D): en `engine/src/obsolescencia.rs`, sobre `axioma_5_forma_finita_no_negativa_y_dos_decimales` (:451), un comentario de 3 líneas: *"Dominio del axioma: `edad ≥ 0`, igual que kbx (`stale.go:82-83`, `TestScore_Axiom5`). Con `--now` anterior al commit `age_days` es negativo y el score también, en los dos binarios (paridad medida 2026-09-15: 35 notas, mín −9.29 con `--now 2026-09-01`). No se clampea: `age_days` es un campo gateado exactamente contra kbx."* El addendum (d) del pre-registro (:508-522) ya lo declara; queda cerrado con este verdict, sin editar el registro.

**Cita**:
- `config.md:58`: *"`stale` con la fórmula de kbx tal cual (1.5/1.0/0.5/NOTIER 0.5, degree 0.2)"*.
- PENDIENTE-PAUL-m4 (kbx) :106-109: *"**FIRMADA tal cual la propuesta** … Sin cambios; el golden mergeado en main es el canónico."*
- Pre-registro :217-224: *"La fórmula y los pesos NO son una decisión de esta campaña … No se reabre."* y :205-208 (criterio: `age_days` … `score` *"coinciden exactamente"*).
- `stale.go:82-83` (kbx): *"Finite and >= 0 for any ageDays >= 0 and degree >= 0 (spec axiom 5)"*.

**Evidencia propia**: comando 1 — con `--now 2026-09-01` sobre `kb-base`, 35 notas negativas, mín −9.29, **byte-idéntico entre exo y kbx**, exit 0 y stderr vacío en los dos. kbx tampoco avisa ni rechaza (`cmd/kbx/stale.go:32-38` solo valida el parseo RFC3339).

**Trade-offs**: (a) mantiene paridad y el gate firmado; deja un output "raro" para un input raro. (b) output más "bonito" a costa de romper la comparación exacta con kbx en un campo gateado, y de reabrir una firma. (c) coste bajo pero valor nulo hoy.

**Qué busqué para objetar**: el argumento más fuerte contra (a) es el del reviewer: el test del axioma 5 "solo prueba edades ≥ 0", así que el axioma queda sin cubrir en el caso real. Comprobé si el axioma, tal como está escrito en la spec M4 (:100-103: *"Scores are finite, ≥ 0, formatted at FIXED 2-decimal precision"*), es incondicional — la letra sí lo es, pero la implementación de referencia que Paul firmó "tal cual" lo acota en su comentario y en su test al dominio `age ≥ 0`, y el golden que Paul declaró canónico se generó con ese código. Firmar el golden es firmar esa lectura. Segundo intento: ¿un commit con fecha futura en producción (clock skew de otra máquina) rompería algo aguas abajo? No: `distill` lee `stale --json` como ranking, no gatea por signo; un negativo va al final. Tercero: ¿el `div_euclid` de `edad_en_dias` (:215) diverge del `math.Floor` de Go en negativos? Ambos redondean hacia −∞; verificado en la paridad (`age_days=-2` en ambos para el mismo commit).

**Rama afectada**: D (comentario en `obsolescencia.rs`; opcional).

---

## Decisión 3 — D-4: fallback sin `[kb] name` y valor base de D-4

**Decisión: (b) abortar `--apply` sin `[kb] name` resoluble; el dry-run sigue funcionando sin config.** Y **se confirma D-4 base = `exo::nombre_kb()`** (no el literal `"wisdom-paul"`).

**Qué cambia exactamente** (rama D, coste ~30 min + tests):
1. `engine/src/main.rs:1485-1488` — sustituir el `unwrap_or_else` por:
   ```rust
   // D-4: el prefijo del `permalink` del archivo es `[kb] name`. Solo
   // `--apply` lo escribe; el dry-run no necesita config (sirve sobre una
   // KB ajena sin `~/.exo`). Sin nombre resoluble, `--apply` falla ANTES de
   // tocar disco: "sin defaults inventados" (config.rs), igual que `write new`.
   let nombre_kb = if args.apply {
       exo::nombre_kb()
           .context("rotate --apply necesita `[kb] name` en la config para el permalink del archivo")?
   } else {
       String::new()
   };
   ```
   `aplica` no cambia: solo usa `nombre_kb` en la rama `escribe` (`rotacion.rs:553-554`).
2. `engine/tests/rotar_cli.rs`:
   - `apply_escribe_de_verdad` (:68-84): pasa a escribir una config válida en el tempdir (`mod common;` + `common::render_config(dir.path(), "prueba", &dir.path().join("x.db"))` → `dir/config.toml`) y fijarla en `EXO_CONFIG`; añade la aserción `permalink: 'prueba/archive/log/` en el archivo.
   - `fallback_d4_sin_kb_name_usa_prefijo_kb_y_avisa_por_stderr` (:156-193) se **sustituye** por `apply_sin_kb_name_falla_antes_de_escribir`: mismo montaje con `cfg_inexistente`, `--apply` ⇒ `!status.success()`, `code() == Some(1)`, stderr contiene `[kb] name`, `archive/` **no existe**, y `log/p-bitacora.md` es byte-idéntico al original.
   - Los tests de dry-run con `cfg_inexistente` (`dry_run_no_toca_disco…`, `ignora_notas…`, `hot_bytes…`, `un_directorio…`, `sin_directorio_log…`) no cambian: siguen probando que el dry-run no exige config.
   - Actualizar el doc-comment de `cfg_inexistente` (:23-27), que hoy describe el fallback.
3. `plugins/exo/skills/distill/rotacion.md` (junto a :13): una frase — *"`--apply` exige `[kb] name` en la config; sin ella sale exit 1 sin tocar disco."*
4. Pre-registro: nada (divergencia 6 ya declara `nombre_kb()`; kbx nunca falla porque hardcodea).
5. Oráculo: `cargo test --release --test rotar_cli` (7 tests, mismo conteo) + `cargo fmt --check` + `cargo clippy --all-targets --locked -- -D warnings`. Compilar como hizo el orquestador (`CARGO_BUILD_JOBS=2`, target ya sembrado en `d-cutover-kbx/engine/target`), no en frío.

**Residuo declarado, no se toca**: con `--kb` apuntando a una KB distinta de `[kb] path`, `rotate --apply` escribe el nombre de la config sin aviso. Es el patrón vigente de `write new` (`main.rs:794-797`) y de toda la precedencia `flag > env > config`; corregirlo (comparar `--kb` con `[kb] path` y avisar) sería una decisión transversal a `write`/`rotate`/`init`, no de D. Queda como línea en el ítem «"exo genérico"» del backlog si Paul quiere abrirla.

**Cita**:
- `engine/src/config.rs:10-11`: *"**Sin defaults inventados**: un default silencioso es la clase de fallo que este proyecto existe para no volver a tener."* — el prefijo `kb` es un default inventado (avisado, pero inventado y persistido en disco).
- `engine/src/lib.rs:112-117`: *"Nombre de proyecto de la KB (prefijo de permalink), EXPLÍCITO en config. Cierra el disenso del gate M4: `write new` lo derivaba de `kb.file_name()`, contra lo que decía la spec §3.1. Coincidían por suerte."* — esto **descarta la opción (c)** (derivar del nombre del directorio): es exactamente la convención que exo ya abandonó a propósito.
- `main.rs:794-797` (`write new`): *"El nombre del proyecto sale de `[kb] name` de la config, EXPLÍCITO … el día que no, reventaba en silencio."* — `write new` no tiene fallback: `exo::nombre_kb()?`. `rotate --apply` es el otro comando que escribe permalinks; debe comportarse igual.
- Plan D :133 (D-4, contra de la opción verbatim): *"exo sirve **cualquier** KB … escribiría un permalink falso en cuanto alguien use exo sobre una KB que no se llame `wisdom-paul`"* — el mismo argumento condena `kb/…`: también es un permalink falso, solo que para todas las KBs.
- Plan D :134 (D-4 b): *"Usar `[kb] name` de la config, el mismo mecanismo que ya usan `write new`/`write append`"* — confirmación del valor base.
- `walker.rs:9-10`: *"`archive/` SE incluye (§6.2 regla 4)"* — el archivo con `kb/…` entra en el índice bajo un namespace que no es el de la KB.

**Evidencia propia**: comando 3. Sin config: exit 0 + archivo persistido con `permalink: 'kb/archive/log/…'`. Con config de otra KB: exit 0, sin aviso, `permalink: 'otra-kb/…'`. Config sin `name` ⇒ error de parseo ⇒ mismo fallback. Es decir: el único caso en que el fallback actúa es "no hay config válida", y en ese caso ningún otro comando que escriba (`write new`, `write append --create`) funciona — `rotate --apply` es la excepción sin justificación escrita.

**Trade-offs**:
- (b) Pro: coherente con `write` y con la doctrina de `config.rs`; nunca metadata falsa en disco; el dry-run (lo que un tercero probaría primero) sigue sin exigir config; menos código que hoy. Contra: una KB sin config no puede rotar con `--apply` — pero tampoco puede hacer nada más que escriba; el mensaje de error ya nombra `exo init --kb <ruta> --name <nombre>` (`config.rs:84-86`).
- (a) Pro: fijado por test, cero trabajo. Contra: persiste un permalink inventado; el aviso va a stderr, que `distill` no lee (`SKILL.md:55` solo mira `data.rotations`).
- (c) Descartada por cita (`lib.rs:114-117`).

**Qué busqué para objetar**: el argumento más fuerte a favor de (a) es "el permalink del archivo es cosmético: el indexer puede derivar el permalink de la ruta". Miré `walker.rs` e `indexer` — `archive/` se indexa y el permalink es la PK del índice (brief + `write append` resuelve permalink→ruta contra el índice, `main.rs:832`); un `kb/archive/log/x` convive con `wisdom-paul/log/x` en la misma tabla. No es cosmético. Segundo: "abortar rompe la paridad con kbx" — no: kbx nunca puede fallar por esto porque hardcodea el nombre; la divergencia 6 ya está declarada y este cambio solo afecta al caso sin config, que el gate (con `~/.exo/config.toml` presente) no ejercita. Tercero: "(b) es over-engineering" — es quitar un `unwrap_or_else` y su string; hay menos líneas después que antes.

**Rama afectada**: D.

---

## Decisión 4 — D-3: exit code de `exo rotate` con fallos parciales

**Decisión: se confirma exit 1.** Nada cambia.

**Qué cambia exactamente**: nada. (El registro del pre-registro ya declara la divergencia 1; `rotacion.md:13-22` ya instruye al consumidor sobre el exit 1 parcial.)

**Cita**:
- `engine/src/main.rs:428-466`: `main` solo conoce tres salidas: `Rechazo`/`GateFallido` ⇒ `exit(3)` con `eprintln!("rechazado: …")`; cualquier otro `Err` ⇒ `exit(1)`. Comentario :439-441: *"Un gate rechazado NO es un error del sistema: es una decisión que se le devuelve al llamador"*. Un `Permission denied` no es una decisión devuelta al llamador.
- Plan E :127-131 (Global Constraints, vigente para D): *"Códigos de salida (`engine/src/main.rs:387-412`): `3` si el error es un `Rechazo`/`GateFallido`; `1` cualquier otro error; `0` éxito; `2` error de uso de clap."*
- Plan D :117-118: *"Recomendación: a. … `rotate_cmd` termina con `anyhow::bail!` tras imprimir cada fallo por stderr, que en `main()` cae en la rama genérica (exit 1)."*
- Pre-registro :105-117 (divergencia 1): *"al final hace `bail!` con la cuenta de fallidas, que en exo sale **1** (ningún fallo de `rotate` es un `GateFallido` …)"*.
- Spec M4 de kbx :110-111: *"Exit codes: 0 on success …; 2 IO/usage."* — kbx nunca separó IO de uso; exo sí (2 = clap). Portar el "2" importaría una tabla de códigos que exo no tiene.

**Evidencia propia**: comando 2 — exo: envelope con la rotación aplicada **antes** del `bail!`, dos líneas `rotate: <ruta>: <error>` en stderr, exit 1. kbx: mismo comportamiento de barrida, exit 2. El consumidor (`rotacion.md:13-22`) ya documenta el exit 1 como "parcial, revisa stderr, no repitas `--apply` a ciegas".

**Trade-offs**: exit 1 mantiene la tabla de exo intacta (0/1/2/3 con significados disjuntos) y diverge de kbx en un número que ningún consumidor del plugin compara (tras D, ninguno invoca `kbx`). Exit 3 daría "uniformidad" con `budget`/`lint`/`ratchet` a costa de mezclar "no pude leer un fichero" con "la KB está mal" — y `main` imprimiría `rechazado:` para un `EACCES`, que es mentir.

**Qué busqué para objetar**: el mejor argumento a favor de 3 es que `distill` corre `rotate --apply` desatendido y "3 = algo requiere tu atención" es más visible. Comprobé cómo lo consume `distill`: no gatea por 3 vs 1, lee `data.rotations` y stderr (`rotacion.md:13-22`). También busqué si algún otro comando de exo usa 1 para un fallo parcial de barrida — `stale` y `targets` fallan con el error crudo de git en exit 1 (`SKILL.md:31`); `rotate` es consistente con ellos. Sin razón para discrepar del plan.

**Rama afectada**: ninguna.

---

## Decisión 5 — Duplicación del bucle de descubrimiento de bash versionado

**Decisión: (b) helper sourceado `scripts/_bash-versionado.sh`.**

**Qué cambia exactamente** (rama E, coste ~20 min + rerun de 2 gates):
1. Crear `scripts/_bash-versionado.sh` (sin bit de ejecución no hace falta; es `.sh`, entra solo en el índice de ambos gates):
   ```bash
   #!/usr/bin/env bash
   # Helper COMPARTIDO: descubre el bash versionado (índice de git: *.sh más
   # ejecutables sin extensión con shebang sh/bash; excluye evals/ y docs/).
   # Lo usan test-shellcheck.sh y test-rutas-personales.sh, que llevaban este
   # bucle copiado línea a línea — si divergen, los dos gates dejan de mirar
   # lo mismo sin avisar. Mismo patrón que plugins/exo/scripts/_truncate-payload.sh.
   #
   # Uso:  . "$(dirname "$0")/_bash-versionado.sh" && bash_versionado
   # Asigna el array global `ficheros` en vez de imprimir (un array no pasa
   # por $(...)). Exige cwd = raíz del repo: los llamadores ya hacen el cd.
   # shellcheck disable=SC2034 # ficheros es la salida: la lee el script que hace source
   bash_versionado() {
     ficheros=()
     while read -r modo blob _etapa ruta; do
       case "$ruta" in evals/*|docs/*) continue ;; esac
       case "$ruta" in
         *.sh) ficheros+=("$ruta") ;;
         *.*) : ;;
         *)
           [ "$modo" = "100755" ] || continue
           if git cat-file -p "$blob" | head -n 1 | grep -Eq '^#!.*[/ ](ba)?sh([[:space:]]|$)'; then
             ficheros+=("$ruta")
           fi
           ;;
       esac
     done < <(git ls-files -s)
   }
   ```
2. En `scripts/test-shellcheck.sh:51-64` y `scripts/test-rutas-personales.sh:24-37`, sustituir el bucle por:
   ```bash
   . "$(dirname "$0")/_bash-versionado.sh" || { echo "test-<nombre>: no puedo cargar scripts/_bash-versionado.sh" >&2; exit 1; }
   bash_versionado
   ```
   **Sin `2>/dev/null`** (a diferencia de los hooks warn-only): un gate debe fallar alto si no carga el helper. Se mantiene en cada gate la guarda existente "no se encontró ningún script — el recorrido está roto" (`test-shellcheck.sh:66-70`, `test-rutas-personales.sh:39-42`): cubre también un helper que cargue pero no rellene.
   La forma `. "$(dirname "$0")/_x.sh"` es la que shellcheck `-x -P SCRIPTDIR` ya sigue hoy sin directiva (`git-add-all-guard.sh:49` pasa el gate; `test-shellcheck.sh:39-40` lo documenta: *"sigue los `. "$(dirname "$0")/_helper.sh"`"*).
3. Actualizar el comentario de cabecera de `test-rutas-personales.sh:14-18` ("Mismo descubrimiento que test-shellcheck.sh…" → "Descubrimiento en `_bash-versionado.sh`, compartido con test-shellcheck.sh").
4. Oráculo: `bash scripts/test-shellcheck.sh` y `bash scripts/test-rutas-personales.sh` → los dos deben decir **47 scripts** (46 + el helper). Falsación: repetir la Falsación 1 del plan E (Task 8, :1533-1538) — sigue en rojo→verde. Comprobación de que el helper es lo que corre: añadir temporalmente `templates/*` a la exclusión del helper y ver que **los dos** conteos cambian a la vez (revertir).
5. `scripts/test-plugin.sh` no se ve afectado (descubre solo `plugins/exo/scripts/test-*.sh`).

**Cita**:
- `plugins/exo/scripts/_truncate-payload.sh:2-4`: *"Helper COMPARTIDO … Lo usan git-add-all-guard.sh y verify-before-commit.sh, que llevaban este bloque copiado línea a línea."* — el repo ya extrajo un helper con exactamente **dos** consumidores por el mismo motivo; el criterio "con dos sería over-engineering" del package E (:42) contradice el precedente del propio repo.
- Package E :42: *"Si divergen, los dos gates dejan de mirar lo mismo sin avisar."* — el orquestador identifica el fallo silencioso y lo deja; campaña E se llama "hooks honestos" (config.md:57).
- `test-shellcheck.sh:39-40`: *"`-x -P SCRIPTDIR`: sigue los `. "$(dirname "$0")/_helper.sh"`, así que un helper roto o una función mal llamada también cuentan."* — el helper queda cubierto por shellcheck sin trabajo extra.

**Evidencia propia**: comando 5 — bucles byte-idénticos (13 líneas) en los dos scripts; precedente sourceado en 2 hooks; conteo hoy 46/46. La divergencia sería silenciosa de verdad: ninguno de los dos gates compara su conteo con el otro y `test-contrato-ci.sh` no los cruza.

**Trade-offs**: (b) un fichero más y un `source` en cada gate; a cambio, una sola definición de "bash versionado" y una sola exclusión que editar. Fallo de carga = rojo explícito, no verde vacío. (a) cero trabajo hoy, y un modo de fallo que nadie detecta hasta que un script nuevo pasa shellcheck pero no el gate de rutas (o al revés). (c) alternativa barata considerada — un test que haga `diff` de los dos bloques extraídos por marcadores — más frágil que el helper y con el mismo número de ficheros tocados.

**Qué busqué para objetar**: el argumento más fuerte contra (b) es el perfil de Paul (solución simple hoy > arquitectura) y la regla de tres. No gana porque el propio repo fijó el umbral en dos con `_truncate-payload.sh` para un bloque de la misma naturaleza (descubrimiento/formateo copiado), y porque el coste de (b) es menor que el de un solo falso verde en CI investigado a mano. Segundo: ¿sourcear rompe en Git Bash/macOS? Los gates corren solo en el job `lint` (ubuntu, `ci.yml:24-64`), y el patrón `. "$(dirname "$0")/…"` ya corre en los hooks en las tres plataformas. Tercero: ¿el helper cambia el conteo y "rompe" el package E ("OK, 46 scripts")? Sube a 47 por la misma razón que subió de 45 a 46 al commitear el propio gate (ledger :1207) — no es regresión, y el package se reescribe una línea.

**Momento**: E aún no está en `origin` (package E :59-62, push pendiente de Paul). Va como commit adicional en `e-hooks-honestos-y-ci` antes del push, re-corriendo `test-shellcheck.sh`, `test-rutas-personales.sh` y `test-contrato-ci.sh`. Si Paul ya ha abierto el PR, va en rama propia post-merge — no bloquea.

**Rama afectada**: E.

---

## Tabla resumen

| # | Decisión | Opción | Rama | Coste estimado | Cambio |
|---|---|---|---|---|---|
| 1 | Gate de rutas: `/home/runner`, `/Users/Shared`, `C:/Users/Public` | **(a) dejar** | E (opcional, backlog) / ninguna | 0–5 min | Nada en código; una frase opcional en el ítem del backlog |
| 2 | `stale` con score negativo | **(a) declarar y dejar** — dominio del axioma 5 es `edad ≥ 0` (kbx `stale.go:82-83`); (b) contradice el gate exacto de `age_days`; (c) superficie sin consumidor | D (opcional) | 5 min | Comentario de 3 líneas sobre `axioma_5_…` en `obsolescencia.rs` |
| 3 | D-4 fallback sin `[kb] name` | **(b) abortar `--apply`**, dry-run sin config sigue; **D-4 base = `nombre_kb()` confirmado**; (c) descartada por `lib.rs:114-117` | D | ~30 min + `cargo test --test rotar_cli` | `main.rs:1485-1488` (menos código), 2 tests de `rotar_cli.rs`, 1 frase en `rotacion.md` |
| 4 | D-3 exit code de `rotate` | **exit 1 confirmado** | ninguna | 0 | Nada |
| 5 | Bucle duplicado de descubrimiento | **(b) helper `scripts/_bash-versionado.sh`** (precedente `_truncate-payload.sh` con 2 consumidores) | E | ~20 min + 2 gates | Helper nuevo; `source` en los 2 gates; conteo 46→47 |

FIRMA: Paul, firmado de antemano (2026-09-15T07:19:39+02:00, cita: "firmo de antemano la decision del consultor"; transcrita por el orquestador).
