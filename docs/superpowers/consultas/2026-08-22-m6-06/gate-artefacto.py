# Artefacto del gate de M6-06 — la lista y la normalización que produjeron los números

Este fichero es el ORIGINAL que midió el gate sobre los 272 prompts reales de Paul.
Se commitea porque **todos los números del gate son propiedades de esta lista
exacta**: 86% de tasa de disparo, FN topical = 0, y "los 39 que salta son ack/git
puro". Una lista reescrita de memoria hereda los claims sin heredar el artefacto que
los hizo ciertos — y §0 retiró la maquinaria para re-medirlos.

Procedencia: scratchpad del consultor Fable (2026-08-22), Apéndice B de
`consultor-m6-06.md`. Rescatado por la review de la spec (B1).

Al implementar `recall-inject.sh`, la lista y `norm()` se traducen a bash TAL CUAL:

- `STOP` tiene **127 entradas únicas** (129 con dos duplicados inofensivos: `una`,
  `para`). La spec decía "~50" — número heredado del verdict, corregido allí.
- `norm()` NO quita toda la puntuación: **conserva `/`, `.` y `-`** tras NFD +
  minúsculas + strip de acentos. Implementar "sin puntuación" literal construye un
  gate distinto del medido.
- La tokenización es por whitespace.

**Delta conocido con la spec**: `gate_B` aquí NO implementa las reglas de skip para
prompts que empiezan por `/` o `!` (comandos al harness), que la spec añadió después.
Esa regla solo puede BAJAR la tasa de disparo respecto al 86% medido aquí, nunca
subirla, y no puede silenciar ningún prompt topical. El resto del gate es idéntico.

`gate_A` (el gate por longitud) se conserva porque es el término de comparación de
todos los deltas del Apéndice B; **está retirado del diseño**, no se implementa.

Dependencia: `prompts_clean.jsonl` (272 prompts humanos extraídos de los .jsonl de
sesión) NO se commitea — contiene los prompts de Paul en crudo. Para re-medir hay que
regenerarlo con el extractor descrito en el Apéndice B.

---

import json, os, re, unicodedata

prompts = [json.loads(l)['t'] for l in open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'prompts_clean.jsonl'))]

def norm(tok):
    t = unicodedata.normalize('NFD', tok.lower())
    t = ''.join(c for c in t if unicodedata.category(c) != 'Mn')
    return re.sub(r'[^a-z0-9/.-]','',t)

# Lista B: función + acks + verbos de sesión sin objeto KB. Corta y auditable.
STOP = set('''el la los las un una unos unas de del a al en con por para y o u e que se lo le les mi tu su es son era esta este esto esa ese eso estas estos ya no ni si sí tambien tampoco pero aunque como cuando donde cual cuales muy mas menos bien mal solo ahora luego antes despues aqui ahi alla hoy
ok okey vale dale va venga listo perfe perfecto genial claro exacto correcto gracias adelante
di haz hazlo corre lanza lanzalo revisa arregla borra quita usa guarda sube baja sigue continua para espera prueba mira pon dime
pushea push commitea commit mergea mergealo merge rama ramas repo repos master main pr
uno una dos tres cuatro cinco 1 2 3 4 5
'''.split())

ACK_RE = re.compile(r"^(s[ií]|ok(ey|i)?|vale|dale( pues)?|va|go|sigue|contin[uú]a|adelante|hazlo|commitea.*|push(ea)?.*|mergea.*|perfecto|genial|bien|no|para|espera|gracias|claro|exacto|correcto|de acuerdo|listo|venga|a ver|muy bien|si dale|jaja\S*)[\s.!…]*$", re.I)

def gate_A(t):  # longitud: skip=True
    if t.startswith('<'): return True
    if len(t.split()) < 4 or len(t.encode()) < 25: return True
    return bool(ACK_RE.match(t))

def gate_B(t):  # léxico: skip si TODOS los tokens son stopword/ack
    if t.startswith('<'): return True
    toks = [norm(w) for w in t.split()]
    toks = [w for w in toks if w]
    if not toks: return True
    return all(w in STOP or w.isdigit() for w in toks)

fa = [t for t in prompts if not gate_A(t)]
fb = [t for t in prompts if not gate_B(t)]
print(f"total {len(prompts)} | dispara A (longitud): {len(fa)} ({100*len(fa)/len(prompts):.0f}%) | dispara B (léxico): {len(fb)} ({100*len(fb)/len(prompts):.0f}%)")
solo_B = [t for t in prompts if gate_A(t) and not gate_B(t)]
solo_A = [t for t in prompts if gate_B(t) and not gate_A(t)]
print(f"\nB dispara y A calla ({len(solo_B)}): TODOS:")
for t in solo_B: print('  +', repr(t[:70]))
print(f"\nA dispara y B calla ({len(solo_A)}):")
for t in solo_A: print('  -', repr(t[:70]))
skip_B = [t for t in prompts if gate_B(t)]
print(f"\nB salta ({len(skip_B)}): TODOS:")
for t in skip_B: print('  ·', repr(t[:60]))
