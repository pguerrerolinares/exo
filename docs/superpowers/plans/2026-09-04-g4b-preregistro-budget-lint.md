# Pre-registro del gate de paridad de `budget` y `lint` (G4b)

> **Esto sí es un pre-registro.** Se escribe el 2026-09-04, y el hecho
> verificable que lo respalda es de `git`, no de prosa: es el segundo commit de
> la rama `g4b-budget-lint`, y el primero (`60deca0`, `plan: G4b — exo budget
> y exo lint, con lo que la review de fable corrigio`) es un commit de plan
> que no toca `engine/src/`. En esta rama no existe todavía ni una sola línea
> de Rust de `presupuesto.rs`, `lint.rs` ni `gate.rs`: nadie ha visto el
> output del binario contra el que se va a comparar, porque ese binario no
> está compilado.
>
> En G4a este mismo documento se escribió al final, con el binario ya
> compilado y corrido, y tuvo que titularse *"Registro del gate de paridad —
> **NO es un pre-registro**"*. Un pre-registro firmado a posteriori es un
> check no falsable: se puede redactar el criterio para que dé verde sabiendo
> ya lo que da. Este se escribe antes, que es lo único que lo hace valer.
>
> Lo que sigue sin observarse, y es la mitad que importa: **el lado Go no se
> ha corrido ni una vez.** No hay toolchain Go en esta máquina —
> `go: command not found`, medido el 2026-09-04 en W11 (`which go` y
> `go version` fallan los dos)—, así que nadie ha visto el output de
> `kbx budget` ni de `kbx doctor` con los que se va a comparar. El criterio de
> abajo se fija ahora, con las dos mitades todavía a ciegas.

## Referencia

- **kbx**: `origin/main`, commit `fe46443`, compilado fresco en la máquina
  Linux antes de la corrida. Es la fuente del port declarada en
  `2026-09-04-g4b-budget-y-lint.md` (§Global Constraints): el checkout local
  de kbx en W11 diverge de `fe46443` y no sirve como referencia, igual que en
  G4a.
- **exo**: el binario de la rama `g4b-budget-lint`, `cargo build --release`,
  **en el estado en que esté cuando se ejecute el gate** — al escribir esto no
  existe, porque las Tasks 2–11 (el código) no se han empezado.
- **Índice**: una copia de solo lectura de `~/.exo/index.db`, igual que en
  G4a. Ningún índice vivo se toca. `exo budget` no la usa (no tiene `--db`:
  no toca el índice, solo el árbol de ficheros); `exo lint` y `kbx doctor` sí.
- **KB**: `wisdom-paul`, el árbol real. En esta máquina, la ruta absoluta es
  `C:/proyectos/homework/wisdom-paul` (vía Git Bash, `/c/proyectos/homework/wisdom-paul`).

## Qué se compara y qué no

Dos comparaciones independientes, no una:

1. `exo budget --json` contra `kbx budget --json`.
2. `exo lint --json` contra `kbx doctor --json` — no contra un `kbx lint` que
   no existe (Corrección 2 del plan completo: los verbos de `fe46443` son
   `doctor, targets, history, diff-since, budget, stale, rotate, ratchet`, y
   los seis checks de doctor se invocan en su modo bare).

A diferencia de `targets` en G4a, aquí **sí** hay comparación byte-a-campo
posible: los dos verbos son deterministas sobre el mismo árbol de ficheros y
el mismo índice, sin `ORDER BY rank` de por medio ni bindings de SQLite que
puedan desempatar distinto. No hay conjunto-vs-secuencia que declarar aquí.

## Divergencias declaradas antes de correr nada

Ninguna sale de comparar outputs — no hay outputs todavía. Todas salen de leer
el plan completo (`2026-09-04-g4b-budget-y-lint.md`, §Adjudicaciones y
§Correcciones) y, en los tres casos donde el plan cita una medición sobre la
KB, de repetir esa medición hoy. Se declaran aquí para que, si aparecen en el
diff, no se racionalicen a posteriori como "diferencias aceptables" — ese es
exactamente el vicio que este documento existe para cerrar.

1. **A1 — `tier` filtra whitespace ASCII, no Unicode.** El port decide alinear
   a ASCII (`stripWhitespace` de kbx borra solo los seis caracteres ASCII;
   `tier()` de exo hacía lo mismo pero con `char::is_whitespace()`, la
   propiedad Unicode). Esto **elimina** la divergencia 3 del registro de G4a
   (que declaraba `tier: co<NBSP>re` como el único punto de desacuerdo real
   entre los dos lados): con el fix de A1 los dos binarios devuelven
   `"co\u{a0}re"` sin filtrar, así que si esa nota existe en la KB caerá en
   `notier` en los dos lados por igual, en vez de divergir.
2. **A4 — la exclusión de `budget` compara el primer segmento, no el basename
   en cada nivel.** `kbx budget` poda por basename en cada nivel del walk
   (`filepath.SkipDir`); el port compara solo el primer segmento de la ruta
   relativa (la semántica de `doctor`, elegida porque degrada hacia rojo: un
   `projects/archive/` futuro lo *chequea* en vez de saltarlo en silencio).
   **Medido el 2026-09-04** en la KB real
   (`find "$KB" -type d \( -iname archive -o -iname docs -o -iname .superpowers \)`):
   existen exactamente `archive/` y `docs/` en la raíz, **ninguno anidado**.
   Las dos semánticas dan hoy el mismo resultado; el impacto es cero.
3. **A5 — `.md` es case-insensitive en los dos verbos.** kbx es incoherente
   internamente (`budget` usa `EqualFold`, case-insensitive; `doctor` usa
   `HasSuffix`, case-sensitive); el port unifica a case-insensitive en los
   dos. **Medido el 2026-09-04**: `find -iname "*.md"` y `find -name "*.md"`
   sobre la KB devuelven el mismo recuento, **172/172**. No hay ningún `.MD`
   ni mezcla de mayúsculas en la KB real hoy, así que el impacto es cero.
4. **A7 — 6 tipos de finding en `lint` frente a 7 en `doctor`.**
   `schema_drift` existía porque kbx y exo eran dos binarios contra un schema
   compartido; con un solo binario deja de tener objeto y no se porta. Es la
   divergencia estructural principal del gate de `lint`, y por eso el
   criterio de abajo excluye `schema_drift` **del lado Go** antes de comparar.
5. **El exit code.** kbx: `1` = hay hallazgos, `2` = error. exo: `1` = error
   genérico, `2` = error de parseo de clap, `3` = gate rechazado (hallazgos).
   Es una divergencia deliberada, declarada en el plan completo
   (§Global Constraints) como la primera vez que exo ejerce el `3`. No afecta
   a la comparación de `data`, solo al exit code del proceso.
6. **Dotdirs y dotfiles.** El walk de exo (`walk_kb_excluyendo`) salta **todo**
   directorio y fichero que empiece por `.`, a cualquier nivel. `kbx budget`
   solo poda los tres basenames de su lista de exclusión
   (`.superpowers`, `archive`, `docs`) y por tanto **sí** recorre `.git/`,
   `.claude/` y `.omc/` — contaría un `.oculto.md` si existiera dentro.
   **Medido el 2026-09-04** en la KB real: el único dotdir presente es
   `.git/` (`find "$KB" -mindepth 1 -iname ".*" -type d`), y no contiene
   ningún `.md` (`find "$KB" -type f -iname ".*.md"` no devuelve nada). El
   impacto es cero hoy. **Se declara de todos modos porque el criterio dice
   que toda diferencia no declarada es un fallo, y esta la introduce el port
   a propósito** — no es un descuido que se vaya a corregir si aparece.
7. **Estrictez UTF-8.** El port lee las notas con `std::fs::read` +
   `String::from_utf8_lossy` (el idioma de `objetivos.rs`, no
   `read_to_string`), igual que Go, que trabaja sobre bytes. Se declara
   porque una versión anterior de este plan usaba `read_to_string`, que habría
   abortado el verbo entero con exit 1 ante un solo byte inválido en una nota
   — un gate que se apaga por una nota mal codificada en vez de clasificarla.
   Con `from_utf8_lossy` los dos lados degradan igual (sustitución de bytes
   inválidos), no fallan.
8. **`index_stale` — el punto que hay que leer despacio.** `exo lint` emitirá
   un tipo de hallazgo que `kbx doctor` no tiene (nace en la Task 9 del plan
   completo). Contado en bruto, esto deja a exo con **7** tipos de finding
   frente a los **7** de kbx — la misma cifra que antes de A7, que había
   bajado exo a 6. **Pero no son los mismos siete.** Tras A7, exo no tiene
   `schema_drift` (que kbx sí tiene) y kbx no tiene `index_stale` (que exo sí
   tiene). Son dos conjuntos de 7 con **seis en común y uno propio cada uno**,
   no el mismo conjunto de 7. Un lector que solo mire el recuento — "siete y
   siete, luego coinciden" — se equivoca: el gate compara excluyendo
   `schema_drift` del lado Go y `index_stale` del lado Rust, es decir,
   comparando los **seis comunes**, no los siete de ningún lado.
9. **El orden de `no_air`.** kbx ordena por `size_bytes` descendente y
   desempata por ruta; el port reproduce el mismo orden. Se declara por si el
   desempate difiere en la práctica (por ejemplo, ante colación de rutas con
   acentos o mayúsculas), aunque el criterio de abajo compara `no_air` como
   conjunto por `path`, no como secuencia, así que un desorden puro no
   gatearía — solo se declara para que una diferencia de **orden observada**
   no se investigue como si fuera una diferencia de **contenido**.

## El corpus

La KB real completa (`wisdom-paul`), no una muestra: `budget` y `lint`
recorren el árbol entero por construcción, así que no hay "topics" que elegir
como en el pre-registro de `targets` — el corpus **es** el árbol.

## Criterio

Fijado ahora, verbatim, para que no se reescriba después de ver un output:

- `budget` **PASA** si `tiers[]` es idéntico campo a campo, y si los conjuntos
  de `offenders`, `waived`, `no_air` y `notier` son idénticos por `path`, y
  para cada `path` coinciden `tier`, `size_bytes` y `budget`.
- **`no_air` gatea como los demás.** Una versión anterior de este criterio lo
  eximía "por A3", y estaba mal razonado: la referencia declarada es `fe46443`
  compilado fresco, que tiene **la misma** fórmula que el port (Corrección 1
  y A3 del plan completo). Contra esa referencia `no_air` tiene que coincidir,
  y eximirlo era debilitar el gate a priori con una justificación que solo
  valdría contra el binario local huérfano de `f0d0564` — el que este mismo
  documento declara que no sirve de referencia. La recalibración de la
  campaña de evicción de la KB (los umbrales 7.391/10.869 frente a
  7.225/10.625) es trabajo aparte y no vive en este gate.
- `lint` **PASA** si el conjunto de `findings` es idéntico por `(type, path)`
  tras excluir los de tipo `schema_drift` **del lado Go** (A7) y los de tipo
  `index_stale` **del lado Rust** (divergencia 8), y si `waived` coincide
  igual. Los `detail` se comparan y una diferencia se investiga, pero no
  gatea por sí sola: son cadenas de presentación, no el hallazgo.
- **Si el lado Rust emite un solo `index_stale`, el gate se aborta y no se
  juzga**: significa que el índice de la copia no corresponde al árbol de la
  KB, y entonces `orphan` está midiendo otra cosa en cada lado. Se re-indexa
  y se repite. Es la condición que hace no comparable la comparación.
- Cualquier diferencia **no** cubierta por una de las nueve divergencias
  declaradas arriba es un **FALLO** del port. La resolución no es
  reclasificarla como divergencia aceptable a posteriori.
- El gate global pasa si pasan las dos comparaciones (`budget` y `lint`).

## Comandos

```bash
mkdir -p /tmp/g4b && cp ~/.exo/index.db /tmp/g4b/index.db
KB=~/…/wisdom-paul

# `exo budget` NO tiene `--db`: no toca el índice, solo el árbol de ficheros.
# `kbx budget` sí lo acepta (lo usa para el fallback de meta.kb_root), pero
# aquí sobra porque `--kb` va explícito en los dos lados.
kbx budget --kb "$KB" --json \
  | jq -S '.data | {tiers, offenders, waived, no_air, notier}' > /tmp/g4b/go-budget.json
./target/release/exo budget --kb "$KB" --json \
  | jq -S '.data | {tiers, offenders, waived, no_air, notier}' > /tmp/g4b/rs-budget.json
diff -u /tmp/g4b/go-budget.json /tmp/g4b/rs-budget.json \
  && echo "PASA: budget" || echo "REVISAR: budget"

kbx doctor --db /tmp/g4b/index.db --kb "$KB" --json \
  | jq -S '.data | {findings: [.findings[] | select(.type != "schema_drift")], waived}' \
  > /tmp/g4b/go-lint.json
./target/release/exo lint --db /tmp/g4b/index.db --kb "$KB" --json \
  | jq -S '.data | {findings: [.findings[] | select(.type != "index_stale")], waived}' \
  > /tmp/g4b/rs-lint.json
diff -u /tmp/g4b/go-lint.json /tmp/g4b/rs-lint.json \
  && echo "PASA: lint" || echo "REVISAR: lint"

# Medición aparte, que NO es el gate: el censo de no_air con la fórmula
# canónica frente al que produjo el binario local de f0d0564 el 09-02. Es el
# número que recalibra la campaña de evicción de la KB (A3).
jq -S '.data.no_air | length' /tmp/g4b/rs-budget.json
```

## Lo que ya está medido

**Vacío**, y se dice: no hay binario `exo budget` ni `exo lint` en esta rama
todavía (las Tasks 2–11 del plan completo no se han empezado), no hay
`kbx` de `fe46443` compilado, y por tanto ninguno de los dos comandos de
arriba se ha ejecutado ni una vez. Las mediciones citadas en la sección de
divergencias (172/172 `.md` en minúscula, cero `archive`/`docs` anidados, el
único dotdir es `.git` y sin `.md` dentro, `go: command not found`) son
mediciones **del terreno** — del estado de la KB y de esta máquina — hechas
hoy para poder declarar impacto nulo con evidencia en vez de por intuición.
No son mediciones del gate: no comparan ningún output de `budget` ni de
`lint`, porque esos outputs no existen.
