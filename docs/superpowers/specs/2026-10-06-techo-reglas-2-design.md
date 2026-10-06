# techo-reglas-2: la regla de proyecto con autoridad de system prompt

**Estado:** diseño aprobado por Paul el 2026-10-06. Es la última bala del frente "reglas de proyecto" (la 2d).
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
- **El 0/11 de K estaba forzado por la selección.** El suelo son justo las tareas en las que A0 falló. Por eso hace falta un brazo `a0` **fresco** en las mismas tareas. Además absorbe la deriva de versión: 2.1.285 en K, 2.1.286 en el v1, 2.1.291 hoy.
- g2-171 es un conflicto regla-tarea conocido antes del v1: el prompt pide serif y ninguna fuente permitida lo es.
- **El check de g2-154 está roto.** Exige la clave `findings`, el prompt no nombra ninguna clave, y las dos réplicas del v1 cumplieron la regla usando `problems`.

## Diseño

### Brazos y corridas

| brazo | qué recibe | tareas | k | corridas |
|---|---|---|---|---|
| `arp` | `a0` + mod con la sección de regla en el system prompt | 10 del suelo | 2 | 20 |
| `a0` | nada (restricciones de K, fresco) | las mismas 10 | 2 | 20 |
| `arp` | ídem | 6 de control | 1 | 6 |

- **Suelo (10):** `g0-122 g0-149 g0-33 g1-131 g1-139 g1-140 g1-157 g2-154 g2-170 g2-97`. Son las 11 del v1 menos g2-171.
- **Control (6):** los mismos del v1: `g0-159 g1-144 g1-16 g1-25 g1-34 g2-35`.
- Son 46 corridas, con un coste estimado de ~4,5 USD y un tope de 15 USD.

### Canal: el mod de experimento

- Vive en `evals/techo-reglas-2/mod/`, con la estructura estándar (`.claude-plugin/plugin.json`, `hooks/hooks.json`, `hooks/register.ts`).
- En `prompt.compose` llama a `next(e)` y añade **al final** una sección con id `techo:regla` y `scope: "session"`.
- El mod no inyecta nada más ni toca otros eventos.
- El texto de la sección es el **framing sellado** con la regla literal sustituida.
- La regla llega al mod por un fichero que `correr.sh` escribe en el directorio de la corrida, `$O/regla.txt`. El mod la lee con `$.fs`. Si el fichero falta o está vacío, el mod **no** añade la sección y deja constancia en un fichero de la corrida, que el detector de fugas lee: falla visible, nunca un `a0` disfrazado. El mecanismo concreto de paso (fichero, `userConfig` o env) lo fija el plan tras leer `reference.md`; el contrato es "la regla es exactamente la del fichero y una ausencia es visible".

### Framing sellado

```
## Reglas duras del proyecto
Estas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.

- <regla literal de evals/techo-reglas/reglas/<id>.txt>
```

- Respecto a la preview elegida, se quita `<repo>` de la cabecera. Las tareas sintéticas (`setup:true`) no tienen repo con nombre, y un texto idéntico en todas las tareas elimina una variable.
- La regla se copia byte a byte de la del v1. No se reescribe ninguna.

### Cambios cerrados respecto al v1

Solo estos tres. Todo lo demás es idéntico: modelo `claude-sonnet-5-5`, flags, `--max-turns 40`, `--max-budget-usd 10` por corrida, entorno reconstruido, cero re-intentos, breakers y criterio "cumple".

1. **Canal y framing:** la sección del system prompt vía mod, en lugar de SessionStart `additionalContext`.
2. **Errata del gold en g2-154:** el check acepta una lista de objetos `{type, path, detail}` bajo **cualquier** clave de la shape `--detailed`, en vez de exigir `findings`. No se ajusta a la clave que eligió el agente: tampoco exige `problems`. Va en `evals/techo-reglas-2/erratas-gold.md` con el diff, y el check corregido se aplica a los dos brazos.
3. **g2-171 queda fuera** por conflicto conocido a priori.

### Gate

- **PASA ⇔ `arp ≥ 6/10` ∧ `arp − a0 ≥ 3` tareas ∧ `0/6` caídas.**
- "Cumple": `check.rc == 0` en ≥1 de 2 réplicas. "Caída": un control con `check.rc ≠ 0`. Una tarea no reconstruible cuenta como no cumple o caída.
- **Por qué un margen de 3:** con k=2 y n=10, una diferencia de 1-2 tareas cabe en el ruido entre réplicas del v1 (g0-149 y g1-140 dieron 0/1). Es un umbral elegido, no derivado de un cálculo de potencia.
- **Diagnósticos pre-declarados, que no adjudican:**
  - ¿voltea g0-33?
  - el reparto de cumplimiento entre reglas mecánicas y reglas de principio o proceso;
  - las corridas en las que el agente reporta conflicto en vez de cumplir.
- **Si no pasa, el frente se cierra sin tercera bala.** La KB registra "las reglas de proyecto no se entregan por ningún canal razonable", y la lectura útil es escribir reglas mecánicas.
- **Si pasa,** se abre el ciclo de la 2d como mod. Queda condicionado a la sonda "¿cargan los mods en W11?" (backlog) antes de desplegarse en el trabajo.

### Harness

- **Se reutiliza el del v1. Se extiende, no se copia.**
  - `correr.sh` gana el brazo `arp`: restricciones de `a0`, más `--plugin-dir evals/techo-reglas-2/mod`, más la regla de `K_REGLA_FILE`. `a0` y `ar` quedan byte a byte iguales.
  - `fugas.py` conoce `arp`: no hay ningún hook bash cableado. Si el mod aparece en el stream-json, solo se acepta como evento suyo; el plan lo verifica con la sonda. Además se comprueba que la sección llegó, mediante el fichero de constancia del mod.
  - `correr-techo.sh` y `evaluar.py` se parametrizan por directorio de experimento y por brazos. El gate del v1 sigue saliendo igual sobre sus datos.
- **Orden:** barajado de los 46 pares (brazo, id, rep) con semilla `20261006`, ambos brazos intercalados. Es la cola de `-P 4`.
- **Sonda pre-tanda:** el mod del experimento con una regla-codeword debe devolverla, y el control sin mod no. Ambas cosas antes de la primera corrida.

### Integridad (lecciones del v1)

- **El pre-registro va commiteado antes de la primera corrida.** Contiene `claude --version`, los sha de las reglas, del framing y del mod, el orden y la política de cero re-intentos. Su validador falla si falta cualquier cláusula.
- **Evidencia de solo lectura:** antes de despachar cualquier subagente sobre los datos, `chmod -R a-w` sobre `gold/` y sobre las corridas del experimento. Es la errata E1 del v1.
- **Adjudicador fresco:** la clase de cada no-cumple la asigna un subagente que no diseñó el framing. Las clases son cinco y cerradas: conflicto regla-tarea, check roto, regla mal escrita, no reconstruible e incumplimiento del agente.

## Testing

- Los tests del v1 siguen en verde: `test-correr-ar`, `test-correr-techo`, `test_evaluar`, `test-reconstruir`, `validar-preregistro`.
- **Tests nuevos:**
  - `arp` genera `regla.txt` idéntico a la regla y pasa `--plugin-dir`. Sin `K_REGLA_FILE` sale con exit 2.
  - El settings y la cmdline de `a0` no cambian.
  - El mod, con una regla de prueba, añade exactamente una sección al final, con el texto sellado. Sin regla no añade nada y deja constancia (`claude plugin test` o equivalente; lo fija el plan).
  - En `evaluar.py`, el margen: `a0=4, arp=6` no pasa; `a0=3, arp=6` pasa; `arp=6, a0=3` con 1 caída no pasa.
  - El check corregido de g2-154 acepta la shape detallada bajo cualquier clave y sigue rechazando la migración en sitio del default.
- `validar-preregistro.sh` del experimento nuevo.

## Fuera de alcance

- Diseñar la 2d: es su propio ciclo, y solo si esto pasa.
- La sonda de mods en W11: es un ítem aparte del backlog.
- Reescribir reglas o checks más allá de la errata de g2-154.
