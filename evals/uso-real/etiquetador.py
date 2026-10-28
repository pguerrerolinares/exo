#!/usr/bin/env python3
"""Etiquetador HTML local de los lotes ciegos de uso real (kill 2 y kill 3 de delegar-decision).

Incrusta los ítems en `etiquetador.html` y escribe una página autocontenida (sin red, se abre como file://). La página
va a la cache, nunca al repo: lleva el texto de los mensajes. Guarda las etiquetas en el localStorage del navegador
por lote y «Exportar JSONL» las descarga:
- kill3: lee `etiquetar-kill3.jsonl` (`kill3.py lote`), etiqueta si|no y exporta el mismo JSONL con "etiqueta"
  rellena, listo para `kill3.py resultado --etiquetas`. Regenerar la página desde un export retoma lo etiquetado.
- kill2: lee `hoja.md` + `muestra.json` de los 30 disparos P2|P3, etiqueta delegar|consultor|mia|? y exporta
  {id, etiqueta, nota}."""
import argparse, hashlib, json, os, re, sys

H = os.path.dirname(os.path.abspath(__file__))
PLANTILLA = os.path.join(H, "etiquetador.html")
HUECO = "/*__CFG__*/{}"

# Definición de la relectura del 2026-10-27 (spec 2026-10-28 §6.3).
INSTR_KILL3 = ("¿Es una corrección tuya que pide delegar (en un subagente/ejecutor o en un consultor) lo que el padre "
               "hacía o iba a hacer él? Sí o no, sin pensar en cuándo pasó.")

MODOS = {
    "kill3": {
        "titulo": "Kill 3 · corrección de delegación sí/no",
        "instr": INSTR_KILL3,
        "ops": [["si", "sí: pide delegar"], ["no", "no"]],
        "positivas": ["si"],
        "metrica": "marcados sí",
        "cortes": None,
        "cab": None,
        "campos": [["previo", "Final del turno del padre", "previo"], ["texto", "Tu mensaje", "texto"]],
        "exporta": "completo",
        "fichero": "etiquetar-kill3.etiquetado.jsonl",
    },
    "kill2": {
        "titulo": "Etiquetado de disparos P2|P3",
        "instr": "Marca lo que habrías querido en ese momento, no lo que pasó.",
        "ops": [["delegar", "quería que delegara (trabajo)"], ["consultor", "quería consultor (decisión)"],
                ["mia", "la pregunta era mía"], ["?", "?"]],
        "positivas": ["delegar", "consultor"],
        "metrica": "precisión = (delegar + consultor) / etiquetados",
        "cortes": [["alto", "≥ 50 %", "gate con bloqueo"], ["medio", "20–50 %", "gate que solo registra ~30 sesiones"],
                   ["bajo", "≤ 20 %", "sin hook"]],
        "cab": "tema",
        "campos": [["antes", "Tú antes", ""], ["cierre", "Cierre del padre", "cierre"], ["resp", "Tu respuesta", "resp"]],
        "exporta": "minimo",
        "fichero": "etiquetas-kill2.jsonl",
    },
}


def items_kill3(path):
    with open(path) as f:
        items = [json.loads(l) for l in f if l.strip()]
    for it in items:
        if not {"id", "previo", "texto"} <= set(it):
            raise ValueError(f"ítem sin id/previo/texto: {str(it)[:80]}")
    return items


def items_kill2(hoja, muestra):
    """Parsea la hoja de los disparos («## n. tema» + campos en negrita + <sub>id corto</sub>) contra muestra.json."""
    with open(hoja) as f:
        bloques = re.split(r"^## ", f.read(), flags=re.M)[1:]
    with open(muestra) as f:
        ids = json.load(f)["muestra"]
    items = []
    for b in bloques:
        n, tema = b.split("\n", 1)[0].split(". ", 1)
        campo = lambda etiqueta: re.search(r"- \*\*" + etiqueta + r":\*\* (.*)", b).group(1).strip()
        corto = re.search(r"<sub>id (\w+)</sub>", b).group(1)
        largo = ids[int(n) - 1]
        if not largo.startswith(corto):
            raise ValueError(f"ítem {n}: id {corto} no casa con muestra.json ({largo})")
        items.append({"id": largo, "tema": tema.strip(), "antes": campo("Tú antes"),
                      "cierre": campo("Cierre del padre"), "resp": campo("Tu respuesta")})
    if len(items) != len(ids):
        raise ValueError(f"{len(items)} ítems en la hoja y {len(ids)} en muestra.json")
    return items


def pagina(modo, items, plantilla=PLANTILLA):
    ids = [it["id"] for it in items]
    if len(set(ids)) != len(ids):
        raise ValueError("ids repetidos en el lote")
    cfg = {**MODOS[modo], "modo": modo, "items": items,
           # Clave del localStorage: cambia si cambia el lote, para no mezclar etiquetas de lotes distintos.
           "lote": hashlib.sha256("\n".join(sorted(ids)).encode()).hexdigest()[:12]}
    with open(plantilla) as f:
        tpl = f.read()
    if tpl.count(HUECO) != 1:
        raise ValueError(f"la plantilla no tiene exactamente un {HUECO}")
    # "</" escapado: un "</script>" dentro de un mensaje cerraría el bloque.
    return tpl.replace(HUECO, json.dumps(cfg, ensure_ascii=False).replace("</", "<\\/"))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="modo", required=True)
    k3 = sub.add_parser("kill3")
    k3.add_argument("--items", required=True, help="etiquetar-kill3.jsonl de kill3.py lote (o un export ya empezado)")
    k3.add_argument("--salida", required=True, help="HTML de salida, en la cache")
    k2 = sub.add_parser("kill2")
    k2.add_argument("--hoja", required=True)
    k2.add_argument("--muestra", required=True)
    k2.add_argument("--salida", required=True, help="HTML de salida, en la cache")
    a = ap.parse_args(argv)
    try:
        items = items_kill3(a.items) if a.modo == "kill3" else items_kill2(a.hoja, a.muestra)
        html = pagina(a.modo, items)
    except (OSError, ValueError, KeyError, AttributeError) as e:
        print(f"etiquetador: {e}", file=sys.stderr)
        return 1
    with open(a.salida, "w") as f:
        f.write(html)
    ya = sum(bool(it.get("etiqueta")) for it in items)
    destino = os.path.abspath(a.salida)
    distro = os.environ.get("WSL_DISTRO_NAME")
    print(f"{len(items)} ítems ({ya} ya etiquetados) en {destino}"
          + (f"\nDesde Windows: \\\\wsl.localhost\\{distro}{destino.replace('/', chr(92))}" if distro else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
