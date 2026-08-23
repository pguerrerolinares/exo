# Ola 1 — Que los instrumentos no mientan: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `process:orchestrate` para
> ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`) para tracking.

**Goal:** Cerrar los tres fallos silenciosos que la sesión de benchmarking del
2026-08-23 encontró en los instrumentos de exo, ANTES de que la Ola 2 los use como
evidencia. Ninguno rompe nada hoy; los tres producen salida con forma válida
estando mal.

**Architecture:** Tres tareas independientes entre sí, sin orden forzado. T1 toca
Rust (engine), T2 toca bash (plugin reflex), T3 es housekeeping fuera de ambos
repos. Se pueden despachar en paralelo a tres ejecutores distintos.

**Tech Stack:** Rust (rusqlite, ya en el árbol), bash 4+, `jq`, `cargo test`.

## Contexto: por qué esta ola va primero

La Ola 2 asciende el sensor `zero-residuo` de aviso a bloqueo, y esa decisión se
apoya en un FP-rate medido **sobre el log que T2 arregla**. Medir con el
instrumento roto ya produjo un número sesgado una vez (reparto real 72/26, no el
51/38 que el log dejaba ver). Y T1 es precondición dura de la Ola 3
(cuantizar el modelo): sin la guarda, un `exo index` por costumbre tras cambiar el
ONNX corrompe el índice sin una sola queja.

## Global Constraints

**Toda tarea las hereda.**

- **Ningún cambio de comportamiento observable salvo el que la tarea declara.**
  Esto es una ola de instrumentación, no de features.
- **Nada de subir techos ni relajar gates** para que algo pase.
- **`git add <rutas>`, nunca `-A`.** Regla de Paul, y además el propio T2 va de eso.
- **Verificación con evidencia fresca del comando**, no con "debería funcionar".
- No tocar `BONUS_SELLADO` ni `ESCALA_FTS_SELLADA` (`engine/src/main.rs:13-26`):
  son el contrato sellado de M2-07 y no entran en esta ola.

---

## T1 — Guarda de modelo en `meta` (Rust)

**Problema:** `engine/src/indexer.rs` cachea embeddings por el **texto exacto del
trozo** (`embeddings_por_texto`, ~línea 319), sin registrar de qué modelo salió el
vector. Hoy solo lo salva por accidente el chequeo de longitud de blob de
`engine/src/vectores.rs:47` (`BYTES_ESPERADOS = 768 * 4`). El día que dos modelos
compartan dimensión —`multilingual-e5-base` es 768d, igual que el jina actual— un
`exo index` incremental mezcla vectores de ambos modelos en la misma tabla,
indistinguibles. Corrupción silenciosa.

**Fix:** escribir el modelo activo en la tabla `meta` (que ya existe) al indexar, y
abortar con error accionable si cambió.

- [ ] Leer el patrón exacto ya existente: `engine/src/indexer.rs:80-91` escribe
      `meta.kb_root` con un `INSERT ... ON CONFLICT` en cada corrida. Copiar esa
      forma, no inventar otra.
- [ ] Al inicio de `indexa()`: leer `meta.modelo_embeddings`. Compararlo con
      `config_embeddings().modelo` (`engine/src/lib.rs:131-145`).
      - Si `meta` no tiene la clave (índice viejo) → escribirla y seguir. Migración
        silenciosa hacia delante, sin romper índices existentes.
      - Si coincide → seguir.
      - Si difiere → **abortar con exit 1** y mensaje accionable por stderr:
        `el índice se construyó con <viejo>, la config pide <nuevo>: corre 'exo rebuild'`.
- [ ] Escribir `meta.modelo_embeddings` (y `meta.dims_embeddings`) al final de una
      indexación con éxito, junto a donde ya se escribe `kb_root`.
- [ ] `exo rebuild` no debe verse afectado: ya borra el fichero de la DB entero
      (`engine/src/main.rs:497-518`), así que parte de cero por construcción.
      **Verificar que sigue siendo así, no asumirlo.**

**Verificación:**
```bash
cd engine && cargo test
# Test nuevo, patrón en engine/tests/cache_embeddings.rs:
#   1. indexa con modelo A sobre una DB temporal (tempfile, ya es dev-dependency)
#   2. cambia la config a modelo B
#   3. indexa otra vez -> DEBE fallar con exit != 0 y mensaje que cite 'rebuild'
#   4. exo rebuild -> DEBE funcionar y dejar meta.modelo_embeddings = B
```
Y una corrida real contra el índice de producción, que NO debe cambiar nada:
```bash
./target/release/exo index --db ~/.exo/index.db --json   # primera vez: escribe la clave
./target/release/exo index --db ~/.exo/index.db --json   # segunda: no-op, sin error
sqlite3 ~/.exo/index.db "SELECT * FROM meta;"
```

**Aviso de acoplamiento:** `meta` es una de las cuatro tablas que consume `kbx`
(`engine/src/schema.rs:15-16` lo documenta). Añadir **filas** no cambia el schema,
así que el canary de kbx no debería moverse — pero **compruébalo** con
`kbx doctor --json` tras la primera corrida, no lo des por hecho.

**Esfuerzo:** ~20-30 líneas + 1 test. Media mañana con la verificación.

---

## T2 — El cap del log deja de sesgar la evidencia (bash)

**Problema medido:** hay dos caps encadenados. Los llamadores cortan a 200
caracteres (`plugins/reflex/scripts/git-add-all-guard.sh:32` y
`verify-before-commit.sh:101`, ambos con `cut -c1-200`) y el helper vuelve a cortar
a 500 (`plugins/reflex/scripts/_reflex-log.sh:22`, `${payload:0:500}`).

Lo grave no es el número: es que **se corta el principio del comando**. Cuando el
comando lleva un heredoc largo delante —una spec, un runbook, un mensaje de
commit— el `git add -A` real cae fuera de la ventana capturada. Consecuencia
medida: la clasificación retroactiva sobre el log estaba sesgada hacia el veredicto
benigno, y el sesgo tiene signo predecible (se pierde lo que está al final, que es
donde vive la acción).

**Fix:** capturar el fragmento **relevante**, no el prefijo. Subir el cap a secas
es el arreglo perezoso: con un heredoc de 4 KB seguiría fallando.

- [ ] En cada llamador, sustituir `cut -c1-200` por una captura de dos partes: los
      primeros ~120 caracteres (contexto) **más la sentencia concreta que disparó
      el reflejo**, separadas por un marcador legible (p. ej. ` … ⟨match⟩ `).
      Cada reflejo ya sabe qué matcheó — es la información que hoy tira.
- [ ] Subir el cap del helper `_reflex-log.sh:22` de 500 a un valor holgado
      (2000 basta) para que el payload compuesto quepa entero.
- [ ] **No romper el contrato de best-effort**: `_reflex-log.sh` declara en su
      cabecera que nunca debe romper el warn-only del reflejo que lo llama.
      Mantener `|| true` y la redirección a `/dev/null` en todas las ramas.
- [ ] Los eventos viejos del log quedan como están: no reescribir el histórico.
      Anotar en la bitácora que los eventos anteriores a este commit tienen el
      sesgo conocido, para que nadie los mezcle con los nuevos al medir.

**Verificación:**
```bash
bash plugins/reflex/scripts/test-git-add-all-guard.sh
# Caso nuevo obligatorio: comando con heredoc de >400 caracteres seguido de
# 'git add -A'. El evento logueado DEBE contener el 'git add -A'.
tail -3 ~/.claude/reflex-log.jsonl | jq -r '.payload'
```

**Contaminación aparte, que sale gratis arreglar aquí:** 88 de las 1.746 entradas
del log (5%) son **sintéticas** — el propio arnés de pruebas escribe en el log de
producción (`session_id` tipo `test-sid`, `test-guard-NNNNNN`, `sim1`). Los tests
deberían exportar `REFLEX_LOG_FILE` a un fichero temporal; el helper ya respeta esa
variable (`_reflex-log.sh:14`), así que es una línea por script de test.

**Esfuerzo:** ~1-2 h con los tests.

---

## T3 — Limpiar el marketplace desincronizado (housekeeping)

**Problema:** `~/.claude/plugins/marketplaces/exo/plugins/reflex` es un checkout
viejo que todavía contiene `basic-memory-recall.sh`, `retrieval-logger.sh` y
`search-before-write.sh` — retirados por M6-03. **No es el que se ejecuta** (corre
`~/.claude/plugins/cache/exo/reflex/0.15.0`), así que no rompe nada hoy. Pero
republicar el marketplace tal cual **resucitaría basic-memory** justo cuando M5b va
a desinstalarlo.

- [ ] Confirmar primero cuál es el origen real de ese checkout (`git -C
      ~/.claude/plugins/marketplaces/exo remote -v` y `log --oneline -3`) y si está
      simplemente desactualizado respecto al repo `exo` o divergido.
- [ ] Si es solo un checkout viejo: refrescarlo desde el origen correcto. **No
      editar ficheros a mano dentro de un directorio gestionado por el harness** —
      se sobrescribe en la próxima sincronización.
- [ ] Verificar que tras el refresco no queda ninguno de los tres scripts muertos:
      ```bash
      ls ~/.claude/plugins/marketplaces/exo/plugins/reflex/scripts/ | grep -E 'basic-memory|retrieval-logger|search-before-write'
      # salida esperada: vacío
      ```
- [ ] **Si el refresco no es trivial o el origen no está claro, PARA y escala a
      Paul.** Esto vive fuera del repo y toca la instalación real de sus plugins:
      no es sitio para improvisar.

**Esfuerzo:** ~30 min si es un `git pull`; escalar si no.

---

## Definition of Done de la ola

- [ ] `cargo test` verde en `engine/`.
- [ ] Los tests de bash de reflex verdes.
- [ ] `kbx doctor --json` sin findings nuevos.
- [ ] `sqlite3 ~/.exo/index.db "SELECT * FROM meta;"` muestra `modelo_embeddings`.
- [ ] Un evento reciente del log muestra el `git add -A` de un comando con heredoc.
- [ ] Ningún script muerto en el checkout del marketplace.
- [ ] Commits scoped por tarea, sin `git add -A`.

## Qué NO entra en esta ola

- Ascender `zero-residuo` a bloqueo (Ola 2: necesita el regex arreglado primero).
- `status`/`superseded_by` en `notas` (Ola 2).
- Cuantizar el ONNX (Ola 3: depende de T1).
- Nada de Windows (Ola 4: bloqueada por el toolchain, que depende de IT).

**Fuentes:** artifact `ce9cedee` (roadmap medido) y `d099beea` (benchmarking),
más la entrada del 2026-08-23 (tarde) en `kb-demo/log/exo-bitacora.md`.
