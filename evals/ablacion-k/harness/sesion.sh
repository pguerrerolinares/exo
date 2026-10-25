#!/usr/bin/env bash
# guardar_sesion <dir de proyecto de claude> <dir de corrida> — sourced por correr.sh (brazo a2s).
# Copia la sesión persistida a <corrida>/sesion.jsonl y mueve el árbol <uuid>/ (subagentes) de CADA sesión
# a <corrida>/sesion-sub/<uuid>; después quita el dir de proyecto si queda vacío. Solo toca rutas construidas
# con el uuid de un .jsonl concreto de <sdir>: nada más de <sdir> se borra.
guardar_sesion() {
  local sdir=$1 O=$2 f u
  [ -n "$sdir" ] && [ -n "$O" ] && [ -d "$sdir" ] || return 0
  cat "$sdir"/*.jsonl > "$O/sesion.jsonl" 2>/dev/null || return 0
  for f in "$sdir"/*.jsonl; do
    u=$(basename "$f" .jsonl)
    if [ -n "$u" ] && [ -d "$sdir/$u" ]; then
      mkdir -p "$O/sesion-sub" && mv "$sdir/$u" "$O/sesion-sub/$u" || return 1
    fi
    rm -f "$f"
  done
  rmdir "$sdir" 2>/dev/null
  return 0
}
