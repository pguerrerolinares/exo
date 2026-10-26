# Rúbrica de selección de reglas duras con la sección llena (#207, parte 3)

**Fecha:** 2026-10-26 · **Estado:** aprobada en brainstorming; revisada por un consultor (Fable, dos rondas), pendiente de review del dueño
**Origen:** issue #207 (cobertura de la siembra), PR #208 (auditoría `exo rules --coverage`), nota del consultor en §9

## 1. Qué se quiere

`Ap` (#206) dejó dicho que lo sembrado se aplica y que lo no sembrado vuelve al
default (2/40). La auditoría de #208 muestra **8 notas de proyecto en `CAP` 10**
(agent-solve-it, bizkaia-now, cge, exo, kbx, lighthouses-bot, pguerrero-music,
pguerrero.me). En ellas, la selección ya recorta: 131 candidatas del pool de M
para 80 huecos, 51 fuera.

El dueño quiere un **criterio durable** que `/document` y `/distill` apliquen
cada vez que una sección `## Reglas duras` esté llena: qué regla entra, en qué
posición y cuál sale. Hoy `routing.md:25` dice «se inserta en su posición de
prioridad» y esa prioridad no está definida en ningún sitio. La misma rúbrica
recoloca **una vez** las 131 candidatas de las 8 notas.

No se mide regla a regla en producción: la rúbrica tiene que poder responderse
en segundos con la regla, la nota y la sesión.

## 2. Evidencia de base

| dato | valor | fuente |
|---|---|---|
| Candidatas en las 8 notas | 131: 17 `cumple`, 19 `no_cumple`, 95 sin medir | `scripts/siembra-reglas.py` + `~/.cache/exo-campana-m/corridas-criba` |
| De las 51 fuera | 15 `cumple`, 3 `no_cumple`, 33 sin medir (31 nunca pasaron el veto) | ídem |
| Reglas con criba A0 (todo el pool) | 125: 68 `no_cumple`, 57 `cumple` | `corridas-criba/*/a0-r*/check.rc` |
| Reglas de proyecto con criba y nota canónica | 53 (29 `no_cumple`): población de la siembra (§5.2 de la spec del 10-25) sin los proyectos con `REPO=None` (CV, viz-components, portfolio-game; 58 con ellos) | recalculado por el orquestador y por el consultor de review |
| Baseline §5.2 (`git`/`codigo` → se queda) sobre esas 53 | kappa 0,109; precisión de «se queda» 7/10 | ídem |
| Cita de origen de la regla presente en la prosa de su nota, fuera de `## Reglas duras` | 41 de 53 (`cumple` 20/24, `no_cumple` 21/29) | ídem |
| Ancho medio del IC95 de Δkappa (bootstrap pareado, n = 53) | ≈ 0,65 | simulación del orquestador y del consultor |
| Gates que deniegan y cubren una regla de las 131 | 2 con claridad (ambas de exo, ya fuera), ~4 a medias | consultor, leyendo hooks y CI |

«A0 cumple» = todas las réplicas a0 de su tarea dan `check.rc` 0.

**Lo que tumbó la primera propuesta.** «Buena práctica genérica sale,
convención del proyecto se queda» no separa los grupos medidos: hay
convenciones arbitrarias en `cumple` (un flag concreto de una herramienta,
una prohibición de framework en un repo, una restricción de arquitectura de una
versión) y buenas prácticas genéricas en `no_cumple` (un test de reproducción
antes de usar resultados, un barrido antes de atribuir mérito, una precaución
de shell). Además pide al modelo predecir su propia conducta, y la
literatura lo encuentra mal calibrado y sobreconfiado en tareas agénticas
(§9; el consultor solo leyó los abstracts).

## 3. Enfoques

| enfoque | trade-off | decisión |
|---|---|---|
| A. «¿Lo haría igual sin la regla?» + coste + gates + frecuencia | autopredicción; no separa los datos; gates afecta a ≤ 5 %; frecuencia no es respondible | descartado |
| B. Convención/arbitraria + coste | hereda el fallo de A | descartado |
| C. Sonda de una llamada «¿qué harías?» sin la regla | mide lo que dice, no lo que hace; fuga de la regla | descartado |
| **D. Choque con el default por señales observables + coste**, con el dato de criba mandando | (b) sigue siendo juicio; se valida contra la criba antes del re-corte | **elegido por el dueño** |
| Solo dato + coste | lo más defendible, pero 95 de 131 no tienen dato | queda como fallback (§6) |

Fuera de alcance por decisión: subir el `CAP`; gates y frecuencia como
criterios; metadatos por regla (cada byte va al system prompt); iniciación
frente a restricción como etiqueta (#193, n=5); un Ap2.

## 4. La rúbrica

La posición en la sección es la prioridad: la primera manda y, con la sección
llena, sale la última. Orden lexicográfico:

1. **Veto por violación.** Una regla que en esta sesión un agente o el dueño
   violó (sea nueva o ya presente) no sale y se coloca por encima de las que no
   tienen veto. No hay contador: el veto vale para la sesión que escribe.
2. **Choque con el default**, en este orden:
   - `P` **paso extra**: pide un paso que nadie daría por su cuenta (regenerar,
     barrer un budget, un test de reproducción, redirigir stdio);
   - `D` **prohíbe el default**: prohíbe la opción atractiva (la lib, el flag,
     el type-checker, el PATH o la estética estándar);
   - `E` **el repo lo encarna**: el repo ya lo hace visible (código, config,
     `.gitignore`, Makefile, gold en `docs/`) y la tarea típica no empuja en
     contra. Va al final, porque A0 lo lee del repo.

   Si la regla tiene criba A0 medida, el dato sustituye al juicio: `no_cumple`
   cuenta como `P`/`D` (se conserva la etiqueta de juicio si la hay; si no, `D`)
   y `cumple` cuenta como `E`.
3. **Coste del error**: `alto` (irreversible, dinero, producción o fallo
   silencioso) > `medio` > `bajo` (cosmético, o lo detecta un test en seguida).
4. **Orden de fichero** (id del pool en el re-corte; orden actual en la sección
   en producción). Sustituye a las claves de §5.2 de la spec del 10-25 (check,
   ámbito, tier): la criba ya entra por el criterio 2 y el resto no se usa.

Una regla `P` de coste bajo va por encima de una `D` de coste alto: el choque
pesa más que el coste. La primacía sale gratis, porque lo frágil (el paso
extra, que #193 apunta como lo que más se diluye) queda arriba.

**La regla desplazada** no se pierde: sigue en la prosa de la nota.
`/document` deja en la bitácora del proyecto una línea «salió `<regla>` por
`<criterio>`».

## 5. Componentes

### 5.1 Doctrina

- `plugins/exo/skills/document/routing.md:25`: sustituye «su posición de
  prioridad» por la rúbrica de §4, en ≤ 8 líneas, con la línea de bitácora de la
  desplazada.
- `plugins/exo/skills/distill/consolidacion.md`: una línea. Al podar o reordenar
  `## Reglas duras` se aplica la misma rúbrica.
- `plugins/exo/skills/distill/chequeos.md` (fila `al_cap` de #208): remite a la
  rúbrica en vez de «la elección es del dueño».
- Sube `version` del plugin (minor).
- El veto lo detecta el agente que escribe, con lo que vio en la transcripción
  de la sesión: una violación observada (un agente o el dueño hizo lo
  contrario). No hay umbral ni contador, y no se testea: es doctrina.

### 5.2 Etiquetador

Un único prompt sellado, `evals/rubrica-reglas/etiquetador.prompt.md`, que
recibe **solo** el texto de la regla y la nota canónica de su proyecto (sin
criba, sin ids de sondas, sin la sesión) y devuelve una línea JSON:

```
{"id": "...", "choque": "P|D|E", "coste": "alto|medio|bajo", "justificacion": "...", "ruta": "<fichero o null>"}
```

- **Qué nota ve (decisión del dueño: forma débil).** La nota canónica del repo
  (la de la tabla `REPO` de `siembra-reglas.py`), leída con
  `git -C <kb> show <commit>:<ruta>` desde un **commit fijo de la KB**, con dos
  recortes:
  - la sección `## Reglas duras` entera;
  - el fragmento de prosa que contiene la `cita` de origen de la regla, del
    campo `cita` del pool: la frase o línea que la contiene. Hace falta porque
    la cita sigue en la prosa en 41 de 53 casos (§2).

  El veredicto cuenta en cuántas reglas se recortó la cita y en cuántas no se
  encontró. Sin acceso al repo de código, `E` se juzga por cómo la nota formula
  lo que el repo hace, no por el repo. `ruta` es **opcional** (el fichero que la
  nota cita, o `null`). Si fuera obligatoria para `E`, el etiquetador se
  refugiaría en `P`/`D` y kappa caería por un artefacto.
- **Invocación.** Igual que las otras evals del repo: `claude -p` con
  `--model claude-sonnet-5-5 --setting-sources "" --strict-mcp-config
  --no-session-persistence --output-format json --max-budget-usd <tope>` y
  stdin `</dev/null`, sin herramientas. Así no entran hooks, CLAUDE.md ni el
  perfil del usuario. La temperatura no la expone el CLI, y el veredicto lo dice.
- **Sello**, en `evals/rubrica-reglas/sello.json` antes de la primera
  corrida: hash del prompt, commit de la KB, versión de `claude` y la lista de
  ids del universo. Cambiar cualquiera de estos datos invalida la validación.

Es el mismo artefacto para la validación (§5.3) y el re-corte (§5.4), así que
lo validado es lo que se usa.

**Privacidad: el repo de exo es público.** Al repo solo van ids, etiquetas
(`choque`, `coste`), el sello y el veredicto agregado. El texto de las reglas,
las justificaciones, `ruta` (rutas de repos privados) y las notas se quedan en
`~/.cache/exo-rubrica-reglas/`, como el pool de M. La spec, el plan y el
veredicto no citan texto de reglas: solo ids o descripciones genéricas.

### 5.3 Validación pre-registrada

- Universo: las 53 reglas de proyecto con criba y nota canónica (población de
  la siembra: `generica=false`, `ambito` ∈ {codigo, git, dominio-proyecto},
  proyecto ≠ transversal, proyecto con repo en la tabla `REPO` (no `None`), con
  `a0-r*/check.rc`). Se deriva reutilizando el filtro de `propuesta()` de
  `siembra-reglas.py`, no con un cálculo aparte. Lista sellada en
  `evals/rubrica-reglas/universo.txt` (un id por línea, ordenada) antes de
  correr. El script imprime el tamaño y no lo compara con 53: el número que
  adjudica es el sellado.
- `sin_etiqueta`: se excluye de los dos brazos (el análisis es pareado) y se
  cuenta. Si hay más de 3 (≈ 5 %), la validación es nula y se revisa el prompt.
- Kappa degenerado en un remuestreo (una sola clase en la verdad o en la
  predicción, pe = 1): ese remuestreo se descarta y se cuenta. Si se descarta
  más del 1 %, se dice en el veredicto.
- Predicción de (b): `P`/`D` → «se queda»; `E` → «sale». Verdad: `no_cumple`
  → se queda; `cumple` → sale.
- Baseline: §5.2 de la spec del 10-25, reducido a lo que no usa la criba
  (`git`/`codigo` → se queda; `dominio-proyecto` → sale).
- **Regla de decisión (sellada):** si el **kappa puntual de (b) es ≤ 0** (el
  etiquetador no supera el azar), la rúbrica pierde el criterio 2 de juicio y
  queda «veto + dato + coste + fichero». Si no, D queda. Con n = 53, un
  etiquetador de puro ruido la dispara ≈ 57 % de las veces y uno de kappa 0,3
  nunca (simulación del consultor de review).
- **Por qué no el Δ pareado como regla.** La regla anterior («D cae si el
  límite superior del IC95 de Δkappa < 0») era decorativa: con n = 53 el IC
  mide ≈ 0,65 de ancho y se dispara ≈ 5 % con un etiquetador de ruido (10 de
  200 simulaciones del orquestador). Decisión del dueño: sustituirla.
- Descriptivos, sin peso en la decisión:
  - Δkappa = kappa(b) − kappa(baseline), con bootstrap pareado (10.000
    remuestreos de reglas, semilla fija) e IC95 por percentiles;
  - kappa y precisión de cada uno;
  - matriz de confusión;
  - proporción de `no_cumple` por tipo de choque (kappa por grupo no sirve:
    dentro de un grupo la predicción es de una sola clase);
  - recortes de cita hechos y no encontrados.
- Coste esperado: < 1 USD. Salida: `evals/rubrica-reglas/verdict.md` +
  `resultado.json`.

### 5.4 Re-corte de las 8 notas

1. Se etiquetan las 131 candidatas. Las del universo de §5.3 **reutilizan su
   etiqueta de la validación** y no se vuelven a etiquetar; las demás, con el
   mismo prompt sellado. Las etiquetas van a `evals/rubrica-reglas/etiquetas.jsonl`
   (ids y etiquetas); textos y justificaciones, a la caché (§5.2).
2. **Líneas de la sección que no están en el pool** (añadidas a mano después de
   la siembra): se etiquetan igual, con id `kb:<slug-nota>:<n>` y sin criba.
3. **Control de calidad:** una segunda llamada independiente con el mismo prompt
   sobre 20 reglas elegidas al azar entre las 131 (semilla fija). Si discrepa en
   `choque` o en `coste` en más de 6 de las 20 (> 30 %), el lote se devuelve y
   se revisa el prompt. Un prompt nuevo es un sello nuevo, lo que obliga a
   repetir §5.3.
   Orden de ejecución: sellar → validación (§5.3) → etiquetar el resto →
   control → revisión del dueño.
   Tope de coste: el re-corte son ≈ 170 llamadas (≈ 3× la validación). El
   script lleva `--max-budget-usd` por llamada y un tope total de 5 USD, y se
   para si lo alcanza (§6).
4. `scripts/siembra-reglas.py --etiquetas <fichero>` ordena con la rúbrica de §4
   (dato de criba > juicio; en el re-corte no hay veto) y propone las 10
   primeras por nota.
5. El dueño revisa las **etiquetas**, no el corte. Las que corrija se apuntan en
   el JSONL con `"corregida_por": "dueño"`.
6. Se reescriben las 8 secciones en la KB, en el orden resultante, con commit
   scoped. El texto es la línea que ya existe en la sección si la regla ya
   estaba; si no, el `regla` del pool. Una nota que pierda el aire de su
   presupuesto se parte antes (la doctrina de la KB manda; no se sube el
   techo).

Las 3 reglas `no_cumple` que hoy están fuera entrarán por el dato. Coinciden con
sondas de M: **M queda quemado como test de la selección**. Las ~28 `no_cumple`
con tarea y check que no son sondas se reservan sin usar, como material para
un Ap2.

## 6. Error handling

- Etiqueta inválida (JSON roto o valor fuera de dominio): se
  reintenta una vez y, si vuelve a fallar, la regla queda `sin_etiqueta` y en el
  orden cae detrás de todas las etiquetadas. Se cuentan en el veredicto.
- `claude -p` falla o se agota el presupuesto: el script se para sin escribir
  resultados parciales como si fueran completos.
- En producción, si `/document` no puede responder el choque (regla ambigua),
  la regla nueva se inserta por coste y orden de fichero, y se dice en la
  bitácora.

## 7. Testing

- El código nuevo vive en `evals/rubrica-reglas/` (etiquetador, validación,
  kappa y bootstrap), con sus tests en `evals/rubrica-reglas/test_*.py`
  (`unittest`, como `evals/ap/test_evaluar_ap.py`). La función de orden se
  extrae a un módulo que importa `siembra-reglas.py`, y sus tests van en la
  misma carpeta. Se añade un paso de CI que corre
  `python3 -m unittest discover -s evals/rubrica-reglas -p "test_*.py"`, como el
  de `evals/retrieval-heldout`.
- Orden, con datos sintéticos:
  - El veto gana a todo.
  - `P` de coste bajo va por encima de `D` de coste alto.
  - El dato de criba sustituye la etiqueta: `cumple` con etiqueta `P` → `E`.
  - Las `sin_etiqueta` van al final.
  - Empate total → orden de id.
  - Falla si el orden depende del orden de entrada del JSONL.
- Kappa y regla de decisión: kappa conocido con datos sintéticos (predicción
  perfecta → 1; constante → degenerado, sin dividir por cero); kappa ≤ 0 →
  «pierde el criterio 2». Bootstrap descriptivo: dos predictores idénticos → IC
  que contiene 0. Falla si la semilla fija no reproduce el IC al bit.
- Recorte de la nota: quita `## Reglas duras` y el fragmento con la cita, y
  cuenta las citas no encontradas.
- El etiquetador no se testea con un modelo en CI: se valida en §5.3.

## 8. Condición que invalidaría la recomendación

La regla de decisión de §5.3 (kappa puntual de (b) ≤ 0). Además,
el control de §5.4.2 (> 30 % de discrepancia) invalida el lote de etiquetas,
no la rúbrica.

**Lo que esta spec no puede saber:** la criba `cumple` puede deberse a que la
tarea no empujaba en contra, no a que el repo lo encarne. Ese sesgo afecta
igual a §5.2 y a (b), y sin tareas nuevas no se separa. Que el baseline sobre
53 reglas generalice a las transversales: [sin dato]; aquí no se usa con
ellas. Que `--setting-sources ""` impida cargar el CLAUDE.md del cwd: [sin
dato]. El plan lo comprueba con una llamada de humo antes de sellar.

## 9. Rastro del consultor

Nota completa (dos rondas) en el scratchpad de la sesión, `consultor-rubrica.md`
(efímera; lo que importa está en §2–§5).

- Ronda 1: tumbó el criterio «genérica/convención» con los datos y propuso D.
  Literatura citada, toda leída solo en abstract: Binder et al. 2024 «Looking
  Inward» (arXiv 2410.13787); Nottingham et al. 2026 «Open Problems in LLM
  Self-Prediction»; Betley et al. 2025 (arXiv 2501.11120); Jaroslawicz et al.
  2025 IFScale (arXiv 2507.11538, sesgo hacia las primeras instrucciones);
  Geng et al. 2025 «Control Illusion» (arXiv 2502.15851). Sobre la selección de
  instrucciones con un límite de contexto no encontró nada: [sin dato].
- Ronda 2, con las dudas del orquestador:
  - la señal «violación en sesión» no discrimina entre la regla nueva y las
    presentes, y queda como veto;
  - retiró sus umbrales 0,30/0,70, que igualaban al baseline, y propuso el Δ
    pareado;
  - dio un orden único;
  - propuso el re-corte auditable por etiquetas.

  La regla «inconcluso no invalida» es decisión del dueño, no del consultor.
- Review adversarial de la spec (consultor fresco, Fable), con nota en
  `scratchpad/consultor-review-spec.md`, también efímera. Pidió cuatro cambios
  y los cuatro entraron:
  - universo de 53 y no de 58;
  - regla de decisión «kappa(b) ≤ 0», con el Δ solo como descriptivo;
  - recorte también de la cita en la prosa;
  - `ruta` opcional y en la caché, y tope de coste en el re-corte.

  El orquestador recalculó el universo, la fuga 41/53 y el ancho del IC.
  Verificó también que los flags de `claude -p` 2.1.310 existen y que no hay
  temperatura.

## 10. Decisiones del orquestador (no acordadas explícitamente)

- El etiquetador corre con `claude -p` y no como subagente de la sesión, porque
  se puede reproducir y sellar.
- El universo de validación es la población de la siembra con nota canónica
  (53) y no las 125 con criba: las transversales y los proyectos con
  `REPO=None` no tienen nota de proyecto que pasarle al etiquetador.
- El tope de coste del re-corte es de 5 USD.
- El baseline es §5.2 reducido a lo que no usa la criba (`git`/`codigo` → se
  queda).
- 10.000 remuestreos y una semilla fija.
- Los ficheros viven en `evals/rubrica-reglas/`.
- Respuestas al builder check (12 preguntas). Todas son mías salvo la de qué
  nota ve el etiquetador, que la decidió el dueño:
  - invocación de `claude -p` y sello ampliado (§5.2);
  - privacidad por repo público (§5.2);
  - derivación del universo, `sin_etiqueta` y kappa degenerado (§5.3);
  - reutilizar las etiquetas de la validación, líneas fuera del pool,
    muestreo y orden del control (§5.4);
  - el orden de fichero sustituye a las claves de §5.2 (§4);
  - el veto lo detecta el agente por la transcripción, sin test (§5.1);
  - ubicación de tests y CI (§7).

## 11. Entregas

1. **PR en exo**:
   - doctrina (§5.1, plugin minor);
   - `evals/rubrica-reglas/` con sus tests;
   - `--etiquetas` en `siembra-reglas.py`;
   - el sello.

   La validación se corre después del merge del sello y su veredicto va en un
   PR aparte, como `Ap`.
2. **Re-corte**:
   - etiquetas y control en el PR del veredicto;
   - las 8 notas en un commit aparte en la KB (otro repo), tras la revisión
     del dueño.
