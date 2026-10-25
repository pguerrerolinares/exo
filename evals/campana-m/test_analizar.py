import json
import os

import pytest

import analizar as A

BR4 = ("a0", "ar", "a2", "n")


def ci(media, lo, hi):
    return {"media": media, "lo": lo, "hi": hi}


def mk(**cambios):
    """m neutro: nada dispara ninguna regla; cada test cambia lo suyo."""
    m = {
        "n_sondas": 40, "oraculo_ok": 30, "frac_sucia": 0.0,
        "d_n_a2": ci(0.0, -0.3, 0.3), "d_a2_n": ci(0.0, -0.3, 0.3),
        "d_a2_a0": ci(0.3, 0.1, 0.5), "d_n_a0": ci(0.3, 0.1, 0.5),
        "tokens_mediana": {"a2": 100, "n": 100},
        "neutro": {"a2": ci(0.0, -0.05, 0.05), "n": ci(0.0, -0.05, 0.05)},
        "desempate": {"a2": {"valor": 0.001, "sobrecarga_bytes": 6000},
                      "n": {"valor": 0.001, "sobrecarga_bytes": 6000}},
    }
    m.update(cambios)
    return m


# ---------- funciones puras ----------

def test_rc2_es_fallo():
    assert A.exito_por_sonda([0, 2, 1]) == pytest.approx(1 / 3)


def test_bootstrap_reproduce_k():
    deltas = [0.0] * 34 + [1, 1, 1, 0.5, -0.5, -0.5]
    a = {f"t{i:02d}": d for i, d in enumerate(deltas)}
    b = {k: 0.0 for k in a}
    media, lo, hi = A.delta_bootstrap(a, b, semilla=20260923)
    assert (media, lo, hi) == pytest.approx((0.0625, -0.025, 0.1625))
    # Un valor constante no pasa estas dos
    z = {f"t{i}": 0.0 for i in range(10)}
    assert A.delta_bootstrap(z, z) == (0, 0, 0)
    u = {f"t{i}": 1.0 for i in range(10)}
    assert A.delta_bootstrap(u, z) == (1, 1, 1)


def test_signos():
    a = {"x": 1, "y": 1, "z": 0, "w": 0.5}
    b = {"x": 0, "y": 0, "z": 1, "w": 0.5}
    assert A.signos(a, b) == (2, 1, pytest.approx(1.0))
    assert A.signos(b, b) == (0, 0, 1.0)


# ---------- veredicto sobre m ----------

def test_r1_gana_n():
    base = dict(d_n_a2=ci(0.0, -0.10, 0.10))
    assert A.veredicto(mk(**base, tokens_mediana={"a2": 200, "n": 100})) == ["GANA N"]
    assert A.veredicto(mk(**base, tokens_mediana={"a2": 100, "n": 200})) == ["NO CONCLUYENTE", "DESEMPATE: ninguno"]
    assert A.veredicto(mk(**base, tokens_mediana={"a2": 100, "n": 100})) == ["NO CONCLUYENTE", "DESEMPATE: ninguno"]
    # IC demasiado bajo, aunque n gaste menos
    peor = dict(d_n_a2=ci(-0.1, -0.20, 0.0), tokens_mediana={"a2": 200, "n": 100})
    assert "GANA N" not in A.veredicto(mk(**peor))
    # borde exacto: estricto
    borde = dict(d_n_a2=ci(-0.1, -0.15, 0.0), tokens_mediana={"a2": 200, "n": 100})
    assert "GANA N" not in A.veredicto(mk(**borde))


def test_r2_exo():
    assert A.veredicto(mk(d_a2_n=ci(0.20, 0.05, 0.35))) == ["EXO SOBREVIVE"]
    assert "EXO SOBREVIVE" not in A.veredicto(mk(d_a2_n=ci(0.10, 0.01, 0.2)))
    assert "EXO SOBREVIVE" not in A.veredicto(mk(d_a2_n=ci(0.20, -0.01, 0.4)))
    # bordes: Δ = 0,15 exacto sí (>=); límite inferior 0 exacto no (>)
    assert "EXO SOBREVIVE" in A.veredicto(mk(d_a2_n=ci(0.15, 0.01, 0.3)))
    assert "EXO SOBREVIVE" not in A.veredicto(mk(d_a2_n=ci(0.15, 0.0, 0.3)))


def test_invalidacion_oraculo():
    v = A.veredicto(mk(oraculo_ok=11))
    assert "CUELLO = CUMPLIMIENTO" in v
    assert not {"GANA N", "EXO SOBREVIVE", "NO CONCLUYENTE"} & set(v)
    assert "CUELLO = CUMPLIMIENTO" not in A.veredicto(mk(oraculo_ok=12))


def test_invalidacion_x_a0():
    for lo in (0.0, -0.05):
        v = A.veredicto(mk(d_a2_a0=ci(0.05, lo, 0.2), d_a2_n=ci(0.3, 0.2, 0.4)))
        assert "SOLO COSTE" in v and "EXO SOBREVIVE" not in v
    assert "SOLO COSTE" not in A.veredicto(mk(d_a2_a0=ci(0.1, 0.01, 0.2)))


def test_r3_dano_independiente_y_multiple():
    v = A.veredicto(mk(d_n_a2=ci(0.0, -0.10, 0.10), tokens_mediana={"a2": 200, "n": 100},
                       neutro={"a2": ci(0, -0.05, 0.05), "n": ci(-0.3, -0.4, -0.1)}))
    assert set(v) == {"GANA N", "DAÑO n"}
    # borde exacto del neutro: estricto
    v = A.veredicto(mk(neutro={"a2": ci(0, -0.15, 0.05), "n": ci(0, -0.05, 0.05)}))
    assert "DAÑO a2" in v and "DAÑO n" not in v


def test_ni_x_ni_n_demostraron_efecto():
    ni = ["NI X NI N DEMOSTRARON EFECTO", "RECOMENDACIÓN: memoria mínima", "DESEMPATE: ninguno"]
    # positivo, y el borde lo == 0 también cuenta como sin efecto
    assert A.veredicto(mk(d_a2_a0=ci(0.0, -0.1, 0.1), d_n_a0=ci(0.0, -0.2, 0.1))) == ni
    assert A.veredicto(mk(d_a2_a0=ci(0.05, 0.0, 0.1), d_n_a0=ci(0.0, 0.0, 0.1))) == ni
    # negativo: basta que uno demuestre efecto
    assert A.veredicto(mk(d_a2_a0=ci(0.0, -0.1, 0.1), d_n_a0=ci(0.2, 0.01, 0.3))) == ["SOLO COSTE"]
    assert A.veredicto(mk(d_a2_a0=ci(0.2, 0.01, 0.3), d_n_a0=ci(0.0, -0.1, 0.1)))[0] not in ni
    # n sin evaluar: no hay veredicto de «ni X ni N»
    v = A.veredicto(mk(d_a2_a0=ci(0.0, -0.1, 0.1), d_n_a0=None))
    assert v[0] not in ni and "SIN DATOS N-A0" in v
    # el cuello manda sobre este veredicto
    assert A.veredicto(mk(oraculo_ok=11, d_a2_a0=ci(0, -0.1, 0.1), d_n_a0=ci(0, -0.1, 0.1)))[0] == "CUELLO = CUMPLIMIENTO"


def test_ni_x_ni_n_reporta_desempate():
    gana_n = {"a2": {"valor": 0.001, "sobrecarga_bytes": 9000}, "n": {"valor": 0.002, "sobrecarga_bytes": 6000}}
    v = A.veredicto(mk(d_a2_a0=ci(0.0, -0.1, 0.1), d_n_a0=ci(0.0, -0.2, 0.1), desempate=gana_n))
    assert v == ["NI X NI N DEMOSTRARON EFECTO", "RECOMENDACIÓN: memoria mínima", "DESEMPATE: n"]


def test_coste_es_mediana_de_deltas(monkeypatch):
    # media = 40; mediana = 10. El texto del verdict usa la mediana (la media la infla una cola).
    monkeypatch.setattr(A, "_deltas", lambda a, b: {"s1": 10, "s2": 10, "s3": 100})
    assert A._coste(None, None) == {"d_tokens": 10, "n_sondas": 3}


def test_no_adjudicable_en_veredicto():
    assert A.veredicto(mk(frac_sucia=0.06)) == ["NO ADJUDICABLE"]
    assert "NO ADJUDICABLE" not in A.veredicto(mk(frac_sucia=0.05))


def test_desempate_solo_sin_reglas():
    gana_a2 = {"a2": {"valor": 0.003, "sobrecarga_bytes": 5900},
               "n": {"valor": 0.001, "sobrecarga_bytes": 8000}}
    gana_n = {"n": gana_a2["a2"], "a2": gana_a2["n"]}
    # R1 verdadera con el desempate apuntando al otro lado: manda R1, sin etiqueta de desempate
    r1 = dict(d_n_a2=ci(0.0, -0.10, 0.10), tokens_mediana={"a2": 200, "n": 100})
    assert A.veredicto(mk(**r1, desempate=gana_a2)) == ["GANA N"]
    # sin reglas: NO CONCLUYENTE, el desempate solo se etiqueta
    assert A.veredicto(mk(desempate=gana_n)) == ["NO CONCLUYENTE", "DESEMPATE: n"]
    assert A.veredicto(mk(desempate=gana_a2)) == ["NO CONCLUYENTE", "DESEMPATE: a2"]
    mixto = {"a2": {"valor": 0.003, "sobrecarga_bytes": 8000},
             "n": {"valor": 0.001, "sobrecarga_bytes": 5900}}
    assert A.veredicto(mk(desempate=mixto)) == ["NO CONCLUYENTE", "DESEMPATE: ninguno"]
    # con R3 disparada no decide
    dano = {"a2": ci(0, -0.05, 0.05), "n": ci(-0.3, -0.4, -0.1)}
    assert A.veredicto(mk(desempate=gana_n, neutro=dano)) == ["DAÑO n"]
    # sobrecarga sin dato: sin dominancia
    sin = {"a2": {"valor": 0.003, "sobrecarga_bytes": None},
           "n": {"valor": 0.001, "sobrecarga_bytes": 8000}}
    assert A.veredicto(mk(desempate=sin)) == ["NO CONCLUYENTE", "DESEMPATE: ninguno"]


def test_sin_datos_no_pasa_por_vacuidad():
    v = A.veredicto(mk(d_n_a2=None, tokens_mediana={"a2": 200, "n": 100}))
    assert "GANA N" not in v and "SIN DATOS R1" in v
    v = A.veredicto(mk(neutro={"a2": ci(0, -0.05, 0.05), "n": None}))
    assert "SIN DATOS R3" in v and "DAÑO n" not in v
    assert A.delta_bootstrap({"a": 1}, {"b": 0}) is None


def test_sin_neutras_y_sin_inyectado(tmp_path):
    r = analizar(tmp_path, {"t1": "core", "t2": "core"})
    assert r["neutro"] == {"a2": None, "n": None}
    assert "SIN DATOS R3" in r["veredicto"]
    assert r["desempate"]["a2"]["sobrecarga_bytes"] is None


def test_cociente_R():
    assert A.cociente_R(0.1, 0.6, 0.35) == pytest.approx(0.5)
    assert A.cociente_R(0.6, 0.6, 0.7) is None
    assert A.cociente_R(0.6, 0.1, 0.7) is None


# ---------- sobre un K_ROOT de fixture ----------

def run(root, tarea, brazo, rep, rc, *, tokens=1000, fin="completed", inyectado=None,
        sucia="", fuga=None, version="2.1.285 (Claude Code)",
        meta=None, base="corridas"):
    d = os.path.join(root, base, tarea, f"{brazo}-r{rep}")
    os.makedirs(d)
    if meta is None:
        meta = {"tokens_in": tokens, "usd": 1.0, "turnos": 5, "seg": 10, "fin": fin}
    json.dump(meta, open(f"{d}/meta.json", "w"))
    if rc is not None:
        open(f"{d}/check.rc", "w").write(f"{rc}\n")
    open(f"{d}/claude-version.txt", "w").write(version + "\n")
    open(f"{d}/kb-sucia.txt", "w").write(sucia)
    json.dump({"fuga": bool(fuga), "motivos": fuga or []}, open(f"{d}/fugas.json", "w"))
    if inyectado is not None:
        open(f"{d}/inyectado.txt", "wb").write(b"x" * inyectado)


def montar(tmp_path, sondas, neutras=(), res=None, **kw):
    """sondas: {id: tier}; res: {(id, brazo): [rcs]} (por defecto todo falla)."""
    root = str(tmp_path / "K")
    os.makedirs(root)
    res = res or {}
    for t in list(sondas) + list(neutras):
        brs = BR4 if t in sondas else ("a0", "a2", "n")
        reps = 3 if t in sondas else 2
        for b in brs:
            rcs = res.get((t, b), [1] * reps)
            for r, rc in enumerate(rcs, 1):
                run(root, t, b, r, rc, **kw)
    sp = str(tmp_path / "sondas.tsv")
    open(sp, "w").write("".join(f"{t}\t{tier}\n" for t, tier in sondas.items()))
    npth = str(tmp_path / "neutras.txt")
    open(npth, "w").write("".join(f"{t}\n" for t in neutras))
    return root, sp, npth


def analizar(tmp_path, sondas, neutras=(), res=None, **kw):
    root, sp, npth = montar(tmp_path, sondas, neutras, res, **kw)
    return A.analizar(root, sp, npth)


def test_estratifica_tier(tmp_path):
    S = {"c1": "core", "c2": "core", "s1": "stable", "s2": "stable"}
    res = {("c1", "n"): [0, 0, 0], ("s1", "a2"): [0, 0, 0]}
    r = analizar(tmp_path, S, res=res)
    assert r["por_tier"]["core"] == {"n": 2, "media": pytest.approx(0.5)}
    assert r["por_tier"]["stable"] == {"n": 2, "media": pytest.approx(-0.5)}
    assert r["d_n_a2"]["media"] == pytest.approx(0.0)


def test_replica_con_error_cuenta_como_fallo(tmp_path):
    root, sp, npth = montar(tmp_path, {"t1": "core"}, res={("t1", "a2"): [0, 0]})
    run(root, "t1", "a2", 3, 0, meta={"error": "sin result"})
    run(root, "t1", "a2", 4, 0, fin="max_turns")  # cortada aunque el check diera 0
    r = A.analizar(root, sp, npth)
    assert r["exito_sonda"]["a2"]["t1"] == pytest.approx(2 / 4)


def test_solo_replicas_principal(tmp_path):
    S = {"t1": "core", "t2": "core"}
    root, sp, npth = montar(tmp_path, S)
    for t in S:
        for b in BR4:
            for rep in (1, 2):
                run(root, t, b, rep, 0, base="corridas-criba")
    r = A.analizar(root, sp, npth)
    assert r["exito"]["a2"] == 0.0 and r["exito"]["ar"] == 0.0
    assert r["n_corridas"] == 24


def test_oraculo_cuenta_dos_tercios(tmp_path):
    S = {"t1": "core", "t2": "core", "t3": "core"}
    res = {("t1", "ar"): [0, 0, 1], ("t2", "ar"): [0, 1, 1], ("t3", "ar"): [1, 1, 1]}
    r = analizar(tmp_path, S, res=res)
    assert r["oraculo_ok"] == 1 and r["n_sondas"] == 3


def _sucias(tmp_path, k, tipo):
    S = {f"t{i:02d}": "core" for i in range(40)}
    root, sp, npth = montar(tmp_path, S)
    cuenta = 0
    for t in S:
        for b in BR4:
            for rep in (1, 2, 3):
                if cuenta < k:
                    d = f"{root}/corridas/{t}/{b}-r{rep}"
                    if tipo == "kb":
                        open(f"{d}/kb-sucia.txt", "w").write(" M notas/x.md\n")
                    else:
                        json.dump({"fuga": True, "motivos": ["escribe en la memoria nativa"]},
                                  open(f"{d}/fugas.json", "w"))
                    cuenta += 1
    return A.analizar(root, sp, npth)


@pytest.mark.parametrize("tipo", ["kb", "fuga"])
def test_no_adjudicable(tmp_path, tipo):
    (tmp_path / "a").mkdir()
    r25 = _sucias(tmp_path / "a", 25, tipo)
    assert r25["veredicto"] == ["NO ADJUDICABLE"]
    assert len(r25["excluidas"]) == 25
    (tmp_path / "b").mkdir()
    r24 = _sucias(tmp_path / "b", 24, tipo)
    assert "NO ADJUDICABLE" not in r24["veredicto"]
    assert len(r24["excluidas"]) == 24


def test_fuga_sin_escritura_no_cuenta(tmp_path):
    S = {"t1": "core"}
    root, sp, npth = montar(tmp_path, S)
    json.dump({"fuga": True, "motivos": ["invoca exo"]},
              open(f"{root}/corridas/t1/a0-r1/fugas.json", "w"))
    assert A.analizar(root, sp, npth)["excluidas"] == []


def test_coste(tmp_path):
    S = {"s1": "core", "s2": "core", "s3": "core"}
    root, sp, npth = montar(tmp_path, S, res={("s1", "a0"): [1, 1, 1]}, tokens=100)
    # s3: a0 sin ningún tokens_in; no puede entrar en la Δ emparejada
    for rep in (1, 2, 3):
        json.dump({"error": "sin result"}, open(f"{root}/corridas/s3/a0-r{rep}/meta.json", "w"))
    # a0 en s2: solo una réplica con tokens (las otras terminaron con error)
    for rep in (2, 3):
        d = f"{root}/corridas/s2/a0-r{rep}"
        json.dump({"error": "sin result"}, open(f"{d}/meta.json", "w"))
    json.dump({"tokens_in": 200, "fin": "completed"}, open(f"{root}/corridas/s2/a0-r1/meta.json", "w"))
    for t, tk in (("s1", 200), ("s2", 300), ("s3", 900)):
        for rep in (1, 2, 3):
            json.dump({"tokens_in": tk + 10 * (rep - 2), "fin": "completed"},
                      open(f"{root}/corridas/{t}/a2-r{rep}/meta.json", "w"))
    for i, (t, rep) in enumerate((t, r) for t in ('s1', 's2') for r in (1, 2, 3)):
        open(f"{root}/corridas/{t}/a2-r{rep}/inyectado.txt", "wb").write(b"x" * (1000 if i == 5 else 10))
    r = A.analizar(root, sp, npth)
    assert r["coste"]["a2"] == {"d_tokens": pytest.approx(100.0), "n_sondas": 2}  # sin emparejar da 125
    assert r["sobrecarga_bytes"]["a2"] == 10  # mediana; la media da 175


def test_desempate_prevalencia_campo(tmp_path):
    S = {f"t{i}": "core" for i in range(5)}
    root, sp, npth = montar(tmp_path, S, res={("t0", "a2"): [0, 0, 0]}, inyectado=100)
    r = A.analizar(root, sp, npth)
    assert r["desempate"]["a2"]["valor"] == pytest.approx(0.0024)
    assert r["desempate"]["a2"]["sobrecarga_bytes"] == 100
    assert r["desempate"]["n"]["valor"] == pytest.approx(0.0)


def test_version_discrepante(tmp_path):
    root, sp, npth = montar(tmp_path, {"t1": "core"}, res={("t1", "a2"): [0, 1, 1]})
    open(f"{root}/congelacion-principal.txt", "w").write(
        "fecha x\nmodelo claude-sonnet-5-5 · claude 2.1.285 (Claude Code)\n")
    for rep in (1, 2, 3):
        open(f"{root}/corridas/t1/a2-r{rep}/claude-version.txt", "w").write(
            "2.1.286 (Claude Code)\n" if rep == 2 else "2.1.285 (Claude Code)\n")
        mod = "claude-opus-4" if rep == 3 else "claude-sonnet-5-5"
        with open(f"{root}/corridas/t1/a2-r{rep}/transcript.jsonl", "w") as f:
            f.write(json.dumps({"type": "system", "subtype": "hook_started"}) + "\n")
            f.write(json.dumps({"type": "system", "subtype": "init", "model": mod}) + "\n")
    r = A.analizar(root, sp, npth)
    assert sorted(e["corrida"] for e in r["excluidas"]) == ["t1/a2-r2", "t1/a2-r3"]
    assert r["exito_sonda"]["a2"]["t1"] == 1.0  # solo cuenta r1 (rc 0)


def test_neutro_y_suelo_no_se_mezclan(tmp_path):
    S = {f"t{i}": "core" for i in range(10)}
    N = [f"u{i}" for i in range(5)]
    res = {}
    for t in S:                                  # suelo: n gana a a2 por mucho
        res[(t, "n")] = [0, 0, 0]
    for u in N:                                  # neutro: n se rompe frente a a0
        res[(u, "a0")] = [0, 0]
        res[(u, "a2")] = [0, 0]
    r = analizar(tmp_path, S, N, res=res)
    assert r["d_n_a2"]["media"] == pytest.approx(1.0)
    assert r["neutro"]["n"]["media"] == pytest.approx(-1.0)
    assert "DAÑO n" in r["veredicto"] and "DAÑO a2" not in r["veredicto"]
