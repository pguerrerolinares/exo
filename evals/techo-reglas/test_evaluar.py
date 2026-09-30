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


def rc(k, i, r, v):
    d = k / "corridas" / i / f"ar-r{r}"; d.mkdir(parents=True, exist_ok=True)
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
