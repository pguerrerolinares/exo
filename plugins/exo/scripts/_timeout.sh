#!/usr/bin/env bash
# Helper COMPARTIDO: `timeout` portable a macOS.
#
# macOS no trae `timeout` (es GNU coreutils). Llamarlo a pelo sale con 127
# "command not found", y en recall-inject.sh eso degradaba el hook en CADA
# prompt sin inyectar nada (CI macos-latest, run 34720014952).
#
# Uso:
#   . "$SCRIPT_DIR/_timeout.sh"
#   con_timeout SEGUNDOS comando args...
#
# Contrato: el del `timeout` GNU que sustituye — exit 124 si corta; si no, el
# exit del comando (128+señal si murió por señal; 127 si no existe).
#
# Sin `timeout`, cae a perl (de serie en macOS), replicando lo que hace GNU:
# el comando va en SU PROPIO grupo de procesos y al vencer se mata el GRUPO.
# Un `alarm`+`exec` a secas solo mata al proceso directo: si es un script que
# lanza hijos (un `sleep`), el nieto huérfano retiene el pipe de `$(...)` y la
# sustitución espera a que acabe — medido: 30 s con un corte de 5.

con_timeout() {
  local segundos="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$segundos" "$@"
    return
  fi
  perl -e '
    my $t = shift;
    my $pid;
    $SIG{ALRM} = sub { kill "TERM", -$pid if $pid; exit 124 };
    alarm $t;
    $pid = fork;
    exit 125 unless defined $pid;
    if ($pid == 0) { setpgrp(0, 0); exec { $ARGV[0] } @ARGV; exit 127 }
    waitpid $pid, 0;
    exit(($? & 127) ? 128 + ($? & 127) : $? >> 8);
  ' "$segundos" "$@"
}
