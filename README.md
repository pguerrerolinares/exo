# exo

Framework de trabajo agéntico con memoria persistente. Tres capas:
**thin** (skills-router, hooks) → **engine** (index/search/recall/write/budget/doctor)
→ **thick** (KB markdown+frontmatter ≈OKF).

- Spec de diseño: `docs/superpowers/specs/2026-07-16-framework-unificado-design.md`
- Audit trail de consultorías: `docs/superpowers/consultas/`
- Plan de cierre (M2-08 → M5b): `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md`
- Estado: M0, M1a y M2 (E1 read) cerrados · M4 (E2 write) cerrado — `exo write new|append`
  escribe la KB y `/documenta` ya va por el engine · M6-01/02 hechos: `exo recall` sirve
  el arranque de cada sesión. Restan M6-03/04/05, M5a (MCP propio) y M5b (desinstalar
  basic-memory).

## Capa thin: el plugin `process`

`plugins/process/` es la capa de skills, servida por el marketplace declarado en
`.claude-plugin/marketplace.json` de este mismo repo (id de plugin: `process@exo`).
Siete skills: brainstorm · plan · orchestrate · tdd · debug · verify · documenta.
Sustituye a `superpowers` y a `paul-profile:orchestrate-personal` en el uso diario.

## Atribución

`process` destila el catálogo de [`obra/superpowers`](https://github.com/obra/superpowers)
— **MIT, © 2025 Jesse Vincent** — más doctrina propia. La copia literal de la licencia
está en `plugins/process/LICENSES/superpowers.LICENSE`, y el reparto skill a skill
(qué absorbe de superpowers y qué es fuente propia) en `plugins/process/README.md`.

El engine **no** contiene ni vendoriza código de basic-memory (AGPL-3.0-or-later):
el diseño se estudió, el código no se copió. Veto explícito en la spec madre.
