# Línea base de mutation score: exo (pre-A+)

- Rama seleccionada: `campana-l` (Merge campaña L: sueltos pre-v0.2.0 y preparación de M5b), merge `7e29f42` del 2026-09-20. El nombre de rama sale del mensaje del merge; el plan es `docs/superpowers/plans/2026-09-19-campana-l-sueltos-pre-v020.md`.
- Rango: `5efe812..f79d817` (BASE = primer padre = merge-base, porque la rama es descendiente directa del primer padre; HEAD = segundo padre). 9 commits.
- Instrumento: review-package de exo 1.5.0, `EXO_MUTATION_TIMEOUT=3600`, cargo-mutants 27.1.0 (mutmut 2.5.1 en venv2 y stryker en PATH, sin uso: el diff solo tiene Rust).
- Tiempo: 26:52,25 wall (3783 s user, 504 s sys).
- Diff: `linea-base/exo.diff` (451 499 bytes).

## MUTACIÓN (literal)

    ## MUTACIÓN
    MUTACIÓN: 13/13 (100%) — 0 superviviente(s)
    MUTACIÓN [engine]: 13/13 (100%) — 0 superviviente(s)

Ni `parcial` ni `no disponible`: entra en el agregado (13/13).

## Clasificación de supervivientes

No hay supervivientes, nada que clasificar. Observación: solo 13 mutantes viables. El código de producción del diff en `engine/src/` es `buscador.rs` (+145 líneas netas); el resto del rango es docs/scripts/tests. Un 100% con n=13 es una base con poca resolución.

## Tabla de candidatas (rastro de auditoría; merges a main 2026-09-01..09-28, más reciente primero)

| Merge | Rama | Plan con código | Producción mutable en diff | Veredicto |
|---|---|---|---|---|
| 8fe5242, 52d693d, 78b64e9 | origin/main, a-plus-plan2, auditoria-skills | n/a | n/a | Descartados por regla (origin/main, A+; fecha 09-29, fuera de rango) |
| 6c02b06 | backlog-padres-refs | no | no (diff vacío en engine/py/js) | Descartada |
| d489228 | c-backlog-sync | no | no | Descartada |
| 063d07c | ola 2 B2 (vec0 L2, H28) | no (sin plan propio; H28 solo se cita en planes de otras campañas) | sí (engine/src/buscador.rs, vectores.rs...) | Descartada: falla la condición 2 |
| ca40d04 | ola 2 B1 (bench.sh) | no | no (bash) | Descartada |
| 01be694 | ola 2 A (gold J) | solo toca el BORRADOR del pre-registro | solo evals/…/harness/*.py | Descartada: no hay plan ejecutado por el pipeline viejo y el Python es harness de evals, no producción (juicio mío, ver desviaciones) |
| b8e3117 | PR #28 fix-release-publish-idempotente | no | no (release/plugin scripts) | Descartada |
| aaf66c0 | campaña J fase 1 | sí (campana-j-fase1) | solo evals/retrieval-heldout/harness/*.py | Descartada: Python de harness de evals, no producción (juicio mío) |
| 18ba004 | campaña I | sí (campana-i) | no (sin engine/src, py ni js) | Descartada |
| **7e29f42** | **campaña L** | sí (campana-l) | sí (engine/src/buscador.rs) | **SELECCIONADA** |
| 5efe812 | PR #27 fix-23-pie-recall | no | no (recall-inject.sh) | Descartada |
| e9ed208 | PR #26 campana-g | sí (campana-g-engine-deuda-diferida) | sí (engine/src, 36 ficheros) | **RESERVA** (2ª más reciente; no medida) |

Nota sobre la reserva: el pre-registro dice "segunda más reciente de exo" como reserva para otros repos sin candidata; aquí la identifico por si la piden.

## Desviaciones del pre-registro

1. Interpretación de "código de producción" (condición 3): descarté 01be694 y aaf66c0 porque su único código mutable es Python de harness de evals (`evals/retrieval-heldout/harness/`). Si Paul los considera producción, la selección cambiaría a aaf66c0 (J fase 1). Es la decisión con más riesgo de discusión.
2. Ninguna en el instrumento: versiones y timeout según el pre-registro. venv (mutmut 3.x) no se usó; venv2 (2.5.1) se antepuso al PATH aunque no se ejerció.
3. Repo intacto tras la medición (status limpio, sin worktrees residuales).
