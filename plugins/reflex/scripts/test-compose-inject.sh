#!/usr/bin/env bash
# Test standalone para compose-inject.sh (componedor A1, spec transporte §5.1).
# Perfiles por agent_type, cap 2KB, canario por fichero de ventana.
# Fixtures en mktemp -d; nunca toca la KB real ni ~/.claude/reflex-inject-canary.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
COMPOSE="${SCRIPT_DIR}/compose-inject.sh"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Aislamiento global (F3.1): el cap final ahora loguea via _reflex-log.sh
# (default $HOME/.claude/reflex-log.jsonl) cuando de verdad trunca. Ningún
# caso existente lo dispara hoy, pero fijar esto evita que un fixture futuro
# ensucie el log real por accidente.
export REFLEX_LOG_FILE="$TMP/reflex-log.jsonl"

PASS=0
FAIL=0

pass() { printf '[PASS] %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '[FAIL] %s — %s\n' "$1" "$2"; FAIL=$((FAIL+1)); }

contains()     { case "$1" in *"$2"*) return 0 ;; *) return 1 ;; esac; }
not_contains() { case "$1" in *"$2"*) return 1 ;; *) return 0 ;; esac; }

# --- fixture básica: executor.md + KB (core-index con Doctrina compacta / Cores, otra-nota, proj) ---
EXEC_MD="$TMP/executor_basico.md"
cat > "$EXEC_MD" <<'EOF'
---
name: executor
description: fixture de prueba
model: sonnet
---

FRAGMENTO_DOCTRINA_TEST cuerpo de doctrina de prueba para compose-inject.
Segunda línea de doctrina fixture, para verificar que se extrae del cuerpo
(tras el segundo delimitador de frontmatter) y no del frontmatter mismo.
EOF

KB1="$TMP/kb1"
mkdir -p "$KB1/core" "$KB1/projects"
cat > "$KB1/core/core-index.md" <<'EOF'
# Core Index (fixture)

## Doctrina compacta
- ORQUESTADOR: bullet uno de doctrina compacta fixture.
- COST PYRAMID: bullet dos de doctrina compacta fixture.
- Bullet tres de relleno fixture.

## Cores
- core-a: nota core A fixture.
- core-b: nota core B fixture.
EOF
cat > "$KB1/core/otra-nota.md" <<'EOF'
# Título Otra Nota
Contenido de relleno para la fixture.
EOF
cat > "$KB1/projects/proj.md" <<'EOF'
# Título Proyecto Fixture
Contenido de proyecto fixture.
EOF

NO_CANARY="$TMP/no-existe-canario"

# =========================================================================
# Caso 1: --type general-purpose con KB fixture ⇒ header + doctrina +
# doctrina compacta + ≥1 ruta real.
# =========================================================================
{
  OUT1="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose --kb "$KB1")"
  EC1=$?
  if [ $EC1 -eq 0 ] \
     && contains "$OUT1" "=== Contexto inyectado" \
     && contains "$OUT1" "FRAGMENTO_DOCTRINA_TEST" \
     && contains "$OUT1" "ORQUESTADOR" \
     && contains "$OUT1" "$KB1/core/otra-nota.md"; then
    pass "caso1: general-purpose ⇒ header+doctrina+compacta+ruta"
  else
    fail "caso1: general-purpose ⇒ header+doctrina+compacta+ruta" "ec=$EC1 out=$OUT1"
  fi
}

# =========================================================================
# Caso 2: --type reflex:executor (perfil reducido) ⇒ SIN doctrina, SIN
# ORQUESTADOR/COST PYRAMID (sin doctrina compacta), CON ≥1 ruta.
# =========================================================================
{
  OUT2="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type reflex:executor --kb "$KB1")"
  EC2=$?
  if [ $EC2 -eq 0 ] \
     && not_contains "$OUT2" "FRAGMENTO_DOCTRINA_TEST" \
     && not_contains "$OUT2" "ORQUESTADOR" \
     && not_contains "$OUT2" "COST PYRAMID" \
     && contains "$OUT2" "$KB1/core/otra-nota.md"; then
    pass "caso2: reflex:executor (reducido) ⇒ sin doctrina/compacta, con ruta"
  else
    fail "caso2: reflex:executor (reducido) ⇒ sin doctrina/compacta, con ruta" "ec=$EC2 out=$OUT2"
  fi
}

# =========================================================================
# Caso 3: --type Explore (perfil divergente) ⇒ doctrina + rutas con línea
# de índice (título), SIN bloque "Doctrina compacta".
# =========================================================================
{
  OUT3="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type Explore --kb "$KB1")"
  EC3=$?
  if [ $EC3 -eq 0 ] \
     && contains "$OUT3" "FRAGMENTO_DOCTRINA_TEST" \
     && contains "$OUT3" "$KB1/core/otra-nota.md" \
     && contains "$OUT3" "Título Otra Nota" \
     && not_contains "$OUT3" "Doctrina compacta"; then
    pass "caso3: Explore (divergente) ⇒ doctrina+rutas con título, sin compacta"
  else
    fail "caso3: Explore (divergente) ⇒ doctrina+rutas con título, sin compacta" "ec=$EC3 out=$OUT3"
  fi
}

# =========================================================================
# Caso 4: --type tipo-inventado-xyz (perfil _default=doctrina) ⇒ solo
# doctrina, ≤1100 bytes.
# =========================================================================
{
  OUT4_FILE="$TMP/out4.txt"
  REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type tipo-inventado-xyz --kb "$KB1" > "$OUT4_FILE"
  EC4=$?
  OUT4="$(cat "$OUT4_FILE")"
  BYTES4="$(wc -c < "$OUT4_FILE" | tr -d ' ')"
  if [ $EC4 -eq 0 ] \
     && contains "$OUT4" "FRAGMENTO_DOCTRINA_TEST" \
     && not_contains "$OUT4" "Notas canonicas" \
     && [ "$BYTES4" -le 1100 ]; then
    pass "caso4: tipo-inventado-xyz (_default=doctrina) ⇒ solo doctrina, ≤1100B"
  else
    fail "caso4: tipo-inventado-xyz (_default=doctrina) ⇒ solo doctrina, ≤1100B" \
      "ec=$EC4 bytes=$BYTES4 out=$OUT4"
  fi
}

# =========================================================================
# Fixture REALISTA para casos 5-6: executor.md ~1192B de cuerpo, core-index
# con "Doctrina compacta" ≥1KB, 6 rutas largas en projects/.
# =========================================================================
gen_body() {  # gen_body <bytes> — repite una frase ASCII hasta cubrir <bytes>
  local n="$1" phrase body
  phrase="Doctrina de prueba realista para compose-inject: verifica el cap de bytes del perfil ejecucion en escenarios de tamaño real. "
  body=""
  while [ "${#body}" -lt "$n" ]; do
    body="${body}${phrase}"
  done
  printf '%s' "$body" | head -c "$n"
}

EXEC_REAL="$TMP/executor_realista.md"
{
  printf -- '---\nname: executor\ndescription: fixture realista\nmodel: sonnet\n---\n\n'
  gen_body 1192
} > "$EXEC_REAL"

KB5="$TMP/kb5"
mkdir -p "$KB5/core" "$KB5/projects"
{
  printf '# Core Index (fixture realista)\n\n## Doctrina compacta\n'
  gen_body 1100
  printf '\n\n## Cores\n- core-a: nota core A fixture realista.\n- core-b: nota core B fixture realista.\n'
} > "$KB5/core/core-index.md"

i=1
while [ "$i" -le 6 ]; do
  f="$KB5/projects/proyecto-con-nombre-muy-largo-para-simular-rutas-reales-de-produccion-numero-$i-en-el-fixture.md"
  printf '# Título Proyecto Largo %d\nContenido de relleno número %d para robustecer el tamaño del fixture realista de projects.\n' "$i" "$i" > "$f"
  i=$((i+1))
done

CANARY5="$TMP/canario5"
printf 'RFX-A1-K3P7\n' > "$CANARY5"

OUT5_FILE="$TMP/out5.txt"
REFLEX_EXECUTOR_MD="$EXEC_REAL" REFLEX_CANARY_FILE="$CANARY5" \
  "$COMPOSE" --type general-purpose --kb "$KB5" > "$OUT5_FILE"
EC5=$?
BYTES5="$(wc -c < "$OUT5_FILE" | tr -d ' ')"

# =========================================================================
# Caso 5: cap con fixture realista + canario ⇒ bloque total ≤2048 bytes.
# =========================================================================
{
  if [ $EC5 -eq 0 ] && [ "$BYTES5" -le 2048 ]; then
    pass "caso5: fixture realista + canario ⇒ total ≤2048B (bytes=$BYTES5)"
  else
    fail "caso5: fixture realista + canario ⇒ total ≤2048B" "ec=$EC5 bytes=$BYTES5"
  fi
}

# =========================================================================
# Caso 6: sobre el mismo bloque del caso 5, canario presente + ≥1 ruta;
# repetido sin REFLEX_CANARY_FILE ⇒ sin "token".
# =========================================================================
{
  OUT5="$(cat "$OUT5_FILE")"
  OUT6_FILE="$TMP/out6.txt"
  REFLEX_EXECUTOR_MD="$EXEC_REAL" REFLEX_CANARY_FILE="$TMP/no-existe-canario-caso6" \
    "$COMPOSE" --type general-purpose --kb "$KB5" > "$OUT6_FILE"
  EC6=$?
  OUT6="$(cat "$OUT6_FILE")"
  if contains "$OUT5" "incluye el token RFX-A1-K3P7" \
     && contains "$OUT5" "$KB5/projects/" \
     && [ $EC6 -eq 0 ] \
     && not_contains "$OUT6" "token"; then
    pass "caso6: canario presente+ruta en caso5; sin canary-file ⇒ sin 'token'"
  else
    fail "caso6: canario presente+ruta en caso5; sin canary-file ⇒ sin 'token'" \
      "ec6=$EC6 out5=$OUT5 out6=$OUT6"
  fi
}

# =========================================================================
# Caso 7: degradación --kb /ruta/inexistente ⇒ exit 0, solo doctrina, sin
# crash (sin digest de KB).
# =========================================================================
{
  OUT7="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose --kb "/ruta/inexistente-xyz-123")"
  EC7=$?
  if [ $EC7 -eq 0 ] \
     && contains "$OUT7" "FRAGMENTO_DOCTRINA_TEST" \
     && not_contains "$OUT7" "Notas canonicas" \
     && not_contains "$OUT7" "Doctrina compacta"; then
    pass "caso7: --kb inexistente ⇒ exit0, solo doctrina, sin crash"
  else
    fail "caso7: --kb inexistente ⇒ exit0, solo doctrina, sin crash" "ec=$EC7 out=$OUT7"
  fi
}

# =========================================================================
# Caso 8: --type ausente, o perfiles ilegibles ⇒ exit≠0 y stdout vacío.
# =========================================================================
{
  OUT8A="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --kb "$KB1" 2>/dev/null)"
  EC8A=$?
  OUT8B="$(REFLEX_INJECT_PROFILES="/dev/null" REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose --kb "$KB1" 2>/dev/null)"
  EC8B=$?
  if [ $EC8A -ne 0 ] && [ -z "$OUT8A" ] && [ $EC8B -ne 0 ] && [ -z "$OUT8B" ]; then
    pass "caso8: --type ausente / perfiles ilegibles ⇒ exit≠0, stdout vacío"
  else
    fail "caso8: --type ausente / perfiles ilegibles ⇒ exit≠0, stdout vacío" \
      "ec8a=$EC8A out8a=$OUT8A ec8b=$EC8B out8b=$OUT8B"
  fi
}

# =========================================================================
# Caso 9 (I4/M4): cortes por LÍNEAS, no a mitad de línea. Fixture construida a
# medida: 3 líneas cortas + 1 línea larga (04) diseñada para que el corte
# CRUDO a 800B (comportamiento VIEJO, head -c) caiga exactamente a mitad del
# carácter multibyte "í" (2 bytes UTF-8) de esa línea, y 16 líneas más detrás
# que nunca deberían sobrevivir al cap. Verifica: (a) cada línea del bloque
# emitido coincide LITERAL con una línea completa del fuente (ninguna quedó
# cortada a mitad); (b) menos de 20 líneas sobreviven (el cap sí actuó); (c)
# el bloque es UTF-8 válido de punta a punta (iconv -f utf8 -t utf8 no falla —
# un corte a mitad del "í" lo haría fallar, y es justo lo que el corte VIEJO
# produce en esta fixture).
# =========================================================================
EXEC_LINEAS="$TMP/executor_lineas.md"
printf -- '---\nname: executor\ndescription: fixture de lineas\nmodel: sonnet\n---\n\n' > "$EXEC_LINEAS"
i=1
while [ "$i" -le 3 ]; do
  printf 'LINEA_DOCTRINA_%02d: relleno corto de la fixture de lineas, item %02d.\n' "$i" "$i" >> "$EXEC_LINEAS"
  i=$((i+1))
done
CUM3="$(wc -c < "$EXEC_LINEAS" | tr -d ' ')"
PREFIX4="LINEA_DOCTRINA_04: "
PREFIX4_LEN="$(printf '%s' "$PREFIX4" | wc -c)"
# bytes ASCII de relleno tras el prefijo, hasta 1 byte ANTES del limite de 800B
PAD_LEN=$(( 800 - CUM3 - PREFIX4_LEN - 1 ))
PAD="$(printf 'x%.0s' $(seq 1 "$PAD_LEN"))"
printf '%s%sí resto de la linea larga numero 04 que sigue despues del caracter multibyte, con relleno adicional hasta superar con margen el presupuesto de 800 bytes de doctrina().\n' \
  "$PREFIX4" "$PAD" >> "$EXEC_LINEAS"
i=5
while [ "$i" -le 20 ]; do
  printf 'LINEA_DOCTRINA_%02d: relleno corto de la fixture de lineas, item %02d.\n' "$i" "$i" >> "$EXEC_LINEAS"
  i=$((i+1))
done
{
  OUT9_FILE="$TMP/out9.txt"
  REFLEX_EXECUTOR_MD="$EXEC_LINEAS" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type tipo-inventado-para-truncar --kb "$KB1" > "$OUT9_FILE"
  EC9=$?
  N_LINEAS_SALIDA="$(grep -c '^LINEA_DOCTRINA_' "$OUT9_FILE")"
  ALL_MATCH=1
  while IFS= read -r linea; do
    grep -qxF "$linea" "$EXEC_LINEAS" || ALL_MATCH=0
  done < <(grep '^LINEA_DOCTRINA_' "$OUT9_FILE")
  UTF8_OK=1
  iconv -f utf8 -t utf8 "$OUT9_FILE" >/dev/null 2>&1 || UTF8_OK=0
  if [ $EC9 -eq 0 ] && [ "$N_LINEAS_SALIDA" -gt 0 ] && [ "$N_LINEAS_SALIDA" -lt 20 ] \
     && [ "$ALL_MATCH" -eq 1 ] && [ "$UTF8_OK" -eq 1 ]; then
    pass "caso9: cap por líneas ⇒ ninguna línea truncada (n=$N_LINEAS_SALIDA/20), UTF-8 válido"
  else
    fail "caso9: cap por líneas ⇒ ninguna línea truncada, UTF-8 válido" \
      "ec=$EC9 n_lineas=$N_LINEAS_SALIDA all_match=$ALL_MATCH utf8_ok=$UTF8_OK"
  fi
}

# =========================================================================
# Caso 10 (I4): toda ruta emitida en el bloque existe en disco. Reutiliza el
# fixture realista (KB5, 6 rutas largas en projects/) para forzar que el cap
# recorte parte del listado de rutas y verificar que ninguna de las que
# sobreviven quedó truncada a media ruta (lo que rompería el path).
# =========================================================================
{
  OUT10="$(REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type Explore --kb "$KB5")"
  EC10=$?
  RUTAS10="$(printf '%s\n' "$OUT10" | grep '^- /' | awk '{print $2}')"
  N_RUTAS10="$(printf '%s\n' "$RUTAS10" | grep -c .)"
  ALL_EXIST=1
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    [ -f "$p" ] || ALL_EXIST=0
  done <<< "$RUTAS10"
  if [ $EC10 -eq 0 ] && [ "$N_RUTAS10" -gt 0 ] && [ "$ALL_EXIST" -eq 1 ]; then
    pass "caso10: toda ruta emitida en el bloque existe en disco (n=$N_RUTAS10)"
  else
    fail "caso10: toda ruta emitida en el bloque existe en disco" \
      "ec=$EC10 n_rutas=$N_RUTAS10 all_exist=$ALL_EXIST rutas=$RUTAS10"
  fi
}

# =========================================================================
# Caso 11 (I6): --type sin valor ⇒ exit≠0 rápido (no cuelga esperando $2).
# Antes, "shift 2" con un solo posicional restante no avanzaba $# y el parser
# quedaba en loop infinito; timeout 3 detecta la regresión (ec=124=cuelgue).
# =========================================================================
{
  timeout 3 "$COMPOSE" --type >/dev/null 2>&1
  EC11=$?
  timeout 3 "$COMPOSE" --type general-purpose --kb >/dev/null 2>&1
  EC11B=$?
  if [ $EC11 -ne 0 ] && [ $EC11 -ne 124 ] && [ $EC11B -ne 0 ] && [ $EC11B -ne 124 ]; then
    pass "caso11: --type/--kb sin valor ⇒ exit≠0 rápido (sin cuelgue)"
  else
    fail "caso11: --type/--kb sin valor ⇒ exit≠0 rápido (sin cuelgue)" \
      "ec_type=$EC11 ec_kb=$EC11B (124=timeout/cuelgue)"
  fi
}

# =========================================================================
# Fixture para casos 12-13: KB con 40 notas en core/ (rutas() no las cap,
# a diferencia de doctrina()/doctrina_compacta()) — suficientes para que el
# cap FINAL de 2048B (no los internos de 800/550) tenga que recortar líneas.
# =========================================================================
KB12="$TMP/kb12"
mkdir -p "$KB12/core"
i=1
while [ "$i" -le 40 ]; do
  f="$KB12/core/archivo-de-prueba-numero-$i-con-nombre-razonablemente-largo-para-forzar-el-corte-del-cap-final.md"
  printf '# Título Largo De Prueba Numero %d\nRelleno de contenido para el fixture del cap final, item %d.\n' "$i" "$i" > "$f"
  i=$((i+1))
done

# =========================================================================
# Caso 12 (F3.1): cap final SÍ trunca (rutas() sin cap interno desborda el
# presupuesto de 2048B) ⇒ debe quedar logueado un evento "inject-truncated"
# con lines_cut>0 y el budget usado.
# =========================================================================
{
  REFLEXLOG12="$(mktemp)"
  OUT12="$(REFLEX_LOG_FILE="$REFLEXLOG12" REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type Explore --kb "$KB12")"
  EC12=$?

  LINEA12="$(grep '"reflex":"inject-truncated"' "$REFLEXLOG12" 2>/dev/null)"
  if [ -n "$LINEA12" ]; then
    pass "caso12: inject-truncated logueado (rutas() desbordó el cap final)"
  else
    fail "caso12: inject-truncated logueado" "ec=$EC12 log=$(cat "$REFLEXLOG12" 2>/dev/null)"
  fi

  CORTADAS12="$(printf '%s' "$LINEA12" | jq -r '.payload' 2>/dev/null | grep -o 'lines_cut=[0-9]*' | cut -d= -f2)"
  if [ -n "$CORTADAS12" ] && [ "$CORTADAS12" -gt 0 ]; then
    pass "caso12: lines_cut=$CORTADAS12 (>0, el dato que permite reaccionar)"
  else
    fail "caso12: lines_cut>0 en el payload" "linea=$LINEA12"
  fi

  if printf '%s' "$LINEA12" | grep -q 'budget=2048'; then
    pass "caso12: budget=2048 presente en el payload"
  else
    fail "caso12: budget=2048 presente en el payload" "linea=$LINEA12"
  fi

  BYTES12="$(printf '%s' "$OUT12" | wc -c)"
  if [ $EC12 -eq 0 ] && [ "$BYTES12" -le 2048 ]; then
    pass "caso12: pese al log, el bloque sigue saliendo ≤2048B y exit 0 (never-break)"
  else
    fail "caso12: never-break (bloque ≤2048B, exit 0)" "ec=$EC12 bytes=$BYTES12"
  fi

  rm -f "$REFLEXLOG12"
}

# =========================================================================
# Caso 13 (F3.1): cap final NO trunca (fixture chica, caso1) ⇒ NO debe
# aparecer ningún evento "inject-truncated" (sin falsos positivos).
# =========================================================================
{
  REFLEXLOG13="$(mktemp)"
  REFLEX_LOG_FILE="$REFLEXLOG13" REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose --kb "$KB1" >/dev/null
  EC13=$?

  if [ ! -s "$REFLEXLOG13" ]; then
    pass "caso13: sin truncado real ⇒ ningún evento inject-truncated (sin falso positivo)"
  else
    fail "caso13: sin truncado real ⇒ ningún evento inject-truncated" \
      "ec=$EC13 log=$(cat "$REFLEXLOG13" 2>/dev/null)"
  fi

  rm -f "$REFLEXLOG13"
}

# =========================================================================
# Fixture para casos 14-15 (M6-03c, seam de KB): dos KB distinguibles por
# ruta — KB_ENV (la que debe ganar via $EXO_KB) y KB_HOME (la que hoy sirve
# el fallback a ~/.basic-memory/config.json). FAKE_HOME simula el HOME real
# sin tocarlo: compose-inject lee "$HOME/.basic-memory/config.json" literal,
# así que sobreescribir HOME en el entorno del proceso hijo basta para
# probar el fallback sin rozar el config.json de verdad.
# =========================================================================
KB_ENV="$TMP/kb_env_seam"
mkdir -p "$KB_ENV/core" "$KB_ENV/projects"
cat > "$KB_ENV/core/core-index.md" <<'EOF'
# Core Index (fixture KB_ENV)

## Cores
- core-env: nota core del KB_ENV (seam $EXO_KB).
EOF
cat > "$KB_ENV/core/otra-nota-env.md" <<'EOF'
# Título Nota KB_ENV
Contenido de relleno KB_ENV.
EOF

KB_HOME="$TMP/kb_home_fallback"
mkdir -p "$KB_HOME/core" "$KB_HOME/projects"
cat > "$KB_HOME/core/core-index.md" <<'EOF'
# Core Index (fixture KB_HOME)

## Cores
- core-home: nota core del KB_HOME (fallback config.json).
EOF
cat > "$KB_HOME/core/otra-nota-home.md" <<'EOF'
# Título Nota KB_HOME
Contenido de relleno KB_HOME.
EOF

FAKE_HOME="$TMP/fake_home"
mkdir -p "$FAKE_HOME/.basic-memory"
cat > "$FAKE_HOME/.basic-memory/config.json" <<EOF
{"projects": {"kb-demo": {"path": "$KB_HOME"}}}
EOF

# =========================================================================
# Caso 14 (M6-03c): sin --kb, con \$EXO_KB apuntando a KB_ENV y HOME apuntando
# a un config.json que resolvería KB_HOME ⇒ debe ganar KB_ENV (el seam va
# ANTES del fallback a basic-memory/config.json, aunque este exista y sea
# leíble).
# =========================================================================
{
  OUT14="$(HOME="$FAKE_HOME" EXO_KB="$KB_ENV" REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose)"
  EC14=$?
  if [ $EC14 -eq 0 ] \
     && contains "$OUT14" "$KB_ENV/core/otra-nota-env.md" \
     && not_contains "$OUT14" "$KB_HOME/core/otra-nota-home.md"; then
    pass "caso14: \$EXO_KB definida ⇒ gana sobre el fallback a config.json"
  else
    fail "caso14: \$EXO_KB definida ⇒ gana sobre el fallback a config.json" "ec=$EC14 out=$OUT14"
  fi
}

# =========================================================================
# Caso 15 (M6-03c, no-regresión): sin --kb y SIN \$EXO_KB ⇒ el comportamiento
# es idéntico al de hoy: cae al fallback de ~/.basic-memory/config.json (aquí
# simulado con HOME=FAKE_HOME) y resuelve KB_HOME.
# =========================================================================
{
  OUT15="$(HOME="$FAKE_HOME" REFLEX_EXECUTOR_MD="$EXEC_MD" REFLEX_CANARY_FILE="$NO_CANARY" \
    "$COMPOSE" --type general-purpose)"
  EC15=$?
  if [ $EC15 -eq 0 ] \
     && contains "$OUT15" "$KB_HOME/core/otra-nota-home.md" \
     && not_contains "$OUT15" "$KB_ENV/core/otra-nota-env.md"; then
    pass "caso15: sin \$EXO_KB ⇒ no-regresión, fallback a config.json idéntico al de hoy"
  else
    fail "caso15: sin \$EXO_KB ⇒ no-regresión, fallback a config.json idéntico al de hoy" "ec=$EC15 out=$OUT15"
  fi
}

echo ""
TOTAL=$((PASS+FAIL))
echo "=== Resultado: ${PASS}/${TOTAL} pasaron ==="
[ $FAIL -eq 0 ] && exit 0 || exit 1
