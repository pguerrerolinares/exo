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

**Cola por calor.** `no_air` mezcla frentes calientes con fichas congeladas.
Crúzala con `$EXO_BIN stale --json` (`age_days`, días desde el último commit de
la nota): una nota con `age_days` > 30 **no es trabajo de la pasada** — se
anota y se deja. Ataca primero las calientes. Si la cola entera está fría,
dilo en el resumen.

**Disparador de partición.** Para cada nota caliente de la cola:

    git -C $KB_ROOT log --since=7.days --fixed-strings --grep='Gate: <ruta>' --format=%h

≥2 commits (trailers `Gate:` de `/document`) en 7 días, sin commit
`docs(kb): auto-distill` entre medias, indica que el auto-distill no basta:
partición con juicio (test del título, `consolidacion.md`), no otra evicción.
**`--fixed-strings` es obligatorio**: las rutas con ` — ` y paréntesis no
casan como regex y el disparador calla sin avisar.

Revisa `waived`: ¿siguen justificadas las excepciones reconocidas? (p.ej. un
`kbx_orphan_ok` en una nota que recuperó relaciones desaparece de `waived` por
sí solo).

**Falla-fuerte:** si el binario no está, **para** con un mensaje accionable
(`exo no está → cargo build --release en engine/ + copia a
$HOME/.local/bin/exo(.exe)`). No degrades a mano: /distill es offline y
deliberado, el fallo ruidoso es correcto. (El check `schema_drift` que esta
sección citaba murió en G4b — en `exo lint` su hueco lo ocupa `index_stale`, así que
siguen siendo siete tipos de finding, pero no los mismos siete; existía
solo mientras kbx y exo convivían contra el mismo schema,
`engine/src/lint.rs:1-10` — así que ya no hay "schema drift" que mirar.)

### 1b. Gate de deriva + priorización

- **Deriva:** corre `$EXO_BIN lint --json`
  (`{data:{ok,findings:[{type,path,detail}], waived:[{type,path,detail}]}}`).
  `ok:true` significa limpio de findings NO waived. Las excepciones
  reconocidas (`orphan` con `kbx_orphan_ok: true`, `budget_exceeded` dentro de
  su `kbx_budget_max`) aterrizan en `waived`, no en `findings`. Sus findings
  alimentan la limpieza (WS4 del spec Fase 2): `duplicate_dir`, `orphan`,
  `bad_frontmatter`, `root_file`. No los muevas a ciegas — cada `git mv` lo
  gatea el dueño de la KB.
  `broken_link` (`detail: [[texto]]`) es un wikilink sin resolver desde una nota
  fuera de `archive/`: el texto no coincide con ningún `title` ni `permalink`
  exacto. Arréglalo en el origen reusando el title exacto del destino; si el
  destino no existe, quita el link. Lee el índice: si dudas de que esté al día,
  corre antes `$EXO_BIN index`.
- **Priorización:** corre `$EXO_BIN stale --json`
  (`{data:{notes:[{path,tier,age_days,degree,score,...}]}}`, orden descendente
  por `score`). Úsalo para decidir QUÉ notas atacar primero en los pasos 2 y 4,
  en vez de ir a ojo. **`stale` no propaga waivers**: una nota con excepción
  reconocida (p.ej. README/metodología) seguirá apareciendo alta en `stale`
  (es advisory) — no es bug.
- Chequea inject-failed E inject-abstained en reflex-log.jsonl (jq 'select(.reflex=="inject-failed" or .reflex=="inject-abstained")'): >0 sostenido = componedor roto en silencio o payloads sin agent_type — never-break no puede significar semanas sin inyección (spec transporte §7).
- Chequea también `recall-fallback` con `reason=truncated`
  (`jq 'select(.reflex=="recall-fallback" and (.detail|test("truncated")))'`).
  Cada uno es un arranque servido incompleto. Chequea además `recall-truncated`
  (`jq 'select(.reflex=="recall-truncated")'`): el bloque de arranque tiene un
  cap de 1.500 B y el payload (`bytes=<antes> cap=1500`) dice cuánto se recortó.
  El bloque es el contrato de `core-index` más los avisos, y el corte es por el
  final: caen antes los avisos que el contrato. Sostenido = el contrato ha
  crecido más de lo que cabe, y toca evicción (entradas muertas y
  justificaciones, nunca comprimir reglas ni entradas vivas). Las reglas duras
  no viajan en este bloque; su tope se mide aparte:

      exo rules --check --kb <kb>

  Sale 3 y lista `excesos` (nota y nº de reglas) si alguna sección pasa de 10.

## Cobertura de reglas duras (#207)

`cobertura-reglas.sh` cruza dos caras. Lo declarado sale de
`exo rules --coverage --kb <kb>` (envelope `rules-coverage`, `data.notas`:
nota, nº de reglas y estado `ok`/`al_cap`/`excede`/`sin_seccion`/
`seccion_vacia`; sale 0 siempre, el gate es `--check`). Lo no declarado sale
de los `project-rules-skip` del reflex-log: repos donde arrancó una sesión y
`exo rules` no entregó. Hoy esos skips no se le muestran a nadie en la sesión.

- **`al_cap`**: la nota tiene 10 reglas y lo siguiente que entre desplaza a
  otra. No es un fallo. Si en el periodo se decidió algo que debería ser regla
  dura, la rúbrica de `document/routing.md` decide cuál sale: aplícala y anota
  la bitácora «salió `<regla>` por `<criterio>`».
- **`sin_nota` / `sin_seccion` con arranques recientes**: hay sesiones en ese
  repo que vuelven al default. Propón una nota, o una nota hermana
  `<repo> — reglas.md` con `slug: <repo>` y solo `## Reglas duras` cuando el
  repo comparte nota con otros. Si el dueño ya decidió dejarlo fuera, no lo
  repropongas.
- **El último skip anterior a la nota o sección que lo tapó** es un hueco ya
  cerrado; no cuenta.
- Las notas `sin_seccion` que no casan con ningún repo (decisiones, runbooks)
  solo se cuentan en el resumen: no son huecos.
