# Runbook — la pasada de factorización de las dos notas-imán

Fecha: 2026-08-23. Origen: expediente de auditoría del sistema de presupuestos
de la KB (`docs/superpowers/consultas/2026-08-22-presupuestos/`), **Pata B**
(puntos 6, 7 y 8) más la poda de `perfil` de la Pata A.
Estado: **método aprobado por Paul, pendiente de ejecutar.**

**Esto es un runbook, no una spec.** Es cirugía editorial sobre las notas de
Paul: cada movimiento es una decisión de contenido que no admite ejecución
desatendida. No hay plan ni orchestrate; hay pasos con puntos de aprobación.

## Precondiciones (duras, en orden)

1. La guarda de aire está en el `kbx` instalado
   (`kbx ratchet --kb <kb> --json` devuelve findings `no-air-debt`).
   Spec: `kbx/docs/superpowers/specs/2026-08-23-guarda-de-aire-design.md`.
2. El contrato editorial está escrito (`core-index` sincerado, `/consolida` con
   evicción y test del título).
   Spec: `exo/docs/superpowers/specs/2026-08-23-contrato-editorial-design.md`.
3. `git -C ~/Documentos/proyectos/kb-demo status --porcelain` **vacío**. Es
   la precondición del paso 0 de `/consolida` y aquí vale igual: los baselines
   de conservación asumen que HEAD es el estado previo.

## Por qué el método es "recon primero"

La §2 del expediente prescribe partir `doctrina-agentes` por tema y
`desarrollo-agentico` por género, cada una por su cuenta. Medido sobre los
headings reales, eso produce duplicados: el tema de **enforcement /
verificación / gates** está repartido entre **cuatro** notas, no dos.

```
enforcement / verificación / gates
  doctrina-agentes    → Régimen de gates delegado          1.747 B
                        Verificación independiente          3.838 B
                        Mutation testing como validación    1.891 B
  desarrollo-agentico → Documentación ≠ enforcement         3.334 B
                        Delegar la adjudicación (gates)     2.368 B
  fallo-silencioso    → la nota entera (creada hace días)

epistemología de la evidencia
  desarrollo-agentico → Benchmarks: pre-registro…           1.257 B
                        Negativos, controles y potencia     2.129 B
  evidencia-y-divulg. → Evidencia: cómo se cierra en falso
```

Partir cada madre aisladamente crea una hija de "verificación" y otra de
"enforcement" que son la misma nota escrita dos veces, y ambas pisan
`fallo-silencioso`, que ya existe. De ahí el paso 0.

## Estado de partida, medido el 2026-08-23

Los once sellos de la KB, ninguno con 15% de aire (la guarda no los bloquea:
juzga transiciones, no estado — son `no-air-debt`).

| Nota | size | techo | objetivo de poda | Δ |
|---|---|---|---|---|
| `core/doctrina-agentes` | 19.967 | 20.000 | 17.391 | −2.576 |
| `learnings/desarrollo-agentico` | 18.951 | 19.000 | 16.521 | −2.430 |
| `Paul - perfil de trabajo` | 17.991 | 18.000 | 15.652 | −2.339 |
| `Backlog — frentes abiertos` | 28.981 | 30.000 | 26.086 | −2.895 |
| `learnings/pragmatismo-y-pivots` | 14.323 | 15.000 | 13.043 | −1.280 |
| `learnings/evidencia-y-divulgacion` | 8.867 | 10.000 | 8.695 | −172 |
| `projects/agent-solve-it` | 18.848 | 19.000 | 16.521 | −2.327 |
| `projects/lighthouses-bot` | 15.904 | 16.000 | 13.913 | −1.991 |
| `projects/pguerrero-music` | 15.680 | 16.000 | 13.913 | −1.767 |
| `projects/agent-develop` | 15.696 | 17.000 | 14.782 | −914 |
| `projects/pguerrero.me — Hub…` | 13.738 | 14.000 | 12.173 | −1.565 |

Saldar las once serían **20.256 B** de poda. **No es el alcance de esta pasada.**
`ratchet.Seal()` escribe `min(sello_actual, declarado)`: un techo que nadie toca
no es transición y la guarda no lo juzga, así que `--seal` pasará aunque ocho de
estas notas sigan sin aire. Se saldan el día que se toquen.

**Alcance de esta pasada**: `doctrina-agentes`, `desarrollo-agentico` y
`perfil`. Las demás solo si el paso 0 les asigna un bloque entrante.

## Paso 0 — recon (solo lectura, termina en aprobación de Paul)

Sin mover un byte, producir una **tabla bloque → destino** sobre las cuatro
notas del mapa. Comando base:

```sh
KB=~/Documentos/proyectos/kb-demo
awk '/^#/{if(h)printf "%6d  %s\n", c, h; h=$0; c=0} {c+=length($0)+1} \
     END{printf "%6d  %s\n", c, h}' "$KB/<nota>.md"
```

Materia prima, ya medida:

**`core/doctrina-agentes` — 19.967 B, 13 cabeceras, ~80% del crecimiento en
headings sin relación → cajón, partir por tema, madre queda como índice.**

| Bloque | B |
|---|---|
| cabecera | 346 |
| Contrato de memoria | 1.374 |
| Orquestador limpio | 1.837 |
| Recon-first | 692 |
| Cost pyramid | 1.261 |
| Transporte mecánico | 1.326 |
| Completitud del brief (map≠territorio) | 3.773 |
| Consulta adversarial y ratificación | 732 |
| Régimen de gates delegado | 1.747 |
| Verificación independiente | 3.838 |
| Mutation testing como validación del padre | 1.891 |
| Capítulos que viven en nota propia | 642 |
| Relations | 352 |

`Capítulos que viven en nota propia` ya es el índice embrionario y prueba que el
método funcionó una vez. Es la semilla de la madre-índice.

**`learnings/desarrollo-agentico` — 18.951 B, 13 cabeceras, ~65% afines al
título → tema amplio, partir por género.**

| Bloque | B |
|---|---|
| El destello | 499 |
| Principios fundacionales | 1.987 |
| Método de trabajo SDD — la receta | 2.767 |
| Disciplina epistémica del agente | 900 |
| Documentación ≠ enforcement | 3.334 |
| Delegar la adjudicación (gates + remedios) | 2.368 |
| Hooks y guardrails de Claude Code | 738 |
| Benchmarks: pre-registro, saturación, prevalencia | 1.257 |
| Negativos, controles y potencia | 2.129 |
| Esperar procesos largos y subagentes idle | 2.013 |
| Relations (301, **en medio del fichero**, entre Benchmarks y Negativos) + Observations (470, última) | 771 |

Los tres géneros que la §2 nombra son visibles: narrativa de la meta-habilidad
(destello, principios, SDD, disciplina), referencia técnica del harness (hooks,
esperar procesos), epistemología (benchmarks, negativos) — y un cuarto,
enforcement, que es el que cruza con `doctrina-agentes`.

**Vecinas, solo receptoras**: `learnings/evidencia-y-divulgacion` (8.867 B, 3
cabeceras) y `learnings/fallo-silencioso…`.

**Salida del paso 0**: una tabla con una fila por bloque —
`nota origen · cabecera · bytes · destino propuesto · por qué`. Los destinos
posibles son: *se queda en la madre-índice*, *nota nueva X*, *vecina existente
Y*, *baja a bitácora (evicción)*.

**Gate: Paul aprueba la tabla antes de que se mueva nada.** Este es el punto de
control del runbook entero.

## Paso 1 — cirugía

Reglas, todas de la doctrina ya escrita:

- **Bloques enteros.** No se re-resume prosa al moverla (erosión del detalle:
  *brevity bias*, *context collapse*, arXiv:2510.04618). Un bloque sale tal cual
  y se le pone cabecera nueva si hace falta, nada más.
- **Nada se borra.** Lo que no va a una nota nueva va a `log/<slug>-bitacora.md`
  con fecha, o a `archive/`. `git mv` o edición, nunca `rm`.
- **Las vecinas solo reciben.** `evidencia-y-divulgacion` y `fallo-silencioso`
  aceptan bloques (append) pero **no se reescriben ni bajan su techo**. Así no
  disparan la guarda ni entran en el alcance.
- **Antes de appendear: ¿el destino ya dice esto?** Verificado en la review:
  `doctrina-agentes §"Mutation testing como validación del padre"` ("el padre
  muta el código y exige que la suite se entere") y `fallo-silencioso §2` ("la
  prueba definitiva es la mutación, no el color") son **la misma lección con
  evidencia distinta**. Appendear el bloque entero recrearía en la vecina justo
  la duplicación que esta pasada existe para eliminar. Las dos reglas anteriores
  —bloques enteros, vecinas solo reciben— no dan salida a ese caso, así que la
  tabla del paso 0 lleva una columna más: **¿el destino ya lo dice?** Si sí, el
  bloque **no viaja**: lo que viaja es su *evidencia* (los casos, los números)
  como añadido bajo la sección que ya existe en el destino, y la afirmación
  duplicada se retira del origen. Eso no es re-resumir: la formulación que
  sobrevive es la que ya estaba escrita, entera.
- **Título-afirmación** en cada nota nueva: el título dice qué es verdad, no de
  qué trata. Es lo que hace falsable el test del título en la próxima pasada.
- **Frontmatter de las hijas**: `tier` correcto y **sin `kbx_budget_max`** si
  caben en su nominal de tier. Solo declara techo la que no quepa, y entonces
  con 15% de aire desde el primer día (la guarda lo exige en primera
  declaración, con el cap de 2× el nominal de tier ya existente).

## Paso 2 — las madres quedan como índices

- `doctrina-agentes`: **índice corto, puerta única de routing**. Sin él, la
  partición convierte fricción de espacio en fricción de routing — el riesgo que
  el auditor-abogado nombró y la única mitigación acordada. Una línea por hija:
  qué contiene y cuándo ir. Un índice **no se destila**.
- `desarrollo-agentico`: igual, con las hijas por género.
- **`core-index` no cambia** más allá de lo que hizo la spec B: sigue apuntando
  a `[[doctrina-agentes]]` y `[[desarrollo-agentico]]` como puertas únicas. Las
  hijas **no** se listan en `core-index` — ese es el punto de la madre-índice, y
  es lo que impide que la partición coma el presupuesto de 6.144 caracteres.

## Paso 3 — `perfil` (Pata A pura, sin partir)

`Paul - perfil de trabajo`: 0% de headings nuevos en 6 semanas, 8 cabeceras
idénticas. **Converge en estructura: no se parte.** Engordó por dentro
(+9.853 B hasta quedar a 9 B del muro).

Remedio: **evicción editorial** sección por sección hasta ≤ 15.652 B (−2.339),
y resellado. Es el primer caso de prueba de la Pata A: si evicción + aire no
maneja esta nota sin mutilarla, esa pata cojea. Anotar el resultado — es
evidencia, no trámite.

## Paso 4 — saldar el IOU y resellar

- **IOU de la regla de los caps**: la doctrina prometió *"cuando se partan, esto
  sube"* como demanda suprimida. Se salda aquí: las hijas nacen con su tier
  nominal y sin waiver siempre que se pueda.
- **La doctrina diferida por falta de sitio.** La spec del contrato editorial
  declara que la doctrina que no cupo en `doctrina-agentes` (33 B de aire) ni en
  `desarrollo-agentico` (49 B) "se difiere al runbook". Aquí se cobra: una vez
  partidas, las madres-índice y las hijas tienen sitio. **Qué se escribe se
  decide en el paso 0**, con la tabla delante, y no antes — si al llegar aquí
  resulta que no había nada pendiente de escribir, se dice y se cierra la IOU
  en vez de dejarla abierta otra pasada más.
- `kbx ratchet --kb $KB --seal`. **Atómico**: o sella todo o no sella nada. Si
  falla, lista cada techo sin aire con su objetivo de poda — eso no es un error
  del sello, es trabajo que falta. **Nunca subir un techo para que pase.**
- `kbx budget --json` → exit 0.
- Commit scoped: `git -C $KB add <rutas explícitas>`, nunca `-A` ni `.`, sin
  push. Después `git -C $KB tag -f consolida/last HEAD`.

## Verificación (no darlo por hecho)

1. **Conservación, en dos niveles.** El conteo de cabeceras solo ve si un bloque
   entero aparece o desaparece; **no detecta pérdida DENTRO de un bloque** —un
   bullet caído, una frase recortada, una paráfrasis— que es exactamente el modo
   de fallo que "bloques enteros, nunca re-resumir" intenta prevenir. Hacen falta
   los dos:
   - **Bloques**: suma de cabeceras `## ` de las madres antes
     (`git show HEAD:<ruta> | grep -c '^## '`) = las de las madres-índice ahora +
     las de todas las hijas + las movidas a vecinas y bitácoras.
   - **Bytes**: la tabla bloque→destino del paso 0 ya trae los bytes de cada
     bloque. Suma los de origen y compáralos con los de destino. Los únicos
     bytes nuevos legítimos son cabeceras de nota, frontmatter y el texto de los
     índices-madre; los únicos que faltan legítimamente son los evictados a
     bitácora, que están contados como tales. `git diff --shortstat` de la
     operación completa da el control grueso.

   Si cualquiera de los dos no cuadra: **para y revísalo**.
2. **Routing**: buscar en la KB tres conceptos que vivían en las madres
   (`exo search --type hybrid`) y comprobar que se llega a la hija correcta. Es
   la comprobación del riesgo de routing; sin ella la mitigación es una promesa.
3. **Wikilinks**: ningún `[[…]]` entrante roto hacia bloques movidos
   (`kbx doctor --json`, findings `orphan`).
4. **Arranque**: sesión nueva, `core-index` se inyecta entero, sin FALLBACK.
5. `kbx ratchet --json` → cero `no-air` en las notas tocadas; los
   `no-air-debt` restantes son las no tocadas, y eso es correcto.

## Lo que este runbook NO promete

Que el ciclo deje de morder. **No lo hará**: la tasa de reposición es la
producción de doctrina de Paul, es exógena y nadie debe tocarla. El rebote
medido de una poda de 21,7 KB fue **+2.709 B la misma tarde**. Lo que esta
pasada compra es forma (dos cajones convertidos en familias con puerta),
mordiscos más baratos (aire garantizado) y que el mordisco lo pague el
compactador y no el escritor al cerrar sesión.

Si tras esta pasada la primera mordida en caso normal cae sobre una nota
sellada-con-aire, **eso dispara la Fase 2** del expediente (la banda de dos
umbrales gana su código). Anotarlo cuando pase; no medir nada nuevo para
buscarlo.
