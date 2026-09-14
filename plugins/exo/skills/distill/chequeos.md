# Chequeos de los pasos 1 y 1b

**Cuándo cargar:** en los pasos 1 y 1b, al interpretar la salida de
`budget`/`ratchet`/`lint`/`stale` o si algún binario falla.

### 1. Budget check

Corre `$EXO_BIN budget --json`. Devuelve
`{data:{tiers:[{tier,notes,bytes,budget,delta,exceeded}], offenders:[{path,tier,size_bytes,budget}], waived:[{path,tier,size_bytes,budget}]}}`
y **exit 3 si hay algún offender** (incluye NOTIER: nota sin `tier:` o con tier
ilegal), exit 0 si limpio. Una nota que rebasa su
presupuesto de tier pero cae dentro de su `kbx_budget_max: N` de frontmatter
es una excepción reconocida: exit 0, listada en `waived` (no en `offenders`).
Presupuestos por defecto: core=8.500B, stable=12.500B, log=sin límite; excluye
`archive/`, `docs/`, `.superpowers/`.

> Corre también `$EXO_BIN ratchet --kb $KB_ROOT --json` con el árbol limpio. Los findings
> `no-air-debt` listan las notas cuyo techo está sellado a ras: no bloquean nada
> (la guarda juzga transiciones, no estado), pero cada una es un mordisco
> pendiente. Su campo `limit` da el techo que cumpliría y el mensaje el tamaño
> objetivo de poda. Es la cola de trabajo de esta pasada.

Y en el mismo paso 1, la otra mitad de la misma deuda: `exo budget` reporta en
`no_air` (línea `no-air:` en texto) las notas **sin waiver** que están a menos
del 15% de su nominal de tier. La guarda del ratchet solo cubre techos
declarados, así que sin esto una nota que vive de su nominal puede quedarse a
19 bytes del muro sin que nadie lo vea — declarar un waiver te mete bajo
vigilancia y no declararlo te libra de ella. No bloquea (exit 0); cada línea
trae el tamaño objetivo. Trátalas como la misma cola de trabajo que las
`no-air-debt`: el remedio es partir canon/bitácora, no comprimir.

Revisa `waived`: ¿siguen justificadas las excepciones reconocidas? (p.ej. un
`kbx_orphan_ok` en una nota que recuperó relaciones desaparece de `waived` por
sí solo).

**Falla-fuerte:** si el binario no está, **para** con un mensaje accionable
(`exo no está → cargo build --release en engine/ + copia a
$HOME/.local/bin/exo(.exe)`). No degrades a mano: /distill es offline y
deliberado, el fallo ruidoso es correcto. (El check `schema_drift` que esta
sección citaba murió en G4b — `exo lint` emite 6 tipos de finding, no 7;
existía solo mientras kbx y exo convivían contra el mismo schema,
`engine/src/lint.rs:1-5` — así que ya no hay "schema drift" que mirar.)

### 1b. Gate de deriva + priorización

- **Deriva:** corre `$EXO_BIN lint --json`
  (`{data:{ok,findings:[{type,path,detail}], waived:[{type,path,detail}]}}`).
  `ok:true` significa limpio de findings NO waived. Las excepciones
  reconocidas (`orphan` con `kbx_orphan_ok: true`, `budget_exceeded` dentro de
  su `kbx_budget_max`) aterrizan en `waived`, no en `findings`. Sus findings
  alimentan la limpieza (WS4 del spec Fase 2): `duplicate_dir`, `orphan`,
  `bad_frontmatter`, `root_file`. No los muevas a ciegas — cada `git mv` lo
  gatea Paul.
- **Priorización:** corre `$EXO_BIN stale --json`
  (`{data:{notes:[{path,tier,age_days,degree,score,...}]}}`, orden descendente
  por `score`). Úsalo para decidir QUÉ notas atacar primero en los pasos 2 y 4,
  en vez de ir a ojo. **`stale` no propaga waivers**: una nota con excepción
  reconocida (p.ej. README/metodología) seguirá apareciendo alta en `stale`
  (es advisory) — no es bug.
- Chequea inject-failed E inject-abstained en reflex-log.jsonl (jq 'select(.reflex=="inject-failed" or .reflex=="inject-abstained")'): >0 sostenido = componedor roto en silencio o payloads sin agent_type — never-break no puede significar semanas sin inyección (spec transporte §7).
- Chequea también `recall-fallback` con `reason=truncated`
  (`jq 'select(.reflex=="recall-fallback" and (.detail|test("truncated")))'`).
  Cada uno es un arranque servido incompleto: el cuerpo de `core-index`
  sobrevive siempre —el guard busca "Contrato de memoria", que está al
  principio— y lo que se cae por el final son los punteros de actividad
  reciente, sin que nada lo diga. Sostenido = `core-index` está sobresuscrito y
  toca evicción del índice (entradas muertas y justificaciones, nunca comprimir
  entradas vivas). Compruébalo con el bloque real, no con `wc` del fichero:

      exo recall --db ~/.exo/index.db --content \
          --note kb-demo/core/core-index --limit 10 --cap-bytes 6144

  Un `aviso: … truncado` en stderr es la señal.
