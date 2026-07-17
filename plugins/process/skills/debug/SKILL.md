---
name: debug
description: Dos puertas — (1) ante cualquier bug, test que falla o comportamiento inesperado, antes de proponer fixes; (2) cuando estás atascado — mismo error ≥3 veces, terreno desconocido, time-box quemándose — o antes de grindear en solitario algo no familiar. Root cause y recon antes de computar.
---

# debug

Root cause antes de cualquier fix — un fix de síntoma es un fallo, no una
solución.

## Fase 1 · Root cause

- Lee los errores completos: stack trace entero, líneas, paths, códigos.
- Reproduce consistentemente; si no es reproducible, junta más datos, no
  adivines.
- Revisa cambios recientes: git diff, commits, deps, config, entorno.
- Sistemas multi-componente: instrumenta cada boundary (qué entra/qué
  sale/config propagada) y corre UNA vez para ver DÓNDE rompe, antes de
  proponer fixes.
- Traza el data flow hacia atrás hasta el origen del valor malo — fix en
  la fuente, no en el síntoma.

## Fase 2 · Patrón

Localiza código similar que SÍ funciona, lee la referencia COMPLETA, lista
todas las diferencias sin descartar "eso no puede importar", entiende las
dependencias (qué otros componentes necesita, qué config/entorno, qué
asume).

## Fase 3 · Hipótesis

Hipótesis única y explícita ("creo que X porque Y"), test mínimo de una
variable, verifica antes de seguir. Si falla ⇒ NUEVA hipótesis, no apiles
fixes. "No entiendo X" es más honesto que fingir que sabes.

## Fase 4 · Fix

Failing test que reproduce el bug ANTES del fix (`process:tdd`), UN fix al
root cause (sin "ya que estoy aquí"), verifica que resuelve y que nada más
se rompe.

## 3+ fixes fallidos

Para y cuestiona la arquitectura — cada fix revelando acoplamiento nuevo en
otro sitio es un patrón, no una hipótesis fallida. Discute con el humano
antes de más fixes.

## "No root cause"

Si es verdaderamente ambiental/timing/externo: documenta lo investigado,
implementa el handling apropiado (retry/timeout/mensaje), añade
monitoring. El 95% de los "no root cause" son investigación incompleta.

## Técnicas de soporte

`techniques.md`: root-cause-tracing (trazar hacia atrás hasta el trigger
original), defense-in-depth (validar en cada capa tras hallar la causa),
condition-based-waiting (esperar la condición, no un sleep arbitrario).

## Puerta 2 · Atascado o pre-grind

Gate de dificultad: ≥3 intentos contra el mismo error, terreno desconocido,
o time-box quemándose sin señal. Con priors sólidos y avanzando, no te
frenes por ritual.

Anti-patrón que ataca: cambiar una variable y reintentar a ciegas ante el
mismo error — repetir no es progreso.

1. Para de reintentar. Nombra explícitamente qué intentaste y por qué
   falló; si no sabes por qué, ese es el problema a resolver.
2. Retrieve > compute: error literal a la web, docs oficiales (mejor que
   tu memoria paramétrica, que puede estar stale), issues/changelog si
   huele a cambio de versión.
3. Lista tus supuestos y verifica el más barato primero — un
   `--version`/`print`/`ls` resuelve más atascos que otra ronda de
   razonamiento.
4. Reduce el caso al mínimo aislado antes de seguir tocando el sistema
   entero.

Investigación voluminosa (varias búsquedas, docs largas) ⇒ delega a un
subagente barato; quédate con la conclusión, no con el material crudo.
