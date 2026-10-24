import json
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from validar_tareas import valida  # noqa: E402

SANITY_OK = {"vacio_rc": 2, "cumple_rc": 0, "viola_rc": 1}
SETUP_BASE = '#!/usr/bin/env bash\nset -e\nmkdir -p "$1"; cd "$1"; git init -q; touch desde_setup\n' \
             'git -c user.name=t -c user.email=t@x add -A; git -c user.name=t -c user.email=t@x commit -qm i\n'
# El rc lo dicta el fichero `rc` del workdir; sin él, se comporta como `sin_rc`.
CHECK = '#!/usr/bin/env bash\nrc=$(cat "$1/rc" 2>/dev/null) || exit {sin_rc}\nexit "$rc"\n'


def tarea(tmp_path, meta=None, sin_rc=2, cumple=0, viola=1, check=None, nombre="t1"):
    d = tmp_path / nombre
    (d / "pruebas" / "cumple").mkdir(parents=True, exist_ok=True)
    (d / "pruebas" / "viola").mkdir(exist_ok=True)
    (d / "pruebas" / "cumple" / "rc").write_text(str(cumple))
    (d / "pruebas" / "viola" / "rc").write_text(str(viola))
    (d / "setup.sh").write_text(SETUP_BASE)
    (d / "check.sh").write_text(check or CHECK.format(sin_rc=sin_rc))
    for f in ("setup.sh", "check.sh"):
        (d / f).chmod(0o755)
    base = {"regla_id": nombre, "setup": True, "prompt": "Arregla el parser.",
            "check": {"tipo": "script"}, "sanity": SANITY_OK}
    base.update(meta or {})
    base = {k: v for k, v in base.items() if v is not None}
    (d / "tarea.json").write_text(json.dumps(base))
    return d


def test_schema(tmp_path):
    assert valida(tarea(tmp_path)) is None
    assert valida(tarea(tmp_path, {"prompt": None})) == "falta prompt"
    assert valida(tarea(tmp_path, {"sanity": None})) == "falta sanity"
    nc = tmp_path / "nc"
    nc.mkdir()
    (nc / "tarea.json").write_text(json.dumps(
        {"regla_id": "nc", "no_convertible": True, "motivo": "sin check objetivo"}))
    assert valida(nc) is None
    (nc / "tarea.json").write_text(json.dumps({"regla_id": "nc", "no_convertible": True}))
    assert valida(nc) == "falta motivo"


def test_sanity_reproducido(tmp_path):
    # `sanity` declara lo correcto pero el check real devuelve otra cosa.
    assert valida(tarea(tmp_path, cumple=1)) == "sanity cumple=1"
    assert valida(tarea(tmp_path / "b", viola=0)) == "sanity viola=0"


def test_vacio_no_rompe(tmp_path):
    assert valida(tarea(tmp_path, sin_rc=127)).startswith("vacio rc=127")
    assert valida(tarea(tmp_path / "b", sin_rc=1)) is None


def test_prompt_sin_regla(tmp_path):
    for p in ("mira la KB", "Revisa la MEMORIA.", "una nota, rápida", "la Doctrina", "usa exo!", "(exo)"):
        assert valida(tarea(tmp_path / p[:4], {"prompt": p})) == "prompt menciona la memoria", p
    for p in ("anotación", "el notario", "valor exógeno", "las memorias de calidad"):
        assert valida(tarea(tmp_path / p[:4], {"prompt": p})) is None, p


def test_setup_usa_temporal_no_repo_real(tmp_path):
    real = tmp_path / "real"
    real.mkdir()
    (real / "solo_real").write_text("x")
    log = tmp_path / "log.txt"
    chk = f'#!/usr/bin/env bash\nls -A "$1" >> {log}\nexit 2\n'
    d = tarea(tmp_path / "w", {"repo": str(real)}, check=chk)
    # cumple/viola devuelven 2 con este check; lo que interesa es qué workdir vio para el repo base
    valida(d)
    vistos = log.read_text()
    assert "desde_setup" in vistos and "solo_real" not in vistos
    assert (real / "solo_real").exists() and not (real / "desde_setup").exists()


def test_rubrica_solo_schema(tmp_path):
    d = tmp_path / "r"
    d.mkdir()

    def con(items):
        (d / "tarea.json").write_text(json.dumps({"regla_id": "r", "prompt": "Haz X.",
            "check": {"tipo": "rubrica", "items": items}}))
        return valida(d)

    assert con(["a"]) is None and con(["a", "b", "c"]) is None
    assert con([]) == "rubrica: items debe tener de 1 a 3"
    assert con(["a", "b", "c", "d"]) == "rubrica: items debe tener de 1 a 3"


def test_cli_sale_1_si_alguna_ko(tmp_path):
    tarea(tmp_path, nombre="a_ok")
    tarea(tmp_path, {"prompt": None}, nombre="b_ko")
    r = subprocess.run([sys.executable, str(Path(__file__).parent / "validar_tareas.py"), str(tmp_path)],
                       capture_output=True, text=True)
    assert r.returncode == 1
    assert r.stdout.splitlines() == ["a_ok\tok", "b_ko\tKO\tfalta prompt"]
