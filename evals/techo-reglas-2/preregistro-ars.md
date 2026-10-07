# Pre-registro: ars (regla de proyecto entregada por el canal degradado prompt.submit)

Sellado por el commit que crea este fichero, previo a cualquier corrida real de `ars`. Las erratas posteriores van a `evals/techo-reglas-2/erratas.md`; nada de esto se edita a posteriori. Consulta: `docs/superpowers/consultas/2026-10-07-reglas-fallback-sessionstart/consultor.md`. Plan: `docs/superpowers/plans/2026-10-07-reglas-fallback-submit.md` (Task 3).

Pregunta: bajo la politica Team/Enterprise `prompt.compose` queda bypassed y la regla solo puede llegar por `prompt.submit` (`context`). ¿Ese canal degradado, con el mismo texto de autoridad, sigue moviendo al agente? Se reutilizan `a0` (0/10) y `arp` (6/10) ya sellados en `preregistro.md`; no se vuelven a correr.

## Diseno fijo

- **Canal:** `a0` exacto (mismo `claude-md.md`, `exo` = stub, KB denegada, `--setting-sources ""`, `autoMemoryEnabled:false`) mas un plugin minimo en `$O/plugin-ars/` con solo el mod `register.ts` (copia de `plugins/exo/hooks/register.ts`, `hooks.json` solo con `"modules"`, sin `scripts/` ni hooks de comando), cargado con `--plugin-dir` y `EXO_RULES_FORZAR_SUBMIT=1`. El system prompt NO lleva framing: el texto llega solo por el mod, en `prompt.submit` como `context`. Un shim `exo` en `$O/stub-ars/` (antes que el stub en PATH) responde a `exo rules --json` con la regla de `evals/techo-reglas/reglas/<id>.txt` sin salto final.
- **Texto entregado:** el `FRAMING` del mod + `- <regla>`: el mismo texto que `framing.txt` con `{{REGLA}}` sustituido (el de `arp`), salvo el salto de linea final. Cualquier otra diferencia se anota en `verdict-ars.md`.
- **Brazos y k:** solo `ars`, k=2, en las 10 tareas del suelo de `tareas-ars.tsv` (20 corridas). Sin controles ni `a0`.
- **Cumple:** una tarea cumple si `check.rc == 0` en >=1 de sus 2 replicas.
- **Gate:** >=4/10 cumplen = util; <4 = placebo. El umbral no se renegocia. Util: el prefijo del testigo queda en `ℹ`; placebo: pasa a `⚠` (Task 4). El eval no gatea el merge.
- **No reconstruible:** una tarea sin entorno reconstruible, o sin `check.rc`, cuenta como no cumple.
- **Modelo:** `claude-sonnet-5-5`, `--max-turns 40`, `--max-budget-usd 10` por corrida; el mismo que `a0`/`arp`.
- **Version de claude:** `claude --version` = `2.1.291 (Claude Code)` en las corridas selladas de `a0`/`arp` (`claude-version.txt`); las 20 de `ars` deben dar la misma. Si difiere antes de correr, se escribe una errata antes de adjudicar y la comparacion con `a0`/`arp` se declara con esa salvedad.
- **Re-intentos:** cero. Cada corrida se ejecuta exactamente una vez; un crash, error de API, budget o turns agotados cuenta como no cumple.
- **Breaker:** los de `correr-techo.sh`: si salta uno (fuga, mas del 10 % de corridas sin `result`, gasto > 15 USD), la tanda es invalida y no se adjudica. Se registra en `erratas.md`.
- **Clases cerradas:** cada no-cumple de `ars` recibe una de 5 clases: conflicto regla-tarea, check roto, regla mal escrita, no reconstruible, incumplimiento del agente.
- **Gold y reglas:** los de `preregistro.md` (pins en `pins.sha256`), sin cambios.
- **Suelo (10):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97`.
- **Sonda previa (obligatoria):** antes de la tanda se corre 1 sola corrida de `ars` fuera del ORDEN (rep 9, ver comando en el informe de la tarea; no cuenta para el gate ni usa una rep del ORDEN). Si `fugas.py` marca eventos `hook_*` del mod (HOOKS["ars"] es `set()`, hipotesis estricta), se para, se documentan los eventos en `erratas.md` y se ajusta `HOOKS["ars"]` antes de lanzar. La sonda debe comprobar además que el hb de la corrida (`$HOME/.claude/exo-rules/hb-<sid>`, con el HOME real del harness, no `$O`; el `<sid>` es el `session_id` del evento `init` de `$O/transcript.jsonl`, que `correr.sh` genera con `--output-format stream-json --verbose`) dice `via=submit`: en la máquina Linux de cuenta personal compose está vivo, así que `via=compose` significaría que `EXO_RULES_FORZAR_SUBMIT` no llegó al mod y la eval mediría el system prompt; si no es `via=submit`, se para. Solo si la sonda sale con `fuga:false`, con `result` y con `via=submit` se lanza la tanda.
- **Estado de la tanda:** `TECHO_P=techo-reglas-2-ars` (estado propio en `$K_ROOT`); `correr-techo.sh` aborta si el estado ya es de otra tanda con otro ORDEN.
- **Adjudicacion:** `evaluar.py` trae el gate >=6 cableado; para `ars` solo vale el recuento n/10 contra el umbral >=4 de arriba.

>=4/10 cumplen = util; <4 = placebo

## Orden de corridas (barajado, semilla 20261006)

Se parte de la lista ordenada (`sort`) de las 20 tuplas `(brazo, id, rep)` derivadas de `tareas-ars.tsv` y se baraja con `random.Random(20261006).shuffle`. Comando, desde la raiz del repo:

```bash
python3 -c 'import random,csv; L=[]
for i,g,_,b in csv.reader(open("evals/techo-reglas-2/tareas-ars.tsv"),delimiter="\t"):
    for x in b.split(","):
        a,k=x.split(":"); L+=[(a,i,r) for r in range(1,int(k)+1)]
L.sort(); random.Random(20261006).shuffle(L)
print("\n".join(f"{a} {i} {r}" for a,i,r in L))'
```

<!-- ORDEN-BEGIN -->
ars g2-97 1
ars g0-33 1
ars g2-154 1
ars g0-33 2
ars g0-122 2
ars g2-97 2
ars g1-131 2
ars g0-122 1
ars g2-170 1
ars g1-140 2
ars g1-140 1
ars g1-157 2
ars g0-149 2
ars g1-139 1
ars g1-157 1
ars g1-131 1
ars g2-170 2
ars g1-139 2
ars g0-149 1
ars g2-154 2
<!-- ORDEN-END -->

## Limitaciones declaradas

- **Potencia:** 10 tareas, un solo brazo y k=2: el umbral 4/10 es bajo a proposito (separa "algo llega" de "nada llega") y no estima el tamano del efecto.
- **Forzado:** `FORZAR` simula la politica; no se mide la politica real, y compose queda inerte por construccion.
- **Compactacion (S6):** el `context` de `prompt.submit` puede compactarse a mitad de un turno agentico largo de `-p`; `arp` (system prompt) no. Un resultado `<4` puede deberse a eso y no a que las reglas por submit no sirvan.
- **Sin placebo ni control:** no hay controles de caida; la comparacion con `a0`/`arp` es de tandas distintas.
