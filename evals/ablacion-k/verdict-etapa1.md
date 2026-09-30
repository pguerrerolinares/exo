# Campaña K — verdict de la etapa 1 (adjudicación independiente)

Adjudicador fresco, sin participación en el diseño, el gold ni las corridas.
Fecha: 2026-09-30. Todo recomputado desde los datos crudos de
`~/.cache/exo-ablacion-k/` sin usar `analisis_etapa1.py`; el fichero
`analisis-etapa1.txt` del orquestador se usó solo para contrastar al final.
Los scripts auxiliares vivieron en el scratchpad de la sesión; la lógica
esencial va en el anexo A para que sea reproducible.

Material: pre-registro `docs/superpowers/plans/2026-09-23-campana-k-preregistro-ablacion.md`
(§6–§8), `erratas.md` (E1–E9, I1), `congelacion.txt`
(`orden_etapa1_sha256 afcd41303eb78045 · corridas 240`, `modelo claude-sonnet-5-5 · claude 2.1.285`).

---

## 1. Verificación primaria

Éxito por tarea y brazo = fracción de réplicas con `check.rc == 0` (rc 1 y rc 2
cuentan como fallo, según §6 y E3). Δ emparejado por tarea; bootstrap sobre
tareas, 10.000 réplicas, `random.Random(20260923)`, percentiles empíricos;
test de signos exacto bilateral sobre las tareas discordantes.

| estrato | n | éxito A0 | éxito A3 | Δ medio | IC95 bootstrap | A3>A0 / A3<A0 | p signos | concordantes |
|---|---|---|---|---|---|---|---|---|
| S1 | 40 | 0,575 | 0,6375 | **+0,0625** | **[−0,025, +0,1625]** | 4 / 2 | 0,6875 | 34 (85 %) |
| S2 | 20 | 0,800 | 0,750 | **−0,050** | **[−0,150, +0,000]** | 0 / 1 | 1,0000 | 19 (95 %) |

Sensibilidad (sin tareas con algún rc 2: `g1-110`, `g1-147`, `g1-16`):

| estrato | n | éxito A0 | éxito A3 | Δ medio | IC95 | A3>A0 / A3<A0 | p signos |
|---|---|---|---|---|---|---|---|
| S1 sin rc2 | 37 | 0,5946 | 0,6486 | +0,0541 | [−0,0135, +0,1486] | 3 / 1 | 0,6250 |
| S2 sin rc2 | 20 | (sin cambios: S2 no tiene rc 2) | | | | | |

Robustez del IC: el mismo resultado con `numpy.random.default_rng(20260923)`
y con percentil interpolado (`[-0.025, 0.1625]` y `[-0.15, 0.0]`). Con Δ por
tarea en {−1, −½, 0, ½, 1} y n=40, la distribución bootstrap es discreta y los
extremos caen en múltiplos de 1/80; no hay sensibilidad al método de percentil.

**Contraste con el orquestador.** Coinciden todas las cifras primarias, la
sensibilidad, los rc 2 por brazo (`a0: 4, a3: 3`), las cortadas (ninguna) y los
veredictos. Única diferencia: las medianas de `tokens_in`
(orquestador 109.606 / 133.463; aquí 109.382 / 132.527). Causa:
`analisis_etapa1.py:99-100` toma `tk[len(tk)//2]` (elemento 61 de 120, la
mediana superior) en vez de promediar los dos centrales. Es una convención,
no un error de datos. Sin más discrepancias.

## 2. Resultados y reglas

Reglas pre-registradas, citadas de §6 del pre-registro (líneas 184-193):

> **R1 (K1, sobre S1).** `EXO AYUDA` si el límite inferior del IC95 de Δ(A3−A0) > 0 **y** Δ ≥ 0,10. `EFECTO PEQUEÑO O NULO` si el límite superior del IC95 < 0,10. `NO CONCLUYENTE` en cualquier otro caso. Se reporta el IC tal cual y no se amplía N a posteriori.

> **R2 (K2, sobre S2), no-inferioridad.** `SIN DAÑO` si el límite inferior del IC95 de Δ(A3−A0) > −0,10. Si no se cumple: `DAÑO`, y abre ítem Alta en el backlog sobre la abstención de `recall-inject`, con independencia de R1.

Aplicación mecánica:

- **R1 = `NO CONCLUYENTE`.** Límite inferior −0,025 ≤ 0 (no es `EXO AYUDA`); límite superior +0,1625 ≥ 0,10 (no es `EFECTO PEQUEÑO O NULO`).
- **R2 = `DAÑO`.** Límite inferior −0,150 ≤ −0,10.
- **Parada (§6):** R1 ≠ `EXO AYUDA` ⇒ la etapa 2 no se corre. Con `NO CONCLUYENTE`, «no se toca producción y se declara el techo de lo que N permite ver» (§6, línea 199).
- **R2 `DAÑO` ⇒** ítem Alta en el backlog sobre la abstención de `recall-inject` (§6, línea 192). Se reporta tal cual; el contexto va en §5.

S1 por grupo de origen (descriptivo; E4 sustituyó «fuente 1 vs 2» por los tres
lotes de extracción):

| grupo | n | A0 | A3 | Δ | IC95 | +/− |
|---|---|---|---|---|---|---|
| g0 | 16 | 0,719 | 0,750 | +0,031 | [−0,094, +0,188] | 1/1 |
| g1 | 16 | 0,500 | 0,594 | +0,094 | [−0,063, +0,281] | 2/1 |
| g2 | 8 | 0,438 | 0,500 | +0,063 | [0,000, +0,188] | 1/0 |

Secundarias (descriptivas, n=120 por brazo, desde `meta.json`):

| brazo | tokens_in mediana / media / total | tokens_out total | USD total (mediana) | turnos mediana / media / máx | seg mediana / total |
|---|---|---|---|---|---|
| A0 | 109.382 / 156.431 / 18,77 M | 351.947 | 13,04 (0,083) | 7 / 7,9 / 21 | 29,5 / 7.813 |
| A3 | 132.527 / 190.632 / 22,88 M | 404.851 | 16,87 (0,107) | 7 / 8,6 / 24 | 40,5 / 9.315 |

A3 consume un 22 % más de entrada y un 29 % más de dólares; la mediana de
turnos es la misma. Total de la etapa: 41,65 M tokens de entrada, muy por
debajo del tope de 240 M (E3); ninguna tanda se acerca a la media de 2 M por
corrida.

## 3. Integridad

- **240 corridas presentes.** Las 240 líneas de `orden-etapa1.txt` tienen su
  directorio con `meta.json`, `check.rc`, `check.log`, `fugas.json`,
  `transcript.jsonl`, `diff.patch` y, en A3, `reflex.jsonl` (lista de ausentes: vacía).
  `corridas/` tiene además `k-canario/` y `k-subagente/` (canarios de la Task 0, fuera del orden; no se cuentan).
- **Fugas.** Los 240 `fugas.json` tienen `fuga: false`, `motivos: []` y
  `avisos: []`. **Tasa de avisos: 0/120 en A0 y 0/120 en A3.** Ningún intento
  de invocar `exo` desde A0 (E2-ampliación).
- **Versión.** `claude-version.txt` = `2.1.285 (Claude Code)` en las 240;
  `system.init.model = claude-sonnet-5-5` en todos los transcripts revisados.
- **Cortes.** Los 240 eventos `result` de nivel superior son
  `subtype=success, stop_reason=end_turn, terminal_reason=completed, is_error=false`
  (A3 muestra 122 porque `g0-12/a3-r1` y `a3-r2` llevan además el `result` de su subagente).
  `meta.fin = completed` en 240/240; `status.txt` y `err.log` sin errores de
  infraestructura; `permission_denials` vacíos. Máximos: 24 turnos (< 40),
  0,63 USD (< 10), 492 s (< 1800 s de `timeout`). **Cero cortadas por brazo.**
- **Bloque de arranque de A3 constante.** El `additionalContext` del hook
  `SessionStart` tiene sha256 `01c74143…` y 5.888 B en las 120 corridas de A3
  (el bloque distinto de 5.949 B es del canario `k-canario/a3-r1`). El recall
  por prompt (`recall-inject-emitted`) salió en las 120 con `n_hits=3`, y
  devolvió el mismo trío de permalinks en las dos réplicas en 39/40 tareas de S1
  (la excepción es `g1-16`, véase abajo). La salida del recall por prompt no
  queda en el `transcript.jsonl` (stream-json no vuelca el `additionalContext`
  de `UserPromptSubmit`); solo `reflex.jsonl` registra qué notas trajo.
- **Hallazgo de integridad I2 — la KB snapshot fue escrita por un agente y no se restauró.**
  `git -C prep/kb status` muestra 3 ficheros modificados (un capítulo del
  backlog, un destilado de proyecto y su bitácora; +6/−1 líneas). Los escribió
  `g1-16/a3-r2` (Edit×2 + un `printf >>`, 2026-09-30 00:28:59–00:29:03; único
  caso de escritura en `prep/kb` entre las 120 corridas de A3). Ni `correr.sh`
  ni `tanda.sh` restauran la KB entre corridas, y el `--refresh` de
  `recall-inject.sh` reindexó los tres ficheros (`notas.mtime` en `index.db`
  coincide con las escrituras). **Alcance:** 77 corridas de A3 posteriores; el
  bloque `SessionStart` no cambió (hash constante) y el recall solo trajo una
  nota contaminada en `g0-104` (r1 y r2; ambos brazos 1,0, sin efecto) y en
  `g1-16/a3-r1`. **`g1-16/a3-r1` sí quedó afectada:** corrió a las 06:37,
  leyó el capítulo del backlog ya editado por su réplica hermana, constató que
  la KB «ya estaba al día» y por eso se volvió al `docs/backlog.md` del repo,
  que es lo que el check mide. Sin esa lectura, el camino más probable era el
  de `a3-r2` (editar la KB y no tocar el repo, rc 2). El 0,5 de A3 en `g1-16`
  está confundido por estado compartido entre réplicas. Es un fallo de
  aislamiento del harness (E2 vigila lecturas de la KB de producción en A0,
  pero nada vigila escrituras en el snapshot desde A3). Impacto sobre el
  veredicto: si `g1-16` hubiera sido 0,0 en A3, Δ(S1) pasaría a +0,050 y R1
  seguiría `NO CONCLUYENTE`; no cambia nada pre-registrado.
- Lecturas de la KB desde A3 (sin escritura): `g1-147/r2`, `g0-149/r2`,
  `g1-140/r2`, `g1-144/r2` y `g1-16/r1` (5 corridas más). El bloque de
  arranque da la ruta del snapshot, así que el agente puede ir a leerla; en A0
  esa ruta está prohibida por `--disallowedTools` (E1).

## 4. Las 7 tareas discordantes

Método: para cada tarea leí `tarea.json` y `check.sh` del gold, la regla de
origen en `pool/reglas-g*.jsonl`, los cuatro transcripts (herramientas, texto
del agente, resultados), `diff.patch`, `check.log`, y en A3 `reflex.jsonl` y
el bloque `SessionStart`. Donde el check es reproducible sobre el transcript lo
reejecuté con instrumentación en el scratchpad. Las paráfrasis de material
inyectado o de la KB son ≤ 15 palabras.

| tarea | A0 (rc r1,r2) | A3 (rc r1,r2) | Δ | clase |
|---|---|---|---|---|
| g0-115 | 0,5 (1,0) | 0,0 (1,1) | −0,5 | **(c)** artefacto del check |
| g0-12 | 0,0 (1,1) | 1,0 (0,0) | +1,0 | **(a)** efecto de la memoria |
| g1-110 | 0,0 (2,2) | 1,0 (0,0) | +1,0 | **(a) probable** (recall no verificable) |
| g1-14 | 0,0 (1,1) | 1,0 (0,0) | +1,0 | **(a)** efecto de la memoria |
| g1-16 | 1,0 (0,0) | 0,5 (0,2) | −0,5 | **(d)** distracción (+ I2) |
| g2-109 | 0,5 (0,1) | 1,0 (0,0) | +0,5 | **(a)/(b)** plausible, no separable del ruido |
| s2-wagtail-1a94e52b53a9 | 1,0 (0,0) | 0,0 (1,1) | −1,0 | **(b)+(c)** elección estocástica cazada por test sobre-específico; **no** por lo inyectado |

### g0-115 — (c) artefacto del check

Regla (paráfrasis): bundle con `bun build`, tipos con `tsc --emitDeclarationOnly`, sin otro bundler.
Las cuatro corridas hacen lo mismo: añaden scripts `build:core`/`build:umd` con
`bun build`, y ejecutan un script agregado (`bun run build:all` en tres de
ellas, `bun run prepublishOnly` en `a0-r2`) que encadena los builds y
`build:types`. Nadie toca rollup/vite/esbuild.
Reejecución instrumentada de `check.sh` sobre los transcripts (mismos rc que los reales):

```
g0-115/a0-r1: otro=0 bundle=1 tipos=0 tsc_emite=0  rc=1
g0-115/a0-r2: otro=0 bundle=1 tipos=1 tsc_emite=0  rc=0
g0-115/a3-r1: otro=0 bundle=1 tipos=0 tsc_emite=1  rc=1
g0-115/a3-r2: otro=0 bundle=1 tipos=0 tsc_emite=0  rc=1
```

El check solo acredita «tipos» si ve literalmente `bun run build:types`,
`tsc --emitDeclarationOnly` o `bun run prepublishOnly`; `bun run build:all`
(script definido por el agente que invoca `build:types`) no cuenta, aunque el
único motivo por el que `a0-r2` aprueba es que llamó al agregado con el nombre
`prepublishOnly`. Además `a3-r1` ejecutó `tsc -p /tmp/consumer` para comprobar
el subpath desde un consumidor, y el check lo clasifica como «tsc que emite»
porque busca el tsconfig bajo el workdir. Comportamiento real de los cuatro:
conforme. Es un falso suspenso en tres corridas (dos de A3, una de A0); el
resultado verdadero sería 1,0 / 1,0 (concordante). Los `check.log` están vacíos.

### g0-12 — (a) efecto de la memoria

Regla (paráfrasis): delegar investigación y lecturas voluminosas a subagentes; quedarse la conclusión.
El bloque `SessionStart` de A3 contiene la línea de doctrina que dice justo
eso (paráfrasis: «orquestador limpio: delega lecturas voluminosas a subagentes»).
A3 en ambas réplicas: primera acción `Agent` (1 llamada en el padre; los 14 y
23 `Bash` posteriores tienen `parent_tool_use_id`, es decir, son del
subagente). El primer texto de `a3-r1` reformula la doctrina casi palabra por
palabra (≤ 15 palabras: «delego el recorrido a un subagente para quedarme la
conclusión»). A0 en ambas réplicas: 11–12 `Bash` de lectura inline y ningún
`Agent`. El recall por prompt no trajo la nota fuente (permalinks de
`reflex.jsonl` sin relación); el efecto viene del bloque de arranque.
Caveat: el check acredita con una sola llamada `Agent`; mide «delegó», no
«se quedó limpio».

### g1-110 — (a) probable

Regla (paráfrasis): si el refresh del clone shallow falla, `fetch --unshallow` + `merge --ff-only`.
El bloque de arranque no la contiene (grep `unshallow`: 0). El recall trajo
`log/reflex-bitacora`, cuyo trozo 14 de 16 en `index.db` es el único con la
regla y menciona marketplace, shallow y refresh pre-push, términos del prompt;
que ese fuera el snippet inyectado es plausible pero **no verificable** (el
texto del recall no queda en el transcript y no puedo ejecutar `exo`).
Conducta: A0 ejecuta el script, ve «historias no relacionadas», y en ambas
réplicas **se para y reporta** que no quedó en el mismo commit (rc 2, clone
sin mover); `a0-r2` incluso detecta que es shallow y sugiere un `reset --hard`
sin ejecutarlo. A3 en ambas réplicas **arregla**: `a3-r2` con exactamente
`fetch --unshallow … && merge --ff-only FETCH_HEAD` (la forma literal de la
regla) y `a3-r1` con `fetch --depth 4` + `merge --ff-only` tras un intento
fallido, hablando de «mi hipótesis del --depth» antes de investigar. Que las
dos réplicas de A3 partan de la hipótesis shallow y actúen, y las dos de A0 se
queden en el diagnóstico, apunta a la memoria; con k=2 no descarto que sea
temperamento (reportar vs arreglar), y la forma `--depth 4` no es la de la regla.

### g1-14 — (a) efecto de la memoria

Regla (paráfrasis): un índice no se destila; se quitan entradas muertas y justificaciones, no se comprimen las vivas.
El bloque `SessionStart` la contiene casi literal (grep «no se destilan»: 1).
A3: `a3-r1` lo dice antes de tocar nada (paráfrasis ≤ 15 palabras: «es un
índice: quito muertas y no comprimo las vivas»), borra las 7 entradas muertas
y la cláusula de justificación; `a3-r2` hace lo mismo con dos `sed`. Cabezas
vivas intactas ⇒ rc 0. A0: `a0-r1` reescribe el INDEX y luego «recorta un poco
más» acortando descripciones; `a0-r2` reescribe el fichero entero con `Write`.
Ambas quitan también las muertas, pero comprimen vivas ⇒ rc 1. El check
compara cabezas vivas contra el commit inicial y es fiel a la regla.

### g1-16 — (d) distracción, con el confundido I2

Regla (paráfrasis): editar el frente existente del backlog; no abrir sección delta.
El bloque de arranque contiene la regla (grep «Edita el frente»: 1) **y
también** la ruta del snapshot de la KB y el nombre de la nota-puerta del
Backlog con sus capítulos. Efecto: las dos réplicas de A3 empiezan buscando el
backlog **en la KB**, no en el repo. `a3-r2` edita tres notas de la KB (el
capítulo del backlog, el destilado y un append a la bitácora), declara el
backlog «al día en tres sitios de la KB» y **nunca toca `docs/backlog.md`** ⇒
rc 2 (no evaluable) y de paso contamina el snapshot (I2). `a3-r1`, seis horas
después, lee ese capítulo ya editado, deduce que lo atrasado es el
`docs/backlog.md` del repo y lo edita bien (rc 0). A0 va directo a
`docs/backlog.md` y edita el frente en ambas réplicas (rc 0; sin cabeceras
nuevas, ítems F6 y M1 modificados). La inyección llevó al agente a otro sitio:
el sitio equivocado era, irónicamente, la propia memoria. Sin I2, el resultado
probable de A3 era 0,0.

### g2-109 — (a)/(b) mezcla

Regla (paráfrasis): antes de validar un post, regenerar los `out/` y compararlos con los commiteados.
El bloque de arranque no contiene la regla, pero nombra el proyecto y su
frente («publicar writeups»). El recall trajo la bitácora del proyecto
(179 trozos; 4 contienen «regenera», uno de ellos la regla y otro el
aprendizaje del mismo modo de fallo). No verificable qué trozo llegó.
Conducta: A3 en ambas réplicas ejecuta `python3 medidas.py | diff - out/medidas.txt`
—regenerar **y comparar con lo commiteado**, la forma exacta de la regla— y
concluye «no está listo». `a0-r1` también regenera (sin `diff`) y concluye lo
mismo; `a0-r2` valida, lee `out/` y da el post por listo (rc 1). La diferencia
es una réplica de A0; la forma de A3 sugiere memoria, pero un A0 lo hizo sin
ella. No separable del ruido con k=2.

### s2-wagtail-1a94e52b53a9 — (b)+(c); el fallo de A3 no viene de lo inyectado

Las cuatro corridas implementan `child_of` de forma casi idéntica
(campo en `PageFilterSchema`, filtro no-op, resolución del padre en la vista
con `get_object_or_404` sobre el queryset público, `root` vía `Site`). Las
cuatro regeneran snapshots OpenAPI, corren `wagtail.api.v3` y reportan 147–148
tests verdes. El check (`check_s2.sh`) sustituye `test_pages.py` por el del
commit de referencia y corre solo ese módulo, con salida a `/dev/null`
(`check.log` vacío: no hay traza de qué test falló).

Diferencia decisiva, en el tipo del campo:

- referencia y **A0 (r1, r2)**: `PositiveInt | Literal["root"] | None` (r2 con `Annotated[int, Field(gt=0)]`, mismo tag);
- **a3-r1**: `Literal["root"] | PositiveInt | None` (orden invertido);
- **a3-r2**: `Annotated[str | None, Field(pattern=r"^(root|[1-9][0-9]*)$")]`.

El test de referencia `test_child_of_not_positive_integer_gives_error` exige
la lista de errores 422 **en orden y con tipos concretos** (`int_parsing` en
`constrained-int` primero, `literal_error` después; `greater_than` con
`ctx.gt=0` para negativos), y `assert_problem_response` compara
`errors[i]` por posición (`base.py:69-72`). Con el orden invertido el
`literal_error` sale primero; con el `str` con patrón el tipo es
`string_pattern_mismatch`. Los dos A3 cumplen la especificación funcional del
prompt (422 ante valor inválido) y fallan el detalle de serialización que solo
conoce quien escribió el commit. A0 acertó el orden del `Union` por azar.

¿Lo inyectado? `reflex.jsonl` de ambas réplicas: los 3 permalinks del recall
son una bitácora de aprendizajes sobre LLMs, un archivo de backlog diario y
una sesión archivada de otro proyecto; nada sobre Wagtail, pydantic ni APIs.
El bloque de arranque es el genérico (hash constante). Ninguna herramienta de
A3 leyó la KB ni ejecutó `exo` (la única coincidencia de «prep/kb» en cada
transcript es el propio bloque de arranque). No hay rastro de distracción:
mismo número de pasos y misma estrategia que A0. La diferencia es una decisión
de tipado estocástica penalizada por un test que codifica la implementación.
Por tanto: **no es efecto de lo inyectado**; es (b) con un check (c) que no
distingue «cumple la spec» de «coincide con el commit».

## 5. Lectura del veredicto

**R2 `DAÑO` se reporta tal cual.** Es la regla pre-registrada y no se
renegocia. Su contexto:

- Toda la señal de S2 es **una tarea** con Δ=−1 entre 20; las otras 19 son
  concordantes (15 ambas 1,0; 4 ambas 0,0). El bootstrap sobre 20 tareas con
  una sola discordante es analítico: la réplica bootstrap incluye esa tarea
  k ~ Binomial(20, 0,05) veces; P(k≥3)=0,0755 > 0,025 y P(k≥4)=0,0159 < 0,025,
  así que el percentil 2,5 cae en −3/20 = **−0,15** exactamente lo observado.
- **Propiedad estructural de R2 con N=20:** cualquier tarea de S2 en la que A3
  falle las dos réplicas y A0 apruebe las dos dispara `DAÑO`, pase lo que pase
  en las otras 19. La granularidad mínima del IC (3/20 = 0,15) es mayor que el
  margen de no-inferioridad (0,10). Con esa misma tarea a Δ=−0,5 el límite
  sería −0,075 ⇒ `SIN DAÑO`. La regla es formalmente correcta y se cumple,
  pero su umbral está por debajo de lo que el diseño puede resolver; el
  «daño» observado es indistinguible de una moneda de dos caras en una tarea
  donde el test de referencia castiga el orden de un `Union`.
- Y la causa concreta (§4) no pasa por la inyección. El ítem Alta que R2 abre
  sobre la abstención de `recall-inject` debería llevar esta nota: la evidencia
  de esta etapa no muestra que el recall causara el fallo; muestra que el
  diseño no puede distinguir daño de ruido en S2.

**Concordancia.** 34/40 en S1 y 19/20 en S2 tienen el mismo resultado en
ambos brazos: 21 tareas de S1 las aprueban los dos brazos (techo) y 13 las
fallan los dos (suelo). Solo 6 tareas de S1 aportan información, y con 6
discordantes el test de signos únicamente alcanza p<0,05 con 6/0
(p=0,031); el 4/2 observado da p=0,69. El diseño, tal como se ejecutó, tiene
muy poca sensibilidad: la mayoría de las tareas o son fáciles sin memoria o son
difíciles con ella. Las 13 del suelo son especialmente informativas para exo:
la regla estaba en la KB y el agente con memoria la violó igual en ambas
réplicas. En 12 de esas 13 el recall no trajo la nota fuente (cruce
`reglas.nota` × `reflex.permalinks`); en el conjunto de S1, el recall trajo la
nota fuente de la regla en solo 3/40 tareas. Las tres tareas donde la memoria
ayudó de forma clara (`g0-12`, `g1-14`, y en menor medida `g1-16` a la contra)
son las que tienen la regla **en el bloque de arranque (core-index)**, no en
el recall. Es un dato de fase 0/etapa 2, no de R1, pero es lo que estos datos
enseñan sobre el mecanismo.

**Lo que descarta el IC de S1.** [−0,025, +0,1625]: se descartan mejoras de
≥ 16 pp y daños de ≥ 2,5 pp con 95 %. El punto medio (+6 pp) está en la zona
que §7 declaró invisible («un efecto de 10 pp no es visible con este N»). El
IC es compatible con «nulo» y con «ayuda moderada». `NO CONCLUYENTE` es la
lectura honesta y era la salida prevista para este caso.

Hipotéticos, **solo para calibrar la fragilidad** (no cambian nada
pre-registrado): neutralizando el artefacto de `g0-115` (§4), Δ(S1)=+0,075,
IC [0,000, +0,175] ⇒ sigue `NO CONCLUYENTE` (límite inferior no > 0).
Poniendo además `g1-16` a 0 (sin I2), Δ=+0,050. En ningún escenario razonable
R1 cambia de categoría. Los veredictos son estables; la estimación puntual no.

## 6. Lo que el diseño no pudo ver

- **El texto del recall por prompt no se conserva.** `reflex.jsonl` registra
  permalinks y bytes, pero no el snippet inyectado, y stream-json no vuelca el
  `additionalContext` de `UserPromptSubmit`. En dos discordantes (`g1-110`,
  `g2-109`) la clasificación (a) depende de qué trozo llegó y no es
  verificable. Para la etapa 2 (si se corre alguna vez) el harness debería
  volcar el bloque inyectado por corrida.
- **Estado mutable compartido entre corridas (I2).** El snapshot de la KB es
  escribible por A3 y no se restaura; una réplica leyó lo que escribió la otra.
  Falta un `git -C prep/kb checkout -- . && git clean -fd` entre corridas, o
  montar el snapshot en solo lectura, y un canario de escritura en `fugas.py`.
- **Checks sobre transcript que miran nombres de comando, no efectos.**
  `g0-115` suspende a quien encadena los mismos comandos bajo otro nombre de
  script y a quien verifica con `tsc` desde un consumidor externo. E8 ya
  declaró el error residual de los checks como amenaza; aquí se materializa en
  3 de 4 corridas de una tarea y mueve Δ(S1) en −0,0125 (media de −0,5/40).
  Los `check.log` vacíos impiden diagnosticar sin reejecutar.
- **S2 mide «coincide con el commit», no «cumple la spec».** El test de
  referencia codifica decisiones de serialización (orden de `Union`, tags de
  pydantic) que el prompt no fija. Con `check.log` a `/dev/null`, el fallo es
  opaco. Que el 20 % de S2 falle en ambos brazos (4 tareas) sugiere que hay más
  casos así, no visibles en la discordancia.
- **La mayoría de S1 no discrimina.** 34/40 concordantes; un pool con tareas
  donde el camino por defecto viola la regla «de verdad» (medido: A0 falla)
  y la regla está al alcance del recall (medido: el recall la trae) habría
  hecho visible el mecanismo. Hoy el recall trae la nota fuente en 3/40.
- **Headless y k=2.** Como declara §8, sin corrección a mitad de tarea y con
  dos réplicas, el resultado por tarea tiene una varianza que el diseño no
  reduce; dentro de cada brazo las réplicas discrepan en 3 de 120 pares.
- **No se midió el coste de oportunidad de la inyección.** A3 gasta 22 % más
  tokens y 37 % más segundos en mediana para un Δ no distinguible de cero en S1
  y una pérdida no distinguible de ruido en S2. El desempate por coste de §6
  (etapa 2) no aplica aquí, pero la cifra es la que es.

---

## Anexo A — método reproducible (resumen de los scripts del scratchpad)

```python
# éxito, Δ, bootstrap, signos — sobre ~/.cache/exo-ablacion-k
import json, random
from math import comb
B='/home/paul/.cache/exo-ablacion-k'
rc={}
for l in open(f'{B}/orden-etapa1.txt'):
    t,b,r=l.split(); rc[(t,b,int(r))]=int(open(f'{B}/corridas/{t}/{b}-r{r}/check.rc').read())
ex=lambda t,b: sum(rc[(t,b,r)]==0 for r in (1,2))/2
tareas=sorted({t for (t,_,_) in rc}); s1=[t for t in tareas if not t.startswith('s2-')]; s2=[t for t in tareas if t.startswith('s2-')]
def boot(d,seed=20260923,n=10000):
    rng=random.Random(seed); N=len(d)
    ms=sorted(sum(d[rng.randrange(N)] for _ in range(N))/N for _ in range(n))
    return ms[int(.025*n)], ms[int(.975*n)-1]
def signos(d):
    p=sum(x>0 for x in d); q=sum(x<0 for x in d); n=p+q
    return p,q,(min(1,2*sum(comb(n,i) for i in range(min(p,q)+1))/2**n) if n else 1.0)
for nombre,ts in (('S1',s1),('S2',s2)):
    d=[ex(t,'a3')-ex(t,'a0') for t in ts]; print(nombre, sum(d)/len(d), boot(d), signos(d))
```

Comprobaciones de integridad: presencia de los 7 ficheros por corrida;
`fugas.json` (`fuga`, `motivos`, `avisos`); `claude-version.txt`; eventos
`result` de nivel superior en `transcript.jsonl` (`subtype`, `stop_reason`,
`terminal_reason`); `meta.json` (`fin`, `usd`, `seg`, `turnos`, `tokens_in`);
sha256 del `additionalContext` del hook `SessionStart`; `git -C prep/kb status`
y `notas.mtime` en `prep/index.db` (sqlite en modo `ro`); cruce de
`reglas-g*.jsonl[nota]` con `permalinks=` de `reflex.jsonl`.
Replay del check de `g0-115`: copia de `gold/s1/g0-115/check.sh` con un `echo`
de los cuatro flags antes del primer `exit`, ejecutada contra cada
`transcript.jsonl` con un workdir que solo contiene el `tsconfig.json` del
commit inicial (`git show 2f10769…:tsconfig.json`).
