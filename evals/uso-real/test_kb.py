import json
import os
import stat
import subprocess
import tempfile
import unittest
from unittest import mock

import kb


def git(d, *args, fecha=None, autor=None):
    env = dict(os.environ)
    if fecha:
        env["GIT_COMMITTER_DATE"] = fecha
        env["GIT_AUTHOR_DATE"] = autor or fecha
    subprocess.run(
        ["git", "-C", d, "-c", "user.name=t", "-c", "user.email=t@t", *args],
        check=True, capture_output=True, env=env)


def escribe(d, ruta, texto):
    p = os.path.join(d, ruta)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8") as f:
        f.write(texto)


def commit(d, msg, fecha, autor=None):
    git(d, "add", "-A", fecha=fecha)
    git(d, "commit", "-m", msg, fecha=fecha, autor=autor)


class Repo(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.d = self.tmp.name
        git(self.d, "init", "-q")


class TestCommitAntes(Repo):
    def test_commit_antes(self):
        d = self.d
        escribe(d, "a.md", "1")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        escribe(d, "a.md", "2")
        # autor distinto del committer: solo debe contar la fecha de commit
        commit(d, "c2", "2026-10-10T12:00:00+0000", autor="2026-10-10T08:00:00+0000")
        escribe(d, "a.md", "3")
        commit(d, "c3", "2026-10-10T14:00:00+0000")
        h = lambda n: subprocess.run(
            ["git", "-C", d, "rev-parse", f"HEAD~{n}"], capture_output=True,
            text=True, check=True).stdout.strip()
        c1, c2 = h(2), h(1)
        self.assertEqual(kb.commit_antes(d, "2026-10-10T13:00:00Z"), c2)
        self.assertIsNone(kb.commit_antes(d, "2026-10-10T09:00:00Z"))
        # `<` estricto: ts == commit de las 12:00 devuelve el anterior
        self.assertEqual(kb.commit_antes(d, "2026-10-10T12:00:00Z"), c1)
        # 13:30+01:00 == 12:30Z (entre c2 y c3); 12:30+02:00 == 10:30Z (entre c1 y c2)
        self.assertEqual(kb.commit_antes(d, "2026-10-10T13:30:00+01:00"), c2)
        self.assertEqual(kb.commit_antes(d, "2026-10-10T12:30:00+02:00"), c1)


    def test_commit_antes_fecha_no_monotona(self):
        d = self.d
        escribe(d, "a.md", "1")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        escribe(d, "a.md", "2")
        commit(d, "c2", "2026-10-10T12:00:00+0000")
        escribe(d, "a.md", "3")
        commit(d, "c3", "2026-10-10T11:00:00+0000")
        c2 = subprocess.run(["git", "-C", d, "rev-parse", "HEAD~1"], capture_output=True,
                            text=True, check=True).stdout.strip()
        self.assertEqual(kb.commit_antes(d, "2026-10-10T12:30:00Z"), c2)


class TestRutaEn(Repo):
    def test_ruta_en_renombre_encadenado(self):
        d = self.d
        escribe(d, "projects/a.md", "contenido de la nota\nlinea 2\nlinea 3\n")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        git(d, "mv", "projects/a.md", "projects/b.md", fecha="2026-10-10T12:00:00+0000")
        commit(d, "c2", "2026-10-10T12:00:00+0000")
        git(d, "mv", "projects/b.md", "projects/c.md", fecha="2026-10-10T14:00:00+0000")
        commit(d, "c3", "2026-10-10T14:00:00+0000")
        rev = lambda n: subprocess.run(
            ["git", "-C", d, "rev-parse", f"HEAD~{n}"], capture_output=True,
            text=True, check=True).stdout.strip()
        self.assertEqual(kb.ruta_en(d, "projects/c.md", rev(2)), "projects/a.md")
        self.assertEqual(kb.ruta_en(d, "projects/c.md", rev(1)), "projects/b.md")


    def test_ruta_en_renombre(self):
        d = self.d
        escribe(d, "projects/base.md", "x")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        escribe(d, "projects/nota vieja — a.md", "contenido de la nota\nlinea 2\nlinea 3\n")
        commit(d, "c2", "2026-10-10T12:00:00+0000")
        git(d, "mv", "projects/nota vieja — a.md", "projects/nota nueva — b.md", fecha="2026-10-10T14:00:00+0000")
        commit(d, "c3", "2026-10-10T14:00:00+0000")
        c = lambda n: subprocess.run(
            ["git", "-C", d, "rev-parse", f"HEAD~{n}"], capture_output=True,
            text=True, check=True).stdout.strip()
        head = "projects/nota nueva — b.md"
        self.assertEqual(kb.ruta_en(d, head, c(1)), "projects/nota vieja — a.md")
        self.assertEqual(kb.ruta_en(d, head, c(0)), head)
        self.assertIsNone(kb.ruta_en(d, head, c(2)))
        ctx = kb.contexto(d, c(1), kb.ruta_en(d, head, c(1)))
        self.assertTrue(ctx["nota"].startswith("contenido de la nota"))


class TestContexto(Repo):
    def test_nota_ausente(self):
        d = self.d
        escribe(d, "core/a.md", "a")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        escribe(d, "projects/n.md", "texto nota")
        commit(d, "c2", "2026-10-10T12:00:00+0000")
        rev = lambda n: subprocess.run(
            ["git", "-C", d, "rev-parse", f"HEAD~{n}"], capture_output=True,
            text=True, check=True).stdout.strip()
        ausente = kb.contexto(d, rev(1), "projects/n.md")
        self.assertTrue(ausente["nota_ausente"])
        self.assertIsNone(ausente["nota"])
        presente = kb.contexto(d, rev(0), "projects/n.md")
        self.assertFalse(presente["nota_ausente"])
        self.assertEqual(presente["nota"], "texto nota")

    def test_contexto_core(self):
        d = self.d
        escribe(d, "core/a.md", "version vieja")
        escribe(d, "core/reglas-duras.md", "reglas viejas")
        escribe(d, "otro/b.md", "fuera de core")
        escribe(d, "core/sub/x.md", "anidado")
        escribe(d, "core/z.txt", "no md")
        commit(d, "c1", "2026-10-10T10:00:00+0000")
        viejo = subprocess.run(["git", "-C", d, "rev-parse", "HEAD"], capture_output=True,
                               text=True, check=True).stdout.strip()
        escribe(d, "core/a.md", "version nueva")
        escribe(d, "core/reglas-duras.md", "reglas nuevas")
        commit(d, "c2", "2026-10-10T12:00:00+0000")
        ctx = kb.contexto(d, viejo, None)
        self.assertEqual(ctx["core"]["core/a.md"], "version vieja")
        self.assertEqual(ctx["reglas_globales"], "reglas viejas")
        self.assertNotIn("otro/b.md", ctx["core"])
        self.assertNotIn("core/sub/x.md", ctx["core"])
        self.assertNotIn("core/z.txt", ctx["core"])
        self.assertEqual(set(ctx["core"]), {"core/a.md", "core/reglas-duras.md"})
        self.assertIsNone(ctx["nota"])


class TestNotaHead(unittest.TestCase):
    def _bin(self, d, cuerpo):
        p = os.path.join(d, "exo-falso")
        with open(p, "w") as f:
            f.write("#!/bin/sh\n" + cuerpo + "\n")
        os.chmod(p, os.stat(p).st_mode | stat.S_IXUSR)
        return p

    def test_nota_head_sin_binario(self):
        with tempfile.TemporaryDirectory() as d:
            malo = self._bin(d, "exit 1")
            with mock.patch.dict(os.environ, {"EXO_BIN": malo}):
                self.assertIsNone(kb.nota_head(d, d))
            ok = self._bin(d, "echo '%s'" % json.dumps({"data": {"note": "projects/x.md"}}))
            with mock.patch.dict(os.environ, {"EXO_BIN": ok}):
                self.assertEqual(kb.nota_head(d, d), "projects/x.md")
            sin = self._bin(d, "echo '%s'" % json.dumps({"data": {"note": None}}))
            with mock.patch.dict(os.environ, {"EXO_BIN": sin}):
                self.assertIsNone(kb.nota_head(d, d))
            with mock.patch.dict(os.environ, {"EXO_BIN": os.path.join(d, "no-existe")}):
                self.assertIsNone(kb.nota_head(d, d))


if __name__ == "__main__":
    unittest.main()
