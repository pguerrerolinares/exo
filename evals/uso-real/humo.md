# Humo de la medición en uso real (Task 9)

Fecha: 2026-10-26. Cache del humo separada de la corrida real (`<cache>/humo`); no deja veredictos en la cache real.

## Etapa 1 (detectar.py, Haiku)

- Sesión: `6ee89a81-4ccf-4176-8214-f01c65cd9378` (10 mensajes humanos, sesión de `exo`).
- Flags: `--tope-usd 0.15 --solo-sesion <id>`, sin `--sello`.
- rc = 0; coste = 0,00285 USD; 1 lote.
- Los 10 ids volvieron con esquema válido (10 de 10, 0 `sin_veredicto`): 10 `no`, 0 `si`.

## Etapa 2 (atribuir.py, Sonnet)

- Una detección `si` sintetizada en la cache del humo (tipo `rechaza-propuesta`) sobre el mensaje `2c937a71-fa88-4648-aad9-e7906ef71a01` de la misma sesión.
- El texto del mensaje es inventado: el transcript se copió a un directorio temporal con ese único mensaje sustituido; sesión y commit de KB son reales.
- rc = 0; coste = 0,07334 USD; 1 llamada.
- Esquema válido: `atribucion = nueva`, `nota_ausente = false`.
- `commit_kb = 552f06af6ce83895c644989fe7af4be643e7e7ab`, distinto de HEAD de la KB (`36c94c4b03b69e78fdd02ca7d017c34ffb154c5a`).

Coste real total: 0,07619 USD (límite 0,70).

## Notas de proyecto (kb.nota_head + kb.ruta_en), 99 sesiones

- Resuelven nota con texto: 83
- Sin nota de proyecto en HEAD (`exo rules` no devuelve nota para el cwd): 16
- Con nota en HEAD pero sin ruta en el commit de la sesión (`ruta_en` falla, posible pérdida de renombre con `--follow`): 0
- Sin commit de KB anterior al inicio, sin cwd, o ruta sin texto: 0

El minor diferido (`--follow` pierde renombres con edición grande) no se manifiesta en esta población. Los 16 sin nota en HEAD no son fallo de `ruta_en`.

## Sello

`sellar.py` → rc 0, pero `n_sesiones = 99`, fuera del rango esperado 100-115 (106 al escribir la spec). No se fuerza ni se ajusta `--desde`; decisión pendiente del dueño. `sello.json` queda en disco sin commitear.

## Qué NO se vio correr

- Etapa 1 completa (99 sesiones) y etapa 2 completa: son la medición.
- Ninguna detección `si` real en el humo (10 de 10 `no`), así que el camino de reintento por ids faltantes y el tope de coste (exit 3) no se ejercitaron en real.
- La rama `sin_veredicto` y el exit 4 (fallo de proceso) no se ejercitaron en real.
- Detectar/atribuir con `--sello` (el humo fue sin sello).
