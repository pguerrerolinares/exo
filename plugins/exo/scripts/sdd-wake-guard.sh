#!/usr/bin/env bash
# Stop hook (#111): un orquestador SDD no cierra turno con tareas de fondo vivas
# y sin un cron de respaldo armado.
#
# Por qué: el aviso de fin de una tarea de fondo lanzada por un teammate que ya
# está idle se encola en la cola del lead y no reinvoca a nadie (Claude Code
# 2.1.303; reproducido 3 veces). En bizkaia-now 2026-10-18 eso paró una ola
# 6 h 56 min. Las tareas de fondo del propio lead sí lo despiertan, pero el
# hook no sabe de quién es cada una: `background_tasks` no trae dueño.
#
# Qué hace: si hay un ledger SDD activo (`.superpowers/sdd/*/progress.md`
# tocado en las últimas 6 h), alguna tarea de fondo no-teammate en `running`
# y `session_crons` vacío ⇒ `decision: block` pidiendo un CronCreate one-shot
# que siga `skills/orchestrate/despertar.md`. El cron sobrevive a `/compact`
# (`session_cron_carry`) y dispara aunque un teammate esté ocupado; no
# sobrevive a cerrar la sesión.
#
# Abstención: subagentes (`agent_id`), `stop_hook_active`, sin jq, sin ledger
# fresco, sin tareas vivas, o con algún cron ya armado. Los teammates salen
# en `background_tasks` como `running` aunque estén idle: se ignoran.
set -uo pipefail

INPUT="$(cat)"
[[ "$INPUT" =~ \"agent_id\"[[:space:]]*:[[:space:]]*\"[^\"]+\" ]] && exit 0
command -v jq >/dev/null 2>&1 || exit 0

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DESPERTAR="$(cd "$SCRIPT_DIR/../skills/orchestrate" 2>/dev/null && pwd)/despertar.md"
LEDGER_FRESH_MIN="${SDD_WAKE_LEDGER_FRESH_MIN:-360}"

ROW="$(printf '%s' "$INPUT" | jq -r '[
    (.stop_hook_active // false),
    (.cwd // ""),
    ((.session_crons // []) | length),
    ([(.background_tasks // [])[] | select(.type != "teammate" and .status == "running") | (.id // "?")] | join(", "))
  ] | map(tostring) | join("\u001f")' 2>/dev/null)" || exit 0
ROW="${ROW%$'\r'}"
# Separador no-blanco: con tab, un cwd vacío desplazaría los campos.
IFS=$'\x1f' read -r ACTIVE CWD NCRONS LIVE <<< "$ROW"

[ "$ACTIVE" = "true" ] && exit 0
[ -z "$LIVE" ] && exit 0
[ "${NCRONS:-0}" -gt 0 ] 2>/dev/null && exit 0
[ -z "$CWD" ] && CWD="$PWD"
[ -d "$CWD/.superpowers/sdd" ] || exit 0
LEDGER="$(find "$CWD/.superpowers/sdd" -mindepth 2 -maxdepth 2 -name progress.md -mmin "-$LEDGER_FRESH_MIN" 2>/dev/null | head -n 1)"
[ -z "$LEDGER" ] && exit 0

# shellcheck source=_reflex-log.sh
. "$SCRIPT_DIR/_reflex-log.sh" 2>/dev/null && { reflex_log "sdd-wake-guard" "$INPUT" "live=${LIVE} ledger=${LEDGER}" || true; }

REASON="Hay tareas de fondo vivas (${LIVE}) y ningún cron armado. Si alguna es de un teammate, su aviso no despertará a nadie. Antes de cerrar turno, arma un CronCreate one-shot a +N min (N ≤ 20, ajustado a lo que tarde la tarea) con este prompt: «Despertar de orquestación: sigue ${DESPERTAR}». Después cierra turno."
jq -n --arg r "$REASON" '{decision: "block", reason: $r}'
