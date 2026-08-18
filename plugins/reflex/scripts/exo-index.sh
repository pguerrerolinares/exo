#!/usr/bin/env bash
# Stop hook: refresca el índice de exo al cerrar la sesión (M6-01).
#
# Por qué AQUÍ y no en el arranque. basic-memory mantenía su índice al día con
# un watch en segundo plano; exo indexa al invocar, sin daemon. Medido en esta
# máquina: reindexar sin cambios cuesta ~25 ms, pero con una nota editada
# cuesta ~1,5 s (cargar el runtime ONNX + embeber los trozos que cambiaron), y
# tras un clone fresco —todos los mtimes nuevos— más de diez minutos. Poner eso
# en SessionStart convierte un arranque de 14 ms en una espera, y en el peor
# caso en un cuelgue.
#
# Al cierre no molesta a nadie, y es justo cuando la KB acaba de cambiar
# (/documenta escribe al terminar), así que el índice llega fresco al arranque
# siguiente. Medido sobre el repo de la KB: el 95% de sus commits de los
# últimos 60 días se hacen dentro de una sesión de agente, así que este
# disparador cubre casi todo; lo que se edite fuera queda obsoleto como mucho
# una sesión, y `exo recall --refresca` está para forzarlo a mano.
#
# Contrato: nunca bloquea el cierre. Se lanza detached y sale de inmediato.
set -uo pipefail

EXO_BIN="${EXO_BIN:-$(command -v exo 2>/dev/null || echo "$HOME/.local/bin/exo")}"
EXO_INDEX="${EXO_INDEX:-$HOME/.exo/index.db}"
LOG="${EXO_INDEX_LOG:-$HOME/.claude/exo-index.log}"

[ -x "$EXO_BIN" ] || exit 0

# setsid -f: el harness mata el process group del hook al terminar, y un
# nohup+& moriría con él (visto en vivo con el refresco de cache del recall
# viejo, 2026-07-10). Con sesión propia, el indexado sobrevive al cierre.
setsid -f nohup "$EXO_BIN" index --db "$EXO_INDEX" --json >>"$LOG" 2>&1 </dev/null || true
exit 0
