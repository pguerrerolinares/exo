#!/usr/bin/env bash
# Criterio de reapertura del proceso residente (H3, campaña A). READ-ONLY.
#
# La spec 2026-08-22-m6-06-recall-punto-de-uso-design.md §2.2 aceptó «el segundo
# por turno» con reapertura «si duele tras semanas de uso», sin mecanismo que
# detectara el dolor. Esto es ese mecanismo: lee los eventos de recall-inject.sh
# del log de ESTA máquina (W11 y Linux se evalúan por separado).
#
# Uso: recall-latencia.sh [DESDE] [HASTA]   (YYYY-MM-DD, inclusivas; por defecto todo)
# Umbrales HARDCODEADOS adrede: son pre-registro
# (docs/superpowers/plans/2026-09-13-campana-a-preregistro-bench.md §«Criterio de reapertura».)
#
# ENMIENDA (2026-09-19, decisión de Paul #12, campaña I): la métrica de p95
# pasa de `elapsed_ms + refresh_ms` (tiempo INTERNO del engine) a `hook_ms`
# (reloj de PARED del hook `recall-inject.sh` entero). En W11 el shell
# alrededor del binario cuesta tanto como el binario mismo
# (evals/recall-coste/results/w11-2026-09-15.txt: hook p50 2.312 ms frente a
# elapsed_ms+refresh_ms ~993-1.003 ms) -- con la métrica vieja este criterio
# nunca dispara en W11 aunque cada prompt cueste el doble. Umbral (1.500 ms),
# porcentaje de timeouts (2%) y mínimo de disparos (200) NO cambian.
#
# ENMIENDA (2026-09-20, review final de rama, campaña I): $to (timeouts) se
# ancla a `t0`, el `.ts` MÍNIMO de los `emitted` que llevan `hook_ms`, y solo
# cuenta timeouts con `ts >= t0`. Antes contaba TODOS los timeout-guard del
# rango mientras $n solo contaba los `emitted` con `hook_ms` -- sobre un log
# que cruza el despliegue de esta campaña, eso infla el ratio (timeouts
# viejos sobre un denominador que ya solo son disparos nuevos) y puede dar
# REABRIR falso. `t0` es por máquina porque el log entero lo es: no hace
# falta coordinar fechas entre W11 y Linux. Si ningún `emitted` lleva
# `hook_ms` (log íntegro pre-campaña, o instrumento apagado), $to = 0 y el
# veredicto es INSUFICIENTE -- correcto, no hay nada que decidir todavía.
# `DESDE` sigue existiendo para acotar a mano, pero deja de ser la única red.
set -uo pipefail
export LC_NUMERIC=C

LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
DESDE="${1:-0000-00-00}"
HASTA="${2:-9999-12-31}"
UMBRAL_P95_MS=1500
UMBRAL_TIMEOUT_PCT=2
MIN_DISPAROS=200

command -v jq >/dev/null 2>&1 || { echo "jq requerido" >&2; exit 1; }
[ -f "$LOG" ] || { echo "no existe $LOG" >&2; exit 1; }
jq -e . "$LOG" >/dev/null 2>&1 || { echo "ERROR: $LOG contiene JSON inválido" >&2; exit 1; }

jq -rs --arg d "$DESDE" --arg h "${HASTA}T23:59:59Z" \
  --argjson p95max "$UMBRAL_P95_MS" --argjson tmax "$UMBRAL_TIMEOUT_PCT" --argjson nmin "$MIN_DISPAROS" '
  def campo($k): ((capture("(^| )" + $k + "=(?<v>[0-9]+)") | .v | tonumber) // null);
  [ .[] | select(.ts >= $d and .ts <= $h)
        | select((.session_id // "") | ascii_downcase | startswith("test") | not) ] as $ev
  | [ $ev[] | select(.reflex == "recall-inject-emitted")
      | {ts: .ts, hook_ms: ((.payload // "") | campo("hook_ms"))} ] as $emitted
  | [ $emitted[] | select(.hook_ms != null) ] as $conhook
  | ([ $conhook[] | .ts ] | min) as $t0
  | ([ $conhook[] | .hook_ms ] | sort) as $ms
  | ($ms | length) as $n
  | (if $t0 == null then 0
     else [ $ev[] | select(.reflex == "recall-inject-degraded"
                        and ((.payload // "") | contains("reason=timeout-guard"))
                        and (.ts >= $t0)) ] | length
     end) as $to
  | (if $n == 0 then null else $ms[(($n - 1) * 0.5 | floor)] end) as $p50
  | (if $n == 0 then null else $ms[(($n - 1) * 0.95 | floor)] end) as $p95
  | (if ($n + $to) == 0 then 0 else ($to * 100 / ($n + $to)) end) as $tpct
  | "disparos_medidos\t\($n)",
    "timeouts\t\($to)",
    "p50_ms\t\($p50)",
    "p95_ms\t\($p95)",
    "timeout_pct\t\($tpct)",
    ( if ($n + $to) < $nmin then "veredicto\tINSUFICIENTE (menos de \($nmin) disparos)"
      elif ($p95 != null and $p95 > $p95max) or $tpct > $tmax then "veredicto\tREABRIR"
      else "veredicto\tNO-REABRIR" end )
' "$LOG"
