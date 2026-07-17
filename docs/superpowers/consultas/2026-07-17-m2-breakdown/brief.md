# Brief — Consultor Fable: adjudicación del breakdown de M2 (E1 read), framework exo

## Rol
Eres el consultor Fable del régimen de gates delegado del proyecto exo. Paul ha abierto GATE-HUECO-M2 hoy (2026-07-17) y ha delegado en ti las decisiones de diseño del breakdown de M2; él firma tus adjudicaciones. Tu deliverable es un veredicto escrito con una adjudicación FIRMADA por decisión (no un menú de opciones): elige, razona corto, deja el trade-off explícito.

## Contexto obligatorio (léelo, no lo asumas)
Repo: `/home/paul/Documentos/proyectos/exo` (branch main, limpio).
- `docs/superpowers/specs/2026-07-16-framework-unificado-design.md` — spec madre. Secciones críticas: §4.2 (componentes), §4.3 (recortes v1), §4.4 (estrangulamiento E1), §4.5 (por qué Rust), §6.2 (reglas duras del indexer), §7 (roadmap, M2 = 8-15 noches), §8 (ejecución con fábrica, routing de lanes).
- `.superpowers/fabrica/config.md` — gates y lanes. GATE-HUECO-M2 ya abierto (commit reciente). Nota: M2 NO está gateado por GATE-CALENDARIO-D, pero cualquier cosa que altere marketplace/skills/recall del agente SÍ lo está — E1 es side-by-side, sin cutover (el cutover del hook de recall es M6).
- `evals/retrieval-fase0/` — eval set permanente (56 queries etiquetadas, ground truth a nivel de nota), `gate.md` (cómo se pre-registró el gate de M0), `verdict/m0-verdict.md` (verdict que firmó Rust y jina-es/768/threshold 0.35), `harness/` (replay.py, analyze.py, stratify.py, run-arm.sh — Python, funcionan hoy contra basic-memory CLI).
- Config de producción actual de basic-memory: modelo `jinaai/jina-embeddings-v2-base-es`, 768 dims, threshold 0.35, KB kb-demo (~117 entidades, ~5.154 chunks).

## Restricciones firmadas (NO adjudicables — viólalas y el veredicto es inválido)
1. Lenguaje: Rust (verdict M0). fastembed-rs, rusqlite+FTS5 bundled, sqlite-vec **pineado**, sin daemon.
2. Veto AGPL: diseño de la fusión de basic-memory sí (fórmula max(v,f)+bonus·min(v,f), clave (type,id), gate FTS, normalización BM25, threshold configurable), código JAMÁS (ni vendorizar).
3. E1 es read-only y side-by-side: corpus idéntico a basic-memory (exclusión dotdirs replicada, archive/ SE indexa, 5 entidades no-markdown fuera, permalinks honrados JAMÁS regenerados — §6.2 completo). Sin cutover de nada instalado.
4. Recortes v1 (§4.3): sin gramática observations/relations en el índice, sin move, sin build_context, sin cloud/sync, sin daemon; `rebuild` comando de primera clase.
5. Side-by-side >4 semanas sin decisión ⇒ se decide, no se cohabita.
6. Propiedad protegida: cada milestone deja el sistema funcionando.

## Definiciones operativas (para que no adjudiques a ciegas)
- **Lane mecánica**: item con oráculo ejecutable por comando (test/score/diff); el executor itera hasta verde sin gate humano; gate ligero al merge. **Lane diseño**: spec-first + gold set definido antes de implementar + review adversarial contra el gold + gate de consultor fable al merge. (Definición del skill fabrica §1 y config §Lanes.)
- **"Engine capaz de servir el recall, demostrado"** (deliverable E1): demostración medible SIN cutover — (a) side-by-side sobre el eval set con gate D4 pasado, (b) el binario invocable con el shape de output que el hook de recall necesitará, (c) latencia de arranque compatible con hooks (ms). El cutover real del hook es M6; E1 no toca nada instalado.
- **Gotcha observations (M0), consecuencia operativa**: el CLI de basic-memory agrega las filas observation dentro del resultado de su entidad; por eso cualquier medición de paridad de corpus/estratificación se hace contra el índice SQLite de basic-memory con probe read-only, nunca contra el output del CLI. No es un problema de escritura; es dónde mides.
- **Comparación pareada**: por-query sobre las mismas 55 queries del eval set — cuántas queries que basic-memory acierta pierde el engine (rotas) y cuántas que basic-memory falla gana el engine (arregladas). Igual que jina-vs-baseline en M0.

## Decisiones a adjudicar
- **D1 — Layout del código y nombre del binario** (van acopladas a propósito: adjudica ambas): `engine/` como crate único con binario `exo` y subcomandos vs cargo workspace (lib+bin) desde día 1; y nombre `exo` vs `engine` (la mención `engine search --json` en §M6 de la spec es ilustrativa, NO fija nomenclatura — el nombre queda libre y los scripts de M6 se escribirán contra el nombre que elijas). Considera: E3 meterá MCP (rmcp); kbx (Go) convive en el monorepo.
- **D2 — Breakdown de M2 en items ordenados + routing de lane por item**: qué items, en qué orden, y cuáles van a lane mecánica (oráculo por comando) vs lane diseño (spec-first + gold + review adversarial). El config ya fija: indexer = diseño (gold = paridad de permalinks/corpus), fusión/calibración = diseño (gold = eval set M0); gran parte del resto = mecánica. Hace falta el grano fino: scaffold, parser frontmatter, FTS5, embeddings/vectores, grafo de links, recall de SessionStart, CLI/envelope JSON, harness side-by-side.
- **D3 — Harness side-by-side**: reusar el harness Python de M0 añadiendo un arm que invoque el binario Rust (`--json`) vs harness nuevo. Define también el oráculo de paridad de corpus del indexer (comparación de sets de permalinks/chunks contra el índice SQLite de basic-memory, probe RO — ver gotcha: los resultados del CLI agregan observations a entidad; la estratificación se mide contra el índice, no contra output del CLI).
- **D4 — Gate numérico de cierre de M2, pre-registrado ANTES de correr el side-by-side**: define la métrica y el umbral (referencia: basic-memory hoy = hybrid 43/55 en el eval set; M0 usó comparación pareada "≥X arregladas, ≤Y rotas", no proporciones). Debe incluir paridad de corpus del indexer Y calidad de retrieval, y el criterio de "engine CAPAZ de servir el recall, demostrado".
- **D5 — Alcance de la primera campaña de fábrica**: qué subconjunto de items entra en la primera noche/campaña. La reserva de gates fable vigente y su mecánica están en `.superpowers/fabrica/config.md` (léela); aprendizaje M1a: 60% de la reserva se fue en gates ⇒ adjudica también si la subes y a cuánto para esta campaña.
- **D6 — Config del engine**: dónde vive la config (modelo/dims/threshold/ruta KB) y formato. El punto a adjudicar es el acoplamiento: ¿el engine LEE la config viva de basic-memory (`~/.basic-memory/config.json`, read-only) mientras dure el side-by-side, o tiene config propia desde el día 1 (duplicando modelo/dims/threshold con riesgo de divergencia silenciosa entre arms)? Considera que el binario correrá desde hooks (arranque ms) y que en M5b basic-memory desaparece.

## Delta de estándares tácitos de Paul (aplícalos como criterio de adjudicación)
- YAGNI despiadado; odia el over-engineering. Solución que funciona hoy > arquitectura perfecta. Si dudas entre simple y preparado-para-el-futuro, simple.
- Gates numéricos pre-registrados ANTES de mirar resultados (patrón M0); comparación pareada > proporciones.
- Reuso > reescritura (el harness M0 ya existe y está validado; tirar herramienta validada necesita justificación fuerte).
- Evidencia > vibes: cada claim del veredicto con dato o cita de spec (formato `§X` o ruta:línea).
- Trade-offs explícitos: en cada adjudicación, una línea de "qué pierdo eligiendo esto".
- Rigor de atribución (gotchas M0): reindex purga filas rancias; comparar contra baseline pre-purge sobreestima; search_type explícito para atribuir misses.

## Memory packet (basic-memory, proyecto kb-demo — tienes acceso al MCP `basic-memory` vía ToolSearch; usa read_note con estos permalinks si necesitas más contexto)
- `kb-demo/projects/exo-framework-unificado-de-trabajo-agentico` — destilado del proyecto (estado, decisiones raíz, gotchas M0).
- `kb-demo/core/doctrina-agentes` — régimen de gates delegado, doctrina de campañas.
- `kb-demo/log/exo-bitacora` — bitácora de sesiones exo (detalle M0/M1a).
- `kb-demo/backlog-frentes-abiertos` — contexto de calendario (universidad, cge P2).

## Formato del veredicto (tu output final, markdown)
Por cada D1-D6: **Adjudicación** (una frase imperativa) · **Racional** (2-4 frases con citas) · **Trade-off aceptado** (1 frase). Al final: sección "Riesgos que el orquestador debe vigilar" (máx 5 bullets) y "Preguntas que SÍ requieren a Paul" (solo si tocan línea roja: destructivo/externo/agenda; si no hay, dilo explícitamente).
