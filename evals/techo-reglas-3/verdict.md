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

Corregida tras la ronda 3 del consultor fresco, que leyó los transcripts y reconstruyó los workdirs de g1-131.

- **La réplica es fuerte, no «justa».** En `arp1` violan la regla (`rc=1`) g0-122, g1-139, g1-140 y g2-97: exactamente las mismas 4 que en techo-reglas-2. El paso de 6 a 5 lo explica g1-131, que pasa de (2,0) a (2,2), y `rc=2` es «incompleto», no violación. En 2.1.310 no aparece ninguna violación nueva.
- **Con 20 reglas no hay dilución detectable de las reglas mecánicas.** P(`arp20` = 6) vale 0,41 sin dilución, 0,37 con dilución leve, 0,16 con moderada y 0,015 con fuerte. Descarta la fuerte, pero contra la moderada es débil (razón de verosimilitud ≈ 2,6). El `arp20` > `arp1` sale de g1-131 y g1-140, en 1 de 2 réplicas, y es ruido (pre-registrado). En g1-140, las 6 corridas citan reglas del bloque; da `rc=0` solo cuando Cookiebot queda en la página de cookies.
- **g0-33: dilución probable, y es la única medida que tenemos de una regla global y de proceso.** En `arp1` (2/2) lanza `Agent` y cita la regla. En `arp10` y `arp20` (0/4) no lanza ningún `Agent`, verifica él mismo con python y no menciona ninguna regla del bloque. La nota es correcta en las 6: se pierde el mecanismo, no la calidad. No lo explica el encabezado (es igual en los tres brazos), ni una distractora concreta (ninguna habla de agentes ni de verificación), ni una posición extrema (2/5 y 4/10). Fisher 2/2 frente a 0/4: p = 0,067. Las 9 golds mecánicas no diluyen; la única de proceso, sí. La spec (§5.3) da las globales por «cubiertas por §7.1», pero esa cobertura es n = 1 y en contra.
- **g1-131: el `rc=2` mide completitud, no violación.** Los 6 workdirs reconstruidos (`setup.sh` + `diff.patch`) tienen 0 edges alucinados. El `rc=2` sale de que `by_module_name` indexa sin prefijo, `py:` pisa a `ts:` y el prefix-guard devuelve None. No es un check roto, pero para medir dilución es ruido puro. Cuenta como no cumple, como se pre-registró.
- **Potencia:** la dilución moderada de las reglas de proyecto le toca a `Ap` (§7.2), y su pre-registro tiene que fijar la unidad (#190). `Ap` no cubre las globales.
- **Menores del instrumento:** los `check.log` están vacíos; se borra `work/` y lo no trackeado queda fuera de `diff.patch`, así que g1-140 no se puede re-adjudicar; `turnos` de `meta.json` no es fiable (1 turno con 25 mensajes); la enmienda pre-tanda se hizo en el propio pre-registro y no en `erratas.md` (sin corridas detrás, consta en `8daf967`).

## Decisión

`CAP` = 10 para las reglas de proyecto, que es el valor ya desplegado en `exo rules` (engine `v0.10.0`): no hay que cambiar código. Para las globales, la siembra arranca con 5 reglas redactadas como mecánicas, con las más caras de incumplir arriba. Antes de fijar ese tamaño se corre la mini-tanda de g0-33 con 3, 5 y 10 globales (`evals/techo-reglas-3b/`).
