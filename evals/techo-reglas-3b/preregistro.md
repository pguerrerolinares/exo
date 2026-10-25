# Pre-registro: techo-reglas-3b (mini-tanda de globales sobre g0-33)

Sellado por el commit que crea este fichero, antes de cualquier corrida real. Viene de la ronda 3 del consultor sobre techo-reglas-3 (`evals/techo-reglas-3/verdict.md`, «Lectura»): g0-33, la única regla gold que es global y de proceso, cumple sola (2/2) y cae con 10 globales (0/4). Esta mini-tanda decide con cuántas globales arranca la siembra de `core/reglas-duras.md` (#184). Es descriptiva y direccional: una sola tarea, sin potencia para generalizar.

## Diseño fijo

- **Canal, framing, modelo, prep y gold:** los de techo-reglas-3, sin cambios (`framing.txt` y `pins.sha256` copiados; `claude-sonnet-5-5`, `--max-turns 40`, `--max-budget-usd 10`).
- **Bloques:** `generar-bloques.py` reutiliza las colas, los descartes y el render de techo-reglas-3. La lista «del dueño» lleva N globales con la gold en la posición min(4, N-1); la lista «del proyecto» son las 10 distractoras de `arp20`. Brazos: `arp903` (3 globales, 13 reglas en total), `arp905` (5, 15) y `arp910` (10, 20). `arp910` es byte a byte el `arp20` de g0-33. Sha256 en `bloques.sha256`; los bloques no se versionan.
- **k:** 3 por brazo, 9 corridas. Éxito de una corrida = `check.rc == 0`. Se reporta k de 3 por brazo; la referencia sola es el `arp1` de techo-reglas-3 (2/2).
- **Decisión (tamaño inicial de la lista global):** si `arp905` >= 2/3, se siembran 5. Si `arp905` <= 1/3 y `arp903` >= 2/3, se siembran 3. Si los dos <= 1/3, el número no explica la caída: se siembran 5 globales redactadas como mecánicas (sin reglas de proceso del tipo «lanza un agente») y se abre issue sobre la dilución de reglas de proceso. `arp910` es la réplica de `arp20` (0/2) y no decide.
- **Versión de claude:** `claude --version` = `2.1.310 (Claude Code)` al sellar; si difiere antes de correr, se para.
- **Re-intentos y breaker:** los de techo-reglas-3 (cero re-intentos; fuga, más del 10 % sin `result` o gasto > 15 USD invalidan la tanda).
- **Comando:** `TECHO_EXP=evals/techo-reglas-3b bash evals/techo-reglas/correr-techo.sh 3`.

## Orden de corridas (barajado, semilla 20261025)

Mismo procedimiento que techo-reglas-3, sobre `evals/techo-reglas-3b/tareas.tsv`.

<!-- ORDEN-BEGIN -->
arp903 g0-33 3
arp905 g0-33 2
arp903 g0-33 1
arp905 g0-33 1
arp905 g0-33 3
arp903 g0-33 2
arp910 g0-33 2
arp910 g0-33 3
arp910 g0-33 1
<!-- ORDEN-END -->

## Limitaciones declaradas

- **Una tarea, k=3:** distingue una caída total de una recuperación clara; nada más fino. No separa «número de globales» de «tipo de regla» (proceso frente a mecánica).
- **Número total confundido:** al variar las globales, el total pasa de 13 a 15 y a 20 reglas, con la lista de proyecto fija.
- **Posición:** la gold queda la última en `arp903` (posición 2 de 3) y en la posición 4 en `arp905` (también la última) y en `arp910`.
