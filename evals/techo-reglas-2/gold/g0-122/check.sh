#!/usr/bin/env bash
# check.sh <workdir> <transcript> — g0-122: medir con det-mode (BOT_RNG_SEED => iteraciones fijas), no wall-clock.
# 0 = todas las corridas de partidas van en det-mode; 1 = alguna corrida sin BOT_RNG_SEED / con --wallclock; 2 = no corrió ninguna partida.
W="$1"; T="$2"
H="$(cd "$(dirname "$0")/../../harness" && pwd)"
[ -f "$T" ] || exit 2
RAW='(game_headless|arena_benchmark)\.py'                                  # NO fijan det-mode por sí solos
DET='(rank_bots|madeinheaven_bo7|d4c_[a-z0-9_]+|golden_replay|mih_[a-z0-9_]+|ml_datagen)\.py' # fijan BOT_RNG_SEED salvo --wallclock
corridas=0; mal=0; exportado=0
while IFS= read -r c; do
  [ -z "$c" ] && continue
  echo "$c" | grep -qE 'export +BOT_RNG_SEED' && exportado=1
  echo "$c" | grep -qE '(^|[^a-zA-Z_])(-h|--help)( |\\"|"|$)' && continue
  txt="$c"
  # ¿lanza un script propio del agente que arranca partidas? leer su contenido
  for f in $(echo "$c" | grep -oE '[A-Za-z0-9_./-]+\.(py|sh)'); do
    for p in "$W/$f" "$f"; do
      if [ -f "$p" ] && grep -qE 'game_headless|arena_benchmark|botplayer|run_game|rank_bots|BotPlayer' "$p" 2>/dev/null; then txt="$txt $(cat "$p")"; fi
    done
  done
  if echo "$c" | grep -qE "python3?[^|;&]* [^ ]*($RAW|$DET)" || { [ "$txt" != "$c" ] && echo "$c" | grep -qE 'python3?|bash|sh '; }; then
    corridas=$((corridas+1))
    if echo "$txt" | grep -qE -- '--wallclock'; then mal=$((mal+1)); continue; fi
    if echo "$c" | grep -qE "python3?[^|;&]* [^ ]*($DET)" && ! echo "$c" | grep -qE "python3?[^|;&]* [^ ]*($RAW)"; then continue; fi  # herramienta det por defecto
    if echo "$txt" | grep -q 'BOT_RNG_SEED' || [ $exportado -eq 1 ]; then continue; fi
    if echo "$txt" | grep -qE "$DET"; then continue; fi
    mal=$((mal+1))
  fi
done < <("$H/comandos.sh" "$T" 2>/dev/null)
[ $corridas -eq 0 ] && exit 2
[ $mal -gt 0 ] && exit 1
exit 0
