# Pre-registro: test de techo de las reglas de proyecto (gate de la 2d)

Sellado por el commit que crea este fichero, previo a cualquier corrida de T4. Las erratas posteriores van a `evals/techo-reglas/erratas.md`; nada de esto se edita a posteriori.

## Diseno fijo

- **Canal:** hook SessionStart que emite `{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"<regla literal>"}}`. Nunca `--append-system-prompt-file` para la regla. El `claude-md.md` de prep no cambia respecto a K.
- **Brazo `ar`:** `a0` mas ese hook. `exo` = stub, lecturas del snapshot y de la KB de produccion denegadas, `--setting-sources ""`, `autoMemoryEnabled:false`, `--max-budget-usd 10`, `--max-turns 40`.
- **Replicas (k):** k=2 en las 11 tareas del suelo (22 corridas); k=1 en las 6 de control (6 corridas). Total 28.
- **Cumple:** una tarea del suelo cumple si `check.rc == 0` en >=1 de sus 2 replicas (`rc=2`, no evaluable, no cuenta).
- **Caida:** una tarea de control con `check.rc != 0`.
- **Gate:** PASA <=> >=6/11 cumplen **y** 0/6 caidas. Si no, NO PASA. El umbral no se renegocia.
- **No reconstruible:** una tarea cuyo entorno no se puede reconstruir cuenta como no cumple (suelo) o como caida (control).
- **Modelo:** `claude-sonnet-5-5`, el mismo que en K.
- **Version de `claude`:** `claude --version` al momento del commit = `2.1.286 (Claude Code)` (2026-09-30). K uso `2.1.285`. Diferencia de una version menor, declarada y no corregible.
- **Suelo (11):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-171 g2-97`. Fuera g1-147 y g1-57.
- **Reglas:** campo `regla` de la fila con `id` igual en `pool/reglas-g{0,1,2}.jsonl` del tarball `~/.cache/exo-ablacion-k-registro.tar.gz` (sha256 `116964a9ed565e1c6ac6a887c20de956eae5f2aec48cb317bf4092b2db57abd9`), copiado byte a byte con `jq -j` (sin salto de linea anadido) a `reglas/<id>.txt`. Cada id aparece exactamente una vez en el pool y `gold/s1/<id>/tarea.json .regla_id == id`.

## Seleccion de control (mecanica)

Tareas S1 (las 40 de `gold/s1`) con `check.rc == 0` en `corridas/<id>/a0-r1` y `a0-r2` del tarball, sin las 11 del suelo ni g1-147 ni g1-57, ordenadas por `sha256(id)` (`printf %s "$id" | sha256sum`, hex en minusculas, orden lexicografico), tomando las 6 primeras con `reconstruible=si` en `reconstruccion.tsv`. `REG` es el tarball extraido.

```bash
K=~/.cache/exo-ablacion-k
REG=<tarball extraido>
for id in $(ls "$K/gold/s1"); do
  [ "$(cat "$REG/corridas/$id/a0-r1/check.rc" 2>/dev/null)" = 0 ] && [ "$(cat "$REG/corridas/$id/a0-r2/check.rc" 2>/dev/null)" = 0 ] && echo "$id"
done | grep -vxE 'g0-122|g0-149|g0-33|g1-131|g1-139|g1-140|g1-157|g2-154|g2-170|g2-171|g2-97|g1-147|g1-57' |
while read -r id; do printf '%s %s\n' "$(printf %s "$id" | sha256sum | cut -d' ' -f1)" "$id"; done | sort |
while read -r h id; do
  [ "$(awk -F'\t' -v i="$id" '$1==i{print $2}' "$K/reconstruccion.tsv")" = si ] && echo "$h $id"
done | head -6
```

Resultado (22 candidatas con a0 2/2; las 6 primeras reconstruibles; las 40 lo son hoy):

```
121554cd621ac31fe63b5f9b2e692ac88d432a1b20fb2b360d1dc5843c7aa57b g1-25
16880ebf917c9bb3d30347cd110c13bb062f17fb65ad69ac59a91922cce6fc29 g1-144
2262649ca41803b091b1764610149423f93feb5654bb6218563e2d8394109d41 g1-16
3121c45b8756873c7ded7c6bff613ffc28bc03da33e94b468e3e079f11b6dea0 g2-35
39c1614eedee41bb4ee6682645cbeed4e6bcd4d0baedca35955141b8e9d6e9bf g0-159
3c262adc34564bb46ae3f75cdc379a3afe18e3ccdd77762e8e41b047c2202cd6 g1-34
```

## Orden de corridas (barajado, semilla 20260930)

Se parte de la lista ordenada (`sort`) de los 28 pares `id rep` (suelo: rep 1 y 2; control: rep 1) y se baraja con `random.Random(20260930).shuffle`. Comando, desde la raiz del repo:

```bash
python3 -c 'import random,csv; L=[]
for i,g,_ in csv.reader(open("evals/techo-reglas/tareas.tsv"),delimiter="\t"):
    L+=[(i,r) for r in ((1,2) if g=="suelo" else (1,))]
L.sort(); random.Random(20260930).shuffle(L)
print("\n".join(f"{i} {r}" for i,r in L))'
```

Python 3.12.3. Lectura por T4: las lineas del bloque entre `ORDEN-BEGIN` y `ORDEN-END` que casan con `^[a-z0-9-]+ [12]$` son, en orden, las corridas (`id rep`).

<!-- ORDEN-BEGIN -->
g0-149 2
g0-159 1
g0-33 1
g0-149 1
g2-154 1
g1-157 2
g1-139 2
g2-97 2
g1-140 1
g1-25 1
g0-33 2
g2-170 2
g2-97 1
g1-34 1
g2-171 2
g1-131 2
g0-122 2
g0-122 1
g1-144 1
g1-16 1
g2-35 1
g1-139 1
g2-170 1
g1-131 1
g2-171 1
g2-154 2
g1-140 2
g1-157 1
<!-- ORDEN-END -->

## Limitaciones declaradas

Solo se declaran; no cambian umbrales ni criterio.

1. Las fuentes se reconstruyen en su commit limpio. El trabajo sin commitear de K no se puede reconstruir (`sucio_en_K` en el suelo: g0-122=35, g2-170=2, g2-171=2). Una corrida aqui puede diferir de la de K por ese motivo.
2. Las corridas de K de `a0` son el baseline sin regla; el efecto de la regla se lee contra ellas, no contra una repeticion de `a0` hoy.
3. Hoy `claude` es 2.1.286 frente a 2.1.285 de K, y `~/.claude/CLAUDE.md` pudo cambiar (la fila `_prep` de `reconstruccion.tsv` dice `coincide` hoy).
