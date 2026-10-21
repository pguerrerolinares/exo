#!/usr/bin/env bash
# Helper de bash-guards.sh (rama 2, zero-residuo): decide si un comando
# contiene un `git [-C DIR] add (-A|--all|.)` SIN pathspec EN POSICION DE
# COMANDO. El PATRON_ZR de la rama sigue siendo el que LOGUEA (mide, incluido
# el ruido); este helper es el matcher estricto que habilita el deny (#133).
#
# Uso: zr_deny_dirs "$CMD"  -> una linea por cada `add` sin pathspec: el
# directorio efectivo (el de `-C`, o el del ultimo `cd` literal; vacio si
# ninguno). Sin salida = nada que denegar.
#
# Heuristica, no parser de shell. Errores aceptados, todos hacia NO denegar:
#   - las lineas de cuerpo de un heredoc se descartan (prosa que cita el
#     comando: el FP medido);
#   - posicion de comando = inicio de segmento tras `&& || ; | &` o salto de
#     linea, saltando `(`/`{` y asignaciones `VAR=val`; un `git add -A` dentro
#     de comillas, tras `echo`, `grep` o `#` no cuenta;
#   - cualquier token que no sea flag ni redireccion tras el `-A`/`.` es
#     pathspec (`add -A src`, `add -A -- . ':!x'`, `add . src`).
# Un `;` o `|` dentro de comillas parte el segmento en falso: puede dar un
# deny de mas solo si lo que sigue parece `git add -A` literal.

zr_deny_dirs() {
  printf '%s\n' "$1" | awk '
    BEGIN { q = "\047"; term = ""; cur = "" }
    {
      line = $0
      if (term != "") {
        t = line; sub(/^[ \t]+/, "", t); sub(/[ \t]+$/, "", t)
        if (t == term) term = ""
        next
      }
      if (match(line, /<<-?[ \t]*[\047"]?[A-Za-z_][A-Za-z0-9_]*[\047"]?/) \
          && !(RSTART > 1 && substr(line, RSTART - 1, 1) == "<")) {
        w = substr(line, RSTART, RLENGTH)
        sub(/^<<-?[ \t]*/, "", w); gsub(/[\047"]/, "", w)
        term = w
      }
      gsub(/&&|\|\||[;|&]/, "\n", line)
      n = split(line, segs, "\n")
      for (k = 1; k <= n; k++) segmento(segs[k])
    }
    function expande(x,   n) {
      gsub(/["\047]/, "", x)
      if (match(x, /^\$\{?[A-Za-z_][A-Za-z0-9_]*\}?/)) {
        n = substr(x, RSTART, RLENGTH); gsub(/[${}]/, "", n)
        if (n in vars) x = vars[n] substr(x, RLENGTH + 1)
      }
      return x
    }
    function segmento(s,   t, nt, i, d, a, p, skip, tok, v, nm) {
      nt = split(s, t, /[ \t]+/)
      i = 1
      while (i <= nt && (t[i] == "" || t[i] ~ /^[({]+$/ || t[i] ~ /^[A-Za-z_][A-Za-z0-9_]*=/)) {
        if (t[i] ~ /^[A-Za-z_][A-Za-z0-9_]*=/) {
          v = t[i]; sub(/^[^=]*=/, "", v); gsub(/["\047]/, "", v)
          nm = t[i]; sub(/=.*/, "", nm); vars[nm] = v
        }
        i++
      }
      if (t[i] == "cd") {
        d = expande(t[i + 1])
        cur = (d ~ /^\// || cur == "") ? d : cur "/" d
        return
      }
      if (t[i] != "git") return
      i++; d = ""
      if (t[i] == "-C") { d = expande(t[i + 1]); i += 2 }
      if (t[i] != "add") return
      a = t[i + 1]
      if (a != "-A" && a != "--all" && a != ".") return
      skip = 0
      for (p = i + 2; p <= nt; p++) {
        tok = t[p]
        if (tok == "" || tok ~ /^#/) break
        if (skip) { skip = 0; continue }
        if (tok ~ /^[0-9]*[<>]+$/) { skip = 1; continue }
        if (tok ~ /^[0-9]*[<>]/ || tok ~ /^[})]+$/) continue
        if (tok ~ /^-/ && tok != "--") continue
        return
      }
      print (d != "" ? d : cur)
    }'
}
