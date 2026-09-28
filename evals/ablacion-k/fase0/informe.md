# Campaña K — Fase 0: tasa de uso de `recall-inject` (2026-09-28)

Descriptiva: según §3 del pre-registro, **no decide nada**. Script:
`evals/ablacion-k/fase0/uso.py`, con semilla `20260923` y 10.000 réplicas de
bootstrap por sesión. Este fichero solo tiene agregados. El detalle por
evento (permalinks, sesiones) vive fuera del repo, en
`~/.cache/exo-ablacion-k/fase0/`.

Reproducir:

```
python3 evals/ablacion-k/fase0/uso.py --detalle <fichero fuera del repo>
```

## 1. Muestra

- 357 eventos `recall-inject-emitted` con `ts` anterior a la congelación
  (`f324818`, 2026-09-28T21:38:55Z).
- **301 tienen transcript** y entran en el análisis. 56 (16 %) no lo
  tienen: sesiones sin persistencia o ya purgadas. Es sesgo declarado, sin
  corregir.
- 74 sesiones. 897 permalinks inyectados, todos mapeables al índice actual.
- Índice actual por tier: core 5 · stable 63 · log 113 · sin tier 18.

## 2. Validación del instrumento (antes de leer el resultado)

La primera corrida se revisó a mano con una muestra de coincidencias. Salieron
dos fallos, corregidos antes de dar ningún número:

1. **Nombres genéricos.** Las notas de la raíz de la KB (`README.md`,
   `AGENTS.md`) y los títulos de una palabra casaban con cualquier texto. En
   la muestra, por ejemplo, un `find` en otro repo contó como «tocado».
   Arreglo: una ruta sin directorio solo cuenta en forma absoluta o como
   permalink, y un título de una sola palabra no cuenta para (c).
2. **El criterio (b) mezcla dos cosas.** La mayoría de sus coincidencias
   eran salidas de `exo search`/`recall`/`targets` lanzadas por el propio
   agente: su búsqueda devolvía la misma nota que ya se había inyectado. Eso
   es **co-ocurrencia** con la búsqueda agéntica, no uso de la inyección. Se
   reporta por separado: `b: salida de exo` y `b: otra salida`.

Los orígenes de `b: otra salida` se clasificaron sobre las 78 coincidencias
del hilo principal: `grep`, `cat`, `find`, `wc` y `python3` sobre la KB,
lecturas y ediciones. Son plausibles como uso real.

## 3. Resultado

Hilo principal de la sesión (`main`). Con subagentes (`todo`), las cifras
son casi iguales; se pueden ver con el script.

| criterio | U | U_base | lift | IC95 lift |
|---|---|---|---|---|
| **a+b+c (definición de §3)** | **0,409** | **0,179** | **+0,229** | **[+0,150, +0,312]** |
| a: abre o edita la nota | 0,060 | 0,007 | +0,053 | [+0,016, +0,098] |
| b: salida de `exo search/recall/targets` | 0,296 | 0,080 | +0,216 | [+0,135, +0,295] |
| b: otra salida (grep, cat, find…) | 0,259 | 0,123 | +0,136 | [+0,065, +0,216] |
| c: la cita en su texto | 0,030 | 0,013 | +0,017 | [−0,016, +0,050] |
| a+b+c sin las salidas de exo | 0,262 | 0,133 | +0,130 | [+0,062, +0,208] |

## 4. Lectura (descriptiva, con sus límites)

- **Lo que se inyecta no es al azar:** en torno al 41 % de las inyecciones
  acaban tocadas, frente al 18 % del control. La inyección apunta a notas
  relacionadas con lo que el agente hace después.
- **Uso directo, poco:** el agente abre o edita una nota inyectada en el
  **6 %** de los eventos y la cita en el 3 %.
- **La mayor parte del lift es redundancia con la búsqueda agéntica:** en
  el 30 % de los eventos, el propio `exo search` del agente devuelve la
  nota inyectada. Esto no prueba que la inyección sobre; sí anticipa la
  pregunta de R3 (A3 contra A2): si el agente busca por su cuenta, gran
  parte de lo inyectado lo iba a encontrar igual.
- **Lo que esta métrica no ve:** el primer párrafo inyectado puede haber
  servido sin abrir la nota («no tocado» no significa «inútil»). Y
  «tocado» no significa «útil»: la co-ocurrencia no es causalidad. Solo la
  ablación (fase 1) responde a eso.

## 5. Decisiones operativas que §3 no fijaba (declaradas)

- **Sesión:** `main` = transcript principal; `todo` = principal + subagentes.
  Se reportan los dos y la lectura usa `main`.
- **Mapeo de notas:** índice actual (`~/.exo/index.db`) y tier leído del
  frontmatter actual. Las notas renombradas o re-tiereadas desde el evento
  se clasifican con su estado de hoy.
- **Salidas persistidas aparte** (`tool-results/` de outputs grandes): no se
  leen, solo el contenido que está en el transcript. Sesgo a la baja, igual
  en U y en U_base.
- **Sensibilidad no pre-registrada:** que un comando Bash contenga la ruta de
  la nota (por ejemplo un `cat`) sube U a 0,429 y el lift a +0,249 (script).
