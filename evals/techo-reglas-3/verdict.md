# Verdict: techo-reglas-3 (dilución de reglas en el system prompt)

Pre-registro sellado en `e03844f` y enmendado antes de cualquier corrida en `8daf967` (review del consultor). Sonda del canal en 2.1.310: OK (0,019 USD). Tanda: 60/60 corridas con `result` (`completed`), 0 fugas, 0 omitidas, las 60 en `2.1.310 (Claude Code)`, 5,14 USD. No saltó ningún breaker. Registro: `~/.cache/exo-techo-reglas-3-registro.tar.gz`.

## Adjudicación (`evaluar3.py`)

| brazo | cumplen |
|---|---|
| `arp1` (gold sola) | 5/10 |
| `arp10` (gold + 9) | 5/10 |
| `arp20` (gold + 19) | 6/10 |

- **Replica:** `arp1` 5/10 ≥ 5, así que se adjudica. Queda justo en el umbral: techo-reglas-2 dio 6/10 en 2.1.291.
- **CAP: 10.** `arp20` 6/10 ≥ 5.
- **§7.3: no se cumple.** `arp10` 5/10 > 3.

## Por tarea (`check.rc` r1,r2)

| tarea | arp1 | arp10 | arp20 |
|---|---|---|---|
| g0-122 | 1,1 | 1,1 | 1,1 |
| g0-149 | **0,0** | **0,0** | **0,0** |
| g0-33 | **0,0** | 1,1 | 1,1 |
| g1-131 | 2,2 | 2,2 | **0**,2 |
| g1-139 | 1,1 | 1,1 | 1,1 |
| g1-140 | 1,1 | 1,**0** | **0**,1 |
| g1-157 | **0,0** | **0,0** | **0,0** |
| g2-154 | **0,0** | **0,0** | **0,0** |
| g2-170 | **0,0** | **0,0** | **0,0** |
| g2-97 | 1,1 | 1,1 | 1,1 |

## Lectura

- Con 20 reglas no hay dilución detectable. Siete tareas dan el mismo resultado en los tres brazos. El `arp20` > `arp1` sale de g1-131 y g1-140, que cumplen en 1 de 2 réplicas. Por el pre-registro, eso es ruido y no evidencia: sin dilución, P(`arp20` > `arp10`) ≈ 35 %.
- g0-33 es la única tarea que cumple sola y deja de cumplir con distractoras, en las 4 réplicas de `arp10` y `arp20`. Es la gold transversal, que va en la lista «del dueño» rodeada de 9 globales. Es un caso, no un patrón, pero es la forma que tendría la dilución y apunta a las globales, que son lo que se siembra a continuación.
- g1-131 da `rc=2` en 5 de 6 corridas. Según su `check.sh`, eso significa «no evaluable», que incluye «no resolvió los edges legítimos sin alucinar». El agente resolvió con catálogo, pero no todos los edges del catálogo oculto. Cuenta como no cumple, como se pre-registró; en techo-reglas-2 ya dio 2,0.
- Potencia (declarada): este resultado descarta la dilución fuerte (con ≈ 2,7/10 en `arp20` saldría CAP 5 el 72 % de las veces), pero no la moderada. Esa le toca al brazo `Ap` de §7.2, cuyo pre-registro tiene que fijar la unidad de medida (#190).

## Decisión

`CAP` = 10, que es el valor ya desplegado en `exo rules` (engine `v0.10.0`): no hay que cambiar código. Lo siguiente es la siembra de #184.
