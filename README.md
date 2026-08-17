# exo

Framework de trabajo agéntico con memoria persistente. Tres capas:
**thin** (skills-router, hooks) → **engine** (index/search/recall/write/budget/doctor)
→ **thick** (KB markdown+frontmatter ≈OKF).

- Spec de diseño: `docs/superpowers/specs/2026-07-16-framework-unificado-design.md`
- Audit trail de consultorías: `docs/superpowers/consultas/2026-07-16-framework/`
- Plan de cierre (M2-08 → M5b): `docs/superpowers/plans/2026-08-17-cierre-exo-m2-a-m5b.md`
- Estado: M0/M1a cerrados, M2 (E1 read) al 7/9 — `exo index/rebuild/search --type fts|vector|hybrid`
  funcionando sobre la KB real. Falta `exo recall` (M2-08) y la corrida final (M2-09).
  El marketplace vivo sigue siendo agent-develop hasta M1b.
