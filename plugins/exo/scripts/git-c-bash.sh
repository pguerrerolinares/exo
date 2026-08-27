#!/usr/bin/env bash
# PreToolUse (matcher: Bash): reflejo "git -C en vez de cd && git".
# NUNCA bloquea (exit 0 siempre). El anti-patron `cd <path> && git ...` dispara
# prompts de permiso manuales innecesarios (ver CLAUDE.md). Dos modos:
#   1. REWRITE silencioso (updatedInput+allow, log `git-c-rewrite`): solo si el comando
#      ENTERO parsea como `cd PATH && git <read-only>` con PATH literal y REST limpio.
#   2. WARN (additionalContext, log `git-c`): el resto de matches, status quo.
# Abstencion: cualquier otra cosa -> silencio. SIN sentinel: cada ocurrencia es un caso
# real y corregible (avisa por-ocurrencia, como el reflejo #1).
# Aplica en padre Y subagentes (la leccion es universal) -> sin guarda agent_id.
# Capa TRIGGER / clase event-watching del proyecto cerebro+reflejos.
set -uo pipefail

INPUT="$(cat)"
command -v jq >/dev/null 2>&1 || exit 0

CMD="$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)" || CMD=""
[ -z "$CMD" ] && exit 0

# ---- Rama 1: REWRITE silencioso (HECHO parseado, alta confianza) ----
# Solo si el comando ENTERO es `cd PATH && git REST` con PATH literal (sin quotes/
# vars/subst), REST sin metacaracteres de shell y subcomando git de SOLO LECTURA.
# Un rewrite sobre FP corrompe un comando legítimo -> sesgo fuerte a falsos negativos;
# todo lo no-elegible cae a la rama 2 (warn, status quo).
REWRITE_ALLOWLIST='status|log|diff|show|rev-parse|describe|ls-files|blame|shortlog|reflog|grep'
REWRITE_RE='^cd[[:space:]]+([A-Za-z0-9._/~-]+)[[:space:]]*&&[[:space:]]*git[[:space:]]+(.+)$'
NEW_CMD=""
case "$CMD" in
  *$'\n'*) : ;;  # multiline: nunca rewrite
  *)
    if [[ "$CMD" =~ $REWRITE_RE ]]; then
      CD_PATH="${BASH_REMATCH[1]}"
      GIT_REST="${BASH_REMATCH[2]}"
      # PATH que empieza por '-' es arg especial de cd (`-`=OLDPWD, `--`=home) que
      # git -C tomaría literal -> nunca rewrite.
      # PATH con componente '..' -> nunca rewrite: `cd` resuelve '..' LÓGICO (textual)
      # y `git -C` hace chdir() FÍSICO; con symlinks divergen (puede acabar en OTRO
      # repo en silencio — review adversarial 2026-07-05).
      # PATH relativo con CDPATH seteado -> nunca rewrite: cd lo resolvería vía CDPATH,
      # git -C no.
      # REST sin metacaracteres: pipe/redirect/chain/subst cambiarían semántica con -C,
      # y los de glob (* ? [) se expanden en el cwd ORIGINAL, no en PATH -> no equivalente.
      # Subcomando en la allowlist de solo-lectura (allow salta el permission prompt:
      # auto-aprobar mutaciones ampliaría autoridad).
      case "$CD_PATH" in
        -*) : ;;
        ..|../*|*/..|*/../*) : ;;
        [!/~]*) [ -z "${CDPATH:-}" ] && REWRITE_OK=1 ;;
        *) REWRITE_OK=1 ;;
      esac
      if [ "${REWRITE_OK:-0}" = "1" ] \
         && ! printf '%s' "$GIT_REST" | grep -q '[&;|<>`$()*?[]' \
         && printf '%s' "$GIT_REST" | grep -Eq "^(${REWRITE_ALLOWLIST})([[:space:]]|\$)"; then
        NEW_CMD="git -C ${CD_PATH} ${GIT_REST}"
      fi
    fi
    ;;
esac

if [ -n "$NEW_CMD" ]; then
  # log del rewrite (id propio para la telemetría; best-effort)
  . "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "git-c-rewrite" "$INPUT" "${CMD} -> ${NEW_CMD}" || true
  # updatedInput reemplaza el objeto ENTERO: se parte de .tool_input completo.
  # permissionDecisionReason con "allow" solo lo ve el humano (visibilidad en UI).
  if printf '%s' "$INPUT" | jq -c --arg new "$NEW_CMD" \
      '{hookSpecificOutput:{hookEventName:"PreToolUse",
        permissionDecision:"allow",
        permissionDecisionReason:("Reflejo git-c: reescrito a `" + $new + "` (equivalente; evita el cd encadenado)"),
        updatedInput:(.tool_input | .command = $new)}}' 2>/dev/null; then
    exit 0
  fi
  # si jq falló, cae a la rama 2 (warn) — never-break
fi

# ---- Rama 2: WARN (status quo, telemetría de lo que el rewrite no cubre) ----
# Anti-patron: `cd <algo> (&&|;) git ...` -> git como comando justo tras el separador.
# `[^&;|]+` no cruza separadores: solo el `cd X && git` limpio (donde git -C es swap directo)
# dispara; `cd X && make && git` NO (ahi el cd hace falta para make). Sesga a falsos negativos.
printf '%s' "$CMD" | grep -Eq 'cd[[:space:]]+[^&;|]+(&&|;)[[:space:]]*git([[:space:]]|$)' || exit 0

# log del disparo (best-effort, nunca rompe el warn-only)
. "$(dirname "$0")/_reflex-log.sh" 2>/dev/null && reflex_log "git-c" "$INPUT" "$CMD" || true

exit 0
