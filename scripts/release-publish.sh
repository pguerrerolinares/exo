#!/usr/bin/env bash
# Publica los binarios de un tag en su release de GitHub, de forma idempotente.
#
# Antes esto era un `gh release create` suelto dentro de release.yml, y por eso
# el `workflow_dispatch` de "re-publicar un tag ya existente" (release.yml:6-8)
# no servía para lo único que justificaba su existencia: si la release ya está
# creada —porque alguien la abrió a mano para escribir las notas, o porque un
# intento anterior la dejó a medias—, `create` muere con «a release with the
# same tag name already exists» y los binarios se quedan sin publicar con los
# tres builds en verde (medido: run 35521897798, v0.2.0).
#
# Reglas:
#   - release que no existe  -> se crea, con las notas genéricas de abajo.
#   - release que existe      -> se le SUBEN los assets y no se le tocan ni las
#     notas ni el título. Las notas de una release publicada suelen estar
#     escritas a mano y valen más que la plantilla.
#   - release que existe y YA tiene assets -> se PARA, salvo FORCE=1. Volver a
#     subir cambia el sha256 de binarios que alguien ya pudo descargar, y esos
#     hashes son justo lo que install.sh verifica.
#
# Seams de test (scripts/test-release-publish.sh los usa con un `gh` falso):
#   GH_BIN (default gh) · DIST (default dist) · FORCE (1 = reemplazar).
set -euo pipefail
cd "$(dirname "$0")/.." || exit 1

TAG="${1:-${TAG:-}}"
[ -n "$TAG" ] || { echo "release-publish: falta el tag (uso: $0 v1.2.3)" >&2; exit 1; }
GH_BIN="${GH_BIN:-gh}"
DIST="${DIST:-dist}"
REPO="${GITHUB_REPOSITORY:-}"
[ -n "$REPO" ] || { echo "release-publish: falta GITHUB_REPOSITORY" >&2; exit 1; }

# `find` y no un glob: un `"$DIST"/*` que no casa con nada se le pasaría a gh
# como el literal `dist/*`, y la release saldría a cero binarios con el job en
# verde.
assets=()
while IFS= read -r f; do assets+=("$f"); done < <(find "$DIST" -mindepth 1 -maxdepth 1 -type f | sort)
[ "${#assets[@]}" -gt 0 ] || {
  echo "release-publish: no hay ficheros que publicar en el dist '$DIST'" >&2; exit 1; }

# Una sola llamada resuelve las dos preguntas: el exit code dice si la release
# existe, y stdout cuántos assets tiene.
n="$("$GH_BIN" release view "$TAG" --repo "$REPO" --json assets --jq '.assets | length' 2>/dev/null)" || n=""

if [ -z "$n" ]; then
  echo "release-publish: $TAG no existe todavía — creándola con ${#assets[@]} ficheros"
  "$GH_BIN" release create "$TAG" "${assets[@]}" \
    --repo "$REPO" \
    --verify-tag \
    --title "exo $TAG" \
    --notes "Binarios para linux-x86_64, windows-x86_64 y macos-arm64, cada uno con su \`.sha256\`.

Instalación (requiere \`git\` y \`jq\`; ni Rust ni toolchain C):

\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/$REPO/$TAG/install.sh | bash
\`\`\`

En PowerShell:

\`\`\`powershell
irm https://raw.githubusercontent.com/$REPO/$TAG/install.ps1 | iex
\`\`\`

Detalle: \`docs/instalacion.md\`."
elif [ "$n" -eq 0 ]; then
  echo "release-publish: $TAG ya existe y está vacía — subiendo ${#assets[@]} ficheros (notas intactas)"
  "$GH_BIN" release upload "$TAG" "${assets[@]}" --repo "$REPO"
elif [ "${FORCE:-}" = 1 ]; then
  echo "release-publish: $TAG ya tiene $n assets y FORCE=1 — reemplazándolos"
  "$GH_BIN" release upload "$TAG" "${assets[@]}" --repo "$REPO" --clobber
else
  echo "release-publish: la release $TAG ya tiene $n assets publicados." >&2
  echo "  Reemplazarlos cambia los sha256 que install.sh verifica, así que no se hace solo." >&2
  echo "  Si de verdad quieres reemplazarlos, re-lanza el workflow con force=true." >&2
  exit 1
fi
