# Humo del instrumento (Task 7)

- Fecha: 2026-10-26
- `claude`: 2.1.310 (Claude Code), modelo `claude-sonnet-5-5`
- Sello: n = 53.

## Humo de aislamiento (1 llamada)

- Exit 0, coste 0,0159 USD.
- Sin CLAUDE.md, KB, perfil ni reglas duras; aparece un dato de cuenta del dueño que inyecta el harness y que los flags no eliminan (no se reproduce aquí).

## Humo del etiquetador (2 llamadas, `--sello` casado)

Exit 0. `etiquetar.py` no persiste el coste por llamada; cada una quedó bajo el tope `--max-budget-usd 0.10` (sin tope alcanzado, exit 0).

| id | choque | coste |
|----|--------|-------|
| g0-66 | P | medio |
| g0-67 | P | medio |
