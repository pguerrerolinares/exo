# Review de la spec de M6-06 — recall en el punto de uso (2026-08-22)

> Consultor Fable fresco, segundo par de ojos del régimen de gates. Orden de
> lectura respetado: spec a solas → verificación primaria contra código, binario
> (`~/.local/bin/exo`, build 2026-08-22 09:43), índice vivo copiado a scratchpad
> y docs oficiales de hooks (code.claude.com/docs/en/hooks, consultadas hoy) →
> solo al final el verdict del otro consultor y su brief. Cero escrituras fuera
> de scratchpad y de este fichero.

## 1. Verdict global

**Lista para plan con 2 arreglos obligatorios**, ambos baratos (texto en la
spec, ninguno toca el engine ni reabre adjudicaciones). El diseño se sostiene:
los claims que lo soportan verifican con medición propia, la fidelidad al
verdict es alta (ver §5), y los agujeros restantes son de plan, no de diseño.

## 2. Bloqueantes

### B1 — La lista de stopwords validada no existe en ningún sitio durable (§2.1 regla 3, §6)

La spec declara la lista cerrada como "el nuevo punto de mantenimiento" pero no
la enumera, no dice dónde vive, y la describe mal dos veces:

- **"~50 entradas"**: la lista realmente validada (`gates.py`, scratchpad del
  consultor — directorio volátil de sesión) tiene **127 entradas únicas**. El
  "~50" viene arrastrado del verdict (Apéndice B) y no describe el artefacto
  que produjo los números de la spec.
- **"sin puntuación"**: la normalización validada NO quita toda la puntuación —
  conserva `/`, `.` y `-` (`re.sub(r'[^a-z0-9/.-]','',token)` tras NFD +
  minúsculas). Un ejecutor que implemente "sin puntuación" literal construye un
  gate distinto del medido.

Por qué bloquea y no es "decisión de plan": **todos los números del gate son
propiedades de esa lista exacta** — 86% de disparo, FN topical = 0, "los 39 que
salta son todos ack/git puro". El propio test 1 de la spec ("pushea los dos
repos" calla) solo pasa porque `repos`, `dos` y `pushea` están en esa lista
concreta. Si el plan escribe una lista nueva de memoria, hereda los claims sin
heredar el artefacto que los hizo ciertos, y no puede re-medirlos (§0 retiró la
maquinaria). Arreglo: copiar lista + función de normalización de `gates.py` a
un apéndice de `consultas/2026-08-22-m6-06/` (o a la spec) y corregir "~50" y
"sin puntuación". Diez minutos.

### B2 — "exit 1 → empty" enmascara fallos reales como abstención (§3, P2, P3)

Verificado en `main.rs:236-249`: el engine sale con **exit 1 para TODO error**
(`process::exit(1)` del `main`; solo el Rechazo de write usa 3, clap usa 2,
timeout 124). "Recall vacío" es un `bail!` más (`main.rs:441-445`): mismo
código que DB corrupta, config RO ausente, modelo ONNX roto o un `--refresca`
que falla por lock. El data flow de la spec (`exit 1 "recall vacío" → empty`,
`otro → error`) sugiere que existe un "otro" para errores del engine — **no
existe**. Un ejecutor que gatee solo por código implementa un hook donde el
engine roto loguea `empty` para siempre: exactamente la degradación silenciosa
con forma de abstención que P3 jura impedir, y el escenario "el hook está roto
y no dispara nunca" que §2.5 dice querer distinguir con un grep.

El distinguidor existe y es estable: stderr trae `error: recall vacío (modo
consulta): …` en la abstención (medido con caso real: "sella la KB y pushea" →
exit 1, ese stderr exacto) y otra cosa en cualquier fallo. Arreglo: P2 debe
decir "exit 1 **y** stderr contiene `recall vacío` → `empty`; exit 1 sin esa
marca → `error`". Una línea de spec, un grep en el hook, y el test 3 debería
cubrir ambas ramas (exit 1 con marca → empty; exit 1 sin marca → error).

## 3. No bloqueantes (el plan lo resuelve o se corrige al pasar)

1. **Claim rancio en §2.2/P4: "indexer aún no transaccional, fix H1 pendiente".
   Falso desde el 18-ago**: `db5e0ae` ("fix(m6): transacción por nota en el
   indexer") es ancestro de HEAD y el binario instalado es del 22-ago. P5 se
   mantiene igual — el bootstrap de minutos bajo timeout de 30 s basta solo —
   pero la justificación debe corregirse para no propagar un hecho falso a la
   siguiente spec que lo herede. (El verdict H6 carga el mismo error.)
2. **`--refresca` convierte el hook en escritor en ~86% de los turnos.** Con
   sesiones paralelas (esta misma sesión tiene 3 agentes) más el indexer de
   `Stop`, habrá contención: `busy_timeout` de 5 s en el engine y journal
   `delete` (un lector bloquea al escritor, `lib.rs:55-64`). Peor caso benigno:
   el turno paga los 5 s del timeout-guard o un error de lock — con B2
   arreglado queda visible como `error`/`timeout-guard`, no como `empty`. El
   plan puede nombrar el caso y no hacer nada más.
3. **Prompt gigante por argv**: medido — 8 KB de traceback → 1,39-1,44 s con
   hits genéricos pero ≤1 KB; 30 KB → idéntico resultado y coste (el embedder
   trunca; plateau muy por debajo del timeout de 5 s). Único borde real: un
   argumento >128 KB (MAX_ARG_STRLEN, verificado E2BIG con 140 KB) revienta el
   exec del engine → con B2, `reason=error` y exit 0. Benigno; truncar la query
   a N KB en el hook es opcional y YAGNI hoy.
4. **stdout plano en exit 0 se inyecta como contexto** (docs oficiales:
   UserPromptSubmit es excepción junto a SessionStart). Cualquier fuga de
   stdout del script (echo de debug, stderr mal redirigido) se convierte en
   contexto del turno. El test 7 (stdout = JSON válido) lo cubre de refilón;
   vale una frase en el plan para el ejecutor.
5. **Crecimiento del log**: hoy 657 KB / 1583 eventos en ~2 meses; con ~86% de
   turnos emitiendo (payload capado a 500 B por `_reflex-log.sh`), ~1-2 MB/mes.
   Sin rotación, y `exo-recall.sh` lo escanea entero con `jq` en cada compact —
   a años vista sigue siendo sub-segundo. No urge; si algún día molesta, es un
   `logrotate` de una línea.
6. **Filtro de core-index post-`--limite 3`**: si core-index entra en top-3
   quedan 2 punteros sin backfill. Medido 0/90 en top-3: irrelevante. El plan
   decide (pedir `--limite 4` y cortar a 3, o no hacer nada).
7. **Detalles de plan legítimos**: forma exacta de la entrada en `hooks.json`,
   defaults de seams (`EXO_BIN`/`EXO_INDEX`, molde en `exo-recall.sh`),
   `--json`+jq vs sustitución de primera línea (verifiqué que la cabecera del
   modo texto es exactamente una línea — ambas vías valen), tokenización por
   whitespace. Ninguno exige inventar diseño.

## 4. Claims verificados (medición primaria, scripts en scratchpad)

| Claim | Resultado | Cómo |
|---|---|---|
| Sin `--min-similitud` cae al 0.35 de config | **Confirmado** | `semantic_min_similarity: 0.35` en config RO; "sella la KB y pushea" con 0.40 → exit 1; sin flag → 5 hits a 0,385-0,392. Código: `min_similitud_efectivo` → `min_similitud_de_config()` (`buscador.rs:229-235`, `lib.rs:150`) |
| Exit 1 sin hits, indistinguible por código de fallo real | **Confirmado y peor de lo que la spec asume** (→ B2) | exit=1 con stderr `error: recall vacío (modo consulta)…` medido; `main.rs` sale 1 para cualquier error anyhow |
| Exit 2 borra el prompt; timeout default del evento | **Confirmado** | Docs oficiales: "Blocks prompt processing **and erases the prompt**"; default `command` 600 s **bajado a 30 s** para UserPromptSubmit (el 30 de la spec es correcto). Bonus: stdout plano en exit 0 se inyecta como contexto |
| Coste de `--refresca` en proceso hybrid | **Confirmado** | 0,92-0,96 s con y sin `--refresca` sobre copia de DB (Δ dentro del ruido, consistente con +0-80 ms). DB ausente sin flag: `error: DB no encontrada`, exit 1. Con flag: bootstrap confirmado en código (`lib.rs:115`, "si la DB no existe, la construye") — no lo ejecuté (minutos) |
| Forma del modo texto (§2.4) | **Confirmado** | Cabecera = exactamente 1 línea (`=== Recall exo (PARCIAL — no sustituye tu brief) ===`), cuerpo `- ruta — título` + `  · snippet`. Sustituir la primera línea basta |
| Muestreo de §1 | **Confirmado** | FTS-AND "vamos con brainstorm de M6-06" → 0 resultados; FTS "sí" → 114/145 (números exactos de la spec); hybrid corto 0,95-0,97 s ×3 |
| "indexer no transaccional" (P5) | **CAE como hecho** (no como decisión) | `db5e0ae` en HEAD + binario post-fix. Ver no-bloqueante 1 |

## 5. Fidelidad al verdict

Alta. Los números viajan bien (71→86%, FN 2-3%, 0/90, 29%, ~130 B, ~970 B,
114/145, AUC 0,49, medianas 0,018/0,018) y ninguna adjudicación se suavizó: la
spec usa el gate revisado del Apéndice B (no el D3 original), conserva el
trade-off del segundo por turno con la tasa corregida (86%, no el "~60-70%"
pre-revisión de D2), y **endurece** donde el verdict era blando: el timeout
propio de 5 s (P4) y el skip de `/`/`!` no estaban en el cuerpo del verdict y
son adiciones correctas. El `permalinks` añadido a `emitted` resuelve una
inconsistencia interna del verdict (D7 no lo listaba; D4 lo asumía) del lado
bueno. Los dos defectos heredados — "~50 entradas" y el indexer "no
transaccional" — nacen en el verdict, no en la destilación: B1 y el
no-bloqueante 1 los corrigen.

## 6. Costuras nuevas (no vistas por spec ni verdict)

1. **B2 entero**: la semántica real de exit 1 del engine (todo error = 1).
2. **La lista validada vive solo en un scratchpad volátil** (mitad de B1): el
   artefacto que soporta los números del gate no está commiteado en ningún lado.
3. **Contención de escritura por `--refresca`** en sesiones paralelas
   (no-bloqueante 2).
4. **E2BIG a >128 KB** de prompt y el plateau del embedder (no-bloqueante 3) —
   la spec no menciona prompts gigantes; resultado: benignos.
5. **stdout plano = contexto en exit 0** (no-bloqueante 4): hazard de fuga que
   los docs confirman y la spec no nombra.

## 7. Considerado y descartado

- **Pedir un mitigador del modo de fallo social de §6** (que Paul deje de mirar
  los punteros): todo lo que se me ocurrió (rotar cabeceras, marcar "hit
  fuerte/débil", contador de ignorados) o inventa señal que los datos dicen que
  no existe (scores no separan) o es métrica de eficacia disfrazada (§0). El
  diseño ya tiene lo barato: formato + licencia de ignorar + log de emitted
  para greppear uso. Nada que añadir sin violar YAGNI.
- **Truncado de query en el hook** para prompts enormes: medido, el embedder ya
  trunca y el coste hace plateau en ~1,4 s. Añadirlo sería resolver un problema
  que la medición dice que no existe.
- **Dedup por sesión, límite >3, umbral distinto**: re-verificados los datos
  que los descartan (29% solape, 0/90 core-index); las adjudicaciones D4/D6
  están bien cerradas. No se reabren.
- **Exigir rotación del log**: 1-2 MB/mes no es un problema este año.
- **Objetar el skip de prompts que empiezan por `<`** (HTML pegado a pelo):
  el FN existe en teoría, pero Paul antepone texto casi siempre y la regla
  protege de los teammate-messages (reales y grandes — esta sesión los tiene).
  Polaridad correcta, coste real ~nulo.

## Anexo — método

Scripts y salidas en el scratchpad de sesión (comparte directorio con los del
consultor original: `gates.py`, `margen.py`, etc.). DB copiada
(`index-copy.db`), KB real solo leída, `--refresca` ejecutado únicamente contra
la copia. Docs de hooks consultadas vía redirect actual
docs.claude.com → code.claude.com/docs/en/hooks.
