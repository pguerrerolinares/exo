import hashlib
import json
import random
import subprocess
import sys
from collections import Counter
from pathlib import Path

CRIBA = Path(__file__).with_name("criba.py")
SEMILLA = 20261024


def run(*args):
    return subprocess.run([sys.executable, str(CRIBA), *map(str, args)],
                          capture_output=True, text=True)


def corrida(root, tarea, brazo, rep, rc):
    d = root / "corridas-criba" / tarea / f"{brazo}-r{rep}"
    d.mkdir(parents=True)
    (d / "check.rc").write_text(f"{rc}\n")


def selecciona(tmp_path, paso, brazo, casos):
    cand = tmp_path / "cand.txt"
    cand.write_text("".join(f"{t}\n" for t in casos))
    for t, rcs in casos.items():
        for i, rc in enumerate(rcs, 1):
            corrida(tmp_path, t, brazo, i, rc)
    r = run("selecciona", paso, tmp_path, cand)
    assert r.returncode == 0, r.stderr
    return r.stdout.split(), r.stderr


def test_a0_exige_cero(tmp_path):
    casos = {"t12": (1, 2), "t21": (2, 1), "t01": (0, 1), "t10": (1, 0)}
    sobreviven, _ = selecciona(tmp_path, "a0", "a0", casos)
    assert sorted(sobreviven) == ["t12", "t21"]


def test_ar_exige_uno(tmp_path):
    casos = {"t01": (0, 1), "t10": (1, 0), "t12": (1, 2), "t22": (2, 2)}
    sobreviven, _ = selecciona(tmp_path, "ar", "ar", casos)
    assert sorted(sobreviven) == ["t01", "t10"]


def test_faltante_no_sobrevive(tmp_path):
    cand = tmp_path / "cand.txt"
    cand.write_text("sola\ncompleta\n")
    corrida(tmp_path, "sola", "a0", 1, 1)
    corrida(tmp_path, "completa", "a0", 1, 1)
    corrida(tmp_path, "completa", "a0", 2, 1)
    r = run("selecciona", "a0", tmp_path, cand)
    assert r.stdout.split() == ["completa"]
    assert "incompleta" in r.stderr and "sola" in r.stderr
    assert "completa" not in r.stderr.replace("incompleta", "")


def test_principal_cuenta(tmp_path):
    sondas, neutras = tmp_path / "s.txt", tmp_path / "n.txt"
    sondas.write_text("".join(f"s{i}\n" for i in range(40)))
    neutras.write_text("".join(f"n{i}\n" for i in range(15)))
    out = tmp_path / "orden.txt"
    assert run("principal", sondas, neutras, out).returncode == 0
    filas = [tuple(l.split("\t")) for l in out.read_text().splitlines()]
    assert len(filas) == 570
    assert set(Counter(filas).values()) == {1}
    por_sonda = {(b, r) for t, b, r in filas if t == "s7"}
    assert por_sonda == {(b, r) for b in ("a0", "ar", "a2", "n") for r in "123"}
    por_neutra = {(b, r) for t, b, r in filas if t == "n7"}
    assert por_neutra == {(b, r) for b in ("a0", "a2", "n") for r in "12"}
    assert sum(1 for t, _, _ in filas if t.startswith("s")) == 480


def test_principal_recorta_a_40(tmp_path):
    sondas, neutras = tmp_path / "s.txt", tmp_path / "n.txt"
    ids = [f"s{i:02d}" for i in range(45)]
    sondas.write_text("".join(f"{i}\n" for i in ids))
    neutras.write_text("n0\n")
    out = tmp_path / "orden.txt"
    assert run("principal", sondas, neutras, out).returncode == 0
    usadas = {l.split("\t")[0] for l in out.read_text().splitlines()} - {"n0"}
    barajadas = list(ids)
    random.Random(SEMILLA).shuffle(barajadas)
    assert usadas == set(barajadas[:40])


def test_orden_determinista(tmp_path):
    cand = tmp_path / "cand.txt"
    ids = [f"t{i:02d}" for i in range(12)]
    cand.write_text("".join(f"{i}\n" for i in ids))
    outs = []
    for n in (1, 2):
        o = tmp_path / f"o{n}.txt"
        assert run("orden", "a0", cand, o).returncode == 0
        outs.append(o.read_bytes())
    assert outs[0] == outs[1]
    corr = [(t, "a0", r) for t in ids for r in (1, 2)]
    random.Random(SEMILLA).shuffle(corr)
    esperado = "".join(f"{t}\t{b}\t{r}\n" for t, b, r in corr).encode()
    assert hashlib.sha256(outs[0]).hexdigest() == hashlib.sha256(esperado).hexdigest()
    ordenado = "".join(f"{t}\ta0\t{r}\n" for t in ids for r in (1, 2)).encode()
    assert outs[0] != ordenado


def test_principal_escribe_sondas_tsv(tmp_path):
    ids = [f"g0-{i:02d}" for i in range(41)]
    pool, kb = tmp_path / "pool", tmp_path / "kb"
    pool.mkdir(); (kb / "core").mkdir(parents=True); (kb / "stable").mkdir()
    (kb / "core" / "a.md").write_text("---\ntitle: A\ntier: core\n---\ncuerpo\n")
    (kb / "stable" / "b.md").write_text("---\ntitle: B\ntier: stable\n---\ncuerpo\n")
    (kb / "stable" / "sin-tier.md").write_text("---\ntitle: C\n---\ncuerpo\n")
    (kb / "stable" / "sin-fm.md").write_text("tier: core\n---\ncuerpo\n")
    notas = {i: ("core/a.md", "stable/b.md", "stable/sin-tier.md", "no/existe.md", "stable/sin-fm.md")[n % 5] for n, i in enumerate(ids)}
    (pool / "reglas-g0.jsonl").write_text("".join(json.dumps({"id": i, "nota": notas[i], "regla": "r"}) + "\n" for i in ids))
    (pool / "reglas-g1.jsonl").write_text("")
    sondas, neutras, out = tmp_path / "s.txt", tmp_path / "n.txt", tmp_path / "orden.txt"
    sondas.write_text("".join(f"{i}\n" for i in ids)); neutras.write_text("n0\n")
    r = run("principal", sondas, neutras, out, "--pool", pool, "--kb", kb)
    assert r.returncode == 0, r.stderr
    filas = [l.split("\t") for l in (tmp_path / "sondas.tsv").read_text().splitlines()]
    usadas = {l.split("\t")[0] for l in out.read_text().splitlines()} - {"n0"}
    assert len(filas) == 40 and {f[0] for f in filas} == usadas
    esperado = {"core/a.md": "core", "stable/b.md": "stable", "stable/sin-tier.md": "desconocido", "no/existe.md": "desconocido",
                "stable/sin-fm.md": "desconocido"}
    assert all(t == esperado[notas[i]] for i, t in filas)
    assert "desconocido" in r.stderr
