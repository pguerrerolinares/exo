# Brief — Consultor Fable: REVIEW de la spec de M6-06

## Rol

Eres un consultor Fable **fresco** del régimen de gates delegado del proyecto exo.
No escribiste tú el verdict que hay en este directorio: lo escribió otro consultor,
y tu trabajo es precisamente no heredar su marco.

Paul acaba de aprobar el diseño de **M6-06** (recall en el punto de uso), el último
item vivo de M6 y **único bloqueador de M5b**. La spec está escrita y commiteada. El
siguiente paso es `process:plan` → implementación. Tu review es la última puerta
antes de que eso ocurra.

**No toques código de producción, no commitees, no edites la spec.** Scripts de
medición en tu scratchpad, sí.

## Orden de lectura OBLIGATORIO (importa)

1. **Primero, y a solas**: `docs/superpowers/specs/2026-08-22-m6-06-recall-punto-de-uso-design.md`.
   Léela como la leería el ejecutor que va a implementarla sin más contexto que ese
   fichero. Anota TODO lo que no puedas construir sin preguntar. Forma tu juicio aquí.
2. **Después**: verifica contra el código y la máquina lo que la spec afirma (§ más
   abajo).
3. **Solo al final**: `consultas/2026-08-22-m6-06/consultor-m6-06.md` (el verdict del
   otro consultor, con sus apéndices A y B) y `brief.md`. Sirven para comprobar si la
   spec **tergiversó, perdió o suavizó** algo de lo adjudicado — no para adoptar su
   punto de vista.

Contexto de fondo, si lo necesitas: `specs/2026-08-18-cierre-en-regimen-design.md`
§3.2/§3.4 (el encargo original), `plugins/reflex/scripts/exo-recall.sh` (el hook
hermano y molde estético), `plugins/reflex/scripts/_reflex-log.sh`,
`plugins/reflex/hooks/hooks.json`, `engine/src/main.rs` (flags de `exo recall`),
`engine/src/recall.rs`. KB viva en `/home/paul/Documentos/proyectos/kb-demo`,
índice en `~/.exo/index.db`, binario en `~/.local/bin/exo`. `sqlite3` CLI **no está
instalado**; usa `python3 -c` con el módulo `sqlite3`.

## Qué quiero de ti, por orden de valor

### 1. ¿Está lista para plan? (la pregunta que decide)

Un ejecutor sin contexto, con solo esta spec delante, ¿puede construir el hook sin
inventarse nada? Enumera **cada** punto subespecificado, y para cada uno di si es
(a) un agujero que la spec debe tapar antes del plan, o (b) una decisión legítima
de implementación que el plan resolverá. No me des una lista de "sería bueno
aclarar": di cuáles **bloquean**.

### 2. Verifica los claims que la spec hereda (medición primaria propia)

La spec se apoya en hechos medidos por otro. Comprueba por muestreo los que, si
fueran falsos, se llevarían el diseño por delante:

- `exo recall --query` sin `--min-similitud` cae al 0.35 de config (la spec lo llama
  "degradación silenciosa con forma válida" y hace del flag explícito una regla).
- `exo recall --query` sale con **exit 1** cuando no hay hits sobre el umbral, y ese
  exit 1 es indistinguible por código de un fallo real (P2 depende entero de esto).
- En `UserPromptSubmit`, **exit 2 borra el prompt** y el timeout por defecto del
  evento (P1 y P4 dependen de esto; verifica contra documentación oficial, no de
  memoria).
- El coste de `--refresca` dentro del proceso hybrid, y qué pasa exactamente con DB
  ausente (P5).
- La forma real del modo texto de `recall --query` (§2.4 asume que basta con
  sustituir la primera línea: compruébalo con salida real).

Si alguno cae, di qué sección de la spec se lleva por delante.

### 3. Ataca el diseño donde la spec está más cómoda

No repitas sus argumentos: búscale las costuras. Sitios donde yo sospecho que puede
haberlas, sin que te limites a ellos ni asumas que tengo razón:

- **Prompts que pasan el gate y son enormes** (Paul pega logs, diffs, tracebacks de
  varios KB). Van por argv a `exo recall --query`. ¿Límite de argv? ¿Qué embebe el
  modelo con 8 KB de traceback? ¿Cuánto tarda? ¿Trae algo útil? La spec no lo menciona.
- **La lista cerrada de ~50 stopwords**: la spec la declara "el nuevo punto de
  mantenimiento" pero no la enumera, no dice dónde vive ni cómo normaliza (¿folding
  de acentos? ¿"sí"→"si"?). ¿Es un placeholder disfrazado de decisión?
- **Prompts multilínea** y prompts que empiezan con texto pegado (¿empieza por `<`
  un fragmento de HTML/XML que Paul pega? La regla 1 lo saltaría — ¿importa?).
- **Interacción con el resto de la cascada de hooks** y con `/compact`.
- **P3 y el ruido del log**: dispara ~86% de los turnos y emite un evento por
  disparo. ¿Qué le pasa a `reflex-log.jsonl` en un mes? ¿Alguien lo rota?
- **El modo de fallo social** que la propia spec admite en §6: que Paul deje de
  mirar los punteros y el hook se vuelva papel pintado. ¿Hay algo barato en el
  diseño que lo retrase, sin reabrir §0?

### 4. Fidelidad al verdict (solo tras leerlo)

¿La spec perdió, suavizó o tergiversó algo adjudicado? Interesa especialmente lo
que se haya vuelto **más blando** al pasar de verdict a spec, y cualquier número que
haya viajado mal.

## Restricciones que la spec NO puede violar (y tú tampoco al proponer arreglos)

1. Régimen §0: sin métricas nuevas, sin ventanas de medición, sin gates de eficacia.
   (Sí se permite instrumentación de degradación del propio hook.)
2. `--min-similitud 0.40` sellado desde M2-07.
3. Cero cambios en el engine — es la propiedad que hace este item barato. Si
   encuentras algo que **obliga** a tocar Rust, dilo alto: cambia el coste del item.
4. YAGNI. Si tu arreglo es más caro que el problema, no es un arreglo.
5. El hook JAMÁS destruye el prompt de Paul ni bloquea el turno.

## Formato del veredicto

Escríbelo en `docs/superpowers/consultas/2026-08-22-m6-06/review-spec.md`:

1. **Verdict global en una línea**: lista para plan / lista con N arreglos
   obligatorios / no lista.
2. **Bloqueantes** — numerados, cada uno con la sección de la spec que toca y el
   texto concreto que falta o sobra.
3. **No bloqueantes** — lo que el plan puede resolver.
4. **Claims verificados**, con el dato y cómo lo mediste.
5. **Costuras nuevas** que no estaban vistas.
6. **Qué consideraste y descartaste, con la razón.**

Tu respuesta final a mí: ~30 líneas. Verdict, bloqueantes en una línea cada uno,
costuras nuevas. El detalle va al fichero.

Y una petición explícita: **si la spec está bien, dilo y no inventes trabajo.**
Un review que fabrica objeciones para justificarse cuesta más que uno que firma.
