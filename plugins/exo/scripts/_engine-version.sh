#!/usr/bin/env bash
# Helper COMPARTIDO: compara versiones semver X.Y.Z en bash puro — sin
# `sort -V` (no está en macOS/BSD), sin `bc`, sin spawns extra. Lo usan
# exo-recall.sh (Task 2, el único hook con check de versión — decisión del
# dueño de la KB en pre-flight de H: `recall-inject.sh` corre en cada prompt
# y un `exo --version` ahí sería un spawn extra) y el gate
# `scripts/test-versiones.sh`.
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
