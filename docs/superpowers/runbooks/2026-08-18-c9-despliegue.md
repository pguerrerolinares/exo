# C9 — despliegue: lo que ejecuta Paul

> Campaña C9 (M6 completo parcial) cerrada y mergeada a `main`. Lo que sigue es
> **entorno vivo**: pushes y repunte del marketplace. Nada de esto lo ejecuta la
> fábrica.
>
> Plan: `plans/2026-08-18-c9-m6-completo.md` · Spec: `specs/2026-08-18-cierre-en-regimen-design.md`
> Verdict final: `MERGE CON ARREGLOS`, los cuatro Important resueltos antes del merge.

## Ya hecho, no repetir

- **Binario reconstruido e instalado.** `~/.local/bin/exo` era 20 h anterior al
  fix del indexer: el fix de la campaña no estaba en la máquina. Reconstruido
  (`cargo build --release`) e instalado, verificado con `exo index --json`
  (envelope válido, 138 notas). **Rollback: `cp /tmp/exo-rollback ~/.local/bin/exo`.**
  `exo --version` está fijo en `0.1.0` y **no sirve** para distinguir builds.
- **`pre-commit` de la KB repuntado.** Era un symlink a
  `agent-develop/plugins/reflex/scripts/kb-precommit.sh` — el árbol que el paso 4
  borra. Un `pre-commit` roto no da error: git lo trata como ausente y commitea
  con exit 0, así que el trinquete de presupuestos habría muerto en silencio.
  Ahora apunta a `exo/plugins/reflex/scripts/kb-precommit.sh` y **se verificó
  disparando** (nota core sobredimensionada → rechazo).

## Antes de tocar nada: la red de rollback

```bash
cp -a ~/.claude/plugins/cache/exo/reflex/0.13.1 /tmp/reflex-rollback
```

Es el estado que funciona hoy. Restaurarlo devuelve los hooks aunque el catálogo
quede roto.

## Paso 1 — Pushes (el orden NO es cosmético)

`git-subdir` resuelve contra GitHub, no contra el disco. Si el marketplace se
actualiza antes del push, **no encuentra el plugin**: `origin/main` de exo aún no
contiene `plugins/reflex`.

```bash
git -C ~/Documentos/proyectos/exo push origin main
git -C ~/Documentos/proyectos/agent-develop push origin master
```

## Paso 2 — Repuntar el catálogo

```bash
claude plugin marketplace update exo
```

## Paso 3 — Verificación **falsable**

El check del plan original (`ls … | debe seguir 0.13.1`) no distinguía éxito de
no-op: `0.13.1` era justo lo instalado. Por eso la versión subió a **0.14.0** y
estos tres checks sí pueden fallar:

```bash
ls ~/.claude/plugins/cache/exo/reflex/
# Esperado: aparece 0.14.0 (prueba positiva de que el refetch ocurrió)

ls ~/.claude/plugins/cache/exo/reflex/*/scripts/ | grep -c search-before-write
# Esperado: 0 — ese fichero solo existe en la copia vieja

grep -o '"gitCommitSha":"[^"]*"' ~/.claude/plugins/installed_plugins.json | head
# Esperado: un SHA del repo exo, no de exo-plugins
```

Y **reiniciar sesión**: debe arrancar con su bloque de recall. Un subagente
trivial debe seguir recibiendo inyección:

```bash
grep inject-emitted ~/.claude/reflex-log.jsonl | tail -1
```

Si el arranque pierde el recall: `cp -a /tmp/reflex-rollback ~/.claude/plugins/cache/exo/reflex/0.13.1`
y revertir la entrada `reflex` de `marketplace.json` a `"./plugins/reflex"`.

## Paso 4 — Borrar el árbol viejo (NO el mismo día)

**Espera 2-3 sesiones verdes.** Este paso es la única parte irreversible: el
marketplace se resuelve desde el clon de `exo-plugins`, y borrar allí mata la vía
de vuelta.

```bash
rm -rf ~/Documentos/proyectos/agent-develop/plugins/reflex
git -C ~/Documentos/proyectos/agent-develop add -A && git -C ~/Documentos/proyectos/agent-develop commit -m "chore: reflex se sirve desde el monorepo exo"
```

---

## Lo que C9 NO cerró

**M6 sigue abierto.** Quedan dos items que necesitan diseño propio antes de
poder trocearse, y **M5b sigue bloqueado** hasta que estén:

- **M6-04 (kbx al índice del engine)**: mal dimensionado en la spec. `kbx` corre
  6 queries contra 4 tablas del esquema de basic-memory (`entity`, `relation`,
  `search_index`, `project`) y exo tiene otro esquema, sin equivalente de
  `project.path` ni de los `id` numéricos de los JOINs. Dos caminos: vistas de
  compatibilidad o portar las queries. Dato ya ganado: `observation` está en la
  lista `consumed` pero **no lo consulta nadie**.
- **M6-06 (recall en el punto de uso)**: fuera desde la spec §3.2.

## Deuda anotada (no bloquea)

- `engine/src/indexer.rs:161-179` — el segundo bucle de `indexa()` tiene el mismo
  shape que el bug arreglado, pero **se autocura**: los DELETE son idempotentes y
  el fichero sigue sin existir, así que la corrida siguiente vuelve a entrar. 4
  líneas cuando se toque esa función.
- **Hooks fantasma**: `stuck-loop-pretool.sh`, `cost-pyramid` y `test-run-tracker`
  no existen, pero `plugins/reflex/README.md` y `scripts/reflex-fp-adjudicate.prompt.md`
  los listan como vivos. La `description` de `recon-first` ya se arregló (era la
  única que el modelo lee en cada sesión). Decisión de Paul pendiente sobre el resto.
- `$EXO_KB` con espacios no se trimea en `compose-inject.sh` (el script previo
  tampoco sanitizaba `--kb`: no es regresión).
- `skills/consolida/SKILL.md` lleva 13 rutas absolutas a `/home/paul/…`.
- `test-reflex-baseline.sh` es el único test sin bit `+x` (venía así).

## Cerrado, no anotar como deuda

`exo write` no emite envelope JSON en el camino de rechazo: **es diseño**, no un
olvido. `engine/src/main.rs:240-245` lo dice — un gate rechazado no es un error
del sistema, sale con 3 para que el consumidor lo distinga **por exit code, jamás
parseando `data`**. Queda escrito aquí para que la próxima review no lo levante.
