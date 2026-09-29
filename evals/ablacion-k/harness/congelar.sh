#!/usr/bin/env bash
# congelar.sh — paso 2 de §12 de la campaña K. Deja todo fijo antes de la primera corrida:
#   1. snapshot de la KB en el HEAD actual de wisdom-paul (preparar.sh)
#   2. fuentes congeladas de las tareas activas de S1 con repo real (congelar_fuentes.sh)
#   3. gold activo = activas-s1.txt + las 20 de S2; sha256 del tar determinista
#   4. orden de la etapa 1: (tarea, brazo ∈ {a0,a3}, réplica ∈ {1,2}) barajado con la semilla
# Escribe $K_ROOT/congelacion.txt y lo imprime.
set -euo pipefail
H="$(cd "$(dirname "$0")" && pwd)"
K_ROOT="${K_ROOT:-$HOME/.cache/exo-ablacion-k}"; G="$K_ROOT/gold"
KB=/home/paul/Documentos/proyectos/wisdom-paul

kb_commit=$(git -C "$KB" rev-parse HEAD)
"$H/preparar.sh" "$kb_commit" > "$K_ROOT/preparar.log" 2>&1

# Fuentes de S1: solo las activas con repo real.
tmp=$(mktemp -d); while read -r id; do ln -s "$G/s1/$id" "$tmp/$id"; done < "$G/activas-s1.txt"
"$H/congelar_fuentes.sh" "$tmp" > "$K_ROOT/fuentes-s1.log"; rm -rf "$tmp"

# Gold activo y su hash (tar ordenado, sin mtimes ni dueños).
{ sed 's|^|s1/|' "$G/activas-s1.txt"; ls "$G/s2" | sed 's|^|s2/|'; } > "$G/activas.txt"
tar -C "$G" --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -cf "$K_ROOT/gold-activo.tar" -T "$G/activas.txt"
gold_sha=$(sha256sum "$K_ROOT/gold-activo.tar" | cut -d' ' -f1)

# Orden de la etapa 1.
python3 - "$G/activas.txt" "$K_ROOT/orden-etapa1.txt" <<'PY'
import random, sys
ids = [l.strip().split("/", 1)[1] for l in open(sys.argv[1]) if l.strip()]
corr = [(t, b, r) for t in ids for b in ("a0", "a3") for r in (1, 2)]
random.Random(20260923).shuffle(corr)
open(sys.argv[2], "w").write("".join(f"{t}\t{b}\t{r}\n" for t, b, r in corr))
PY

{ echo "fecha $(date -Iseconds)"
  echo "kb_commit $kb_commit"
  sed 's/^/prep: /' "$K_ROOT/prep/manifiesto.txt"
  echo "gold_activo_sha256 $gold_sha"
  echo "tareas_s1 $(wc -l < "$G/activas-s1.txt") · tareas_s2 $(ls "$G/s2" | wc -l)"
  echo "orden_etapa1_sha256 $(sha256sum < "$K_ROOT/orden-etapa1.txt" | cut -c1-16) · corridas $(wc -l < "$K_ROOT/orden-etapa1.txt")"
  echo "modelo claude-sonnet-5-5 · claude $(claude --version)"
  echo "fuentes_s1:"; sed 's/^/  /' "$K_ROOT/fuentes-s1.log"
} > "$K_ROOT/congelacion.txt"
cat "$K_ROOT/congelacion.txt"
