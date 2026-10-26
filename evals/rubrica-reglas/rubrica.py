"""Rúbrica de selección de `## Reglas duras` (spec 2026-10-26, §4 y §5.2): validación de etiquetas, orden y recorte."""
import re

CHOQUES = ("P", "D", "E")
COSTES = ("alto", "medio", "bajo")
PREFIJO_CITA = 60


def valida_etiqueta(obj):
    if not isinstance(obj, dict):
        return None
    if not isinstance(obj.get("id"), str) or not obj["id"]:
        return None
    if obj.get("choque") not in CHOQUES or obj.get("coste") not in COSTES:
        return None
    out = {"id": obj["id"], "choque": obj["choque"], "coste": obj["coste"]}
    for k in ("justificacion", "ruta", "cita_recortada"):
        if k in obj:
            out[k] = obj[k]
    return out


def choque_efectivo(etiqueta, criba):
    if criba is True:
        return "E"
    if criba is False:
        return etiqueta["choque"] if etiqueta and etiqueta["choque"] in ("P", "D") else "D"
    return etiqueta["choque"] if etiqueta else None


def _orden_id(id):
    m = re.fullmatch(r"g(\d+)-(\d+)", id)
    return (int(m[1]), int(m[2])) if m else (float("inf"), 0)


def clave(id, etiqueta, criba, vetada):
    choque = choque_efectivo(etiqueta, criba)
    return (not vetada,
            CHOQUES.index(choque) if choque else len(CHOQUES),
            COSTES.index(etiqueta["coste"]) if etiqueta else len(COSTES),
            _orden_id(id), id)


def orden(items, vetadas=frozenset()):
    return sorted(items, key=lambda i: clave(i["id"], i["etiqueta"], i["criba"], i["id"] in vetadas))


def _norm(s):
    return " ".join(s.lower().split())


def recorta_nota(texto, cita):
    lineas, fuera, en_seccion = texto.split("\n"), [], False
    for l in lineas:
        if re.match(r"##\s+Reglas duras\s*$", l):
            en_seccion = True
        elif l.startswith("## "):
            en_seccion = False
        if not en_seccion:
            fuera.append(l)
    aguja = _norm(cita)[:PREFIJO_CITA]
    if not aguja:
        return "\n".join(fuera), False
    con_cita = [l for l in fuera if aguja in _norm(l)]
    if con_cita:
        return "\n".join(l for l in fuera if aguja not in _norm(l)), True
    bloques = re.split(r"\n[ \t]*\n", "\n".join(fuera))
    quedan = [b for b in bloques if aguja not in _norm(b)]
    if len(quedan) == len(bloques):
        return "\n".join(fuera), False
    return "\n\n".join(quedan), True
