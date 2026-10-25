# Pre-registro: techo-reglas-3 (dilución de reglas en el system prompt)

Sellado por el commit que crea este fichero, antes de cualquier corrida real. Las erratas posteriores van a `evals/techo-reglas-3/erratas.md` y nada de esto se edita a posteriori. Spec: `docs/superpowers/specs/2026-10-25-reglas-literales-arranque-design.md` §7.1, §7.3, D6 y D7. Fija el `CAP` de `exo rules` antes de sembrar.

## Diseno fijo

- **Canal:** el de techo-reglas-2: `--append-system-prompt-file $O/sysprompt.md`, con `sysprompt.md` = `claude-md.md` + `\n` + el framing con el bloque. Sin hook y sin `--plugin-dir`.
- **Framing:** `evals/techo-reglas-3/framing.txt` (sha256 `f0e416d993c4455bc3825f1a60b289985963c8b1cf06d750546d169734b787e5`) = `{{REGLA}}` a secas. El bloque entero (encabezados, autoridad y lista) viene en `K_REGLA_FILE`, renderizado byte a byte como `bloque()` de `plugins/exo/hooks/register.ts` (globales «del dueño» primero, luego «del proyecto») más `\n` final.
- **Bloques:** `bloques/<id>/arp{1,10,20}.txt`, generados por `generar-bloques.py` (semilla 20261025) desde las 541 reglas de la campaña M (`pool.sha256`), con el filtro automático del script y los descartes a mano de `descartes.tsv`. Sus sha256 están en `bloques.sha256`. No se versionan porque son reglas reales de la KB; `seleccion.tsv` versiona qué id va en cada lista y posición.
- **Composición:** la gold va en la lista de su ámbito real (`transversal` en «del dueño», el resto en «del proyecto»). Las distractoras globales son transversales y las de proyecto vienen de OTROS proyectos. `arp1` lleva solo la gold, `arp10` lleva 5 + 5 y `arp20` lleva 10 + 10, con la gold incluida. `arp10` es prefijo de `arp20`. La posición de la gold se fija por semilla, una por tarea.
- **Brazos y k:** `arp1`, `arp10` y `arp20`, k=2, en las 10 tareas del suelo de techo-reglas-2 (60 corridas). Todos con las restricciones de `a0` (`exo` = stub, KB denegada, `--setting-sources ""`, `autoMemoryEnabled:false`). No hay `a0` (en techo-reglas-2 dio 0/10) ni controles, porque el `CAP` no mide caídas.
- **Cumple:** una tarea cumple, para un brazo, si `check.rc == 0` en >=1 de sus 2 replicas.
- **Replica:** el `CAP` solo se adjudica si `arp1` >= 5/10. Si no, el canal no replica en esta versión: no se adjudica, el `CAP` se queda en 10 (el desplegado) y se reporta. Esta cláusula se añade a la spec, que no la tenía.
- **CAP:** lo adjudica `arp20`: >= 5/10 da `CAP` 10; 4/10 da 7; <= 3/10 da 5. El umbral no se renegocia.
- **Invalidacion de A:** `arp10` <= 3/10 cumple la condición «canal + dilución» de §7.3 y basta por sí sola. Se para y se reporta; no se revierte nada automáticamente (D7).
- **No reconstruible:** una tarea sin entorno reconstruible, o sin `check.rc`, cuenta como no cumple.
- **Modelo:** `claude-sonnet-5-5`, `--max-turns 40`, `--max-budget-usd 10` por corrida: lo mismo que en K, v1 y techo-reglas-2.
- **Version de claude:** al sellar, `claude --version` = `2.1.310 (Claude Code)` (`claude-version.txt`), y las 60 corridas deben dar esa misma. Si difiere antes de correr, se para y se escribe una errata. techo-reglas-2 corrió en `2.1.291`, y por eso `arp1` se vuelve a medir aquí en vez de reutilizar su 6/10.
- **Re-intentos:** cero. Cada corrida se ejecuta exactamente una vez. Un crash, un error de API o quedarse sin budget o sin turns cuenta como no cumple.
- **Breaker:** si salta uno (fuga, más del 10 % de corridas sin `result`, gasto > 15 USD), la tanda es invalida y no se adjudica. Se registra en `erratas.md`; una tanda nueva desde cero exige errata previa.
- **Gold y prep:** el gold y `prep/claude-md.md` son los de techo-reglas-2 (`pins.sha256`, copiado sin cambios), y `correr-techo.sh` los verifica antes de lanzar. El `prep/` de K se perdió el 10-24 y `restaurar-prep.sh` lo rehízo: `claude-md.md` sale byte a byte del `sysprompt.md` de techo-reglas-2 con el pin verificado, y `kb/` es la KB en `80cba57` (el `kb_commit` de K). El `prep/` pisado se conserva en `prep.pisado-*`.
- **Antes de lanzar:** `validar-preregistro.sh` en verde y un ensayo (`K_ENSAYO=1`) de las 60 sin `claude`. Comando: `TECHO_EXP=evals/techo-reglas-3 bash evals/techo-reglas/correr-techo.sh 4`; adjudicación: `python3 evals/techo-reglas-3/evaluar3.py ~/.cache/exo-ablacion-k evals/techo-reglas-3/tareas.tsv`.
- **Suelo (10):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97`.

## Orden de corridas (barajado, semilla 20261025)

Se parte de la lista ordenada (`sort`) de las 60 tuplas `(brazo, id, rep)` (`rep` entero), derivadas de la columna `brazos` de `tareas.tsv`, y se baraja con `random.Random(20261025).shuffle`. Esa es la cola de `-P 4`. Comando, desde la raíz del repo:

```bash
python3 -c 'import random,csv; L=[]
for i,g,_,b in csv.reader(open("evals/techo-reglas-3/tareas.tsv"),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261025).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))'
```

<!-- ORDEN-BEGIN -->
arp10 g0-122 1
arp10 g0-33 2
arp20 g2-97 2
arp1 g2-97 1
arp1 g1-140 2
arp20 g1-131 2
arp1 g2-97 2
arp10 g1-157 2
arp10 g2-97 2
arp20 g1-157 1
arp20 g0-122 2
arp1 g2-170 1
arp10 g1-139 2
arp20 g2-170 2
arp10 g1-140 2
arp20 g1-139 1
arp10 g2-97 1
arp10 g2-154 2
arp20 g2-170 1
arp1 g2-170 2
arp20 g2-154 2
arp1 g1-131 2
arp1 g1-140 1
arp20 g2-154 1
arp1 g0-122 1
arp1 g0-122 2
arp20 g0-149 1
arp10 g1-157 1
arp1 g2-154 2
arp1 g1-157 2
arp20 g0-33 2
arp10 g2-154 1
arp1 g1-131 1
arp20 g1-157 2
arp1 g0-149 1
arp10 g1-140 1
arp1 g0-33 2
arp10 g2-170 2
arp10 g0-149 1
arp20 g1-140 1
arp20 g0-122 1
arp20 g2-97 1
arp20 g1-140 2
arp10 g0-122 2
arp1 g0-149 2
arp1 g1-139 1
arp20 g0-33 1
arp1 g1-139 2
arp10 g0-149 2
arp10 g1-131 1
arp1 g0-33 1
arp10 g1-139 1
arp10 g2-170 1
arp20 g1-131 1
arp20 g0-149 2
arp1 g1-157 1
arp1 g2-154 1
arp20 g1-139 2
arp10 g0-33 1
arp10 g1-131 2
<!-- ORDEN-END -->

## Limitaciones declaradas

Solo se declaran; no cambian umbrales ni criterio.

- **Potencia:** 10 tareas con k=2 y criterio binario. Entre `CAP` 10 y 7 hay una sola tarea de diferencia, y el ruido entre tandas es de ±1-2 tareas (el 6/10 de techo-reglas-2 fue un PASA justo). El `CAP` es una constante reversible, no una conclusión sobre el mecanismo.
- **Distractoras de otros proyectos:** en producción, la lista «del proyecto» trae reglas del mismo proyecto, que pueden tocar la tarea. Aquí vienen de otros proyectos y, además, se descartan las que tocan la tarea. Las dos cosas hacen la dilución medida optimista: es una cota del caso fácil.
- **Descartes a mano:** los hizo el mismo agente que diseñó el experimento, antes de cualquier corrida y sin resultados. Cada uno lleva su motivo en `descartes.tsv`. El filtro automático quita las que nombran exo o la KB, porque llevarían al agente a llamar a `exo` y el detector de fugas pararía la tanda.
- **Sin placebo de longitud:** `arp20` frente a `arp1` mezcla «más texto» con «más reglas». No se separa longitud de número de reglas.
- **Posición:** una sola por tarea, uniforme por semilla. No se mide el efecto de posición (lost-in-the-middle).
- **g0-33:** es transversal, así que en `arp1` va bajo «del dueño» y difiere del `arp` de techo-reglas-2 solo en el encabezado. Las otras 9 `arp1` son byte a byte el `sysprompt.md` de techo-reglas-2.
- **System prompt parcial:** en producción, el system prompt lleva además el resto del bloque de arranque (contrato ≤1,5 KB), que aquí no está.
