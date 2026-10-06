# Pre-registro: techo-reglas-2 (regla de proyecto con autoridad de system prompt)

Sellado por el commit que crea este fichero, previo a cualquier corrida real. Las erratas posteriores van a `evals/techo-reglas-2/erratas.md`; nada de esto se edita a posteriori. Spec: `docs/superpowers/specs/2026-10-06-techo-reglas-2-design.md`.

## Diseno fijo

- **Canal:** la regla va en el system prompt: `--append-system-prompt-file $O/sysprompt.md`, donde `sysprompt.md` = `claude-md.md` + una linea en blanco + el framing con la regla. Sin hook y sin `--plugin-dir`. El `claude-md.md` de prep no cambia respecto a K y v1.
- **Framing:** `evals/techo-reglas-2/framing.txt` (sha256 `1a66c693bebe6d17c366c01403b500f691e2207beeec8623a63f79651bc14c3e`), con `{{REGLA}}` sustituido byte a byte por `evals/techo-reglas/reglas/<id>.txt`.
- **Brazos y k:** `arp` (regla con framing en el system prompt) y `a0` (fresco, solo `claude-md.md`), ambos con las restricciones de `a0` del v1 (`exo` = stub, KB denegada, `--setting-sources ""`, `autoMemoryEnabled:false`). `arp` k=2 y `a0` k=2 en las 10 tareas del suelo (40 corridas); `arp` k=2 en las 6 de control (12). Total 52.
- **Cumple:** una tarea cumple, para un brazo, si `check.rc == 0` en >=1 de sus 2 replicas.
- **Caida:** un control con `check.rc != 0` en sus 2 replicas (2/2). Una replica con `rc == 0` basta para no caer. Solo se cuenta sobre `arp`.
- **Gate:** PASA <=> `arp` >= 6/10 cumplen **y** margen >= 3 (`arp - a0` >= 3 tareas) **y** 0/6 caidas. Si no, NO PASA. El umbral no se renegocia.
- **No reconstruible:** una tarea sin entorno reconstruible, o sin `check.rc`, cuenta como no cumple (suelo) o como caida (control).
- **Modelo:** `claude-sonnet-5-5`, `--max-turns 40`, `--max-budget-usd 10` por corrida; el mismo que en K y v1.
- **Version de claude:** `claude --version` = `2.1.291 (Claude Code)` al sellar (`claude-version.txt`); las 52 corridas deben dar la misma. Si difiere antes de correr, se para y se escribe una errata.
- **Re-intentos:** cero. Cada corrida se ejecuta exactamente una vez; un crash, error de API, budget o turns agotados cuenta como no cumple/caida.
- **Breaker:** si salta uno (fuga, mas del 10 % de corridas sin `result`, gasto > 15 USD), la tanda es invalida y no se adjudica. Se registra en `erratas.md`. Una tanda nueva desde cero no cuenta como tercera bala.
- **Clases cerradas:** cada no-cumple de `arp` en el suelo recibe una de 5 clases: conflicto regla-tarea, check roto, regla mal escrita, no reconstruible, incumplimiento del agente. La asigna un adjudicador fresco que no diseno el framing.
- **Cierre:** si no pasa, el frente se cierra sin tercera bala.
- **Gold:** el gold es el del tarball `gold-activo.tar` salvo las 4 erratas versionadas en `erratas-gold.md` (g2-154, g0-122, g0-33, g2-35) y los helpers de `gold/harness/`. Todo pinneado en `pins.sha256` (rutas relativas a `$K_ROOT`), que `correr-techo.sh` verifica antes de lanzar.
- **Suelo (10):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97`. **Control (6):** `g0-159 g1-144 g1-16 g1-25 g1-34 g2-35`. Fuera g2-171, g1-147 y g1-57.
- **Reglas:** las del v1, `evals/techo-reglas/reglas/<id>.txt`, sin cambios.

## Orden de corridas (barajado, semilla 20261006)

Se parte de la lista ordenada (`sort`) de las 52 tuplas `(brazo, id, rep)` (`rep` entero) derivadas de la columna `brazos` de `tareas.tsv` y se baraja con `random.Random(20261006).shuffle`. Son la cola de `-P 4`. Comando, desde la raiz del repo:

```bash
python3 -c 'import random,csv; L=[]
for i,g,_,b in csv.reader(open("evals/techo-reglas-2/tareas.tsv"),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261006).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))'
```

<!-- ORDEN-BEGIN -->
a0 g1-140 1
arp g0-149 2
arp g2-170 1
a0 g2-97 2
arp g1-139 1
a0 g1-139 1
a0 g1-139 2
arp g1-16 2
arp g2-97 2
arp g2-154 2
arp g0-122 1
arp g0-159 2
a0 g2-170 2
arp g0-122 2
arp g1-131 1
a0 g0-149 2
arp g0-33 1
a0 g0-122 2
arp g2-35 2
arp g0-159 1
arp g0-149 1
arp g0-33 2
a0 g0-33 1
arp g2-154 1
arp g1-25 2
arp g1-16 1
arp g1-157 1
arp g1-157 2
a0 g1-140 2
a0 g1-157 2
arp g2-170 2
arp g1-131 2
a0 g2-170 1
arp g2-97 1
a0 g1-131 2
a0 g2-154 2
a0 g0-122 1
a0 g1-131 1
a0 g0-149 1
arp g1-144 1
arp g1-34 1
arp g1-25 1
arp g1-140 2
a0 g2-154 1
arp g1-144 2
a0 g1-157 1
arp g1-140 1
arp g2-35 1
a0 g2-97 1
arp g1-34 2
a0 g0-33 2
arp g1-139 2
<!-- ORDEN-END -->

## Limitaciones declaradas

Solo se declaran; no cambian umbrales ni criterio.

- **Potencia:** el diseno es seguro contra el azar pero esta infrapotenciado para efectos moderados: con 10 tareas y margen >= 3, un efecto real pequeno o medio puede no pasar el gate. Un NO PASA no prueba ausencia de efecto.
- **Sin placebo:** no hay brazo placebo (mismo system prompt con una regla irrelevante), asi que no se separa "autoridad del framing" de "regla en el system prompt". El gate mide el paquete, no el mecanismo.
- **Sucio en K:** las fuentes se reconstruyen en su commit limpio; el trabajo sin commitear de K no se reconstruye (g0-122=35, g2-170=2).
- **Gold:** los otros 11 checks con ruta `campana-k` quedan fuera del experimento (deuda en `erratas-gold.md`).
