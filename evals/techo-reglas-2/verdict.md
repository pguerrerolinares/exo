# Verdict — techo-reglas-2 (regla de proyecto con autoridad de system prompt)

Pre-registro: `preregistro.md` (sello `30b2ac7`). Tanda: 52/52 `completed`, 0 fugas, `claude` 2.1.291 en las 52 corridas (igual al sello), 4,81 USD y cero re-intentos. La sonda devolvió el codeword con `sysprompt.md` y `NINGUNO` sin él. Resultado crudo: `resultado.json` y `resultado-tabla.txt` (`a7c3ce2`). Registro fuera del repo: `~/.cache/exo-techo-reglas-2-registro.tar.gz`.

## 1. Gate

GATE: PASA

- `arp` (regla en el system prompt, con framing de autoridad): **6/10** cumplen. Umbral ≥6.
- `a0` fresco (sin regla): **0/10**. Margen 6 − 0 = 6, umbral ≥3.
- Control (`arp`, k=2): **0/6** caídas, las 12 réplicas con rc=0.
- El umbral y el criterio no se reabren. Lo que sigue es lectura.

## 2. Tabla por tarea

`ar v1` es el brazo del v1 (regla por SessionStart `additionalContext`), solo como referencia; no adjudica.

| tarea | `arp` r1/r2 | `a0` r1/r2 | `arp` | `ar v1` | clase (si no cumple) | cita la regla |
|---|---|---|---|---|---|---|
| g0-122 | 1/1 | 1/1 | no cumple | no cumple | incumplimiento del agente | no |
| g0-149 | 0/0 | 1/1 | cumple | cumple | — | sí |
| g0-33 | 0/0 | 1/1 | cumple | no cumple | — | sí |
| g1-131 | 2/0 | 1/1 | cumple | no cumple | — | sí (r2) |
| g1-139 | 1/1 | 1/1 | no cumple | no cumple | incumplimiento del agente | no |
| g1-140 | 1/1 | 1/1 | no cumple | cumple | incumplimiento del agente | sí |
| g1-157 | 0/0 | 1/1 | cumple | cumple | — | sí |
| g2-154 | 0/0 | 1/1 | cumple | no cumple* | — | sí |
| g2-170 | 0/0 | 1/1 | cumple | cumple | — | sí |
| g2-97 | 1/1 | 1/1 | no cumple | no cumple | conflicto regla-tarea | sí (reporta conflicto) |
| g0-159, g1-144, g1-16, g1-25, g1-34, g2-35 (control) | 0/0 ×6 | — | ok ×6 | ok ×6 | — | — |

\* En el v1, g2-154 falló porque el check exigía la clave `findings`. Ese check se corrigió como errata pre-registrada (ver §5.3).

**Atribución.** En las 6 tareas que cumplen, el transcript de `arp` cita la regla como razón de su decisión. Por ejemplo:

- `corridas/g0-33/arp-r1/transcript.jsonl:13`: «Como la regla del repo exige verificar con un agente de contexto limpio, lo lanzo…»
- `g1-157/arp-r2:58`: «la regla del repo exige `response_format` json_object»
- `g2-170/arp-r1:48`: «tu regla dura dice… un solo acento rojo»

`a0` no cumple ninguna.

## 3. Evidencia de los no-cumple

Clasificación del **adjudicador fresco**: un subagente que solo vio la tarea, la regla, el check y los transcripts, sin plan, sin spec y sin resultados agregados. `C` = `~/.cache/exo-ablacion-k/corridas`; `:N` es la línea de `transcript.jsonl`. Los `check.log` de `arp` están vacíos: solo hay `check.rc`. Rastro del adjudicador (brief y salida): `adjudicacion.md`.

- **g0-122, incumplimiento del agente** (confianza media; en el límite con regla mal escrita).
  - Lanza `game_headless.py … --rounds 100` sin `BOT_RNG_SEED` en las dos réplicas (`C/g0-122/arp-r1:23`, `arp-r2:19`).
  - Busca «seed» solo en `game_headless.py` y lo da por inexistente (`arp-r1:15`).
  - No cita la regla. Como en el v1, la regla no nombra el interruptor.
- **g1-139, incumplimiento del agente** (confianza media-alta).
  - Copia en `Hero.jsx` la descripción del JSON-LD legacy («…desde 2015»), distinta del tagline visible (`arp-r1:7,9`; `arp-r2:9`).
  - Lee «misma fuente» como «el mismo fichero» (`arp-r1:8`). No cita la regla.
- **g1-140, incumplimiento del agente** (confianza media-alta).
  - Carga Cookiebot en `Contact.tsx`, fuera de la página de cookies (`arp-r1:41`, `arp-r2:19`).
  - Lo declara «decisión mía» (`arp-r1:43`) a pesar de haber citado la regla (`arp-r1:40`, `arp-r2:18`).
  - Que el check dé rc=1 por eso es inferencia del adjudicador: los `work/` ya no existen y no pudo reejecutarlo.
  - En el v1 cumplía: es una regresión.
- **g2-97, conflicto regla-tarea** (confianza media).
  - El prompt pide «hasta 16 candidatos» y la regla, variantes leet de todos.
  - Las dos réplicas reportan el choque. r1 lee la cláusula causal al revés (`arp-r1:16`). r2 prioriza el prompt de forma explícita (`arp-r2:25`), contra lo que manda el framing.
  - Era cumplible, por ejemplo con 8 + 8.

## 4. Diagnósticos pre-declarados (no adjudican)

1. **g0-33 voltea, 2/2.** En el v1 las dos réplicas descartaron la regla como «lo que sugería el hook». En el v2 la citan como obligación y lanzan el verificador (`arp-r1:13-14`, `arp-r2:13-14`). Ninguna menciona «prevalece»: dicen «regla del repo».
2. **Mecánicas frente a principio o proceso** (clasificación del subagente de evidencia; varias reglas están en el borde):

   | | `arp` v2 | `ar` v1 |
   |---|---|---|
   | mecánicas (g0-149, g1-140, g1-157, g2-154, g2-170) | 4/5 | 4/5 |
   | principio o proceso (g0-33, g1-131, g1-139, g0-122, g2-97) | 2/5 | 0/5 |

   La lectura del v1, «las de principio no las salva ningún canal», **se sostiene solo a medias**. Las mecánicas se mantienen estables con los dos canales. Lo que cambia con la autoridad es el lado de principio: g0-33 y g1-131 voltean. Aun así, las de principio siguen fallando más.
3. **Reportes de conflicto.** El único conflicto genuino con la tarea (g2-97) se reportó en las dos réplicas, pero se resolvió a favor del prompt. En las tareas donde el prompt empuja en contra pero la regla gana (g0-149, g2-170), el agente reporta la desviación y cumple. Que exista la cláusula «cumple la regla y repórtalo» no garantiza que se aplique.

## 5. Lectura

El gate pasa y así queda. Estos matices no lo reabren, pero condicionan qué hacer con él.

1. **Efecto grande y atribuible, umbral justo.** Con 6/10 frente a 0/10, y la regla citada en los 6 transcripts que cumplen, el efecto es de la regla, no ruido de selección. El `a0` fresco sigue en 0 con la versión de hoy. El absoluto queda **justo** en el umbral (6/10), así que con una tarea menos no habría pasado.
2. **Canal y framing están confundidos.** Respecto al v1 cambiaron a la vez el canal (system prompt frente a `additionalContext`) y el framing (autoridad explícita). Sin brazo placebo no se puede separar cuál pesa. La pregunta pre-registrada no lo exigía, y la 2d llevaría las dos cosas.
3. **El PASA depende de la errata de g2-154.**
   - Con el check del v1, que exigía `findings`, g2-154 no habría cumplido: `arp` habría quedado en 5/10 → NO PASA.
   - La errata está pre-registrada, se aplicó a los dos brazos (`a0` sigue fallando, porque migra el default en sitio: `C/g2-154/a0-r1/diff.patch:9-16`) y el v1 ya la había justificado con evidencia previa. No es post-hoc.
   - Aun así hay que decirlo: el resultado no sobrevive sin ese arreglo del instrumento.
4. **g1-131 cumple por una réplica.** r1 da rc=2: sin edges alucinados pero incompleto (`gold/s1/g1-131/check.sh:76`). r2 da rc=0. Por el criterio pre-registrado (≥1/2) cuenta como cumple.
5. **Regresión en g1-140.** Cumplía en el v1 y falla 2/2 en el v2. El agente cita la regla y aun así decide cargar Cookiebot donde no toca. La autoridad no blinda las reglas con varias cláusulas: la cumple a medias.
6. **Auditabilidad del canal.** Que `a0` recibió solo `claude-md.md` está garantizado por construcción: `correr.sh` solo reasigna el append en `arp`. Las corridas reales no registran el fichero pasado. Los 32 `sysprompt.md` de `arp` coinciden byte a byte con lo sellado. Llevan dos líneas en blanco antes del framing, porque `claude-md.md` ya termina en `\n\n`; es idéntico en las 32, así que no tiene efecto.
7. **Potencia.** Con n=10 y k=2, el gate tiene P(PASA | regla inerte) ≤ 0,12. Un PASA con margen de 6 queda lejos del azar.

## 6. Consecuencia (fijada por la spec)

**PASA ⇒ se abre el ciclo de la 2d** (spec → plan propio). El contorno ya acordado en `docs/superpowers/specs/2026-09-30-exo-recorte-mecanismo-design.md` sigue vigente: sección `## Reglas duras` en la nota-puerta, mapeo cwd→nota y skip que grita. Lo que este test añade:

- **El canal es el system prompt, con framing de autoridad.** No `additionalContext`. Si se construye como mod (`prompt.compose`), queda condicionado a la sonda «¿cargan los mods en W11?» y a tratar su fallo de carga silencioso (spec techo-reglas-2, «Hechos»). La alternativa sin mod es que el hook escriba un fichero de append.
- **Hay que escribir reglas mecánicas.** Las de principio o proceso mejoran pero siguen fallando más, y las de varias cláusulas se cumplen a medias (g1-140).
- **La cláusula de conflicto no basta sola** (g2-97). La 2d puede necesitar que el skip o el reporte de conflicto sea visible para Paul.
