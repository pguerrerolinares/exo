# M3 + M1b — cutover de skills y marketplace: diseño

> **Régimen de esta spec:** las decisiones las adjudicaron tres consultores fable
> independientes (mecánica del cutover, barrido de dependencias, marketplace).
> Paul firmó sus decisiones por adelantado. Donde dos verdicts colisionaron, el
> orquestador reconcilió con evidencia verificada y lo dejó escrito (§6).
> Esto es la síntesis ejecutable, no una propuesta a aprobar.

**Goal:** que el proceso de trabajo de Paul deje de depender de `superpowers` y
pase a servirse de plugins propios bajo una sola identidad, `exo`. Cierra la
campaña C8 del plan `2026-08-17-cierre-exo-m2-a-m5b.md`.

**Spec madre:** `2026-07-16-framework-unificado-design.md` §5.2 (qué se absorbe),
§5.3 (checklist de cutover), §8 (régimen de gates).
**Spec de las skills:** `2026-07-17-prep-m3-process-skills-design.md`.
**Verdicts:** `docs/superpowers/consultas/2026-08-18-c8/`.

---

## 1. El hallazgo que redefine M3-01

El plan describía M3-01 como un gesto: «Mismo día: `superpowers` disabled +
`process` enabled». La foto real al abrir la campaña era otra: `plugins/process/`
tenía las 7 skills con paridad 135/135 verificada, **pero no era instalable**.
Le faltaba `.claude-plugin/plugin.json` y no figuraba en ningún catálogo.

```
$ claude plugin validate plugins/process     # antes: componentes válidos
✔ Validation passed
$ claude plugin validate .                   # antes: no hay plugin que instalar
✘ directory: No manifest found in directory
```

M3-01 no era un flag: era **empaquetar primero, apagar después**. De ahí que la
campaña se parta en dos fases con dueños distintos (§5).

## 2. El plugin `process`

Un solo fichero nuevo, `plugins/process/.claude-plugin/plugin.json`, campo a
campo calcado de los dos plugins que ya funcionan (`paul-profile`, `reflex`).
Sin `hooks.json` y sin declaración de `skills`: el autodescubrimiento de
`skills/` es el default, y process es solo skills por diseño.

Id resultante: **`process@exo`**. Versión `1.0.0`, que se bumpea en cada cambio
— es el mecanismo por el que el harness propaga updates.

## 3. Dónde vive el catálogo: un solo `exo`

**El repo `agent-develop` se renombra a `exo-plugins` y su catálogo pasa a
llamarse `exo`.** Rename y no repo nuevo porque los redirects de GitHub cubren
justo la pieza frágil — `git clone/fetch/push` al nombre viejo siguen
funcionando — y el clon vivo de `~/.claude/plugins/marketplaces/` no tiene
ninguna ventana sin servir. Con repo nuevo, reflex (50.354 usos, sirve los hooks
de cada sesión) corre el riesgo de quedar stale.

El campo `name` cambia **a la vez** que el repo, aunque el plan vendía el rename
como «mantiene la identidad» a coste cero. Razón: C8 ya obliga a cirugía de
`enabledPlugins` ese mismo día; la ventana está abierta y el coste marginal de
renovar el sufijo es de minutos, una sola vez. La alternativa es arrastrar
`@agent-develop` para siempre en un framework que se llama exo.

Foto final del catálogo — cuatro plugins:

| plugin | source | versión | por qué |
|---|---|---|---|
| `process` | `git-subdir` → repo `exo`, path `plugins/process` | 1.0.0 | su fuente de verdad es exo, donde co-evoluciona con el engine y sus evals |
| `paul-profile` | `./plugins/paul-profile` | 0.6.0 | queda solo `fabrica`, que es **instancia**, no framework |
| `reflex` | `./plugins/reflex` | 0.13.1 | sin cambios de fondo; se corrige el drift del catálogo |
| `workflow-lint` | `github` → repo aparte, público | 0.1.0 | 39 usos, coste de mantenimiento cero; sacarlo no compra nada |

**`process` no se copia físicamente al repo del catálogo.** `git-subdir` (source
verificado como real, §7) permite una sola copia del plugin, en el repo donde se
edita. Copiarlo crearía dos árboles que divergen.

**Drift corregido y regla de higiene nueva:** el catálogo declaraba reflex
`0.11.0` mientras `plugin.json` decía `0.13.0` (y 0.13.0 era lo instalado).
A partir de aquí, la versión del catálogo se sincroniza con la del `plugin.json`
en cada release.

## 4. Lo que muere con `superpowers` y no era obvio

El modo de fallo que importaba no era el ruidoso. Comprobado empíricamente: una
skill invocada que no existe da `Unknown skill` **visible**, no silencio ni
alucinación. **El fallo silencioso está en la prosa**: una skill que dice «el
motor es `superpowers:X`» no invoca nada — el agente simplemente procede sin la
metodología.

Cuatro referencias vivas de esa clase, todas arregladas:

1. `fabrica/SKILL.md:8-9` — el motor por pieza de **toda sesión-fábrica**
   apuntaba a dos skills que desaparecen. Ahora apunta a `process:orchestrate`,
   que ES la fusión de ambas.
2. `paul-profile:orchestrate-personal` — **se retira**, no se aliasa. Un alias
   recrearía el layering que la fusión eliminó y dejaría dos rutas divergentes
   al mismo motor. La spec madre ya lo había firmado («se absorben al monorepo:
   paul-profile menos fabrica»).
3. `reflex/skills/recon-first/SKILL.md:54` — puntero a
   `superpowers:systematic-debugging` → `process:debug`.
4. `a1-freeze-watch.sh` — cron diario que hashea `orchestrate-personal` desde la
   caché. Sin esa skill, appendearía «FREEZE ROTO» espurio cada día. La ventana
   A1 cerró el 2026-08-02: el script y su test se retiran.

**`fabrica` se queda en `paul-profile`, no migra a `process`.** La spec separa
framework (exportable) de instancia (personal): `process` es lo primero,
`fabrica` lo segundo — protocolo personal más su propio guard PreToolUse.

**Lo que NO se toca**, y conviene que quede escrito porque parece limpieza:
la atribución MIT (13 hits — es M3-04, tocarla es regresión), las cachés de
`superpowers 6.x` (son el rollback), y **cualquier ruta `.superpowers/` o
`docs/superpowers/`**: son convenciones de filesystem sin relación con el
plugin, y limpiarlas rompería fabrica, el walker del engine y el guard de main.

## 5. El sustituto de `using-superpowers` (M3-05)

`using-superpowers` funciona porque un hook SessionStart lo **fuerza**. Su
sustituto no puede ser otra skill (usar una skill para forzar el uso de skills
es circular), así que va donde el arranque ya inyecta contenido cada sesión: el
`core-index` de la KB, que es lo que sirve `exo recall`.

Se preserva el movimiento, no la prosa: invocar ANTES de responder, la regla del
1%, la exención de subagentes, y el routing al catálogo de las 7 skills. Se tira
lo que la spec madre manda tirar (el «announce», la tabla de red flags, la
platform adaptation).

**Medido en producción, no en aritmética:**

```
$ exo recall --db ~/.exo/index.db --contenido --nota kb-demo/core/core-index \
             --limite 10 --cap-bytes 6144
→ 5563 B, bullet presente, sin aviso de truncado
```

core-index: 4255 → 4725 B. **Margen real del bloque: 581 B**, no los ~1.400 que
sugería la cuenta sobre la nota sola — el bloque incluye el digest de recientes,
que crece con la actividad de la KB. Es deuda anotada (§8), no bloqueo.

## 6. Dónde se reconcilió un choque entre verdicts

Dos consultores firmados colisionaron: A adjudicó que **exo** fuera su propio
marketplace por path local; C adjudicó que el catálogo renombrado se llamara
**exo**. Ambos nombres son el mismo, y la doc del harness es explícita:
«adding a second marketplace with the same name replaces the first». Dos
catálogos `exo` registrados se pisan en silencio.

**Gana C.** La única justificación de A para el path local era «el único camino
sin push», y se cae contra su propio A4: el propio A adjudica que
`marketplace add` e `install` escriben `settings.json` y por tanto son de Paul —
que pushea e instala en el mismo gesto. El path local no ahorraba nada, y un
catálogo único es justamente el objetivo de M1b.

Sobrevive de A todo lo demás: el `plugin.json`, el texto de core-index y el
orden del cutover.

## 7. Verificación primaria

- `claude plugin validate` verde sobre `plugins/process` y sobre el catálogo.
- **`git-subdir` es un source real, con control negativo**: el mismo JSON con un
  source inventado falla (`plugins.0.source: Invalid input`), así que el «pass»
  del source bueno significa algo.
- Suite de reflex 9/9 · `fabrica-main-guard` 19/19 · engine 114/114.
- `exo recall` en producción (§5).
- Sonda empírica del modo de fallo de una skill ausente (§4).

## 8. Deuda que sale de aquí

1. **Margen del bloque de arranque: 581 B.** Crecerá el digest de recientes antes
   que core-index. Cuando trunque, el aviso queda en `reflex-log.jsonl`.
2. **El config de fábrica de exo miente en un punto**: dice que el guard
   PreToolUse «no cubre `kb-demo`». Sí lo cubre — denegó un commit a `main`
   de ese repo durante esta campaña y obligó al worktree. A corregir en el config.
3. `README.md` y `COMPANY-INTEGRATION-SPEC.md` de `exo-plugins` conservan prosa
   stale sobre superpowers. Cosmético; el segundo describe entornos ajenos donde
   superpowers ES el motor, y se queda tal cual a propósito.
4. El `README` de `process` dice «destilado de superpowers 6.1.1»; la instalada
   viva es 6.3.0. La paridad se verificó contra 6.1.1 por decisión de spec: no es
   error. Si algún movimiento de 6.2/6.3 duele, se añade entonces.
5. `kb-demo/projects/agent-develop.md` (líneas 86, 128) queda desactualizada
   sobre el layering. Se arregla por flujo normal de `/documenta`, no aquí.
6. **Deuda de proceso, señalada por el gate**: la medición «en producción» de §5
   se hizo contra `~/.exo/index.db`, el índice real, que `exo recall --contenido`
   relee del disco. Eso dejó el índice sirviendo el bullet desde la rama antes de
   que la rama estuviera mergeada — medir mutó lo medido. Converge con el merge y
   aquí no tuvo consecuencia, pero el patrón correcto es medir contra una DB
   efímera, o reindexar la base tras medir.

## 9. Desviaciones respecto al plan §Campaña 8

| Plan | Qué pasó | Por qué |
|---|---|---|
| M3-01 «mismo día: disabled + enabled» | Se parte en Fase 1 (empaquetado, orquestador) y Fase 2 (flip, Paul) | `process` no era instalable; y editar `enabledPlugins` cae en la línea roja del config §Ejecución de gates («cambios a `.claude/settings.json`»), que es de Paul |
| M3-02 «conserva el dispatch» | **Ya se cumplía**; no hubo diff | Verificado por el orquestador y confirmado por consultor independiente |
| M3-04 «atribución día uno» | Estaba dentro del plugin; faltaba en la raíz del monorepo | Es donde la recomendó el consultor de la capa thin en su día |
| M1b-01 «rename … mantiene la identidad» | Rename **y** cambio del `name` del catálogo a `exo` | El plan asumía coste cero por no tocar el sufijo; la ventana ya está abierta por el cutover y después vuelve a costar un remove/add completo |
| M1b-02 «decidir si workflow-lint entra» | Entra, sin cambios | 39 usos, coste cero, y la spec ya lo había dejado fuera del monorepo |
| §Deuda «cachés huérfanas reflex 0.6.0/0.8.0» | La lista real es mayor: reflex 0.6.0/0.8.0/0.11.0/0.12.0 y paul-profile 0.2.1 | Se barren con `claude plugin prune` en el gesto de Paul |
| §Deuda «`crontab -r` → C6» | C6 cerró sin ejecutarlo; se hereda aquí, y **no** como `crontab -r` | `-r` borra el crontab entero; el residuo es una sola línea |
