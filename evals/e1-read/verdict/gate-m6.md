# Gate M6 — cutover del recall de arranque (m6-01 + m6-cutover-recall)

Consultor de gate fresco (fable, delegado por Paul). Verificación primaria
propia: todos los comandos de abajo los corrí yo; ningún número viene del
reporte sin recomputar.

Ramas juzgadas:
- `m6-01` en exo (4 commits sobre `3d7f073`: 8a0e0c1 refresca, d516879
  cache de embeddings, fc31aaf `--contenido`/`--nota`, 6b3b491 reporte).
- `m6-cutover-recall` en agent-develop (a1818ea, reflex 0.13.0:
  SessionStart → `exo-recall.sh`, Stop += `exo-index.sh`).

## Qué verifiqué y con qué comando

1. **Suite completa** — `cargo test` en `.worktrees/m6-01/engine`:
   **98 passed / 0 failed** (1 ignored: `jina_es_embebe_a_768`, smoke del
   modelo, preexistente). Coincide con lo que exige el brief (98/98).

2. **Paridad de contenido del bloque de arranque** (la cuestión central):
   corrí ambos scripts con stdin `{}` y diffeé el
   `additionalContext` extraído con jq.
   - Viejo (`basic-memory-recall.sh`): 4541 bytes, exit 0.
   - Nuevo (`exo-recall.sh`): 4636 bytes, exit 0, ~10 ms.
   - **El cuerpo del core-index (contrato de memoria + doctrina compacta +
     mapa de cores) es byte-idéntico en ambos.** Difieren solo: la cabecera
     nueva (`=== Recall exo (PARCIAL…) ===` + ruta del fichero) y el digest
     de actividad (ver no-bloqueante 1).

3. **Caminos de fallo del hook nuevo** — cinco casos, con `REFLEX_LOG_FILE`
   redirigido a scratchpad para no ensuciar el log real:
   - `EXO_BIN=/no/existe` → fallback, exit 0, evento `reason=no-engine`.
   - `EXO_INDEX=/no/existe.db` → fallback, exit 0, `reason=no-index`.
   - índice corrupto (200 bytes de urandom) → fallback, exit 0,
     `reason=empty`.
   - `EXO_RECALL_NOTA` apuntando a una nota sin contrato → fallback,
     `reason=no-contract` (guard semántico conservado; cubre también KB
     movida: cuerpo ilegible ⇒ bloque sin "Contrato de memoria" ⇒ fallback).
   - nota inexistente → fallback, `reason=empty`.
   Los cinco eventos quedaron greppables en el JSONL. El fallback ya no
   menciona basic-memory.

4. **`exo-index.sh`** — probado en vivo contra copia del índice: el hook
   retorna en **9 ms** (setsid -f + nohup, detached; no puede colgar el
   cierre) y el envelope aparece en el log
   (`saltadas=138`). Dos lanzamientos simultáneos: ambos envelopes limpios,
   `PRAGMA integrity_check` → ok, 138 notas. Con contención real de
   escritura el perdedor recibiría SQLITE_BUSY (no hay `busy_timeout`), el
   error queda en el log y el siguiente Stop lo recupera — pérdida de un
   indexado, nunca corrupción.

5. **Cache de embeddings (d516879)** — revisión de código dura:
   - La clave es el **texto exacto** del trozo (no hash, no posición):
     colisiones imposibles, inserciones que desplazan `orden` siguen
     acertando.
   - Textos repetidos dentro de una nota: dedup en `pendientes` (una pasada
     por el modelo) y cada `orden` resuelve del mapa; correcto.
   - Blob corrupto: `vectores::lee` descarta longitudes no múltiplo de 4 →
     re-embebe (recuperable). Lectura ANTES del borrado; fila sin vector
     simplemente no entra al mapa.
   - **Prueba empírica del reuso** (la fuerte): `cp -a` de la KB +
     `touch` de las 138 notas + reindexado sobre copia del índice de
     producción → `indexadas=138, trozos_embebidos=0,
     trozos_reusados=3018` en 65 s (modelo nunca cargado; el peor caso
     ">10 min" del reporte queda en ~1 min con cache caliente).

6. **Eval de regresión** (el chequeo que más pesa) —
   `replay-engine.py --db ~/.exo/index.db --tipo hybrid` + `analyze.py`,
   contra la referencia de hoy `metrics-engine-hybrid-m209.md`:
   - Índice de producción: **hybrid 48/55** — idéntico a la referencia,
     con el sweep de threshold línea por línea igual (48/48/46/38/23/14/0)
     y las mismas 7 misses.
   - Índice reindexado 100 % vía cache (los 3018 vectores reusados):
     **48/55 y métricas byte-idénticas** (diff vacío contra la corrida
     directa). Si el cache mapeara un solo vector al texto equivocado, aquí
     se habría visto.
   Ficheros: `results/metrics-engine-hybrid-gate-m6{,-reuse}.md`.

7. **Pin post-compactación** — `source=compact` con un reflex-log simulado
   (`verify-before-commit` + `git-c`): el bloque añade las dos reglas
   reforzadas. El bug del id que no matcheaba está arreglado de verdad, y el
   pin funciona incluso cuando la base es el fallback.

## Bloqueantes

Ninguno.

## No bloqueantes

1. **Digest de actividad más pobre que el viejo**: top-5 por `git_epoch`
   (default `--limite 5`) frente a hasta 15 permalinks por actividad 3d.
   En la comparación real el viejo listaba 9 notas y el nuevo 5; la nota de
   hoy (`inbox/2026-08-17-gotchas-windows`) sale en el viejo y no en el
   nuevo. No es corrupción — es semántica distinta (recencia git vs mtime) +
   cap más corto. El bloque nuevo usa 4636 de 6144 bytes: hay hueco para
   pasar `--limite 10` en `exo-recall.sh` si se quiere paridad más fina.
2. **Truncado por cap sin rastro**: si el core-index creciera por encima de
   6144 bytes, el camino viejo caía a fallback con evento `oversize`; el
   nuevo trunca por líneas y el aviso va a stderr, que el hook descarta.
   Doctrina truncada en silencio es improbable hoy (margen ~1,5 KB y techo
   sellado de cores), pero rompe el principio "degradación con rastro"
   (F3.1). Barato: loguear un evento si `exo recall` avisó de truncado.
3. **`abre_db` sin `busy_timeout`**: un recall concurrente con un index en
   plena escritura puede recibir BUSY inmediato → fallback `empty` una
   sesión. `PRAGMA busy_timeout=1000` lo mitigaría casi gratis.
4. **`vectores::lee` valida `% 4`, no la dimensión**: un blob truncado a
   múltiplo de 4 pasaría el filtro y fallaría ruidosamente al insertarse en
   vec0 (dimensión fija 768) — error visible, no silencioso, pero
   `len == 3072` sería el chequeo directo.
5. **El cache no versiona el modelo**: cambiar de modelo de embeddings
   reutilizaría vectores del modelo viejo. Mitigado porque ese cambio
   implica `exo rebuild` (DB nueva); conviene recordarlo en la nota del
   cambio de modelo si algún día llega.
6. Cosmético: la cabecera "PARCIAL — no sustituye tu brief" está redactada
   para subagentes pero se inyecta en el SessionStart del padre.

## Juicio

El listón del cutover era que el agente no pierda nada al arrancar: la
doctrina llega byte-idéntica, los fallos degradan con rastro y exit 0, el
cierre no se bloquea, y el cambio de más riesgo silencioso (el cache) queda
probado por el camino más duro posible (reindexado total servido por cache →
eval idéntico al de referencia). Las diferencias reales encontradas (digest
más corto, truncado sin evento) son mejoras de seguimiento, no regresiones
del contrato.

GATE: MERGED — 2026-08-17 23:48 (consultor fable delegado)
