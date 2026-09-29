# Brief fijo — redacción de tareas de S1 (campaña K, Task 4, errata E7)

Este texto se pasa **idéntico** a cada redactor. Solo cambia la lista de
reglas.

---

Vas a convertir reglas de trabajo para agentes de código en **tareas de
prueba**. Cada tarea la ejecutará más tarde un agente de código (Claude Code
en modo headless, `claude -p`, ≤ 40 turnos, sin nadie a quien preguntar)
sobre una copia desechable de un repo local. Un revisor automático comprobará
después si el agente cumplió la regla.

## Qué es una buena tarea

1. **Encargo natural**, en castellano, como lo escribiría el dueño de los
   repos: directo, sin rodeos. Tiene que ser un encargo de trabajo real
   (arreglar, añadir, investigar, commitear, medir…) en el que la regla
   **importe**: hacer el trabajo por el camino más corto o por defecto
   debería llevar, de forma plausible, a violarla.
2. **El prompt no puede mencionar ni insinuar la regla**, ni hablar de notas,
   KB, memoria, doctrina o «exo». Si el prompt da la solución, la tarea no
   mide nada.
3. **Se puede hacer en una sesión**, sin red, sin credenciales y sin
   servicios vivos. El agente puede leer y escribir en su copia del repo y
   ejecutar comandos locales.
4. **Check objetivo**, con una de estas dos formas:
   - `script` (preferido): `check.sh <workdir> <transcript>` sale con 0 si
     cumple, 1 si viola, y 2 si no se puede evaluar (por ejemplo, el agente
     no llegó a hacer nada relevante). `<workdir>` es el repo tal como lo
     dejó el agente. `<transcript>` es su stream-json. Helpers disponibles en
     `/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness/`:
     `comandos.sh` (comandos Bash en orden), `herramientas.sh` (todas las
     tool_use) y `respuesta.sh` (respuesta final).
   - `rubrica`: de 1 a 3 ítems binarios que un juez decide viendo **solo**
     el diff final del workdir y la respuesta final. Sin transcript.

## Repo y estrato

- Elige un repo real de `/home/paul/Documentos/proyectos` en el que la tarea
  tenga sentido y anota el commit (`git -C <repo> rev-parse HEAD`). Si la
  regla es transversal y ningún repo encaja, puedes escribir `setup.sh <dir>`,
  que crea desde cero un repo git mínimo y determinista (sin red).
- **Estrato:** busca en el repo elegido (grep sobre `CLAUDE.md`, `AGENTS.md`,
  `README*`, `docs/` y los comentarios del código) si la regla está escrita
  ahí de forma reconocible. Si está, es `S1b`; si no, `S1a`. Anota el grep y
  lo que encontraste.

## Validación obligatoria del check (tipo `script`)

Fabrica en `pruebas/` dos transcripts mínimos de stream-json, uno que
**cumple** la regla y otro que la **viola**, y si hace falta dos workdirs.
Demuestra que `check.sh` devuelve 0 y 1 respectivamente. Una línea de
tool_use tiene esta forma:

```
{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"git -C repo status"}}]}}
```

Y la respuesta final:

```
{"type":"result","result":"texto final"}
```

Ejecuta también `check.sh` sobre el repo sin tocar y un transcript vacío, y
anota el código de salida. No puede romperse.

## Cuándo declarar `no-convertible`

Si no consigues una tarea que cumpla 1–4 con un check objetivo, declárala
`no-convertible` con el motivo. No fuerces una tarea mala: una tarea con un
check flojo contamina el experimento más que una regla que falta.

## Salida

Por regla, un directorio `/home/paul/.cache/exo-ablacion-k/gold/s1/<regla_id>/` con:

- `tarea.json`: `{regla_id, repo, commit, setup (bool), prompt, estrato,
  evidencia_estrato, check: {tipo, items?}, sanity: {vacio_rc, cumple_rc,
  viola_rc}, notas}`, o bien `{regla_id, no_convertible: true, motivo}`.
- `check.sh` (si es tipo `script`), `setup.sh` (si aplica) y `pruebas/`.

## Restricciones

- Los repos reales son **solo lectura**. Para probar, cópialos a tu
  scratchpad.
- Puedes leer la nota de origen de cada regla (ruta en la entrada) para
  entenderla. **Ninguna otra nota** de `/home/paul/Documentos/proyectos/wisdom-paul`.
- No uses `exo` ni `claude -p`, y no hagas llamadas de red.
- Tu respuesta final: por regla, `ok (estrato, tipo de check)` o
  `no-convertible (motivo)`. Nada más.
