# Registro del gate de paridad de `targets` (G4a) — **NO es un pre-registro**

> **Lo primero, porque cambia cuánto vale este documento.** El plan de G4a
> pedía escribir esto *antes de compilar el binario Rust y antes de ver ningún
> output suyo*. **No se cumplió**: cuando se redacta, el binario está
> compilado y se ha corrido contra una copia del índice vivo y la KB real.
> Así que esto es un **registro posterior**, y se llama por su nombre en vez
> de firmarse como pre-registro, que es lo que lo convertiría en un check no
> falsable de manual.
>
> Lo que sí sigue sin observarse, y es la mitad que importa: **el lado Go no
> se ha corrido ni una vez**. No hay toolchain Go en esta máquina
> (`go: command not found`, medido el 2026-09-02), así que nadie ha visto el
> output de `kbx targets` con el que se va a comparar. El criterio de abajo se
> fija ahora, con esa mitad todavía a ciegas.

## Referencia

- **kbx**: `origin/main`, commit `fe46443`, compilado fresco con `make install`
  en la máquina Linux antes de la corrida. Añadir `-ldflags` con el commit al
  Makefile de kbx: hoy `kbx` no tiene flag de versión, así que "el kbx
  instalado" no es evidencia falsable de qué código se compara.
  - **Ojo**: el checkout local de kbx en W11 está en `f0d0564`, que **diverge**
    de `fe46443` (1 commit por delante, 18 por detrás). No sirve como
    referencia. Hay que traer `fe46443` explícitamente.
- **exo**: el binario de la rama `g4a-targets`, `cargo build --release`.
- **Índice**: una copia de solo lectura de `~/.exo/index.db` en `/tmp/g4a/`.
  Ningún índice vivo se toca. Los dos binarios corren contra **el mismo
  fichero**: kbx ya consume el esquema de exo (`notas`, `notas_fts`,
  `aristas`) desde M6-04, así que esto es paridad real, no dos índices
  parecidos.
- **KB**: `wisdom-paul`, el árbol real.

## Qué se compara y qué no

`targets` **no es byte-comparable** y se compara **como conjunto, no como
secuencia**: ordena por rank bm25 **sin tie-break** (`ORDER BY rank`), y dos
bindings distintos de SQLite (mattn/go-sqlite3 frente a rusqlite bundled)
pueden ordenar los empates de forma distinta.

- **No** se comparan: el orden, el score, ni el texto del `snippet`.
- **Sí** se comparan: el conjunto de `permalink`, y por cada permalink presente
  en ambos, los campos `tier`, `size_bytes` y `last_commit` — que salen de
  disco y de git, no del binding de SQLite, y por tanto **tienen que coincidir
  exactamente**.

## Divergencias ya conocidas, declaradas antes de correr nada

Ninguna se descubrió comparando: todas salen de leer los dos códigos. Se
declaran aquí para que, si aparecen en el diff, no se racionalicen a
posteriori como "diferencias aceptables".

1. **Una lectura de disco por candidata, no dos.** Go abre cada fichero dos
   veces (y lo confiesa: *"one extra open on top of the one ExtractHeadings
   already does"*); el port lo unificó. **Impacto en el output: ninguno** —
   las dos lecturas de Go fallan juntas, así que el trío degradado
   (`tier=""`, `size_bytes=0`, `headings=[]`) es idéntico.
2. **Normalización del pathspec de git.** El port hace
   `ruta_rel.replace('\\', "/")` antes de pasárselo a `git log`; Go no. Medido
   el 2026-09-02: Git for Windows acepta el `\` igual, así que sin la
   conversión los tests siguen verdes. Es guarda defensiva, no invariante.
   **Impacto esperado: ninguno.**
3. **`tier` y el whitespace Unicode — la única divergencia con impacto real.**
   El port filtra con `char::is_whitespace()` (propiedad Unicode White_Space:
   NBSP, U+2007, U+3000…); el `stripWhitespace` de Go mapea **solo ASCII**
   (`' '`, `'\t'`, `'\n'`, `'\r'`, `'\v'`, `'\f'`). Con `tier: co<NBSP>re`, Go
   devuelve `"co\u{a0}re"` y Rust `"core"`. **`tier` es uno de los tres campos
   que el criterio exige idénticos**, así que si alguna nota de la KB real
   tiene whitespace no-ASCII en su `tier`, el gate va a marcar diferencia y
   **será real, no ruido**. Decisión pendiente del dueño (ver abajo).
4. **Tope de línea.** Go usa `bufio.Scanner`, con tope de 64 KB por línea y
   `scanner.Err()` ignorado; Rust usa `str::lines()`, sin tope. Con una línea
   de más de 64 KB los dos binarios diferirían en `headings`. Rust es el
   correcto de los dos. No se espera ninguna línea así en la KB.
5. **Exit code de los errores de uso.** `kbx targets --limit 0` sale **2**; el
   port sale **1** (`anyhow` genérico), porque en exo el 2 es de clap y el 3 es
   "gate rechazado". No afecta a la comparación de `data`, pero se declara
   porque el criterio de G4b/G4c sí gateará por exit code.

## Los topics

Los mismos cinco del pre-registro de M6-04, para que las dos medidas sean
comparables entre sí: un término técnico, un nombre propio, una palabra de
dominio, un acrónimo y una consulta multipalabra.

1. `indexer`
2. `reflex`
3. `memoria`
4. `kbx`
5. `recall en el punto de uso`

## Criterio

Por topic, con `--limit 10` en los dos binarios:

- **PASA** si el conjunto de permalinks es **idéntico**, y si para cada
  permalink `tier`, `size_bytes` y `last_commit` coinciden exactamente.
- Una diferencia de conjunto se explica por **empate de rank en la frontera del
  límite** (un permalink en la posición 10 de uno y la 11 del otro). Esa es la
  única explicación admisible; cualquier otra es un FALLO del port.
- Una diferencia en `tier`, `size_bytes` o `last_commit` es **siempre** un
  fallo, **salvo** que sea exactamente el caso 3 de arriba (whitespace no-ASCII
  en el `tier` declarado), que ya está declarado y cuya resolución es una
  decisión de producto, no un bug del port.
- El gate global pasa si **pasan los cinco topics**. Con cuatro o menos, el
  port se investiga antes de seguir con G4b.

## Comandos

```bash
mkdir -p /tmp/g4a && cp ~/.exo/index.db /tmp/g4a/index.db
KB=~/…/wisdom-paul
for t in "indexer" "reflex" "memoria" "kbx" "recall en el punto de uso"; do
  slug=$(echo "$t" | tr ' ' '-')
  kbx targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/go-$slug.json"
  ./target/release/exo targets --db /tmp/g4a/index.db --kb "$KB" --limit 10 --json "$t" \
    | jq -S '.data.candidates | map({permalink, tier, size_bytes, last_commit}) | sort_by(.permalink)' \
    > "/tmp/g4a/rs-$slug.json"
  diff -u "/tmp/g4a/go-$slug.json" "/tmp/g4a/rs-$slug.json" \
    && echo "PASA: $t" || echo "REVISAR: $t"
done
```

El informe va a `docs/superpowers/runbooks/`, con los dos commits comparados,
el diff de cada topic (o su ausencia), el veredicto por topic y el global. Si
algún topic falla, el informe dice **qué** difiere y **por qué**.

## Lo único que ya está medido del lado Rust

Contra una copia del índice vivo (23 MB) y la KB real, el 2026-09-02:

- envelope con `schema_version: 2`, `command: "targets"`, y las seis claves de
  candidato en inglés;
- `size_bytes` = **20348**, idéntico al tamaño del fichero en disco;
- `last_commit` = `2026-08-27T00:03:42+02:00`, salido de git;
- `tier` = `log`, leído del frontmatter en disco;
- salida UTF-8 correcta (el mojibake que apareció al inspeccionarla era del
  `python -m json.tool` decodificando stdin como cp1252 en Windows, no del
  binario).

Eso demuestra que el lado Rust produce lo que dice producir. **No demuestra
paridad**, que es lo que este gate existe para medir y sigue sin correrse.
