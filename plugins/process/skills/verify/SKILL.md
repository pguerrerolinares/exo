---
name: verify
description: Usa antes de declarar trabajo completo, arreglado o pasando — antes de commitear, de crear un PR o de aceptar el trabajo de un subagente. Evidencia fresca del comando antes de cualquier claim.
---

# verify

Antes de declarar trabajo completo, arreglado o pasando — antes de
commitear, crear un PR, o aceptar el trabajo de un subagente — evidencia
fresca del comando, siempre. Afirmar sin verificar es deshonestidad, no
eficiencia.

Solo auto-verificación barata pre-commit. Esta skill NO despacha reviewers ni
subagentes — eso vive en `process:orchestrate` (mezclarlos es spam de
reviews o dilución del gate).

## Regla

Si no corriste el comando en ESTE mensaje, no puedes afirmar que pasa. La
regla aplica a toda variante del claim — paráfrasis, sinónimos, implicación
de éxito — antes de commit, PR, completar tarea, pasar a la siguiente, o
delegar.

## Gate function

1. Identifica qué comando prueba el claim.
2. Córrelo completo y fresco.
3. Lee TODO el output, el exit code, cuenta los fallos.
4. ¿Confirma el claim? Si no, reporta el estado real con evidencia.
5. Solo entonces afirma.

Tabla claim→evidencia: tests pasan = output fresco con 0 fallos; linter
limpio = 0 errores; build = exit 0 (linter ≠ compilador); bug arreglado = el
síntoma original testeado pasa; requirements cumplidos = checklist línea a
línea, no "tests pass".

Test de regresión = ciclo red-green VERIFICADO: escribir → pasa → revertir
el fix → DEBE fallar → restaurar → pasa. Sin eso no hay test de regresión.

Trabajo delegado: el reporte de éxito de un agente no es evidencia — verifica
el diff del VCS y los cambios reales antes de reportar estado.

## Red flags que obligan a parar

"should/probably/seems to", expresar satisfacción antes de verificar,
commit/push/PR sin verificación, verificación parcial, "solo esta vez",
cansancio.

## Gate del padre (al aceptar trabajo de un hijo)

1. **¿Funciona?** — tests, linter, type-checker, security scan del proyecto.
2. **Verificación real donde aplique** — UI: conduce la app de verdad (p.ej.
   Playwright) y mírala en desktop Y mobile; un build pasando NO es prueba
   visual. Backend: pega al endpoint/DB real.
3. **Calidad de ingeniería, gate de release no nice-to-have**: reuse-first
   (componentes/utilidades existentes antes que hand-rolling), tooling
   correcto (el package manager del lockfile), deps explícitas (nada
   confiado transitivamente), DRY (dos usos del mismo patrón ⇒ extraer).
4. **Escrutina el diff** buscando lo que el autor pudo pasar por alto — orden
   de rutas, drops silenciosos, bugs latentes, seguridad. Commit atómico solo
   después de pasar esto.
