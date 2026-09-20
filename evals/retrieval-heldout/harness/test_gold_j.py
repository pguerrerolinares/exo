import contextlib
import io
import json
import subprocess
import sys
import tempfile
import time
import unittest
import urllib.error
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import acuerdo as ac  # noqa: E402
import juez as jz  # noqa: E402
import limpia_agent_search as la  # noqa: E402


class TestLimpiaAgentSearch(unittest.TestCase):
    def test_criterio(self):
        lineas = [
            'exo search --db ~/.exo/index.db --type hybrid --json "fusion rrf combsum"',
            'exo search --type fts --json "<query>"',
            'exo search --json "memoria v2" | jq .data',
            "exo search sin query ni comillas … (doc)",
            'exo search --json "Fusion  RRF combsum"',
            'exo search --json "fabrica campaña"',
            'exo search --json "x"',
            "cargo test --release",
        ]
        out, desc = la.limpia(lineas, [la.normaliza("fabrica campaña")])
        self.assertEqual([c["query"] for c in out], ["fusion rrf combsum"])
        self.assertEqual(desc, {"no_parsea": 1, "marcador": 3, "vacia": 1, "dup-exclusion": 1, "dup-pool": 1})


class TestJuez(unittest.TestCase):
    def test_paquete_y_parsea(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "core" / "n.md"
            p.parent.mkdir()
            p.write_text("---\ntitle: Nota A\npermalink: kb/core/n\n---\ncuerpo de la nota " + "x" * 5000, encoding="utf-8")
            rutas = jz.rutas_por_permalink(d)
            self.assertEqual(rutas, {"kb/core/n": str(p)})
            paq = jz.paquete({"id": "c1", "query": "q", "source": "prompt", "candidatos": ["kb/core/n", "kb/zz"]}, rutas)
        self.assertEqual(paq["candidatos"], ["kb/core/n", "kb/zz"])
        self.assertIn("título: Nota A", paq["texto"])
        self.assertIn("(nota no encontrada en el snapshot)", paq["texto"])
        self.assertLess(len(paq["texto"]), 2 * jz.MAX_CHARS)
        # F1 del review de rama (2026-09-20): la ceguera de los jueces era
        # asimétrica -- `source` es la etiqueta de estrato que §3 del
        # borrador prohíbe ("los dos ven exactamente el mismo paquete...
        # sin etiquetas") y que Global Constraints prohíbe también ("ningún
        # juez ve la etiqueta del otro"). El campo NO debe estar en lo que
        # un juez ve: el paquete es exactamente {id, candidatos, texto}.
        self.assertEqual(set(paq), {"id", "candidatos", "texto"})
        self.assertNotIn("source", paq)
        cands = paq["candidatos"]
        ok, err = jz.parsea('{"expected": "kb/core/n", "acceptable": ["kb/zz"], "razon": "r"}', cands)
        self.assertIsNone(err)
        self.assertEqual(ok["expected"], "kb/core/n")
        for mala in ('{"expected": "kb/otro", "acceptable": [], "razon": ""}',
                     '{"expected": null, "acceptable": ["kb/zz"], "razon": ""}',
                     '{"expected": "kb/core/n", "acceptable": ["kb/core/n"], "razon": ""}',
                     '{"expected": "kb/core/n", "acceptable": []}',
                     "no json"):
            self.assertIsNone(jz.parsea(mala, cands)[0], mala)
        self.assertEqual(jz.parsea('{"expected": null, "acceptable": [], "razon": "nada"}', [])[0]["expected"], None)

    def test_kimi_con_http_falso(self):
        paq = {"id": "c1", "candidatos": ["kb/a"], "texto": "CONSULTA: q"}
        llamadas = []

        def http(url, key, cuerpo=None, timeout=120):
            llamadas.append((url, cuerpo["model"], cuerpo["response_format"]["type"]))
            return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5}}

        d, err = jz.kimi(paq, "k", "kimi-k3", http=http)
        self.assertIsNone(err)
        self.assertEqual((d["expected"], d["usage"]["prompt_tokens"]), ("kb/a", 10))
        self.assertEqual(llamadas, [(jz.BASE + "/chat/completions", "kimi-k3", "json_schema")])
        # temperature 1, no 0: la API de Moonshot rechaza 0 con 400 ("invalid
        # temperature: only 1 is allowed for this model") en kimi-k3 y
        # kimi-k2.6 (probado con llamadas reales); no es preferencia de
        # estilo, es el único valor que la API acepta.
        self.assertEqual(
            jz.cuerpo_peticion(paq, "m")["temperature"], 1,
            "temperature debe ser 1: la API de Moonshot da 400 con 0 (única "
            "opción que acepta, no una elección de determinismo)",
        )


class TestJuezPresupuesto(unittest.TestCase):
    """Enmienda 2026-09-20 (orquestador, Task 4 de J): circuit breaker de
    gasto real en `juez.procesa_lote`. Sin red: el HTTP se inyecta, como en
    TestJuez.test_kimi_con_http_falso."""

    def _paquetes(self, d, ids):
        p = Path(d) / "paquetes.jsonl"
        with open(p, "w", encoding="utf-8") as fh:
            for i in ids:
                fh.write(json.dumps({"id": i, "candidatos": ["kb/a"], "texto": f"CONSULTA: {i}"}, ensure_ascii=False) + "\n")
        return p

    def test_breaker_para_en_seco_y_no_hace_la_siguiente_llamada(self):
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2", "c3"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 200000, "completion_tokens": 20000}}

            # coste por llamada = 200000/1e6*3 + 20000/1e6*15 = 0.6 + 0.3 = 0.9
            # tope 1.0: tras c1 (0.9) sigue; tras c2 (1.8) para; c3 NO se llama.
            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         tope_usd=1.0, http=http)
            self.assertEqual(len(llamadas), 2)
            self.assertEqual(resultado["parada_por_tope"], "c2")
            self.assertAlmostEqual(resultado["gasto_usd"], 1.8)
            with open(out, encoding="utf-8") as fh:
                escritas = [json.loads(l)["id"] for l in fh]
            self.assertEqual(escritas, ["c1", "c2"])

    def test_coste_calculado_con_precios_conocidos(self):
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 1000000, "completion_tokens": 500000}}

            # 1.000.000/1e6*3.0 + 500.000/1e6*15.0 = 3.0 + 7.5 = 10.5
            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         tope_usd=100.0, http=http)
            self.assertEqual(len(llamadas), 1)
            self.assertNotIn("parada_por_tope", resultado)
            self.assertAlmostEqual(resultado["gasto_usd"], 10.5)

    def test_reanuda_hereda_gasto_de_filas_previas(self):
        """C1: `--out` ya escrito no debe hacer que el breaker "olvide" lo
        gastado en una corrida previa. Repro (escalado) del review
        adversarial: 6 llamadas de $0,90 con tope $5; RUN1 procesa 5
        ($4,50, no dispara), RUN2 reanuda con el mismo --out y debe heredar
        los $4,50 -no arrancar en $0- al procesar la 6ª."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2", "c3", "c4", "c5", "c6"])
            out = Path(d) / "out.jsonl"

            def http(url, key, cuerpo=None, timeout=120):
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 200000, "completion_tokens": 20000}}  # $0.90/llamada

            r1 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0,
                                  tope_usd=5.0, maximo=5, http=http)
            self.assertNotIn("parada_por_tope", r1)
            self.assertAlmostEqual(r1["gasto_usd"], 4.5)

            r2 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0,
                                  tope_usd=5.0, http=http)
            # RUN2 hereda los $4.50 de RUN1: procesar c6 (+$0.90) da $5.40, no $0.90.
            self.assertAlmostEqual(r2["gasto_usd"], 5.4)
            self.assertEqual(r2.get("parada_por_tope"), "c6")

    def test_reintentos_fallidos_de_parsea_cuentan_coste(self):
        """C2: una respuesta que pasa el json_schema strict pero falla
        parsea() (permalink alucinado, fuera de candidatos) es una llamada
        FACTURADA igual; su coste debe contar, y el tope debe pararse
        también entre reintentos (no solo entre paquetes)."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                # pasa el schema, pero "kb/alucinado" no está en candidatos -> parsea() la rechaza.
                return {"choices": [{"message": {"content": '{"expected": "kb/alucinado", "acceptable": [], "razon": "x"}'}}],
                        "usage": {"prompt_tokens": 200000, "completion_tokens": 20000}}  # $0.90/llamada

            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         tope_usd=1.0, http=http)
            # reintentos=3 por defecto: intento1 (gasto 0->0.9, no supera 1.0, se lanza intento2),
            # intento2 (0.9->1.8, ya facturado), intento3 NO se lanza (1.8 > 1.0 antes de lanzarlo).
            self.assertEqual(len(llamadas), 2)
            self.assertAlmostEqual(resultado["gasto_usd"], 1.8)
            self.assertEqual(resultado.get("parada_por_tope"), "c1")
            self.assertEqual(resultado["errores"], [{"id": "c1", "error": "tope superado: reintento no lanzado"}])
            with open(out, encoding="utf-8") as fh:
                self.assertEqual(fh.read(), "")

    def test_usage_ausente_o_con_campos_none_para_en_seco(self):
        """C3: usage ausente o con campos no numéricos (None) es error
        explícito -> parada en seco, nunca gasto cero silencioso ni
        estimado."""
        for descripcion, extra in [
            ("sin clave usage", {}),
            ("usage con campos None", {"usage": {"prompt_tokens": None, "completion_tokens": None}}),
        ]:
            with self.subTest(descripcion):
                with tempfile.TemporaryDirectory() as d:
                    ruta = self._paquetes(d, ["c1"])
                    out = Path(d) / "out.jsonl"

                    def http(url, key, cuerpo=None, timeout=120, _extra=extra):
                        r = {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}]}
                        r.update(_extra)
                        return r

                    resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                                 precio_entrada=3.0, precio_salida=15.0,
                                                 tope_usd=0.5, http=http)
                    self.assertEqual(resultado.get("parada_por_usage_invalido"), "c1", descripcion)
                    self.assertAlmostEqual(resultado["gasto_usd"], 0.0, msg=descripcion)
                    with open(out, encoding="utf-8") as fh:
                        self.assertEqual(fh.read(), "", descripcion)

    def test_tope_borde_exacto_no_para_al_igualar_solo_al_superar(self):
        """C4: el criterio es SUPERAR (>), no ALCANZAR (>=): un gasto que
        iguala el tope exactamente no dispara el breaker; superarlo sí."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2"])
            out = Path(d) / "out.jsonl"
            llamadas = []

            def http(url, key, cuerpo=None, timeout=120):
                llamadas.append(cuerpo)
                # precio_entrada=1.0, prompt_tokens=1_000_000 -> coste exacto $1.00/llamada.
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 1_000_000, "completion_tokens": 0}}

            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=1.0, precio_salida=15.0,
                                         tope_usd=1.0, http=http)
            # c1 cuesta exactamente $1.00 == tope: NO para (iguala, no supera) -> sigue con c2.
            # c2 lleva el acumulado a $2.00 > $1.00: PARA.
            self.assertEqual(len(llamadas), 2)
            self.assertEqual(resultado.get("parada_por_tope"), "c2")
            self.assertAlmostEqual(resultado["gasto_usd"], 2.0)


class TestJuezDiarioGasto(unittest.TestCase):
    """Enmienda 2026-09-20 (c), segunda ronda de review adversarial sobre el
    breaker: cinco hallazgos más, cerrados con el diario append-only
    `<out>.gasto.jsonl`. Sin red: el HTTP se inyecta."""

    def _paquetes(self, d, ids):
        p = Path(d) / "paquetes.jsonl"
        with open(p, "w", encoding="utf-8") as fh:
            for i in ids:
                fh.write(json.dumps({"id": i, "candidatos": ["kb/a"], "texto": f"CONSULTA: {i}"}, ensure_ascii=False) + "\n")
        return p

    def test_F1_urlerror_registra_desconocida_y_para_por_tolerancia_default(self):
        """F1: una respuesta perdida (URLError) tras salir hacia la API es
        una llamada de facturación DESCONOCIDA -no gasto $0 invisible-, y
        con la tolerancia por defecto (0) la primera ya para el pipeline en
        seco."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2", "c3"])
            out = Path(d) / "out.jsonl"

            def http(url, key, cuerpo=None, timeout=120):
                raise urllib.error.URLError("conexión perdida (la API pudo haber facturado igual)")

            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         reintentos=1, http=http)
            self.assertEqual(resultado.get("parada_por_gasto_desconocido"), "c1")
            self.assertEqual(resultado["desconocidas"], 1)
            self.assertAlmostEqual(resultado["gasto_usd"], 0.0)  # nunca se inventa coste
            with open(f"{out}.gasto.jsonl", encoding="utf-8") as fh:
                diario = [json.loads(l) for l in fh]
            # (d): cabecera + marca en_vuelo (ANTES de llamar) + resolución "desconocida".
            self.assertEqual(len(diario), 3)
            self.assertEqual(diario[0]["tipo"], "header")
            self.assertEqual((diario[1]["tipo"], diario[1]["id"], diario[1]["intento"]), ("en_vuelo", "c1", 1))
            resolucion = diario[2]
            self.assertEqual(resolucion["tipo"], "desconocida")
            self.assertEqual(resolucion["id"], "c1")
            self.assertIsNone(resolucion["usd"])
            self.assertIn("URLError", resolucion["motivo_desconocida"])

    def test_F2_lock_exclusivo_falla_rapido_con_mismo_out(self):
        """F2: con el lock de `<out>.gasto.jsonl.lock` ya tomado (simula un
        segundo proceso concurrente con el mismo --out), procesa_lote falla
        rápido con SystemExit y NUNCA llama a la API."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            ruta_lock = f"{out}.gasto.jsonl.lock"
            lf = open(ruta_lock, "w")
            jz.fcntl.flock(lf, jz.fcntl.LOCK_EX | jz.fcntl.LOCK_NB)

            def http(url, key, cuerpo=None, timeout=120):
                raise AssertionError("no debería llamarse a la API con el lock tomado por otro proceso")

            try:
                with self.assertRaises(SystemExit):
                    jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                     precio_entrada=3.0, precio_salida=15.0, http=http)
            finally:
                jz.fcntl.flock(lf, jz.fcntl.LOCK_UN)
                lf.close()

    def test_F3_usage_nan_no_apaga_el_freno_permanentemente(self):
        """F3: NaN es `float` pero json.loads lo acepta como literal; sin
        `math.isfinite()` el gasto queda en NaN y `gasto > tope` es SIEMPRE
        False. Cubre también tokens negativos (mismo `_numerico`)."""
        for descripcion, usage in [
            ("prompt_tokens NaN", {"prompt_tokens": float("nan"), "completion_tokens": 5}),
            ("completion_tokens -Infinity", {"prompt_tokens": 5, "completion_tokens": float("-inf")}),
            ("prompt_tokens negativo", {"prompt_tokens": -100, "completion_tokens": 5}),
        ]:
            with self.subTest(descripcion):
                with tempfile.TemporaryDirectory() as d:
                    ruta = self._paquetes(d, ["c1"])
                    out = Path(d) / "out.jsonl"

                    def http(url, key, cuerpo=None, timeout=120, _usage=usage):
                        return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                                "usage": _usage}

                    resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                                 precio_entrada=3.0, precio_salida=15.0,
                                                 reintentos=1, http=http)
                    self.assertEqual(resultado.get("parada_por_usage_invalido"), "c1", descripcion)

    def test_F4_diario_truncado_da_systemexit_explicito(self):
        """F4: una última línea truncada (crash a medio flush) del diario da
        un SystemExit con mensaje explícito, no un json.JSONDecodeError
        crudo."""
        with tempfile.TemporaryDirectory() as d:
            ruta_gasto = Path(d) / "out.jsonl.gasto.jsonl"
            ruta_gasto.write_text(
                '{"id": "c1", "intento": 1, "ts": 1.0, "prompt_tokens": 10, "completion_tokens": 5, '
                '"usd": 0.001, "precio_entrada": 3.0, "precio_salida": 15.0}\n'
                '{"id": "c2", "intento": 1, "ts": 2.0, "prompt_tok',  # truncada a medio flush
                encoding="utf-8",
            )
            with self.assertRaises(SystemExit) as cm:
                jz.reconstruye_gasto(str(ruta_gasto), 3.0, 15.0)
            self.assertIn("no parsea como JSON", str(cm.exception))
            self.assertIn("línea 2", str(cm.exception))

    def test_F5_aviso_si_los_precios_difieren_entre_run_y_resume(self):
        """F5: el `usd` se persiste YA CALCULADO por llamada; reanudar con
        precios distintos (dos tarifas reales de Moonshot, k3 y k2.6) no
        recalcula el gasto heredado, pero SÍ avisa por stderr."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2"])
            out = Path(d) / "out.jsonl"

            def http(url, key, cuerpo=None, timeout=120):
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                        "usage": {"prompt_tokens": 1_000_000, "completion_tokens": 0}}

            # RUN1 a precios k3 ($3/$15): coste de c1 = $3.00.
            r1 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0, maximo=1, http=http)
            self.assertAlmostEqual(r1["gasto_usd"], 3.0)

            # RUN2 reanudado por error con precios k2.6 ($0.95/$4): si se recalculara el
            # heredado a precio de hoy, el total sería 0.95+0.95=1.90; con usd persistido es 3.95.
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                r2 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                      precio_entrada=0.95, precio_salida=4.0,
                                      tope_usd=100.0, http=http)
            self.assertAlmostEqual(r2["gasto_usd"], 3.0 + 0.95)
            self.assertIn("AVISO", buf.getvalue())
            self.assertIn("difieren", buf.getvalue())

    def test_cota_gasto_recuperable_desde_el_diario_tras_racha_fallida_y_crash(self):
        """La cota real de fuga que motivó el diario: una racha larga de
        paquetes que fallan parsea() (permalink alucinado -> nunca escriben
        fila en --out) y un crash simulado a mitad deben dejar el gasto
        recuperable desde el diario -no un residual de céntimos, sino todo
        lo facturado antes del crash, que puede llegar hasta el tope
        entero-."""
        with tempfile.TemporaryDirectory() as d:
            ids = [f"c{i}" for i in range(1, 6)]
            ruta = self._paquetes(d, ids)
            out = Path(d) / "out.jsonl"
            llamadas = {"n": 0}

            def http(url, key, cuerpo=None, timeout=120):
                llamadas["n"] += 1
                if llamadas["n"] == 4:
                    raise RuntimeError("crash simulado a mitad (no es un fallo de red)")
                return {"choices": [{"message": {"content": '{"expected": "kb/alucinado", "acceptable": [], "razon": "x"}'}}],
                        "usage": {"prompt_tokens": 200000, "completion_tokens": 20000}}  # $0.90/llamada

            with self.assertRaises(RuntimeError):
                jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                 precio_entrada=3.0, precio_salida=15.0,
                                 tope_usd=100.0, reintentos=1, http=http)

            # bajo el diseño VIEJO (gasto reconstruido de --out): --out está vacío -> $0 tras el
            # "crash", aunque ya se facturaron 3 llamadas ($0.90 c/u = $2.70).
            self.assertEqual(out.read_text(encoding="utf-8"), "")
            gasto, tok, desconocidas, usage_invalidas = jz.reconstruye_gasto(f"{out}.gasto.jsonl", 3.0, 15.0)
            self.assertAlmostEqual(gasto, 2.70)
            # (d): la 4ª llamada murió DENTRO de http() -tras la marca en_vuelo fsyncada, sin
            # resolución posterior-; reconstruye_gasto la cuenta como desconocida, no como cero
            # (antes de (d) esto era exactamente el hueco C2: 0, invisible).
            self.assertEqual(desconocidas, 1)
            self.assertEqual(usage_invalidas, 0)


class TestJuezCoherenciaDiario(unittest.TestCase):
    """Enmienda 2026-09-20 (d), tercera ronda de review adversarial sobre el
    breaker: C1 (diario y --out deben ser verificablemente coherentes o el
    pipeline no arranca), C2 (marca 'en_vuelo' fsyncada ANTES de llamar,
    sobrevive a un kill -9 real) y el hallazgo Important (usage_invalido no
    consume --tolerancia-desconocidas). Sin red salvo el test de kill -9,
    que lanza un subproceso real pero con un http() inyectado que nunca
    toca la red."""

    def _paquetes(self, d, ids, nombre="paquetes.jsonl"):
        p = Path(d) / nombre
        with open(p, "w", encoding="utf-8") as fh:
            for i in ids:
                fh.write(json.dumps({"id": i, "candidatos": ["kb/a"], "texto": f"CONSULTA: {i}"}, ensure_ascii=False) + "\n")
        return p

    def _http_ok(self, url, key, cuerpo=None, timeout=120):
        return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}],
                "usage": {"prompt_tokens": 1000, "completion_tokens": 100}}

    def test_C1_out_con_filas_validas_sin_diario_aborta(self):
        """C1: --out con filas válidas, diario perdido (borrado, vacío, u
        otro --out con typo) -> SystemExit, nunca gasto heredado en $0 en
        silencio."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2"])
            out = Path(d) / "out.jsonl"
            r1 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0,
                                  maximo=1, http=self._http_ok)
            self.assertEqual(r1["ok"], 1)
            Path(f"{out}.gasto.jsonl").unlink()  # el diario se pierde; --out queda con la fila

            with self.assertRaises(SystemExit) as cm:
                jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                 precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)
            msg = str(cm.exception)
            self.assertIn("c1", msg)
            self.assertIn("--acepto-riesgo-gasto-no-verificable", msg)

    def test_C1_out_con_id_sin_entrada_ok_correspondiente_aborta(self):
        """C1: diario presente pero sin la entrada 'ok' de uno de los ids ya
        escritos en --out (diario incompleto/desincronizado) -> SystemExit
        listando exactamente el id que falta."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2"])
            out = Path(d) / "out.jsonl"
            jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                             precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)

            ruta_diario = f"{out}.gasto.jsonl"
            lineas = [json.loads(l) for l in Path(ruta_diario).read_text(encoding="utf-8").splitlines() if l.strip()]
            lineas_sin_c2_ok = [l for l in lineas if not (l.get("tipo") == "ok" and l.get("id") == "c2")]
            self.assertLess(len(lineas_sin_c2_ok), len(lineas))
            with open(ruta_diario, "w", encoding="utf-8") as fh:
                for l in lineas_sin_c2_ok:
                    fh.write(json.dumps(l, ensure_ascii=False) + "\n")

            with self.assertRaises(SystemExit) as cm:
                jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                 precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)
            msg = str(cm.exception)
            self.assertIn("c2", msg)
            self.assertNotIn("'c1'", msg)

    def test_C1_diario_ajeno_detectado_por_identidad_de_out(self):
        """C1: un diario de OTRO trabajo pegado en la ruta de --out (mismo
        nombre de fichero, --out distinto) se detecta por la identidad de
        fichero registrada en la cabecera, aunque los ids coincidan."""
        with tempfile.TemporaryDirectory() as d:
            rutaA = self._paquetes(d, ["c1"], "paqA.jsonl")
            outA = Path(d) / "outA.jsonl"
            jz.procesa_lote(str(rutaA), str(outA), "k", "kimi-k3",
                             precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)

            rutaB = self._paquetes(d, ["c1"], "paqB.jsonl")
            outB = Path(d) / "outB.jsonl"
            outB.touch()  # outB existe con identidad de fichero propia, distinta de outA
            Path(f"{outB}.gasto.jsonl").write_text(
                Path(f"{outA}.gasto.jsonl").read_text(encoding="utf-8"), encoding="utf-8")

            with self.assertRaises(SystemExit) as cm:
                jz.procesa_lote(str(rutaB), str(outB), "k", "kimi-k3",
                                 precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)
            self.assertIn("otro trabajo", str(cm.exception))

    def test_C1_override_explicito_permite_arrancar_y_deja_rastro(self):
        """C1: --acepto-riesgo-gasto-no-verificable evita el SystemExit,
        pero deja un rastro 'override_trazabilidad' en el propio diario y
        avisa por stderr -no es un bypass silencioso-."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1", "c2"])
            out = Path(d) / "out.jsonl"
            jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                             precio_entrada=3.0, precio_salida=15.0,
                             maximo=1, http=self._http_ok)
            Path(f"{out}.gasto.jsonl").unlink()

            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                             precio_entrada=3.0, precio_salida=15.0,
                                             http=self._http_ok, acepto_riesgo_gasto=True)
            self.assertIn("AVISO", buf.getvalue())
            self.assertIn("--acepto-riesgo-gasto-no-verificable", buf.getvalue())
            self.assertEqual(resultado["ok"], 1)  # c1 ya estaba en --out (se salta), procesa c2

            lineas = [json.loads(l) for l in Path(f"{out}.gasto.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
            self.assertTrue(any(l.get("tipo") == "override_trazabilidad" for l in lineas))

    def test_Important_usage_invalido_heredado_no_consume_tolerancia_desconocidas(self):
        """Important: USAGE_INVALIDO (C3, bug de schema) y desconocida real
        (F1, llamada perdida) son motivos distintos; solo la segunda debe
        consumir --tolerancia-desconocidas al reanudar. Con la tolerancia
        por defecto (0), si se conflacionaran, reanudar tras un
        USAGE_INVALIDO ya resuelto pararía en seco de inmediato -un
        breaker distinto disparado por error-."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"

            def http_sin_usage(url, key, cuerpo=None, timeout=120):
                return {"choices": [{"message": {"content": '{"expected": "kb/a", "acceptable": [], "razon": "ok"}'}}]}

            r1 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0, http=http_sin_usage)
            self.assertEqual(r1.get("parada_por_usage_invalido"), "c1")

            r2 = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                  precio_entrada=3.0, precio_salida=15.0, http=self._http_ok)
            self.assertNotIn("parada_por_gasto_desconocido", r2)
            self.assertEqual(r2["ok"], 1)
            with open(out, encoding="utf-8") as fh:
                self.assertEqual([json.loads(l)["id"] for l in fh], ["c1"])

    def test_C2_kill_9_real_marca_en_vuelo_sobrevive_al_crash(self):
        """C2: un `kill -9` REAL entre 'se factura' (marca en_vuelo
        fsyncada) y 'vuelve la respuesta' no debe dejar el diario sin
        rastro de esa llamada. Lanza un subproceso real y lo mata de
        verdad -un test que solo simule la excepción no prueba el
        fsync()-; luego lee el diario en disco desde este proceso."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            ruta_diario = f"{out}.gasto.jsonl"
            harness_dir = str(Path(__file__).resolve().parent)
            script = Path(d) / "runner.py"
            script.write_text(
                "import sys, time\n"
                f"sys.path.insert(0, {harness_dir!r})\n"
                "import juez as jz\n"
                "def http(url, key, cuerpo=None, timeout=120):\n"
                "    time.sleep(60)\n"
                "    raise AssertionError('no debería llegar aqui: se mata antes')\n"
                f"jz.procesa_lote({str(ruta)!r}, {str(out)!r}, 'k', 'kimi-k3', "
                "precio_entrada=3.0, precio_salida=15.0, reintentos=1, http=http)\n",
                encoding="utf-8",
            )
            proc = subprocess.Popen([sys.executable, str(script)])
            try:
                limite = time.time() + 10
                marcado = False
                while time.time() < limite:
                    if Path(ruta_diario).exists():
                        lineas = [json.loads(l) for l in Path(ruta_diario).read_text(encoding="utf-8").splitlines() if l.strip()]
                        if any(l.get("tipo") == "en_vuelo" and l.get("id") == "c1" for l in lineas):
                            marcado = True
                            break
                    time.sleep(0.02)
                self.assertTrue(marcado, "la marca en_vuelo no apareció en el diario a tiempo")
                proc.kill()  # SIGKILL real -no una excepción simulada dentro del proceso vivo-
                proc.wait(timeout=5)
            finally:
                if proc.poll() is None:
                    proc.kill()
                    proc.wait(timeout=5)

            lineas = [json.loads(l) for l in Path(ruta_diario).read_text(encoding="utf-8").splitlines() if l.strip()]
            self.assertTrue(any(l.get("tipo") == "en_vuelo" and l.get("id") == "c1" for l in lineas))
            self.assertFalse(any(l.get("tipo") in ("ok", "desconocida", "usage_invalido") for l in lineas))

            gasto, tok, desconocidas, usage_invalidas = jz.reconstruye_gasto(ruta_diario, 3.0, 15.0)
            self.assertAlmostEqual(gasto, 0.0)
            self.assertEqual(desconocidas, 1)  # marca sin resolver -> desconocida, nunca cero
            self.assertEqual(usage_invalidas, 0)

    def test_C2_resume_tras_kill_9_no_pierde_la_marca_huerfana(self):
        """Enmienda (e), el bug real que motivó el fix: el test de arriba
        (test_C2_kill_9_real_...) solo comprueba el estado DENTRO de la
        corrida que crashea; nunca comprueba el resume posterior, que es
        donde vivía el bug. `kimi()` numera los intentos con
        `for i in range(reintentos)`, arrancando SIEMPRE en 1 -sin memoria
        de que este mismo id ya tuvo un intento 1 en una corrida anterior-.
        Al reanudar con el mismo --out tras el kill -9, el reintento vuelve
        a numerarse intento=1 y, si esta vez la respuesta llega bien, su
        resolución 'ok' (id=c1, intento=1) coincidía -antes de (e)- con la
        clave de la marca huérfana del crash y la borraba del libro de
        pendientes al reconstruir: esa llamada, que el proveedor pudo haber
        facturado durante el crash, dejaba de contar como dólar o como
        desconocida. Repro: mata un proceso real (mismo patrón que arriba)
        justo después de que escriba y fsyncee la marca en_vuelo, luego
        reanuda con el MISMO --out y un http() que esta vez responde bien,
        y comprueba que tras el resume la marca huérfana SIGUE contando
        como desconocida y que el gasto acumulado la incluye (no la
        sustituye). Solo alcanzable con --tolerancia-desconocidas > 0 (con
        el default, la propia desconocida del crash ya para el pipeline al
        reanudar); por eso el resume la sube a 1."""
        with tempfile.TemporaryDirectory() as d:
            ruta = self._paquetes(d, ["c1"])
            out = Path(d) / "out.jsonl"
            ruta_diario = f"{out}.gasto.jsonl"
            harness_dir = str(Path(__file__).resolve().parent)
            script = Path(d) / "runner.py"
            script.write_text(
                "import sys, time\n"
                f"sys.path.insert(0, {harness_dir!r})\n"
                "import juez as jz\n"
                "def http(url, key, cuerpo=None, timeout=120):\n"
                "    time.sleep(60)\n"
                "    raise AssertionError('no debería llegar aqui: se mata antes')\n"
                f"jz.procesa_lote({str(ruta)!r}, {str(out)!r}, 'k', 'kimi-k3', "
                "precio_entrada=3.0, precio_salida=15.0, reintentos=1, http=http)\n",
                encoding="utf-8",
            )
            proc = subprocess.Popen([sys.executable, str(script)])
            try:
                limite = time.time() + 10
                marcado = False
                while time.time() < limite:
                    if Path(ruta_diario).exists():
                        lineas = [json.loads(l) for l in Path(ruta_diario).read_text(encoding="utf-8").splitlines() if l.strip()]
                        if any(l.get("tipo") == "en_vuelo" and l.get("id") == "c1" for l in lineas):
                            marcado = True
                            break
                    time.sleep(0.02)
                self.assertTrue(marcado, "la marca en_vuelo no apareció en el diario a tiempo")
                proc.kill()  # SIGKILL real
                proc.wait(timeout=5)
            finally:
                if proc.poll() is None:
                    proc.kill()
                    proc.wait(timeout=5)

            # estado tras el crash: --out vacío (nunca llegó a escribir la fila de c1).
            self.assertEqual(out.read_text(encoding="utf-8"), "")

            # RESUME: mismo --out, mismo id c1, esta vez http() responde bien. tolerancia=1
            # para poder completar el resume pese a la desconocida heredada del crash.
            resultado = jz.procesa_lote(str(ruta), str(out), "k", "kimi-k3",
                                         precio_entrada=3.0, precio_salida=15.0,
                                         reintentos=1, tolerancia_desconocidas=1,
                                         http=self._http_ok)
            self.assertEqual(resultado["ok"], 1)
            # LA ASERCIÓN QUE FALLABA ANTES DE (e): la huérfana del crash SIGUE contando,
            # el 'ok' del resume (mismo id, mismo número de intento) no la borra.
            self.assertEqual(resultado["desconocidas"], 1)
            # coste real del resume (_http_ok: prompt_tokens=1000, completion_tokens=100):
            # 1000/1e6*3 + 100/1e6*15 = 0.003 + 0.0015 = 0.0045 -- el gasto INCLUYE la llamada
            # de resume, y la huérfana (usd null) no se confunde con coste cero.
            self.assertAlmostEqual(resultado["gasto_usd"], 0.0045)

            # y reconstruye_gasto sobre el diario final, releído desde cero, coincide.
            gasto, tok, desconocidas, usage_invalidas = jz.reconstruye_gasto(ruta_diario, 3.0, 15.0)
            self.assertEqual(desconocidas, 1)
            self.assertAlmostEqual(gasto, 0.0045)
            self.assertEqual(usage_invalidas, 0)


class TestAcuerdo(unittest.TestCase):
    def j(self, e, a=(), r="r"):
        return {"expected": e, "acceptable": list(a), "razon": r}

    def test_acuerdo_fila_y_fusiona(self):
        self.assertEqual(ac.acuerdo_fila(self.j("a"), self.j("a")), "estricto")
        self.assertEqual(ac.acuerdo_fila(self.j(None), self.j(None)), "estricto")
        self.assertEqual(ac.acuerdo_fila(self.j("a", ["b"]), self.j("b")), "lenient")
        self.assertIsNone(ac.acuerdo_fila(self.j("a"), self.j("b")))
        self.assertIsNone(ac.acuerdo_fila(self.j(None), self.j("b")))
        self.assertEqual(ac.fusiona(self.j("a", ["b", "c"]), self.j("a", ["c", "d"]), "estricto"), ("a", ["c"]))
        self.assertEqual(ac.fusiona(self.j("a", ["b"]), self.j("b", ["a", "c"]), "lenient"), ("a", ["b"]))
        self.assertEqual(ac.fusiona(self.j("b", ["a"]), self.j("a", ["b"]), "lenient"), ("b", ["a"]))

    def test_fusiona_extra_solo_entra_si_el_otro_juez_tambien_lo_admite(self):
        """Review 2026-09-20, I-2: `acceptable_permalinks` son "solo los
        admitidos por ambos jueces" (borrador §3, D-J11: "entra solo con
        acuerdo"). Caso: fable dice expected="a" (acceptable=[]) y kimi dice
        expected="x" (acceptable=["a"]) -> lenient porque a.expected="a" está
        en b.acceptable=["a"]. El "x" de kimi NO está en el acceptable de
        fable (que es []), así que "x" -una nota que solo vio kimi- no debe
        colarse en `acceptable_permalinks`."""
        fable = self.j("a", [])
        kimi = self.j("x", ["a"])
        self.assertEqual(ac.acuerdo_fila(fable, kimi), "lenient")
        self.assertEqual(ac.fusiona(fable, kimi, "lenient"), ("a", []))

        # Caso simétrico con a y b invertidos (b.expected="a" en a.acceptable="x"->no,
        # aquí forzamos la otra rama del if): kimi decide, y el expected de fable
        # descartado solo entra si kimi lo admite en su acceptable.
        fable2 = self.j("y", ["b"])
        kimi2 = self.j("b", [])
        self.assertEqual(ac.acuerdo_fila(fable2, kimi2), "lenient")
        self.assertEqual(ac.fusiona(fable2, kimi2, "lenient"), ("b", []))

    def test_fusiona_extra_entra_cuando_ambos_lo_admiten(self):
        """Control positivo del fix de I-2: si el "extra" SÍ está en el
        acceptable del otro juez, entra normalmente (no se rompe el caso
        legítimo, solo el asimétrico)."""
        fable = self.j("a", ["x"])
        kimi = self.j("x", ["a"])
        self.assertEqual(ac.fusiona(fable, kimi, "lenient"), ("a", ["x"]))

    def test_kappa(self):
        self.assertAlmostEqual(ac.kappa([("a", "a"), ("b", "b"), ("a", "b"), ("b", "a")]), 0.0)
        self.assertEqual(ac.kappa([("a", "a"), ("b", "b")]), 1.0)
        self.assertEqual(ac.kappa([(None, None)] * 4), 1.0)
        # 2x2 clásico: po=0.8, pe=0.5 -> 0.6
        pares = [("x", "x")] * 4 + [("y", "y")] * 4 + [("x", "y"), ("y", "x")]
        self.assertAlmostEqual(ac.kappa(pares), 0.6)

    def test_construye(self):
        cands = [{"id": "c1", "query": "q1", "source": "prompt", "candidatos": ["kb/a", "kb/b"]},
                 {"id": "c2", "query": "q2", "source": "negativo", "candidatos": ["kb/a"]},
                 {"id": "c3", "query": "q3", "source": "archive", "candidatos": ["kb/log/x", "kb/archive/log/x-1"]},
                 {"id": "c4", "query": "q4", "source": "hard", "candidatos": ["kb/a", "kb/b"]},
                 {"id": "c5", "query": "q5", "source": "hard", "candidatos": ["kb/a"]}]
        fab = {"c1": self.j("kb/a"), "c2": self.j(None), "c3": self.j("kb/log/x"), "c4": self.j("kb/a"), "c5": self.j("kb/a")}
        kim = {"c1": self.j("kb/a", ["kb/b"]), "c2": self.j(None), "c3": self.j("kb/log/x"), "c4": self.j("kb/b")}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, {"kb/a": "q1 texto"}, 0.6, 0.7)
        self.assertEqual([g["id"] for g in gold], ["c1", "c2"])
        self.assertEqual(gold[0]["acceptable_permalinks"], [])
        self.assertEqual(sorted(d["motivo"] for d in desc), ["archive sin expected en archive/", "desacuerdo", "sin juicio de ambos"])
        self.assertIn("| negativo | 1 | 1 | 1.000 |", inf)
        self.assertIn("auditoría del sesgo léxico", inf)
        self.assertNotIn("q1", inf)
        self.assertNotIn("kb/a", inf)
        self.assertEqual(po, 0.75)
        self.assertIn("solape_lexico=", gold[0]["notes"])
        # I-3 del review (2026-09-20): κ/p_o excluyendo el estrato `negativo`
        # es descriptivo (no decide `pasa`) y aparece como línea propia del
        # informe. Cálculo a mano de las 3 filas juzgadas sin `negativo`
        # (c1 estricto, c3 estricto, c4 desacuerdo): p_o = 2/3, κ = 0.5.
        self.assertIn("κ / p_o sin `negativo` ni candidatos vacíos: 0.500 / 0.667 (sobre 3 filas; descriptivo, no decide)", inf)

    def test_construye_kappa_po_sin_negativo_no_cambia_exit(self):
        """La línea descriptiva κ/p_o sin `negativo` no debe alterar `pasa`
        ni el `k`/`po` que decide el exit code (suelo firmado 0,60 ∧ 0,70,
        D-J11): siguen calculándose SOBRE TODAS las filas, `negativo`
        incluido."""
        cands = [{"id": "c1", "query": "q1", "source": "prompt", "candidatos": ["kb/a"]},
                 {"id": "c2", "query": "q2", "source": "negativo", "candidatos": ["kb/a"]}]
        fab = {"c1": self.j("kb/a"), "c2": self.j(None)}
        kim = {"c1": self.j("kb/a"), "c2": self.j(None)}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, None, 0.60, 0.70)
        # con negativo incluido: 2 pares perfectamente de acuerdo -> κ=1.0, po=1.0.
        self.assertEqual((k, po), (1.0, 1.0))
        self.assertIn("κ / p_o sin `negativo` ni candidatos vacíos: 1.000 / 1.000 (sobre 1 filas; descriptivo, no decide)", inf)

    def test_construye_kappa_po_sin_negativo_excluye_tambien_candidatos_vacios(self):
        """F3 del review de rama (2026-09-20): el acuerdo forzado null-null
        por construcción no son solo las filas `source == "negativo"` -- las
        filas con `candidatos` vacío (p.ej. 26 del estrato `prompt` sin
        candidata plausible) llevan en el paquete "(sin candidatas: expected
        debe ser null)" y son null-null por el mismo motivo estructural.
        Antes solo se excluía `negativo`; ahora también se excluyen las de
        candidatos vacíos, sin tocar `pasa` ni el suelo firmado (0,60 ∧ 0,70)
        -sigue siendo descriptivo (I-3 del review original)."""
        cands = [{"id": "c1", "query": "q1", "source": "prompt", "candidatos": ["kb/a"]},
                 {"id": "c2", "query": "q2", "source": "negativo", "candidatos": ["kb/a"]},
                 {"id": "c3", "query": "q3", "source": "prompt", "candidatos": []}]
        fab = {"c1": self.j("kb/a"), "c2": self.j(None), "c3": self.j(None)}
        kim = {"c1": self.j("kb/b"), "c2": self.j(None), "c3": self.j(None)}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, None, 0.0, 0.0)
        # `pasa`/`k`/`po` globales no cambian: siguen sobre las 3 filas
        # (c1 desacuerdo, c2 y c3 acuerdo estricto null-null).
        self.assertAlmostEqual(po, 2 / 3)
        # sin `negativo` NI candidatos vacíos solo queda c1 (desacuerdo total):
        # κ=0.0, p_o=0.0, sobre 1 fila -- no las 2 que darían si c3 (candidatos
        # vacíos) se colara como si fuera una fila juzgada de verdad.
        self.assertIn("κ / p_o sin `negativo` ni candidatos vacíos: 0.000 / 0.000 (sobre 1 filas; descriptivo, no decide)", inf)

    def test_construye_audita_solape_contra_el_expected_del_gold_no_del_primer_juez(self):
        """F2 del review de rama (2026-09-20): en un acuerdo lenient decidido
        por el SEGUNDO juez (aquí kimi), `expected_permalink` del gold es el
        de kimi ("kb/y"), pero la auditoría de sesgo léxico usaba
        `fab.expected or kim.expected` -que toma el de fable ("kb/x") por ser
        el primero no-nulo- TAMBIÉN para las filas del gold, no solo para los
        descartes (donde esa asimetría SÍ está declarada, Minor M-4). Repro
        exacto del reviewer: expected del gold en "kb/y", pero el
        `solape_lexico` anotado se calculaba contra "kb/x"."""
        cands = [{"id": "c1", "query": "alfa beta gamma", "source": "prompt", "candidatos": ["kb/x", "kb/y"]}]
        # fable propone expected="kb/x" (con "kb/y" en su acceptable);
        # kimi propone expected="kb/y" sin acceptable. acuerdo_fila: no
        # estricto (difieren); b.expected="kb/y" está en a.acceptable=["kb/y"]
        # -> lenient, y en fusiona() gana kimi (el `if` de "a" no aplica
        # porque a.expected="kb/x" NO está en b.acceptable=[]): exp="kb/y".
        fab = {"c1": self.j("kb/x", ["kb/y"])}
        kim = {"c1": self.j("kb/y", [])}
        textos = {"kb/x": "esto no comparte ningun token con la consulta",
                  "kb/y": "aqui si: alfa beta gamma aparecen en el cuerpo"}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, textos, 0.0, 0.0)
        self.assertEqual(gold[0]["expected_permalink"], "kb/y")
        # el solape correcto es contra "kb/y" (1.0), no contra "kb/x" (0.0).
        self.assertIn("solape_lexico=1.00", gold[0]["notes"])
        self.assertNotIn("solape_lexico=0.00", gold[0]["notes"])

    def test_construye_auditoria_lexica_por_estrato(self):
        """F8 del review de rama (2026-09-20): la auditoría pooled ("filas
        acordadas frente a descartadas") no distingue (a) sesgo léxico del
        juez de (b) un estrato con candidatas pobres que se descarta más por
        motivos ajenos al solape. Dos estratos con perfiles opuestos:
        `prompt` tiene una fila acordada de solape alto (1.0) y una
        descartada de solape bajo (0.0) -perfil "sesgo léxico"-; `hard`
        tiene una única fila acordada, de solape BAJO (0.33 < 0.5, cae en el
        subconjunto «léxicamente difícil» del guard de D-A) y CERO
        descartes -perfil "candidatas pobres en otro estrato", no aplica
        aquí, pero separa la tabla por estrato en vez de mezclarla en el
        pool-. La tabla por estrato debe reportar cada mediana por separado
        y la composición por estrato del subconjunto léxicamente difícil."""
        cands = [{"id": "c1", "query": "palabra clave", "source": "prompt", "candidatos": ["kb/a"]},
                 {"id": "c2", "query": "palabra clave dos", "source": "prompt", "candidatos": ["kb/x"]},
                 {"id": "c3", "query": "otra palabra clave", "source": "hard", "candidatos": ["kb/b"]}]
        fab = {"c1": self.j("kb/a"), "c2": self.j("kb/x"), "c3": self.j("kb/b")}
        kim = {"c1": self.j("kb/a"), "c2": self.j("kb/y"), "c3": self.j("kb/b")}
        textos = {"kb/a": "aqui la palabra clave aparece", "kb/x": "nada que ver", "kb/b": "la palabra esta aqui"}
        gold, desc, inf, k, po = ac.construye(cands, fab, kim, textos, 0.0, 0.0)
        self.assertIn("auditoría léxica por estrato", inf)
        # hard: 1 fila total, 0 descartes (tasa 0.000), mediana acordadas
        # 0.33 (solape bajo pese a no tener descartes), sin descartadas (—).
        self.assertIn("| hard | 1 | 0.000 | 0.33 | None |", inf)
        # prompt: 2 filas totales, 1 descarte de 2 (tasa 0.500), mediana
        # acordadas 1.0 (c1), mediana descartadas 0.0 (c2, desacuerdo).
        self.assertIn("| prompt | 2 | 0.500 | 1.0 | 0.0 |", inf)
        # composición por estrato del subconjunto léxicamente difícil
        # (solape < 0,5): solo `hard` (c3, solape 0.33); `prompt` no aporta
        # ninguna fila de gold con solape < 0,5 (c1 tiene solape 1.0).
        self.assertIn("composición del subconjunto «léxicamente difícil» (solape < 0,5, guard de D-A §6) por estrato: hard=1", inf)

    def test_solape_lexico(self):
        self.assertEqual(ac.solape_lexico("fusión rrf combsum", "la fusion por rrf"), 0.5)
        self.assertEqual(ac.solape_lexico("a b", "nada"), 0.0)

    def test_kappa_po_min_son_obligatorios_sin_default(self):
        """F9 del review de rama (2026-09-20): un suelo firmado (D-J11:
        κ ≥ 0,60 ∧ p_o ≥ 0,70) no debe poder olvidarse por omitirlo en el
        CLI -un `default` deja que el código, no la firma de Paul, decida
        qué suelo aplica si el operador olvida pasarlo. `--kappa-min` y
        `--po-min` son ahora obligatorios (`required=True`, sin `default`),
        igual que `--precio-entrada`/`--precio-salida` en `juez.py kimi`
        por la misma razón (Global Constraints, precios sin default)."""
        with tempfile.TemporaryDirectory() as d:
            cands = Path(d) / "c.jsonl"
            cands.write_text(json.dumps({"id": "c1", "query": "q", "source": "prompt", "candidatos": []}) + "\n", encoding="utf-8")
            fab = Path(d) / "f.jsonl"
            fab.write_text(json.dumps({"id": "c1", "expected": None, "acceptable": [], "razon": "r"}) + "\n", encoding="utf-8")
            kim = Path(d) / "k.jsonl"
            kim.write_text(json.dumps({"id": "c1", "expected": None, "acceptable": [], "razon": "r"}) + "\n", encoding="utf-8")
            base = ["acuerdo.py", "--candidatos", str(cands), "--fable", str(fab), "--kimi", str(kim),
                    "--gold-out", str(Path(d) / "g.jsonl"), "--descartes-out", str(Path(d) / "d.jsonl"),
                    "--informe-out", str(Path(d) / "i.md")]
            for argv in (base + ["--po-min", "0.70"],  # falta --kappa-min
                         base + ["--kappa-min", "0.60"]):  # falta --po-min
                viejo = sys.argv
                sys.argv = argv
                try:
                    with self.assertRaises(SystemExit) as ctx, contextlib.redirect_stderr(io.StringIO()):
                        ac.main()
                    self.assertEqual(ctx.exception.code, 2)
                finally:
                    sys.argv = viejo


if __name__ == "__main__":
    unittest.main()
