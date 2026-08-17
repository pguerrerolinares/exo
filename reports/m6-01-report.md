# Reporte — M6-01: índice fresco sin daemon

Campaña C6, primer item. Implementado por el ORQUESTADOR (no por un executor:
en C5 dos de tres subagentes dejaron de responder a mitad y hubo que rematar
igual; item pequeño, se declara para que conste).

Rama `m6-01`, base `3d7f073`.

## El problema que resuelve

basic-memory mantenía su índice al día con un **watch** en segundo plano. exo
indexa **al invocar**, sin daemon (spec §4.2: "incremental por mtime/git al
invocar, sin daemon salvo que duela"). Sin nada que refresque, el hook de
recall de M6 serviría un bloque de una KB rancia — y lo haría en silencio, que
es la peor forma de fallar.

## Lo implementado

- `exo::refresca_indice(kb, db)` — contrato nombrado y testeado aparte del CLI.
  Es `indexer::indexa` sin adornos; existe como función propia para poder
  probar el comportamiento sin pasar por el binario.
- `exo recall --refresca` — refresca antes de servir. El resumen va a
  **stderr** (stdout es exclusivo del envelope/bloque, contrato §4) y solo se
  imprime si de verdad cambió algo.
- 4 tests nuevos (`engine/tests/refresca.rs`), TDD estricto rojo→verde:
  - `recall_sin_refrescar_sirve_indice_rancio` — documenta el fallo que
    justifica el item: sin refresco, una nota añadida tras indexar NO aparece.
  - `refresca_indice_antes_de_servir_incluye_la_nota_nueva` — y el incremental
    de verdad lo es: `indexadas=1, saltadas=1`.
  - `refresca_sin_cambios_no_reindexa_nada`.
  - `refresca_crea_el_indice_si_no_existe` — bootstrap de máquina limpia.

Suite completa: **90/90 verdes** (86 previos + 4).

## Medición en vivo (KB real, 138 notas)

| Caso | Tiempo | Comentario |
|---|---|---|
| `recall` sin `--refresca` | 14 ms (p95, M2-08) | referencia |
| `--refresca`, **nada que reindexar** | **21-25 ms** | el caso normal del hook |
| `--refresca`, **1 nota modificada** | **3,4 s** | carga del modelo ONNX + embed de sus trozos |
| `--refresca`, **todos los mtimes nuevos** | **>10 min** | reindexado completo (138 notas) |

## Decisión de diseño que estos números imponen a M6-02 (cutover del hook)

**El hook de SessionStart NO debe llamar a `--refresca` a ciegas.** Con el
presupuesto de arranque en <100 ms, una sola nota editada desde la sesión
anterior lo multiplica por 34 (3,4 s), y el peor caso —mtimes nuevos, p. ej.
tras un `git clone` fresco de la KB— bloquearía el arranque más de diez
minutos. Eso no es una latencia: es un cuelgue.

Reparto correcto, y así entra en el brief de M6-02:

- **SessionStart** → `exo recall` **sin** `--refresca`: 14 ms, sirve lo que
  haya en el índice.
- **Stop** (cierre de sesión) → `exo index`: ahí 3,4 s no molestan a nadie, y
  es justo cuando la KB acaba de cambiar (`/documenta` escribe al cerrar).
- `--refresca` queda como **red de seguridad manual y bootstrap**, no como
  camino por defecto del hot path.

Ese reparto tiene además la propiedad de que el índice está fresco *antes* del
siguiente arranque, que es cuando hace falta, en vez de pagarlo en el momento
en que el usuario espera.

## Cómo se descubrió el peor caso (declarado, porque fue un error)

Al medir el caso "1 nota modificada" copié la KB con `cp -r`, que **no
preserva mtimes**: las 138 notas parecieron modificadas y el refresco entró en
un reindexado completo que se comió un timeout de 10 minutos. El error de
método dio el dato del peor caso, que es el que motiva la decisión de arriba.
La medición buena se repitió con `cp -a`.

## Lo que NO entra en este item

El cutover en sí (M6-02..05: reapuntar el hook de recall, reescribir el
FALLBACK embebido, mover reflex al monorepo, repuntar kbx, cutover de
doctrina) **espera OK explícito de Paul**: cambia el entorno vivo de sus
sesiones, y el config §Línea roja nombra guards y settings como no delegables.
