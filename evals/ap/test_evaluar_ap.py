"""Tests de evaluar-ap.py con datos sintéticos. N=16, K=5: r=13, k=13, s=15 (umbral.py --k 5)."""
import importlib.util, os, subprocess, sys, textwrap

import pytest

H = os.path.dirname(os.path.abspath(__file__))
_s = importlib.util.spec_from_file_location("evaluar_ap", os.path.join(H, "evaluar-ap.py"))
ev = importlib.util.module_from_spec(_s); _s.loader.exec_module(ev)
U = ev.umbral(5)


def rcs(cuentas):
    """cuentas[i] = réplicas que cumplen de 5 → {sonda: [rc]}."""
    return {f"s{i}": [0] * c + [1] * (5 - c) for i, c in enumerate(cuentas)}


def decide(ap, ap1, **kw):
    return ev.decide(rcs(ap), rcs(ap1), U, **kw)


def test_umbrales_leen_k_de_un_sitio():
    assert ev.lee_k() == 5 == U.K
    assert U.umbrales(16) == (13, 13, 15)


def test_pasa():
    o = decide([5] * 16, [5] * 16)
    assert o["veredicto"] == "Ap PASA" and o["replica"] and not o["primario_cae"] and not o["secundario_cae"]


def test_cae_por_el_pareado():
    o = decide([3] * 16, [5] * 16)  # ap pasa en las 16 (absoluto no cae) pero 0,4 por debajo del control
    assert o["ap_pasan"] == 16 and not o["secundario_cae"]
    assert o["primario_cae"] and o["delta"]["p97_5"] < 0 and o["veredicto"] == "Ap CAE"


def test_cae_solo_por_el_absoluto():
    o = decide([5] * 12 + [2] * 4, [3] * 6 + [5] * 10)  # +0,4 en 6 sondas compensa -0,6 en 4: Δ = 0
    assert o["replica"] and o["ap1_pasan"] == 16 >= o["s"]
    assert o["ap_pasan"] == 12 < o["k"] and o["secundario_cae"]
    assert not o["primario_cae"] and o["delta"]["p97_5"] >= 0
    assert o["veredicto"] == "Ap CAE"


def test_ap1_en_la_banda_con_el_absoluto_bloqueado():
    o = decide([5] * 12 + [2] * 4, [5] * 14 + [0] * 2)
    assert o["r"] <= o["ap1_pasan"] == 14 < o["s"] and o["replica"] and o["banda_ap1"]
    assert o["ap_pasan"] < o["k"] and not o["secundario_aplica"] and not o["secundario_cae"]
    assert not o["primario_cae"] and o["veredicto"] == "Ap PASA"


def test_sin_replica_por_sondas_no_se_adjudica_aunque_el_pareado_muestre_dilucion():
    o = decide([0] * 16, [5] * 12 + [2] * 4)  # ap1: 12 < r y canal 0,85
    assert o["ap1_pasan"] == 12 < o["r"] and o["ap1_exito_corrida"] >= 0.8 and not o["replica"]
    assert o["primario_cae"] and o["dilucion_pareado_sin_replica"] and not o["adjudica"]
    assert "sin réplica" in o["veredicto"]


def test_sin_replica_por_corrida_bajo_080():
    o = decide([3] * 16, [3] * 16)  # las 16 pasan (>= r) pero el éxito por corrida es 0,60
    assert o["ap1_pasan"] == 16 >= o["r"] and o["ap1_exito_corrida"] == pytest.approx(0.6)
    assert not o["replica"] and not o["adjudica"] and "sin réplica" in o["veredicto"]


def test_n_bajo_n_min_solo_se_reporta():
    o = decide([0] * 10, [5] * 10)
    assert o["N"] == 10 < o["N_MIN"] and not o["adjudica"] and o["primario_cae"]
    assert o["veredicto"].startswith("solo se reporta") and "r" not in o


def test_descriptivos_no_adjudican_y_cruzan_por_sonda():
    n = 16
    proy = {f"s{i}": f"p{i % 4}" for i in range(n)}
    cob = {f"s{i}": {"id": f"s{i}", "proyecto": proy[f"s{i}"], "n_proyecto": "9" if i < 4 else "5", "bytes": "100",
                     "pos": "0", "sim_max": "0.3"} for i in range(n)}
    tipos = {f"s{i}": "I" if i % 2 else "R" for i in range(n)}
    o = decide([5] * 16, [5] * 16, proyecto=proy, cobertura=cob, tipos=tipos)
    assert o["estrato_>=8_reglas"]["n"] == 4 and o["sensibilidad_por_proyecto"]["proyectos"] == 4
    assert set(o["por_tipo"]) == {"I", "R"} and o["veredicto"] == "Ap PASA"


def _tanda(tmp, falta=None, malo=None):
    """K_ROOT sintético con 16 cubiertas (ap y ap1 5/5) más una transversal solo en ap, y dir de experimento."""
    k, d = tmp / "k", tmp / "exp"; d.mkdir(); k.mkdir()
    ids = [f"g0-{i}" for i in range(16)] + ["g9-1"]
    (k / "reconstruccion.tsv").write_text("".join(f"{i}\tsi\tok\n" for i in ids))
    (d / "tareas.tsv").write_text("".join(
        f"{i}\tproyecto\tx\t" + ("ap:5,ap1:5" if i != "g9-1" else "ap:5") + "\n" for i in ids))
    cab = "id\tproyecto\tambito\tcubierta\tstatus\treason\tn_global\tn_proyecto\tbytes\tpos\tsim_max\ten_global\n"
    (d / "cobertura.tsv").write_text("# kb=x\n" + cab + "".join(
        f"{i}\tp{n % 3}\t{'transversal' if i == 'g9-1' else 'proyecto'}\t{'no' if i == 'g9-1' else 'si'}\tok\t\t5\t6\t900\t1\t0.3\t\n"
        for n, i in enumerate(ids)))
    for i in ids:
        for b in ("ap", "ap1"):
            if b == "ap1" and i == "g9-1":
                continue
            for r in range(1, 6):
                c = k / "corridas" / i / f"{b}-r{r}"; c.mkdir(parents=True)
                if (i, b, r) != falta:
                    (c / "check.rc").write_text("1\n" if (i, b, r) == malo else "0\n")
    return k, d


def _evalua(k, d, out):
    return subprocess.run([sys.executable, os.path.join(H, "evaluar-ap.py"), str(k), "--dir", str(d), "--out", str(out)],
                          capture_output=True, text=True)


def test_corrida_sin_rc_invalida_la_tanda(tmp_path):
    k, d = _tanda(tmp_path, falta=("g0-3", "ap1", 2))
    p = _evalua(k, d, tmp_path / "r.json")
    assert p.returncode == 1 and "tanda inválida" in p.stderr and "g0-3/ap1-r2" in p.stderr
    assert not (tmp_path / "r.json").exists()


def test_rc_distinto_de_cero_es_no_cumple_y_sale_resultado(tmp_path):
    import json
    k, d = _tanda(tmp_path, malo=("g0-3", "ap", 1))
    p = _evalua(k, d, tmp_path / "r.json")
    assert p.returncode == 0, p.stderr
    r = json.load(open(tmp_path / "r.json"))
    assert r["exito"]["g0-3"]["ap"] == pytest.approx(0.8) and r["exito"]["g0-3"]["ap1"] == 1.0
    assert r["veredicto"] == "Ap PASA" and r["transversales_ap"] == {"g9-1": 1.0}
    assert "Δ" in p.stdout and "VEREDICTO" in p.stdout
