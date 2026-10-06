import json, subprocess, sys, pathlib
import pytest
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from evaluar import evalua

SUELO = [f"s{i}" for i in range(11)]
CONTROL = [f"c{i}" for i in range(6)]


@pytest.fixture
def k(tmp_path):
    (tmp_path / "tareas.tsv").write_text("".join(f"{i}\tsuelo\tx\n" for i in SUELO) + "".join(f"{i}\tcontrol\tx\n" for i in CONTROL))
    (tmp_path / "reconstruccion.tsv").write_text("".join(f"{i}\tsi\tok\n" for i in SUELO + CONTROL))
    for i in SUELO:
        rc(tmp_path, i, 1, 0); rc(tmp_path, i, 2, 0)
    for i in CONTROL:
        rc(tmp_path, i, 1, 0)
    return tmp_path


def rc(k, i, r, v, brazo="ar"):
    d = k / "corridas" / i / f"{brazo}-r{r}"; d.mkdir(parents=True, exist_ok=True)
    if v is None:
        (d / "check.rc").unlink(missing_ok=True)
    else:
        (d / "check.rc").write_text(f"{v}\n")


def gate(k):
    return evalua(str(k), str(k / "tareas.tsv"), str(k / "resultado.json"))[1]


def rompe(k, n, v=1):
    for i in SUELO[n:]:
        rc(k, i, 1, v); rc(k, i, 2, v)


def test_seis_de_once_pasa(k):
    rompe(k, 6)
    assert gate(k) == "GATE: PASA"


def test_cinco_de_once_no_pasa(k):
    rompe(k, 5)
    assert gate(k) == "GATE: NO PASA (5/11 < 6)"


def test_una_caida_no_pasa(k):
    rc(k, "c0", 1, 1)
    assert gate(k) == "GATE: NO PASA (1/6 caídas)"


def test_ambos_criterios_dan_los_dos_motivos(k):
    rompe(k, 5); rc(k, "c0", 1, 1)
    assert gate(k) == "GATE: NO PASA (5/11 < 6; 1/6 caídas)"


def test_una_de_dos_replicas_cumple(k):
    rompe(k, 0)
    for i in SUELO[:6]:
        rc(k, i, 1, 1); rc(k, i, 2, 0)
    assert gate(k) == "GATE: PASA"


def test_corrida_ausente_no_cumple(k):
    rompe(k, 6)
    rc(k, "s0", 1, None); rc(k, "s0", 2, None)   # sin check.rc: 5/11
    assert gate(k) == "GATE: NO PASA (5/11 < 6)"
    rc(k, "s0", 1, 0)
    rc(k, "c1", 1, None)                          # control ausente: caída
    assert gate(k) == "GATE: NO PASA (1/6 caídas)"


def test_no_reconstruible_no_cumple_ni_cae(k):
    rompe(k, 6)
    rec = (k / "reconstruccion.tsv").read_text().replace("s0\tsi", "s0\tno").replace("c2\tsi", "c2\tno")
    (k / "reconstruccion.tsv").write_text(rec)
    assert gate(k) == "GATE: NO PASA (5/11 < 6; 1/6 caídas)"


def test_rc2_no_cumple(k):
    rompe(k, 6)
    rc(k, "s0", 1, 2); rc(k, "s0", 2, 2)
    assert gate(k) == "GATE: NO PASA (5/11 < 6)"


def test_cli_ultima_linea_y_json(k):
    out = subprocess.run([sys.executable, str(pathlib.Path(__file__).parent / "evaluar.py"), str(k), str(k / "tareas.tsv"), str(k / "r.json")],
                         capture_output=True, text=True, check=True).stdout.strip().splitlines()
    assert out[-1] == "GATE: PASA"
    assert json.load(open(k / "r.json"))["gate"] == "GATE: PASA"


def test_grupo_desconocido_falla_claro(k):
    (k / "tareas.tsv").write_text((k / "tareas.tsv").read_text().replace("c0\tcontrol", "c0\tcontrl"))
    with pytest.raises(SystemExit, match="grupo desconocido"):
        gate(k)


def test_conteos_distintos_de_11_y_6_fallan(k):
    lineas = (k / "tareas.tsv").read_text().splitlines()
    (k / "tareas.tsv").write_text("\n".join(lineas[1:]) + "\n")
    with pytest.raises(SystemExit, match="11 suelo y 6 control"):
        gate(k)


def test_tarea_ausente_de_reconstruccion_no_cumple(k):
    rompe(k, 6)
    rec = "".join(l for l in (k / "reconstruccion.tsv").read_text().splitlines(True) if not l.startswith("s0\t"))
    (k / "reconstruccion.tsv").write_text(rec)
    assert gate(k) == "GATE: NO PASA (5/11 < 6)"


def test_tabla_muestra_fin_y_usd(k):
    d = k / "corridas" / "s0" / "ar-r1"
    (d / "meta.json").write_text('{"fin":"completed","usd":1.5}')
    out = subprocess.run([sys.executable, str(pathlib.Path(__file__).parent / "evaluar.py"), str(k), str(k / "tareas.tsv"), str(k / "r.json")],
                         capture_output=True, text=True, check=True).stdout
    assert "completed" in out and "1.50" in out


def test_sin_reconstruccion_tsv_falla_explicito(k):
    (k / "reconstruccion.tsv").unlink()
    with pytest.raises(SystemExit, match="reconstruccion.tsv"):
        gate(k)


# --- v2: brazos, base y margen, control 2/2 ---
REAL_K = pathlib.Path.home() / ".cache" / "exo-ablacion-k"
S10, C6 = SUELO[:10], CONTROL


@pytest.fixture
def k2(tmp_path):
    """10 suelo (arp:2,a0:2) + 6 control (arp:2); arp cumple los 10, a0 ninguno, controles ok."""
    (tmp_path / "tareas.tsv").write_text("".join(f"{i}\tsuelo\tx\tarp:2,a0:2\n" for i in S10)
                                         + "".join(f"{i}\tcontrol\tx\tarp:2\n" for i in C6))
    (tmp_path / "reconstruccion.tsv").write_text("".join(f"{i}\tsi\tok\n" for i in S10 + C6))
    for i in S10:
        for r in (1, 2):
            rc(tmp_path, i, r, 1, "a0"); rc(tmp_path, i, r, 0, "arp")
    for i in C6:
        for r in (1, 2):
            rc(tmp_path, i, r, 0, "arp")
    return tmp_path


def gate2(k):
    return evalua(str(k), str(k / "tareas.tsv"), str(k / "r.json"), "arp", "a0", 3, (10, 6))[1]


def arp_a_6(k):
    for i in S10[6:]:
        for r in (1, 2):
            rc(k, i, r, 1, "arp")


def a0_cumple(k, n):
    for i in S10[:n]:
        rc(k, i, 1, 0, "a0")


def test_margen_a0_4_arp_6_no_pasa(k2):
    arp_a_6(k2); a0_cumple(k2, 4)
    assert gate2(k2) == "GATE: NO PASA (margen 6\u22124=2 < 3)"


def test_margen_a0_3_arp_6_pasa(k2):
    arp_a_6(k2); a0_cumple(k2, 3)
    assert gate2(k2) == "GATE: PASA"


def test_control_k2_una_replica_ok_no_cae(k2):
    arp_a_6(k2); a0_cumple(k2, 3)
    rc(k2, "c0", 1, 1, "arp")                       # r1=1, r2=0: no cae
    assert gate2(k2) == "GATE: PASA"
    rc(k2, "c1", 1, 1, "arp"); rc(k2, "c1", 2, 1, "arp")   # 1/1: cae
    assert gate2(k2) == "GATE: NO PASA (1/6 caídas)"


def test_a0_cumple_misma_regla(k2):
    arp_a_6(k2)
    rc(k2, "s0", 1, 0, "a0"); rc(k2, "s0", 2, 1, "a0")
    rc(k2, "s1", 1, 1, "a0"); rc(k2, "s1", 2, 0, "a0")
    rc(k2, "s2", 1, 0, "a0"); rc(k2, "s2", 2, 0, "a0")
    assert evalua(str(k2), str(k2 / "tareas.tsv"), str(k2 / "r.json"), "arp", "a0", 3, (10, 6))[2][0]["resultado"] == "cumple"
    assert json.load(open(k2 / "r.json"))["base_cumplen"] == 3


def test_esperado_v2_distinto_de_v1_falla(k2):
    with pytest.raises(SystemExit, match="11 suelo y 6 control"):
        evalua(str(k2), str(k2 / "tareas.tsv"), str(k2 / "r.json"), "arp", "a0", 3)


@pytest.mark.skipif(not (REAL_K / "corridas").is_dir(), reason="sin datos reales del v1")
def test_v1_identico(tmp_path):
    aqui = pathlib.Path(__file__).parent
    out = subprocess.run([sys.executable, str(aqui / "evaluar.py"), str(REAL_K), str(aqui / "tareas.tsv"), str(tmp_path / "r.json")],
                         capture_output=True, text=True, check=True).stdout
    assert out == (aqui / "resultado-tabla.txt").read_text()
