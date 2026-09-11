#!/usr/bin/env bash
# Instalador de exo desde GitHub Releases.
#
# Deja el binario en $EXO_DIR (default ~/.local/bin) y en ningún otro sitio,
# porque ahí es donde el pre-commit de la KB lo busca literalmente
# (plugins/exo/scripts/kb-precommit.sh:18) y, si no está, ese gate sale 0:
# commit permitido, sin gate, sin romper nada. Instalar en otro punto del PATH
# apaga el gate sin que nadie se entere.
set -euo pipefail

REPO="${EXO_REPO:-pguerrerolinares/exo}"
EXO_DIR="${EXO_DIR:-$HOME/.local/bin}"
VERSION="${EXO_VERSION:-latest}"

necesito() {
  command -v "$1" >/dev/null 2>&1 || { echo "install: falta '$1' en el PATH" >&2; exit 1; }
}
necesito curl
necesito uname

case "$(uname -s)" in
  Linux)
    # El guard de arquitectura es simétrico al de Darwin, y por la misma razón:
    # la release solo publica x86_64 para Linux. Sin él, en un ARM el checksum
    # CUADRA —el fichero baja entero, solo que es de otra arquitectura—, el
    # binario se copia encima de `~/.local/bin/exo` y el fallo llega tarde y
    # mal, como un «Exec format error» del `--version`.
    case "$(uname -m)" in
      x86_64|amd64) target=x86_64-unknown-linux-gnu; bin=exo ;;
      *) echo "install: no hay binario publicado para Linux $(uname -m); compila desde fuente (docs/instalacion.md)" >&2; exit 1 ;;
    esac ;;
  Darwin)
    case "$(uname -m)" in
      arm64|aarch64) target=aarch64-apple-darwin; bin=exo ;;
      *) echo "install: no hay binario publicado para macOS Intel; compila desde fuente (docs/instalacion.md)" >&2; exit 1 ;;
    esac ;;
  MINGW*|MSYS*|CYGWIN*)
    target=x86_64-pc-windows-msvc; bin=exo.exe ;;
  *)
    echo "install: plataforma no soportada: $(uname -s)" >&2; exit 1 ;;
esac

asset="exo-$target"
case "$bin" in *.exe) asset="$asset.exe";; esac

# EXO_BASE_URL es un seam de test declarado (scripts/test-install.sh sirve una
# release falsa por file://). En uso normal no se pone.
if [ -n "${EXO_BASE_URL:-}" ]; then
  base="$EXO_BASE_URL"
elif [ "$VERSION" = latest ]; then
  base="https://github.com/$REPO/releases/latest/download"
else
  base="https://github.com/$REPO/releases/download/$VERSION"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "install: bajando $asset de $base"
curl -fsSL "$base/$asset"        -o "$tmp/$asset"
curl -fsSL "$base/$asset.sha256" -o "$tmp/$asset.sha256"

# El checksum se verifica SIEMPRE y sin `|| true`: un binario a medias que se
# ejecuta es peor que uno que no se instala. Y se verifica ANTES de copiar, no
# después, para que un fallo no deje nada en el destino.
if command -v shasum >/dev/null 2>&1; then
  ( cd "$tmp" && shasum -a 256 -c "$asset.sha256" )
elif command -v sha256sum >/dev/null 2>&1; then
  ( cd "$tmp" && sha256sum -c "$asset.sha256" )
else
  echo "install: ni shasum ni sha256sum — no puedo verificar el binario" >&2
  exit 1
fi

mkdir -p "$EXO_DIR"
cp "$tmp/$asset" "$EXO_DIR/$bin"
chmod 0755 "$EXO_DIR/$bin"
echo "install: instalado en $EXO_DIR/$bin"

case ":$PATH:" in
  *":$EXO_DIR:"*) ;;
  *) echo "install: AVISO — $EXO_DIR no está en tu PATH; añádelo a tu perfil" >&2 ;;
esac

"$EXO_DIR/$bin" --version

# `exo init` necesita --kb y --name, que un instalador no puede inventarse. Se
# encadena SOLO si el usuario los ha dado por entorno; si no, se imprime el
# comando. Divergencia declarada respecto a la spec de G5 ("encadena exo
# init"): inventar una KB por defecto sería peor que pedirla.
cfg="${EXO_CONFIG:-$HOME/.exo/config.toml}"
if [ -f "$cfg" ]; then
  echo "install: config ya existente en $cfg — no se toca"
elif [ -n "${EXO_INIT_KB:-}" ] && [ -n "${EXO_INIT_NAME:-}" ]; then
  "$EXO_DIR/$bin" init --kb "$EXO_INIT_KB" --name "$EXO_INIT_NAME"
else
  echo "install: no hay config en $cfg. Créala con:"
  echo "  $EXO_DIR/$bin init --kb <ruta-de-tu-kb> --name <nombre>"
fi

# doctor cierra la instalación diciendo qué falta en ESTA máquina. No hace
# fallar al instalador: su exit 3 significa "la máquina tiene deuda", no "la
# instalación falló". El informe se imprime entero, que es el punto.
# El exit 3 de doctor significa «esta máquina tiene deuda», no «la instalación
# falló», así que no se propaga — pero tampoco se traga en silencio:
# install.ps1 lo dice, y este no puede ser el más callado de los dos.
if ! "$EXO_DIR/$bin" doctor; then
  ec=$?
  echo "install: doctor ha marcado deuda (exit $ec) — mira las filas 'fail' de arriba" >&2
fi
