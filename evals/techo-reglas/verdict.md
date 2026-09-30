# Verdict — test de techo de las reglas de proyecto (gate de la 2d)

Pre-registro: `preregistro.md` (sello `1921aee`). Corridas: 28/28 `completed`, 0 fugas, 2,73 USD, cero re-intentos. Sonda de canal OK (`TECHO-7Q`) antes de la tanda. Resultado crudo: `resultado.json`, `resultado-tabla.txt` (`2fc8424`). Registro fuera del repo: `~/.cache/exo-techo-registro.tar.gz`.

## 1. Gate

GATE: NO PASA (4/11 < 6)

- Suelo: 4/11 cumplen (g0-149, g1-140, g1-157, g2-170). Umbral ≥6.
- Control: 0/6 caídas. La inyección por SessionStart no rompe tareas que A0 ya resolvía.
- El umbral y el criterio no se reabren. Lo que sigue es lectura.

## 2. Tabla por tarea

`a0 K` = brazo sin regla de la campaña K (tarball). `ar` = esta tanda.

| tarea | grupo | a0 K r1/r2 | ar r1/r2 | resultado | clase (si no cumple) |
|---|---|---|---|---|---|
| g0-122 | suelo | 1/1 | 1/1 | no cumple | regla mal escrita |
| g0-149 | suelo | 1/1 | 0/1 | cumple | — |
| g0-33 | suelo | 1/1 | 1/1 | no cumple | fuera de las 4: incumplimiento limpio (ver §3) |
| g1-131 | suelo | 1/1 | 2/1 | no cumple | check roto (r1) · fuera de las 4 (r2), ver §3 |
| g1-139 | suelo | 1/1 | 1/1 | no cumple | regla mal escrita |
| g1-140 | suelo | 1/1 | 0/1 | cumple | — |
| g1-157 | suelo | 1/1 | 0/0 | cumple | — |
| g2-154 | suelo | 1/1 | 1/1 | no cumple | check roto |
| g2-170 | suelo | 1/1 | 0/0 | cumple | — |
| g2-171 | suelo | 1/1 | 1/1 | no cumple | conflicto regla-tarea |
| g2-97 | suelo | 1/1 | 1/1 | no cumple | regla mal escrita |
| g0-159 | control | — | 0 | ok | — |
| g1-144 | control | — | 0 | ok | — |
| g1-16 | control | — | 0 | ok | — |
| g1-25 | control | — | 0 | ok | — |
| g1-34 | control | — | 0 | ok | — |
| g2-35 | control | — | 0 | ok | — |

Todas las corridas terminaron con `fin = completed`: ningún "cumple" sale de una corrida cortada.

## 3. Evidencia de las tareas que no cumplen

`C` = `~/.cache/exo-ablacion-k/corridas`; `G` = `~/.cache/exo-ablacion-k/gold/s1`. `:N` es la línea de `transcript.jsonl`.

- **g0-122, regla mal escrita.** La regla ("det-mode reproducible, RNG con seed…") no nombra el interruptor, que es la variable `BOT_RNG_SEED` (`fuentes/g0-122/bot/common/src/rng.rs:6,18`). Ni el README ni `--help` la mencionan. El check exige `BOT_RNG_SEED` en el comando. r1 busca "seed" solo en `game_headless.py` y concluye que no hay flag (`C/g0-122/ar-r1:14`). r2 cita la regla y la da por cumplida porque "no hay wall-clock en juego" (`C/g0-122/ar-r2:30`). Hay un matiz: un grep fuera de `engine/` la habría encontrado.
- **g0-33, fuera de las 4 clases: incumplimiento limpio y deliberado.** La regla es clara y aplicable. Las dos réplicas la ven y deciden no lanzar el agente verificador: r1 "no con el agente de contexto limpio que sugería el hook" (`C/g0-33/ar-r1:25`), r2 "No lancé el agente de verificación que sugería el hook" (`C/g0-33/ar-r2:20`). Las cifras entregadas son correctas. Cumplen la intención de verificar contra la fuente primaria, pero no la letra. Ninguna de las 4 clases del plan describe esto, así que no la fuerzo.
- **g1-131, check roto (r1, rc=2) · incumplimiento de ejecución (r2).** Reproducido reconstruyendo con `setup.sh` + `diff.patch`. `build_table` indexa sin prefijo de lenguaje, y `py:` y `ts:` colisionan. r1 añade un guard de prefijo, no alucina ningún edge (que es lo que pide la regla) y deja sin resolver dos positivos legítimos. El check da 2 por completitud, y para cumplirlo habría que arreglar `build_table`, que el prompt no pide tocar. r2 cita la regla (`C/g1-131/ar-r2:15`, "nunca cruza `py:` ↔ `ts:`"), pero cruza lenguajes y alucina `discount`. En r2 la regla llegó y se intentó cumplir, pero la implementación falló.
- **g1-139, regla mal escrita.** La regla dice "los datos estructurados salen de la misma fuente que el contenido visible". El agente toma nombre y descripción del JSON-LD de `legacy/index.html` (`C/g1-139/ar-r1:12-13`, `ar-r2:13`), que es una "misma fuente" defendible. El check exige el titular o tagline literal y prohíbe "desde 2015": es una lectura más estricta que el texto de la regla. Solo falla la cláusula `STALE`.
- **g2-154, check roto.** Las dos réplicas hacen lo que pide la regla: el default de `doctor --check-schema` conserva `{ok,missing}`, la shape nueva va tras `--detailed` y `gate.sh` sigue saliendo 1 (`C/g2-154/ar-r1:17`, `ar-r2:16,19`). El check exige la clave `findings` (`G/g2-154/check.sh:27`), el agente usó `problems` (`C/g2-154/ar-r1/diff.patch`) y el prompt no nombra la clave. Con `problems` en el check, r2 da rc=0.
- **g2-171, conflicto regla-tarea.** El prompt pide "tipografía serif en cursiva" y ninguna de las tres fuentes permitidas es serif. Las notas de la tarea ya lo reconocen. Las dos réplicas ven el dilema (`C/g2-171/ar-r1:29`, `ar-r2:62`), resuelven con `ui-serif, Georgia, …, serif` y dejan la decisión al usuario. El check cuenta `ui-serif` como violación.
- **g2-97, regla mal escrita.** La regla es "Genera variantes leet … sin usarlas como seguro de envío porque el formulario normaliza leet", y su cláusula causal se lee como razón para no generarlas. Las dos réplicas la usan así: `C/g2-97/ar-r1:12` ("solo habrían gastado slots") y `ar-r2:46`. Hay además una tensión con el tope de 16 candidatos del prompt. `docs/postmortems.md` aclara la intención, pero ningún agente lo leyó.

## 4. Canal

- El hook entregó la regla en 22/22 corridas del suelo. En la línea 2 de cada `transcript.jsonl` hay un `hook_response` SessionStart con `exit_code=0`, y el `additionalContext` es idéntico byte a byte a `reglas/<id>.txt`.
- Hay prueba de que el modelo la vio (la cita o la parafrasea) en 10 corridas. En otras 2, las de g2-170, es probable. En las 10 restantes no hay eco. No hay ninguna evidencia de que no llegara. Los thinking blocks vienen redactados, así que en esas 10 no se distingue "no la vio" de "la vio y no la citó".

## 5. Lectura

El gate no pasa y así queda. Estos matices no lo reabren, pero cambian qué significa el número:

1. **La regla mueve el suelo de 0 a 4.** En K, A0 fue rc=1 en las 22 corridas de estas 11 tareas. La inyección por SessionStart no es inerte: en g0-149 r1 y g1-157 r2 el agente cita la regla al cumplir. En g2-170 y g1-140 r1 no hay cita, pero sin regla fallaban 2/2 (atribución por inferencia).
2. **El techo medido lo limita el instrumento tanto como el agente.** De las 7 tareas que no cumplen:
   - solo g0-33 es un incumplimiento limpio de una regla clara: el agente la vio y decidió no aplicarla;
   - g1-131 es mixta: r1 cae por el check (`check roto`) y r2 es un intento fallido;
   - las otras 5 son instrumento: 1 `check roto` (g2-154), 3 `regla mal escrita` (g0-122, g1-139, g2-97) y 1 `conflicto regla-tarea` (g2-171).
   
   Esto no autoriza a recontar: el pre-registro fija el suelo con estas reglas y estos checks. Sí dice que "4/11" mide el paquete regla+check+agente, no el agente solo.
3. **g2-170 era el conflicto regla-tarea conocido y ahora cumple 2/2.** Las dos réplicas usan un solo acento rojo y no el azul para `info`. La expectativa previa no se confirmó, y la clase se apoya en el transcript, como pedía el plan.
4. **Inconsistencia entre réplicas.** g0-149 y g1-140 cumplen en r1 y fallan en r2. En los dos casos el prompt empuja en contra de la regla ("déjalo montado"; Cookiebot en otra página). Con k=2 no se puede separar el ruido de la sensibilidad al prompt.
5. **Reconstrucción.**
   - g0-122 (35 cambios sin commitear en K), g2-170 y g2-171 (2 cada una) corrieron sobre el commit limpio. Por la evidencia, no parece afectar a ninguna de las tres, pero no está probado.
   - Diferencias respecto a K: `claude` 2.1.286 frente a 2.1.285. `claude-md.md` coincide (`6f3c6f382f90e794`).
   - Incidente posterior a las corridas: un `check.sh` del gold se modificó y se restauró (ver `erratas.md` E1). No afecta a ningún `check.rc`.

La conclusión operativa para la 2d la toma Paul, no este documento. El número dice que las reglas de proyecto inyectadas no alcanzan el techo pre-registrado. La lectura dice que la mitad del hueco está en cómo se escribieron las reglas y los checks.
