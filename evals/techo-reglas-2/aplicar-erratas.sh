#!/usr/bin/env bash
# aplicar-erratas.sh [K_ROOT] — instala las erratas del gold (T2) en $K_ROOT. Única escritura permitida sobre el gold.
# exit 2 si el destino no es escribible.
set -eu
D="$(cd "$(dirname "$0")" && pwd)"
K="${1:-${K_ROOT:-$HOME/.cache/exo-ablacion-k}}"
for dir in "$K/gold" "$K/gold/s1"; do
  [ -d "$dir" ] && [ -w "$dir" ] || { echo "aplicar-erratas: $dir no existe o no es escribible" >&2; exit 2; }
done
mkdir -p "$K/gold/harness"
for id in g2-154 g0-122 g0-33 g2-35; do
  [ -w "$K/gold/s1/$id" ] || { echo "aplicar-erratas: $K/gold/s1/$id no es escribible" >&2; exit 2; }
  install -m 755 "$D/gold/$id/check.sh" "$K/gold/s1/$id/check.sh"
done
for f in "$D"/gold/harness/*.sh; do install -m 755 "$f" "$K/gold/harness/$(basename "$f")"; done
echo "erratas aplicadas en $K/gold"
