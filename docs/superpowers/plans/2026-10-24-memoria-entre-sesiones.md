# Plan: memoria entre sesiones — etapa 1 (campaña M)

> For agentic workers: ejecución con `exo:orchestrate`. Las Tasks 1–8 son
> código; la parte «Operación» es un runbook que corre el orquestador, con
> gates del mantenedor, después de cerrar la ola 2.

**Goal:** construir y correr el experimento que decide si la auto memory
nativa de Claude Code es no inferior a exo (o al revés), con un oráculo como
ancla y sondas cribadas.

**Architecture:**
- Se extiende el harness de la campaña K (`evals/ablacion-k/harness/`) con el
  brazo `n` (auto memory nativa). `a2` hace de X y `ar` de oráculo O.
- Lo nuevo de la campaña vive en `evals/campana-m/`: generador de memoria,
  validador de tareas, selección de la criba, congelación, análisis y piloto
  de escritura.
- Los datos privados viven en `K_ROOT=~/.cache/exo-campana-m`, separado de
  los de K.

**Tech Stack:** bash, Python 3 (stdlib + pytest), jq, `claude -p` (sonnet),
exo 0.9.0 con `cuda`.

**Spec:** `docs/superpowers/specs/2026-10-24-memoria-entre-sesiones-design.md`

**Global Constraints:**
- Modelo de las corridas: `claude-sonnet-5-5`. `--max-budget-usd 10` por
  corrida y `--max-turns 40`, como K.
- Brazos: `a0` (A0), `ar` (O), `a2` (X), `n` (N). El neutro corre solo `a0`,
  `a2` y `n`.
- Criba: A0 × 2 réplicas, se queda la sonda con 0/2; después O × 2 réplicas,
  se queda la que da ≥ 1/2. Objetivo: 40 sondas-suelo. Si no se llega a 40,
  se para y se escala; no se rellena.
- Principal: 40 sondas × {a0, ar, a2, n} × 3 réplicas = 480 corridas.
  Neutro: 15 × {a0, a2, n} × 2 = 90 corridas.
- Éxito por sonda: fracción de réplicas con `check.rc == 0` (rc 1 y rc 2
  cuentan como fallo).
- Bootstrap sobre sondas, 10.000 réplicas, `random.Random(20261024)`,
  percentiles empíricos. Test de signos exacto bilateral sobre las
  discordantes.
- Reglas, margen 0,15:
  - R1: límite inferior del IC95 de Δ(n − a2) > −0,15, y además la mediana
    de tokens_in de n < la de a2 ⇒ `GANA N`.
  - R2: límite inferior del IC95 de Δ(a2 − n) > 0 y Δ ≥ 0,15 ⇒
    `EXO SOBREVIVE`.
  - R3: en el neutro, límite inferior del IC95 de Δ(brazo − a0) > −0,15 para
    a2 y para n; si no se cumple ⇒ `DAÑO <brazo>`.
  - Resto: `NO CONCLUYENTE`, y manda el coste (solo a2 frente a n).
- Invalidaciones:
  - (1) O recupera < 0,30 (menos de 12/40 sondas con ≥ 2/3 réplicas rc 0)
    ⇒ `CUELLO = CUMPLIMIENTO`.
  - (2) límite inferior del IC95 de Δ(a2 − a0) ≤ 0 ⇒ se decide solo por
    coste.
  - (3) > 5 % de corridas con escritura fuera de lo previsto ⇒ no se
    adjudica.
- «Sin memoria» no es un veredicto admisible. El suelo es el CLAUDE.md
  mínimo (`prep/claude-md.md`), que llevan todos los brazos.
- Solo cuentan las réplicas de la corrida principal; nunca las de la criba.
- `MEMORY.md` de N: una línea por nota core + stable, formato
  `- [<título>](<ruta relativa>) — <description o primera frase>`, ≤ 200
  líneas y ≤ 25.000 bytes.
- Topes: 130 USD en total entre criba, piloto y principal. El breaker de
  `tanda.sh` para si la media de tokens_in por corrida pasa de 2M.

**Decisiones del plan (no están en la spec; revisables):**
1. **Snapshot = el de K** (`kb_commit 80cba57`, 200 notas), reindexado con
   exo 0.9.0 en GPU. Garantiza que las reglas del pool, extraídas de ese
   commit, están en la KB que ven X y N. Una KB de hoy habría movido o
   destilado reglas, y X y N perderían por construcción.
2. **Neutro = 15 de las 20 tareas S2 de K**, muestreadas con
   `random.Random(20261024)`. Ya están validadas contra ese mismo snapshot.
   Sustituye a «regenerar con `brief-s2.md`» (§8 de la spec), que solo haría
   falta con otro snapshot.
3. **Pool = las 541 reglas de K.** Las 40 que ya tienen tarea van directas a
   la criba; el resto pasa por veto y redacción hasta tener ≥ 200
   candidatas. Las «decisiones fechadas» nuevas de la spec solo se extraen si
   el rendimiento de la criba no da 40.
4. `K_ROOT` propio (`~/.cache/exo-campana-m`): los datos de K no se tocan.

## Setup del worktree

- `python3 -m pytest --version` → pytest instalado (9.x). No hace falta nada
  más.

## Cierre de ola

- `python3 -m pytest -q evals/campana-m` → verde = `N passed`, 0 failed.
- `for t in evals/campana-m/test-*.sh; do bash "$t" || exit 1; done` →
  verde = todas las líneas `ok`, ninguna `FAIL`, exit 0.
- `bash evals/techo-reglas/test-correr-ar.sh && bash evals/techo-reglas-2/test-correr-arp.sh && bash evals/techo-reglas-2/test-correr-ars.sh`
  → verde = exit 0 (los brazos viejos no cambian).

## Mutación

- `evals/campana-m`: manual

## Recursos compartidos

- Las Tasks 1–8 no tocan `~/.cache`. Sus tests usan `mktemp -d` y fixtures
  propias.
- `evals/ablacion-k/harness/correr.sh` lo toca solo la Task 1.
- `evals/ablacion-k/harness/fugas.py` lo toca solo la Task 2.
- `evals/ablacion-k/harness/tanda.sh` lo toca solo la Task 7.
- La GPU y la API de Claude se usan solo en la parte de Operación.

## Olas

- **Ola 1, en paralelo:** T1, T2, T3, T4, T5, T6. Ficheros disjuntos y sin
  dependencias entre ellas.
- **Ola 2, en paralelo:** T7 (wiring: preparar, congelar y tanda; consume
  T1, T3 y T6) y T8 (piloto de escritura; consume T1).

---

### Task 1: brazo `n` y aislamiento por corrida en correr.sh

**Files:**
- Modify: `evals/ablacion-k/harness/correr.sh`
- Test: `evals/campana-m/test-correr-n.sh`

**Interfaces:**
- Produces: `correr.sh <dir tarea> n <rep>` usa `$P/memoria/` (de @Task 3)
  copiado a `$O/memoria/` en cada corrida
- Produces: variable `K_MEMORIA_DIR` (default `$P/memoria`)
- Produces: `$O/inyectado.txt` con lo que el brazo inyecta al arrancar:
  `regla-ctx.json` en `ar`, la salida de `exo-recall.sh` en `a2`,
  `MEMORY.md` en `n` y vacío en `a0`
- Produces: `$O/kb-sucia.txt` = salida de `git -C $P/kb status --porcelain`
  tras la corrida, antes de restaurar

**Tests:**
- `n_settings_activa_memoria`: en `K_ENSAYO=1` con brazo `n`,
  `settings.json` tiene `autoMemoryEnabled: true` y `autoMemoryDirectory`
  igual a `$O/memoria`. Falla si `n` hereda el `false` fijo de la línea 81.
- `otros_brazos_memoria_off`: con `a0`, `ar` y `a2`, `settings.json` tiene
  `autoMemoryEnabled: false` y no tiene `autoMemoryDirectory`. Falla si el
  cambio por brazo filtra la memoria a los demás.
- `n_copia_fresca`: dos corridas en ensayo con `n` y la réplica 1 y 2 dan
  dos `$O/memoria` distintas, idénticas byte a byte a `$K_MEMORIA_DIR`
  (`diff -r` vacío). Falla si se comparte la carpeta entre corridas.
- `n_deniega_snapshot`: con `n`, `cmdline.txt` contiene
  `deny=Read(/$P/kb/**)` y `PATH` empieza por `$P/stub`. Falla si N puede
  leer el snapshot o llamar a `exo`.
- `restaura_kb`: un fichero `.md` escrito en `$P/kb` desde un `claude`
  falso (stub en el PATH del test) queda listado en `kb-sucia.txt` y
  desaparece de `$P/kb` tras la corrida (`git status --porcelain` vacío).
  Falla si el I2 de K sigue abierto.
- `invariancia_brazos_viejos`: en ensayo, `settings.json` y `cmdline.txt` de
  `a0`, `ar`, `arp` y `ars` son idénticos a los de antes del cambio, salvo
  la clave `autoMemoryEnabled`, que sigue `false`. Falla si se rompe la
  reproducibilidad de K y techo-reglas.

**Verificación:** `bash evals/campana-m/test-correr-n.sh` → todas `ok`;
además los tres tests de brazos viejos del Cierre de ola, exit 0.

**Review Focus:**
- La restauración de la KB corre también cuando `claude` sale con error o
  por timeout (test `restaura_kb` con un stub que sale 1).
- `inyectado.txt` de `n` es el `MEMORY.md` copiado, no el de `$P`
  (test `n_copia_fresca` lo compara).
- `K_ENSAYO=1` sigue saliendo antes de lanzar `claude` y antes de restaurar
  nada.

**Notas:** la restauración es `git -C "$P/kb" checkout -- . && git -C "$P/kb" clean -fdq`,
y va después del check y de `fugas.py`, para que los dos vean el estado que
dejó el agente.

### Task 2: fugas.py para el brazo n y escrituras fuera de lo previsto

**Files:**
- Modify: `evals/ablacion-k/harness/fugas.py`
- Test: `evals/campana-m/test_fugas_m.py`

**Interfaces:**
- Produces: `HOOKS["n"] = set()`
- Produces: motivo `"escribe en el snapshot de la KB"` (todos los brazos) y
  motivo `"escribe en la memoria nativa"` (todos los brazos)
- Produces: `n` sujeto a las mismas reglas que `a0` frente a `exo` y al
  snapshot

**Tests:**
- `n_sin_hooks_limpio`: transcript sin eventos de hook con brazo `n` da
  `fuga: false`. Falla si `HOOKS` no tiene `n` (KeyError).
- `n_invoca_exo_es_fuga`: tool_use Bash `exo search x` sin el texto del stub
  en su resultado da el motivo `invoca exo sin bloqueo del stub`.
- `escritura_snapshot`: tool_use `Write` o `Edit` con `file_path` dentro de
  `$K_ROOT/prep/kb/` da `escribe en el snapshot de la KB` en `a2` y en `n`.
  Falla si solo se vigilan las lecturas.
- `escritura_memoria`: tool_use `Write` con `file_path` que contiene
  `/memoria/` bajo el directorio de la corrida da `escribe en la memoria
  nativa`.
- `lectura_memoria_n_permitida`: tool_use `Read` de `$O/memoria/MEMORY.md`
  en `n` da `fuga: false`. Falla si la lectura legítima de N cuenta como
  fuga.

**Verificación:** `python3 -m pytest -q evals/campana-m/test_fugas_m.py` →
5 passed; Cierre de ola verde.

**Review Focus:**
- La escritura por Bash (`printf >> $P/kb/x.md`, `tee`, `sed -i`) también
  cuenta: es lo que hizo `g1-16/a3-r2` en K. Necesita su test
  (`escritura_snapshot_bash`).
- El directorio de la corrida llega como argumento `d`; la ruta de memoria
  se deriva de `d`, no de `K_ROOT`.

### Task 3: generador de la memoria nativa de N

**Files:**
- Create: `evals/campana-m/gen_memoria.py`
- Test: `evals/campana-m/test_gen_memoria.py`

**Interfaces:**
- Produces: `gen_memoria.py <kb> <salida>`. Escribe `<salida>/MEMORY.md` y
  copia todas las notas `.md` de `<kb>` (sin `.git/`) bajo
  `<salida>/<ruta relativa>`, como topic files. Imprime
  `lineas=<n> bytes=<b> notas=<m>` y sale 0; sale 1 si se pasan los límites.
- Produces: `linea_indice(titulo: str, ruta: str, cuerpo: str, description: str | None) -> str`

**Tests:**
- `solo_core_y_stable`: una KB fixture con notas `tier: core`, `stable`,
  `log` y sin tier da un `MEMORY.md` con exactamente las de core y stable.
  Falla si se cuela un log o un archive.
- `formato_linea`: `linea_indice("T", "a/b.md", "Primera frase. Segunda.", None)`
  da `- [T](a/b.md) — Primera frase.`, y con `description="D"` da
  `- [T](a/b.md) — D`.
- `limites`: con 201 notas core, sale 1 y no escribe `MEMORY.md`; también
  con 199 notas cuyas líneas suman > 25.000 bytes. Falla si se trunca en
  silencio, que es lo que haría la plataforma.
- `topic_files_completos`: el número de `.md` bajo `<salida>`, menos
  `MEMORY.md`, es igual al de la KB sin `.git/`, y cada uno es idéntico byte
  a byte al original.
- `determinista`: dos ejecuciones dan `MEMORY.md` idénticos (orden por ruta).

**Verificación:** `python3 -m pytest -q evals/campana-m/test_gen_memoria.py`
→ 5 passed.

**Review Focus:**
- Notas sin `title` en el frontmatter: el título sale del nombre del
  fichero sin `.md` (test).
- Una «primera frase» que empieza por un encabezado `#` o por el
  frontmatter: se salta hasta el primer párrafo de texto (test).
- Rutas con espacios y em dash, que en la KB son comunes: el enlace
  markdown queda con la ruta tal cual, entre `<>` si lleva espacios (test).

### Task 4: análisis y reglas de decisión

**Files:**
- Create: `evals/campana-m/analizar.py`
- Test: `evals/campana-m/test_analizar.py`

**Interfaces:**
- Produces: `analizar.py <K_ROOT> <sondas.tsv> <neutras.txt>` →
  `analisis.json` y una tabla en stdout
- Produces: `exito_por_sonda(rcs: list[int]) -> float`
- Produces: `delta_bootstrap(a: dict[str, float], b: dict[str, float], semilla=20261024, n=10_000) -> tuple[float, float, float]`
  (media, límite inferior, límite superior)
- Produces: `signos(a, b) -> tuple[int, int, float]` (a > b, a < b, p
  bilateral)
- Produces: `veredicto(m: dict) -> list[str]` con las etiquetas exactas
  `GANA N`, `EXO SOBREVIVE`, `NO CONCLUYENTE`, `DAÑO a2`, `DAÑO n`,
  `CUELLO = CUMPLIMIENTO`, `SOLO COSTE`, `NO ADJUDICABLE`
- `sondas.tsv`: `id<TAB>tier_origen` (core|stable|log|archive)

**Tests:**
- `rc2_es_fallo`: `exito_por_sonda([0, 2, 1])` da `1/3`. Falla si rc 2 se
  excluye.
- `bootstrap_reproduce_k`: con las 40 Δ de K reconstruidas (34 en 0 y
  {1, 1, 1, ½, −½, −½}) da IC95 `[-0.025, 0.1625]`. Falla si cambian la
  semilla, el método o el remuestreo.
- `r1_gana_n`: un fixture con Δ(n − a2) ≈ 0, IC [−0,10, +0,10] y tokens_in
  de n menores da `GANA N`; el mismo con tokens_in de n mayores da
  `NO CONCLUYENTE`.
- `r2_exo`: Δ(a2 − n) = 0,20 con límite inferior 0,05 da `EXO SOBREVIVE`;
  con Δ = 0,10 (límite inferior > 0) no lo da.
- `invalidacion_oraculo`: 11/40 sondas con O ≥ 2/3 da `CUELLO =
  CUMPLIMIENTO` y ningún veredicto de acceso; 12/40 no lo da (umbral
  exacto).
- `invalidacion_x_a0`: límite inferior de Δ(a2 − a0) ≤ 0 da `SOLO COSTE`.
- `no_adjudicable`: 25/480 corridas con `kb-sucia.txt` no vacío o fuga de
  escritura da `NO ADJUDICABLE`; 24/480 no (5 % exacto: > 5 %).
- `solo_replicas_principal`: corridas en `corridas-criba/` no entran en
  ninguna métrica.
- `estratifica_tier`: `analisis.json` lleva Δ(n − a2) por `tier_origen`
  además del global.
- `coste`: Δ emparejada de tokens_in frente a a0 sobre las 55 sondas, y la
  sobrecarga fija = bytes de `inyectado.txt` por brazo (mediana).
- `cociente_R`: con éxito medio a0 = 0,1, ar = 0,6 y n = 0,35, R(n) = 0,5.
  Con ar ≤ a0, R sale `null` y no da error de división.
- `desempate_prevalencia`: `analisis.json` lleva, para a2 y n,
  `0.012 × Δ(brazo − a0)` junto a la sobrecarga fija. El campo existe
  siempre, pero `veredicto` solo lo usa cuando no hay R1, R2 ni R3.
- `version_discrepante`: una corrida cuyo `claude-version.txt` o modelo
  difiere de `congelacion-principal.txt` queda excluida y se lista en
  `excluidas`.

**Verificación:** `python3 -m pytest -q evals/campana-m/test_analizar.py`
→ 13 passed.

**Review Focus:**
- Sondas a las que les falta una réplica (`meta.json` con `error`): cuentan
  como fallo, no se descartan (test).
- R3 se evalúa sobre el neutro y R1/R2 sobre las sondas-suelo; no se
  mezclan (test con un neutro malo y un suelo bueno).
- Varias etiquetas a la vez (p. ej. `GANA N` + `DAÑO n`): `veredicto`
  devuelve las dos (test).

### Task 5: validador de tareas candidatas

**Files:**
- Create: `evals/campana-m/validar_tareas.py`
- Test: `evals/campana-m/test_validar_tareas.py`

**Interfaces:**
- Produces: `validar_tareas.py <dir gold/s1>` → una línea por tarea,
  `<id>\tok` o `<id>\tKO\t<motivo>`; sale 1 si alguna es KO
- Produces: `valida(d: Path) -> str | None` (None = ok)

**Tests:**
- `schema`: `tarea.json` sin `prompt` o sin `sanity` da KO con `falta
  prompt` / `falta sanity`. Una tarea `no_convertible: true` con `motivo` da
  ok y se cuenta aparte.
- `sanity_reproducido`: si `check.sh` sobre `pruebas/cumple` no sale 0, o
  sobre `pruebas/viola` no sale 1, KO con `sanity cumple=<rc>` /
  `sanity viola=<rc>`. Falla si se fía del `sanity` declarado.
- `vacio_no_rompe`: `check.sh` sobre el repo sin tocar y un transcript vacío
  debe salir 0, 1 o 2; otro rc (p. ej. 127) da KO.
- `prompt_sin_regla`: un prompt que contiene `KB`, `memoria`, `nota`,
  `doctrina` o `exo` (palabra entera, sin distinguir mayúsculas) da KO
  `prompt menciona la memoria`.

**Verificación:** `python3 -m pytest -q evals/campana-m/test_validar_tareas.py`
→ 4 passed.

**Review Focus:**
- Tareas `setup: true`: el repo de prueba sale de ejecutar `setup.sh` en un
  temporal, nunca del repo real (test).
- Checks de tipo `rubrica` (sin `check.sh`): se validan solo por schema
  (`items` de 1 a 3) (test).

### Task 6: selección de la criba y orden de las corridas

**Files:**
- Create: `evals/campana-m/criba.py`
- Test: `evals/campana-m/test_criba.py`

**Interfaces:**
- Produces: `criba.py orden <paso a0|ar> <candidatas.txt> <salida>` →
  fichero de orden `tarea\tbrazo\trep` con 2 réplicas, barajado con
  `random.Random(20261024)`
- Produces: `criba.py selecciona <paso> <K_ROOT> <candidatas.txt>` →
  supervivientes a stdout. Paso `a0`: rc ≠ 0 en las 2 réplicas. Paso `ar`:
  rc == 0 en ≥ 1 de 2.
- Produces: `criba.py principal <sondas.txt> <neutras.txt> <salida>` → 480
  + 90 líneas barajadas con la misma semilla
- Lee las corridas de la criba en `$K_ROOT/corridas-criba/`

**Tests:**
- `a0_exige_cero`: rc {1, 2} sobrevive; {0, 1} no. Falla si rc 2 cuenta
  como acierto de A0.
- `ar_exige_uno`: {0, 1} sobrevive; {1, 2} no.
- `faltante_no_sobrevive`: candidata con una sola réplica de a0 no
  sobrevive y sale en stderr como `incompleta`.
- `principal_cuenta`: 40 sondas y 15 neutras dan 570 líneas; brazos {a0,
  ar, a2, n} × {1, 2, 3} para sondas y {a0, a2, n} × {1, 2} para neutras.
- `orden_determinista`: dos ejecuciones dan el mismo fichero (sha igual).

**Verificación:** `python3 -m pytest -q evals/campana-m/test_criba.py` → 5
passed.

**Review Focus:**
- Más de 40 supervivientes: `selecciona` no recorta. Recortar es una
  decisión pre-registrada (las primeras 40 por orden barajado con la
  semilla) y va en `principal` (test).

### Task 7: preparar, congelar y tandas de la campaña M

**Files:**
- Create: `evals/campana-m/preparar-m.sh`
- Create: `evals/campana-m/congelar-m.sh`
- Modify: `evals/ablacion-k/harness/tanda.sh`
- Test: `evals/campana-m/test-preparar-m.sh`

**Interfaces:**
- Consumes: `gen_memoria.py <kb> <salida>` @Task 3
- Consumes: `criba.py principal ...` @Task 6
- Consumes: `correr.sh ... n` @Task 1
- Produces: `preparar-m.sh`:
  - llama a `harness/preparar.sh 80cba57878d37583be1f2654ecb7d96c51dad8ab`
    con `K_ROOT=~/.cache/exo-campana-m`;
  - luego `exo rebuild` con el binario `cuda`, y exige
    `meta.dispositivo_embeddings = cuda`;
  - luego `gen_memoria.py $P/kb $P/memoria`.
- Produces: `congelar-m.sh <fase criba|principal>` → `congelacion-<fase>.txt`
  con:
  - sha256 de `~/.cache/exo-ablacion-k-registro.tar.gz` (debe ser
    `116964a9ed565e1c6ac6a887c20de956eae5f2aec48cb317bf4092b2db57abd9`);
  - `kb_commit`, `exo --version`, `claude --version` y el modelo;
  - sha de `prep/memoria/MEMORY.md`, sha del gold activo, sha del orden y la
    semilla `20261024`.
- Produces: `tanda.sh`, nuevas variables:
  - `K_ORDEN` (default `$K_ROOT/orden-etapa1.txt`, como hasta ahora);
  - `K_TANDA` (corridas por tanda, default 40);
  - `K_CORRIDAS` (default `corridas`);
  - `K_TOPE_USD` (si la suma de `usd` de todas las corridas de `K_CORRIDAS`
    lo supera ⇒ PARAR).

**Tests:**
- `tanda_default_igual`: sin las variables nuevas, `tanda.sh` lee
  `orden-etapa1.txt` en bloques de 40. Falla si cambia el comportamiento de
  K.
- `tope_usd`: con `K_TOPE_USD=1` y `meta.json` cuya suma de `usd` es 1,5,
  el resumen dice `PARAR: coste` y sale 1.
- `congelar_sha_tarball`: con un tarball de sha distinto,
  `congelar-m.sh criba` sale 1 con `tarball de K no coincide`.
- `preparar_exige_cuda`: con un `exo` stub cuya db no tiene
  `dispositivo_embeddings`, `preparar-m.sh` sale 1 con `índice no
  embebido en cuda`.

**Verificación:** `bash evals/campana-m/test-preparar-m.sh` → todas `ok`;
Cierre de ola verde.

**Review Focus:**
- `preparar.sh` hace `rm -rf "$P"`: con `K_ROOT` mal exportado borraría el
  prep de K. Test: sin `K_ROOT` explícito, `preparar-m.sh` se niega a
  correr.

### Task 8: piloto de escritura (descriptivo)

**Files:**
- Create: `evals/campana-m/escritura.sh`
- Test: `evals/campana-m/test-escritura.sh`

**Interfaces:**
- Consumes: aislamiento de `correr.sh` (flags y deny) @Task 1
- Produces: `escritura.sh <sesion.json> <brazo x|n> <salida>`.
  `sesion.json` = `{id, turnos: [prompt...], decision}`. Corre los turnos
  con `claude -p --resume` sobre una copia del snapshot (x, con
  `--plugin-dir plugins/exo`) o de `prep/memoria` (n). Después envía el
  cierre literal `cierra la sesión y persiste lo decidido` y deja en
  `<salida>/escrito.diff` el diff de la KB (x) o de la carpeta de memoria
  (n).

**Tests:**
- `cierre_literal`: en ensayo (`K_ENSAYO=1`), el último prompt de
  `cmdline.txt` es exactamente `cierra la sesión y persiste lo decidido` en
  los dos brazos. Falla si el disparador difiere entre brazos.
- `copias_aisladas`: el diff se calcula sobre la copia y el snapshot
  original queda intacto (`git status --porcelain` vacío).
- `brazo_desconocido`: brazo `a0` sale 2.

**Verificación:** `bash evals/campana-m/test-escritura.sh` → todas `ok`.

**Review Focus:**
- En x, `/document` necesita el plugin. Hay que comprobar en ensayo que
  `--plugin-dir` apunta a `plugins/exo` del repo, no a la caché instalada.

---

## Operación (runbook del orquestador, tras la ola 2)

Cada paso con **[gate]** espera el OK explícito del mantenedor. Ningún paso
gasta API sin haber pasado su gate.

1. **Preparar.** `K_ROOT=~/.cache/exo-campana-m evals/campana-m/preparar-m.sh`.
   Verde = `lineas=` ≤ 200 y `bytes=` ≤ 25000 en la salida de
   `gen_memoria`, más `dispositivo_embeddings = cuda`.
2. **Canario de N [gate].** Una corrida `n` sobre una tarea sintética cuyo
   check es «la respuesta final cita una línea literal de `MEMORY.md`».
   - Verde: el transcript muestra el contenido de `MEMORY.md` en el contexto
     o en la respuesta.
   - Si falla: plan B de la spec §8 (`--setting-sources project`). Si
     también falla, se para y se escala.
   - Coste < 1 USD.
3. **Veto y redacción.** Sobre las 501 reglas sin tarea:
   - veto con `pool/brief-veto.md`, en lotes de ≤ 60;
   - redacción con `pool/brief-redaccion.md`, sin cambiar el brief, en lotes
     de ~40, por subagentes que no ven brazos ni resultados.
   - Cada lote pasa `validar_tareas.py` antes de entrar; un KO se descarta,
     no se arregla a mano.
   - Para al llegar a 200 candidatas válidas (incluidas las 40 de K).
   - Ruta de las fuentes: congelar con `harness/congelar_fuentes.sh`.
4. **Congelar la criba [gate].** `congelar-m.sh criba`. Coste estimado de la
   criba: ~40 USD.
5. **Criba.** Paso A0 y después paso O, con
   `K_ORDEN=orden-criba-<paso>.txt K_CORRIDAS=corridas-criba K_TOPE_USD=130 tanda.sh`.
   - **Tras el paso A0 [gate]:** se reporta el rendimiento. Si la proyección
     da < 40 sondas-suelo, se para y se escala (spec §4.2).
6. **Piloto de N.** 5 sondas cribadas × `n` × 1 réplica. Se reporta la cola
   de tokens_in.
7. **Congelar la principal [gate].**
   - Pre-registro `evals/campana-m/preregistro.md`, commiteado antes de la
     primera corrida: las reglas y las invalidaciones copiadas de los Global
     Constraints, el sha del orden, la semilla y las neutras elegidas.
   - `congelar-m.sh principal`.
8. **Principal.** 570 corridas en tandas de 40, con breakers. Si un breaker
   para, se reporta y se espera **[gate]**.
9. **Análisis.** `analizar.py`.
10. **Adjudicación.** Un agente fresco, sin participación previa, recomputa
    desde los datos crudos y escribe `evals/campana-m/verdict.md`, como en
    K.
11. **Piloto de escritura.**
    - Sesiones sintéticas: un subagente las deriva de 10 sondas «decisión».
    - `escritura.sh` en x y n.
    - Un juez a ciegas puntúa: (a) ¿está D?, (b) bytes, (c) tamaño del diff
      para dejarlo limpio, (d) lectura de vuelta.
    - Va en una sección descriptiva aparte del verdict.
