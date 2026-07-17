# Gold — process:verify (paridad de movimientos)

Fuentes:
- VBC = `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/verification-before-completion/SKILL.md` (139 líneas)
- OP = `~/.claude/plugins/cache/agent-develop/paul-profile/0.5.0/skills/orchestrate-personal/SKILL.md`, sección "Parent validation gate" (líneas 82-99) — framework §5.2: "gate de validación de orchestrate-personal"

Uso: ver `evals/prep-m3/README.md`.

## Límite de diseño (framework §5.2 — verificable por ausencia)

- [ ] `process:verify` contiene SOLO auto-verificación barata pre-commit; NO despacha reviewers ni subagentes — framework §5.2: "Solo auto-verificación barata pre-commit; el reviewer-dispatch (caro, pre-merge) vive en orchestrate — mezclarlos = spam de reviews o dilución del gate". Presencia de cualquier dispatch de review en verify = fallo.

## Movimientos — verification-before-completion

- [ ] Principio: evidencia antes de claims, siempre; afirmar completado sin verificar es deshonestidad, no eficiencia — VBC líneas 10-14
- [ ] Regla: sin evidencia fresca de verificación no hay claim de éxito; si no corriste el comando en ESTE mensaje, no puedes afirmar que pasa — VBC líneas 16-23 (destilada sin el grito)
- [ ] Gate function: (1) identificar qué comando prueba el claim → (2) correrlo completo y fresco → (3) leer TODO el output, exit code, contar fallos → (4) ¿confirma? si no, reportar el estado real con evidencia → (5) solo entonces afirmar — VBC líneas 25-38
- [ ] Tabla claim→evidencia: tests pasan = output fresco con 0 fallos; linter limpio = 0 errores; build = exit 0 (linter ≠ compilador); bug arreglado = el síntoma original testeado pasa; requirements cumplidos = checklist línea a línea, no "tests pass" — VBC líneas 40-50
- [ ] Red flags que obligan a parar: "should/probably/seems to", expresar satisfacción antes de verificar, commit/push/PR sin verificación, verificación parcial, "solo esta vez", cansancio — VBC líneas 52-61
- [ ] Test de regresión = ciclo red-green VERIFICADO: escribir → pasa → revertir el fix → DEBE fallar → restaurar → pasa; sin eso no hay test de regresión — VBC líneas 84-88
- [ ] Trabajo delegado: el reporte de éxito de un agente no es evidencia — verificar el diff del VCS y los cambios reales antes de reportar estado — VBC líneas 48-49 + 102-105
- [ ] La regla aplica a toda variante del claim (paráfrasis, sinónimos, implicación de éxito), antes de commit, PR, completar tarea, pasar a la siguiente o delegar — VBC líneas 117-131

## Movimientos — parent validation gate (orchestrate-personal)

- [ ] Gate 1 · ¿funciona?: correr tests, linter, type-checker y security scan del proyecto — OP línea 83
- [ ] Gate 2 · verificación real donde aplique: UI ⇒ conducir la app de verdad (p.ej. Playwright) y mirarla en desktop Y mobile — un build pasando NO es prueba visual; backend ⇒ pegar al endpoint/DB real — OP líneas 84-87
- [ ] Gate 3 · calidad de ingeniería como gate de release, no nice-to-have: reuse-first (componentes/utilidades existentes y la librería de primitivas del proyecto antes que hand-rolling), tooling correcto (el package manager del lockfile), deps explícitas (nada confiado transitivamente), DRY (dos usos del mismo patrón ⇒ extraer) — OP líneas 88-97
- [ ] Gate 4 · escrutinar el diff buscando lo que el autor pudo pasar por alto (orden de rutas, drops silenciosos, bugs latentes, seguridad); commit atómico solo después de pasar — OP líneas 98-99

## DESCARTES (corpus negativo)

- IRON LAW en mayúsculas (VBC líneas 16-22), "Skip any step = lying, not verifying" (línea 37) y "Violating the letter…" (línea 13): gritos — framework §5.2; las reglas se conservan destiladas (arriba).
- §Rationalization Prevention tabla completa (VBC líneas 63-74): catálogo excusa/réplica — framework §5.2 "gates dogmáticos"; se destila a los red flags (conservados).
- §Why This Matters / "24 failure memories" / "you'll be replaced" (VBC líneas 108-115): prosa.
- El resto del parent validation gate que sí despacha o coordina (validar al hijo, review dispatch, filtrar findings): vive en `process:orchestrate`, no aquí — framework §5.2 (límite de diseño, arriba).
