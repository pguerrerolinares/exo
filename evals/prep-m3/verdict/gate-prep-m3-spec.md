# Verdict de GATE — rama `prep-m3-skills` (spec+gold del plugin `process`)

- **Fecha**: 2026-07-17
- **Rol**: consultor delegado de gate (régimen framework §8 / config §Ejecución
  de gates), dispatch fresco, sin participación en ninguna fase de la pieza.
- **Pregunta**: ¿la rama `prep-m3-skills` (6 commits sobre `master@99f38fd`) es
  mergeable como entrega spec-first del ítem prep-M3 (framework §5.3 paso 1 y
  solo paso 1), con el gold como oráculo válido de la implementación futura?

## Veredicto: MERGED

Las 4 condiciones del régimen (framework §8 a-d) se cumplen y la verificación
primaria propia no encontró nada bloqueante. Detalle y citas abajo.

## 1. Scope — sin creep hacia el cutover (verificado sobre el diff completo)

`git diff master...prep-m3-skills --name-only` = 10 ficheros, TODOS bajo
`docs/superpowers/specs/` y `evals/prep-m3/`. Nada toca `plugins/`,
marketplace, `.claude/`, core-index, ni instala/deshabilita nada. La spec §1
deja fuera explícitamente el cutover, citando la formulación literal del
config (§GATE-CALENDARIO-D: *"Bloquea: … M3 cutover real (paso 2 del
checklist §5.3 en adelante) … y cualquier cambio que altere
marketplace/skills/recall del agente"*). Coherencia verificada hasta en los
rincones: el §6 de la spec DISEÑA la línea de routing y el contador de
no-disparos pero no los activa (la nota core-index de kb-demo no se toca;
`evals/prep-m3/no-disparos.md` NO existe en la rama — el README línea 14-16 lo
declara "Futuro, post-cutover M3 … No existe aún: se crea en el cutover, no en
esta pieza").

## 2. La cadena de verificación es real (muestreo primario propio)

Verifiqué yo mismo, contra los ficheros fuente reales (paths de las cabeceras
de cada gold), **20 movimientos + 5 DESCARTES en 6 de las 7 skills** —
por encima del mínimo del mandato (≥10 ítems, ≥4 skills, ≥2 DESCARTES,
paridad crítica incluida). Todos con base textual literal:

- **PARIDAD CRÍTICA (orchestrate)**: OP línea 51 = *"Despacha los ejecutores
  como `subagent_type: reflex:executor`, nunca `general-purpose`"* ✓; OP
  líneas 26-28 = *"Salvo roles con `model` fijo en su definición — p.ej.
  `reflex:executor` — donde NO debes pasar `model` en el dispatch (lo
  pisarías)"* ✓; framework §5.3.2 literal ✓; y el DESCARTE asociado es real:
  `implementer-prompt.md` fuente, líneas 6-9, dice exactamente
  `Subagent (general-purpose)` + `model: [MODEL — REQUIRED…]` ✓. (Nit no
  bloqueante: la cita "OP 50-51" incluye la línea de cabecera de sección y
  "27-28" arranca una línea tarde — off-by-one de borde de rango, contenido
  literal presente.)
- **orchestrate**, ítems añadidos por los fixes: SDD 164-165 (no directivas
  open-ended) ✓, SDD 166-167 (no re-correr tests del implementer) ✓, SDD
  203-207 (package del review final con `MERGE_BASE` vía
  `git merge-base main HEAD`) ✓ — las líneas corregidas por el fixer son las
  correctas. Red lines SDD 370 (nunca main/master sin consentimiento) ✓ y 373
  (nunca implementers paralelos) ✓. Memory packet OP 57 ✓;
  investigate-don't-stop OP 110-112 ✓.
- **DESCARTE O4 corregido (orchestrate)**: contra el mapa real de secciones de
  DPA — §Real Example 136-161, §Key Benefits 163-168, §Verification 170-176,
  §Real-World Impact 178-185. El rango descartado (136-161 + 163-168 +
  178-185) excluye exactamente §Verification 170-176, que el gold conserva
  como movimiento ✓. El fix elimina la colisión que el verdict adversarial
  señaló.
- **tdd**: borrar-código-pre-test (37-45: *"Write code before the test?
  Delete it. Start over."*) ✓; Verify RED (113-128: *"Test fails (not
  errors) … Fails because feature missing (not typos)"*) ✓.
- **debug**: la corrección obligatoria del hook es verificable — RF línea 3
  contiene efectivamente *"Triggered by the reflex `stuck-loop` hook"* y
  framework §5.2 lo declara muerto (*"commit 590d6ca"*) ✓; movimiento
  recon 3 (RF 39-42, verificar el supuesto más barato) ✓; DESCARTE RF 17
  (*"Fuente canónica … manda la nota"*) literal ✓.
- **documenta**: degradación visible (35-38, literal incluida la línea `kbx
  unavailable → search_notes fallback` — el seed del patrón único §3.4 de la
  spec) ✓; regla de oro (40) ✓; fix DOC1 (48: *"Ya NO se crea 'una nota por
  sesión' en `sesiones/`…"*) ✓; commit scoped (58-63: nunca `git add -A`,
  `git -C`, no push) ✓; DESCARTES observations/relations (55-56, formato de
  fila literal; los wikilinks se conservan per framework §6.3) ✓ y trailer
  (64) ✓.
- **brainstorm**: preguntas una-a-una + purpose/constraints/success criteria
  (26 + 72-73, los fixes B1/B2 aplicados apuntan a las líneas correctas) ✓;
  cobertura del diseño (86, no 87 — fix B1 correcto) ✓.
- **verify**: gate function (VBC 25-38) ✓; red-green verificado (VBC 84-88:
  *"Write → Run (pass) → Revert fix → Run (MUST FAIL) → Restore"*) ✓.

Conclusión del muestreo: el verdict adversarial (`gold-verification.md`,
commit 70fe3cc) no miente — su cobertura declarada del 100%, sus 13 findings
y sus correcciones de línea son consistentes con lo que las fuentes dicen.

## 3. Spec contra criterio raíz

- **Tabla §5.2 respetada**: la tabla §4 de la spec reproduce la del framework
  sin fuentes nuevas ni mapeos inventados; la fila `documenta` está cubierta
  por framework §5.2 (*"`/documenta` entra a process"*). La única extensión —
  gold orchestrate absorbe de orchestrate-personal más movimientos que los 4
  nombrados en el paréntesis de la tabla — fue flaggeada por el adversarial
  (O1), corregida en la cabecera del gold ("como mínimo … además se absorben
  los movimientos de OP que morirían con ella en el cutover") y es
  materialmente correcta: framework §5.3.2 mata `paul-profile:orchestrate-personal`
  entera en el cutover (*"ambas mueren"*), así que no absorber
  padre-valida-siempre / pre-flight recon / autonomous runs los perdería sin
  decisión. Misma fuente de la fila, no fuente nueva. La acepto.
- **Formato §5.1 respetado**: frontmatter de disparo + body ~30-50 (solo
  body, `[adjudicado]` registrado) + reference files en el directorio de la
  skill (especificados, no escritos) + patrón único de degradación
  generalizado del fallback de /documenta (verifiqué el seed literal en la
  fuente, líneas 35-38).
- **Cutover explícitamente FUERA**: spec §1 completo; ver §1 de este verdict.
- **Kill-criteria no vagos**: los 4 de spec §9 tienen acción concreta
  (overflow ⇒ reference files ⇒ escalar, nunca recortar checklist; cap 2
  retries ⇒ parar; movimiento sin cita = fallo aunque el resto pase; paridad
  crítica no negociable en retry). Matan de verdad.
- **Fixes post-review reales**: commit 8d13296 aplica exactamente los 13
  menores (conté los hunks contra la lista B1-B2/P1-P2/O1-O5/D1-D3/DOC1);
  commit a3fcc0f añade el ítem de degradación del memory packet en gold
  orchestrate, espejo del de documenta línea 10, como exige spec §3.4 ✓.

## 4. Adjudicación PENDIENTE-CONSULTOR: idioma castellano

**CONFIRMADO: castellano con términos técnicos en inglés**, con esta base —
mejor que la que dio el spec-writer:

- **Framework §1 (la cita que decide)**: *"Personal-primero: el usuario del
  framework es Paul. La genericidad es una propiedad de diseño … no un
  producto: no se construye nada cuyo único consumidor sería un tercero
  hipotético"*. Elegir inglés hoy sería exactamente eso: construir para el
  tercero hipotético del marketplace futuro. El registro de trabajo de Paul
  (perfil global: castellano con términos técnicos en inglés sin traducir) es
  el del usuario real.
- **No es superficie irreversible**: framework §8 nombra como irreversible el
  *formato* de skill (estructura frontmatter + body + reference files), no el
  idioma del texto. Cambiar idioma después es un rewrite barato de texto que
  no rompe ningún contrato (los names, paths y el patrón de degradación no
  cambian). Reversible en M1b/implementación ⇒ no exige a Paul hoy.
- **Corrección a la evidencia del spec-writer** (spec §3.1: "el patrón ya
  operativo en las fuentes propias (recon-first, /documenta)"): es impreciso.
  Verifiqué las fuentes: la description de recon-first está en INGLÉS (RF
  línea 3) y el body de orchestrate-personal es mayormente inglés; solo
  /documenta es castellano pleno (y el body de recon-first). La decisión se
  sostiene por framework §1 + perfil de Paul, no por ese "patrón". La cita de
  este verdict sustituye a la de la spec como registro de adjudicación.
- **Reserva registrada (no bloquea)**: si el marketplace llega a publicarse
  para terceros (M1b o después), el idioma de las descriptions se re-adjudica
  entonces como decisión de producto — y esa sí es de Paul, porque publicar
  fuera es línea roja no delegable (config §Ejecución de gates). Hasta
  entonces, castellano.

## Qué busqué para objetar

1. **Scope-creep enmascarado hacia el cutover**: diff completo fichero a
   fichero; busqué activaciones escondidas en §6 (línea de core-index,
   contador) y la creación anticipada de `no-disparos.md` — no existen; el
   README lo declara futuro explícitamente.
2. **Ítems de gold sin base real**: muestreo propio de 20 movimientos + 5
   DESCARTES contra las fuentes (6 skills, incluida la paridad crítica con
   sus tres citas y el implementer-prompt fuente) — 0 ítems sin base; el
   único residuo es un off-by-one de borde en "OP 50-51/27-28" (contenido
   literal presente).
3. **DESCARTES que matan movimientos**: ataqué el rango O4 corregido contra
   el mapa real de secciones de DPA (excluye limpio §Verification 170-176);
   los descartes de tdd/debug/documenta conservan la regla destilada que
   dicen conservar (borrar-código-pre-test, hipótesis única, wikilinks).
4. **Contradicciones spec↔gold que la review final no viera**: el límite
   no-dispatch de verify (spec §5.6) tiene su ítem verificable-por-ausencia en
   gold verify línea 11 y su contraparte en orchestrate ✓; los dos únicos
   probes de §3.4 (documenta, orchestrate) tienen ítems espejo en ambos golds
   (documenta línea 10 / orchestrate línea 59) ✓.
5. **Kill-criteria vagos que no maten nada**: los 4 de §9 tienen trigger y
   acción concretos; el cap de retries viene del config (§Presupuesto, citado).
6. **"Solo paso 1" violado en algún rincón**: además del diff, revisé que la
   spec no comprometa acciones presentes fuera de scope — la atribución MIT
   (§8) se define como contenido del *primer commit de implementación*, no de
   esta rama: correcto, esta rama no copia código aún.
7. **Cadena de verdicts citándose en círculo**: mi muestreo fue directo a
   fuente, independiente de las citas del verdict adversarial, y confirmó sus
   correcciones de línea (SDD 164-165/166-167, brainstorm 26/86) — la cadena
   toca fuente real.
8. **La evidencia de la decisión de idioma**: la ataqué y ENCONTRÉ el fallo
   (recon-first tiene description en inglés) — no cambia el veredicto porque
   la base correcta es framework §1, pero queda corregido arriba (§4).

Convergencias que no encontré motivo para romper: paridad crítica (triple
cita exacta + corpus negativo real en implementer-prompt), la tabla de
absorción como fuente única, y el reparto verify↔orchestrate del parent
validation gate.
