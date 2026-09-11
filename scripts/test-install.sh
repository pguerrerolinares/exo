#!/usr/bin/env bash
# Gate de install.sh, sin red: se fabrica una "release" en un directorio local
# y se sirve por file://, que curl sabe leer.
#
# El caso que importa es el segundo: un checksum que no cuadra tiene que
# ABORTAR y NO dejar binario instalado. Un instalador que verifica el hash y
# sigue igual es peor que uno que no lo verifica, porque parece seguro.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
fallos=0

case "$(uname -s)" in
  Linux)  asset="exo-x86_64-unknown-linux-gnu"; bin="exo" ;;
  Darwin) asset="exo-aarch64-apple-darwin"; bin="exo" ;;
  MINGW*|MSYS*|CYGWIN*) asset="exo-x86_64-pc-windows-msvc.exe"; bin="exo.exe" ;;
  *) echo "test-install: plataforma no soportada por el test: $(uname -s)" >&2; exit 1 ;;
esac

fabrica_release() {
  local dir="$1" contenido="$2"
  mkdir -p "$dir"
  printf '%s' "$contenido" > "$dir/$asset"
  # Mismo fallback que install.sh: si el test exigiera `shasum` donde el
  # instalador funciona con `sha256sum`, no podria correr en maquinas que el
  # producto si soporta.
  if command -v shasum >/dev/null 2>&1; then
    ( cd "$dir" && shasum -a 256 "$asset" > "$asset.sha256" )
  else
    ( cd "$dir" && sha256sum "$asset" > "$asset.sha256" )
  fi
}

# file_url: construye un file:// que el curl instalado sepa abrir. En
# Linux/macOS, $dir (POSIX) tal cual basta. En Git Bash el curl empaquetado es
# el build nativo de mingw-w64 (no enlaza contra el runtime msys), así que NO
# traduce rutas POSIX por su cuenta: un file:///tmp/... que mktemp produce le
# resulta ilegible aunque el fichero exista. cygpath -m da la ruta con letra de
# unidad que ese curl sí entiende; donde no hay cygpath (Linux/macOS) esta
# rama no se toca.
file_url() {
  local dir="$1"
  if command -v cygpath >/dev/null 2>&1; then
    printf 'file://%s' "$(cygpath -m "$dir")"
  else
    printf 'file://%s' "$dir"
  fi
}

# --- Caso 1: checksum correcto -> instala y el binario queda donde el gate mira
rel="$TMP/release-ok"; dest="$TMP/bin-ok"
fabrica_release "$rel" '#!/bin/sh
echo exo-falso'
if EXO_BASE_URL="$(file_url "$rel")" EXO_DIR="$dest" bash ./install.sh >"$TMP/ok.log" 2>&1; then
  if [ -x "$dest/$bin" ]; then
    echo "test-install: OK — instalado en $dest/$bin"
  else
    echo "test-install: FALLO — salió 0 pero no hay binario en $dest/$bin" >&2
    fallos=1
  fi
else
  echo "test-install: FALLO — el caso bueno no instaló (exit $?)" >&2
  cat "$TMP/ok.log" >&2
  fallos=1
fi

# --- Caso 2: checksum manipulado -> aborta y NO deja binario
rel="$TMP/release-mala"; dest="$TMP/bin-malo"
fabrica_release "$rel" '#!/bin/sh
echo exo-falso'
# Se corrompe el binario DESPUÉS de firmar: el .sha256 ya no cuadra.
printf 'basura' >> "$rel/$asset"
if EXO_BASE_URL="$(file_url "$rel")" EXO_DIR="$dest" bash ./install.sh >"$TMP/malo.log" 2>&1; then
  echo "test-install: FALLO — un checksum que no cuadra salió 0" >&2
  fallos=1
else
  if [ -e "$dest/$bin" ]; then
    echo "test-install: FALLO — abortó pero dejó el binario instalado" >&2
    fallos=1
  else
    echo "test-install: OK — checksum malo aborta y no instala nada"
  fi
fi

[ "$fallos" -eq 0 ] || { echo "test-install: hay fallos" >&2; exit 1; }
echo "test-install: OK — los dos casos"
