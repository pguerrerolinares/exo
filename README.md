# exo

Framework de trabajo agéntico con memoria persistente. Tres capas:
**thin** (skills-router, hooks) → **engine** (hoy: `init`/`config`/`index`/`rebuild`/
`search`/`write`/`recall`, ver `exo --help` · planeado: `budget`, `doctor`) →
**thick** (KB markdown+frontmatter ≈OKF).

El engine es un binario Rust (`exo`) que se construye desde `engine/` y arranca con
`~/.exo/config.toml` — sin dependencia de `basic-memory` para funcionar (la única
lectura de `~/.basic-memory/config.json` que queda es `exo init --from-basic-memory`,
una migración explícita y de una sola vez). La capa thin (`plugins/reflex/`,
`plugins/process/`) invoca ese binario desde hooks y scripts de shell.

- Spec de diseño original: `docs/superpowers/specs/2026-07-16-framework-unificado-design.md`
- Spec de exo genérico (config propia, D8/D9): `docs/superpowers/specs/2026-08-26-exo-generico-design.md`
- Audit trail de consultorías: `docs/superpowers/consultas/`
- Plan de cierre (M2-08 → M5b): `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md`
- **Deuda abierta y hallazgos sin barrer: `docs/backlog.md`** — léelo antes de asumir
  que algo está terminado solo porque este README lo menciona.
- Estado (2026-08-26): M0, M1a y M2 (E1 read) cerrados · M4 (E2 write) cerrado —
  `exo write new|append` escribe la KB y `/documenta` ya va por el engine · M6-01/02
  hechos: `exo recall` sirve el arranque de cada sesión · **ola 1A cerrada**: el
  engine tiene config propia (`engine/src/config.rs`, precedencia
  `flag > env > config > error accionable`), cero código de producción lee ya
  `~/.basic-memory/config.json`, el envelope JSON usa claves en inglés
  (`SCHEMA_VERSION` 2) y los flags largos del CLI están en inglés (con alias
  oculto español durante el cutover — ver backlog, "Retirar los aliases
  españoles del CLI en 1.1"). Restan M6-03/04/05, el resto de M5a (MCP propio)
  y M5b (desinstalar basic-memory). Ver `docs/backlog.md` para las cuatro
  deudas que dejó abiertas la ola 1A, entre ellas que la suite de tests no es
  hermética fuera de esta máquina y que el cutover binario↔scripts no tiene
  ninguna guardia de orden todavía.

## Capa thin: el plugin `process`

`plugins/process/` es la capa de skills. Siete: brainstorm · plan · orchestrate ·
tdd · debug · verify · documenta. Sustituye a `superpowers` y a
`paul-profile:orchestrate-personal` en el uso diario.

Este repo es la **fuente de verdad** del plugin (co-evoluciona con el engine y con
sus evals de paridad en `evals/prep-m3/`), pero **no lo publica**: el catálogo vive
en el marketplace `exo` (repo `exo-plugins`, antes `agent-develop`), que lo sirve
por `git-subdir` apuntando aquí. Id de plugin: `process@exo`.

## Atribución

`process` destila el catálogo de [`obra/superpowers`](https://github.com/obra/superpowers)
— **MIT, © 2025 Jesse Vincent** — más doctrina propia. La copia literal de la licencia
está en `plugins/process/LICENSES/superpowers.LICENSE`, y el reparto skill a skill
(qué absorbe de superpowers y qué es fuente propia) en `plugins/process/README.md`.

El engine **no** contiene ni vendoriza código de basic-memory (AGPL-3.0-or-later):
el diseño se estudió, el código no se copió. Veto explícito en la spec madre.
