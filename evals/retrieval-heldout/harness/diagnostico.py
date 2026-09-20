#!/usr/bin/env python3
"""Diagnóstico por fila de las 55 in-sample (campaña J, T0; pre-registro
borrador §2.7). DESCRIPTIVO: no decide nada, no re-etiqueta, no consume
held-out. Cruza tres estados del mismo conjunto de 55 queries:
  hist : misses del sweep sellado (metrics-engine-hybrid-b0.0-e0.6.md, 49/55,
         KB de 138 notas, 2026-07), leídos de las líneas "- MISS `...`"
  S    : captura de la campaña C (cap-base-55.jsonl, binario 4f4d2a8,
         snapshot 885246d, 174 notas) — 43/55
  hoy  : captura contra el índice de producción con el binario actual
y por fila explica el miss con las listas capturadas: rango de la nota
relevante en vector (lista completa y admitida ≥0.40), en FTS y en hybrid;
df de cada token de la query en el índice (por qué el AND da 0); cuántas
notas de archive/ y qué rotaciones de la nota esperada ocupan el top-5.

Dos salidas: --publico (ids mNN, sin texto de query ni permalinks: el repo
es público) y --privado (todo, a $PRIV_J).
Uso: diagnostico.py --gold IN55 --hist MD --cap-s CAP --cap-hoy CAP --db IDX
     --kb-hoy KB --publico OUT.md --privado OUT.md
"""
import argparse
import json
import math
import re
import sqlite3
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from metricas import K_HIT, UMBRAL_SELLADO, carga_captura, carga_gold, relevantes  # noqa: E402
from pool import normaliza  # noqa: E402
from valida_gold import permalinks_snapshot  # noqa: E402

# Filas que la auditoría del 2026-09-04 mandó re-verificar (9 SUPERADA + 2
# AMBIGUA; exo-bitacora archivada 2026-07-17_2026-09-04). Numeración 1-based
# de eval.jsonl = id mNN que asigna carga_gold (la fila 9 nula cuenta).
REVERIFICAR = {"m07", "m13", "m22", "m23", "m24", "m25", "m27", "m29", "m30", "m34", "m38"}
DF_RARO = 0.25  # F1 del borrador §4: token raro si 1 <= df <= ceil(DF_RARO * n_notas)
CAUSAS = ("vector-bajo-umbral", "vector-lejos", "fusion-desplaza", "fts-and-vacio",
          "fts-sin-relevante", "rotacion-en-top5", "etiqueta-reverificar", "captura-error")


def misses_historicos(ruta_md):
    """Prefijos (≤60 chars) de las queries MISS de un metrics-*.md de fase0."""
    return re.findall(r"^- MISS `(.*?)` → ", Path(ruta_md).read_text(encoding="utf-8"), flags=re.M)


def empareja_prefijos(prefijos, filas):
    """prefijo -> id. Gana la igualdad exacta normalizada; si no, el único
    startswith. 0 o >1 candidatos = error (no se adivina)."""
    out = {}
    for p in prefijos:
        n = normaliza(p)
        exactas = [f["id"] for f in filas if normaliza(f["query"]) == n]
        cands = exactas if len(exactas) == 1 else [f["id"] for f in filas if normaliza(f["query"]).startswith(n)]
        if len(cands) != 1:
            raise ValueError(f"prefijo sin fila única: {p!r} -> {cands}")
        out[p] = cands[0]
    return out


def rango(lista, rel):
    for i, (p, _) in enumerate(lista, start=1):
        if p in rel:
            return i
    return None


def es_rotacion(expected, permalink):
    """`<kb>/archive/log/<base>-<fechas>` es rotación de `<kb>/log/<base>`."""
    if "/archive/" in expected or "/log/" not in expected or "/archive/log/" not in permalink:
        return False
    base = expected.rsplit("/log/", 1)[1]
    return permalink.rsplit("/archive/log/", 1)[1].startswith(base + "-")


def df_tokens(conn, query):
    """df de cada token tal y como lo vería prepara_query (token entre
    comillas, comillas internas duplicadas): nº de notas con MATCH."""
    out = []
    for tok in query.split():
        frase = '"' + tok.replace('"', '""') + '"'
        try:
            n = conn.execute("SELECT count(*) FROM notas_fts WHERE notas_fts MATCH ?", (frase,)).fetchone()[0]
        except sqlite3.OperationalError:
            n = -1
        out.append((tok, n))
    return out


def diagnostica_fila(fila, cap, dfs, estricto=False):
    rel = relevantes(fila, estricto)
    exp = fila["expected_permalink"]
    d = {"id": fila["id"], "source": fila.get("source"), "hit5": False, "rango_vector": None,
         "score_vector": None, "rango_vector_adm": None, "n_fts": 0, "rango_fts": None,
         "rango_hybrid": None, "archive_top5": 0, "rotacion_top5": False, "top5": [],
         "tokens_df0": [t for t, n in dfs if n == 0], "n_tokens": len(dfs),
         "reverificar": fila["id"] in REVERIFICAR, "causas": []}
    if cap is None:
        d["causas"].append("captura-error")
        return d
    vec, fts, hyb = cap["vector"], cap["fts"], cap["hybrid"]
    vec_adm = [(p, s) for p, s in vec if s >= UMBRAL_SELLADO]
    rv, rva, rf, rh = rango(vec, rel), rango(vec_adm, rel), rango(fts, rel), rango(hyb, rel)
    sv = next((s for p, s in vec if p in rel), None)
    top5 = [p for p, _ in hyb[:K_HIT]]
    d.update(hit5=rh is not None and rh <= K_HIT, rango_vector=rv, score_vector=sv, rango_vector_adm=rva,
             n_fts=len(fts), rango_fts=rf, rango_hybrid=rh, top5=top5,
             archive_top5=sum("/archive/" in p for p in top5),
             rotacion_top5=any(es_rotacion(exp, p) for p in top5))
    if d["hit5"]:
        return d
    if sv is None or sv < UMBRAL_SELLADO:
        d["causas"].append("vector-bajo-umbral")
    elif rva is not None and rva > K_HIT:
        d["causas"].append("vector-lejos")
    elif rva is not None and rva <= K_HIT:
        d["causas"].append("fusion-desplaza")
    if len(fts) == 0:
        d["causas"].append("fts-and-vacio")
    elif rf is None:
        d["causas"].append("fts-sin-relevante")
    if d["rotacion_top5"]:
        d["causas"].append("rotacion-en-top5")
    if d["reverificar"]:
        d["causas"].append("etiqueta-reverificar")
    return d


def _f(x, nd=3):
    return "—" if x is None else (round(x, nd) if isinstance(x, float) else x)


def tokens_raros(dfs, n_notas):
    tope = math.ceil(DF_RARO * n_notas)
    return [t for t, n in dfs if 1 <= n <= tope]


def informe(filas, hist_ids, diag_s, diag_hoy, muertas_hoy, publico, dfs=None, n_notas=0):
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    ids = [f["id"] for f in no_nulas]
    miss_s = {i for i in ids if not diag_s[i]["hit5"]}
    miss_hoy = {i for i in ids if not diag_hoy[i]["hit5"]}
    n = len(ids)
    L = ["# Diagnóstico por fila de las 55 in-sample — campaña J, T0 (descriptivo; no decide)", ""]
    L.append(f"- no nulas: {n} · hist misses {len(hist_ids)} ({n - len(hist_ids)}/{n}) · S misses {len(miss_s)} ({n - len(miss_s)}/{n}) · hoy misses {len(miss_hoy)} ({n - len(miss_hoy)}/{n})")
    L.append(f"- transición hist→S: hit→miss {len(miss_s - hist_ids)} · miss→hit {len(hist_ids - miss_s)} · miss→miss {len(hist_ids & miss_s)}")
    L.append(f"- transición S→hoy: hit→miss {len(miss_hoy - miss_s)} · miss→hit {len(miss_s - miss_hoy)} · miss→miss {len(miss_s & miss_hoy)}")
    L.append(f"- etiquetas cuyo expected_permalink no existe en la KB de hoy: {len(muertas_hoy)}")
    L.append(f"- filas marcadas re-verificar (auditoría 2026-09-04): {len(REVERIFICAR & set(ids))}; de ellas miss en S: {len(REVERIFICAR & miss_s)}")
    if dfs is not None:
        con_raro = sum(1 for i in ids if tokens_raros(dfs[i], n_notas))
        L.append(f"- no nulas con ≥1 token raro (1 ≤ df ≤ ⌈{DF_RARO}·{n_notas}⌉ = {math.ceil(DF_RARO * n_notas)}): {con_raro}/{n} — filas en las que F1 (FTS OR sobre raros) tendría canal léxico; de los misses en S: {sum(1 for i in miss_s if tokens_raros(dfs[i], n_notas))}/{len(miss_s)}")
    for nombre, dg, ms in (("S", diag_s, miss_s), ("hoy", diag_hoy, miss_hoy)):
        c = Counter(x for i in ms for x in dg[i]["causas"])
        L += ["", f"## causas de los {len(ms)} misses en {nombre} (una fila puede llevar varias)", ""]
        L += [f"- {k}: {c.get(k, 0)}" for k in CAUSAS]
        L.append(f"- plazas de archive/ en el top-5 de las no nulas: {sum(dg[i]['archive_top5'] for i in ids)}/{K_HIT * n}")
        L.append(f"- no nulas con FTS vacío (AND): {sum(1 for i in ids if dg[i]['n_fts'] == 0)}/{n}")
    L += ["", "## por fila (no nulas; rv=rango vector completo, sv=score vector, rva=rango vector ≥0.40, nF=candidatos FTS, rf=rango FTS, rh=rango hybrid, a5=archive en top-5, rot=rotación de la esperada en top-5, df0=tokens con df 0 / tokens)", "",
          "| id | src | hist | S | hoy | rv_S | sv_S | rva_S | nF_S | rf_S | rh_S | a5_S | rot_S | df0 | causas_S | causas_hoy |", "|" + "---|" * 16]
    for f in no_nulas:
        i = f["id"]
        s, h = diag_s[i], diag_hoy[i]
        L.append(f"| {i} | {f.get('source')} | {'miss' if i in hist_ids else 'hit'} | {'miss' if i in miss_s else 'hit'} | {'miss' if i in miss_hoy else 'hit'} | {_f(s['rango_vector'])} | {_f(s['score_vector'])} | {_f(s['rango_vector_adm'])} | {s['n_fts']} | {_f(s['rango_fts'])} | {_f(s['rango_hybrid'])} | {s['archive_top5']} | {'sí' if s['rotacion_top5'] else 'no'} | {len(s['tokens_df0'])}/{s['n_tokens']} | {','.join(s['causas']) or '—'} | {','.join(h['causas']) or '—'} |")
    if not publico:
        L += ["", "## detalle privado (query, esperada, top-5 en S, tokens con df 0)", ""]
        for f in no_nulas:
            s = diag_s[f["id"]]
            L.append(f"- {f['id']} `{f['query']}` → `{f['expected_permalink']}` · top5_S: {s['top5']} · df0: {s['tokens_df0']}")
    return "\n".join(L) + "\n"


def main():
    ap = argparse.ArgumentParser()
    for k in ("gold", "hist", "cap-s", "cap-hoy", "db", "kb-hoy", "publico", "privado"):
        ap.add_argument(f"--{k}", required=True)
    a = ap.parse_args()
    filas = carga_gold(a.gold)
    hist_ids = set(empareja_prefijos(misses_historicos(a.hist), filas).values())
    conn = sqlite3.connect(f"file:{a.db}?mode=ro", uri=True)
    dfs = {f["id"]: df_tokens(conn, f["query"]) for f in filas}
    cap_s, cap_hoy = carga_captura(a.cap_s), carga_captura(a.cap_hoy)
    no_nulas = [f for f in filas if f.get("expected_permalink")]
    diag_s = {f["id"]: diagnostica_fila(f, cap_s.get(f["id"]), dfs[f["id"]]) for f in no_nulas}
    diag_hoy = {f["id"]: diagnostica_fila(f, cap_hoy.get(f["id"]), dfs[f["id"]]) for f in no_nulas}
    n_notas = conn.execute("SELECT count(*) FROM notas").fetchone()[0]
    perms_hoy = permalinks_snapshot(a.kb_hoy)
    muertas = [f["id"] for f in no_nulas if f["expected_permalink"] not in perms_hoy]
    Path(a.publico).write_text(informe(filas, hist_ids, diag_s, diag_hoy, muertas, True, dfs, n_notas), encoding="utf-8")
    Path(a.privado).write_text(informe(filas, hist_ids, diag_s, diag_hoy, muertas, False, dfs, n_notas), encoding="utf-8")
    print(json.dumps({"no_nulas": len(no_nulas), "miss_hist": len(hist_ids),
                      "miss_S": sum(1 for d in diag_s.values() if not d["hit5"]),
                      "miss_hoy": sum(1 for d in diag_hoy.values() if not d["hit5"]),
                      "muertas_hoy": len(muertas)}))


if __name__ == "__main__":
    main()
