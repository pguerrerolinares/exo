---
name: debug
description: Dos puertas — (1) ante cualquier bug, test que falla o comportamiento inesperado, antes de proponer fixes; (2) cuando estás atascado — mismo error ≥3 veces, terreno desconocido, time-box quemándose — o antes de grindear en solitario algo no familiar. Root cause y recon antes de computar.
---

# debug

Root cause antes de cualquier fix — un fix de síntoma es un fallo, no una
solución.

## Puerta 1 · Root cause (las 4 fases)

Fase 1: lee los errores completos; reproduce consistentemente (si no,
junta más datos, no adivines); revisa cambios recientes; en sistemas
multi-componente instrumenta cada boundary y corre UNA vez para ver DÓNDE
rompe; traza el data flow hacia atrás hasta el origen — fix en la fuente,
no en el síntoma. Fase 2: localiza código similar que SÍ funciona, lee la
referencia COMPLETA, lista todas las diferencias sin descartar "eso no
puede importar", entiende sus dependencias. Fase 3: hipótesis única y
explícita ("creo que X porque Y"), test mínimo de una variable; si falla
⇒ NUEVA hipótesis, no apiles fixes — "no entiendo X" es más honesto que
fingir. Fase 4: failing test que reproduce el bug ANTES del fix
(`exo:tdd`), UN fix al root cause (sin "ya que estoy aquí"), verifica
que resuelve y que nada más se rompe.

3+ fixes fallidos ⇒ para y cuestiona la arquitectura (cada fix revela
acoplamiento nuevo en otro sitio: es un patrón, no una hipótesis fallida)
— discútelo con el humano antes de más fixes. "No root cause" verdadero
(ambiental/timing/externo): documenta lo investigado, implementa el
handling apropiado, añade monitoring — el 95% de los "no root cause" son
investigación incompleta. Técnicas de soporte (root-cause-tracing,
defense-in-depth, condition-based-waiting): `techniques.md`.

## Puerta 2 · Atascado o pre-grind

Gate: ≥3 intentos contra el mismo error, terreno desconocido, o time-box
quemándose sin señal — con priors sólidos y avanzando, no te frenes por
ritual. Anti-patrón: cambiar una variable y reintentar a ciegas ante el
mismo error — repetir no es progreso.

1. Para de reintentar; nombra explícitamente qué intentaste y por qué
   falló — si no sabes por qué, ese es el problema.
2. Retrieve > compute: error literal a la web, docs oficiales, issues/
   changelog si huele a cambio de versión.
3. Lista tus supuestos y verifica el más barato primero.
4. Reduce el caso al mínimo aislado antes de seguir tocando el sistema
   entero.

Investigación voluminosa ⇒ delega a un subagente barato; quédate con la
conclusión, no con el material crudo.
