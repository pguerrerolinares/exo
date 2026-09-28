# Campaña K — erratas al pre-registro congelado

Aprobadas por Paul el 2026-09-29, tras la auditoría
`evals/ablacion-k/auditoria-f0-t0.md`. Se registran **antes** de extraer el
pool (Task 2) y **antes** de cualquier corrida de la fase 1. Ninguna se
decidió viendo resultados de la ablación, porque todavía no hay ninguno.
El verdict final las cita.

## E1 — D3: el CLAUDE.md de los brazos va sin la sección de memoria (B1)

**Problema.** La línea 19 de `~/.claude/CLAUDE.md` ordena usar «SIEMPRE»
`exo search` y la KB `wisdom-paul`. Con D3 tal como estaba, A0 recibe la
orden de usar exo. Pasa una de dos cosas: si la herramienta está disponible
hay fuga, y si no, A0 queda penalizado de forma artificial (turnos perdidos
buscando algo que no existe).

**Errata.** En los cuatro brazos se inyecta el CLAUDE.md global **sin la
sección `## Memoria de sesiones`**, porque sin exo esa sección no
existiría. El resto del fichero (perfil, estilo, git) sigue igual en todos
los brazos, así que el sentido de D3 no cambia: la pregunta sigue siendo si
exo aporta por encima de la memoria nativa. Las instrucciones de memoria
llegan solo por el core-index (A1–A3). El recorte lo hace el harness en cada
corrida y se registra el sha256 del resultado.

**Aislamiento añadido en A0:** `--disallowedTools` sobre lecturas de la ruta
de la KB y de `~/.exo/`, y `exo` fuera de alcance (stub, como A1).

## E2 — §9: la fuga se detecta por canal y por ruta, no por texto

Sustituye el canario literal de §9, que da falsos positivos
(`evals/ablacion-k/task0-recon.md`). Cuenta como fuga en un brazo que no
debería tenerla cualquiera de estos casos:

- eventos de hook (`hook_*`, `additionalContext`) fuera de lo que el brazo
  cablea;
- un `tool_use` que invoca `exo` (A0/A1), también mediante wrappers o rutas
  absolutas;
- `recall-inject-emitted` en el log del brazo (A0/A1/A2);
- **(añadido por B1)** un `tool_use` cuya entrada contiene la ruta de la KB
  o de `~/.exo/` (A0).

Las apariciones de la cadena `=== Recall exo` dentro de un `tool_result`
**no** cuentan.

## E3 — D2: parada solo por consumo y regla simétrica de corte (I5)

Sustituye las tandas «si el ritmo es sostenible» y el freno de
`--max-budget-usd 3` fijados en `06450e0`.

- Tandas de 40 corridas. **Única regla de parada entre tandas:** la media de
  tokens de entrada por corrida de la tanda supera **2M**. Se evalúa sin
  mirar ningún resultado: ningún éxito se computa hasta cerrar la etapa.
- Tope por corrida: `--max-budget-usd 10` (unidad de lista, usada como
  límite de consumo). **Una corrida cortada cuenta como fallo, en cualquier
  brazo**, y la tasa de cortes por brazo se reporta.
- Se mantiene el tope de etapa: 240M tokens de entrada.
