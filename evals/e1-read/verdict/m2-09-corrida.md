# M2-09 — corrida final de E1 read (2026-08-17)

Corrida de las tres patas del gate pre-registrado (`evals/e1-read/gate.md`).
**Régimen**: fase de cierre — config `.superpowers/fabrica/config.md`
§ACTUALIZACIÓN 2026-08-17: este gate es **INFORMATIVO, no bloqueante**. Los
números se reportan tal cual salieron, sin renegociar el texto pre-registrado
y sin usar el régimen ligero para maquillar nada.

Ejecutada por el orquestador (el executor de m2-08 quedó idle tras su Tarea 2).

## Condiciones de la corrida

- Ambos arms corridos **el mismo día**, 2026-08-17, entre las 22:35 y las 22:49.
- Índice del engine: `kb-completa.db`, 138 notas / 3018 trozos / 526 aristas.
- Índice de basic-memory: vivo, `updated_at` máximo 21:36 del mismo día.
- Config de bm restaurada tras el replay (`semantic_min_similarity=0.35`,
  confirmado por el propio script).
- Engine con la config sellada de M2-07: `bonus=0.0`, `β=0.6`,
  `--min-similitud 0.40` explícito (no hardcodeado, D-f3).
- **Drift de corpus anotado**: el índice del engine se construyó con
  `kb-demo` en `7ef4fba` y el gold de bm se selló con la KB ya en
  `83333bd`. El conjunto de permalinks es idéntico (pata 1 = ∅), así que el
  drift no movió el corpus; se declara igualmente porque el gate exige mismo
  estado y esto es lo que hubo.

## Pata 1 — Paridad de corpus: **PASA**

```
sellado: 138 entidades, archive=54, dotdirs_dentro=0, head=83333bd791e7
gold=138 engine=138 faltan=0 sobran=0   (exit 0)
```

Diff de permalinks a nivel entidad = **∅**, cero tolerancia, como exige el
texto. Exclusiones verificadas: dotdirs fuera (0 dentro), `archive/` dentro
(54), 5 entidades no-markdown fuera (143 entidades en bm − 5 = 138), 0
permalinks regenerados.

**Cambio declarado en el harness**: `corpus-parity.py` traía
`REF_ENTIDADES=117±12`, umbral del corpus de julio, que hoy habría parado el
sellado con 138 notas. Se subió a `138±14` con el motivo escrito en el propio
fichero. Ese umbral **no es el gate** (es un guard contra sellar el gold de un
índice a medias); el gate es la paridad = ∅, que se cumple.

## Pata 2 — Retrieval pareado: **MIXTO** (pasa una mitad del criterio, falla la otra)

| Arm (mismo día) | hit@5 |
|---|---|
| **engine-hybrid** | **48/55** |
| bm-hybrid | 39/55 |
| bm-vector | 40/55 |
| bm-text | 39/55 |
| engine-vector (testigo) | 44/55 |
| engine-fts (testigo) | 25/55 |

**Pareada engine-hybrid vs bm-hybrid: ARREGLA 13 · ROMPE 4.**

Contra el texto literal del gate ("rompe ≤2 y arregla ≥ las que rompe"):

- "arregla ≥ las que rompe": **se cumple con holgura** (13 ≥ 4).
- "rompe ≤ 2": **NO se cumple** (4). Se dice claro, no se maquilla.

Sanity-check de ingeniería del gate.md ("engine-hybrid < 46/55 señalaría
fusión mal calibrada"): 48 ≥ 46, **no se dispara**.

### Atribución de las 4 roturas (obligatoria por el gate)

Con los arms fts y vector del engine del mismo día como testigos:

| Query (recortada) | fts | vector | clase |
|---|---|---|---|
| `fabrica campaign harness config gate merge asíncrono roadmap` | miss | **HIT** | **fusion-miss** |
| `fabrica roadmap campana lighthouses diversidad bots Fase 3` | miss | miss | both-miss |
| `Frente 9 lighthouses Fase 4 divergencia core split thin-core` | miss | miss | both-miss |
| `esa utilidad de terminal … resumen estructural barato de mis notas` | miss | miss | both-miss |

Lectura honesta: **solo 1 de las 4 es atribuible a la fusión**. Las otras 3 son
both-miss — el engine no recupera esa nota por ninguna vía, así que no son un
defecto de calibración sino de recuperabilidad del contenido. La única
accionable desde la fusión es la primera (el vector la encuentra y la fusión
la pierde), que es exactamente el patrón `fusion-miss` ya conocido y anotado
en M2-07.

### Comparación con el diagnóstico de M2-07

M2-07 midió 49/55 con ARREGLA 7 / ROMPE 1 contra un bm de 43/55. Hoy: engine
48/55, bm 39/55. El engine se mantiene; **el que ha bajado es basic-memory**
(43 → 39), con el corpus crecido de 115 a 138 notas. Eso explica que suba
tanto el ARREGLA (13) y que el ROMPE también suba: con más notas nuevas hay
más queries donde los dos motores divergen.

## Pata 3 — Recall demostrado: **PASA**

(a) `exo recall --json` valida contra el contrato del envelope: una sola línea,
`schema_version: 1`, `command: "recall"`, `data` con
`{modo, query, cap_bytes, truncado, notas[]}`, `score`/`snippet` a `null` en
modo arranque, nada humano en stdout.

(b) Latencia (hyperfine, 20 corridas, índice real):

| Caso | p95 | presupuesto |
|---|---|---|
| Arranque FTS-only | **14,0 ms** | <100 ms ✅ |
| Consulta hybrid en frío | **1032 ms** | <2,0 s ✅ (bm hoy: 4,4 s mediana) |

## Veredicto de la corrida

- **Pata 1: PASA.** Pata 3: **PASA**.
- **Pata 2: pasa el criterio de balance (13 arregladas frente a 4 rotas) y
  falla el tope literal de roturas (4 > 2).**

Bajo el régimen pre-registrado original, esa pata 2 habría ido a adjudicación
de consultor fable con el texto delante. Bajo el régimen de cierre vigente el
gate es informativo, así que **la corrida no bloquea el merge de la rama**; la
decisión de merge la adjudica igualmente el consultor, con estos números y sin
que nadie los haya retocado.

Residuo abierto, sin bloquear: la query `fabrica campaign harness…` es un
fusion-miss real (el vector la encuentra, la fusión la pierde). Candidata a
mirar si algún día se retoma la calibración; no es trabajo de esta campaña.
