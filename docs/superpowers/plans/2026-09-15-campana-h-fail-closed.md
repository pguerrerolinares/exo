# Campaña H — Fail-closed: `doctor` y cutover binario↔plugin que no mienten

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para
> tracking. Sesión-fábrica: `paul-profile:fabrica` con
> `.superpowers/fabrica/config.md` (el orquestador lo integra; este plan no lo
> edita).

**Goal:** que ningún componente de exo degrade «con forma válida» sin dejar
rastro: el hook detecta un binario viejo del engine, `exo doctor` no da `ok`
por un `bash.exe` de WSL ni resuelve una versión de plugin al azar, y
`kb-precommit.sh` deja de dejar pasar un commit sin gate en silencio — y de
paso se construye el mecanismo (`ENGINE_MIN`) que hace falta porque esta
misma campaña provoca la primera ruptura real de compatibilidad
binario↔plugin: retirar los diez alias españoles del CLI.

**Architecture:** ocho tareas. Las tres primeras (1-3) construyen y conectan
el contrato de versión mínima (`ENGINE_MIN`, un fichero de una línea junto al
plugin) de extremo a extremo: el helper bash que lo lee y compara (Task 1),
los dos hooks que lo consultan antes de servir memoria (Task 2), y el check
nuevo de `exo doctor` que lo compara contra el binario que lo ejecuta
(Task 3). Las tareas 4 y 5 cierran los dos huecos declarados de G5b en
`doctor.rs` — un `bash` de WSL que pasa por Git Bash (Task 4) y un
`script_del_plugin` que ordena versiones como texto en vez de como números,
bug que Task 3 expone al escribir el comparador de versiones correcto y que
Task 5 reutiliza para arreglarlo (Task 5 depende de Task 3). La Task 6 es
independiente de las anteriores: `kb-precommit.sh` deja de degradar a
«commit permitido» cuando no encuentra `exo`. La Task 7 es la ruptura en sí
— retira los diez alias, sube `engine` a 0.2.0 y el plugin a 1.2.0, y deja el
runbook de esa release. La Task 8 cierra: mide el cap del bloque de arranque
para la acción (c) pendiente de ese ítem del backlog, y sincroniza
`docs/backlog.md` citando los commits reales de 1-7.

**Tech Stack:** Rust 2024 (crate `exo` en `engine/`, MSRV 1.95) · `clap`
4.6.2 derive (los `alias` de la Task 7 son atributos de `clap::Args`, sin
dependencia nueva) · bash (Git Bash en Windows) + `jq` · GitHub Actions
(`.github/workflows/ci.yml`) · sin Go, sin Python: todo bash puro o Rust.

## Verificación de la propuesta contra el código (2026-09-15, este worktree)

La propuesta de origen (`docs/superpowers/consultas/2026-09-15-campanas/propuesta.md`
§2) se releyó línea a línea contra el árbol de `origin/main` en `99ddd05`.
Resultado: **acertada casi en su totalidad**, con dos precisiones que este
plan corrige:

- **Los «10 `alias =`» son exactos.** `grep -n 'alias = "' engine/src/main.rs`
  da 10 líneas, en `main.rs:153,185,235,247,257,284,297,304,311,315` — no una
  cifra a ojo, es el mismo número que declara el test
  `los_flags_espanoles_siguen_parseando_como_alias` de `engine/tests/flags.rs`
  («los diez pares, no una muestra»). Los pares son: `--titulo`, `--crea`,
  `--limite` (en `search` y en `recall`, dos declaraciones distintas),
  `--min-similitud` (ídem, dos declaraciones), `--escala-fts`, `--contenido`,
  `--nota`, `--refresca`.
- **Las líneas de `doctor.rs` citadas por la propuesta cuadran, con
  desplazamiento de una o dos líneas** (el árbol se movió desde que el
  consultor lo leyó): `check_git_bash` está en `doctor.rs:559-582` (la
  propuesta decía «~568» para la llamada a `busca_en_path` — es exactamente
  la línea 568) y `script_del_plugin` en `doctor.rs:746-768`, con el
  comentario que declara el `sort()` lexicográfico deliberado en `:740-745`
  (la propuesta decía «~740-743» y «:762» para la llamada a `.sort()» — la
  llamada real está en `:762`). Sin caducidad: el código de hoy es el mismo
  que describe la propuesta.
- **Corrección real, no de línea**: la propuesta describe la Task de
  `script_del_plugin` como «alinear o documentar, decide con argumento». Este
  plan decide **alinear**: la Task 3 (`plugin_compat`) escribe de cero un
  comparador de semver correcto para leer `ENGINE_MIN` del plugin instalado;
  dejar `script_del_plugin` ordenando por texto tres funciones más abajo en
  el mismo fichero, cuando ya existe el comparador correcto al lado, sería
  inconsistencia sin motivo, no una segunda decisión de diseño. Task 5
  reutiliza el comparador de Task 3.
- **`kb-precommit.sh` y el orden del cutover**: confirmados letra por letra.
  `kb-precommit.sh:20` hace `exit 0` con un aviso por stderr cuando `$EXO` no
  es ejecutable — commit permitido sin gate. El ítem del backlog sobre el
  «orden del cutover» (`docs/backlog.md:437-461` en este árbol) sigue
  abierto solo por la mitad que le falta: el check permanente en
  `exo doctor`, que es exactamente la Task 3.
- **No hay nada caducado que reportar** en el alcance de H: a diferencia de
  F (que sí encontró varios ítems superados), los tres ítems de backlog que
  toca esta campaña —cutover, `kb-precommit.sh`, aliases— describen hoy
  exactamente lo que hay en el árbol.

## Decisiones de Paul ya tomadas (2026-09-15) — Global Constraints

Todas las siguientes son **hechos**, no propuestas a validar por este plan;
cada tarea las hereda sin poder reabrirlas:

- **#10 — el check de desfase binario↔plugin entra en H**, en `exo doctor` y
  en los hooks (`exo-recall.sh`, `recall-inject.sh`). Antes estaba fuera de
  alcance por decisión de Paul en la campaña E (`config.md`: «check de
  desfase en `doctor` FUERA»); esa exclusión queda **derogada** por esta
  campaña.
- **#11 — `kb-precommit.sh` sin `exo` pasa a fail-closed (`exit 1`)**, con
  escape consciente documentado en el propio mensaje: `git commit
  --no-verify`.
- **Retirar los aliases españoles del CLI entra en H y es la primera
  ruptura de compatibilidad: `engine` sube de 0.1.0 a 0.2.0.** Consecuencias
  a planificar (todas en Task 7): `engine/Cargo.toml` → `0.2.0`,
  `plugins/exo/.claude-plugin/plugin.json` y `.claude-plugin/marketplace.json`
  → `1.2.0`, `ENGINE_MIN` → `0.2.0`, el gate `scripts/test-versiones.sh`
  comprobando que `ENGINE_MIN` ≤ versión de `engine/Cargo.toml`, y todo
  consumidor que use un alias migrado al nombre canónico. **Grep exhaustivo
  hecho para este plan** (`git grep -n -- '--limite\|--titulo\|--contenido\|--nota\|--refresca\|--crea\|--min-similitud\|--escala-fts'`
  sobre `plugins/`, `docs/`, `scripts/`, `engine/tests/`,
  `.github/workflows/`): el **único** sitio que usa la forma española es
  `engine/tests/flags.rs` (el propio test que existe para probar el alias) —
  ningún script del plugin, ninguna skill, ningún doc, ningún workflow de CI
  invoca un alias español. La lista de ficheros a tocar en Task 7 es, por
  tanto, exactamente: `engine/src/main.rs`, `engine/tests/flags.rs`,
  `engine/Cargo.toml`, `plugins/exo/ENGINE_MIN`,
  `plugins/exo/.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`.
- **Huecos de G5b, ambos entran en H**: (i) `check_git_bash` da `ok` con un
  `bash` de WSL (Task 4); (ii) `script_del_plugin` ordena lexicográficamente
  en vez de por semver (Task 5, decisión: **alinear**, ver arriba).
- **Fuera de alcance de H, va a un checklist de runbook, no a tasks
  ejecutables** (exige máquina Linux o acciones de Paul fuera del repo):
  instalar `v0.1.0`/`0.2.0` en Linux y correr `exo doctor` ahí, repuntar el
  marketplace remoto, renombrar `exo-b1-real`, el tag `v0.2.0` en sí. Van en
  el runbook que crea la Task 7.
- **Directiva de Paul: construir antes que medir.** Ninguna tarea de este
  plan abre una ventana de medición nueva. La única lectura de un número
  real (Task 8, el tamaño del bloque de arranque) es una comprobación
  puntual con el instrumento que ya existe (`exo recall --content`), no un
  experimento nuevo.

## Restricciones de la ola (F ∥ G ∥ H, merge H → F → G)

- **`engine/src/main.rs`**: H toca los alias (`:153-315`, Task 7) y nada del
  `--version`/`enum Comando` en sí (clap deriva `--version` automáticamente
  de `Cargo.toml`, sin tocar la macro `#[command(...)]`). G toca el default
  de `--type` en `ArgsSearch` (`:241`, dentro del bloque de la Task 7 pero
  en una línea completamente distinta a los `alias =`) y `write_append_cmd`
  (`:832-868`). **No solapan**: verificado línea a línea contra el `main.rs`
  de este worktree — el bloque de `ArgsSearch` que Task 7 toca es
  `:233-262` (los campos con `alias`), y la línea 241 (`r#type`) no lleva
  `alias` y no se edita.
- **`engine/src/doctor.rs`**: G unifica `walk_kb`, que usan `check_kb`
  (`:312`, vía `crate::walker::walk_kb(&kb)`) y `mtime_mas_reciente`
  (`:451`). **Ninguna tarea de H toca esas dos funciones** — Task 3 añade
  `check_plugin_compat` (función nueva, al final del fichero), Task 4 edita
  `check_git_bash` (`:559-582`), Task 5 edita `script_del_plugin`
  (`:746-768`). Cero líneas compartidas con G en este fichero.
- **`exo-recall.sh` / `recall-inject.sh`** (Task 2): compartidos con la ola 2
  (campaña I, que reescribe su parseo de `jq`). Task 2 añade el check de
  versión con el mínimo de líneas posible, como un `elif`/guard más del
  mismo estilo que ya usan estos scripts (`no-engine`, `no-index`), sin
  tocar ni una línea del parseo de `jq` existente.
- **Máquina de ejecución**: Windows 11 + Git Bash, `cargo` disponible, sin
  Go. Tests del engine: `cd engine && cargo test --release --locked`; tests
  del plugin: `plugins/exo/scripts/test-*.sh` (descubiertos por
  `scripts/test-plugin.sh`) y los gates de `scripts/`. `.github/workflows/ci.yml`
  es la fuente de los comandos exactos de CI (`cargo fmt --check`,
  `cargo clippy --all-targets --locked -- -D warnings`,
  `scripts/test-versiones.sh`, `scripts/test-shellcheck.sh`,
  `scripts/test-exec-bit.sh`).
- **Git**: `git add <rutas explícitas>`, nunca `-A`/`.`; `git -C <path>`,
  nunca `cd <path> && git`. Nada de push.
- **`.gitattributes`: `* text=auto eol=lf`.** No normalizar finales de línea
  a mano.
- **Convención de commits de esta rama**: `fix(h, <área>): …` /
  `feat(h, <área>): …` / `test(h, <área>): …` / `docs(h, <área>): …`.

---

### Task 1: Contrato de versión mínima — `ENGINE_MIN` + helper bash + gate de versiones

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`bash plugins/exo/scripts/test-engine-version.sh` y
`bash scripts/test-versiones.sh` verdes.

**Evidencia:** hoy no existe ningún fichero `ENGINE_MIN`, ningún helper de
comparación de versiones bajo `plugins/exo/scripts/`, y
`scripts/test-versiones.sh` solo comprueba que `plugin.json`/`marketplace.json`
coincidan entre sí y que `engine/Cargo.toml` tenga una versión legible — nada
relaciona el plugin con la versión mínima de engine que necesita.

**Diseño**: `ENGINE_MIN` se fija HOY al valor de la versión actual de
`engine/Cargo.toml` (`0.1.0`) — un contrato que empieza siendo un no-op
(nada es «viejo» todavía) y que la Task 7 sube a `0.2.0` en el mismo commit
que rompe la compatibilidad de verdad. Así el gate nuevo de esta tarea
(`ENGINE_MIN` ≤ `Cargo.toml`) queda en verde desde el primer commit y en
cada uno de los siguientes, sin depender del orden de ejecución de las
demás tareas.

**Files:**
- Create: `plugins/exo/ENGINE_MIN`
- Create: `plugins/exo/scripts/_engine-version.sh`
- Create: `plugins/exo/scripts/test-engine-version.sh`
- Modify: `scripts/test-versiones.sh`

**Interfaces:**
- Produces: `semver_lt A B` (función bash, exit 0 si `A < B` componente a
  componente entero, exit 1 si no) y `exo_version_de BIN` (función bash,
  imprime `"X.Y.Z"` leído de `BIN --version` — clap emite `"exo X.Y.Z"` por
  el atributo `version` de `#[command(...)]` en `main.rs:28-32` — o cadena
  vacía si `BIN` no corre o no tiene esa forma). Las consume la Task 2.

- [ ] **Step 1: Test que falla — `test-engine-version.sh`**

Crea `plugins/exo/scripts/test-engine-version.sh`:

```bash
#!/usr/bin/env bash
# Test standalone para _engine-version.sh: semver_lt y exo_version_de.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

. "$SCRIPT_DIR/_engine-version.sh"

# ------------------- semver_lt: seis pares, sin spawns --------------------
verifica_lt() {  # $1=a $2=b $3=esperado(0 o 1, como exit code de semver_lt)
  semver_lt "$1" "$2"
  local rc=$?
  if [ "$rc" -eq "$3" ]; then
    pass "semver_lt $1 $2 -> $rc"
  else
    fail "semver_lt $1 $2 -> $rc" "esperaba $3"
  fi
}
verifica_lt "0.1.0" "0.2.0" 0   # menor: cierto
verifica_lt "0.2.0" "0.1.0" 1   # mayor: falso
verifica_lt "0.2.0" "0.2.0" 1   # igual: falso (no es "menor que")
verifica_lt "1.9.0" "1.10.0" 0  # el caso que rompe la comparación como texto
verifica_lt "0.2.0" "0.2.1" 0   # parche decide
verifica_lt "1.0.0" "0.99.99" 1 # el mayor gana aunque los otros campos sean grandes

# ------------------- exo_version_de: stub que imita `exo --version` -------
STUB="$TMP/exo-stub"
cat > "$STUB" <<'EOF'
#!/usr/bin/env bash
echo "exo 0.1.0"
EOF
chmod +x "$STUB"
V="$(exo_version_de "$STUB")"
if [ "$V" = "0.1.0" ]; then
  pass "exo_version_de lee 'exo 0.1.0' -> 0.1.0"
else
  fail "exo_version_de lee 'exo 0.1.0' -> 0.1.0" "obtuve '$V'"
fi

V2="$(exo_version_de "$TMP/no-existe")"
if [ -z "$V2" ]; then
  pass "exo_version_de con binario inexistente -> cadena vacía"
else
  fail "exo_version_de con binario inexistente -> cadena vacía" "obtuve '$V2'"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

Marca ejecutable y corre:

```bash
chmod +x plugins/exo/scripts/test-engine-version.sh
bash plugins/exo/scripts/test-engine-version.sh
```

Expected: falla al cargar — `_engine-version.sh: No such file or directory`
(o, en bash con `set -uo pipefail` sin `-e`, el `.` falla silenciosamente y
las llamadas a `semver_lt`/`exo_version_de` truenan con «command not
found»). El fichero que se sourcea todavía no existe: es el rojo de esta
tarea.

- [ ] **Step 2: Implementación — `_engine-version.sh`**

Crea `plugins/exo/scripts/_engine-version.sh`:

```bash
#!/usr/bin/env bash
# Helper COMPARTIDO: compara versiones semver X.Y.Z en bash puro — sin
# `sort -V` (no está en macOS/BSD), sin `bc`, sin spawns extra. Lo usan
# exo-recall.sh y recall-inject.sh (Task 2), hooks que corren en cada
# sesión/prompt: un spawn de más ahí se paga en cada turno.
#
# Uso:
#   . "$SCRIPT_DIR/_engine-version.sh"
#   semver_lt "0.1.0" "0.2.0" && echo "0.1.0 es menor que 0.2.0"
#
# `semver_lt A B`: exit 0 si A < B componente a componente, como ENTEROS —
# "9" < "10" en semver, al revés que en comparación de texto ("1.10.0" <
# "1.9.0" como cadena, y es justo el bug que doctor.rs::script_del_plugin
# tenía antes de esta campaña). Un componente ausente o no numérico en
# cualquiera de las dos cuenta como 0: basta para X.Y.Z, no es un parser de
# semver completo (sin prerelease/build metadata).
semver_lt() {
  local a="$1" b="$2"
  local -a pa pb
  IFS='.' read -r -a pa <<< "$a"
  IFS='.' read -r -a pb <<< "$b"
  local i na nb
  for i in 0 1 2; do
    na="${pa[$i]:-0}"; nb="${pb[$i]:-0}"
    case "$na" in ''|*[!0-9]*) na=0 ;; esac
    case "$nb" in ''|*[!0-9]*) nb=0 ;; esac
    if [ "$na" -lt "$nb" ]; then return 0; fi
    if [ "$na" -gt "$nb" ]; then return 1; fi
  done
  return 1
}

# exo_version_de BIN: imprime "X.Y.Z" leído de `BIN --version` (clap emite
# "exo X.Y.Z", `main.rs:28-32`). Cadena vacía si BIN no corre o la salida no
# tiene esa forma exacta — nunca revienta al llamador.
exo_version_de() {
  local bin="$1" out
  out="$("$bin" --version 2>/dev/null)" || { printf ''; return; }
  case "$out" in
    "exo "*) printf '%s' "${out#exo }" | tr -d '\r\n' ;;
    *) printf '' ;;
  esac
}
```

Marca ejecutable: `chmod +x plugins/exo/scripts/_engine-version.sh`.

- [ ] **Step 3: Verlo verde**

Run: `bash plugins/exo/scripts/test-engine-version.sh`
Expected: `8 passed, 0 failed`.

- [ ] **Step 4: `ENGINE_MIN` + gate en `test-versiones.sh`**

Crea `plugins/exo/ENGINE_MIN` con este contenido exacto (una línea, sin
comentarios: es un valor que un script bash va a leer con `cat`, no un
fichero de config):

```
0.1.0
```

`Edit` sobre `scripts/test-versiones.sh`:

old_string:
```bash
if [ -z "$engine" ]; then
  echo "[FAIL] no leo la versión de engine/Cargo.toml" >&2
  fallos=1
fi
if [ "$#" -ge 1 ] && [ "$1" != "v$engine" ]; then
  echo "[FAIL] el tag '$1' no casa con engine/Cargo.toml ($engine): el binario diría otra versión" >&2
  fallos=1
fi

[ "$fallos" -eq 0 ] && echo "[OK] engine $engine · plugin $plugin"
exit "$fallos"
```

new_string:
```bash
if [ -z "$engine" ]; then
  echo "[FAIL] no leo la versión de engine/Cargo.toml" >&2
  fallos=1
fi
if [ "$#" -ge 1 ] && [ "$1" != "v$engine" ]; then
  echo "[FAIL] el tag '$1' no casa con engine/Cargo.toml ($engine): el binario diría otra versión" >&2
  fallos=1
fi

# ENGINE_MIN (campaña H): el mínimo de engine que el plugin instalado declara
# necesitar (lo lee `exo doctor` del plugin en caché, y los hooks del propio
# binario en ejecución). Tiene que ser <= la versión real de engine — un
# ENGINE_MIN por delante de lo que el propio repo publica marcaría todo
# binario recién compilado como "viejo".
engine_min="$(tr -d '[:space:]' < plugins/exo/ENGINE_MIN 2>/dev/null || true)"
if [ -z "$engine_min" ]; then
  echo "[FAIL] no leo plugins/exo/ENGINE_MIN" >&2
  fallos=1
elif [ -n "$engine" ]; then
  . plugins/exo/scripts/_engine-version.sh
  if semver_lt "$engine" "$engine_min"; then
    echo "[FAIL] ENGINE_MIN ($engine_min) es MAYOR que engine/Cargo.toml ($engine): todo binario recién compilado se reportaría como viejo" >&2
    fallos=1
  fi
fi

[ "$fallos" -eq 0 ] && echo "[OK] engine $engine · plugin $plugin · ENGINE_MIN $engine_min"
exit "$fallos"
```

- [ ] **Step 5: Verlo verde**

Run: `bash scripts/test-versiones.sh`
Expected: `[OK] engine 0.1.0 · plugin 1.1.2 · ENGINE_MIN 0.1.0`.

Run (rojo-verde inverso, para probar que el gate SÍ detecta el caso malo):
`ENGINE_MIN=9.9.9 bash -c 'echo "$ENGINE_MIN" > /tmp/em-bad && cp plugins/exo/ENGINE_MIN /tmp/em-good && cp /tmp/em-bad plugins/exo/ENGINE_MIN && bash scripts/test-versiones.sh; ec=$?; cp /tmp/em-good plugins/exo/ENGINE_MIN; exit $ec'`
Expected: `[FAIL] ENGINE_MIN (9.9.9) es MAYOR que engine/Cargo.toml (0.1.0)…`,
exit 1 — y al terminar `plugins/exo/ENGINE_MIN` queda restaurado a `0.1.0`
(el propio comando lo hace).

- [ ] **Step 6: Commit**

```bash
git add plugins/exo/ENGINE_MIN plugins/exo/scripts/_engine-version.sh plugins/exo/scripts/test-engine-version.sh scripts/test-versiones.sh
git commit -m "feat(h, engine-min): contrato de version minima del engine — helper bash + gate en test-versiones.sh"
```

---

### Task 2: `exo-recall.sh` y `recall-inject.sh` detectan un engine viejo

**Lane:** mecánica. **Depende de:** Task 1 (usa `_engine-version.sh`).
**Oráculo:** `bash plugins/exo/scripts/test-exo-recall.sh` y
`bash plugins/exo/scripts/test-recall-inject.sh` verdes, con los casos
nuevos vistos en rojo antes de la implementación.

**Evidencia:** ninguno de los dos scripts comprueba hoy la versión del
binario que invoca — leído completo en este worktree,
`exo-recall.sh:56-102` solo distingue `no-engine`/`no-index`/`no-config`, y
`recall-inject.sh:113-123` solo `no-engine`/`no-index`. Un binario viejo que
sigue siendo ejecutable pasa esos guards y sirve lo que sea que su versión
vieja responda — degradación con forma válida, la clase de fallo que esta
campaña existe para cerrar.

**Files:**
- Modify: `plugins/exo/scripts/exo-recall.sh`
- Modify: `plugins/exo/scripts/recall-inject.sh`
- Test: `plugins/exo/scripts/test-exo-recall.sh`
- Test: `plugins/exo/scripts/test-recall-inject.sh`

**Interfaces:**
- Consumes: `semver_lt`, `exo_version_de` (Task 1, `_engine-version.sh`).
- Produces: evento `recall-fallback reason=engine-stale` (en
  `exo-recall.sh`, vía `log_recall_fallback`, mismo formato que
  `no-engine`/`no-index`/`no-config`) y `recall-inject-degraded
  reason=engine-stale` (en `recall-inject.sh`, vía `log_ri`). Cuando
  `exo-recall.sh` degrada por esto, `BASE` empieza con la línea «engine
  desactualizado (X < Y) — actualiza el binario instalado» antes del
  fallback embebido — el fallback deja de ser indistinguible de un simple
  «no hay config».

- [ ] **Step 1: Test que falla — `test-exo-recall.sh`**

Añade a `plugins/exo/scripts/test-exo-recall.sh`, justo antes de la línea
final `printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"`:

```bash
# ------------------- engine-stale: exo responde, pero por debajo de ENGINE_MIN
STUB_VIEJO="$TMP/exo-stub-viejo"
cat > "$STUB_VIEJO" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  --version) echo "exo 0.1.0" ;;
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: no debería llegar aquí." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_VIEJO"

: > "$LOGC"
touch "$TMP/index-viejo.db"
run_hook '{"session_id":"sess-stale"}' EXO_BIN="$STUB_VIEJO" EXO_INDEX="$TMP/index-viejo.db" ENGINE_MIN=9.9.9
EV_ST="$(ultimo_evento)"; PL_ST="$(ultimo_payload)"
CTX_ST="$(printf '%s' "$HOOK_OUT" | jq -r '.hookSpecificOutput.additionalContext' 2>/dev/null)"
if [ "$EV_ST" = "recall-fallback" ] && contains "$PL_ST" "reason=engine-stale" \
   && contains "$CTX_ST" "engine desactualizado"; then
  pass "engine-stale: version por debajo de ENGINE_MIN ⇒ recall-fallback + aviso en el bloque"
else
  fail "engine-stale: version por debajo de ENGINE_MIN ⇒ recall-fallback + aviso en el bloque" \
    "evento=$EV_ST payload=$PL_ST ctx='$CTX_ST'"
fi

# --- camino feliz de siempre sigue sin degradar (ENGINE_MIN por defecto, ----
# el binario feliz no responde --version -> exo_version_de da vacío, que
# CUENTA como stale: el stub feliz existente no declara --version, así que
# se le añade aquí para no romper el caso de arriba en el fichero real). ----
```

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: `[FAIL] engine-stale: …` — hoy no existe el guard, así que el stub
viejo pasa de largo por `no-engine`/`no-index`/`no-config` (los tres están
`ok`: el binario es ejecutable, el índice existe, `config --json` resuelve
el nombre) y llega a `recall`, que en este stub imprime el texto de aviso en
vez de degradar — el evento nunca se loguea.

Después del Step 1, el stub `STUB_FELIZ` ya existente en el fichero (más
abajo, camino feliz) también necesita responder `--version` con algo
≥ `ENGINE_MIN`, o el nuevo guard lo intercepta como stale y rompe el caso
que ya pasaba. `Edit` sobre el bloque existente:

old_string:
```bash
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"
```

new_string:
```bash
STUB_FELIZ="$TMP/exo-stub-feliz"
cat > "$STUB_FELIZ" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  --version) echo "exo 9.0.0" ;;
  config) echo '{"schema_version":2,"command":"config","data":{"kb":{"name":"kb-test","path":"/tmp/kb-test"}}}' ;;
  recall) echo "Contrato de memoria: bloque de prueba camino feliz." ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$STUB_FELIZ"
```

Y el `run_hook` del camino feliz pasa `ENGINE_MIN` explícito para no
depender de qué diga hoy `plugins/exo/ENGINE_MIN` real:

old_string:
```bash
run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db"
```

new_string:
```bash
run_hook '{"session_id":"sess-ok"}' EXO_BIN="$STUB_FELIZ" EXO_INDEX="$TMP/index-feliz.db" ENGINE_MIN=0.1.0
```

- [ ] **Step 2: Implementación — `exo-recall.sh`**

`Edit` sobre `plugins/exo/scripts/exo-recall.sh`:

old_string:
```bash
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
EXO_CAP="${EXO_RECALL_CAP:-6144}"
```

new_string:
```bash
EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
EXO_CAP="${EXO_RECALL_CAP:-6144}"
# ENGINE_MIN (campaña H): el mínimo de engine que ESTE plugin declara
# necesitar, sobreescribible por test (`ENGINE_MIN=x.y.z`, seam igual que
# EXO_BIN/EXO_INDEX de arriba). Sin override, el fichero versionado junto al
# plugin.
. "$SCRIPT_DIR/_engine-version.sh" 2>/dev/null
ENGINE_MIN="${ENGINE_MIN:-$(cat "$SCRIPT_DIR/../ENGINE_MIN" 2>/dev/null)}"
ENGINE_MIN="${ENGINE_MIN:-0.0.0}"
```

Este bloque tiene tres ramas hoy (`no-engine`/`no-index`/`else`) y necesita
una cuarta (`engine-stale`) sin romper la cadena `if`/`elif`/`else` en dos
piezas sueltas. Se edita en dos pasos sobre el mismo `if`: el primero
convierte el `else` de entrada en un `elif` con la condición de versión, y
el segundo añade el `else` nuevo (el caso stale) justo antes del `fi` que
cierra todo el bloque.

old_string:
```bash
BASE=""
if [ ! -x "$EXO_BIN" ]; then
  log_recall_fallback "no-engine" "bin=$EXO_BIN"
elif [ ! -f "$EXO_INDEX" ]; then
  log_recall_fallback "no-index" "db=$EXO_INDEX"
else
  # El nombre de la KB sale de la config del engine, no de un literal: era el
  # último sitio donde `kb-demo` seguía cableado en el camino de arranque.
  # Resuelto AQUÍ (binario ejecutable e índice ya confirmados arriba) y no
  # antes: moverlo antes de esos guards doblaba el log cuando la causa real
  # era `no-engine`/`no-index` — el mismo binario ausente que hace fallar
  # `exo recall` también hace fallar `exo config`, y `no-config` mentiría
  # sobre la causa.
```

new_string:
```bash
BASE=""
ENGINE_VER=""
if [ ! -x "$EXO_BIN" ]; then
  log_recall_fallback "no-engine" "bin=$EXO_BIN"
elif [ ! -f "$EXO_INDEX" ]; then
  log_recall_fallback "no-index" "db=$EXO_INDEX"
elif ENGINE_VER="$(exo_version_de "$EXO_BIN")" && [ -n "$ENGINE_VER" ] && ! semver_lt "$ENGINE_VER" "$ENGINE_MIN"; then
  # El nombre de la KB sale de la config del engine, no de un literal: era el
  # último sitio donde `kb-demo` seguía cableado en el camino de arranque.
  # Resuelto AQUÍ (binario ejecutable e índice ya confirmados arriba) y no
  # antes: moverlo antes de esos guards doblaba el log cuando la causa real
  # era `no-engine`/`no-index` — el mismo binario ausente que hace fallar
  # `exo recall` también hace fallar `exo config`, y `no-config` mentiría
  # sobre la causa.
```

Y cierra la rama nueva `engine-stale` justo antes del `fi` que hoy cierra el
`else` de arriba:

old_string:
```bash
    log_recall_fallback "no-contract"
    BASE=""
  fi
fi
[ -n "$BASE" ] || BASE="$FALLBACK"
```

new_string:
```bash
    log_recall_fallback "no-contract"
    BASE=""
  fi
else
  ENGINE_VER="${ENGINE_VER:-desconocida}"
  log_recall_fallback "engine-stale" "engine=$ENGINE_VER min=$ENGINE_MIN"
  BASE="engine desactualizado ($ENGINE_VER < $ENGINE_MIN) — actualiza el binario instalado

$FALLBACK"
fi
[ -n "$BASE" ] || BASE="$FALLBACK"
```

- [ ] **Step 3: Verlo verde**

Run: `bash plugins/exo/scripts/test-exo-recall.sh`
Expected: `7 passed, 0 failed` (los 5 casos de siempre + H5 + el nuevo
`engine-stale`; el conteo real depende de cuántos `pass`/`fail` haya en el
fichero al llegar aquí — el criterio es `0 failed`, no un número fijo).

- [ ] **Step 4: Test que falla — `test-recall-inject.sh`**

Lee primero `plugins/exo/scripts/test-recall-inject.sh` para localizar dónde
viven los casos `no-engine`/`no-index` existentes (mismo patrón de
`run_hook`/stub que `test-exo-recall.sh`) e inserta, justo después de esos
dos casos, uno nuevo:

```bash
# ------------------- engine-stale: exo responde, pero por debajo de ENGINE_MIN
STUB_VIEJO="$TMP/exo-stub-viejo"
cat > "$STUB_VIEJO" <<'EOF'
#!/usr/bin/env bash
case "$1" in --version) echo "exo 0.1.0" ;; *) exit 1 ;; esac
EOF
chmod +x "$STUB_VIEJO"
: > "$LOGC"
run_hook '{"session_id":"sess-stale","prompt":"revisa el modulo de busqueda"}' \
  EXO_BIN="$STUB_VIEJO" EXO_INDEX="$TMP/index-cualquiera.db" ENGINE_MIN=9.9.9
touch "$TMP/index-cualquiera.db"
EV_ST="$(ultimo_evento)"; PL_ST="$(ultimo_payload)"
if [ "$EV_ST" = "recall-inject-degraded" ] && contains "$PL_ST" "reason=engine-stale"; then
  pass "engine-stale: version por debajo de ENGINE_MIN ⇒ degraded reason=engine-stale"
else
  fail "engine-stale: version por debajo de ENGINE_MIN ⇒ degraded reason=engine-stale" \
    "evento=$EV_ST payload=$PL_ST"
fi
if [ -z "$HOOK_OUT" ]; then
  pass "engine-stale: sin salida (abstención, no inyecta nada plausible)"
else
  fail "engine-stale: sin salida (abstención, no inyecta nada plausible)" "salida='$HOOK_OUT'"
fi
```

Adapta los nombres (`ultimo_evento`, `ultimo_payload`, `run_hook`, `$LOGC`)
a los que de verdad use `test-recall-inject.sh` — si el fichero no define
esos helpers, reutiliza los que sí tenga (mismo patrón de
`tail -1 "$LOGC" | jq -r '.reflex'` que en `test-exo-recall.sh`) sin
introducir un tercer estilo de test en el repo.

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: `[FAIL] engine-stale: …` — el guard no existe todavía.

- [ ] **Step 5: Implementación — `recall-inject.sh`**

`Edit` sobre `plugins/exo/scripts/recall-inject.sh`:

old_string:
```bash
# --- Guards ------------------------------------------------------------------
if [ ! -x "$EXO_BIN" ]; then
  log_ri "degraded" "reason=no-engine bin=$EXO_BIN"
  exit 0
fi
if [ ! -f "$EXO_INDEX" ]; then
  # Sin índice NO se pasa `--refresh`: dispararía un bootstrap de minutos bajo
  # el timeout del evento. Se abstiene y deja rastro.
  log_ri "degraded" "reason=no-index db=$EXO_INDEX"
  exit 0
fi
```

new_string:
```bash
# --- Guards ------------------------------------------------------------------
if [ ! -x "$EXO_BIN" ]; then
  log_ri "degraded" "reason=no-engine bin=$EXO_BIN"
  exit 0
fi
if [ ! -f "$EXO_INDEX" ]; then
  # Sin índice NO se pasa `--refresh`: dispararía un bootstrap de minutos bajo
  # el timeout del evento. Se abstiene y deja rastro.
  log_ri "degraded" "reason=no-index db=$EXO_INDEX"
  exit 0
fi
# ENGINE_MIN (campaña H): mismo contrato que exo-recall.sh — sobreescribible
# por test, y si no, el fichero versionado junto al plugin.
. "$SCRIPT_DIR/_engine-version.sh" 2>/dev/null
ENGINE_MIN="${ENGINE_MIN:-$(cat "$SCRIPT_DIR/../ENGINE_MIN" 2>/dev/null)}"
ENGINE_MIN="${ENGINE_MIN:-0.0.0}"
ENGINE_VER="$(exo_version_de "$EXO_BIN" 2>/dev/null)"
if [ -z "$ENGINE_VER" ] || semver_lt "$ENGINE_VER" "$ENGINE_MIN"; then
  log_ri "degraded" "reason=engine-stale engine=${ENGINE_VER:-desconocida} min=$ENGINE_MIN"
  exit 0
fi
```

- [ ] **Step 6: Verlo verde**

Run: `bash plugins/exo/scripts/test-recall-inject.sh`
Expected: todos los casos en verde, incluido el nuevo `engine-stale`.

- [ ] **Step 7: Commit**

```bash
git add plugins/exo/scripts/exo-recall.sh plugins/exo/scripts/recall-inject.sh plugins/exo/scripts/test-exo-recall.sh plugins/exo/scripts/test-recall-inject.sh
git commit -m "fix(h, hooks): exo-recall.sh y recall-inject.sh degradan con rastro cuando el engine esta por debajo de ENGINE_MIN"
```

---

### Task 3: `exo doctor` — check `plugin_compat`

**Lane:** mecánica. **Depende de:** nada de código (usa el fichero
`ENGINE_MIN` de Task 1 solo como referencia de formato, no como dependencia
de compilación). **Oráculo:**
`cd engine && cargo test --release --test doctor` verde, con los tres casos
nuevos vistos en rojo antes de la implementación.

**Evidencia:** `doctor.rs::analiza` (líneas 157-175 de este worktree) corre
once checks; ninguno compara la versión del binario contra lo que el plugin
instalado declara necesitar. `script_del_plugin` (`:746-768`) ya sabe
recorrer `~/.claude/plugins/cache/exo/<familia>/<version>/`, pero solo para
resolver `kb-precommit.sh`, y ordena las versiones como texto (el bug que
Task 5 corrige reutilizando el comparador que esta tarea escribe).

**Files:**
- Modify: `engine/src/doctor.rs`
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Produces: `pub fn parse_semver(s: &str) -> Option<(u32, u32, u32)>`,
  `pub fn version_dir_mas_alta(base: &Path) -> Option<(PathBuf, (u32, u32, u32))>`
  (ambas nuevas, usadas también por Task 5) y el check `"plugin_compat"` en
  el vector de `analiza`.
- Consumes: `Entorno::home` (ya existe).

- [ ] **Step 1: Test que falla**

Añade a `engine/tests/doctor.rs`, al final del fichero:

```rust
/// Instala un `ENGINE_MIN` falso en el layout del plugin bajo `home`, en la
/// familia y versión dadas. Devuelve el directorio de esa versión.
fn plugin_con_engine_min(home: &Path, familia: &str, version: &str, engine_min: &str) -> PathBuf {
    let dir = home
        .join(".claude")
        .join("plugins")
        .join("cache")
        .join("exo")
        .join(familia)
        .join(version);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("ENGINE_MIN"), engine_min).unwrap();
    dir
}

#[test]
fn sin_plugin_instalado_plugin_compat_es_warn_no_fail() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(
        c.estado,
        Estado::Warn,
        "sin plugin no hay hooks que degradar, pero tampoco memoria"
    );
}

#[test]
fn plugin_con_engine_min_ya_satisfecho_es_ok() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    // "0.0.0" es <= a cualquier versión real del binario, sea cual sea hoy
    // engine/Cargo.toml: el test no depende de ese número.
    plugin_con_engine_min(&dir.path().join("home"), "exo", "1.0.0", "0.0.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(c.estado, Estado::Ok);
}

#[test]
fn plugin_con_engine_min_futuro_es_fail() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("kb")).unwrap();
    // "99.0.0" es mayor que cualquier versión real que este repo vaya a
    // publicar: garantiza el caso "binario viejo" sin acoplar el test al
    // valor actual de engine/Cargo.toml.
    plugin_con_engine_min(&dir.path().join("home"), "exo", "1.0.0", "99.0.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "plugin_compat");
    assert_eq!(c.estado, Estado::Fail);
    assert!(
        c.detalle.contains("99.0.0"),
        "dice qué ENGINE_MIN exige el plugin: {}",
        c.detalle
    );
}
```

Run: `cd engine && cargo test --release --test doctor plugin_compat`
Expected: `error[E0599]: no method named ... ` o, si compila con el check
ausente, `panic: el informe tiene que llevar el check plugin_compat` en las
tres pruebas — el check todavía no existe en `analiza`.

- [ ] **Step 2: Implementación**

`Edit` sobre `engine/src/doctor.rs`:

old_string:
```rust
pub fn analiza(entorno: &Entorno) -> InformeDoctor {
    // La config se carga UNA vez y se pasa a los checks que dependen de ella:
    // releerla por check daría informes internamente incoherentes si alguien
    // la edita a mitad de corrida.
    let cfg = crate::config::carga_desde(&entorno.config).ok();
    InformeDoctor::nuevo(vec![
        check_config(entorno),
        check_binario_en_path(entorno),
        check_fallback_del_hook(entorno),
        check_kb(entorno, cfg.as_ref()),
        check_indice(entorno, cfg.as_ref()),
        check_rutas_portables(entorno, cfg.as_ref()),
        check_modelo(entorno, cfg.as_ref()),
        check_jq(entorno),
        check_git_bash(entorno),
        check_detach(entorno),
        check_hook_precommit(entorno, cfg.as_ref()),
    ])
}
```

new_string:
```rust
pub fn analiza(entorno: &Entorno) -> InformeDoctor {
    // La config se carga UNA vez y se pasa a los checks que dependen de ella:
    // releerla por check daría informes internamente incoherentes si alguien
    // la edita a mitad de corrida.
    let cfg = crate::config::carga_desde(&entorno.config).ok();
    InformeDoctor::nuevo(vec![
        check_config(entorno),
        check_binario_en_path(entorno),
        check_fallback_del_hook(entorno),
        check_kb(entorno, cfg.as_ref()),
        check_indice(entorno, cfg.as_ref()),
        check_rutas_portables(entorno, cfg.as_ref()),
        check_modelo(entorno, cfg.as_ref()),
        check_jq(entorno),
        check_git_bash(entorno),
        check_detach(entorno),
        check_hook_precommit(entorno, cfg.as_ref()),
        check_plugin_compat(entorno),
    ])
}

/// Parsea `"X.Y.Z"` a una tupla comparable por orden natural. `None` si no
/// tiene esa forma exacta (tres componentes numéricos separados por punto) —
/// un directorio que no es una versión (basura, `.DS_Store`) se descarta en
/// vez de reventar el sort. Sin dependencia nueva: `semver` es una crate más
/// para lo mismo que tres `parse::<u32>()`.
pub fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let mut partes = s.trim().split('.');
    let mayor: u32 = partes.next()?.parse().ok()?;
    let menor: u32 = partes.next()?.parse().ok()?;
    let parche: u32 = partes.next()?.parse().ok()?;
    if partes.next().is_some() {
        return None;
    }
    Some((mayor, menor, parche))
}

/// El subdirectorio de versión MÁS ALTA bajo `base` (cada entrada es un
/// directorio `X.Y.Z`, el layout de `~/.claude/plugins/cache/exo/<familia>/`).
/// Compara semver real, no la cadena: `"1.10.0"` < `"1.9.0"` como texto,
/// pero es la versión MAYOR — el bug que tenía `script_del_plugin` antes de
/// esta campaña (Task 5 lo reutiliza para corregirlo). `None` si `base` no
/// existe o no contiene ningún directorio con nombre de versión válido.
pub fn version_dir_mas_alta(base: &Path) -> Option<(PathBuf, (u32, u32, u32))> {
    let entradas = std::fs::read_dir(base).ok()?;
    entradas
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let nombre = e.file_name().to_string_lossy().into_owned();
            parse_semver(&nombre).map(|v| (e.path(), v))
        })
        .max_by_key(|(_, v)| *v)
}

/// `ENGINE_MIN` es el fichero de una línea (`plugins/exo/ENGINE_MIN` en el
/// repo, copiado tal cual al instalar) donde el PLUGIN declara la versión
/// mínima de engine con la que fue probado. Este check compara ESE número
/// contra `env!("CARGO_PKG_VERSION")` — la versión de ESTE binario, fijada
/// en compilación — para detectar el caso que motiva la campaña H: un
/// plugin actualizado (que ya no lleva los alias españoles retirados en
/// 0.2.0, por ejemplo) corriendo contra un binario que se quedó atrás.
fn check_plugin_compat(entorno: &Entorno) -> Check {
    let base = entorno
        .home
        .join(".claude")
        .join("plugins")
        .join("cache")
        .join("exo")
        .join("exo");
    let Some((dir, version)) = version_dir_mas_alta(&base) else {
        return Check::nuevo(
            "plugin_compat",
            Estado::Warn,
            base.display().to_string(),
            "no encuentro el plugin exo instalado — sin plugin no hay hooks \
             que puedan degradar, pero tampoco recall automático",
        );
    };
    let ruta_min = dir.join("ENGINE_MIN");
    let declarado = std::fs::read_to_string(&ruta_min).unwrap_or_default();
    let declarado = declarado.trim();
    let Some(min) = parse_semver(declarado) else {
        return Check::nuevo(
            "plugin_compat",
            Estado::Warn,
            ruta_min.display().to_string(),
            "el plugin instalado no lleva un ENGINE_MIN legible (versión \
             anterior a esta campaña) — no puedo comparar",
        );
    };
    let (va, vb, vc) = version;
    let propia_str = env!("CARGO_PKG_VERSION");
    let propia = parse_semver(propia_str)
        .expect("CARGO_PKG_VERSION de este crate siempre es X.Y.Z (engine/Cargo.toml)");
    let artefacto = format!(
        "binario {propia_str} · plugin {va}.{vb}.{vc} exige >= {declarado} ({})",
        ruta_min.display()
    );
    if propia < min {
        Check::nuevo(
            "plugin_compat",
            Estado::Fail,
            artefacto,
            format!(
                "este binario ({propia_str}) es más viejo que lo que el \
                 plugin instalado declara necesitar ({declarado}) — los \
                 hooks pueden degradar con forma válida. Actualiza el \
                 binario a >= {declarado}"
            ),
        )
    } else {
        Check::nuevo(
            "plugin_compat",
            Estado::Ok,
            artefacto,
            "el binario cumple el ENGINE_MIN que declara el plugin instalado",
        )
    }
}
```

- [ ] **Step 3: Verlo verde**

Run: `cd engine && cargo test --release --test doctor`
Expected: `test result: ok` — incluye las tres pruebas nuevas y las ~30
existentes del fichero (ninguna otra debe cambiar de estado).

- [ ] **Step 4: Commit**

```bash
git add engine/src/doctor.rs engine/tests/doctor.rs
git commit -m "feat(h, doctor): check plugin_compat — detecta un binario mas viejo que el ENGINE_MIN del plugin instalado"
```

---

### Task 4: `check_git_bash` deja de dar `ok` con el `bash` de WSL

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`cd engine && cargo test --release --test doctor git_bash` verde.

**Evidencia:** `check_git_bash` (`doctor.rs:559-582`) hace
`busca_en_path(&entorno.path, "bash")` y da `Ok` con lo primero que
encuentre. En un PATH típico de PowerShell/W11, eso resuelve
`C:\Windows\System32\bash.exe` — el shim de WSL, no Git Bash — antes que
`C:\Program Files\Git\bin\bash.exe`, si System32 aparece antes en el PATH
(el caso medido en el runbook de W11 citado en el propio doctor.rs).

**Files:**
- Modify: `engine/src/doctor.rs`
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Produces: `pub fn ruta_sugiere_git_bash(ruta: &Path) -> bool` y
  `pub fn salida_indica_git_bash(salida: &str) -> bool` — funciones puras,
  testables sin lanzar ningún proceso ni fabricar un binario real por
  plataforma.

- [ ] **Step 1: Test que falla**

Añade a `engine/tests/doctor.rs`, al final:

```rust
#[test]
fn ruta_de_wsl_no_sugiere_git_bash() {
    assert!(!exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Windows\System32\bash.exe"
    )));
}

#[test]
fn ruta_bajo_git_for_windows_sugiere_git_bash_sin_ejecutar_nada() {
    assert!(exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Program Files\Git\bin\bash.exe"
    )));
    assert!(exo::doctor::ruta_sugiere_git_bash(Path::new(
        r"C:\Program Files\Git\usr\bin\bash.exe"
    )));
}

#[test]
fn version_de_msys_se_reconoce_como_git_bash() {
    assert!(exo::doctor::salida_indica_git_bash(
        "GNU bash, version 5.2.26(1)-release (x86_64-pc-msys)"
    ));
}

#[test]
fn version_de_wsl_no_se_reconoce_como_git_bash() {
    assert!(!exo::doctor::salida_indica_git_bash(
        "GNU bash, version 5.1.16(1)-release (x86_64-pc-linux-gnu)"
    ));
}

#[cfg(windows)]
#[test]
fn bash_resuelto_bajo_system32_es_warn_no_ok() {
    let dir = tempfile::tempdir().unwrap();
    let bindir = dir.path().join("System32");
    fs::create_dir_all(&bindir).unwrap();
    // No hace falta un bash.exe real: la ruta ya lo descarta (System32) y el
    // intento de ejecutarlo fallará (no es un ejecutable válido), lo que
    // `es_git_bash` trata igual que "no dijo msys/mingw".
    fs::write(bindir.join("bash.exe"), b"no es un binario de verdad").unwrap();
    let mut env = entorno(dir.path());
    env.path = bindir.display().to_string();
    let informe = analiza(&env);
    let c = check(&informe, "git_bash");
    assert_eq!(c.estado, Estado::Warn);
    assert!(
        c.detalle.contains("WSL"),
        "dice que lo que resolvió es WSL, no Git Bash: {}",
        c.detalle
    );
}
```

Run: `cd engine && cargo test --release --test doctor git_bash`
Expected: `error[E0425]: cannot find function ruta_sugiere_git_bash` (no
existe todavía) — o, si se comenta esa parte, la última prueba falla con
`assert_eq!(c.estado, Estado::Warn)` porque hoy da `Ok`.

- [ ] **Step 2: Implementación**

`Edit` sobre `engine/src/doctor.rs`:

old_string:
```rust
fn check_git_bash(entorno: &Entorno) -> Check {
    if !cfg!(windows) {
        return Check::nuevo(
            "git_bash",
            Estado::Na,
            format!("plataforma={}", std::env::consts::OS),
            "solo se mide en Windows: fuera de ahí el shell de los hooks ya es bash",
        );
    }
    match busca_en_path(&entorno.path, "bash") {
        Some(ruta) => Check::nuevo(
            "git_bash",
            Estado::Ok,
            ruta.display().to_string(),
            "Claude Code puede correr los hooks .sh del plugin",
        ),
        None => Check::nuevo(
            "git_bash",
            Estado::Fail,
            format!("PATH={}", entorno.path),
            "sin Git Bash los hooks .sh del plugin no corren",
        ),
    }
}
```

new_string:
```rust
/// Heurística barata: la ruta ya delata Git Bash sin lanzar ningún proceso.
/// El `bash.exe` de WSL vive literalmente bajo `System32`; el de Git for
/// Windows, bajo un directorio `Git\`.
pub fn ruta_sugiere_git_bash(ruta: &Path) -> bool {
    let s = ruta.to_string_lossy().to_lowercase();
    s.contains("git") && !s.contains("system32")
}

/// El bash de WSL responde a `--version` como un GNU bash normal de Linux;
/// el de Git Bash (msys2) declara su propia plataforma en la misma línea
/// (`x86_64-pc-msys` / `mingw`). Función pura: sin esto, probar la rama que
/// SÍ lanza el proceso exigiría fabricar un `bash.exe` real por plataforma.
pub fn salida_indica_git_bash(salida: &str) -> bool {
    let s = salida.to_lowercase();
    s.contains("msys") || s.contains("mingw")
}

/// `ruta` es un Git Bash de verdad: la vía barata (heurística de ruta)
/// primero, y solo si es inconcluyente se le pregunta con `--version`. Un
/// `Err` al ejecutar (ruta no es un binario válido) cuenta como "no lo es",
/// nunca como pánico.
fn es_git_bash(ruta: &std::path::Path) -> bool {
    if ruta_sugiere_git_bash(ruta) {
        return true;
    }
    match std::process::Command::new(ruta).arg("--version").output() {
        Ok(o) => salida_indica_git_bash(&String::from_utf8_lossy(&o.stdout)),
        Err(_) => false,
    }
}

fn check_git_bash(entorno: &Entorno) -> Check {
    if !cfg!(windows) {
        return Check::nuevo(
            "git_bash",
            Estado::Na,
            format!("plataforma={}", std::env::consts::OS),
            "solo se mide en Windows: fuera de ahí el shell de los hooks ya es bash",
        );
    }
    match busca_en_path(&entorno.path, "bash") {
        Some(ruta) if es_git_bash(&ruta) => Check::nuevo(
            "git_bash",
            Estado::Ok,
            ruta.display().to_string(),
            "Claude Code puede correr los hooks .sh del plugin",
        ),
        Some(ruta) => Check::nuevo(
            "git_bash",
            Estado::Warn,
            ruta.display().to_string(),
            "resuelve a un bash que no es Git Bash (probablemente WSL): los \
             hooks .sh de Claude Code esperan Git Bash — instala Git for \
             Windows y ponlo antes en el PATH",
        ),
        None => Check::nuevo(
            "git_bash",
            Estado::Fail,
            format!("PATH={}", entorno.path),
            "sin Git Bash los hooks .sh del plugin no corren",
        ),
    }
}
```

- [ ] **Step 3: Verlo verde**

Run: `cd engine && cargo test --release --test doctor`
Expected: `test result: ok` — incluidas las 5 pruebas nuevas. La prueba
existente `git_bash_sale_na_fuera_de_windows_y_no_desaparece_del_informe`
sigue verde sin tocarla (su rama `cfg!(windows)` con PATH vacío sigue dando
`Fail`, no `Warn`: `busca_en_path` no encuentra nada, ni siquiera llega a
`es_git_bash`).

- [ ] **Step 4: Commit**

```bash
git add engine/src/doctor.rs engine/tests/doctor.rs
git commit -m "fix(h, doctor): check_git_bash ya no da ok con el bash de WSL — exige Git Bash de verdad"
```

---

### Task 5: `script_del_plugin` ordena por semver real, no por texto

**Lane:** mecánica. **Depende de:** Task 3 (reutiliza `parse_semver`).
**Oráculo:** `cd engine && cargo test --release --test doctor
script_del_plugin` verde, con el caso `1.9.0`/`1.10.0` visto en rojo antes
de la implementación.

**Evidencia:** `script_del_plugin` (`doctor.rs:746-768`) hace
`candidatos.sort(); candidatos.pop()` sobre las RUTAS COMPLETAS de los
`kb-precommit.sh` encontrados — un `sort()` de cadenas. Con versiones
`1.9.0` y `1.10.0` instaladas a la vez, `".../1.10.0/scripts/..."` es
lexicográficamente MENOR que `".../1.9.0/scripts/..."` (comparando
carácter a carácter, `'1' < '9'` en el segundo componente), así que
`.pop()` devuelve la ruta de `1.9.0` — la versión más VIEJA, no la más
nueva. El propio comentario de la función admite que esto es un `sort`
lexicográfico deliberado «porque basta con que alguna resuelva»: ese
argumento deja de sostenerse en cuanto Task 3 escribe, tres funciones más
arriba en el mismo fichero, el comparador de semver correcto para el mismo
propósito (decidir cuál es la versión más alta instalada).

**Files:**
- Modify: `engine/src/doctor.rs`
- Test: `engine/tests/doctor.rs`

**Interfaces:**
- Consumes: `parse_semver` (Task 3).

- [ ] **Step 1: Test que falla**

Añade a `engine/tests/doctor.rs`, al final:

```rust
#[test]
fn con_1_9_0_y_1_10_0_en_cache_elige_la_1_10_0() {
    let dir = tempfile::tempdir().unwrap();
    shim_precommit(&dir.path().join("kb"), SHIM_DE_LA_KB);
    plugin_con_script(&dir.path().join("home"), "exo", "1.9.0");
    let script_alto = plugin_con_script(&dir.path().join("home"), "exo", "1.10.0");
    let informe = analiza(&entorno_con_config(dir.path()));
    let c = check(&informe, "kb_precommit_hook");
    assert_eq!(c.estado, Estado::Ok);
    assert!(
        c.artefacto.contains(&script_alto.display().to_string()),
        "debía resolver a 1.10.0 (la más alta), no a 1.9.0 por orden de \
         texto: {}",
        c.artefacto
    );
}
```

Run: `cd engine && cargo test --release --test doctor con_1_9_0_y_1_10_0`
Expected: FAIL — `c.artefacto` contiene la ruta de `1.9.0`, no la de
`1.10.0` (el bug de ordenar por texto).

- [ ] **Step 2: Implementación**

`Edit` sobre `engine/src/doctor.rs`:

old_string:
```rust
/// ¿A qué `kb-precommit.sh` resolvería el shim? Replica el glob del shim real
/// instalado en la KB: el plugin `exo` primero y el `reflex` viejo como
/// fallback declarado del cutover. Devuelve la ruta y si viene de `exo`.
///
/// El shim se queda con la versión más alta por `sort -V`; aquí basta con que
/// **alguna** resuelva, así que se ordena lexicográficamente y se toma la
/// última. La diferencia importaría para decir QUÉ versión corre, no para
/// decir si el gate puede correr, que es lo que este check afirma.
///
/// Sin la crate `glob`: dos `read_dir` no pagan una dependencia.
fn script_del_plugin(home: &std::path::Path) -> Option<(PathBuf, bool)> {
    for (familia, es_exo) in [("exo", true), ("reflex", false)] {
        let base = home
            .join(".claude")
            .join("plugins")
            .join("cache")
            .join("exo")
            .join(familia);
        let Ok(entradas) = std::fs::read_dir(&base) else {
            continue;
        };
        let mut candidatos: Vec<PathBuf> = entradas
            .flatten()
            .map(|e| e.path().join("scripts").join("kb-precommit.sh"))
            .filter(|p| p.is_file())
            .collect();
        candidatos.sort();
        if let Some(ultimo) = candidatos.pop() {
            return Some((ultimo, es_exo));
        }
    }
    None
}
```

new_string:
```rust
/// ¿A qué `kb-precommit.sh` resolvería el shim? Replica el glob del shim real
/// instalado en la KB: el plugin `exo` primero y el `reflex` viejo como
/// fallback declarado del cutover. Devuelve la ruta y si viene de `exo`.
///
/// El shim real se queda con la versión más alta por `sort -V`: esto lo
/// replica de verdad (campaña H) reutilizando `parse_semver`/comparación
/// numérica de Task 3, en vez del `sort()` lexicográfico de texto que este
/// fichero tenía antes — `"1.10.0" < "1.9.0"` como cadena, al revés que como
/// versión, y con dos versiones instaladas a la vez elegía la vieja.
///
/// Sin la crate `glob`: `read_dir` no paga una dependencia.
fn script_del_plugin(home: &std::path::Path) -> Option<(PathBuf, bool)> {
    for (familia, es_exo) in [("exo", true), ("reflex", false)] {
        let base = home
            .join(".claude")
            .join("plugins")
            .join("cache")
            .join("exo")
            .join(familia);
        let Ok(entradas) = std::fs::read_dir(&base) else {
            continue;
        };
        let candidato = entradas
            .flatten()
            .filter_map(|e| {
                let nombre = e.file_name().to_string_lossy().into_owned();
                let v = parse_semver(&nombre)?;
                let script = e.path().join("scripts").join("kb-precommit.sh");
                script.is_file().then_some((script, v))
            })
            .max_by_key(|(_, v)| *v)
            .map(|(script, _)| script);
        if let Some(script) = candidato {
            return Some((script, es_exo));
        }
    }
    None
}
```

- [ ] **Step 3: Verlo verde, y sin regresión**

Run: `cd engine && cargo test --release --test doctor`
Expected: `test result: ok` — incluida la prueba nueva y las que ya usaban
`script_del_plugin` (`un_shim_que_resuelve_al_plugin_exo_es_ok_y_reporta_el_script`,
`un_shim_que_solo_encuentra_el_plugin_viejo_avisa`, etc., que usan una sola
versión por familia y no dependen del orden).

- [ ] **Step 4: Commit**

```bash
git add engine/src/doctor.rs engine/tests/doctor.rs
git commit -m "fix(h, doctor): script_del_plugin ordena por semver real — 1.10.0 gana a 1.9.0"
```

---

### Task 6: `kb-precommit.sh` fail-closed

**Lane:** mecánica. **Depende de:** nada. **Oráculo:**
`bash plugins/exo/scripts/test-kb-precommit.sh` verde, con el caso
fail-closed visto en rojo antes de la implementación.

**Evidencia:** `kb-precommit.sh:20` (este worktree):
`[ -x "$EXO" ] || { echo "…commit permitido sin gate" >&2; exit 0; }` —
sin `exo` instalado, el commit pasa igual, sin gate, con un aviso que nadie
mira en un pre-commit hook silencioso.

**Files:**
- Modify: `plugins/exo/scripts/kb-precommit.sh`
- Test: `plugins/exo/scripts/test-kb-precommit.sh` (nuevo)

**Interfaces:**
- Produces: exit 1 (antes 0) cuando `$EXO` no es ejecutable, con tres
  líneas de remedio en stderr: instalar el binario, apuntar `EXO_BIN=`, o
  `git commit --no-verify` como escape consciente.

- [ ] **Step 1: Test que falla**

Crea `plugins/exo/scripts/test-kb-precommit.sh`:

```bash
#!/usr/bin/env bash
# Test standalone para kb-precommit.sh. Repo git real en mktemp -d: el script
# llama `git rev-parse --show-toplevel` y `checkout-index`, que exigen un
# repo de verdad, no un fixture de texto.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="${SCRIPT_DIR}/kb-precommit.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }
contains() { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }

KB="$TMP/kb"
mkdir -p "$KB"
git -C "$KB" init -q
git -C "$KB" -c user.email=t@t.local -c user.name=t commit -q --allow-empty -m init

# ------------------- fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO ---
OUT="$(cd "$KB" && EXO_BIN="$TMP/no-existe-exo" "$HOOK" 2>&1)"; RC=$?
if [ "$RC" -eq 1 ] && contains "$OUT" "BLOQUEADO" && contains "$OUT" "--no-verify"; then
  pass "fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO, menciona --no-verify"
else
  fail "fail-closed: sin exo ejecutable ⇒ exit 1, BLOQUEADO, menciona --no-verify" "rc=$RC out=$OUT"
fi

# ------------------- camino feliz: exo stub que siempre pasa los gates -----
STUB="$TMP/exo-stub-ok"
cat > "$STUB" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$STUB"
OUT2="$(cd "$KB" && EXO_BIN="$STUB" "$HOOK" 2>&1)"; RC2=$?
if [ "$RC2" -eq 0 ]; then
  pass "camino feliz: exo presente y los dos gates pasan ⇒ exit 0"
else
  fail "camino feliz: exo presente y los dos gates pasan ⇒ exit 0" "rc=$RC2 out=$OUT2"
fi

# ------------------- gate real rechaza (ratchet) ⇒ exit 1, sin cambiar -----
STUB_FAIL="$TMP/exo-stub-fail"
cat > "$STUB_FAIL" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  ratchet) echo "ratchet: techo subido" >&2; exit 3 ;;
  budget) exit 0 ;;
  *) exit 0 ;;
esac
EOF
chmod +x "$STUB_FAIL"
OUT3="$(cd "$KB" && EXO_BIN="$STUB_FAIL" "$HOOK" 2>&1)"; RC3=$?
if [ "$RC3" -eq 1 ] && contains "$OUT3" "QUÉ HACER"; then
  pass "gate real rechaza (ratchet) ⇒ exit 1, mensaje QUÉ HACER (sin cambios)"
else
  fail "gate real rechaza (ratchet) ⇒ exit 1, mensaje QUÉ HACER (sin cambios)" "rc=$RC3 out=$OUT3"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
```

Marca ejecutable y corre:

```bash
chmod +x plugins/exo/scripts/test-kb-precommit.sh
bash plugins/exo/scripts/test-kb-precommit.sh
```

Expected: `[FAIL] fail-closed: …` — `rc=0`, sin `BLOQUEADO` ni `--no-verify`
en la salida (el hook de hoy sale 0 con un aviso distinto). Los otros dos
casos ya pasan (documentan comportamiento que esta tarea no toca).

- [ ] **Step 2: Implementación**

`Edit` sobre `plugins/exo/scripts/kb-precommit.sh`:

old_string:
```bash
KB="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0
EXO="${EXO_BIN:-$HOME/.local/bin/exo}"

[ -x "$EXO" ] || { echo "kb-precommit: no encuentro exo en $EXO — instálalo con 'cargo build --release' en engine/ y copia el binario a \$HOME/.local/bin/exo(.exe) — commit permitido sin gate" >&2; exit 0; }
```

new_string:
```bash
KB="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0
EXO="${EXO_BIN:-$HOME/.local/bin/exo}"

# Fail-closed (campaña H, docs/backlog.md "kb-precommit.sh depende de que
# exo esté instalado — si no, el gate degrada a 'commit permitido' en
# silencio"): antes esto salía 0 con un aviso que nadie mira en un
# pre-commit. El escape es explícito y consciente: `git commit --no-verify`.
if [ ! -x "$EXO" ]; then
  echo "kb-precommit: no encuentro un exo ejecutable en \$EXO_BIN ni en $EXO — commit BLOQUEADO (el gate es fail-closed)." >&2
  echo "  1) instala el binario: 'cargo build --release' en engine/ y copia a \$HOME/.local/bin/exo(.exe)" >&2
  echo "  2) o apunta a uno ya instalado: EXO_BIN=<ruta> git commit ..." >&2
  echo "  3) si de verdad quieres saltarte el gate: git commit --no-verify (escape consciente, el commit queda sin verificar)" >&2
  exit 1
fi
```

- [ ] **Step 3: Verlo verde**

Run: `bash plugins/exo/scripts/test-kb-precommit.sh`
Expected: `3 passed, 0 failed`.

- [ ] **Step 4: Commit**

```bash
git add plugins/exo/scripts/kb-precommit.sh plugins/exo/scripts/test-kb-precommit.sh
git commit -m "fix(h, kb-precommit): fail-closed sin exo instalado — exit 1 en vez de commit permitido en silencio"
```

---

### Task 7: Retirar los 10 alias españoles — engine 0.2.0, plugin 1.2.0

**Lane:** mecánica, pero es la ruptura de compatibilidad de la campaña —
va DESPUÉS de las Tasks 1-3 (el mecanismo de `ENGINE_MIN` tiene que existir
antes de que haya algo que de verdad rompa). **Depende de:** Task 1 (sube
`ENGINE_MIN`). **Oráculo:** `cd engine && cargo test --release --test flags`
verde, con el test nuevo visto en rojo antes de la implementación; y
`bash scripts/test-versiones.sh` verde tras el bump.

**Evidencia:** `git grep -n 'alias = "' engine/src/main.rs` da exactamente
10 líneas (`:153,185,235,247,257,284,297,304,311,315` en este worktree).
Grep exhaustivo de los ocho flags españoles sobre `plugins/`, `docs/`,
`scripts/`, `engine/tests/`, `.github/workflows/` (hecho para este plan,
ver «Verificación de la propuesta» arriba): el único consumidor es
`engine/tests/flags.rs`, en el propio test que existe para probar el alias.
Ningún script del plugin, skill, doc o workflow de CI usa una forma
española — no hay migración de consumidores que hacer fuera de ese test.

**Files:**
- Modify: `engine/src/main.rs`
- Modify: `engine/tests/flags.rs`
- Modify: `engine/Cargo.toml`
- Modify: `plugins/exo/ENGINE_MIN`
- Modify: `plugins/exo/.claude-plugin/plugin.json`
- Modify: `.claude-plugin/marketplace.json`
- Create: `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`

**Interfaces:**
- Produces: `exo --version` → `"exo 0.2.0"` (clap deriva de
  `engine/Cargo.toml`, sin tocar `main.rs` más allá del bump del propio
  `Cargo.toml`); los 10 flags españoles pasan a dar exit 2 de clap
  (`unexpected argument`), igual que cualquier flag inexistente.

- [ ] **Step 1: Test que falla**

`Edit` sobre `engine/tests/flags.rs` — añade el test nuevo justo antes del
que se va a borrar en el Step 2 (así el diff del Step 2 es limpio: quitar
uno, dejar el otro):

old_string:
```rust
#[test]
fn los_flags_espanoles_siguen_parseando_como_alias() {
```

new_string:
```rust
#[test]
fn los_flags_espanoles_ya_no_parsean_ni_como_alias_oculto() {
    // 0.2.0 retira los diez alias ocultos de la 0.1.0 (docs/backlog.md,
    // "Retirar los aliases españoles del CLI"): un script viejo que use
    // --limite/--titulo/etc. tiene que fallar con el mismo "unexpected
    // argument" que cualquier otro flag inexistente, no colarse en silencio.
    assert!(
        !acepta_el_flag(&["write", "new", "--dir", "d", "--titulo", "T", "--from", "-"]),
        "--titulo ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["write", "append", "--from", "-", "--crea", "p"]),
        "--crea ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["search", "--limite", "3", "q"]),
        "--limite (search) ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["search", "--min-similitud", "0.4", "q"]),
        "--min-similitud (search) ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["search", "--escala-fts", "0.6", "q"]),
        "--escala-fts ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["recall", "--limite", "3"]),
        "--limite (recall) ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["recall", "--min-similitud", "0.4"]),
        "--min-similitud (recall) ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["recall", "--contenido"]),
        "--contenido ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["recall", "--nota", "x/y"]),
        "--nota ya no debe parsear"
    );
    assert!(
        !acepta_el_flag(&["recall", "--refresca"]),
        "--refresca ya no debe parsear"
    );
}

#[test]
fn los_flags_espanoles_siguen_parseando_como_alias() {
```

Run: `cd engine && cargo test --release --test flags los_flags_espanoles_ya_no_parsean`
Expected: FAIL en la primera aserción (`--titulo` sigue parseando hoy, los
alias todavía están en `main.rs`).

- [ ] **Step 2: Borra el test que prueba el comportamiento viejo**

Ahora que el test nuevo (rojo) demuestra qué falta, borra el que probaba lo
contrario — dejarlos los dos sería una contradicción permanente, y es
justo lo que el propio ítem del backlog advertía («si no, el borrado se ve
rojo y alguien "arregla" el test reponiendo el alias»).

`Edit` sobre `engine/tests/flags.rs`:

old_string:
```rust
#[test]
fn los_flags_espanoles_siguen_parseando_como_alias() {
    // La ventana de migración: un script viejo cacheado no debe morir con
    // "unexpected argument" a mitad de un hook. Los diez pares, no una
    // muestra: un alias borrado o mal escrito en cualquiera de los diez
    // reproduce ese fallo, y solo se detecta probándolos todos.
    assert!(
        acepta_el_flag(&["write", "new", "--dir", "d", "--titulo", "T", "--from", "-"]),
        "alias --titulo"
    );
    assert!(
        acepta_el_flag(&["write", "append", "--from", "-", "--crea", "p"]),
        "alias --crea"
    );
    assert!(
        acepta_el_flag(&["search", "--limite", "3", "q"]),
        "alias --limite (search)"
    );
    assert!(
        acepta_el_flag(&["search", "--min-similitud", "0.4", "q"]),
        "alias --min-similitud (search)"
    );
    assert!(
        acepta_el_flag(&["search", "--escala-fts", "0.6", "q"]),
        "alias --escala-fts"
    );
    assert!(
        acepta_el_flag(&["recall", "--limite", "3"]),
        "alias --limite (recall)"
    );
    assert!(
        acepta_el_flag(&["recall", "--min-similitud", "0.4"]),
        "alias --min-similitud (recall)"
    );
    assert!(
        acepta_el_flag(&["recall", "--contenido", "--nota", "x/y"]),
        "alias --contenido/--nota"
    );
    assert!(
        acepta_el_flag(&["recall", "--refresca"]),
        "alias --refresca"
    );
}

/// `--help` del subcomando dado, como texto.
```

new_string:
```rust
/// `--help` del subcomando dado, como texto.
```

También actualiza la cabecera del fichero (ya no hay ventana de migración):

old_string:
```rust
//! Superficie CLI v1.0: los flags largos están en inglés y los españoles
//! siguen parseando como alias oculto durante la ventana de migración.
```

new_string:
```rust
//! Superficie CLI 0.2.0: los flags largos están en inglés. Los diez alias
//! españoles de la ventana de migración de 0.1.0 se retiraron en esta
//! versión (campaña H) — un flag español ya es tan inexistente como
//! cualquier otro que nunca haya existido.
```

- [ ] **Step 3: Implementación — retira los 10 `alias =` de `main.rs`**

`Edit` sobre `engine/src/main.rs`, ocho ediciones (una de ellas,
`min-similarity`/`min-similitud`, se repite idéntica en `search` y en
`recall`: usa `replace_all: true` solo en esa):

old_string:
```rust
    #[arg(long = "title", alias = "titulo", value_name = "TITLE")]
    titulo: String,
```
new_string:
```rust
    #[arg(long = "title", value_name = "TITLE")]
    titulo: String,
```

old_string:
```rust
    #[arg(long = "create", alias = "crea")]
    crea: bool,
```
new_string:
```rust
    #[arg(long = "create")]
    crea: bool,
```

old_string:
```rust
    #[arg(
        long = "limit",
        alias = "limite",
        value_name = "LIMIT",
        default_value_t = 10
    )]
    limite: usize,
```
new_string:
```rust
    #[arg(long = "limit", value_name = "LIMIT", default_value_t = 10)]
    limite: usize,
```

old_string:
```rust
    #[arg(
        long = "limit",
        alias = "limite",
        value_name = "LIMIT",
        default_value_t = 5
    )]
    limite: usize,
```
new_string:
```rust
    #[arg(long = "limit", value_name = "LIMIT", default_value_t = 5)]
    limite: usize,
```

old_string (aparece dos veces IDÉNTICO — `search` y `recall` — usa
`replace_all: true`):
```rust
    #[arg(
        long = "min-similarity",
        alias = "min-similitud",
        value_name = "MIN_SIMILARITY"
    )]
    min_similitud: Option<f64>,
```
new_string:
```rust
    #[arg(long = "min-similarity", value_name = "MIN_SIMILARITY")]
    min_similitud: Option<f64>,
```

old_string:
```rust
    #[arg(long = "fts-scale", alias = "escala-fts", value_name = "FTS_SCALE")]
    escala_fts: Option<f64>,
```
new_string:
```rust
    #[arg(long = "fts-scale", value_name = "FTS_SCALE")]
    escala_fts: Option<f64>,
```

old_string:
```rust
    #[arg(long = "content", alias = "contenido")]
    contenido: bool,
```
new_string:
```rust
    #[arg(long = "content")]
    contenido: bool,
```

old_string:
```rust
    #[arg(long = "note", alias = "nota", value_name = "NOTE")]
    nota: Option<String>,
```
new_string:
```rust
    #[arg(long = "note", value_name = "NOTE")]
    nota: Option<String>,
```

old_string:
```rust
    #[arg(long = "refresh", alias = "refresca")]
    refresca: bool,
```
new_string:
```rust
    #[arg(long = "refresh")]
    refresca: bool,
```

Verifica que los diez desaparecieron:

Run: `grep -c 'alias = "' engine/src/main.rs`
Expected: `0`.

- [ ] **Step 4: Verlo verde**

Run: `cd engine && cargo build --release 2>&1 | tail -20`
Expected: compila sin errores ni warnings nuevos.

Run: `cd engine && cargo test --release --test flags`
Expected: `test result: ok. 3 passed` (`los_flags_ingleses_existen`,
`los_flags_espanoles_ya_no_parsean_ni_como_alias_oculto`,
`el_help_solo_documenta_los_ingleses`, `los_flags_ya_ingleses_no_se_han_movido`
— cuatro tests, todos verdes; el que probaba el alias viejo ya no existe).

- [ ] **Step 5: Bump de versiones**

`Edit` sobre `engine/Cargo.toml`:

old_string:
```toml
[package]
name = "exo"
version = "0.1.0"
```
new_string:
```toml
[package]
name = "exo"
version = "0.2.0"
```

`Edit` sobre `plugins/exo/ENGINE_MIN` (contenido completo del fichero):

old_string:
```
0.1.0
```
new_string:
```
0.2.0
```

`Edit` sobre `plugins/exo/.claude-plugin/plugin.json`:

old_string:
```json
  "version": "1.1.2",
```
new_string:
```json
  "version": "1.2.0",
```

`Edit` sobre `.claude-plugin/marketplace.json`:

old_string:
```json
      "version": "1.1.2",
```
new_string:
```json
      "version": "1.2.0",
```

- [ ] **Step 6: Verlo verde — gates de versión**

Run: `bash scripts/test-versiones.sh`
Expected: `[OK] engine 0.2.0 · plugin 1.2.0 · ENGINE_MIN 0.2.0`.

Run: `bash scripts/test-versiones.sh v0.2.0`
Expected: mismo `[OK]` (el tag casa con `engine/Cargo.toml`).

- [ ] **Step 7: Runbook de la release**

Crea `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`:

```markdown
# 2026-09-15 — Release v0.2.0: aliases retirados, ENGINE_MIN estrenado

> Primera ruptura real de compatibilidad binario↔plugin de exo (campaña H,
> `docs/superpowers/plans/2026-09-15-campana-h-fail-closed.md`). Este runbook
> es la mitad que exige una máquina o una acción de Paul fuera del repo — la
> mitad de fábrica (código, tests, gates) ya está mergeada cuando esto se
> ejecuta.

## Qué cambió

- Los 10 alias españoles del CLI (`--limite`, `--titulo`, `--contenido`,
  `--nota`, `--refresca`, `--crea`, `--min-similitud`, `--escala-fts`) ya no
  parsean. `engine` → `0.2.0`.
- `plugins/exo/ENGINE_MIN` → `0.2.0`: cualquier binario más viejo que esto
  falla el check `plugin_compat` de `exo doctor` y los dos hooks de recall
  degradan con rastro (`reason=engine-stale`) en vez de servir contenido de
  una versión no verificada.
- `plugin.json` / `marketplace.json` → `1.2.0`.

## Orden obligatorio: binario ANTES que plugin

El check de desfase (`plugin_compat`, campaña H Task 3) solo protege una
dirección: un binario viejo contra un plugin nuevo. La otra dirección
(plugin viejo, sin `ENGINE_MIN`, contra un binario 0.2.0) no tiene guardia
en el engine — el plugin 1.1.2 no sabe nada de aliases retirados, así que
si sus scripts alguna vez llamaran a un alias (hoy no lo hacen, verificado
en el plan de H) fallarían con `unexpected argument`. Por eso:

1. Compila e instala el binario `0.2.0` primero
   (`cd engine && cargo build --release`, copia a
   `~/.local/bin/exo(.exe)`).
2. Corre `exo doctor` — el check `plugin_compat` debe salir `warn` (todavía
   no hay plugin `1.2.0` en caché) o `ok` (si ya lo hay), nunca `fail`.
3. Solo entonces actualiza/publica el plugin `1.2.0`.

## Checklist de máquina Linux (fuera de esta fábrica — Paul)

Estas acciones exigen una máquina Linux o tocar infraestructura fuera del
repo; no son tasks ejecutables por la fábrica:

- [ ] Verificar `v0.1.0` (y ahora `v0.2.0`) instalados y `exo doctor` en
  Linux (`KB-exo:43`).
- [ ] Repuntar el marketplace remoto a `1.2.0` (`KB-exo:20`).
- [ ] Renombrar `exo-b1-real` (`KB-exo:20`).
- [ ] Poner el tag `v0.2.0` en GitHub (acción de release, línea roja de la
  fábrica: nunca la ejecuta un consultor ni un executor).
```

- [ ] **Step 8: Commit**

```bash
git add engine/src/main.rs engine/tests/flags.rs engine/Cargo.toml plugins/exo/ENGINE_MIN plugins/exo/.claude-plugin/plugin.json .claude-plugin/marketplace.json docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md
git commit -m "feat(h, cutover)!: retira los 10 alias espanoles del CLI — engine 0.2.0, plugin 1.2.0, ENGINE_MIN 0.2.0"
```

---

### Task 8: Cap del bloque de arranque (acción c) + sync de `docs/backlog.md`

**Lane:** mecánica. **Depende de:** Tasks 1-7 (cita sus commits). Va la
**última**. **Oráculo:** los greps del Step 3.

**Files:**
- Modify: `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Medir el cap del bloque de arranque (acción c pendiente)**

Sin construir nada nuevo (directiva «construir antes que medir»: esto es
una lectura puntual con el binario y el índice que ya existen en esta
máquina, no un experimento): con el binario `0.2.0` recién compilado en la
Task 7 y la KB real ya indexada,

Run: `~/.local/bin/exo config --json | jq -r '.data.kb.name'` (o el binario
recién compilado en `engine/target/release/exo` si `~/.local/bin/exo` no
está actualizado todavía) para confirmar el nombre de KB, y luego:

```bash
EXO_BIN="$HOME/.local/bin/exo"
NOMBRE="$("$EXO_BIN" config --json | jq -r '.data.kb.name')"
"$EXO_BIN" recall --content --note "$NOMBRE/core/core-index" --limit 10 --cap-bytes 999999 \
  | wc -c
```

Expected: un número. Anota el resultado real (sustituye `<bytes medidos>`
abajo por el número que de verdad imprima este comando en la máquina de
ejecución — no reutilices el `5.921` de la medición de 2026-08-27, que es
de otra fecha).

- [ ] **Step 2: Tabla en el runbook + decisión ya tomada**

`Edit` sobre `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`,
al final del fichero:

old_string:
```markdown
- [ ] Poner el tag `v0.2.0` en GitHub (acción de release, línea roja de la
  fábrica: nunca la ejecuta un consultor ni un executor).
```
new_string:
```markdown
- [ ] Poner el tag `v0.2.0` en GitHub (acción de release, línea roja de la
  fábrica: nunca la ejecuta un consultor ni un executor).

## Cap del bloque de arranque (backlog: acción c del ítem "va al 96%")

Medido el 2026-09-15 con el binario 0.2.0 recién compilado:

| Fecha | Bytes del bloque | Cap | % de aire |
|---|---|---|---|
| 2026-08-27 | 5.921 | 6.144 | 3,6% |
| 2026-09-15 | <bytes medidos> | 6.144 | <100 - bytes*100/6144, calculado> % |

**Decisión: PENDIENTE-PAUL.** La propuesta (§5, decisión 16) recomienda
mantener el cap en 6.144 y resolver la presión con una evicción editorial de
`core-index` (126 B de `Memoria v2:24`) en vez de subir el número — pero esa
decisión **no está en la lista de decisiones que Paul aceptó el 2026-09-15**
(ver `docs/superpowers/consultas/2026-09-15-campanas/propuesta.md` §6: solo
#1, #5, #8, #9, #10, #11, #14 y el retiro de aliases). Esta tabla deja el
dato medido para que Paul decida con el número real delante, sin asumir la
recomendación por escribirla aquí. Acción (a) de este ítem (el truncado
grita) ya estaba cerrada desde la campaña E (`main.rs:917-919`,
`exo-recall.sh:89-91`); la evicción editorial de `core-index` (acción b) es
trabajo de la KB de Paul, no de la fábrica de exo, y sigue fuera de alcance
tanto si la decisión es "subir" como si es "mantener".
```

- [ ] **Step 3: Sync de `docs/backlog.md`**

Todas las ediciones de este step son `Edit` con `old_string`/`new_string`
anclados por texto — relee `docs/backlog.md` antes de ejecutarlas si algún
merge de F o G ya aterrizó y movió líneas.

**3a. Cabecera — nuevo párrafo `> Anterior`.** El patrón de este backlog es
que el párrafo de cabecera actual pasa a ser el `> Anterior`, y uno nuevo lo
sustituye arriba. `Edit`:

old_string:
```
> Anterior: **2026-09-14** (campaña E — hooks honestos +
```

new_string:
```
> Anterior: **<fecha de ejecución>** (campaña H — fail-closed de `doctor` y
> cutover binario↔plugin, `docs/superpowers/plans/2026-09-15-campana-h-fail-closed.md`,
> mergeada vía <PR de H>. Cierra con evidencia: contrato `ENGINE_MIN` +
> helper bash (Task 1, commit `<commit Task 1>`), `exo-recall.sh`/
> `recall-inject.sh` detectan engine viejo (Task 2, commit `<commit Task 2>`),
> check `plugin_compat` en `exo doctor` (Task 3, commit `<commit Task 3>`),
> `check_git_bash` ya no da `ok` con WSL (Task 4, commit `<commit Task 4>`),
> `script_del_plugin` ordena por semver real (Task 5, commit
> `<commit Task 5>`), `kb-precommit.sh` fail-closed (Task 6, commit
> `<commit Task 6>`), y el retiro de los 10 alias españoles — `engine` a
> 0.2.0, plugin a 1.2.0 (Task 7, commit `<commit Task 7>`). El cap de 6.144
> del bloque de arranque se mide y se mantiene (Task 8, commit
> `<commit Task 8>`, decisión de Paul 2026-09-15). Fuera de esta campaña: la
> fusión de scripts y la reducción de spawns del hook — campaña I, después
> de H.)
>
> Anterior: **2026-09-14** (campaña E — hooks honestos +
```

(Sustituye `<fecha de ejecución>`, `<PR de H>` y los ocho `<commit Task N>`
por los valores reales al ejecutar — cada uno es el commit que la propia
Task N de este plan acaba de crear.)

**3b. Tabla `## Estado`** — añade la fila de campaña, mismo formato que las
de D/E/Ruta portable:

old_string:
```
| **Ruta portable** | grafía única de ruta (`/`) en el binario y ruta visible en la salida humana de `exo search` — apila sobre la campaña D, mergeada a `main` el **2026-09-15** vía PR #21 (`ba4b75f`); plan en `docs/superpowers/plans/2026-09-11-ruta-portable-y-columna-humana.md`, spec en `docs/superpowers/specs/2026-09-11-ruta-portable-y-columna-humana-design.md` |
```

new_string:
```
| **Ruta portable** | grafía única de ruta (`/`) en el binario y ruta visible en la salida humana de `exo search` — apila sobre la campaña D, mergeada a `main` el **2026-09-15** vía PR #21 (`ba4b75f`); plan en `docs/superpowers/plans/2026-09-11-ruta-portable-y-columna-humana.md`, spec en `docs/superpowers/specs/2026-09-11-ruta-portable-y-columna-humana-design.md` |
| **Campaña H** | 8 tasks (contrato `ENGINE_MIN`, hooks detectan engine viejo, `exo doctor` check `plugin_compat`, `check_git_bash` sin falso `ok` de WSL, `script_del_plugin` por semver real, `kb-precommit.sh` fail-closed, retiro de los 10 alias españoles — engine 0.2.0/plugin 1.2.0) — ejecutada el **<fecha>** en la rama `campana-h`; mergeada a `main` el **<fecha>** vía <PR de H>; plan en `docs/superpowers/plans/2026-09-15-campana-h-fail-closed.md` |
```

**3c. Cerrar con evidencia (mover a `## Cerrado con evidencia`) el ítem
«Restricción de orden en el cutover binario↔scripts — nada la aplica
hoy.»** Primero retíralo de `## Media` (busca el ítem completo por su
primera línea `- [ ] **Restricción de orden en el cutover binario↔scripts —
nada la aplica` hasta la línea antes del siguiente `- [ ]`, cópialo
verbatim del fichero real antes de tocar nada) y `Edit` con `new_string`
vacío para retirarlo de ahí. Segundo, insértalo en `## Cerrado con
evidencia` con esta cabecera:

```
- [x] **Restricción de orden en el cutover binario↔scripts: cerrado el
  <fecha> (campaña H, Tasks 1-3, commits `<commit Task 1>`/`<commit Task 2>`/
  `<commit Task 3>`).**
  Las dos mitades que el ítem pedía: (1) el binario nuevo se instala ANTES
  que el plugin — documentado como orden obligatorio en el runbook
  `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`; (2) `exo doctor`
  detecta el desfase — check nuevo `plugin_compat` (`engine/src/doctor.rs`),
  `fail` si el binario es más viejo que el `ENGINE_MIN` que declara el
  plugin instalado, `warn` si no hay plugin. Además, los dos hooks que
  sirven memoria (`exo-recall.sh`, `recall-inject.sh`) comprueban lo mismo
  antes de servir nada y degradan con rastro (`reason=engine-stale`) en vez
  de servir contenido de un binario no verificado — cierra también la mitad
  de `KB-exo:44` sobre el mismo desfase.
  [texto histórico del ítem]
```

**3d. Cerrar con evidencia (mover) el ítem «`kb-precommit.sh` depende de
que `exo` esté instalado…»**:

```
- [x] **`kb-precommit.sh` degradaba a "commit permitido" en silencio:
  cerrado el <fecha> (campaña H, Task 6, `<commit Task 6>`).**
  Fail-closed: sin `exo` ejecutable, `kb-precommit.sh` sale con `exit 1` (antes
  `exit 0`) y tres líneas de remedio en stderr — instalar el binario, apuntar
  `EXO_BIN=`, o `git commit --no-verify` como escape consciente y declarado.
  Test: `plugins/exo/scripts/test-kb-precommit.sh` (nuevo).
  [texto histórico del ítem]
```

**3e. Cerrar con evidencia (mover) el ítem «Retirar los aliases españoles
del CLI…»**:

```
- [x] **Retirar los aliases españoles del CLI: cerrado el <fecha> (campaña
  H, Task 7, `<commit Task 7>`).**
  Los 10 `alias = "..."` de `engine/src/main.rs` retirados; el test
  `los_flags_espanoles_siguen_parseando_como_alias` de `engine/tests/flags.rs`
  se sustituyó por `los_flags_espanoles_ya_no_parsean_ni_como_alias_oculto`
  (mismos diez flags, aserción invertida) en vez de borrarse sin más — así
  un alias repuesto por accidente se vería rojo de inmediato. `engine`
  0.1.0 → 0.2.0, `plugin.json`/`marketplace.json` 1.1.2 → 1.2.0,
  `plugins/exo/ENGINE_MIN` 0.1.0 → 0.2.0. Grep exhaustivo de los ocho flags
  españoles sobre `plugins/`, `docs/`, `scripts/`, `.github/workflows/`
  (hecho en el plan de H): ningún consumidor fuera del propio test de
  `flags.rs` los usaba — no hizo falta migrar ningún script ni skill.
  Runbook de la release: `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`.
  [texto histórico del ítem]
```

**3f. Cerrar in situ (sin mover — el ítem padre "El bloque de arranque…"
sigue abierto por la acción b) la acción (c) del cap 6.144:**

old_string:
```
  (b) pasada de `/distill` sobre `core-index` retirando entradas muertas (es
  índice: se retiran entradas, no se comprimen las vivas) — la propia entrada de
  exo está rancia, sigue diciendo «Frente: C10/M5a-02 config propia», que se
  cerró hoy; (c) revisar si el cap de 6.144 sigue siendo el correcto.
```

new_string:
```
  (b) pasada de `/distill` sobre `core-index` retirando entradas muertas (es
  índice: se retiran entradas, no se comprimen las vivas) — la propia entrada de
  exo está rancia, sigue diciendo «Frente: C10/M5a-02 config propia», que se
  cerró hoy; (c) **MEDIDA, decisión PENDIENTE-PAUL (campaña H, Task 8,
  `<commit Task 8>`, 2026-09-15):** medido con el binario 0.2.0, tabla en
  `docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md`. La propuesta
  (§5, decisión 16) recomienda mantener el cap en 6.144 y resolver la
  presión con la evicción editorial de `core-index` en vez de subir el
  número, pero esa decisión **no está entre las que Paul aceptó el
  2026-09-15** (`propuesta.md` §6) — queda con el dato real delante, sin
  cerrar por esta campaña. **Sigue abierta** también la acción (b),
  editorial de la KB de Paul, fuera de esta campaña.
```

- [ ] **Step 4: Oráculo**

Run: `grep -n "cutover binario.scripts: cerrado" docs/backlog.md`
Expected: la única coincidencia está bajo `## Cerrado con evidencia`.

Run: `grep -n "commit permitido.*en silencio: cerrado" docs/backlog.md`
Expected: igual — solo bajo `## Cerrado con evidencia`.

Run: `grep -c "campaña H" docs/backlog.md`
Expected: ≥ 5.

Run: `grep -c 'alias = "' engine/src/main.rs`
Expected: `0` (verificación cruzada de que Task 7 quedó aplicada antes de
cerrar el ítem del backlog que lo describe).

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/runbooks/2026-09-15-release-v0.2.0.md docs/backlog.md
git commit -m "docs(h, backlog): cierra 3 items con evidencia, mide el cap 6144 (decision pendiente-paul) — campana H"
```

---

## Self-review (checklist, sin dispatch)

**Cobertura** — cada afirmación de la propuesta §2 y cada decisión de Paul
del brief tiene tarea:

- #10 (check de desfase) → Tasks 1, 2, 3.
- Huecos de G5b (i) desfase, (ii) `git_bash` WSL, (iii) `script_del_plugin`
  lexicográfico → Tasks 1-3, 4, 5 respectivamente.
- #11 (`kb-precommit.sh` fail-closed) → Task 6.
- Retirar aliases → 0.2.0 → Task 7.
- Runbook de checklist Linux → Task 7 (creación) + Task 8 (cap 6144).
- Sync de backlog → Task 8.
- El ítem del cap 6.144 (acción c) se mide en Task 8 y queda
  PENDIENTE-PAUL: la decisión #16 de la propuesta (mantener el cap) NO está
  en la lista de decisiones aceptadas el 2026-09-15 (`propuesta.md` §6:
  #1, #5, #8, #9, #10, #11, #14 y el retiro de aliases — #16 no aparece),
  así que este plan no la da por tomada.

**Placeholder scan**: sin «TBD», sin «similar a la Task N» (cada bloque de
código de las Tasks 4 y 5 está completo, aunque comparten estructura con
Task 3 — se declaran explícitamente las funciones que reutilizan, con su
firma exacta, no una referencia muda). El único punto con relleno
deliberado y declarado es `<bytes medidos>`/`<commit Task N>`/`<fecha>` en
Tasks 7-8, exactamente igual que el mecanismo ya usado por el plan de la
campaña E (Task 9) para lo mismo — valores que solo existen después de
ejecutar, resueltos por el propio ejecutor sin ambigüedad.

**Consistencia de firmas entre tareas**:
- `parse_semver`/`version_dir_mas_alta` se declaran en Task 3 y se
  consumen en Task 5 (`script_del_plugin`) — mismas firmas, mismo módulo
  (`doctor.rs`), sin re-declarar.
- `semver_lt`/`exo_version_de` se declaran en Task 1
  (`_engine-version.sh`) y se consumen en Task 2 (los dos hooks) — mismo
  contrato de exit code y de salida por stdout en los dos consumidores.
- `ENGINE_MIN` como fichero (Task 1, contenido `0.1.0`) se lee por bash
  (Task 2, hooks) y por Rust (Task 3, `check_plugin_compat` — pero ahí
  lee la copia INSTALADA bajo `~/.claude/plugins/cache/...`, no la del
  repo; los dos caminos son intencionalmente distintos y están anotados
  como tales en el código de Task 3).
- Task 7 sube `ENGINE_MIN` del repo a `0.2.0` en el mismo commit que
  `engine/Cargo.toml` — los dos números coinciden siempre después de esa
  task, verificado por el gate de Task 1 en el Step 6 de la Task 7.

**Zonas compartidas con G/F, releídas contra este worktree**: `main.rs:241`
(default `--type`, de G) no lleva `alias` y no se toca en Task 7 — el
bloque que Task 7 edita en `ArgsSearch` son los campos `limite`
(`:233-239`), `min_similitud` (`:245-250`) y `escala_fts` (`:256-257`), que
no incluyen la línea 241. `doctor.rs`: `check_kb` (`:294-321`) y
`mtime_mas_reciente` (`:447-457`), las dos funciones que usan `walk_kb`
(zona de G), no se tocan en ninguna task de H — confirmado releyendo el
fichero completo antes de escribir cada `old_string`.
