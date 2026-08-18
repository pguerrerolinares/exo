# Verdict de gate — C8: M3 (cutover de skills) + M1b (marketplace)

- **Fecha**: 2026-08-18T07:48:04+02:00 · **Consultor**: fable delegado, fresco (sin participación en diseño ni implementación de C8; no consulté al orquestador).
- **Juzga**: exo rama `c8-m3` (base `89dbe95`, HEAD `4d339df`) · agent-develop rama `c8-m3` (base `fe4d2d6`, HEAD `4043a56`) · kb-demo rama `c8-m3-05` (base `1683537`, HEAD `4b02960`). Merge-bases verificados: cada rama nace exactamente de su base declarada.
- **Criterio**: plan `2026-08-17-cierre-exo-m2-a-m5b.md` §Campaña 8 y §0; spec madre `2026-07-16-framework-unificado-design.md` §5.2/§5.3/§8; config `.superpowers/fabrica/config.md` §Ejecución de gates; spec de la pieza `2026-08-18-m3-m1b-cutover-design.md`; los tres verdicts de `consultas/2026-08-18-c8/` (auditados, no asumidos).

## VEREDICTO: **MERGED** — con 2 hallazgos medios y 4 bajos anotados como deuda. Cero altos.

---

## 1. Verificación primaria propia (re-corrida, no leída)

Todos los oráculos del package re-corridos por mí, con salida real:

| Oráculo | Comando | Resultado |
|---|---|---|
| Manifest de process | `claude plugin validate .worktrees/c8-m3/plugins/process` | `✔ Validation passed` |
| Manifest del catálogo | `claude plugin validate agent-develop/.worktrees/c8-m3` | `✔ Validation passed` |
| Control negativo del validador | mismo JSON con `"source-inventado"` en vez de `git-subdir` | `✘ plugins.0.source: Invalid input` — el pass del source bueno significa algo |
| Engine | `cargo test --manifest-path engine/Cargo.toml` | suma por suite: 34+14+3+16+2+16+7+5+5+4+2+2+4 = **114 passed, 0 failed** (1 ignored) |
| Suite reflex | `for t in test-*.sh; do bash $t; done` (worktree agent-develop) | **9/9 suites verdes** (73/73, 42/42, 16/16, 7/7, 26/26, 5/5, 11/11, 6/6, 7/7) |
| Guard de fábrica | `fabrica-main-guard.test.sh` | `PASS=19 FAIL=0` |
| Recall en producción | `exo recall --db ~/.exo/index.db --contenido --nota kb-demo/core/core-index --limite 10 --cap-bytes 6144` | **5487 B** hoy (5563 B en el package — varía con el digest de recientes), bullet ROUTING DE PROCESO presente (1 hit), **sin truncado** (grep "trunc" vacío) |

Oráculos que el package NO citaba, corridos además:

- **Credenciales del camino `git-subdir` a repo privado** (lupa #3): `git ls-remote https://github.com/pguerrerolinares/exo.git HEAD` → `89dbe95… HEAD`, exit 0; ídem `agent-develop.git` → `fe4d2d6…`. El clone https con el helper de gh que usará `claude plugin install process@exo` **funciona hoy contra ambos repos privados**. Visibilidades confirmadas por `gh repo view`: exo PRIVATE, agent-develop PRIVATE, workflow-lint PUBLIC.
- **Línea roja «nada de push»**: HEADs remotos = bases (`89dbe95`, `fe4d2d6`, `1683537`) y `ls-remote origin "refs/heads/c8*"` **vacío en los tres repos**. Nada se pushó. Los tres worktrees limpios (`status --short` vacío).
- **M3-02 en fuente**: `plugins/process/skills/orchestrate/SKILL.md:14` («`subagent_type: reflex:executor`, **nunca** `general-purpose`, **sin** `model`») e `implementer-prompt.md:7` — el dispatch se conserva sin `model`. Sin diff en esos ficheros, como declara el package.
- **M3-04 sin regresión** (lupa #5): 11 ficheros de `plugins/process/` con «Jesse Vincent» (los 13 hits de B) + `LICENSES/superpowers.LICENSE` intactos; `git diff main...HEAD | grep '^-.*\(MIT\|Jesse\)'` **vacío** — el diff no borra ni una línea de atribución, y AÑADE la sección «Atribución» al README raíz (`README.md:26-33`), que era lo que faltaba. Veto AGPL reafirmado por escrito en ese mismo bloque; el diff de C8 es docs+JSON, no introduce dependencia alguna.
- **Barrido propio de consumidores** (lupa #2): grep de `orchestrate-personal` en `~/.claude/CLAUDE.md`, `~/.claude/commands/`, kb-demo (fuera de worktrees) y exo. Resto: solo prosa histórica (logs/archive/research de la KB), la deuda ya declarada `projects/agent-develop.md` (spec §8.5), y dos restos de linaje cosméticos (bajo B3). Ningún consumidor operativo sin cubrir. Cero `superpowers:X` invocables en las skills de process (solo atribución); hooks.json de reflex sin referencia a a1-* ni a skills.

## 2. Los seis puntos de la lupa, adjudicados

1. **Reconciliación §6 — CORRECTA.** Leí los tres verdicts completos. A y C colisionaban en el name `exo` («adding a second marketplace with the same name replaces the first»). La razón del orquestador se sostiene contra el propio texto de A: A justificaba el path local como «el único camino sin push», pero su A4 adjudica que `marketplace add`/`install` escriben settings y son de Paul — que pushea en el mismo gesto (runbook Fase 1). El path local no ahorraba nada y duplicaba el name. De A sobrevive lo que debía: el `plugin.json` (byte-idéntico al adjudicado en A2), el bullet A3 (texto exacto en el diff de kb-demo) y el orden process-ON-antes-de-superpowers-OFF (runbook Fase 4a/4b). El commit `27663ce` borra el marketplace propio de exo; verificado: `exo/.claude-plugin/` no existe en el worktree. Nada material de A se perdió.
2. **`orchestrate-personal` sin alias — CORRECTO.** Retirada firmada por spec madre («paul-profile menos fabrica») y sin consumidor vivo (barrido propio arriba + B1). El fallo de invocación es visible (`Unknown skill`, sonda B2), no silencioso.
3. **`git-subdir` a repo privado — RIESGO CERRADO empíricamente** hasta donde se puede sin instalar (ls-remote https OK contra ambos privados). El resto queda cubierto por el humo de Fase 4a ANTES de apagar superpowers: si el install falla, falla visible y no se ha apagado nada.
4. **Orden del runbook — CORRECTO.** Fase 1 (push exo) precede al install (Fase 3); rename (Fase 2) precede al `marketplace add exo-plugins`; remove+add+installs van seguidos con sesiones cerradas (la ventana sin reflex no sirve a nadie); Fase 4a deja superpowers encendido hasta humo verde → la línea roja M3-01 es cumplible incluso con fallo a mitad; cachés superpowers 6.x se conservan como rollback. La invariante de B3 (fabrica 0.6.0 nunca activa sin process instalado) se preserva aunque el orden literal de B («flip antes de push») no se siga: entre Fase 2 y Fase 4a no corre ninguna sesión. Única arruga: el cron (MEDIO-2 abajo).
5. **M3-04 — sin regresión** (evidencia arriba).
6. **Corte a1 — CORRECTO.** Se retira el watchdog (`a1-freeze-watch.sh` + su test, 560 líneas) que vigilaba una ventana cerrada el 2026-08-02 y que romperá de todas formas al morir `orchestrate-personal`; se conserva `a1-gate.sh` + `test-a1-gate.sh` (73/73 verdes), que es la calculadora reproducible del verdict A1 — audit trail, no operación. hooks.json no referencia a ninguno.

## 3. Hallazgos

**ALTOS: ninguno.**

**MEDIOS** (deuda, se mergea igual):

- **M-1. El índice vivo se adelantó al gate.** `~/.exo/index.db` ya sirve el bullet ROUTING DE PROCESO bajo el permalink `kb-demo/core/core-index` (verificado: `exo search --type fts "ROUTING DE PROCESO"` lo devuelve como 1er hit y el recall lo incluye), mientras `main` de kb-demo tiene **0 hits** de ese texto. La medición «en producción» de la spec §5 se hizo mutando producción: el arranque de toda sesión sirve doctrina de una rama sin mergear desde antes de este verdict, y con `exo index` incremental por mtime un rechazo NO se habría autocorregido solo. Con MERGED converge al mergear; la deuda es de proceso: medir sin mutar (DB efímera) o reindexar main tras la medición.
- **M-2. Ventana de cron roto entre el merge y la Fase 5.** `crontab -l` → `7 9 * * * /home/paul/Documentos/proyectos/agent-develop/plugins/reflex/scripts/a1-freeze-watch.sh` — apunta al **working tree del repo**, no a la caché. En cuanto el orquestador mergee `c8-m3` a master, el fichero desaparece del checkout y el cron falla con ENOENT cada mañana hasta que Paul ejecute la Fase 5 (que puede ser días después). Nada se rompe — es ruido — pero la premisa del runbook («sin esto appendea FREEZE ROTO espurio») queda desactualizada post-merge. Fix trivial: una línea en el runbook diciendo que la Fase 5 es independiente del resto y conviene ejecutarla el mismo día del merge (o primera).

**BAJOS** (una línea cada uno):

- B-1. `reflex/README.md:5` y la description de reflex en `marketplace.json` conservan prosa de linaje sobre `orchestrate-personal` — documental, misma clase que la deuda §8.3 de la spec.
- B-2. `autoUpdate: true` sobre marketplace privado por HTTPS tiene intermitencia documentada del background pull (riesgo C5-2, preexistente y benigno; alternativa SSH ya anotada por C).
- B-3. El margen real del bloque de arranque es fino y variable (581 B según spec; hoy 657 B libres con 5487 B servidos) — ya declarado como deuda §8.1, lo confirmo con medición propia.
- B-4. Las deudas §8.2 (config de fábrica desactualizado sobre el guard de kb-demo), §8.4 (README de process cita 6.1.1) y §8.5 (`projects/agent-develop.md`) quedan bien declaradas y con dueño; nada que añadir.

## 4. Qué busqué para objetar (mandato de disenso)

- **«El package miente en algún oráculo»** — re-corrí los siete; el único delta es el byte-count del recall (5563→5487 B, explicado por el digest de recientes). REFUTADA.
- **«La reconciliación §6 enterró algo de A que importaba»** — leí el verdict A completo buscando un argumento del path local que NO dependiera de «sin push»: no existe; el propio A llama a su camino «el ÚNICO ejecutable hoy» y esa premisa muere con A4+runbook. Verifiqué además que lo que la spec dice conservar de A (plugin.json, bullet, orden) está conservado byte a byte. REFUTADA.
- **«git-subdir a privado falla en este entorno»** — la hipótesis del propio package (lupa #3): la ataqué con el experimento que nadie había corrido (`git ls-remote` https contra ambos privados, que es el camino exacto del clone). REFUTADA hasta el límite de lo probable sin instalar; el resto lo cubre el humo 4a.
- **«Hay una ventana sin skills/reflex en el runbook»** — tracé fase a fase buscando un estado en que una sesión corra sin recall o con fabrica 0.6.0 sin process: no existe si se respeta «sesiones cerradas» (P3). Lo que SÍ encontré buscando esto es M-2 (el cron apunta al working tree, no a la caché — dato que ni el package ni la spec recogen). HALLAZGO.
- **«El barrido de consumidores se dejó alguno»** — re-barrí por mi cuenta `~/.claude` (CLAUDE.md, commands), la KB entera y exo: solo histórico y linaje. REFUTADA.
- **«La medición 'en producción' del recall era inocua»** — la intenté confirmar y encontré lo contrario: el DB vivo contiene contenido no mergeado bajo el permalink canónico (main: 0 hits; índice: hit). HALLAZGO (M-1).
- **«El corte a1 tira de más o de menos»** — leí qué hace cada script: gate = cálculo reproducible del verdict firmado (se queda, tests verdes); watch = vigilancia de ventana cerrada (se va). Busqué referencias cruzadas en hooks.json y suites: ninguna. REFUTADA.

## 5. Condición de validez

Las 4 condiciones del régimen (config §Ejecución de gates): fresco ✓ · verificación primaria propia ✓ (todo comando citado arriba fue ejecutado por este consultor) · sección de disenso ✓ · este fichero commiteado en la rama `c8-m3` de exo antes del merge ✓.
