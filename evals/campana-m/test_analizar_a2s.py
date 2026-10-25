import json

from analizar_a2s import busca, veredicto


def _m(busca_frac, lo, sucia=0.0):
    return {"frac_sucia": sucia, "frac_busca": busca_frac, "d_a2s_a0": None if lo is None else {"lo": lo}}


def test_veredicto_ramas():
    # Falla si la condición pre-registrada no exige las dos patas (busca ≥ 0,5 Y lo > 0).
    assert veredicto(_m(0.5, 0.01))[0] == "EXO CON SEARCH-FIRST DEMOSTRÓ EFECTO"
    assert veredicto(_m(0.9, 0.0)) == ["BUSCA Y NO APLICA", "MEMORIA MÍNIMA CONFIRMADA"]
    assert veredicto(_m(0.49, 0.2)) == ["EFECTO SIN BÚSQUEDA", "MEMORIA MÍNIMA CONFIRMADA"]
    assert veredicto(_m(0.1, -0.1)) == ["EL REFLEJO NO HACE BUSCAR", "MEMORIA MÍNIMA CONFIRMADA"]
    assert veredicto(_m(0.9, 0.5, sucia=0.06)) == ["NO ADJUDICABLE"]
    assert veredicto(_m(0.9, None)) == ["SIN DATOS a2s-a0"]


def _transcript(tmp_path, cmds):
    p = tmp_path / "t.jsonl"
    p.write_text("\n".join(json.dumps({"type": "assistant", "message": {"content": [
        {"type": "tool_use", "name": "Bash", "input": {"command": c}}]}}) for c in cmds) + "\nno-json\n")
    return str(p)


def test_busca_solo_invocaciones_reales(tmp_path):
    # Falla si `exo --help`, `exo lint` o el texto «exo search» en un echo cuentan como búsqueda.
    assert busca(_transcript(tmp_path, ["exo search --type hybrid foo"]))
    assert busca(_transcript(tmp_path, ["cd x && /usr/bin/exo recall --query y"]))
    assert busca(_transcript(tmp_path, ["exo targets foo --json"]))
    assert not busca(_transcript(tmp_path, ["exo --help", "exo lint", "echo 'usa exo-search'"]))
    assert not busca(_transcript(tmp_path, ["exo targets --help", "exo search -h", "exo recall x --help"]))
    assert busca(_transcript(tmp_path, ["exo targets --help; exo search foo"]))
    assert not busca(str(tmp_path / "no-existe.jsonl"))
