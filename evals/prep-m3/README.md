# Eval prep-M3 — paridad de movimientos del plugin `process`

Gold set de la pieza prep-M3 (framework §5.3 paso 1: skills de `process`
escritas y revisadas, SIN instalar). Spec de diseño:
`docs/superpowers/specs/2026-07-17-prep-m3-process-skills-design.md`.

## Qué hay aquí

- `gold/<skill>.md` — un checklist por skill (brainstorm, plan, orchestrate,
  tdd, debug, verify, documenta). Cada ítem es UN movimiento esencial que la
  skill destilada debe conservar, con cita exacta de fuente (path del
  SKILL.md fuente + sección/líneas). Cada fichero cierra con una sección
  **DESCARTES**: lo que se tira deliberadamente y su porqué en una línea.
- (Futuro, post-cutover M3) `no-disparos.md` — contador append-only de "skill
  que debió disparar y no disparó"; diseño en la spec §6. No existe aún: se
  crea en el cutover, no en esta pieza.

## Cómo se usa el gold

Este eval no tiene oráculo mecánico — config de fábrica §Oráculos: *"el
'oráculo' es el checklist de paridad de movimientos vs la skill superpowers
absorbida (spec §5.2, tabla), verificado por el consultor-gate, no por
comando"*.

1. El revisor (humano o consultor-gate, fresco: sin haber participado en la
   implementación que juzga) abre la skill implementada
   (`plugins/process/skills/<skill>/`) al lado de `gold/<skill>.md`.
2. Por cada ítem del checklist: ¿el movimiento está presente en el SKILL.md o
   en sus reference files? Marca `[x]` (presente) o lo deja `[ ]` (ausente).
   Presente/ausente, **sin juicio estético** — la redacción destilada puede
   diferir de la fuente; lo que se verifica es que el movimiento sobrevive.
3. Por cada ítem de DESCARTES: verificar la AUSENCIA. Un descarte que
   reaparece en la skill implementada es fallo de paridad.
4. Barrido inverso (0 movimientos nuevos): todo movimiento de la skill
   implementada que no esté en el checklist ni tenga cita de fuente se
   reporta como "movimiento nuevo sin cita".

## Criterio de cierre

Una skill implementada se acepta cuando:

- **Paridad 100%**: todos los ítems de su gold marcados presentes (incluido,
  en `orchestrate`, el ítem PARIDAD CRÍTICA: dispatch
  `subagent_type: reflex:executor` SIN `model` — framework §5.3.2).
- **0 movimientos nuevos sin cita**: el barrido inverso no encuentra nada.
- **0 descartes resucitados**.

El verdict del consultor-gate se escribe versionado (config §Ejecución de
gates, condición 4) — patrón `evals/prep-m3/verdict/<skill|lote>.md`.

## Kill-criteria (pre-registrados en la spec §9 — resumen)

1. Skill no cabe en ~30-50 líneas de body sin perder movimientos ⇒ carne a
   reference files; si aun así pierde movimientos ⇒ escalar, no recortar el
   checklist.
2. **Cap de retries por eval: 2** (config §Presupuesto). Tras 2 retries sin
   paridad ⇒ parar y escalar.
3. Movimiento nuevo sin cita = fallo aunque el resto esté al 100%.
4. El ítem PARIDAD CRÍTICA no se negocia en ningún retry.

## Qué NO es esta pieza

Ni instalar `process`, ni deshabilitar superpowers, ni tocar
fabrica//consolida, ni los pasos 2-4 del checklist §5.3 — todo eso está
gateado por `GATE-CALENDARIO-D` (config de fábrica).
