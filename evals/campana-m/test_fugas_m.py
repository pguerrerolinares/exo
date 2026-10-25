import json, os, subprocess, sys
import pytest

FUGAS = os.path.join(os.path.dirname(__file__), "..", "ablacion-k", "harness", "fugas.py")
SNAP_MSG = "escribe en el snapshot de la KB"
MEM_MSG = "escribe en la memoria nativa"


@pytest.fixture
def mundo(tmp_path):
    k = tmp_path / "k"
    o = tmp_path / "corrida"
    (k / "prep" / "kb").mkdir(parents=True)
    (o / "memoria").mkdir(parents=True)
    return str(k), str(o)


def tool(name, **inp):
    return {"type": "assistant", "message": {"content": [{"type": "tool_use", "id": "t1", "name": name, "input": inp}]}}


def hook(ev):
    return {"type": "system", "subtype": "hook_started", "hook_event": ev}


def correr(mundo, brazo, eventos):
    k, o = mundo
    with open(os.path.join(o, "transcript.jsonl"), "w") as f:
        for e in eventos:
            f.write(json.dumps(e) + "\n")
    p = subprocess.run([sys.executable, FUGAS, o, brazo], env={**os.environ, "K_ROOT": k}, capture_output=True, text=True)
    return p.returncode, json.loads(p.stdout)


def test_n_sin_hooks_limpio(mundo):
    rc, r = correr(mundo, "n", [tool("Bash", command="ls")])
    assert (rc, r["fuga"]) == (0, False)
    rc, r = correr(mundo, "n", [hook("SessionStart")])
    assert r["fuga"] is True and r["motivos"] == ["hook no cableado: SessionStart"]


def test_n_invoca_exo_es_fuga(mundo):
    rc, r = correr(mundo, "n", [tool("Bash", command="exo search x")])
    assert r["motivos"] == ["invoca exo sin bloqueo del stub"]
    rc, r = correr(mundo, "n", [tool("Bash", command="ls")])
    assert r["fuga"] is False


@pytest.mark.parametrize("brazo", ["a2", "n"])
@pytest.mark.parametrize("herr", ["Write", "Edit"])
def test_escritura_snapshot(mundo, brazo, herr):
    k, o = mundo
    rc, r = correr(mundo, brazo, [tool(herr, file_path=f"{k}/prep/kb/x.md")])
    assert rc == 1 and SNAP_MSG in r["motivos"]
    rc, r = correr(mundo, brazo, [tool("Write", file_path=f"{o}/work/x.py")])
    assert r["motivos"] == []


def test_lectura_snapshot_a2_limpia(mundo):
    k, _ = mundo
    rc, r = correr(mundo, "a2", [tool("Read", file_path=f"{k}/prep/kb/x.md")])
    assert (rc, r["motivos"]) == (0, [])


@pytest.mark.parametrize("cmd", [
    "printf 'x' >> {s}/x.md",
    "echo x | tee {s}/x.md",
    "sed -i 's/a/b/' {s}/x.md",
    "cp work/x.md {s}/x.md",
    "rm {s}/x.md",
])
def test_escritura_snapshot_bash(mundo, cmd):
    k, _ = mundo
    rc, r = correr(mundo, "a2", [tool("Bash", command=cmd.format(s=f"{k}/prep/kb"))])
    assert rc == 1 and r["motivos"] == [SNAP_MSG]


def test_bash_lectura_snapshot_a2_limpia(mundo):
    k, _ = mundo
    rc, r = correr(mundo, "a2", [tool("Bash", command=f"cat {k}/prep/kb/x.md 2>/dev/null; cp {k}/prep/kb/x.md work/")])
    assert (rc, r["motivos"]) == (0, [])


def test_escritura_memoria(mundo):
    _, o = mundo
    ev = [tool("Write", file_path=f"{o}/memoria/notas.md")]
    rc, r = correr(mundo, "n", ev)
    assert (rc, r["fuga"], r["motivos"], r["avisos"]) == (0, False, [], [MEM_MSG])
    for brazo in ("a0", "ar", "a2"):
        rc, r = correr(mundo, brazo, ev)
        assert rc == 1 and r["motivos"] == [MEM_MSG]
    for ruta in (f"{o}/work/memoria/x.py", f"{o}/work/y.py"):
        rc, r = correr(mundo, "a2", [tool("Write", file_path=ruta)])
        assert r["motivos"] == []


@pytest.mark.parametrize("brazo", ["a2", "n"])
def test_escritura_snapshot_bash_cd_relativa(mundo, brazo):
    k, _ = mundo
    # escenario real de K (g1-16/a3-r2), con la raíz sustituida por la del tmp
    cmd = f"cd {k}/prep/kb && printf '\\n## x\\n' >> log/code-graph-engine-bitacora.md"
    rc, r = correr(mundo, brazo, [tool("Bash", command=cmd)])
    assert rc == 1 and SNAP_MSG in r["motivos"]


def test_bash_cd_snapshot_lectura_limpia(mundo):
    k, _ = mundo
    rc, r = correr(mundo, "a2", [tool("Bash", command=f"cd {k}/prep/kb && grep -r x . 2>&1 >/dev/null; ls > /dev/null")])
    assert (rc, r["motivos"]) == (0, [])


def test_lectura_memoria_n_permitida(mundo):
    k, o = mundo
    rc, r = correr(mundo, "n", [tool("Read", file_path=f"{o}/memoria/MEMORY.md"),
                                tool("Write", file_path=f"{k}/prep/kb/x.md")])
    # en n, escribir en el snapshot también lo "toca" (regla de a0); la lectura de memoria no añade nada
    assert r["motivos"] == [SNAP_MSG, "toca el snapshot de la KB"]
    rc, r = correr(mundo, "n", [tool("Read", file_path=f"{o}/memoria/MEMORY.md")])
    assert (rc, r["motivos"]) == (0, [])


def resultado(texto, is_error=True, id="t1"):
    return {"type": "user", "message": {"content": [{"type": "tool_result", "tool_use_id": id, "is_error": is_error, "content": texto}]}}


DENEGADO = "Permission to use Bash with command ls ~/.exo 2>&1 | head has been denied."


def test_intento_denegado_no_es_fuga(mundo):
    rc, r = correr(mundo, "n", [tool("Bash", command="ls ~/.exo 2>&1 | head"), resultado(DENEGADO)])
    assert (rc, r["fuga"]) == (0, False)
    assert r["avisos"] == ["intento denegado por permisos"]


def test_acceso_real_a_exo_sigue_siendo_fuga(mundo):
    ok = resultado("config.toml", is_error=False)
    rc, r = correr(mundo, "n", [tool("Bash", command="ls ~/.exo"), ok])
    assert rc == 1 and r["motivos"] == ["toca la KB de producción o ~/.exo"]
    rc, r = correr(mundo, "n", [tool("Bash", command="ls ~/.exo"), resultado("ls: No such file")])
    assert rc == 1
