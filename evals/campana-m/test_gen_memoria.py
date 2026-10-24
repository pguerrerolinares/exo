import re
import subprocess
import sys
from pathlib import Path

AQUI = Path(__file__).parent
GEN = AQUI / "gen_memoria.py"
sys.path.insert(0, str(AQUI))

import gen_memoria  # noqa: E402


def nota(kb, ruta, tier=None, title=None, cuerpo="Texto de la nota. Mas.", description=None):
    p = kb / ruta
    p.parent.mkdir(parents=True, exist_ok=True)
    fm = []
    if title:
        fm.append(f"title: {title}")
    if tier:
        fm.append(f"tier: {tier}")
    if description:
        fm.append(f"description: {description}")
    pre = "---\n" + "\n".join(fm) + "\n---\n" if fm else ""
    p.write_text(pre + cuerpo + "\n", encoding="utf-8")
    return p


def gen(kb, out):
    return subprocess.run([sys.executable, str(GEN), str(kb), str(out)],
                          capture_output=True, text=True)


def lineas_indice(out):
    return (out / "MEMORY.md").read_text(encoding="utf-8").splitlines()


def test_solo_core_y_stable(tmp_path):
    kb = tmp_path / "kb"
    nota(kb, "core/a.md", "core", "A")
    nota(kb, "s/b.md", "stable", "B")
    nota(kb, "l/c.md", "log", "C")
    nota(kb, "n/d.md", None, "D")
    nota(kb, "archive/e.md", "log", "E")
    nota(kb, "archive/f.md", "core", "F")
    r = gen(kb, tmp_path / "o")
    assert r.returncode == 0, r.stderr
    ls = lineas_indice(tmp_path / "o")
    assert len(ls) == 3
    txt = "\n".join(ls)
    for t in ("[A](", "[B](", "[F]("):
        assert t in txt
    for t in ("[C](", "[D](", "[E]("):
        assert t not in txt
    assert "notas=6" in r.stdout


def test_formato_linea():
    assert gen_memoria.linea_indice("T", "a/b.md", "Primera frase. Segunda.", None) == "- [T](a/b.md) — Primera frase."
    assert gen_memoria.linea_indice("T", "a/b.md", "Primera frase. Segunda.", "D") == "- [T](a/b.md) — D"
    # encabezado y frontmatter se saltan hasta el primer párrafo
    assert gen_memoria.linea_indice("T", "a.md", "---\ntitle: x\n---\n\n# Titulo\n\nHola mundo. Fin.", None) == "- [T](a.md) — Hola mundo."
    # ruta con espacios y em dash va entre <>
    assert gen_memoria.linea_indice("T", "a b/c — d.md", "X.", None) == "- [T](<a b/c — d.md>) — X."


def test_titulo_del_nombre_de_fichero(tmp_path):
    kb = tmp_path / "kb"
    nota(kb, "core/Sin título — nota.md", "core")
    gen(kb, tmp_path / "o")
    assert lineas_indice(tmp_path / "o")[0].startswith("- [Sin título — nota](<core/Sin título — nota.md>)")


def test_limites(tmp_path):
    kb = tmp_path / "kb"
    for i in range(201):
        nota(kb, f"c/{i:03}.md", "core", f"N{i}")
    r = gen(kb, tmp_path / "o")
    assert r.returncode == 1
    assert not (tmp_path / "o" / "MEMORY.md").exists()

    kb2 = tmp_path / "kb2"
    for i in range(200):
        nota(kb2, f"c/{i:03}.md", "core", f"N{i}")
    assert gen(kb2, tmp_path / "o2").returncode == 0

    # 199 notas con >25.000 bytes
    kb3 = tmp_path / "kb3"
    for i in range(199):
        nota(kb3, f"c/{i:03}.md", "core", f"N{i}", description="d" * 130)
    r = gen(kb3, tmp_path / "o3")
    assert r.returncode == 1
    assert not (tmp_path / "o3" / "MEMORY.md").exists()


def _kb_bytes(tmp_path, nombre, total):
    kb = tmp_path / nombre
    nota(kb, "c/000.md", "core", "N", description="x")
    base = len(gen_memoria.linea_indice("N", "c/000.md", "", "x").encode()) + 1
    nota(kb, "c/000.md", "core", "N", description="x" * (1 + total - base))
    return kb


def test_limite_bytes_exacto(tmp_path):
    ok = _kb_bytes(tmp_path, "ok", 25000)
    r = gen(ok, tmp_path / "o1")
    assert r.returncode == 0 and "bytes=25000" in r.stdout
    mal = _kb_bytes(tmp_path, "mal", 25001)
    assert gen(mal, tmp_path / "o2").returncode == 1


def test_topic_files_completos(tmp_path):
    kb = tmp_path / "kb"
    nota(kb, "core/a b — c.md", "core", "A")
    nota(kb, "log/x.md", "log")
    nota(kb, "archive/y.md", "stable")
    (kb / ".git").mkdir()
    (kb / ".git" / "z.md").write_text("no")
    (kb / "otro.txt").write_text("no")
    out = tmp_path / "o"
    assert gen(kb, out).returncode == 0
    orig = {p.relative_to(kb) for p in kb.rglob("*.md") if ".git" not in p.parts}
    copia = {p.relative_to(out) for p in out.rglob("*.md")} - {Path("MEMORY.md")}
    assert copia == orig and len(orig) == 3
    for rel in orig:
        assert (out / rel).read_bytes() == (kb / rel).read_bytes()


def test_determinista(tmp_path):
    kb = tmp_path / "kb"
    for n in ["z", "m", "a"]:
        nota(kb, f"{n}/{n}.md", "core", n.upper())
    nota(kb, "b/b.md", "stable", "B")
    gen(kb, tmp_path / "o1")
    gen(kb, tmp_path / "o2")
    a = lineas_indice(tmp_path / "o1")
    assert a == lineas_indice(tmp_path / "o2")
    rutas = [re.search(r"\]\(<?([^>)]+)", l).group(1) for l in a]
    assert len(rutas) == 4 and rutas == sorted(rutas)
