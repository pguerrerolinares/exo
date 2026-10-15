#!/usr/bin/env bash
# Helper COMPARTIDO: `timeout` con fallback para entornos sin el GNU.
#
# `timeout` es GNU coreutils. Llamarlo a pelo falla en dos casos:
#  - macOS/BSD no lo traen: exit 127 "command not found".
#  - Windows: Git Bash SÍ trae coreutils (/usr/bin/timeout), pero lanzado con
#    `Git\usr\bin\bash.exe` el PATH no incluye /usr/bin y `timeout` resuelve a
#    /c/WINDOWS/system32/timeout, el TIMEOUT.EXE de Windows. Con sintaxis GNU
#    imprime "ERROR: Sintaxis no válida" y sale con 1 SIN ejecutar el comando.
# En ambos los hooks se degradaban en CADA disparo sin inyectar nada.
#
# Uso:
#   . "$SCRIPT_DIR/_timeout.sh"
#   con_timeout SEGUNDOS comando args...
#
# Contrato: el del `timeout` GNU que sustituye — exit 124 si corta; si no, el
# exit del comando (128+señal si murió por señal; 127 si no existe).
#
# Sin un `timeout` GNU (ausente o TIMEOUT.EXE), cae a perl, replicando lo que hace GNU:
# el comando va en SU PROPIO grupo de procesos y al vencer se mata el GRUPO.
# Un `alarm`+`exec` a secas solo mata al proceso directo: si es un script que
# lanza hijos (un `sleep`), el nieto huérfano retiene el pipe de `$(...)` y la
# sustitución espera a que acabe — medido: 30 s con un corte de 5.
#
# Sin perl en el PATH reducido de Windows se prueba /usr/bin/perl (Git for
# Windows lo trae); si tampoco, exit 127 con aviso: sin timeout fiable no se
# ejecuta el comando, para no colgar un hook.

# Falla si `timeout` no existe o resuelve bajo System32 (TIMEOUT.EXE).
_timeout_gnu() {
  local t
  t="$(command -v timeout 2>/dev/null)" || return 1
  case "$t" in
    */[Ss][Yy][Ss][Tt][Ee][Mm]32/*) return 1 ;;
  esac
}

con_timeout() {
  local segundos="$1"
  shift
  if _timeout_gnu; then
    timeout "$segundos" "$@"
    return
  fi
  local perl=perl
  if ! command -v perl >/dev/null 2>&1; then
    if [ -x /usr/bin/perl ]; then
      perl=/usr/bin/perl
    else
      printf 'con_timeout: sin timeout GNU ni perl\n' >&2
      return 127
    fi
  fi
  # shellcheck disable=SC2016 # código perl literal: sus $ no son de bash
  "$perl" -e '
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
