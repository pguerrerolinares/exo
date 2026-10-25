#!/usr/bin/env python3
"""Tests de evaluar3.py (sin claude): tabla del CAP, cláusula de réplica, §7.3 y tanda incompleta."""
import os, subprocess, sys, tempfile, unittest

H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import evaluar3  # noqa: E402

IDS = [f"t{i}" for i in range(10)]


def k_root(cumplen, faltan=()):
    """cumplen = {brazo: n}: las n primeras tareas con r2 rc 0 (r1 rc 1); el resto rc 1 en las dos."""
    d = tempfile.mkdtemp()
    open(f"{d}/reconstruccion.tsv", "w").write("".join(f"{i}\tsi\n" for i in IDS))
    tsv = f"{d}/tareas.tsv"
    open(tsv, "w").write("".join(f"{i}\tsuelo\tx/{{brazo}}.txt\tarp1:2,arp10:2,arp20:2\n" for i in IDS))
    for b, n in cumplen.items():
        for j, i in enumerate(IDS):
            for r in (1, 2):
                if (b, i, r) in faltan:
                    continue
                o = f"{d}/corridas/{i}/{b}-r{r}"; os.makedirs(o)
                open(f"{o}/check.rc", "w").write("0\n" if (j < n and r == 2) else "1\n")
    return d, tsv


class T(unittest.TestCase):
    def caso(self, n1, n10, n20):
        d, tsv = k_root({"arp1": n1, "arp10": n10, "arp20": n20})
        return evaluar3.adjudica(d, tsv)[0]

    def test_tabla_del_cap(self):
        for n20, cap in ((10, 10), (5, 10), (4, 7), (3, 5), (0, 5)):
            self.assertEqual(self.caso(6, 6, n20)["cap"], cap, n20)

    def test_cumple_con_una_de_dos_replicas(self):
        self.assertEqual(self.caso(6, 5, 4)["cumplen"], {"arp1": 6, "arp10": 5, "arp20": 4})

    def test_sin_replica_no_hay_cap(self):
        o = self.caso(4, 4, 2)
        self.assertFalse(o["replica"]); self.assertIsNone(o["cap"])
        self.assertTrue(self.caso(5, 4, 2)["replica"])

    def test_7_3(self):
        self.assertTrue(self.caso(6, 3, 5)["invalida_A_7_3"])
        self.assertFalse(self.caso(6, 4, 5)["invalida_A_7_3"])

    def test_replica_ausente_invalida_la_tanda(self):
        d, tsv = k_root({"arp1": 6, "arp10": 6, "arp20": 6}, faltan={("arp20", "t3", 1)})
        p = subprocess.run([sys.executable, f"{H}/evaluar3.py", d, tsv, f"{d}/r.json"], capture_output=True, text=True)
        self.assertEqual(p.returncode, 1); self.assertIn("tanda inválida", p.stderr)


if __name__ == "__main__":
    unittest.main()
