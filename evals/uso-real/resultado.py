#!/usr/bin/env python3
"""Resultado público de la medición (spec 2026-10-26 §4.3-§4.6): solo conteos, ids y veredictos.

La población se rehace con el `desde` del sello y se compara con `sesiones_sha256`; si no casa, sello_ok=false y exit 2.
`final` solo se calcula si la puerta abre la etapa 2 y hay --atribuciones. --validacion2 se copia tal cual.
Códigos de salida: 0 ok, 1 entrada inválida, 2 el sello no casa."""
import argparse, glob, json, os, sys

FRACCION_SIN_VEREDICTO = 0.10
H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, H)
import deterministas, juez, puerta, transcripts
from sellar import sha_sesiones


def _jsonl(path):
    with open(path) as f:
        return [json.loads(l) for l in f if l.strip()]


def _json(path):
    with open(path) as f:
        return json.load(f)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    for x in ("detecciones", "proyectos", "sello", "salida"):
        ap.add_argument("--" + x, required=True)
    for x in ("atribuciones", "validacion", "validacion2"):
        ap.add_argument("--" + x)
    a = ap.parse_args()

    sello = _json(a.sello)
    sesiones, _ = transcripts.poblacion(a.proyectos, sello.get("desde", "2026-10-06"), sello.get("hasta"))
    sello_ok = (sha_sesiones(sesiones) == sello["sesiones_sha256"] and len(sesiones) == sello["n_sesiones"]
                and not juez.comprueba_sello(sello, {"filtro_sha256": transcripts.filtro_sha256()}))
    if not sello_ok:
        with open(a.salida, "w") as f:
            json.dump({"sello_ok": False}, f)
            f.write("\n")
        print("resultado: la población no casa con el sello", file=sys.stderr)
        return 2

    dets = _jsonl(a.detecciones)
    poblacion = {(s["id"], m["id"]) for s in sesiones for m in s["mensajes"]}
    vistas = {(d.get("sesion"), d.get("id")) for d in dets}
    ajenas = vistas - poblacion
    if ajenas:
        print(f"resultado: {len(ajenas)} detecciones fuera de la población sellada, p. ej. {sorted(map(str, ajenas))[0]}",
              file=sys.stderr)
        return 1
    faltan = len(poblacion - vistas)
    validacion = _json(a.validacion) if a.validacion else None
    p = puerta.decide(sum(d.get("correccion") == "si" for d in dets), sello["n_sesiones"])
    # Un juez sin validar no estabiliza ni abre la etapa 2: solo juez_valido True deja pasar la puerta.
    if validacion is None:
        p["estado"] = "sin_validar"
    elif validacion.get("juez_valido") is not True:
        p["estado"] = "juez_invalido"
    if faltan:
        p["estado"] = "incompleto"

    atribs = _jsonl(a.atribuciones) if a.atribuciones else []
    final = None
    n_sin_etapa2 = sum(bool(x.get("sin_veredicto")) for x in atribs)
    if atribs and p["estado"] == "etapa2":
        final = puerta.final([x["atribucion"] for x in atribs if "atribucion" in x], sello["n_sesiones"])
        # Un positivo sin atribución no es «decidido»: si son muchos, la tasa final sesgaría hacia estabilizar.
        if n_sin_etapa2 > FRACCION_SIN_VEREDICTO * p["k"]:
            final["estado"] = "incompleto"

    hasta = sello.get("hasta")
    por_repo = {}
    for s in sesiones:
        base = os.path.join(a.proyectos, s["repo_dir"])
        r = por_repo.setdefault(s["repo_dir"], ([], []))
        r[0].append(os.path.join(base, s["id"] + ".jsonl"))
        r[1].extend(sorted(glob.glob(os.path.join(base, s["id"], "subagents", "*.jsonl"))))

    out = {
        "puerta": p,
        "final": final,
        "deterministas": {
            "principales": deterministas.cuenta([f for r in por_repo.values() for f in r[0]], hasta),
            "subagentes": deterministas.cuenta([f for r in por_repo.values() for f in r[1]], hasta),
            "por_repo": {repo: {"principales": deterministas.cuenta(r[0], hasta),
                                "subagentes": deterministas.cuenta(r[1], hasta)} for repo, r in sorted(por_repo.items())},
        },
        "n_sin_veredicto": sum(bool(d.get("sin_veredicto")) for d in dets),
        "n_sin_veredicto_etapa2": n_sin_etapa2,
        "n_detecciones_faltantes": faltan,
        "n_nota_ausente": sum(bool(x.get("nota_ausente")) for x in atribs),
        "validacion": validacion,
        "desde": sello.get("desde", "2026-10-06"),
        "hasta": hasta,
        "sello_ok": True,
    }
    if a.validacion2:
        out["validacion2"] = _json(a.validacion2)
    with open(a.salida, "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
