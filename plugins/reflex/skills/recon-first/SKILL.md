---
name: recon-first
description: Use when stuck on a hard problem, hitting the same error repeatedly, or before grinding solo on something unfamiliar/time-boxed. The "look before you leap" move — retrieve and verify assumptions before computing. Invoke directly when you notice you're spinning.
---

# recon-first

**El anti-patrón que esto ataca:** cambiar una variable y reintentar a ciegas ante el
mismo error. Repetir no es progreso. Cuando estás atascado, el desbloqueo casi nunca
es *insistir más fuerte* — es **parar a recoger información**.

> Validado en la práctica (agent-solve-it: la investigación online fue el desbloqueo en
> los 4 retos; el fallo recurrente era grindear en solitario antes de buscar) y en la
> literatura ("Look Before You Leap": explorar antes de actuar mata loops y triplica la
> auto-recuperación de errores, aunque suba poco el rendimiento bruto).

> Fuente canónica de esta doctrina: nota [[doctrina-agentes]] en la KB (proyecto `kb-demo`, engine `exo`). Si esta skill y la nota divergen, manda la nota; actualiza la skill.

## Cuándo aplica (gate de dificultad)

Esto NO es para toda tarea — explorar cuando ya tienes buenos priors *añade ruido* y
cuesta (tool-use tax). Aplica cuando:

- Llevas **≥3 intentos** contra el mismo error sin avanzar.
- El terreno es **desconocido** (API/librería/dominio que no dominas).
- La tarea es **time-boxed** y estás quemando presupuesto sin señal.

Si tienes priors sólidos y vas avanzando, sigue — no te frenes por ritual.

## Los movimientos (en orden)

1. **Para de reintentar lo mismo.** Nombra explícitamente qué has intentado ya y por
   qué falló. Si no sabes *por qué* falló, ese es el problema a resolver, no el siguiente
   intento.
2. **Busca el error / consulta la fuente.** Mensaje de error literal a la web; docs
   oficiales de la librería (preferible a tu memoria paramétrica, que puede estar stale);
   issues/changelog si huele a cambio de versión. **Retrieve > compute** en lo que no
   estuvo en tus pesos.
3. **Lista tus supuestos y verifica el más barato primero.** ¿Qué estás *asumiendo* que
   es verdad y no has comprobado? (la versión instalada, el path, el formato del input,
   que el servicio está arriba). Un `--version` / un `print` / un `ls` resuelve más
   atascos que otra ronda de razonamiento.
4. **Reduce el caso.** Reproduce el fallo en el mínimo aislado posible antes de seguir
   tocando el sistema entero.

## Delegación (orquestador limpio)

Si la investigación es voluminosa (varias búsquedas, leer docs largas), **delégala a un
subagente** (Explore para búsquedas/lecturas; un research-agent con modelo barato) y
quédate con la conclusión — no ensucies tu contexto con el material crudo.

## Si está disponible

Para un flujo de depuración riguroso, `process:debug` es un buen
complemento (hipótesis → experimento → confirmación). Úsalo si lo tienes; este skill no
depende de él.
