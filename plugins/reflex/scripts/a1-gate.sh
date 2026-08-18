#!/usr/bin/env bash
# A1 — computo mecanico del gate (docs/superpowers/evals/2026-08-02-a1-gate.md,
# spec transporte §7). READ-ONLY: nunca muta el log ni los transcripts/meta.json.
# Uso: a1-gate.sh <DESDE> <HASTA>  (fechas YYYY-MM-DD, ventana inclusiva).
# Umbrales HARDCODEADOS adrede: son pre-registro del gate doc, no config.
set -uo pipefail
export LC_NUMERIC=C   # el locale del sistema (p.ej. es_ES.UTF-8) usa coma decimal en
                       # awk printf/comparaciones; fuerza punto para no romper el parseo.

DESDE="${1:-}"
HASTA="${2:-}"
if [ -z "$DESDE" ] || [ -z "$HASTA" ]; then
  echo "uso: a1-gate.sh <DESDE> <HASTA>  (YYYY-MM-DD)" >&2
  exit 1
fi

LOG="${REFLEX_LOG_FILE:-$HOME/.claude/reflex-log.jsonl}"
PROJ="${REFLEX_PROJECTS_DIR:-$HOME/.claude/projects}"
CANARY_TOKEN="RFX-A1-K3P7"   # gate doc: token fijo pre-registrado (Global Constraints del plan)
U2_UMBRAL="0.3345"

command -v jq >/dev/null 2>&1 || { echo "jq requerido" >&2; exit 1; }
[ -f "$LOG" ] || { echo "no existe $LOG" >&2; exit 1; }
# log vacio es un caso distinto de log corrupto: mensaje propio antes del check de validez JSON.
[ -s "$LOG" ] || { echo "log vacío (0 líneas)" >&2; exit 1; }
jq -e . "$LOG" >/dev/null 2>&1 || { echo "ERROR: $LOG contiene JSON invalido" >&2; exit 1; }

# -u (UTC): el numerador filtra el log por .ts, que es UTC (ver HASTA_TS abajo);
# si esto se calculara en hora LOCAL (p.ej. CEST +0200) los dos relojes quedarian
# desalineados 1-2h en ambos bordes de la ventana (skew de zona horaria, M1).
FROM_EPOCH="$(date -u -d "$DESDE 00:00:00" +%s 2>/dev/null)" || { echo "DESDE invalido: $DESDE" >&2; exit 1; }
TO_EPOCH="$(date -u -d "$HASTA 23:59:59" +%s 2>/dev/null)" || { echo "HASTA invalido: $HASTA" >&2; exit 1; }
HASTA_TS="${HASTA}T23:59:59Z"

FILTERED="$(mktemp)"
trap 'rm -f "$FILTERED"' EXIT

# Ventana + mismo filtro de sesiones-test que reflex-baseline.sh/u2-baseline-pre.sh:
# session_id que empieza por "test" (case-insensitive) o payload con "LIVE-TEST".
jq -c --arg d "$DESDE" --arg h "$HASTA_TS" '
  select(.ts >= $d and .ts <= $h)
  | select(((.session_id // "") | ascii_downcase | startswith("test") | not)
           and (((.payload // "") | contains("LIVE-TEST")) | not))
' "$LOG" > "$FILTERED"

# --- Conteo por AGENTE, no por evento (M5) ---
# Un agente continuado con SendMessage redispara SubagentStart: el hook emite
# otra vez con el MISMO agent_id, mientras el filesystem sigue teniendo UN solo
# agent-<id>.meta.json. Contar eventos cruzaba eventos (numerador) contra
# agentes (denominador de ENTREGA) y daba entrega_pct >100% — medido en el
# historico real del inyector: 44 eventos para 16 agentes unicos (2,75x), con
# un agente redisparado 12 veces. Los tres sesgos que eso introducia empujaban
# en la MISMA direccion (PASS): ENTREGA inflada por encima de 100 enmascarando
# misses reales, denominador de U2 inflado => violaciones/dispatch a la baja, y
# transcripts de agentes conversados ponderados N veces en U1. La unidad de las
# tres metricas es el DISPATCH = un agente, asi que se cuentan agentes
# distintos. Pre-registrado antes de ver un dato de la ventana (DESDE=2026-08-04).
#
# agent_id vacio: no es deduplicable (no se sabe si dos lineas son el mismo
# agente), asi que cada evento sin agent_id cuenta como uno — conservador en la
# direccion incomoda: si el hook dejara de poblar agent_id, ENTREGA no se
# "arregla" sola por colapso a 1.
cuenta_agentes() { # <evento> -> agentes distintos con >=1 evento de ese tipo
  jq -s --arg ev "$1" '
    [.[] | select(.reflex==$ev)] as $l
    | ([$l[] | select((.agent_id // "")!="") | .agent_id] | unique | length)
      + ([$l[] | select((.agent_id // "")=="")] | length)
  ' "$FILTERED"
}
emitidos="$(cuenta_agentes inject-emitted)"
fallidos="$(cuenta_agentes inject-failed)"
skipped_depth="$(cuenta_agentes inject-skipped-depth)"
abstained="$(cuenta_agentes inject-abstained)"
violaciones_subagente="$(jq -s '[.[] | select((.agent_id // "")!="" and (.reflex=="git-c" or .reflex=="git-c-rewrite" or .reflex=="zero-residuo"))] | length' "$FILTERED")"
sesiones_con_compact="$(jq -s -r '[.[] | select(.reflex=="compact") | .session_id] | unique | length' "$FILTERED")"
# sesiones = distinct session_id que alimentan el denominador de U2 (inject-emitted/-failed/-skipped-depth);
# es el mismo n de sesiones que exige el n-minimo de U2 (>=5 sesiones), no el total de actividad de la ventana.
sesiones="$(jq -s -r '[.[] | select(.reflex=="inject-emitted" or .reflex=="inject-failed" or .reflex=="inject-skipped-depth") | .session_id] | unique | length' "$FILTERED")"
# tipos: informativo, tambien por agente distinto (M5) — si no, un executor
# conversado 12 veces dominaria el desglose y sugeriria un mix de trafico falso.
tipos="$(jq -s -r '
  [.[] | select(.reflex=="inject-emitted")] as $l
  | (($l | map(select((.agent_id // "")!="")) | unique_by(.agent_id))
     + ($l | map(select((.agent_id // "")==""))))
  | group_by(.agent_type) | map("\(.[0].agent_type):\(length)") | join(",")' "$FILTERED")"

# --- dispatches_fs: TODOS los agent-*.meta.json con mtime en ventana (M2) ---
# La politica de profundidad de la spec (spawnDepth>1 => no inyectar, §5.3) no
# existe en produccion: SubagentStart corre ANTES de que el harness escriba el
# meta.json, asi que el fallback "meta ausente => inyectar" gana siempre
# (verificado: 0 lineas inject-skipped-depth en todo el historico real; y
# reejecutando el hook a mano con el meta ya en disco SI hace skip). Se inyecta
# a TODOS los dispatches, incluidos nietos (spawnDepth>1) que "emitidos" ya
# cuenta — filtrar aqui por spawnDepth<=1 excluia esos nietos del denominador
# y no del numerador, inflando entrega_pct. Elegibles = todos.
dispatches_fs=0
if [ -d "$PROJ" ]; then
  while IFS= read -r -d '' m; do
    mt="$(stat -c %Y "$m" 2>/dev/null)" || continue
    [ "$mt" -ge "$FROM_EPOCH" ] && [ "$mt" -le "$TO_EPOCH" ] || continue
    # mismo filtro de sesiones-test que el resto del gate: excluye dispatches
    # bajo directorios de sesion "test*" (case-insensitive) del denominador de
    # ENTREGA, para no contaminarlo con trafico de fixtures/pruebas.
    sess="$(basename "$(dirname "$(dirname "$m")")")"
    case "$(printf '%s' "$sess" | tr '[:upper:]' '[:lower:]')" in test*) continue ;; esac
    dispatches_fs=$((dispatches_fs+1))
  done < <(find "$PROJ" -path "*/subagents/agent-*.meta.json" -print0 2>/dev/null)
fi

# --- U1: join por LOG (I3), no grep por filesystem. Para cada inject-emitted
# del log con agent_id!="" en ventana, localiza el transcript de hijo
# <projects>/*/<session_id>/subagents/agent-<agent_id>.jsonl; si existe Y
# contiene la linea de canario (confirmacion de que el canario estaba activo
# cuando se genero ese transcript), cuenta como recibido. De esos, cuantos
# citan el token en alguna de las superficies de CIERRE (no "cualquiera"):
# ver el criterio de union detallado justo abajo.
#
# Que cuenta como "citado" (M3 — operacionalizacion de "mensaje final"):
# UNION de los dos candidatos de cierre. Un transcript cuenta como citado si
# el token aparece en el ULTIMO bloque .type=="text" de assistant O en el
# ULTIMO tool_use .name=="SendMessage" (.input.message + .input.content,
# concatenados si ambos existen; SE DESCARTA .input.summary a proposito, es
# un recap corto, no el mensaje). Basta con que aparezca en cualquiera de
# los dos — no hace falta que aparezca en ambos.
#
# Por que union y no "el ultimo cronologico" ni "solo SendMessage si existe"
# (las dos variantes que se probaron y descartaron antes de llegar a esta):
# verificado contra transcripts reales (incluido
# agent-asmoke-c2-teammate-09de8373c18309d2.jsonl, sesion
# 0f5135bc-4be4-4686-82d1-8f8009af14f0, mas 5 transcripts adicionales con
# SendMessage bajo ~/.claude/projects) que en la ruta de equipo, tras el
# SendMessage, el agente SIEMPRE cierra con un turno de texto adicional (es
# el cierre normal del loop agentico: el harness requiere terminar en un
# turno sin tool_use). Eso da DOS superficies de cierre en la ruta de
# equipo, y "en tu mensaje final" —lo que pide literalmente el canario— es
# ambiguo entre ambas desde el punto de vista del agente que la recibe:
#   - "ultimo cronologico puro" (texto, por venir despues) deja la rama
#     SendMessage inerte — reproduce el bug original que se queria arreglar.
#   - "solo el ultimo SendMessage si existe" invierte el sesgo: un agente
#     que pone el token en su texto de cierre (y no en el SendMessage) se
#     contaria como NO citado, pese a haber obedecido la instruccion del
#     canario de forma perfectamente razonable — mide DONDE puso el token
#     en vez de SI honro la instruccion al cerrar, que es lo que U1 existe
#     para medir.
# La union mide "si", no "donde". En la ruta del tool Agent solo hay una
# superficie de cierre (el texto) y el comportamiento no cambia respecto al
# jq original.
#
# Esta operacionalizacion es MAS PERMISIVA que el jq original (puede contar
# como citado un caso que antes no lo era; nunca al reves) y por tanto
# empuja U1 al ALZA — en la direccion opuesta al fix de M2 (elegibles = todos
# los dispatches), que empuja ENTREGA a la BAJA. Los dos ajustes van en
# direcciones opuestas y quedan pre-registrados en este comentario ANTES de
# ver un solo dato de la ventana de medicion: es lo que separa calibrar el
# instrumento de amañar el resultado.
#
# El alcance sigue siendo estricto: el ULTIMO de cada tipo, no "el token
# aparece en cualquier parte del transcript". Un token citado a mitad de la
# ejecucion y abandonado al final no cuenta.
#
# u1_sin_transcript (M4): un inject-emitted cuyo transcript ya no existe
# (poda/retencion) antes desaparecia del computo sin dejar rastro — el glob
# "for tf in ...; do [ -f "$tf" ] || continue; done" simplemente no itera (sin
# nullglob, bash expande el patron a si mismo literal si no hay match) y
# u1_jq_fallidos solo captura jq roto, no fichero ausente. Se usa un flag
# por-par (found_tf) en vez de contar iteraciones, precisamente por ese
# gotcha del glob-que-no-matchea.
u1_recibieron=0
u1_citaron=0
u1_jq_fallidos=0
u1_sin_transcript=0
while IFS=$'\t' read -r sid aid; do
  [ -n "$sid" ] && [ -n "$aid" ] || continue
  found_tf=0
  for tf in "$PROJ"/*/"$sid"/subagents/"agent-${aid}.jsonl"; do
    [ -f "$tf" ] || continue
    found_tf=1
    grep -q "Marca de medicion" "$tf" 2>/dev/null || continue
    u1_recibieron=$((u1_recibieron+1))
    # last_candidates = ultimo bloque text + salto de linea + ultimo payload
    # SendMessage (union, ver comentario arriba); el separador evita que el
    # token se cuele por el borde de la concatenacion sin estar realmente
    # completo en ninguno de los dos (CANARY_TOKEN no contiene saltos de
    # linea, asi que no hay forma de que el separador lo reconstruya).
    if last_candidates="$(jq -s -r '
        [.[] | select(.type=="assistant") | .message.content[]?] as $items
        | ([$items[] | select(.type=="text") | .text] | last // "") as $last_text
        | ([$items[] | select(.type=="tool_use" and .name=="SendMessage")] | last) as $last_sm_obj
        | ( if $last_sm_obj != null then
              ([$last_sm_obj.input.message, $last_sm_obj.input.content] | map(select(. != null)) | join("\n"))
            else "" end
          ) as $last_sm
        | $last_text + "\n" + $last_sm
      ' "$tf" 2>/dev/null)"; then
      case "$last_candidates" in *"$CANARY_TOKEN"*) u1_citaron=$((u1_citaron+1)) ;; esac
    else
      u1_jq_fallidos=$((u1_jq_fallidos+1))
    fi
  done
  [ "$found_tf" -eq 1 ] || u1_sin_transcript=$((u1_sin_transcript+1))
done < <(jq -s -r '[.[] | select(.reflex=="inject-emitted" and (.agent_id // "")!="")
                    | [.session_id, .agent_id]] | unique | .[] | @tsv' "$FILTERED")

# --- porcentajes/ratios ---
if [ "$dispatches_fs" -gt 0 ]; then
  entrega_pct="$(awk -v e="$emitidos" -v d="$dispatches_fs" 'BEGIN{printf "%.2f", e*100/d}')"
else
  entrega_pct="INCOMPUTABLE"
fi

if [ "$u1_recibieron" -gt 0 ]; then
  u1_pct=$(( u1_citaron*100/u1_recibieron ))
else
  u1_pct=0
fi

# denom_u2: el pre-registro lo define como emitidos+fallidos+skipped+abstained.
# Sumar los cuatro contadores ya dedupeados volveria a contar dos veces a un
# agente que aparece en dos eventos distintos (p.ej. inject-failed en el primer
# dispatch e inject-emitted al continuarlo), que es el mismo error de unidad a
# menor escala. Se cuenta el conjunto UNION de agentes con cualquier evento de
# inyeccion: coincide exactamente con la suma cuando no hay solapamiento (el
# caso normal), y no infla el denominador cuando lo hay.
denom_u2="$(jq -s '
  [.[] | select(.reflex=="inject-emitted" or .reflex=="inject-failed"
                or .reflex=="inject-skipped-depth" or .reflex=="inject-abstained")] as $l
  | ([$l[] | select((.agent_id // "")!="") | .agent_id] | unique | length)
    + ([$l[] | select((.agent_id // "")=="")] | length)
' "$FILTERED")"
if [ "$denom_u2" -gt 0 ]; then
  u2="$(awk -v v="$violaciones_subagente" -v den="$denom_u2" 'BEGIN{printf "%.4f", v/den}')"
else
  u2="INCOMPUTABLE"
fi

# --- veredictos (umbrales del gate doc, hardcodeados adrede) ---
if [ "$dispatches_fs" -eq 0 ]; then
  veredicto_entrega="INSUFICIENTE-N"
elif awk -v x="$entrega_pct" 'BEGIN{exit !(x>=95)}'; then
  veredicto_entrega="PASS"
else
  veredicto_entrega="FAIL"
fi

if [ "$u1_recibieron" -lt 20 ]; then
  veredicto_u1="INSUFICIENTE-N"
elif [ "$u1_pct" -ge 60 ]; then
  veredicto_u1="PASS"
else
  veredicto_u1="FAIL"
fi

if [ "$emitidos" -lt 30 ] || [ "$sesiones" -lt 5 ]; then
  veredicto_u2="INSUFICIENTE-N"
elif awk -v x="$u2" -v u="$U2_UMBRAL" 'BEGIN{exit !(x<=u)}'; then
  veredicto_u2="PASS"
else
  veredicto_u2="FAIL"
fi

echo "emitidos=$emitidos"
echo "fallidos=$fallidos"
echo "skipped_depth=$skipped_depth"
echo "abstained=$abstained"
echo "dispatches_fs=$dispatches_fs"
echo "entrega_pct=$entrega_pct"
echo "violaciones_subagente=$violaciones_subagente"
echo "u2=$u2"
echo "u2_umbral=$U2_UMBRAL"
echo "u1_recibieron=$u1_recibieron"
echo "u1_citaron=$u1_citaron"
echo "u1_pct=$u1_pct"
echo "u1_jq_fallidos=$u1_jq_fallidos"
echo "u1_sin_transcript=$u1_sin_transcript"
echo "sesiones=$sesiones"
echo "sesiones_con_compact=$sesiones_con_compact"
echo "tipos=$tipos"
echo "veredicto_entrega=$veredicto_entrega"
echo "veredicto_u1=$veredicto_u1"
echo "veredicto_u2=$veredicto_u2"
exit 0
