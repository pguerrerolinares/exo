#!/usr/bin/env bash
# Tests de upstream-watchdog.sh. Siempre con <ahora_iso> explícito: nunca el reloj real.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
W="$RAIZ/scripts/upstream-watchdog.sh"
AHORA="2026-09-29T12:00:00Z"
PASS=0; FAIL=0
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT

check() { # nombre, esperado_exit, patrón, fichero, [max]
  local n="$1" ex="$2" pat="$3" f="$4" out rc
  out="$(bash "$W" "$f" "$AHORA" ${5:+"$5"} 2>&1)"; rc=$?
  if [ "$rc" = "$ex" ] && printf '%s' "$out" | grep -qF -- "$pat"; then
    echo "PASS: $n"; PASS=$((PASS+1))
  else
    echo "FAIL: $n (exit=$rc, esperado=$ex, salida='$out')"; FAIL=$((FAIL+1))
  fi
}

echo '[{"body":"latido\nestado: ok","created_at":"2026-09-28T12:00:00Z"}]' > "$T/reciente.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-21T12:00:00Z"}]' > "$T/frontera.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-21T11:59:59Z"}]' > "$T/pasada.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-09T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-09-27T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-08-01T00:00:00Z"}]' > "$T/desordenado.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-10T12:00:00Z"}]' > "$T/nueve.json"
echo '[]' > "$T/vacio.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-20T12:00:00Z"},{"body":"latido\nestado: alerta","created_at":"2026-09-28T12:00:00Z"}]' > "$T/alerta.json"
echo '[{"body":"latido\nestado: ok\nfallo a mitad\nestado: alerta","created_at":"2026-09-28T12:00:00Z"}]' > "$T/dos_estados.json"
echo '[{"body":"latido\nestado: ok","created_at":"2026-09-10T12:00:00Z"},{"body":"un comentario humano","created_at":"2026-09-28T12:00:00Z"}]' > "$T/humano.json"
echo '[{"body":"latido\nestado: alerta","created_at":"2026-09-10T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-09-28T12:00:00Z"}]' > "$T/alerta_vieja.json"

check latido_reciente 0 "latido OK" "$T/reciente.json"
check latido_caducado 1 "latido caducado" "$T/nueve.json" 8
check frontera 0 "latido OK" "$T/frontera.json"
check frontera_mas_un_segundo 1 "latido caducado" "$T/pasada.json"
check usa_el_ultimo 0 "latido OK" "$T/desordenado.json"
check max_dias_parametro 1 "latido caducado" "$T/reciente.json" 0
check sin_issue 1 "issue upstream-sync: estado no existe" "-"
check sin_latidos 1 "sin latidos" "$T/vacio.json"

check ultimo_es_alerta 1 "latido en alerta: 2026-09-28T12:00:00Z" "$T/alerta.json"
check ultima_linea_estado_manda 1 "latido en alerta" "$T/dos_estados.json"
check comentario_sin_estado_no_cuenta 1 "latido caducado" "$T/humano.json"
check ok_reciente 0 "latido OK" "$T/alerta_vieja.json"

mkdir "$T/nobin"
out="$(PATH="$T/nobin" "$(command -v bash)" "$W" "$T/reciente.json" "$AHORA" 2>&1)"; rc=$?
if [ "$rc" = 1 ] && [ "$out" = "jq no disponible" ]; then echo "PASS: sin_jq"; PASS=$((PASS+1)); else echo "FAIL: sin_jq ($rc, $out)"; FAIL=$((FAIL+1)); fi

echo "$PASS PASS, $FAIL FAIL"
[ "$FAIL" = 0 ]
