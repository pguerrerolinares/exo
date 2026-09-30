# Verdict 2: línea base de mutation score, pipeline pre-A+

> Medido el 2026-09-29 con `linea-base-enmienda-1.md`, instrumento `026c166` (exo 1.5.1). Las corridas fueron secuenciales. Los informes por repo están en `linea-base-2/`.

## Resultado

| Repo | Rama | Mutantes (antes → ahora) | Resultado | Tiempo | ¿Entra en el agregado? |
|---|---|---|---|---|---|
| exo | campaña L | 13 (cargo, sin cambios) | **13/13 (100%)** | 26:52 (medición 1) | sí |
| pguerrero-music | `feat/portafolio-playlists` | 923 → **212** | **139/212 (65,6%)**, 73 supervivientes | 17:40 | sí |
| bizkaia-now (raíz) | B1 mapa | 915 → **285** | `parcial (timeout 3600s)`: 269/285 evaluados, 62 caught | 60:01 | no, va a muestreo |
| bizkaia-now (`web`) | B1 mapa | — | `no disponible (stryker no instalado)` | — | no |

**Agregado** (Σcaught/Σviables sobre las ramas completas): **152/225 = 67,6%** (exo + music).

**bizkaia, con el muestreo fijado en la enmienda** (n=150 sin reemplazo, semilla `20260929`, ids 1..285 de mutmut): **33/150 = 22,0%, IC95 Wilson [16,1 %, 29,3 %]**. Es coherente con el 23% del progreso parcial. Se reporta aparte, como exige la enmienda.

## Lectura

- **Instrumento:** acotar al diff redujo los mutantes de Python unas 3-4 veces, y music terminó en 18 min. Con diffs de una sola tarea, que son más pequeños que una rama entera, entra en el timeout por defecto. El hallazgo 1 del verdict anterior queda resuelto para Python.
- **Base:** hay mucha heterogeneidad entre repos (100%, 66% y 22%). El agregado de 67,6% pesa sobre todo música: con n=2 ramas completas no hay nada que generalizar. **Como umbral de "no empeora" para la serie A+ se usa la comparación por repo, no el agregado.**
- **Supervivientes:** de los clasificados, 19 de 20 son **huecos reales de test**, no mutantes equivalentes. En bizkaia los 10 están en `scripts/rematerialize.py`, cubierto solo por la E2E que `pytest` ignora por defecto. Es un hueco concreto y accionable en ese repo, independiente de A+.

## Desviaciones (declaradas; ninguna cambia parámetros del script)

1. Se instaló `whatthepatch` 1.0.7 en el venv de bizkaia (faltaba). Es entorno.
2. `RESET search_path` en la BD postgis, como en la medición 1. Es entorno.
3. **Muestreo:** para ejecutar ids sueltos se extrajo HEAD con `git archive` y los mutantes se regeneraron con `--runner true`. Una primera pasada devolvió "survived" en 1 s por estados cacheados. Esa pasada **se descartó por inválida** (el instrumento no ejecutó los tests) y los estados se resetearon a `untested`. Solo cuenta la segunda. La enmienda no prevé este caso. Se declara porque es un segundo intento, aunque la causa sea un fallo de medición y no un resultado indeseado.
4. Las ramas son `master`, no `main`.

## Pendiente

- El muestreo lo ejecutó a mano el agente de medición. `review-package` no lo implementa. Si se va a usar en la serie, hay que llevarlo al script.
- `web` de bizkaia (TS) no se mide porque el proyecto no declara stryker. Se queda fuera, sin cambiar el protocolo.
