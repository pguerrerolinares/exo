#!/usr/bin/env bash
# Pre-commit gate de la KB kb-demo (spec F1.b).
#
# Juzga el INDEX, no el working tree: `exo ratchet --staged` lee lo que el
# commit va a contener. Sin eso hay dos fallos — rechazos por una nota que otra
# sesión edita en paralelo, y (peor) falsos OK, porque stagear un techo subido y
# restaurar el fichero en disco mete la subida en HEAD en verde, y el fichero de
# sellos solo vigila la transición: lo que entra queda blanqueado.
#
# Desde G4c (2026-09-10) el gate depende de `exo`, no de `kbx` — cutover de
# invocaciones de binario, el formato de `.kbx-ratchet.json` no cambia.
#
# Instalar:  ln -sf <este fichero> <kb>/.git/hooks/pre-commit
# Saltar:    git commit --no-verify   (declarado: es un gate contra el descuido)
set -uo pipefail

KB="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

# Resolución del binario, mismo orden de precedencia que usan los hooks del
# plugin (exo-recall.sh/recall-inject.sh): EXO_BIN > $HOME/.local/bin/exo(.exe)
# > `command -v exo`. El literal de ~/.local/bin va primero porque es el sitio
# que `exo doctor` (`hook_fallback_binary`) sabe reportar por nombre; el PATH
# es el último recurso, no el default (review final de H, I2: antes esto no
# miraba el PATH en absoluto y bloqueaba con un `exo` bien instalado pero
# resuelto solo por PATH).
if [ -n "${EXO_BIN:-}" ]; then
  EXO="$EXO_BIN"
elif [ -x "$HOME/.local/bin/exo" ]; then
  EXO="$HOME/.local/bin/exo"
elif [ -x "$HOME/.local/bin/exo.exe" ]; then
  EXO="$HOME/.local/bin/exo.exe"
elif command -v exo >/dev/null 2>&1; then
  EXO="$(command -v exo)"
else
  EXO="$HOME/.local/bin/exo"
fi

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

fail=0

# --- Trinquete: los techos de waiver solo bajan --------------------------------
if ! out="$("$EXO" ratchet --kb "$KB" --staged 2>&1)"; then
  echo "$out" >&2
  fail=1
fi

# --- Presupuestos: sobre el snapshot staged, no sobre el disco -----------------
# checkout-index materializa exactamente el index; así un offender que solo
# existe en el working tree (otra sesión a medias) no bloquea este commit.
snap="$(mktemp -d)"
trap 'rm -rf "$snap"' EXIT
if git -C "$KB" checkout-index -a --prefix="$snap/" 2>/dev/null; then
  if ! out="$("$EXO" budget --kb "$snap" 2>&1)"; then
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
  2. Si la bitácora es la que ha crecido: exo rotate --kb <kb> --apply
     archiva su cola fría en archive/log/.
  3. Si nada de eso aplica: deja el commit pendiente y díselo a Paul.
     Un commit sin hacer se arregla en un minuto; una nota mutilada, no.
────────────────────────────────────────────────────────────────────────
EOF
exit 1
