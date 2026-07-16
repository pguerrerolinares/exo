# M0 — Fase 0: retrieval-eval de basic-memory — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Medir con eval set etiquetado si el retrieval de la KB se arregla cambiando el modelo de embeddings por config, y emitir las dos decisiones que gatean el engine: urgencia del write-path propio y lenguaje (Rust vs Go), adjudicadas por consultor Fable (régimen §8 de la spec).

**Architecture:** Harness de replay vía CLI de basic-memory (nunca MCP — protege el retrieval-log y el pre-registro de métrica D), 3 brazos (baseline bge-small-en / jina-es / MiniLM multilingüe), comparación pareada con gate numérico pre-registrado ANTES de tocar config, sweep de threshold offline sobre scores capturados, y atribución de misses (FTS/vector/threshold) vía búsquedas por tipo.

**Tech Stack:** Python 3 (stdlib: json, subprocess), CLI `basic-memory` 0.22.1 (fastembed 0.8.0), jq, git.

**Spec fuente:** `docs/superpowers/specs/2026-07-16-framework-unificado-design.md` §4.1 (léela antes de empezar).

## Global Constraints

- **Replay SOLO vía CLI** (`basic-memory tool search-notes`), jamás vía tools MCP en sesión Claude: el hook retrieval-logger appendearía queries sintéticas al log fuente del eval set.
- Si alguna sesión Claude hace búsquedas MCP durante M0, debe llamarse `test-*` (el FILTER de reflex-baseline.sh excluye ese prefijo).
- El gate numérico se commitea ANTES del primer cambio de config (pre-registro; cambiarlo después invalida el experimento).
- Candidatos EXACTOS: `jinaai/jina-embeddings-v2-base-es` (con `semantic_embedding_dimensions: 768` — sin ese campo el provider revienta: hardcodea 384) y `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` (384, drop-in). **PROHIBIDO** usar familia E5 o bge-m3 en este experimento (E5 exige prefijos query:/passage: que el stack no pone; bge-m3 no existe en fastembed 0.8.0). Nombre HF completo en config.
- `~/.basic-memory/config.json` se snapshotea antes de tocarlo y se restaura al estado ganador (o baseline) al cierre. Es config VIVA del sistema de Paul.
- Todo artefacto vive en `evals/retrieval-fase0/` del repo exo. Excepción: el doc de pre-registro de métrica D (`docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md`) vive en agent-develop (es de reflex) — su append de desviación se commitea ALLÍ, en commit propio.
- Gates de tareas: consultor Fable independiente con verdict-artifact commiteado (spec §8). Línea roja: nada destructivo/externo sin Paul.

## File Structure

```
exo/evals/retrieval-fase0/
├── snapshot/
│   ├── reflex-retrieval-log.jsonl    # copia congelada del log
│   └── config-baseline.json          # copia de ~/.basic-memory/config.json
├── queries.jsonl                     # {query, source: "log"|"hard"}
├── eval.jsonl                        # + expected_permalink, observation_hit
├── gate.md                           # pre-registro del gate numérico
├── harness/
│   ├── replay.py                     # ejecuta un brazo completo → results/<arm>.jsonl
│   └── analyze.py                    # paired metrics + sweep + atribución → results/metrics-<arm>.md
├── results/                          # baseline.jsonl, jina-es.jsonl, minilm.jsonl, metrics-*.md
└── verdict/                          # verdict-artifacts del consultor fable
```

---

### Task 1: Recon de la interfaz CLI y snapshot de fuentes

**Files:**
- Create: `evals/retrieval-fase0/snapshot/reflex-retrieval-log.jsonl`
- Create: `evals/retrieval-fase0/snapshot/config-baseline.json`
- Create: `evals/retrieval-fase0/snapshot/cli-interface.md`

**Interfaces:**
- Produces: los flags REALES del CLI que `replay.py` (Task 4) usará; snapshots congelados.

- [ ] **Step 1: Snapshot del log y la config (antes de nada)**

```bash
mkdir -p ~/Documentos/proyectos/exo/evals/retrieval-fase0/{snapshot,harness,results,verdict}
cp ~/.claude/reflex-retrieval-log.jsonl ~/Documentos/proyectos/exo/evals/retrieval-fase0/snapshot/
cp ~/.basic-memory/config.json ~/Documentos/proyectos/exo/evals/retrieval-fase0/snapshot/config-baseline.json
```

- [ ] **Step 2: Verificar la interfaz real del CLI**

```bash
basic-memory tool search-notes --help
basic-memory tool read-note --help
basic-memory sync --help
```

Documentar en `snapshot/cli-interface.md`: nombres exactos de flags para query, project, page-size, search-type, min-similarity y formato de salida; y el comando que dispara re-index/re-embed tras cambio de modelo. Los comandos de este plan asumen `--query`, `--project`, `--page-size`, `--search-type`, `--min-similarity`; **si difieren, se ajusta la constante CMD de replay.py (Task 4), no el resto del plan**.

- [ ] **Step 3: Smoke test del CLI con salida JSON**

```bash
basic-memory tool search-notes --project kb-demo --query "kbx" --page-size 5
```

Expected: JSON/texto con resultados que incluyen permalink, type y score. Anotar en cli-interface.md la forma exacta (campo de score, campo de type).

- [ ] **Step 4: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/snapshot
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): snapshot de log/config + interfaz CLI verificada"
```

---

### Task 2: Extracción de queries + casos duros

**Files:**
- Create: `evals/retrieval-fase0/queries.jsonl`

**Interfaces:**
- Consumes: `snapshot/reflex-retrieval-log.jsonl`
- Produces: `queries.jsonl` con líneas `{"query": str, "source": "log"|"hard"}` — Task 3 lo etiqueta.

- [ ] **Step 1: Extraer las queries únicas del log**

```bash
cd ~/Documentos/proyectos/exo/evals/retrieval-fase0
jq -r 'select(.tool=="mcp__basic-memory__search_notes") | .target' snapshot/reflex-retrieval-log.jsonl \
  | sort -u | jq -R '{query: ., source: "log"}' > queries.jsonl
wc -l queries.jsonl
```

Expected: ~46 líneas (≥40; si <40, revisar el nombre del campo en el log — puede ser `.target` u otro; ajustar el filtro jq mirando una línea real con `head -1 snapshot/reflex-retrieval-log.jsonl | jq .`).

- [ ] **Step 2: Autoría de 5-10 casos duros (consultor, no Paul — desviación registrada)**

La spec pide casos duros "de memoria de Paul"; bajo el régimen §8 los autoriza un consultor Fable para no meter a Paul en el critical path. Despachar un subagente **fable** con este brief: "Lee 10-15 notas de kb-demo (mezcla de learnings/, projects/ y log/ — vía filesystem `~/Documentos/proyectos/kb-demo/`, NO vía MCP). Escribe 5-10 queries CONCEPTUALES en castellano que un usuario haría para re-encontrar contenido de esas notas SIN usar sus keywords/codenames literales (parafrasea la idea, no el título). Son los casos donde un embedding solo-inglés duele. Devuelve JSONL: `{\"query\": ..., \"source\": \"hard\", \"author_expected\": \"<permalink de la nota fuente>\"}`."

Append al fichero:

```bash
cat casos-duros.jsonl >> queries.jsonl && rm casos-duros.jsonl
wc -l queries.jsonl
```

Expected: 51-56 líneas.

- [ ] **Step 3: Entrada PENDIENTE-PAUL no bloqueante**

Añadir al ledger de la campaña (o al informe de cierre si no hay ledger): "Paul puede aportar casos duros propios ('busqué X y no salió Y') en cualquier momento; se añaden al eval set permanente. No bloquea M0."

- [ ] **Step 4: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/queries.jsonl
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): 46 queries reales + casos duros conceptuales del consultor"
```

---

### Task 3: Etiquetado ground truth (eval.jsonl)

**Files:**
- Create: `evals/retrieval-fase0/eval.jsonl`

**Interfaces:**
- Consumes: `queries.jsonl`
- Produces: `eval.jsonl`: `{"query": str, "source": str, "expected_permalink": str|null, "notes": str}`. `expected_permalink=null` = query sin respuesta correcta en la KB (se excluye de métricas pero se documenta).

- [ ] **Step 1: Etiquetar con subagente sonnet asistido por filesystem**

Despachar un subagente **sonnet** con este brief: "Para cada línea de `evals/retrieval-fase0/queries.jsonl`: encuentra la nota de `~/Documentos/proyectos/kb-demo/` que un usuario razonable esperaría recuperar (usa grep/glob/lectura de títulos y frontmatter `permalink:`; NO uses tools MCP de basic-memory). Las `source: hard` traen `author_expected` — verifícalo, no lo copies a ciegas. Si la query es ambigua entre 2+ notas, elige la mejor y anota las alternativas en `notes`. Si no existe nota correcta, `expected_permalink: null` con el porqué. Output: eval.jsonl con el schema de arriba."

- [ ] **Step 2: Validar el schema**

```bash
cd ~/Documentos/proyectos/exo/evals/retrieval-fase0
python3 - <<'EOF'
import json
rows = [json.loads(l) for l in open("eval.jsonl")]
assert len(rows) >= 45, f"solo {len(rows)} filas"
for r in rows:
    assert set(r) >= {"query", "source", "expected_permalink"}, r
labeled = [r for r in rows if r["expected_permalink"]]
print(f"{len(rows)} filas, {len(labeled)} etiquetadas, {len(rows)-len(labeled)} null")
EOF
```

Expected: `≥45 filas` y mayoría etiquetadas.

- [ ] **Step 3: GATE — review de etiquetas por consultor Fable**

Consultor Fable fresco (no el autor de los casos duros): "Muestrea 15 etiquetas de eval.jsonl (incluye todas las `null` y 5 `hard`), verifica contra la KB real que el expected_permalink es el que un usuario querría. Verdict-artifact a `verdict/labels.md`: APROBADO o lista de correcciones." Aplicar correcciones si las hay.

- [ ] **Step 4: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/eval.jsonl evals/retrieval-fase0/verdict/labels.md
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): ground truth etiquetado + verdict de labels"
```

---

### Task 4: Harness de replay

**Files:**
- Create: `evals/retrieval-fase0/harness/replay.py`

**Interfaces:**
- Consumes: `eval.jsonl`, CLI verificado en Task 1.
- Produces: `results/<arm>.jsonl`: una fila por (query × search_type∈{hybrid,text,vector}) con top-10 `[{permalink,type,score}]`. `analyze.py` (Task 5) lo consume.

- [ ] **Step 1: Escribir replay.py**

```python
#!/usr/bin/env python3
"""Replay del eval set vía CLI de basic-memory. Uso: replay.py <arm-name>"""
import json, subprocess, sys, time
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
# Ajustar flags aquí si Task 1 encontró otra interfaz (snapshot/cli-interface.md):
CMD = ["basic-memory", "tool", "search-notes", "--project", "kb-demo",
       "--page-size", "10", "--min-similarity", "0.0"]
SEARCH_TYPES = ["hybrid", "text", "vector"]

def search(query, stype):
    out = subprocess.run(CMD + ["--query", query, "--search-type", stype],
                         capture_output=True, text=True, timeout=120)
    if out.returncode != 0:
        return {"error": out.stderr[-500:]}
    data = json.loads(out.stdout)
    results = data.get("results", data if isinstance(data, list) else [])
    return {"results": [{"permalink": r.get("permalink"), "type": r.get("type"),
                         "score": r.get("score")} for r in results[:10]]}

def main(arm):
    rows = [json.loads(l) for l in open(BASE / "eval.jsonl")]
    outpath = BASE / "results" / f"{arm}.jsonl"
    with open(outpath, "w") as f:
        for i, row in enumerate(rows):
            for stype in SEARCH_TYPES:
                t0 = time.monotonic()
                res = search(row["query"], stype)
                f.write(json.dumps({"query": row["query"], "search_type": stype,
                                    "elapsed_s": round(time.monotonic() - t0, 2), **res},
                                   ensure_ascii=False) + "\n")
                f.flush()
            print(f"[{i+1}/{len(rows)}] {row['query'][:50]}", file=sys.stderr)
    print(f"OK → {outpath}", file=sys.stderr)

if __name__ == "__main__":
    main(sys.argv[1])
```

- [ ] **Step 2: Smoke test con una query**

```bash
cd ~/Documentos/proyectos/exo/evals/retrieval-fase0
python3 - <<'EOF'
# smoke: una búsqueda directa con la CMD del harness
import subprocess, json
out = subprocess.run(["basic-memory","tool","search-notes","--project","kb-demo",
                      "--page-size","10","--min-similarity","0.0","--query","kbx","--search-type","hybrid"],
                     capture_output=True, text=True)
print(out.stdout[:400] or out.stderr[:400])
EOF
```

Expected: JSON con resultados. Si los flags fallan, corregir `CMD`/parseo con `snapshot/cli-interface.md` y repetir.

- [ ] **Step 3: Run del brazo baseline (config actual, sin tocar nada)**

```bash
python3 harness/replay.py baseline
wc -l results/baseline.jsonl
```

Expected: `filas = |eval.jsonl| × 3` (~156). Duración ~10-15 min (3.5s/llamada fría).

- [ ] **Step 4: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/harness/replay.py evals/retrieval-fase0/results/baseline.jsonl
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): harness de replay + brazo baseline capturado"
```

---

### Task 5: Análisis + estratificación observation-hits

**Files:**
- Create: `evals/retrieval-fase0/harness/analyze.py`

**Interfaces:**
- Consumes: `results/<arm>.jsonl`, `eval.jsonl`.
- Produces: `results/metrics-<arm>.md` y (con 2 args) comparación pareada; marca `observation_hit` por query en el baseline (lo usa el gate §6.3 de la spec).

- [ ] **Step 1: Escribir analyze.py**

```python
#!/usr/bin/env python3
"""Métricas de un brazo o comparación pareada. Uso: analyze.py <arm> [<arm-vs>]"""
import json, sys
from collections import defaultdict
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
K = 5
THRESHOLDS = [0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65]

def load(arm):
    rows = defaultdict(dict)
    for l in open(BASE / "results" / f"{arm}.jsonl"):
        r = json.loads(l)
        rows[r["query"]][r["search_type"]] = r.get("results", [])
    return rows

def hit(results, expected, k=K, thr=None):
    top = [r for r in results if thr is None or r.get("score") is None or r["score"] >= thr][:k]
    return any(r["permalink"] == expected for r in top)

def main():
    arm = sys.argv[1]
    gold = {json.loads(l)["query"]: json.loads(l) for l in open(BASE / "eval.jsonl")}
    labeled = {q: g for q, g in gold.items() if g["expected_permalink"]}
    data = load(arm)
    lines = [f"# metrics — {arm} (hit@{K}, {len(labeled)} queries etiquetadas)\n"]

    for stype in ["hybrid", "text", "vector"]:
        hits = [q for q, g in labeled.items() if hit(data[q].get(stype, []), g["expected_permalink"])]
        lines.append(f"- **{stype}**: {len(hits)}/{len(labeled)}")

    obs = [q for q in labeled if any(r.get("type") == "observation" for r in data[q].get("hybrid", [])[:K])]
    lines.append(f"- queries con observation-hit en top-{K} (hybrid): {len(obs)} → {sorted(obs)[:10]}")

    lines.append(f"\n## sweep de threshold (hybrid, filtro por score)")
    for t in THRESHOLDS:
        n = sum(hit(data[q]["hybrid"], g["expected_permalink"], thr=t) for q, g in labeled.items() if "hybrid" in data[q])
        lines.append(f"- thr={t}: {n}/{len(labeled)}")

    lines.append(f"\n## atribución de misses (hybrid, thr=None)")
    for q, g in sorted(labeled.items()):
        if hit(data[q].get("hybrid", []), g["expected_permalink"]):
            continue
        t = hit(data[q].get("text", []), g["expected_permalink"])
        v = hit(data[q].get("vector", []), g["expected_permalink"])
        cls = "fusion-miss" if (t or v) else "both-miss"
        lines.append(f"- MISS `{q[:60]}` → text={'HIT' if t else 'miss'} vector={'HIT' if v else 'miss'} [{cls}]")

    if len(sys.argv) > 2:
        other = load(sys.argv[2])
        fixed = [q for q, g in labeled.items()
                 if hit(data[q].get("hybrid", []), g["expected_permalink"])
                 and not hit(other[q].get("hybrid", []), g["expected_permalink"])]
        broken = [q for q, g in labeled.items()
                  if not hit(data[q].get("hybrid", []), g["expected_permalink"])
                  and hit(other[q].get("hybrid", []), g["expected_permalink"])]
        lines.append(f"\n## pareada {arm} vs {sys.argv[2]}: ARREGLA {len(fixed)} {fixed} · ROMPE {len(broken)} {broken}")

    out = BASE / "results" / f"metrics-{arm}.md"
    out.write_text("\n".join(lines) + "\n")
    print(out)

if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Correr sobre baseline y verificar salida**

```bash
cd ~/Documentos/proyectos/exo/evals/retrieval-fase0
python3 harness/analyze.py baseline && cat results/metrics-baseline.md
```

Expected: métricas por search_type, lista de observation-hit queries (debe ser >0 — si es 0, revisar el campo `type` del parseo contra cli-interface.md), sweep y atribución.

- [ ] **Step 3: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/harness/analyze.py evals/retrieval-fase0/results/metrics-baseline.md
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): análisis + baseline medido (estratificación observation-hits incluida)"
```

---

### Task 6: Pre-registro del gate (ANTES de tocar config)

**Files:**
- Create: `evals/retrieval-fase0/gate.md`
- Modify: `docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md` (append de desviación)

**Interfaces:**
- Produces: el gate que Task 8 adjudica. Inmutable tras el commit.

- [ ] **Step 1: Escribir gate.md**

```markdown
# Gate pre-registrado — M0 Fase 0 (fijado antes de cambiar config)

Métrica primaria: hit@5 hybrid sobre queries etiquetadas, comparación PAREADA vs baseline,
cada brazo en su mejor threshold del sweep (mismo procedimiento para todos los brazos).

1. GANA un candidato si: arregla ≥5 queries y rompe ≤1 (pareada).
2. SEMÁNTICA LOCAL LOAD-BEARING si: con el mejor brazo, ≥3 queries etiquetadas
   tienen HIT en vector u hybrid y MISS en text (la semántica aporta lo que FTS no puede).
   → decide lenguaje del engine (spec §4.5): load-bearing ⇒ Rust; si no ⇒ Go.
3. URGENCIA DEL ENGINE: si ningún candidato GANA y la atribución muestra misses
   mayoritariamente fusion-miss/threshold (no del modelo), el fix pertenece al motor
   propio y M2 sube de prioridad; si un candidato GANA, M2 baja a "estrangulamiento
   tranquilo" (config-fix aplicado, dolor mitigado).
4. Empate o resultado ambiguo ⇒ el consultor Fable adjudica con este texto delante;
   no se re-negocian los números post-hoc.
```

- [ ] **Step 2: Append de desviación al pre-registro de métrica D**

Añadir al final de `~/Documentos/proyectos/agent-develop/docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md` (repo agent-develop):

```markdown
## Desviación registrada 2026-07-16 (M0 Fase 0 del framework exo)
Durante la ventana se cambió el modelo de embeddings de basic-memory (experimento M0,
gate pre-registrado en evals/retrieval-fase0/gate.md). Canal causal de la métrica D
verificado como independiente (sensores = conducta Bash; doctrina viaja por system
prompt + read-note determinista). Replay vía CLI (sin hooks MCP); snapshot previo de
reflex-retrieval-log.jsonl en evals/retrieval-fase0/snapshot/.
```

- [ ] **Step 3: Commit (esto sella el pre-registro)**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/gate.md
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): gate numérico pre-registrado (sellado antes de tocar config)"
git -C ~/Documentos/proyectos/agent-develop add docs/superpowers/evals/2026-07-09-reflex-v2-baseline.md
git -C ~/Documentos/proyectos/agent-develop commit -m "docs(evals): desviación M0 registrada en pre-registro de métrica D"
```

---

### Task 7: Brazos candidatos (jina-es y MiniLM)

**Files:**
- Create: `evals/retrieval-fase0/results/{jina-es,minilm}.jsonl` + `metrics-{jina-es,minilm}.md`
- Modify: `~/.basic-memory/config.json` (temporal, con snapshot en Task 1)

**Interfaces:**
- Consumes: harness (Tasks 4-5), gate sellado (Task 6).
- Produces: métricas pareadas por candidato para el verdict (Task 8).

- [ ] **Step 1: Config → jina-es**

```bash
python3 - <<'EOF'
import json, pathlib
p = pathlib.Path.home() / ".basic-memory" / "config.json"
c = json.loads(p.read_text())
c["semantic_embedding_model"] = "jinaai/jina-embeddings-v2-base-es"
c["semantic_embedding_dimensions"] = 768
p.write_text(json.dumps(c, indent=2))
print(c["semantic_embedding_model"], c["semantic_embedding_dimensions"])
EOF
```

Expected: `jinaai/jina-embeddings-v2-base-es 768`

- [ ] **Step 2: Re-index y esperar a que termine**

```bash
basic-memory sync 2>&1 | tail -5
```

(Comando exacto según `snapshot/cli-interface.md` de Task 1. El cambio de dims recrea la tabla vectorial y re-embeddea automáticamente — ~5.154 chunks, primera vez descarga el modelo ~0.64 GB.) Verificar que terminó: una búsqueda vector devuelve resultados con score:

```bash
basic-memory tool search-notes --project kb-demo --query "doctrina de agentes" --search-type vector --page-size 3
```

Expected: ≥1 resultado con score (no vacío ni error).

- [ ] **Step 3: Replay + métricas del brazo**

```bash
cd ~/Documentos/proyectos/exo/evals/retrieval-fase0
python3 harness/replay.py jina-es
python3 harness/analyze.py jina-es baseline && cat results/metrics-jina-es.md
```

Expected: fichero de métricas con sección "pareada jina-es vs baseline: ARREGLA n ROMPE m".

- [ ] **Step 4: Repetir Steps 1-3 para MiniLM**

Config: `semantic_embedding_model = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"`, `semantic_embedding_dimensions = 384`. Luego `basic-memory sync`, `replay.py minilm`, `analyze.py minilm baseline`.

- [ ] **Step 5: Commit**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/results/
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): brazos jina-es y minilm medidos (pareada vs baseline)"
```

---

### Task 8: Verdict del consultor Fable + estado final

**Files:**
- Create: `evals/retrieval-fase0/verdict/m0-verdict.md`
- Modify: `~/.basic-memory/config.json` (estado final = ganador o baseline)
- Modify: `docs/superpowers/specs/2026-07-16-framework-unificado-design.md` §10 (decisión de lenguaje)

**Interfaces:**
- Consumes: gate.md + metrics-*.md + atribución.
- Produces: las 2 decisiones gateadas (lenguaje, urgencia M2), firmadas por el régimen §8.

- [ ] **Step 1: Despachar consultor Fable adjudicador**

Consultor **fable** fresco (no participó en tasks previas). Brief: "Adjudica el gate de `evals/retrieval-fase0/gate.md` (léelo primero, es inmutable) contra `results/metrics-*.md`. Verificación primaria: re-corre `analyze.py` tú mismo y muestrea 5 misses/hits leyendo las notas reales para confirmar que las métricas no mienten. Emite verdict-artifact a `verdict/m0-verdict.md` con: (1) ¿qué brazo gana o ninguno? (cita las cifras); (2) ¿semántica load-bearing? → lenguaje del engine per spec §4.5; (3) urgencia de M2; (4) config final a dejar aplicada. Cita textual del gate en cada decisión — sin cita, verdict inválido."

- [ ] **Step 2: Aplicar el estado final que dicte el verdict**

Si ganó un candidato: dejar su config aplicada (ya lo está si fue el último brazo, si no re-aplicar con el snippet de Task 7 Step 1). Si ninguno ganó: restaurar baseline:

```bash
cp ~/Documentos/proyectos/exo/evals/retrieval-fase0/snapshot/config-baseline.json ~/.basic-memory/config.json
basic-memory sync 2>&1 | tail -3
```

Verificar con la búsqueda smoke de Task 7 Step 2.

- [ ] **Step 3: Actualizar la spec con la decisión de lenguaje**

En `docs/superpowers/specs/2026-07-16-framework-unificado-design.md` §10, reemplazar la línea `1. **Lenguaje del engine** — output de M0...` por la decisión del verdict con fecha y link al verdict-artifact.

- [ ] **Step 4: Commit final + documentar en KB**

```bash
git -C ~/Documentos/proyectos/exo add evals/retrieval-fase0/verdict/ docs/superpowers/specs/2026-07-16-framework-unificado-design.md
git -C ~/Documentos/proyectos/exo commit -m "eval(m0): verdict fable — lenguaje del engine y urgencia de M2 adjudicados"
```

Cerrar con `/documenta` (sesión con decisiones: M0 ejecutado, resultado del gate, lenguaje firmado) — append a la bitácora del frente, delta al canon si cambia doctrina.
