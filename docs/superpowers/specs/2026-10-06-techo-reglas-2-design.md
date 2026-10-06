# techo-reglas-2: la regla de proyecto con autoridad de system prompt

**Estado:** diseño aprobado por Paul el 2026-10-06. Enmendado el mismo día: el canal pasa de mod a `--append-system-prompt-file` (ver «Hechos»). Es la última bala del frente "reglas de proyecto" (la 2d).
**Antecedente:** `evals/techo-reglas/verdict.md`, con el v1 en `GATE: NO PASA (4/11 < 6)`. Ese verdict queda sellado: este test lo sucede y no lo enmienda.

## Por qué existe

- El v1 inyectó la regla literal por SessionStart `additionalContext`. Llegó en 22/22 corridas, pero el agente la trató como sugerencia. En g0-33 dice literalmente "no lancé el agente … que sugería el hook".
- Por tanto el NO PASA vale como "no demostrado a bajo coste", no como "refutado". El IC95 de 4/11 es [0,11, 0,69], y contiene el umbral.
- Los mods de Claude Code (`prompt.compose`) permiten añadir una sección propia al **system prompt**, que es el canal de más autoridad. Si la 2d se construyera, ese sería su canal.
- **Pregunta:** con la regla en el system prompt y con framing de autoridad, ¿la cumple el agente, y cuánto más que sin ella?

## Hechos verificados que condicionan el diseño

- **Sonda del 2026-10-06** (`claude` 2.1.291), con un mod que añade una sección vía `prompt.compose`:
  - Corre bajo `claude -p --setting-sources "" --strict-mcp-config --settings <json> --plugin-dir <mod>` y devuelve el codeword con o sin `CLAUDE_CODE_ENABLE_FUNCTION_HOOKS=1`.
  - El control negativo, sin `--plugin-dir`, responde `NINGUNO`.
  - El canal funciona en el modo headless del harness.
- **Comparación mod frente a append (2026-10-06).** Las lecturas del agente estaban deshabilitadas y un mod inspector volcó las secciones de `prompt.compose`.
  - `--append-system-prompt-file` y el mod entregan ambos el codeword en el system prompt, al final.
  - El append no aparece como sección de `prompt.compose`: el engine lo añade tras componer. El mod queda como la última sección.
  - La autoridad es la misma (system prompt, al final). La diferencia es de fontanería.
- **Un mod que no carga falla en silencio.** Un `register.ts` inválido deja la corrida con exit 0 y el `result` normal en el JSON; el error solo aparece en el debug log.
  - Visto en vivo: el primer «COMPOSE-9X» del modo mod era falso. El agente lo leyó de un fichero del cwd, no del system prompt.
  - En una tanda, eso produce `a0` disfrazados de `arp`.
  - Por eso el experimento usa append: es determinista desde el flag y no añade ninguna pieza que pueda fallar sin avisar.
- **El 0/11 de K estaba forzado por la selección.** El suelo son justo las tareas en las que A0 falló. Por eso hace falta un brazo `a0` **fresco** en las mismas tareas. Además absorbe la deriva de versión: 2.1.285 en K, 2.1.286 en el v1, 2.1.291 hoy.
- g2-171 es un conflicto regla-tarea conocido antes del v1: el prompt pide serif y ninguna fuente permitida lo es.
- **El check de g2-154 está roto.** Exige la clave `findings`, el prompt no nombra ninguna clave, y las dos réplicas del v1 cumplieron la regla usando `problems`.

## Diseño

### Brazos y corridas

| brazo | qué recibe | tareas | k | corridas |
|---|---|---|---|---|
| `arp` | `a0` + framing y regla en el system prompt (append) | 10 del suelo | 2 | 20 |
| `a0` | nada (restricciones de K, fresco) | las mismas 10 | 2 | 20 |
| `arp` | ídem | 6 de control | 2 | 12 |

- **Suelo (10):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97`. Son las 11 del v1 menos g2-171.
- **Control (6):** los mismos del v1: `g0-159 g1-144 g1-16 g1-25 g1-34 g2-35`.
- Son 52 corridas, con un coste estimado de ~5 USD y un tope de 15 USD.

### Canal: system prompt vía append

- Todos los brazos de K ya pasan `--append-system-prompt-file "$P/claude-md.md"`. En `arp`, `correr.sh` escribe `$O/sysprompt.md`, que es `claude-md.md` seguido de una línea en blanco y el **framing sellado** con la regla literal sustituida. Pasa ese fichero en lugar de `claude-md.md`.
- La regla viene de `K_REGLA_FILE`. Si falta o está vacío, `correr.sh` sale con exit 2, igual que `ar` en el v1.
- No hay hook ni `--plugin-dir`. `a0` sigue pasando exactamente `claude-md.md`.
- **Si pasa,** la 2d se podría construir como mod (`prompt.compose`). Esa decisión de implementación queda para su propio ciclo.

### Framing sellado

```
## Reglas duras del proyecto
Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.

- <regla literal de evals/techo-reglas/reglas/<id>.txt>
```

- Respecto a la preview elegida, se quita `<repo>` de la cabecera. Las tareas sintéticas (`setup:true`) no tienen repo con nombre, y un texto idéntico en todas las tareas elimina una variable.
- La regla se copia byte a byte de la del v1. No se reescribe ninguna.

### Cambios cerrados respecto al v1

Solo estos cinco (los dos últimos los añadió la auditoría del plan el 2026-10-06, decididos por Paul). Todo lo demás es idéntico: modelo `claude-sonnet-5-5`, flags, `--max-turns 40`, `--max-budget-usd 10` por corrida, entorno reconstruido, cero re-intentos, breakers y criterio "cumple".

1. **Canal y framing:** framing y regla en el system prompt (append tras `claude-md.md`), en lugar de SessionStart `additionalContext`.
2. **Errata del gold en g2-154:** el check acepta una lista de objetos `{type, path, detail}` bajo **cualquier** clave de la shape `--detailed`, en vez de exigir `findings`. No se ajusta a la clave que eligió el agente: tampoco exige `problems`. Va en `evals/techo-reglas-2/erratas-gold.md` con el diff, y el check corregido se aplica a los dos brazos.
3. **g2-171 queda fuera** por conflicto conocido a priori.
4. **Errata hermética de los helpers del gold.** 14 checks del gold tienen cableada la ruta `.worktrees/campana-k/evals/ablacion-k/harness`. Ese worktree se borró después del v1, y entre ellos están g0-122, g0-33 y g2-35 (control). Hoy darían rc=2: g2-35 caería con certeza y g0-33 sería infalsable. En esos 3 checks, la ruta pasa a ser relativa al propio gold (`gold/harness/`), donde se copian `herramientas.sh` y `comandos.sh` del harness de `main`. No cambia su lógica. Va a `erratas-gold.md` con un test que reproduce el rc del v1 sobre los transcripts del v1.
5. **Control con k=2.** «Caída» = un control con `check.rc ≠ 0` en **sus 2 réplicas**, simétrico a «cumple» (≥1/2). Con k=1, P(0/6 caídas) ≈ 0,74 si cada control pasa el 95 % de las veces; con k=2 sube a ≈ 0,98. Cuesta 6 corridas más.

### Gate

- **PASA ⇔ `arp ≥ 6/10` ∧ `arp − a0 ≥ 3` tareas ∧ `0/6` caídas.**
- "Cumple": `check.rc == 0` en ≥1 de 2 réplicas. "Caída": un control con `check.rc ≠ 0` en sus 2 réplicas. Una tarea no reconstruible cuenta como no cumple o caída.
- **Potencia** (auditoría, n=10, k=2):
  - si la regla es inerte, P(PASA) ≤ 0,12: es un gate seguro contra el azar;
  - con `arp` en q=0,6-0,7 y `a0` bajo, P(PASA) ≈ 0,6-0,8;
  - si `a0` fresco regresa a q≈0,5, P(PASA) baja a 0,23-0,39.
  
  Está infrapotenciado para efectos moderados: un NO PASA no refuta un efecto pequeño.
- **Breaker a mitad de tanda:** la tanda es **inválida y no se adjudica**. La causa va a `erratas.md`. Una tanda nueva desde cero no cuenta como tercera bala, porque la segunda no llegó a medir.
- **Limitaciones declaradas:** no hay brazo placebo (framing sin regla), así que no se separa «autoridad» de «regla en el system prompt». La pregunta de la spec tampoco lo exige.
- **Por qué un margen de 3:** con k=2 y n=10, una diferencia de 1-2 tareas cabe en el ruido entre réplicas del v1 (g0-149 y g1-140 dieron 0/1). Es un umbral elegido, no derivado de un cálculo de potencia.
- **Diagnósticos pre-declarados, que no adjudican:**
  - ¿voltea g0-33?
  - el reparto de cumplimiento entre reglas mecánicas y reglas de principio o proceso;
  - las corridas en las que el agente reporta conflicto en vez de cumplir.
- **Si no pasa, el frente se cierra sin tercera bala.** La KB registra "las reglas de proyecto no se entregan por ningún canal razonable", y la lectura útil es escribir reglas mecánicas.
- **Si pasa,** se abre el ciclo de la 2d. Si se construye como mod, queda condicionado a la sonda "¿cargan los mods en W11?" (backlog) antes de desplegarse en el trabajo.

### Harness

- **Se reutiliza el del v1. Se extiende, no se copia.**
  - `correr.sh` gana el brazo `arp`: restricciones de `a0` y `$O/sysprompt.md` en vez de `claude-md.md`. `a0` y `ar` quedan byte a byte iguales.
  - `fugas.py` conoce `arp`, con las mismas reglas que `a0`: ningún hook cableado y snapshot y `exo` vetados.
  - `correr-techo.sh` y `evaluar.py` se parametrizan por directorio de experimento y por brazos. El gate del v1 sigue saliendo igual sobre sus datos.
- **Orden:** `random.Random(20261006).shuffle` sobre `sorted` de las tuplas `(brazo, id, rep)`, con `rep` entero: 52 pares, brazos intercalados. Es la cola de `-P 4`.
- **Sonda pre-tanda:** `correr.sh arp` en modo ensayo, con una regla-codeword, genera un `sysprompt.md`. Una corrida corta con ese fichero devuelve el codeword y una con `claude-md.md` solo no lo devuelve. Las dos van antes de la primera corrida, con `--tools ""` para que el agente no pueda leer el codeword del disco ni a través de un subagente, y sobre una tarea fuera del experimento (g1-57), porque `correr.sh` hace `rm -rf` de su directorio de corrida.

### Integridad (lecciones del v1)

- **El pre-registro va commiteado antes de la primera corrida.** Contiene `claude --version`, los sha de las reglas y del framing, el del check corregido de g2-154, el orden y la política de cero re-intentos. Su validador falla si falta cualquier cláusula.
- **Orden de preparación, que es obligatorio:**
  1. `reconstruir.sh`;
  2. aplicar las erratas del gold: el check de g2-154 y los 3 checks con helpers más `gold/harness/`;
  3. pinnear en `pins.sha256` los sha de los 16 checks, los helpers y `prep/claude-md.md`, y declararlo en `preregistro.md` («gold = tarball salvo las erratas de `erratas-gold.md`»);
  4. `chmod -R a-w` sobre `gold/`;
  5. commit del pre-registro;
  6. sonda;
  7. tanda.
  
  Si `reconstruir.sh` vuelve a correr después de la errata, la pisa en silencio al reextraer del tar. Por eso `correr-techo.sh` verifica el sha pinneado del check de g2-154 antes de lanzar, y si no coincide sale con exit 2.
- **Evidencia de solo lectura:** antes de despachar cualquier subagente sobre los datos, `chmod -R a-w` sobre `gold/` y sobre las corridas del experimento. Es la errata E1 del v1.
- **Adjudicador fresco:** la clase de cada no-cumple la asigna un subagente que no diseñó el framing. Las clases son cinco y cerradas: conflicto regla-tarea, check roto, regla mal escrita, no reconstruible e incumplimiento del agente.

## Testing

- Los tests del v1 siguen en verde: `test-correr-ar`, `test-correr-techo`, `test_evaluar`, `test-reconstruir`, `validar-preregistro`.
- **Tests nuevos:**
  - `arp` genera `sysprompt.md` = `claude-md.md` + línea en blanco + framing con la regla, byte a byte, incluidas comillas, `\` y saltos de línea. Sin `K_REGLA_FILE` sale con exit 2.
  - El settings y la cmdline de `a0` no cambian (pasa `claude-md.md`).
  - `correr-techo.sh` sale con exit 2 si el sha del check de g2-154 no es el pinneado.
  - En `evaluar.py`, el margen: `a0=4, arp=6` no pasa; `a0=3, arp=6` pasa; `arp=6, a0=3` con 1 caída no pasa.
  - El check corregido de g2-154 acepta la shape detallada bajo cualquier clave. Contra el `diff.patch` real de `a0` de K (`corridas/g2-154/a0-r1/` del tarball), sigue rechazando la migración en sitio.
- `validar-preregistro.sh` del experimento nuevo.

## Fuera de alcance

- Diseñar la 2d: es su propio ciclo, y solo si esto pasa.
- La sonda de mods en W11: es un ítem aparte del backlog.
- Reescribir reglas o checks más allá de la errata de g2-154.
