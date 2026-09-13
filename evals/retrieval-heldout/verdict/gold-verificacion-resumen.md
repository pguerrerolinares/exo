# Verificación adversarial del gold — campaña C (resumen publicable)

- Fecha: 2026-09-13
- Verificador: fable fresco, filesystem-only sobre snapshot `885246df3a428fa3895c209030eb90a8e254005f`
- Filas verificadas: 147 / 147 (todas)
- CORRECTO: 92 · DEFENDIBLE: 14 · CORREGIR: 41 (aplicadas: 41)
  - CORREGIR → null: 19 (todas del estrato `prompt`: la nota solo era identificable por el contexto de la sesión, no por el texto de la query)
  - CORREGIR → cambia `expected_permalink`: 6
  - CORREGIR → solo añade `acceptable_permalinks`: 16
- Por estrato (no nulas tras corregir): prompt=22 · agent-search=34 · hard=36
- Nulas (corpus negativo): 55 · filas con acceptable_permalinks: 39
- `sha256(gold.jsonl)` tras corregir: `1b21bbddd2e402ccb3dab5dd8253774c9f51749d43f6523d061fe2dd7f5fe148` (el de congelación lo fija la Task 5)
- Problemas sistémicos señalados, pendientes de decisión en la aprobación del gold: citas literales de prompts dentro de notas de la KB (ventaja léxica), y queries `hard` cortas que casan con cabeceras frente a `hard` largas que son paráfrasis.
- Veredicto completo por fila: privado (`$PRIV/gold-verificacion.md`) — el repo es público.
