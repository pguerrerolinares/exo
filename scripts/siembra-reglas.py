#!/usr/bin/env python3
"""siembra-reglas.py <pool_dir> <criba_dir> <kb> [--json] — propuesta de siembra de `## Reglas duras` (#184, spec §5.2).

Candidatas: reglas del pool de la campaña M con generica=false y ambito ∈ {codigo, git, dominio-proyecto}, asignadas a
un repo por la tabla REPO (el campo `proyecto` del pool no es la clave del repo en varios casos). Orden por repo:
  1. primero las que A0 NO cumple sin regla (A0 cumple = todas las réplicas <criba_dir>/<id>/a0-r*/check.rc dan 0;
     sin corrida, no se excluye)
  2. las que tienen `check`
  3. git/codigo antes que dominio-proyecto
  4. nota de origen tier core antes que stable
  5. orden de fichero (g0 < g1 < g2, id numérico)
Se proponen las CAP primeras; el corte de los repos que desbordan lo revisa el dueño. Solo lee: no toca la KB.
"""
import glob, json, os, re, sys

CAP = 10
AMBITOS = {"codigo", "git", "dominio-proyecto"}
# proyecto del pool -> (clave del repo = basename del toplevel de git, nota canónica relativa a projects/)
REPO = {
    "pguerrero-music": ("pguerrero-music", "pguerrero-music.md"),
    "lighthouses_aicontest": ("lighthouses_aicontest", "lighthouses-bot.md"),
    "lighthouses-bot": ("lighthouses_aicontest", "lighthouses-bot.md"),
    "agent-solve-it": ("agent-solve-it", "agent-solve-it.md"),
    "exo": ("exo", "exo — framework unificado de trabajo agéntico.md"),
    "bizkaia-now": ("bizkaia-now", "bizkaia-now — mapa de eventos de Bizkaia y banco de pruebas de Jev.md"),
    "pguerrero.me": ("pguerrero-me", "pguerrero.me — Hub personal - portfolio con Lab explorable de LLMs.md"),
    "kbx": ("kbx", "kbx — explorador determinista de la KB (Go).md"),
    "code-graph-engine": ("code-graph-engine", "cge/cge — motor code-graph en Go.md"),
    "cge": ("code-graph-engine", "cge/cge — motor code-graph en Go.md"),
    "llm-apps": ("llm-apps", "llm-apps.md"),
    "ocr-ml-docs": ("ocr-ml-docs", "ocr-ml-docs.md"),
    "jev-shadow": ("jev-shadow", "jev-shadow.md"),
    "finnk-publica-react": ("finnk-publica-react", "finnk/finnk-publica-react — repo hermano React del site público Finnk.md"),
    "open-wisdom": ("open-wisdom", "openwisdom/OpenWisdom.md"),
    "ai-news-platform": ("ai-news-platform", "ai-news-pipeline.md"),
    "a11y-crawler": ("a11y-crawler-v2", "accesibilidad-saga.md"),
    "recon": ("recon", "recon.md"),
    "foss-jam-kit": ("foss-jam-kit", "foss-jam-kit.md"),
    "wisdom-ai-news": ("wisdom-ai-news", "wisdom-ai-news.md"),
    "kbi-broker": ("kbi-broker", "finanzas-kutxabank.md"),
    "agent-develop": ("agent-develop", "agent-develop.md"),
    # Sin repo propio o con nota compartida por varios repos: no se siembran (se listan aparte).
    "CV": (None, None),
    "viz-components": (None, None),
    "portfolio-game": (None, None),
}


def orden_id(i):
    g, n = i.split("-")
    return (g, int(n))


def a0_cumple(criba, i):
    rcs = [leer(f).strip() for f in glob.glob(os.path.join(criba, i, "a0-r*", "check.rc"))]
    return bool(rcs) and all(r == "0" for r in rcs)


def leer(path):
    with open(path) as f:
        return f.read()


def tier(kb, nota):
    try:
        m = re.search(r"^tier:\s*(\S+)", leer(os.path.join(kb, nota)), re.M)
        return m.group(1) if m else "stable"
    except OSError:
        return "stable"


def es_candidata(r):
    return not r["generica"] and r["ambito"] in AMBITOS and r["proyecto"] != "transversal"


def propuesta(pool_dir, criba, kb):
    reglas = [json.loads(l) for f in sorted(glob.glob(os.path.join(pool_dir, "reglas-g*.jsonl"))) for l in leer(f).splitlines() if l.strip()]
    por_repo, sin_repo, desconocidos = {}, {}, set()
    for r in reglas:
        if not es_candidata(r):
            continue
        if r["proyecto"] not in REPO:
            desconocidos.add(r["proyecto"]); continue
        repo, nota = REPO[r["proyecto"]]
        if repo is None:
            sin_repo.setdefault(r["proyecto"], []).append(r["id"]); continue
        clave = (a0_cumple(criba, r["id"]), not r.get("check"), r["ambito"] == "dominio-proyecto",
                 tier(kb, r["nota"]) != "core", orden_id(r["id"]))
        por_repo.setdefault((repo, nota), []).append((clave, r))
    if desconocidos:
        sys.exit(f"siembra-reglas: proyectos sin entrada en REPO: {sorted(desconocidos)}")
    out = []
    for (repo, nota), cs in sorted(por_repo.items()):
        cs.sort(key=lambda x: x[0])
        out.append({"repo": repo, "nota": nota, "candidatas": len(cs), "desborda": len(cs) > CAP,
                    "reglas": [{"id": r["id"], "a0_cumple": k[0], "ambito": r["ambito"], "cita": r["cita"],
                                "regla": r["regla"], "entra": n < CAP} for n, (k, r) in enumerate(cs)]})
    return out, sin_repo


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    out, sin_repo = propuesta(*sys.argv[1:4])
    if "--json" in sys.argv:
        json.dump({"repos": out, "sin_repo": sin_repo}, sys.stdout, ensure_ascii=False, indent=1); return
    for p in out:
        print(f"== {p['repo']} ← projects/{p['nota']} ({p['candidatas']} candidatas{', DESBORDA' if p['desborda'] else ''})")
        for i, r in enumerate(p["reglas"]):
            print(f"  {'+' if r['entra'] else '-'}{i + 1:2} {r['id']:7} {'a0ok ' if r['a0_cumple'] else ''}[{r['ambito']}] {r['regla']}")
    print(f"sin repo (no se siembran): {sin_repo}")


if __name__ == "__main__":
    main()
