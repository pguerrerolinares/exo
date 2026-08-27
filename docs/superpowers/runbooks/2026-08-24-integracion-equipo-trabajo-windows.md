# 2026-08-24 — Integración del entorno exo en el equipo de trabajo (Windows)

> Runbook de una máquina **Windows 11 corporativa** (`C:\proyectos\homework`), sin
> WSL, con Git Bash como único shell POSIX. Es la primera vez que el framework
> corre fuera de Linux, así que la mitad del valor de este documento son las
> trampas de plataforma: lo que se comportó distinto y, sobre todo, **lo que
> mintió**.
>
> Estado final: engine compilado y sirviendo, KB indexada (155 notas / 3.184
> trozos), plugins `process` y `reflex` instalados desde el marketplace remoto,
> `superpowers` desinstalado.

## Mapa de la máquina

| Pieza | Dónde |
|---|---|
| Monorepo `exo` | `C:\proyectos\homework\exo` |
| Marketplace `exo-plugins` | `C:\proyectos\homework\exo-plugins` |
| KB `kb-demo` | `C:\proyectos\homework\kb-demo` |
| `kbx` (Go, **sin compilar**) | `C:\proyectos\homework\kbx` |
| Binario | `~/.local/bin/exo.exe` (en el PATH de Git Bash) |
| Índice | `~/.exo/index.db` |
| Config RO | `~/.basic-memory/config.json` |

## Prerequisitos que hubo que instalar

- **`jq`** (`winget install jqlang.jq`). Winget lo deja como *alias de ejecución*
  en `WindowsApps`, y **bash no resuelve esos alias**: `command -v jq` falla
  dentro de un hook aunque funcione en PowerShell. Sin `jq` **todos** los hooks
  de reflex enmudecen, porque es lo que construye su JSON de salida. Solución:
  copiar el `.exe` real a `~/.local/bin`, que ya está en el PATH de Git Bash.
- **Rust** (`winget install Rustlang.Rustup`) + `rustup default stable`.
- **MSVC** — ver la sección de trampas, que es donde está la historia.

## Config RO de la KB

El engine resuelve la raíz de la KB y la config de embeddings leyendo
`~/.basic-memory/config.json` (`lib.rs:83` y `lib.rs:131`). En una máquina sin
basic-memory hay que crearlo a mano:

```json
{
  "projects": { "kb-demo": { "path": "C:/proyectos/homework/kb-demo" } },
  "default_project": "kb-demo",
  "semantic_embedding_model": "jinaai/jina-embeddings-v2-base-es",
  "semantic_embedding_dimensions": 768,
  "semantic_min_similarity": 0.35
}
```

Barras normales en el `path`: el resto del stack las digiere sin problema.

## Instalación

```bash
claude plugin marketplace add https://github.com/pguerrerolinares/exo-plugins.git
claude plugin install process@exo
claude plugin install reflex@exo
claude plugin uninstall superpowers@claude-plugins-official
```

Compilar e instalar el engine:

```bash
cd /c/proyectos/homework/exo/engine
cargo build --release          # 33 s con las deps ya bajadas
cp target/release/exo.exe ~/.local/bin/exo.exe
mkdir -p ~/.exo
exo index --db ~/.exo/index.db --json   # 6m08s en frío (descarga el modelo)
```

**El binario va como `exo.exe` y en el PATH, no como `~/.local/bin/exo`.** Los
scripts hacen `command -v exo`, que en msys sí resuelve el `.exe`; pero su
fallback literal (`$HOME/.local/bin/exo`, sin extensión) falla el test `-x` y
mandaría todos los hooks al camino "no-engine" en silencio.

## Verificación falsable

```bash
exo index --db ~/.exo/index.db --json
# {"command":"index","data":{"indexadas":155,"trozos_embebidos":3184,...}}

exo search --db ~/.exo/index.db --type vector --limite 3 "memoria persistente"
# 3 filas con score: si esto va, los embeddings van

echo '{"session_id":"t","source":"startup"}' \
  | bash ~/.claude/plugins/cache/exo/reflex/*/scripts/exo-recall.sh \
  | jq -r '.hookSpecificOutput.additionalContext' | grep -c 'Contrato de memoria'
# 1 ⇒ bloque real. 0 ⇒ está sirviendo el fallback embebido.

grep recall-fallback ~/.claude/reflex-log.jsonl | tail -3
# reason=no-engine / no-index / no-contract ⇒ algo está roto y el arranque miente
```

Resultado el 2026-08-24: los cuatro checks en verde. FTS, vector e hybrid
devuelven resultados; `recall-inject` (UserPromptSubmit) y `subagent-inject`
(SubagentStart) emiten JSON válido con contenido real. `exo write new` crea nota
con frontmatter completo y el dup-gate consulta el índice real.

Suite del engine, tras los dos arreglos de portabilidad de abajo:

```bash
cargo test --release --no-fail-fast 2>/dev/null | grep -E '^test result'
echo "CARGO_EXIT=${PIPESTATUS[0]}"   # el de la tubería NO sirve
# 131 passed · 0 failed · 1 ignored (smoke::jina_es_embebe_a_768, ya lo estaba)
```

## Trampas de plataforma

### `setsid` no existe en Git Bash — el índice no se refrescaba nunca

`exo-index.sh` (hook `Stop`) lanzaba el reindexado con `setsid -f nohup … || true`.
En msys `setsid` no existe: la línea fallaba, el `|| true` se la tragaba y el
script salía 0. **Meses de sesiones en Windows sin refrescar el índice, sin un
solo rastro.** Exactamente la degradación silenciosa que `exo-recall.sh` se
esfuerza en evitar con sus eventos de log.

Arreglo en `fix/exo-index-portable`: si hay `setsid`, comportamiento idéntico;
si no, detach vía `cmd //c start //b` lanzando un `bash -c` interior (`start` no
sabe redirigir la salida del proceso que lanza, y los argumentos viajan como ENV
para no pelearse con el quoting msys→cmd). Si no hay ninguna vía, deja evento
`index-fallback / reason=no-detach` en vez de callar. 18 tests en
`test-exo-index.sh`, incluido uno que mata el process group del lanzador con el
indexado a medias y comprueba que sobrevive — la propiedad exacta por la que
existía el `setsid`.

**Ese arreglo no está desplegado**: el plugin instalado viene del marketplace de
GitHub, así que hasta que la rama no se empuje y suba la versión de `reflex`, la
máquina sigue con el script viejo.

### La suite de tests no compilaba en Windows — y por tanto no corría NINGUNO

`tests/guarda_modelo.rs` (el test del guard de modelo, de la Ola 1) usaba
`std::os::unix::fs::PermissionsExt` y `Permissions::from_mode(0o000)` para
provocar un error de lectura real a mitad del indexado. En Windows eso no
compila, y **cargo aborta la suite entera si un solo target no compila**: el
resultado era 0 tests ejecutados, no "un test menos".

Peor: el fallo se disfrazaba de éxito. `cargo test … | tail -40` devolvía
código 0 — el de `tail`, no el de cargo. El código real era 101.

Arreglo en `fix/test-guarda-modelo-portable`: en vez de `#[cfg(unix)]`, se
sustituye el mecanismo por **UTF-8 inválido en el fichero** (`read_to_string`
valida UTF-8 y devuelve un `io::Error` de kind `InvalidData`, el mismo camino de
código que el `EACCES`). Portable, determinista y sin perder cobertura en
ninguna plataforma. De paso elimina un no-op que ya existía: bajo root el
`chmod 0o000` no se honra y el test se auto-saltaba, así que en CI en contenedor
no probaba nada.

La idea de usar un directorio en vez de un fichero se descartó tras leer
`src/walker.rs:28-37`: `visita` solo hace `push` en la rama
`is_file() && extension == "md"`, así que un directorio nunca llegaría a
`parsea_nota`.

### Los tests asumen rutas de tempdir cortas

`recall_contenido::contenido_respeta_el_cap_de_bytes` fijaba `--cap-bytes 120`.
`recall.rs` mete la **ruta absoluta** en cada entrada del bloque; en `/tmp` son
~25 bytes y en `C:\Users\…\AppData\Local\Temp\…` ~55, así que en Windows ni la
primera línea de contenido cabe bajo ese cap y el recall aborta con "sin notas
core ni recientes que servir". El test no medía lo que creía medir: medía la
longitud del tempdir del SO. Medido: cabecera 55 + blanco 1, quedan 64 para una
línea de título que en Windows cuesta 67. **Tres bytes decidían la plataforma.**

Arreglo en `fix/test-recall-cap-portable`: los caps se **derivan del bloque
realmente renderizado** en la máquina (render con 8192, desglose por líneas,
sumas acumuladas), así que la longitud de la ruta ya está dentro del número. El
test quedó más estricto que antes — barre diez caps donde había uno, y exige
prefijo literal y líneas enteras. Se comprobó que sigue mordiendo mutando el
truncado de producción de tres formas distintas: las tres lo ponen rojo.
`src/recall.rs` quedó idéntico a `main` (verificado con `git diff`).

### El `jq` nativo de Windows emite CRLF — y solo muerde en un patrón

`jq` instalado por winget es un **binario PE32+ nativo**, no msys: abre stdout en
modo texto y termina las líneas con `\r\n`. La mayoría del repo es inmune porque
captura `jq` con `$(...)`, y bash **sí** elimina el `\r` final en sustitución de
comando. El patrón que no sobrevive es `while read` desde *process substitution*:

```bash
while IFS=$'\t' read -r sid aid; do …; done < <(jq -r '…|@tsv' …)
# aid vale "a1\r" → el glob agent-${aid}.jsonl busca agent-a1<CR>.jsonl
```

Único sitio del repo con ese patrón: `a1-gate.sh:201-202`. Tumbaba 22 de los 73
checks de su suite y dejaba la métrica U1 permanentemente en
`INSUFICIENTE-N` — con sesgo conservador (nunca un PASS falso), lo que hace el
fallo aún más fácil de no ver. Con un `jq` de msys2 no pasaría: es específico de
tener el nativo en el PATH.

### `iconv` no existe en Git Bash

Solo están las DLLs, no el ejecutable. `test-compose-inject.sh:302` validaba
UTF-8 con `iconv -f utf8 -t utf8 … || UTF8_OK=0`: el shell devuelve 127, el `||`
se dispara y la aserción falla **sin haber validado nada**. Un test que suspende
por una razón que no tiene que ver con lo que dice medir. El producto estaba
bien: el corte por líneas enteras de `compose-inject.sh` no parte caracteres
multibyte, verificado con payload real.

### Rutas absolutas a `/home/paul`

`skills/consolida/SKILL.md` lleva 13 y depende de `kbx`, que aquí no está
compilado (no hay toolchain de Go). El skill queda inservible en esta máquina.
Deuda ya anotada en el runbook de C9.

### Los hooks `.sh` sí corren

Claude Code en Windows usa Git Bash como shell por defecto para hooks cuando
está instalado, así que `"${CLAUDE_PLUGIN_ROOT}"/scripts/x.sh` funciona tal cual
— no hizo falta ningún wrapper `.cmd`. Verificado extremo a extremo con los
cuatro hooks de reflex.

## Lo que mintió (para la próxima)

- **`winget` devolvió `Successfully installed` con código 0 tres veces sin
  instalar el compilador.** El workload `Microsoft.VisualStudio.Workload.VCTools`
  **sin `--includeRecommended` no trae MSVC**: el compilador es un componente
  *recomendado*, no requerido. Se instalaron LLVM, el SDK y los redistribuibles,
  y nada más. Solo se ve mirando `VC\Tools\MSVC` en disco.
- **`--override` de winget se perdió por el camino.** El log del instalador
  (`%TEMP%\dd_installer_*.log`) mostraba `Raw Command line: setup.exe` sin un
  solo argumento, y una sesión interactiva abierta hora y media que se cerró con
  exit 0. Winget informa del código del bootstrapper, no del trabajo real.
- **`setup.exe` no se auto-eleva.** Con `--quiet` sale con 5007 y
  `isadmin: False` en el log: *"Commands with --quiet or --passive should be run
  elevated from the beginning"*. El que sí se eleva es el bootstrapper
  `vs_BuildTools.exe`. Exit 1602 = el UAC salió y nadie lo aceptó.
- **`vswhere` sin `-products *` no lista Build Tools.** Devuelve vacío y parece
  que no hay nada instalado. Un diagnóstico entero se fue por ahí.
- **`ps -W` lista el binario sin extensión.** Un `until ! ps -W | grep -qi
  'exo.exe'` da por terminado un indexado que sigue vivo; relanzarlo encima da
  `database is locked`. Grepear `/exo` o el PID.
- **`cargo test … | tail -40` devolvió 0 con la suite rota.** El código de salida
  de una tubería es el del ÚLTIMO comando. Cargo salía con 101. Usar
  `${PIPESTATUS[0]}` o no meter cargo en una tubería.

Patrón común: **el código de salida no es evidencia**. Lo que valió en los seis
casos fue mirar el artefacto real — el directorio, el log del instalador, el
tamaño de la DB, el proceso, las líneas `test result:`.

## Deuda detectada, no tocada: rutas absolutas en el bloque de recall

`recall_arranque` emite cada core como `# {titulo} ({ruta_absoluta})`, mientras
que las recientes van por **permalink** (`kb/otra`, ~17 bytes con título). Misma
función, mismo bloque, dos convenciones de identidad.

Por qué importa aquí y no en Linux: el bloque real de esta máquina salió de
**6113 bytes sobre un cap de 6144** y descartó 2 líneas de 50. El margen eran 31
bytes; una sola ruta `C:\Users\paul\…` mide más que eso. Y el coste no está
acotado: los cores **no llevan `limite`** — se emiten todos —, así que el gasto
es `N_cores × longitud_ruta` y crece con cada core nuevo. Es la variable que peor
compite con la doctrina, porque crece justo cuando más doctrina tienes.

Contraargumento real, y por eso no se ha tocado: la ruta absoluta es
**accionable** — el agente hace `Read` directo sin resolver el permalink.

Dirección propuesta si algún día se aborda: emitir la raíz de la KB **una vez**
en la cabecera y las notas relativas a ella. Se conserva la accionabilidad, se
paga el prefijo común (que tiene cero información) una vez en vez de N, y el
bloque pasa a ser **estable entre máquinas** — lo que mata de raíz la clase de
bug "pasa en Linux, falla en Windows" que produjo los dos arreglos de tests de
esta jornada.

No es urgente: el cap se respeta y el aviso por stderr funciona. Es deuda de
eficiencia y portabilidad, no de corrección.

## Pendiente

- **Desplegar los arreglos.** Tres ramas apiladas, ninguna empujada:
  `fix/exo-index-portable` → `fix/test-guarda-modelo-portable` →
  `fix/test-recall-cap-portable`. El arreglo del hook **no llega a esta máquina**
  hasta que se empuje, suba la versión de `reflex` y se repunte el catálogo: el
  plugin instalado viene del marketplace de GitHub, no del repo local.
- Decidir sobre `consolida` + `kbx` en Windows.
- `exo` en `extraKnownMarketplaces` quedó **sin `autoUpdate`** (equipo-x sí lo tiene).
- Los commits quedaron firmados con la identidad corporativa de esta máquina
  (`dev@example.invalid`) en un repo personal.
- `cargo fmt --check` no es gate en este repo: 90 diffs preexistentes.

## Actualización 2026-08-26: plugin único exo

Los plugins `process` y `reflex` se fusionaron en un plugin único `exo`
v1.0.0 (nueve skills, agente `executor`). El repo `exo` pasó a ser su propio
marketplace: ya no se sirve desde `exo-plugins`. Los comandos de instalación
de más arriba (`claude plugin install process@exo` / `reflex@exo` contra el
marketplace `exo-plugins`) documentan lo que se hizo aquel día y no se tocan;
los equivalentes de hoy son:

```bash
claude plugin marketplace add pguerrerolinares/exo
claude plugin install exo@exo
```

Y la ruta de verificación de `exo-recall.sh` de más arriba
(`cache/exo/reflex/*/scripts/exo-recall.sh`) pasa a:

```bash
~/.claude/plugins/cache/exo/exo/*/scripts/exo-recall.sh
```
