#!/usr/bin/env bash
# congelar-m.sh <criba|principal> — congela la campaña M antes de gastar API.
# Escribe $K_ROOT/congelacion-<fase>.txt (tanda.sh lo lee con K_CONGELACION). El formato conserva la línea
# `modelo <m> · claude <v>` que lee analizar.py.
set -euo pipefail
fase=${1:-}
case "$fase" in criba|principal) ;; *) echo "uso: congelar-m.sh <criba|principal>" >&2; exit 2;; esac
K_ROOT="${K_ROOT:-$HOME/.cache/exo-campana-m}"; P="$K_ROOT/prep"; G="$K_ROOT/gold"
TARBALL="${K_TARBALL:-$HOME/.cache/exo-ablacion-k-registro.tar.gz}"
ESPERADO="${K_TARBALL_SHA:-116964a9ed565e1c6ac6a887c20de956eae5f2aec48cb317bf4092b2db57abd9}"

sha_tar=$(sha256sum "$TARBALL" | cut -d' ' -f1)
[ "$sha_tar" = "$ESPERADO" ] || { echo "congelar-m: tarball de K no coincide ($sha_tar != $ESPERADO)" >&2; exit 1; }

tar -C "$G" --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -cf "$K_ROOT/gold-activo.tar" -T "$G/activas.txt"
if [ "$fase" = criba ]; then ordenes=("$K_ROOT"/orden-criba-*.txt); else ordenes=("$K_ROOT/orden-principal.txt"); fi
orden_sha=$(cat "${ordenes[@]}" | sha256sum | cut -d' ' -f1)

{ echo "fecha $(date -Iseconds)"
  echo "fase $fase"
  echo "tarball_k_sha256 $sha_tar"
  echo "kb_commit $(awk '$1=="kb_commit"{print $2}' "$P/manifiesto.txt")"
  echo "exo $(exo --version)"
  echo "memoria_sha256 $(sha256sum < "$P/memoria/MEMORY.md" | cut -c1-16)"
  echo "gold_activo_sha256 $(sha256sum < "$K_ROOT/gold-activo.tar" | cut -d' ' -f1)"
  echo "orden_sha256 $orden_sha · ficheros $(printf '%s ' "${ordenes[@]##*/}")"
  echo "semilla 20261024"
  echo "paralelo 1"
  echo "modelo claude-sonnet-5-5 · claude $(claude --version)"
} > "$K_ROOT/congelacion-$fase.txt"
cat "$K_ROOT/congelacion-$fase.txt"
