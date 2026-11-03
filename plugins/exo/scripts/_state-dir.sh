#!/usr/bin/env bash
# Rutas de estado de los reflejos. Lo cargan TODOS los reflejos: nada de
# sustituciones de comando ni subshells, solo expansiones de parámetros.
# Precedencia (igual que `exo config`): variable propia > $EXO_STATE_DIR/<x> >
# ~/.exo/state/<x>. Vacía cuenta como ausente (`:-`).

export EXO_STATE_DIR="${EXO_STATE_DIR:-$HOME/.exo/state}"
export REFLEX_LOG_FILE="${REFLEX_LOG_FILE:-$EXO_STATE_DIR/reflex-log.jsonl}"
# Sin override propio: Rust y register.ts la derivan siempre de EXO_STATE_DIR.
export EXO_RULES_DIR="$EXO_STATE_DIR/exo-rules"
export EXO_INDEX_LOG="${EXO_INDEX_LOG:-$EXO_STATE_DIR/exo-index.log}"
export REFLEX_CANARY_FILE="${REFLEX_CANARY_FILE:-$EXO_STATE_DIR/reflex-inject-canary}"
