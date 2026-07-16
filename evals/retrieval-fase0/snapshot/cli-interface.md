# Basic Memory CLI Interface — Snapshot Congelado

## Resumen de cambios vs. asumido en plan

El brief asumía flags específicos (`--query`, `--project`, `--page-size`, `--search-type`, `--min-similarity`). Este documento reporta los flags REALES y su mapeo.

## search-notes: Interfaz Real

### Firma

```bash
basic-memory tool search-notes [OPTIONS] [QUERY]
```

### Argumentos

- `[QUERY]`: STRING — argumento posicional (opcional si se usan filtros de metadata).

### Flags documentados

| Flag | Tipo | Descripción |
|------|------|-------------|
| `--project` | TEXT | Proyecto a consultar; si no se proporciona, usa default |
| `--project-id` | TEXT | UUID del proyecto (precedencia sobre `--project`) |
| `--page` | INTEGER | Número de página [default: 1] |
| `--page-size` | INTEGER | Resultados por página [default: 10] |
| `--vector` | BOOLEAN | Usar vector retrieval |
| `--hybrid` | BOOLEAN | Usar hybrid retrieval |
| `--permalink` | BOOLEAN | Buscar en valores de permalink |
| `--title` | BOOLEAN | Buscar en valores de título |
| `--tag` | TEXT | Filtro por tag en frontmatter (repetible) |
| `--status` | TEXT | Filtro por status en frontmatter |
| `--type` | TEXT | Filtro por type en frontmatter (repetible) |
| `--entity-type` | TEXT | Filtro por tipo de entidad: entity, observation, relation (repetible) |
| `--category` | TEXT | Filtro por categoría de observation (repetible) |
| `--meta` | TEXT | Filtro por metadata key=value en frontmatter (repetible) |
| `--filter` | TEXT | JSON metadata filter avanzado |
| `--after_date` | TEXT | Filtro de fecha: '2d', '1 week', etc. |
| `--local` | BOOLEAN | Fuerza routing local (ignora cloud mode) |
| `--cloud` | BOOLEAN | Fuerza routing cloud |

**IMPORTANTE**: No existe un flag `--query` ni `--search-type` ni `--min-similarity` a nivel de CLI. El `query` es argumento posicional.

Comando que dispara re-embed tras cambio de modelo:
```bash
basic-memory reindex [--model <nombre>]
```

### Forma de la salida JSON

```json
{
  "results": [
    {
      "title": "string",
      "type": "string (entity|observation|relation)",
      "score": "float (score de similaridad)",
      "entity": "string (full path de la entidad)",
      "permalink": "string (permalink universal)",
      "content": "string (contenido truncado o completo)"
    }
  ]
}
```

**Campos clave**:
- `score`: float, descending (relevancia)
- `type`: string, valores observados: "entity", "observation", "relation"
- `permalink`: string, formato `{project}/{path}`

### Comando de status/re-index

```bash
basic-memory status                    # Ver estado de sincronización
basic-memory reindex                   # Reconstruir índices sin borrar DB
basic-memory reindex --model <name>    # Re-embed con modelo específico
```

## read-note: Interfaz Real

### Firma

```bash
basic-memory tool read-note [OPTIONS] IDENTIFIER
```

### Argumentos

- `IDENTIFIER`: STRING (required) — nombre o permalink de la nota

### Flags

| Flag | Tipo | Descripción |
|------|------|-------------|
| `--project` | TEXT | Proyecto a consultar |
| `--project-id` | TEXT | UUID del proyecto |
| `--include-frontmatter` | BOOLEAN | Incluir YAML frontmatter en output |
| `--local` | BOOLEAN | Fuerza local routing |
| `--cloud` | BOOLEAN | Fuerza cloud routing |

## Ejemplo de invocación funcional completa

```bash
# Búsqueda con query posicional, page-size, project explícito
basic-memory tool search-notes --project kb-demo "kbx" --page-size 5

# Con vector search activo
basic-memory tool search-notes --project kb-demo --vector "agent development" --page-size 10

# Leer nota específica
basic-memory tool read-note kb-demo/log/kbx-bitacora --include-frontmatter

# Re-index para refrescar embeddings
basic-memory reindex
```

## Cambios para replay.py (Task 4)

La constante `CMD` en replay.py debe ajustarse:

```python
# ANTES (asumido):
CMD = [
    "basic-memory", "tool", "search-notes",
    "--project", project,
    "--query", query,
    "--page-size", str(page_size),
    "--search-type", search_type,
    "--min-similarity", str(min_similarity)
]

# AHORA (correcto):
CMD = [
    "basic-memory", "tool", "search-notes",
    "--project", project,
    query,  # ARGUMENTO POSICIONAL
    "--page-size", str(page_size)
    # --search-type y --min-similarity NO EXISTEN en CLI
]
```

## Notas de arquitectura

1. **Vector vs. Hybrid**: se controlan con flags booleanos (`--vector`, `--hybrid`), no con un parámetro nombrado `search-type`.
2. **Min-similarity**: no tiene control a nivel CLI; está hardcodeado en config (ej: 0.55 en config-baseline.json).
3. **Proyectos**: El CLI acepta `--project-id` para disambiguar proyectos de mismo nombre en cloud.
4. **Routing**: Flags `--local` / `--cloud` controlan si se usa API local o cloud.

## Status del snapshot

- **Fecha**: 2026-07-16T20:00:00Z (approx)
- **Versión basic-memory**: verificada en vivo
- **Proyecto de test**: kb-demo
- **Config de referencia**: config-baseline.json (snapshot)
- **Log de ref**: reflex-retrieval-log.jsonl (snapshot)
