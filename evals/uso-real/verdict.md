# Veredicto: medición en uso real (paso a)

**Fecha:** 2026-10-27 · **Resultado:** no se estabiliza exo; pasa a (b), empezando por la delegación.
**Desviación declarada:** el juez de la etapa 1 no pasó la validación sellada. El veredicto sale de las etiquetas del dueño, no del pipeline `resultado.py`, y la clasificación por categoría es post-hoc. Abajo, qué se hizo y por qué.

## Población y etapa 1

- Sello: 99 sesiones `cli`, del 2026-10-06 a 2026-10-26T22:35:13Z; 914 mensajes humanos.
- Juez (Haiku, lotes de 10): 161 `si` (1,63/sesión), 0 `sin_veredicto`. Coste: 0,263 USD.
- Validación contra 80 etiquetas ciegas del dueño (`validacion.json`): kappa 0,275, precisión 0,60, recall 0,41 ⇒ **`juez_valido: false`** (umbral sellado 0,4). El juez se queda corto: no caza 17 de las 29 correcciones del dueño.
- La regex de candidatos casó solo 16 de 914 mensajes (la spec suponía unos 130), así que la muestra fue 16 regex + 64 al azar. El plan B sellado (el dueño etiqueta los candidatos regex) quedó ya cubierto dentro de las 80 y da una cota inferior vacua (0,08/sesión).

## Nivel total, con las etiquetas del dueño

- Estrato al azar: 21 de 64 son corrección (0,33; Wilson 95 % 0,23–0,45). Estrato regex: 8 de 16.
- Extrapolado: ~303 correcciones, **3,06/sesión** (suma de extremos de Wilson por estrato: 2,09–4,20; es conservador, no un IC). La spec estimaba 0,2–0,5 a ojo: falló por 6-10×.
- Consecuencia: con 0,33 correcciones por mensaje, la puerta de la etapa 1 (IC superior < 0,4) no podía cerrarse nunca. La decisión real era si la fracción de decididas está por debajo del 10 %.

## Correcciones en decisiones ya tomadas

- Clasificación manual de las 29 por un consultor (Fable), por lo que pide el dueño: 4 delegar a subagentes, 4 lanzar un consultor, 5 otra cosa ya escrita en la KB, 12 nuevas, 4 no eran corrección.
- La doctrina de delegación está en la KB desde el commit del 2026-08-23 (anterior a toda la población), fuera de cualquier `## Reglas duras`: cuenta como «decidida».
- Relectura del dueño, ciega a cuáles eran cuáles (8 + 4 distractores): confirma 7 de 8 como delegación y 0 de 4 distractores.
- Las 7 caen en el estrato al azar: 7/64 × 898 / 99 = **0,99 correcciones de delegación por sesión, IC 95 % Clopper-Pearson 0,41–1,93**. Solo con la delegación, la cota inferior supera el umbral de 0,4 del dueño. Margen mínimo: con 6/64 la cota baja a 0,32.
- Las 5 «otra cosa en la KB» no se suman: sin validar, y no hacen falta para la decisión.

## Por qué no se corrió la etapa 2

`atribuir.py` carga la nota de proyecto y `core/`, pero la doctrina de delegación vive en `learnings/` (fuera del contexto del atribuidor). Con su prompt, marcaría la delegación como `nueva` y empujaría hacia `unidad_vacia`: la respuesta contraria a los datos. Queda como defecto del instrumento si se vuelve a usar.

## Qué sigue

- (b), empezando por la delegación: es una regla de iniciación (falla por omisión, #193) y la doctrina vive en el cajón, no en el canal. Métrica de referencia para medir el efecto: 0,99 correcciones de delegación por sesión (0,41–1,93), a repetir tras unas 30 sesiones.
- No visto: repos y `prompt_snapshot` de las 7 sesiones; conteos deterministas de §4.5 (`resultado.py` no se corrió con juez inválido).
- Coste total de la medición: 0,34 USD más dos sesiones de etiquetado del dueño.
