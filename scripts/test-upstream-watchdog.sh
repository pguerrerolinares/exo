#!/usr/bin/env bash
# Tests de upstream-watchdog.sh. Siempre con <ahora_iso> explícito: nunca el reloj real.
set -uo pipefail
RAIZ="$(git rev-parse --show-toplevel)" || exit 1
W="$RAIZ/scripts/upstream-watchdog.sh"
AHORA="2026-09-29T12:00:00Z"
PASS=0; FAIL=0
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT

check() { # nombre, esperado_exit, patrón, fichero, [max [tag fecha [max_tag]]]
  local n="$1" ex="$2" pat="$3" f="$4" out rc
  shift 4
  out="$(bash "$W" "$f" "$AHORA" "$@" 2>&1)"; rc=$?
  if [ "$rc" = "$ex" ] && printf '%s' "$out" | grep -qF -- "$pat"; then
    echo "PASS: $n"; PASS=$((PASS+1))
  else
    echo "FAIL: $n (exit=$rc, esperado=$ex, salida='$out')"; FAIL=$((FAIL+1))
  fi
}

printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-28T12:00:00Z"}]' > "$T/reciente.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-14T12:00:00Z"}]' > "$T/frontera.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-14T11:59:59Z"}]' > "$T/pasada.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-20T12:00:00Z"}]' > "$T/nueve_dias.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-09T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-09-27T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-08-01T00:00:00Z"}]' > "$T/desordenado.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-10T12:00:00Z"}]' > "$T/nueve.json"
printf '%s\n' '[]' > "$T/vacio.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-20T12:00:00Z"},{"body":"latido\nestado: alerta","created_at":"2026-09-28T12:00:00Z"}]' > "$T/alerta.json"
printf '%s\n' '[{"body":"latido\nestado: ok\nfallo a mitad\nestado: alerta","created_at":"2026-09-28T12:00:00Z"}]' > "$T/dos_estados.json"
printf '%s\n' '[{"body":"latido\nestado: ok","created_at":"2026-09-10T12:00:00Z"},{"body":"un comentario humano","created_at":"2026-09-28T12:00:00Z"}]' > "$T/humano.json"
printf '%s\n' '[{"body":"latido\nestado: alerta","created_at":"2026-09-10T12:00:00Z"},{"body":"latido\nestado: ok","created_at":"2026-09-28T12:00:00Z"}]' > "$T/alerta_vieja.json"

check latido_reciente 0 "latido OK" "$T/reciente.json"
check latido_caducado 1 "latido caducado" "$T/nueve.json" 8
check frontera 0 "latido OK" "$T/frontera.json"
check nueve_dias_verde 0 "latido OK" "$T/nueve_dias.json"
check frontera_mas_un_segundo 1 "latido caducado" "$T/pasada.json"
check usa_el_ultimo 0 "latido OK" "$T/desordenado.json"
check max_dias_parametro 1 "latido caducado" "$T/reciente.json" 0
check sin_issue 1 "issue upstream-sync: estado no existe" "-"
check sin_latidos 1 "sin latidos" "$T/vacio.json"

check ultimo_es_alerta 1 "latido en alerta: 2026-09-28T12:00:00Z" "$T/alerta.json"
check ultima_linea_estado_manda 1 "latido en alerta" "$T/dos_estados.json"
check comentario_sin_estado_no_cuenta 1 "latido caducado" "$T/humano.json"
check ok_reciente 0 "latido OK" "$T/alerta_vieja.json"

# Frescura del tag: AHORA=2026-09-29T12:00Z, latido de hace 1 día con los formatos reales de #30.
hb() { printf '%s\n' "[{\"body\":\"$2\\nestado: ok\",\"created_at\":\"2026-09-28T12:00:00Z\"}]" > "$T/$1.json"; }
hb tag_min 'tag revisado: v7.0.0 (desde v6.4.2)'
hb tag_may 'Tag revisado: v7.0.0 (sin tags nuevos; pasada de pendientes)'
hb tag_sin 'fecha: 2026-09-28'
hb tag_vacio 'Tag revisado:'
hb tag_sinv 'tag revisado: 7.0.0'
check tag_nuevo_4_dias_rojo 1 "tag sin revisar: upstream v7.0.1 de hace 4 días" "$T/tag_min.json" 15 v7.0.1 2026-09-25T11:00:00Z
check tag_nuevo_1_dia_verde 0 "tag nuevo v7.0.1 de hace 1 días" "$T/tag_min.json" 15 v7.0.1 2026-09-28T11:00:00Z
check tag_nuevo_frontera_3_dias_verde 0 "latido OK" "$T/tag_min.json" 15 v7.0.1 2026-09-26T12:00:00Z
check tag_nuevo_frontera_mas_un_segundo_rojo 1 "tag sin revisar" "$T/tag_min.json" 15 v7.0.1 2026-09-26T11:59:59Z
check tag_max_dias_parametro 1 "tag sin revisar" "$T/tag_min.json" 15 v7.0.1 2026-09-28T11:00:00Z 0
check tag_mismo_verde 0 "tag al día: v7.0.0" "$T/tag_min.json" 15 v7.0.0 2025-01-01T00:00:00Z
check tag_mayuscula_verde 0 "tag al día: v7.0.0" "$T/tag_may.json" 15 v7.0.0 2025-01-01T00:00:00Z
check tag_mayuscula_nuevo_rojo 1 "revisó v7.0.0" "$T/tag_may.json" 15 v7.0.1 2026-09-25T11:00:00Z
check tag_sin_v_equivale 0 "tag al día" "$T/tag_sinv.json" 15 v7.0.0 2025-01-01T00:00:00Z
check latido_sin_tag_rojo 1 "latido sin línea 'tag revisado'" "$T/tag_sin.json" 15 v7.0.0 2025-01-01T00:00:00Z
check latido_tag_vacio_rojo 1 "latido sin línea 'tag revisado'" "$T/tag_vacio.json" 15 v7.0.0 2025-01-01T00:00:00Z
check tag_upstream_vacio_rojo 1 "tag upstream no obtenido" "$T/tag_min.json" 15 "" ""
check fecha_tag_ilegible_rojo 1 "fecha del tag ilegible" "$T/tag_min.json" 15 v7.0.1 basura
check tag_no_enmascara_latido_caducado 1 "latido caducado" "$T/nueve.json" 8 v7.0.0 2025-01-01T00:00:00Z

mkdir "$T/nobin"
out="$(PATH="$T/nobin" "$(command -v bash)" "$W" "$T/reciente.json" "$AHORA" 2>&1)"; rc=$?
if [ "$rc" = 1 ] && [ "$out" = "jq no disponible" ]; then echo "PASS: sin_jq"; PASS=$((PASS+1)); else echo "FAIL: sin_jq ($rc, $out)"; FAIL=$((FAIL+1)); fi

echo "$PASS PASS, $FAIL FAIL"
[ "$FAIL" = 0 ]
