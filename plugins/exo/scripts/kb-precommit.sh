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
# Instalar:  el shim de docs/instalar-con-agente.md («Gate de pre-commit de la KB»)
#            en <kb>/.git/hooks/pre-commit + chmod +x. Resuelve el plugin instalado:
#            sigue sus actualizaciones y vale en Windows, donde `ln -s` copia.
# Saltar:    git commit --no-verify   (declarado: es un gate contra el descuido)
set -uo pipefail

KB="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

# Resolución del binario, mismo orden de precedencia que usan los hooks del
# plugin (exo-recall.sh:34): EXO_BIN > `command -v exo`
# > $HOME/.local/bin/exo(.exe). El PATH gana sobre el literal de ~/.local/bin
# porque así resuelven los hooks; divergir aquí significa que el pre-commit
# y los hooks pueden acabar usando binarios distintos en la misma máquina
# (bug de precedencia detectado en review, fix de H).
if [ -n "${EXO_BIN:-}" ]; then
  EXO="$EXO_BIN"
elif command -v exo >/dev/null 2>&1; then
  EXO="$(command -v exo)"
elif [ -x "$HOME/.local/bin/exo" ]; then
  EXO="$HOME/.local/bin/exo"
elif [ -x "$HOME/.local/bin/exo.exe" ]; then
  EXO="$HOME/.local/bin/exo.exe"
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

  # Reglas duras (CAP por sección): exit 2 = engine anterior sin `rules --check`
  # (clap), solo avisa; 3 = excesos; cualquier otro exit es fallo del engine.
  out="$("$EXO" rules --check --kb "$snap" 2>&1)"; rc=$?
  case "$rc" in
    0) ;;
    2) echo "kb-precommit: el engine no conoce «rules --check»; reglas duras sin verificar" >&2 ;;
    *) echo "$out" >&2; fail=1 ;;
  esac
else
  echo "kb-precommit: no pude materializar el index; presupuestos no verificados" >&2
fi

# --- Frontmatter: tier (siempre) y tags (solo notas nuevas) --------------------
# `exo lint` no sirve aquí: no mira tags ni wikilinks, exige el índice y juzga
# el working tree, no el index. Se lee el frontmatter de lo staged con
# `git show :ruta`. En notas modificadas la falta de tags solo avisa: hay notas
# históricas sin tags y bloquearlas frenaría cualquier edición.
fm_campos() { # stdin: nota; imprime "tier" y/o "tags" si están presentes y válidos
  awk '
    NR == 1 { if ($0 != "---") exit; next }
    /^---[ \t]*$/ { exit }
    /^tier:[ \t]*(core|stable|log)[ \t]*$/ { print "tier" }
    /^tags:/ {
      v = $0; sub(/^tags:[ \t]*/, "", v)
      if (v != "" && v !~ /^\[[ \t]*\]$/) print "tags"
      else entags = 1
      next
    }
    entags && /^[ \t]*-[ \t]*[^ \t]/ { print "tags"; entags = 0; next }
    entags && !/^[ \t]/ { entags = 0 }
  '
}
while IFS= read -r -d '' estado && IFS= read -r -d '' ruta; do
  case "$ruta" in *.md) ;; *) continue ;; esac
  # .distill/ es el sidecar de auto-distill (skills/document/auto-distill.md):
  # bloques movidos verbatim, sin frontmatter, que se anexan a la bitácora.
  case "$ruta" in docs/*|.distill/*|README.md|AGENTS.md) continue ;; esac
  campos="$(git -C "$KB" show ":$ruta" 2>/dev/null | fm_campos)"
  case "$campos" in *tier*) ;; *) echo "$ruta: falta tier" >&2; fail=1 ;; esac
  case "$campos" in
    *tags*) ;;
    *)
      if [ "$estado" = "A" ]; then
        echo "$ruta: falta tags" >&2; fail=1
      else
        echo "kb-precommit: aviso: $ruta no declara tags" >&2
      fi ;;
  esac
done < <(git -C "$KB" diff --cached --name-status -z --diff-filter=AM)

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
  3. Si nada de eso aplica: deja el commit pendiente y díselo al dueño de la KB.
     Un commit sin hacer se arregla en un minuto; una nota mutilada, no.
────────────────────────────────────────────────────────────────────────
EOF
exit 1
