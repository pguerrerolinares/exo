#!/usr/bin/env bash
# Pre-commit gate de la KB kb-demo (spec F1.b).
#
# Juzga el INDEX, no el working tree: `kbx ratchet --staged` lee lo que el
# commit va a contener. Sin eso hay dos fallos — rechazos por una nota que otra
# sesión edita en paralelo, y (peor) falsos OK, porque stagear un techo subido y
# restaurar el fichero en disco mete la subida en HEAD en verde, y el fichero de
# sellos solo vigila la transición: lo que entra queda blanqueado.
#
# Instalar:  ln -sf <este fichero> <kb>/.git/hooks/pre-commit
# Saltar:    git commit --no-verify   (declarado: es un gate contra el descuido)
set -uo pipefail

KB="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0
KBX="${KBX_BIN:-$HOME/.local/bin/kbx}"

[ -x "$KBX" ] || { echo "kb-precommit: no encuentro kbx en $KBX — commit permitido" >&2; exit 0; }

fail=0

# --- Trinquete: los techos de waiver solo bajan --------------------------------
if ! out="$("$KBX" ratchet --kb "$KB" --staged 2>&1)"; then
  echo "$out" >&2
  fail=1
fi

# --- Presupuestos: sobre el snapshot staged, no sobre el disco -----------------
# checkout-index materializa exactamente el index; así un offender que solo
# existe en el working tree (otra sesión a medias) no bloquea este commit.
snap="$(mktemp -d)"
trap 'rm -rf "$snap"' EXIT
if git -C "$KB" checkout-index -a --prefix="$snap/" 2>/dev/null; then
  if ! out="$("$KBX" budget --kb "$snap" 2>&1)"; then
    echo "$out" >&2
    fail=1
  fi
else
  echo "kb-precommit: no pude materializar el index; presupuestos no verificados" >&2
fi

[ "$fail" -eq 0 ] && exit 0

cat >&2 <<'EOF'

────────────────────────────────────────────────────────────────────────
El gate de la KB ha rechazado este commit. QUÉ NO HACER:

  ✗ NO subas kbx_budget_max ni edites .kbx-ratchet.json para que pase.
    El techo solo baja: subirlo es exactamente lo que este gate existe
    para impedir.
  ✗ NO recortes la nota a lo bruto para que quepa. Perder el matiz es
    peor que el commit rechazado.

QUÉ HACER, en orden:

  1. Si la nota creció con histórico: PÁRTELA. Mueve lo fechado a su
     bitácora (log/<slug>-bitacora.md). El canon se queda con el destilado.
  2. Si la bitácora es la que ha crecido: kbx rotate --kb <kb> --apply
     archiva su cola fría en archive/log/.
  3. Si nada de eso aplica: deja el commit pendiente y díselo a Paul.
     Un commit sin hacer se arregla en un minuto; una nota mutilada, no.
────────────────────────────────────────────────────────────────────────
EOF
exit 1
