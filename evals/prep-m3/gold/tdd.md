# Gold — process:tdd (paridad de movimientos)

Fuente: `~/.claude/plugins/cache/claude-plugins-official/superpowers/6.1.1/skills/test-driven-development/SKILL.md` (371 líneas) + `testing-anti-patterns.md`.
Uso: ver `evals/prep-m3/README.md`.

## Movimientos

- [ ] Ciclo completo: escribir el test primero → verlo fallar → código mínimo → verlo pasar → refactor en verde — SKILL.md §Overview (líneas 10-12) + §Red-Green-Refactor (líneas 47-69)
- [ ] Core principle: si no viste el test fallar, no sabes si testea lo correcto — línea 12
- [ ] Regla central: no código de producción sin un test que falló primero — §The Iron Law (líneas 31-35, destilada sin el grito) + §Final Rule (líneas 364-369)
- [ ] Código escrito antes del test se BORRA y se reimplementa desde los tests (no se guarda "como referencia", no se "adapta") — líneas 37-45
- [ ] Verify RED obligatorio: correr el test y confirmar que falla (no que erra) por la razón esperada — feature ausente, no typo; si pasa de primeras, el test está mal — líneas 113-128
- [ ] GREEN mínimo: el código más simple que pasa el test; sin features extra, sin refactor ajeno, sin "mejoras" más allá del test (YAGNI) — líneas 130-166
- [ ] Verify GREEN obligatorio: el test pasa, el resto sigue verde, output pristine (sin errores ni warnings); si falla, se arregla el código, no el test — líneas 168-183
- [ ] REFACTOR solo en verde: quitar duplicación, mejorar nombres, extraer helpers; sin añadir comportamiento — líneas 185-192
- [ ] Propiedades de buen test: un comportamiento por test ("and" en el nombre ⇒ partir), nombre que describe conducta, código real (mocks solo si inevitable) — líneas 108-111 + §Good Tests (líneas 198-204)
- [ ] Excepciones legítimas SOLO con permiso del humano: prototipos desechables, código generado, ficheros de config — líneas 24-28 + línea 371
- [ ] Bug fix = primero un failing test que reproduce el bug, luego el ciclo; nunca arreglar un bug sin test — §Debugging Integration (líneas 351-355)
- [ ] Checklist de verificación pre-completado (test por función nueva, cada test visto fallar por la razón esperada, todo verde, output pristine, edge cases cubiertos); si no se puede marcar todo ⇒ no fue TDD — líneas 327-340
- [ ] Tabla when-stuck: no sé testear ⇒ escribir la API deseada / preguntar; test complicado ⇒ diseño complicado, simplificar interfaz; todo requiere mock ⇒ acoplamiento, inyectar dependencias — líneas 342-349
- [ ] Anti-patrones de testing (reference file): nunca testear comportamiento de mocks, nunca añadir métodos test-only a clases de producción, nunca mockear sin entender la dependencia — testing-anti-patterns.md líneas 13-19 (+ gate "¿estoy testeando el componente o la existencia del mock?", líneas 51-60)

## DESCARTES (corpus negativo)

- "Violating the letter of the rules is violating the spirit of the rules" (línea 14) y el bloque IRON LAW en mayúsculas (líneas 31-35): gritos — framework §5.2 "se tira … los gritos"; la regla se conserva destilada (arriba).
- Ensayo §Why Order Matters completo (líneas 206-254): prosa justificativa anti-racionalización; el contenido operativo ya está en los movimientos.
- Tablas §Common Rationalizations (líneas 256-270) y §Red Flags (líneas 272-288) como catálogo de diálogo: gates dogmáticos en formato excusa/réplica — framework §5.2; se destilan a la regla "código antes del test ⇒ borrar y empezar de nuevo" (conservada arriba).
- Ejemplo bug-fix completo (líneas 290-325) y digraph (líneas 49-69 como gráfico): prosa/formato; el ciclo se conserva como regla.
- Ejemplos <Good>/<Bad> extensos en TypeScript (líneas 75-105, 134-164): ilustración; pueden sobrevivir resumidos en el reference file, no son movimiento exigible del body.
