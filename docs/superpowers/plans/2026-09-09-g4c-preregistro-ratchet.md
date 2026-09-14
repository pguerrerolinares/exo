# Pre-registro del gate de paridad de `ratchet` (G4c)

> **Esto sí es un pre-registro.** Se escribe el 2026-09-09, antes de que exista
> una sola línea de `engine/src/trinquete.rs`. El hecho verificable que lo
> respalda es de `git`, no de prosa: en la rama `g4c-ratchet-cutover` este
> documento y el plan son los únicos commits, y ninguno toca `engine/src/`.
> Nadie ha visto todavía el output del binario contra el que se va a comparar,
> porque ese binario no está compilado.
>
> Es el mismo contrato que G4b y por el mismo motivo: en G4a el documento se
> escribió al final, con el binario ya corrido, y hubo que titularlo *"Registro
> del gate de paridad — **NO es un pre-registro**"*. Un criterio firmado a
> posteriori es un check no falsable — se redacta para que dé verde sabiendo ya
> lo que da.
>
> Y como en G4b, **el lado Go sigue sin correrse ni una vez**: no hay toolchain
> Go en esta máquina (W11; `go version` y `which go` fallan los dos, medido el
> 2026-09-09). Nadie ha visto el output de `kbx ratchet`. El criterio de abajo
> se fija con las dos mitades a ciegas.

## Referencia

- **kbx**: `fe46443`, compilado fresco en la máquina Linux antes de la corrida.
  Es la fuente del port declarada en el plan (§Global Constraints). El checkout
  local de kbx en W11 está en `ee2b27c`/`f0d0564`, que **no tiene** la guarda de
  aire (`6332c85`) ni el fix del sello huérfano (`0ae126d`): no sirve como
  referencia, igual que en G4a y G4b.
- **exo**: el binario de la rama `g4c-ratchet-cutover`, `cargo build --release`,
  en el estado en que esté cuando se ejecute el gate. Al escribir esto no
  existe.
- **KB**: `wisdom-paul`, el árbol real, en una **copia de trabajo desechable**.
  Esto es distinto de G4a/G4b y no es cosmético: `ratchet` lee HEAD y el índice
  de git, y `--seal` **escribe** `.kbx-ratchet.json`. La corrida del gate se
  hace sobre `git clone` de la KB a un directorio temporal, nunca sobre el árbol
  vivo.
- **Sello de partida**: el `.kbx-ratchet.json` real de la KB al 2026-09-09, con
  sus **11 entradas** (`ceilings`), medido hoy. Es el fixture de campo: la
  corrida de paridad que importa es la que se hace contra los sellos reales, no
  contra un fixture sintético.

## Qué se compara

Una sola comparación, en tres invocaciones:

1. `exo ratchet --kb <copia>` contra `kbx ratchet --kb <copia>` (modo working tree).
2. `exo ratchet --kb <copia> --staged` contra `kbx ratchet --kb <copia> --staged`,
   con un cambio preparado en el índice.
3. `exo ratchet --kb <copia> --json` contra `kbx ratchet --kb <copia> --json`.

El criterio se ejerce sobre **(3)**, el JSON, porque es el único de los tres
cuya forma está definida campo a campo. Los modos texto se comparan como
señal de humo, no como criterio.

## Criterio, fijado antes de correr nada

**PASA** si, sobre la misma copia de la KB y el mismo sello de partida, los dos
binarios coinciden en:

- **`applied`** (el booleano de abstención) — idéntico.
- **El conjunto de `findings`**, comparado como **conjunto de tripletas
  `(path, kind, limit)`** — mismo cardinal y mismo contenido. Se compara como
  conjunto y no como secuencia porque el orden es parte del contrato de kbx
  (`sort.SliceStable` por path) y del port, y una divergencia de orden con el
  mismo contenido es un fallo de ordenación, no de semántica: se anota aparte.
- **El veredicto de ruptura**: que `Failed()` de kbx y `fallido()` de exo
  coincidan. Esto es lo que gatea de verdad, y es la única línea cuyo
  desacuerdo es fatal por sí solo.

**Los campos `was` y `now` se comparan pero su desacuerdo NO tumba el gate por
sí mismo**: se anota como divergencia y se adjudica. Son cifras de mensaje.
`limit` sí entra en el criterio duro porque es la cifra accionable ("poda a N")
y un desacuerdo ahí significa aritmética distinta.

**NO PASA** si difieren `applied`, el conjunto de `(path, kind, limit)`, o el
veredicto de ruptura. Un NO PASA no se racionaliza: o se arregla el port, o la
divergencia se declara aquí abajo **antes** de volver a correr.

## Divergencias declaradas antes de correr nada

Ninguna sale de comparar outputs — no hay outputs. Todas salen de leer el
código Go de `fe46443` y el plan. Se declaran ahora para que, si aparecen en el
diff, no se racionalicen después como "aceptables".

1. **Exit codes: divergencia deliberada.** kbx usa `0` limpio / `1` gate roto /
   `2` error de uso o IO. exo usa `0` limpio / `3` gate rechazado / `1` error /
   `2` error de parseo de clap. Es el contrato de exo desde G4b y **no se
   alinea**. El gate compara el veredicto, no el número.

2. **`--seal` y `--staged` juntos.** kbx los rechaza como error de uso (exit 2).
   exo los rechaza en clap (`conflicts_with`), que sale con exit 2 también, pero
   por otra vía y con otro texto. Se declara: mismo efecto, distinto mensaje.

3. **Orden de iteración: exo es determinista por construcción, kbx no.** El
   emparejamiento de renames de kbx recorre un `map` de Go, cuyo orden no está
   garantizado; hay un test (`TestRenamePairingIsDeterministic`) que lo corre 20
   veces justamente para detectar el problema. El port usa `BTreeMap`, donde el
   orden es una propiedad del tipo. **Esto sube el listón, no lo baja**: si los
   dos binarios divergen en un caso de emparejamiento ambiguo, el sospechoso es
   kbx, y el remedio es fijar el fixture, no relajar exo.

4. **`git diff --cached` y la config `diff.renames` del entorno.** kbx invoca
   `git diff --cached --name-only --diff-filter=ACMR` **sin** `--no-renames`
   (verificado en `fe46443`, `internal/ratchet/staged.go`), así que qué ficheros
   aparecen bajo un rename depende de la config de la máquina. El plan de G4a
   listaba "`--no-renames` explícito" como trampa a heredar; **el código Go no
   lo hace**. Adjudicado en el plan (A3): el port **sí** pasa `--no-renames`.
   ⇒ Bajo un rename staged con `diff.renames=true` en la máquina del gate, los
   dos binarios **pueden** ver conjuntos de ficheros distintos. Si eso ocurre,
   es una divergencia **esperada y declarada aquí**, no un fallo del port. Para
   que el gate no dependa del entorno, la corrida se hace con
   `-c diff.renames=false` en las dos mitades, y aparte se anota el resultado
   con la config por defecto.

5. **Unicode en las claves del sello.** El sello real tiene claves con `—`
   (guion largo) y acentos. Go serializa con `json.Marshal` de un string, que
   escapa a `\uXXXX` solo lo que debe; `serde_json` escapa distinto por defecto.
   El fichero **escrito** por `--seal` puede diferir byte a byte del de kbx sin
   que difiera el JSON *parseado*. El criterio compara el sello **parseado**, no
   sus bytes. Se declara porque un diff de bytes en `.kbx-ratchet.json` es un
   falso positivo previsible.

6. **`no-air-debt` sobre los 11 sellos reales.** Los sellos de la KB de hoy
   arrastran deuda: hay notas selladas sin el 15% de aire. En reposo eso emite
   `no-air-debt`, que **no rompe** por diseño. Se espera que las dos mitades
   emitan la misma lista de deuda; si exo emitiera una sola entrada de más o de
   menos, el port está mal aunque el gate siga en verde. Por eso la deuda entra
   en el conjunto comparado y no solo el veredicto.

## Lo que este gate NO cubre

- **No se corre en W11.** Sin Go, la mitad de referencia no existe aquí. Este
  documento queda escrito y el gate **pendiente de la máquina Linux**, igual que
  el de `targets` de G4a. Se ejecutan juntos: los dos exigen exactamente el
  mismo prerequisito (compilar kbx `fe46443`), y correrlos en la misma sesión es
  un solo montaje de entorno.
- **No cubre `--seal` contra kbx.** Sellar escribe, y dos binarios sellando
  sobre la misma copia se pisan. `--seal` se verifica por sus propios tests
  (atomicidad: si algo no tiene aire, no se escribe nada) y por inspección del
  fichero resultante, no por paridad.
- **No cubre `rotate`, `stale`, `history` ni `diff-since`**, que siguen sin
  portar. El cutover que hace este plan es parcial por eso, y el plan lo declara.

## Registro de la corrida

> A rellenar **en la máquina Linux**, cuando se ejecute. Hasta entonces, vacío
> a propósito: un registro con resultados escritos antes de correr es
> exactamente lo que este documento existe para impedir.

- Fecha: 2026-09-14
- Commit de exo: `4d047f4` (rama `d-gates`)
- Commit de kbx: `fe46443`
- KB: copia de trabajo desechable clonada de `/tmp/campana-d/kb-base` (a su
  vez clon de `wisdom-paul`, `main`@`4bf1dc4`) a
  `/tmp/campana-d/ratchet-kb.N0ib`, con `diff.renames=false` fijado como
  config del repo (Adjudicación A3). Sello de partida `.kbx-ratchet.json`
  con 11 `ceilings`, confirmado antes de correr nada — coincide con el
  fixture de campo declarado arriba (§Referencia).
- Invocación 1 (working tree): `kbx ratchet --kb <copia>` exit=0,
  `exo ratchet --kb <copia>` exit=0. Salida de texto **idéntica** byte a
  byte (`diff -u` limpio), 8 líneas cada una, todas `no-air-debt` (deuda,
  no rompe).
- Invocación 2 (`--staged`, con un cambio preparado — una línea añadida a
  `log/accesibilidad-saga-bitacora.md` y `git add -A`): `kbx` exit=0, `exo`
  exit=0. Salida de texto **idéntica** byte a byte, 8 líneas, mismo
  contenido que la invocación 1 (el cambio de prueba no altera el juicio de
  `no-air-debt`). Cambio de prueba deshecho con `git reset --hard` antes de
  la invocación 3, working tree confirmado limpio después.
- Invocación 3 (`--json`, la que gatea el criterio): `kbx` exit=0, `exo`
  exit=0. `jq -S '.data | {applied, findings: (.findings | sort_by(.path,
  .kind, .limit))}'` sobre cada salida: 53 líneas cada fichero
  (`go-ratchet.json` / `rs-ratchet.json`), `diff -u` limpio →
  **PASA: ratchet --json**. `applied`: `true` en las dos mitades.
  `findings`: 8 en cada una, mismo conjunto `(path, kind, limit)`, mismo
  `now` en cada tripleta (campo no gateante, coincide igual). Todos
  `kind: "no-air-debt"` — sin sorpresas de otros `kind` no cubiertos por el
  fixture actual.
- `applied` coincide: **sí** (`true` / `true`).
- Conjunto `(path, kind, limit)` coincide: **sí** — 8/8, mismo cardinal,
  mismo contenido, verificado por `diff -u` sobre el JSON ordenado
  (`sort_by`), no solo por conteo.
- Veredicto de ruptura coincide: **sí** — exit `0` en las tres invocaciones
  para las dos mitades (ninguna rompe; los 8 findings son `no-air-debt`,
  que por diseño no bloquea). No se observó exit `1` (kbx: gate roto) ni
  `3` (exo: gate rechazado) en ninguna de las dos mitades.
- Divergencias observadas y su adjudicación:
  - `schema_version` difiere (`1` en kbx, `2` en exo) — **fuera del alcance
    del criterio**: el envelope completo no se compara, solo `.data`
    filtrado por el propio script del pre-registro (§Qué se compara,
    invocación 3); consistente con Global Constraints del plan
    ("`SCHEMA_VERSION` sigue en 2, sin tocar"). No es una de las 6
    divergencias declaradas porque no hace falta declararla: no entra en
    lo comparado.
  - Ninguna de las 6 divergencias pre-declaradas (exit codes, `--seal`+
    `--staged`, orden de iteración, `diff.renames`, Unicode en claves del
    sello, `no-air-debt`) se manifestó como discrepancia real en esta
    corrida — el fixture de campo (8 `no-air-debt`, 0 renames staged, sin
    `--seal`) no ejerció los casos límite que esas divergencias anticipan;
    quedan como riesgo latente no cubierto por este fixture, no como fallo.
  - No se observó ninguna otra divergencia: `was`/`now` coinciden en todos
    los `findings`, orden de `findings` coincide tras el `sort_by` (no se
    puede aislar si habría divergido sin ordenar, pero el criterio exige
    conjunto, no secuencia).
- **PASA / NO PASA**: **PASA** — las tres invocaciones cumplen el criterio
  fijado (`applied` idéntico, conjunto `(path, kind, limit)` idéntico,
  veredicto de ruptura idéntico); ninguna divergencia observada cae fuera
  de lo ya declarado o del alcance del criterio.
