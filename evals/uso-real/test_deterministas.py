import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import deterministas as d


def _linea_asistente(*bloques):
    return json.dumps({"type": "assistant", "message": {"content": list(bloques)}})


def _bash(cmd):
    return {"type": "tool_use", "name": "Bash", "input": {"command": cmd}}


def _jsonl(lineas):
    f = tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False)
    f.write("\n".join(lineas) + "\n")
    f.close()
    return f.name


class TestViolaciones(unittest.TestCase):
    def test_cd_git_positivos(self):
        for c in ("cd x && git status", "cd x; git log", 'cd "a b" &&  git -C . diff'):
            self.assertEqual(d.violaciones(c), ["cd-git"], c)
        self.assertEqual(d.violaciones("git -C x status"), [])

    def test_cd_git_negativos(self):
        for c in ("git -C x status", "cd x && make", "echo cd && gitk", "echo cd && git status"):
            self.assertEqual(d.violaciones(c), [], c)
        self.assertEqual(d.violaciones("cd x && git status"), ["cd-git"])

    def test_stash_checkout(self):
        for c in ("git stash", "git stash push", "git stash pop"):
            self.assertEqual(d.violaciones(c), ["git-stash"], c)
        for c in ("git stash list", "git stash show", "git checkout -b r", "git checkout main"):
            self.assertEqual(d.violaciones(c), [], c)
        self.assertEqual(d.violaciones("git checkout -- f"), ["git-checkout-fichero"])

    def test_add_A(self):
        self.assertEqual(d.violaciones("git add -A"), ["git-add-A"])
        self.assertEqual(d.violaciones("git add --all"), ["git-add-A"])
        self.assertEqual(d.violaciones("git add src/a.py"), [])
        self.assertEqual(d.violaciones("git add -p f"), [])

    def test_heredoc_cuenta(self):
        self.assertEqual(d.violaciones("cat <<EOF\ngit stash\nEOF"), ["git-stash"])


class TestCuenta(unittest.TestCase):
    def test_comandos_bash_ignora_no_bash(self):
        p = _jsonl([
            _linea_asistente({"type": "text", "text": "cd x && git status"}, _bash("ls")),
            _linea_asistente({"type": "tool_use", "name": "Read", "input": {"command": "rm y"}}),
            json.dumps({"type": "user", "message": {"content": [_bash("git stash")]}}),
        ])
        self.assertEqual(d.comandos_bash(p), ["ls"])

    def test_cuenta_por_100(self):
        p = _jsonl([_linea_asistente(_bash("cd x && git status"), _bash("ls"), _bash("pwd"), _bash("make"))])
        r = d.cuenta([p])
        self.assertEqual(r["llamadas_bash"], 4)
        self.assertEqual(r["por_regla"]["cd-git"], 1)
        self.assertEqual(r["por_100"]["cd-git"], 25.0)
        p2 = _jsonl([
            _linea_asistente(_bash("cd x && git status"), _bash("git stash"), _bash("a"), _bash("b")),
            _linea_asistente({"type": "text", "text": "git stash"}, _bash("c"), _bash("d"), _bash("e"), _bash("f")),
            _linea_asistente({"type": "tool_use", "name": "Edit", "input": {}}),
        ])
        r = d.cuenta([p2])
        self.assertEqual(r["llamadas_bash"], 8)
        self.assertEqual(r["por_100"]["cd-git"], 12.5)
        self.assertEqual(r["por_100"]["git-stash"], 12.5)

    def test_hasta_deja_fuera_lo_posterior(self):
        def ev(ts, cmd):
            return json.dumps({"type": "assistant", "timestamp": ts, "message": {"content": [_bash(cmd)]}})
        p = _jsonl([ev("2026-10-10T10:00:00Z", "ls"), ev("2026-10-10T12:00:00+02:00", "ls"),
                    ev("2026-10-10T10:30:00Z", "git stash"), ev("2026-10-10T10:00:01Z", "ls"),
                    json.dumps({"type": "assistant", "message": {"content": [_bash("sin ts")]}}),
                    ev("basura", "x")])
        self.assertEqual(d.comandos_bash(p, "2026-10-10T10:00:01Z"), ["ls", "ls"])
        self.assertEqual(len(d.comandos_bash(p)), 6)
        r = d.cuenta([p], "2026-10-10T10:20:00Z")
        self.assertEqual((r["llamadas_bash"], r["por_regla"]), (3, {}))
        self.assertEqual(d.cuenta([p], "2026-10-10T11:00:00Z")["por_regla"], {"git-stash": 1})

    def test_cuenta_vacia(self):
        self.assertEqual(d.cuenta([]), {"llamadas_bash": 0, "por_regla": {}, "por_100": {}})


if __name__ == "__main__":
    unittest.main()
