# Una sola grafía de ruta, y la ruta en la salida humana de `search`

**Fecha:** 2026-09-11 · **Estado:** aprobada, pendiente de plan
**Origen:** incidente observado en una sesión de `/document` (abajo)

## 1. El incidente

Un agente quiso sacar rutas de fichero de la KB y escribió:

```bash
exo search --type hybrid --json "<query>" | jq -r '.data[] | "\(.score) \(.permalink) \(.path)"'
```

```
jq: error (at <stdin>:1): Cannot index number with string ("path")
```

`.data` es un objeto (`elapsed_s`, `query`, `results`, `search_type`), no un array;
`.data[]` itera VALORES y el primero es un número. La ruta correcta es
`.data.results[]`. El agente quemó dos tool-calls descubriéndolo por prueba y error.

El error fue el síntoma barato. Debajo había dos defectos, y el segundo llevaba
meses sirviendo datos rotos sin que nadie lo viera.

## 2. Diagnóstico

### 2.1 Defecto A — la salida humana de `search` no lleva la ruta

`engine/src/main.rs:917` imprime tres columnas:

```rust
println!("{}\t{}\t{:.4}", r.permalink, r.tipo, r.score);
```

`path` se quedó fuera, pese a que el doc-comment del campo
(`engine/src/buscador.rs:24-28`) dice exactamente para qué existe: «sin este
campo, cuando muera basic-memory el agente no tiene forma de localizar el
fichero que va a editar con `Edit`» — porque **el permalink no es invertible**
(el slug come acentos, espacios y em-dashes).

Consecuencia: el único camino al dato pasa por `--json` + jq. El agente del
incidente no tuvo mala suerte; tomó el único camino que había.

### 2.2 Defecto B — dos grafías de ruta en el mismo binario

Hay una sola fuente de rutas relativas (`notas.ruta`, única columna con rutas del
esquema, `engine/src/schema.rs:22`), pero **dos funciones que la construyen**, en
módulos hermanos, para la misma operación:

```
engine/src/walker.rs:118-131    strip_prefix(raiz).to_string_lossy().replace('\\', "/")   → "/"
engine/src/indexer.rs:461-472   strip_prefix(kb).to_string_lossy().into_owned()           → nativo
```

`ruta_relativa` no tiene el `.replace` que el walker sí tiene. Medido sobre la KB
real en Windows el 2026-09-11:

| Superficie | Alimentada por | Valor real |
|---|---|---|
| `budget --json` `.data.offenders[].path` | walker | `projects/lighthouses-bot.md` |
| `lint --json` `.data.waived[].path` | walker | `README.md` |
| `search --json` `.data.results[].path` | índice | `backlog\Backlog — Memoria v2.md` |
| `recall --json` / humano | índice + `kb.join()` | `C:/proyectos/homework/wisdom-paul\backlog\….md` |

La fila de `recall` es la cara: `engine/src/recall.rs:549-551` hace
`kb.join(&nota.ruta)` con `kb` en `/` (viene de config) y `ruta` en `\` (viene del
índice), y produce una ruta **mixta**.

Y esa ruta mixta es la que `plugins/exo/scripts/recall-inject.sh` inyecta en
**cada `UserPromptSubmit`**. Bloque real capturado el 2026-09-11:

```
=== Recall exo (… material de la KB en C:/proyectos/homework …) ===
- wisdom-paul\log\kbx-bitacora.md — kbx-bitacora
- wisdom-paul\archive\log\backlog-diario-2026-07-05_2026-08-03.md — backlog-diario-…
```

Tres daños en dos líneas:

1. La raíz común sale mal (`C:/proyectos/homework` en vez de `…/wisdom-paul`),
   porque `recall-inject.sh:272-289` hace `split("/")` sobre una ruta mixta.
2. Las rutas llevan `\`: pegadas en bash, la barra se come como escape y la ruta
   deja de existir.
3. La deduplicación título↔stem nunca acierta (`kbx-bitacora — kbx-bitacora`),
   por el mismo `split("/")`.

Nadie lo había visto porque **la forma es válida** — ley 1 de
[[Fallo silencioso — el instrumento que no grita]].

### 2.3 La prosa miente, y un test ya la contradecía

`plugins/exo/agents/executor.md:13` y `plugins/exo/skills/document/SKILL.md:21`
dicen que `search --json` devuelve **`ruta`**. El JSON emite `path`: `ruta` es el
nombre del campo Rust y muere en el `#[serde(rename = "path")]` de
`buscador.rs:30`. Un agente que escriba `.data.results[] | .ruta` obtiene `null`
por fila **sin error de jq**.

Lo llamativo: `engine/tests/contrato_envelope.rs:112` ya asevera que `ruta` no
sobrevive a la serialización. El test dice que `ruta` no existe y la prosa dice
que sí; los dos en verde, porque ninguno mira al otro. «Contrato por prosa», ley 3.

Un muestreo de 9 afirmaciones de forma/campo/flag en `plugins/exo/**/*.md`
contra structs y `--help` dio 8 correctas: `ruta` es la única mentira. No hace
falta auditar toda la superficie.

## 3. Decisiones

### D0 — `ruta_relativa` adopta la convención que el repo ya tiene

`engine/src/indexer.rs:461-472` añade `.replace('\\', "/")`, igualando
`walker.rs:131`. **No es una convención nueva**: el repo ya la decidió, y una
función de dos no se enteró.

Se normaliza **incondicionalmente**, igual que el walker, no bajo `#[cfg(windows)]`.
Motivo: en Unix `\` es un carácter legal en un nombre de fichero, así que ambas
funciones tienen ahí la misma arista — pero si el indexer la gateara y el walker
no, discreparían en Linux ante un fichero `a\b.md`, y esa nota se vería
«borrada» y reinsertada en cada corrida. Discrepar es peor que compartir la
arista. La arista va al backlog como **un** ítem que cubre a las dos.

Con D0, `search --json` y `write append` quedan correctos sin tocar su punto de
emisión: leen `notas.ruta` y la emiten tal cual. Las rutas **absolutas** no —
ver D0b.

### D0b — las rutas absolutas se concatenan con `/`, nunca con `Path::join`

`Path::join` en Windows empuja con `\` cuando lo que se añade no empieza por
separador. Prueba, en la salida real de hoy:

```
C:/proyectos/homework/wisdom-paul\backlog\Backlog — Memoria v2.md
                                 ↑ este lo puso join, no el índice
```

Así que D0 por sí sola deja `recall` en `…wisdom-paul\backlog/Backlog.md`: un
`\` menos, mixta igual. Toda **emisión** de ruta absoluta concatena
explícitamente con `/`:

| Sitio | Hoy | Después |
|---|---|---|
| `engine/src/recall.rs:551` | `kb.join(&nota.ruta).display().to_string()` | `format!("{}/{}", kb_portable, nota.ruta)` |
| `engine/src/escritor.rs:300,347` (`ruta_abs` del envelope) | `ruta_abs.display().to_string()` | ídem |
| D1, la 4ª columna | — | ídem |

`kb_portable` es la raíz con `\`→`/` (la config puede traerla en cualquiera de
las dos formas). El `PathBuf` sigue usándose para **E/S** —`escribe_atomico`,
`anexa`— sin tocar: esto es sobre lo que se *emite*, no sobre cómo se abre un
fichero.

Regla para el ejecutor: si vas a `display()` una ruta que va a salir por stdout
o por el envelope, ese `display()` es el bug. Es el mismo olvido que D0 arregla
en `ruta_relativa`, una capa más arriba.

### D0c — migración en sitio, idempotente, en el camino de escritura

`indexa` compara por cadena exacta (`indexer.rs:178` y `:242`:
`existentes` del índice contra `vistas` recién calculadas). El día que
`ruta_relativa` devuelva `/`, las filas viejas con `\` no casan: cada nota se ve
nueva **y** cada fila vieja se ve borrada → reindex completo con re-embedding.

Migración:

```sql
UPDATE notas SET ruta = replace(ruta, '\', '/') WHERE ruta LIKE '%\%';
```

Idempotente (la segunda corrida afecta 0 filas) y sin colisión posible con
`UNIQUE(ruta)`: una fila tiene un solo separador, no dos variantes de sí misma.

**Dónde corre:** en el camino de **escritura** (`indexa`, y por tanto
`exo index` / `exo rebuild`). **No en `abre_db`**: ahí la pagarían también
`search`, `targets` y `recall`, y este repo tiene la regla contraria — los
comandos de solo lectura no escriben (precedente y test:
`engine/tests/targets_cli.rs`, `una_db_inexistente_falla_y_no_se_crea`).

**La ventana:** un índice sin migrar sigue sirviendo `\` hasta el siguiente
`exo index`. Para que esa ventana no sea silenciosa, `exo doctor` gana un check
falsable: si `notas` tiene alguna fila con `\`, lo dice y nombra el remedio
(`exo index`). Un estado transitorio que no se puede observar es la ley 5
(«ausencia ≠ evidencia»).

### D1 — 4ª columna en la salida humana de `search`, con ruta absoluta

```rust
println!("{}\t{}\t{:.4}\t{}", r.permalink, r.tipo, r.score, ruta);
```

**Absoluta**, no relativa: el cwd del agente es el repo en el que trabaja, no la
KB, así que una ruta relativa no se la puede pasar a `Edit` sin resolver antes la
raíz — que solo sale de `exo config --json | jq`. Es decir, la columna relativa
no mataría el jq, lo desplazaría un comando. El repo ya tomó esta decisión para
`write` (`engine/src/main.rs:793-795`: «una línea humana con la ruta absoluta —
lo que `/documenta` necesita para su commit scoped») y `recall` hace lo mismo.

La raíz se resuelve como en `targets_cmd` (`main.rs:935`, `resuelve_kb`), y
`search` gana un flag `--kb` por simetría. Si la KB no resuelve en modo humano:
`bail!` con el remedio nombrado (`pasa --kb, o corre exo init`), **nunca** una
ruta relativa «a veces» — eso sería forma válida con semántica variable.

`--json` **no se toca**: sigue emitiendo `path` relativa (ya normalizada por D0).
La asimetría relativa(JSON)/absoluta(humana) es deliberada y queda declarada
aquí: es *visible* —nadie confunde `C:/…/x.md` con `x.md`— a diferencia de una
divergencia de separador, que es silenciosa. Unificar la semántica de `path`
entre comandos va al backlog.

**Cambio de comportamiento sobre v0.1.0 publicada:** `exo search` en modo humano
pasa a exigir una KB resoluble. Los tests que invocan `search` sin config
(`engine/tests/precedencia.rs:51-303`) usan `--json` y no se ven afectados.

### D2 — ruta ausente: un token, y el aviso a stderr

`path` es `Option<String>`: `None` cuando el permalink está en el índice pero no
en `notas`. **No es «índice rancio»**: el indexer borra `notas`, `notas_fts` y
`vectores` juntas por permalink (`indexer.rs:242-250`), así que ese estado es un
índice **inconsistente**. El remedio que nombra el mensaje es `exo rebuild`, no
`exo index`.

- Columna: el literal `(sin-ruta:rebuild)` — **un token, sin espacios ni tabs**,
  para que `awk '{print $4}'` (sin `-F`, el hábito más común) no imprima `(sin`.
- stderr: `aviso: N de M resultados sin ruta (índice inconsistente): exo rebuild`,
  por el canal que ya usan los avisos de cobertura vector (`main.rs:905-911`).

Ningún valor in-band es fiable: en Unix **cualquier** cadena es un nombre de
fichero legal, así que un marcador siempre es indistinguible de una ruta rara.
La señal de verdad es el stderr; el marcador solo evita que la columna mienta
por omisión.

**El aviso NO entra en `resultado.avisos` / `warnings`.** Dos motivos: metería
una clave nueva en el JSON que D1 promete no tocar, y contradice la definición
del campo (`buscador.rs:41`: «degradaciones que el consumidor NO puede inferir
de `results`») — un `path: null` sí se infiere.

### D3 — la prosa deja de llevar jq y deja de mentir

| Fichero | Cambio |
|---|---|
| `plugins/exo/agents/executor.md:13` | `ruta`→`path`; comando sin jq; quitar `--db ~/.exo/index.db` (la config ya lo resuelve, y obliga a un `exo config` que la prosa no menciona) |
| `plugins/exo/skills/document/SKILL.md:20-24` | ídem; el párrafo del «permalink NO es invertible» se mantiene entero — es el porqué de la 4ª columna |
| `docs/arquitectura.md` §3.5 (~L192) y tabla L289 | añadir la forma de las dos salidas |

Camino documentado del agente, sin jq:

```bash
exo search --type hybrid --limit 5 "<query>"
# permalink \t type \t score \t ruta absoluta
```

Donde la prosa siga citando `--json` (scripts, no agentes), la receta literal:

```bash
jq -r '.data.results[] | "\(.score)  \(.permalink)  \(.path)"'
```

Y se nombra la alternativa que ya existía: `exo recall --query "<topic>" --limit 5`
da ruta absoluta y snippet sin jq (es `busca_hybrid` por debajo). `search` se
queda como el probe cuando hace falta el **score** o no se quiere el cap de
bytes de `recall`. Que haya dos caminos humanos al mismo dato se dice
explícitamente, para que el próximo incidente no invente un tercero.

### D4 — gates

**`engine/tests/buscador_cli.rs`** (nuevo; CI, 3 SO; helper `kb_con_indice()`):

- `la_salida_humana_lleva_cuatro_columnas_y_la_cuarta_es_la_ruta` — split por `\t`,
  `len == 4`, la 4ª es absoluta y **existe en disco**.
- `la_ruta_humana_no_lleva_barra_invertida`.
- `sin_ruta_la_columna_lo_dice_y_stderr_avisa` — índice inconsistente a propósito;
  el marcador **y** el aviso en el mismo test, porque media señal es la que no grita.
- `el_marcador_no_lleva_whitespace`.
- `sin_kb_resoluble_el_modo_humano_falla_con_remedio` — `bail!`, no ruta relativa.
- `el_json_no_cambia` — **ejerciendo el binario** con `--json`, no serializando el
  struct: `contrato_envelope.rs:92-118` ya hace lo segundo y no habría atrapado
  nada de este incidente.

**`engine/tests/indexer.rs`** (suite existente):

- `ruta_relativa_normaliza_el_separador`.
- `la_migracion_es_idempotente` — segunda corrida, 0 filas afectadas.
- `tras_migrar_el_incremental_no_ve_nada_nuevo_ni_borrado` — el test que protege
  contra el re-embedding masivo.

**`engine/tests/recall.rs` y `engine/tests/escritor.rs`** (suites existentes),
para D0b:

- `la_ruta_absoluta_de_recall_no_lleva_barra_invertida` — sobre la salida del
  binario, humana **y** `--json`. Es el test que habría cazado el bug que lleva
  meses en el bloque de cada prompt.
- `el_absolute_path_de_write_no_lleva_barra_invertida` — `write new` y
  `write append`, que llegan a `ruta_abs` por caminos distintos
  (`escritor.rs:281` compone, `main.rs:760` lo resuelve por índice).

Los dos deben aseverar **ausencia de `\` en toda la cadena**, no solo que
empiece bien: el defecto original era un separador interior, y un test que mire
solo el prefijo lo bendice.

**`engine/tests/doctor*.rs`**: el check de rutas nativas, en rojo y en verde.

**`plugins/exo/scripts/test-contrato-engine.sh`**: extendido con los predicados de
`search --json` (`.data.results[]` con `permalink`/`path` string no vacío) contra
el binario real, con la misma abstención exit≠0 que ya usa para `recall`.
**CORREGIDO el 2026-09-15**: cuando se escribió esta spec, el gate no corría en
CI y la decisión era declararlo en la cabecera. Entre medias `origin/main` avanzó
162 commits y lo cableó: `.github/workflows/ci.yml:176` lanza
`scripts/test-contrato-ci.sh`, un wrapper que monta su propio fixture (KB semilla
de `exo init` + índice, con `EXO_CONFIG` aislado). La cabecera que este trabajo
iba a escribir habría sido **falsa**, así que se conserva la de `main`.

Consecuencia que sí añade este trabajo: como el gate corre contra una KB
**semilla**, y no contra una KB poblada, los predicados de `search` llevan un
guard de vacuidad. Sin él, `.data.results[0]` sobre una lista vacía opera contra
`null` y el gate pasaría —o fallaría— por vacuidad, sin haber ejercido nada.

**`kb_con_indice()` NO se mueve a `engine/tests/common/mod.rs`** (revisado al
planificar, 2026-09-11). Las tres «copias» no son copias: `objetivos.rs:17` monta
alpha + beta + gamma + `informe.pdf`, mientras `targets_cli.rs:16` y
`contrato_envelope.rs:124` montan solo alpha. Unificarlas no es deduplicar, es
cambiar el fixture de suites que dependen de él — riesgo real a cambio de estética.
`buscador_cli.rs` lleva su helper local, como el resto. Ver backlog.

## 4. Fuera de alcance (backlog, no este trabajo)

- **Gate de prosa** — un script que extraiga de `plugins/exo/**/*.md` cada
  expresión jq y cada línea `exo …` en bloque de código, valide las flags contra
  `--help` y evalúe cada ruta jq contra un envelope real exigiendo no-`null`.
  Es lo único que habría cazado los DOS defectos del incidente (`.data[]` y
  `ruta`), y es pieza con diseño propio.
- `walker.rs:131` normaliza `\`→`/` incondicionalmente: en Unix `\` es legal en un
  nombre de fichero. Un ítem que cubre walker **e** indexer (ver D0).
- `exo targets` no da `path` en ningún modo (`objetivos.rs:98-107`).
- Unificar el fixture `kb_con_indice()` de `objetivos.rs` / `targets_cli.rs` /
  `contrato_envelope.rs` en `tests/common/mod.rs`: hoy montan KBs distintas
  (una con beta/gamma/pdf, dos con solo alpha), así que es un trabajo de
  unificación de fixtures, no un `git mv`.
- Semántica de `path` desigual entre comandos bajo la misma clave: relativa en
  `search --json`, absoluta en `recall --json`.
- La spec sellada §4.1 (`docs/superpowers/specs/2026-07-17-indexer-design.md`)
  **no lista `path`**: el contrato sellado ya derivó del binario, y la verdad vive
  hoy en el struct y en `contrato_envelope.rs`.
- `recall-inject.sh`: la dedup título↔stem y la raíz común se arreglan solas al
  normalizar, pero no tienen test propio.

## 5. Evidencia

Todo lo medido el 2026-09-11 contra el binario v0.1.0 instalado y la KB real:

```
$ exo search --type hybrid --json --limit 2 "fallo silencioso" | jq -c '.data | keys'
["elapsed_s","query","results","search_type"]

$ exo search --type hybrid --limit 3 "fallo silencioso"
wisdom-paul/backlog/backlog-memoria-v2	entity	0.6000
wisdom-paul/log/doctrina-agentes-bitacora	entity	0.5849

$ exo recall --query "fallo silencioso" --limit 3
- C:/proyectos/homework/wisdom-paul\backlog\Backlog — Memoria v2.md — Backlog — Memoria v2

$ exo budget --json | jq -c '.data.offenders[0]'
{"budget":16000,"path":"projects/lighthouses-bot.md","size_bytes":15904,"tier":"stable"}
```
