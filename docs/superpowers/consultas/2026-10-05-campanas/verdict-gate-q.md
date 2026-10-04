# Verdict GATE — rama `q-w11` (ola 3, campaña Q: exo en W11 en lugar de superpowers)

Consultor fable fresco, 2026-10-05T01:39:54+02:00. No participé en ninguna fase de la rama.
Objeto: `.worktrees/q-w11` @ `b146d71`, 6 commits sobre `main` `beecead`. Régimen:
`.superpowers/fabrica/config.md` §"Ejecución de gates" (4 condiciones) + lección 1 del bloque
ACTUALIZACIÓN 2026-09-20. Criterio: KB `wisdom-paul/backlog/Backlog — exo.md:45,48`.

## Veredicto

**GATE: MERGED-con-condición — vía PR, nunca merge local.** Condición: se mergea solo con el
check requerido `install gate (windows-latest)` en verde en el PR de `q-w11`.

Línea para el review-package (la appendea el orquestador; yo no toco el package):

```
GATE: MERGED-con-condición — PR; mergear solo con «install gate (windows-latest)» verde (consultor fable, 2026-10-05T01:39:54+02:00, verdict=docs/superpowers/consultas/2026-10-05-campanas/verdict-gate-q.md)
```

Condición 4 del régimen: este fichero se commitea en `q-w11` ANTES del push/PR. No hay
`GATE-EXEC` local: el merge lo ejecuta Paul al mergear el PR (línea roja: `git push` = Paul).

## Citas textuales que sostienen el veredicto

1. `config.md` §"Ejecución de gates", condiciones 1-3 (cumplidas por este dispatch):
   > 1. **Fresco** […] 2. **Verificación primaria propia**: el consultor re-corre los oráculos citados (no se fía de un resumen) […] 3. **Mandato explícito de disenso** […]

2. `config.md:90-93`, lección 1 (manda la vía):
   > **Un fix de portabilidad a un SO que no se puede probar en local se mergea por PR, nunca por push directo.** […] **El CI del SO que no se puede probar es parte del fix, no una comprobación posterior.**

   Aquí no hay `powershell`, `pwsh` ni `dotnet`: `install.ps1` y `scripts/test-install.ps1` solo se han leído. Caso idéntico al de la lección.

3. Branch protection de `main`, leída por mí (`gh api repos/pguerrerolinares/exo/branches/main/protection/required_status_checks --jq .contexts`):
   > ["checks estáticos","scripts de plugin ejecutables","MSRV declarada (1.95)","test (ubuntu-latest)","tests del plugin (ubuntu-latest)","install gate (ubuntu-latest)","install gate (windows-latest)"]

   `install gate (windows-latest)` es check requerido: la condición es ejecutable, no aspiracional. `.github/workflows/ci.yml:285-286` lo corre con `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-install.ps1`.

4. Criterio, KB `Backlog — exo.md:45`:
   > **Shipeo al trabajo (W11)**: precondición, dejar superpowers. `install.ps1` baja `jq.exe` real, añade `~/.local/bin` al PATH de usuario, hace `exo init` y `claude plugin disable superpowers` (avisando, nunca uninstall); check en `exo doctor` y aviso en SessionStart si superpowers vuelve activo (scope project/managed).

   KB `:48`: `hook_ms` p95 W11 = 1671 ms > 1.500 (Linux 1035-1173), "Queda publicar la cifra en `plugins/exo/README.md`". La rama la publica (`plugins/exo/README.md` §"Latencia del hook SessionStart", mismas cifras, fecha 2026-09-24).

5. Verdict de diseño previo `verdicts/ola3-q-disable-superpowers.md` → B' (disable por defecto, `--scope user`, opt-out, nunca uninstall). `b146d71` lo implementa punto por punto (flags, orden opt-out → claude ausente → pre-check → ejecutar, `Write-Warning` sin `throw`, tests 8-11). Verificado contra `install.ps1:210-258` y `test-install.ps1:312-407`.

## Verificación primaria propia (cwd = worktree, rc reales sin pipes)

| Oráculo | Resultado | rc |
|---|---|---|
| `cd engine && cargo test --release --locked` | todos los `test result: ok`; 0 `FAILED`/`panicked` (186 unit + suites de integración, incl. `doctor` 4 tests nuevos y `doctor_cli` contrato de 13 checks) | 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | `Finished dev profile`, sin warnings | 0 |
| `bash scripts/test-install.sh` | `OK — los dos casos` | 0 |
| `bash scripts/test-plugin.sh` | `OK — 18/18 suites del plugin en verde` | 0 |
| `bash plugins/exo/scripts/test-exo-recall-superpowers.sh` | `7 pass, 0 fail` | 0 |
| `bash scripts/test-hooks-json.sh` | `[OK]` | 0 |
| `bash scripts/test-docs-vivos.sh` | `[OK]` | 0 |
| `BASE=main bash scripts/plugin-bump-gate.sh` | `OK — 1.5.13 → 1.5.14` | 0 |

`git status --short` del worktree: limpio antes de escribir este fichero. `main` local sigue en
`beecead` (plugin 1.5.13); `o-deuda` NO está mergeada; no hay PRs abiertos (`gh pr list`).

**Windows: NO ejecutado.** Todo lo de la sección siguiente es lectura, no evidencia. El oráculo de
`test-install.ps1` es el job `install gate (windows-latest)` del PR; el oráculo de la rama
`PathScope=User` (ver hallazgo M1) es el PAUL-STEP `irm … | iex` en W11.

## Lectura adversarial de `install.ps1` y `scripts/test-install.ps1` (PS 5.1, `irm | iex`)

Commits sin review final: `ce2f3fe` (hook) y `b146d71` (`install.ps1` + tests). Punto por punto:

**P/Invoke `SendMessageTimeout` (`install.ps1:164-166`).** Firma
`IntPtr SendMessageTimeout(IntPtr, uint, UIntPtr, string lParam, uint, uint, out UIntPtr)` con
`CharSet.Auto` y llamada `([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$broadcastRes)`
con `$broadcastRes = [UIntPtr]::Zero`: es byte a byte el patrón canónico (el de Chocolatey
`Update-SessionEnvironment`): HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, `[ref]` sobre
variable con valor del tipo del `out`. Compila con `Add-Type -MemberDefinition` en 5.1 (csc del .NET
Framework; el propio test ya usa `Add-Type` en `:47`). Se ejecuta una vez por proceso (sin colisión
"type already exists"). Envuelto en try/catch → nunca tumba la instalación. **OK.**

**`GetValue('Path', '', 'DoNotExpandEnvironmentNames')` (`:144`).** `Get-Item 'HKCU:\Environment'`
devuelve `RegistryKey`; con 3 args solo hay una sobrecarga y PS convierte el string al enum
`RegistryValueOptions` (conversión string→enum estándar en binding de métodos). Default `''` si no
hay `Path`. **OK.** No está en try/catch: si fallara, lanza (ver M1/M5).

**`Set-ItemProperty -Type ExpandString` (`:160`).** `-Type <RegistryValueKind>` es el parámetro
dinámico del proveedor Registry para `Set-ItemProperty` en 5.1. Escribir REG_EXPAND_SZ sobre un
`Path` que era REG_SZ es inocuo. La comparación de idempotencia (`:149-156`) compara cada entrada
cruda Y expandida contra `$Dir` normalizado → detecta `%USERPROFILE%\.local\bin`. **OK.**

**Llamada nativa a `claude` bajo `$ErrorActionPreference='Stop'` (`:243-253`).** En 5.1, con el
stderr del proceso PowerShell redirigido (caso exacto del harness: `*> $log`), cada línea de
stderr de un nativo se promueve a `NativeCommandError` y bajo `Stop` es terminante. El código
baja a `Continue` solo alrededor de `& claude`, lee `$LASTEXITCODE`, try/catch a `-1`, `finally`
restaura. `Get-Command claude -ErrorAction SilentlyContinue` (`:218`) anula el `Stop` para ese
cmdlet. `$spKey` se pasa como variable (el `@` no se interpreta). **OK.** Nota: `& $destino doctor`
(`:205`) y los `--version` corren bajo `Stop` sin esa protección; en consola real (stderr no
redirigido) 5.1 no envuelve, y en CI el payload solo escribe stdout. Pre-existente, no de Q.

**`$LASTEXITCODE` de un `.cmd` con `exit /b` (`test-install.ps1:320`).** PS lanza el `.cmd` vía
`cmd /c`; `exit /b N` deja ERRORLEVEL=N y `cmd /c` sale con él → `$LASTEXITCODE=N`. Es la mecánica
que verifica el caso 11 (stub sale 1 → `WARNING`). **OK**, y lo verifica el CI.

**Sintaxis del stub (`:320`).** Tres líneas ASCII CRLF: `@echo off`, `>>"<log>" echo %*`
(redirección prefijada, ruta de `GetTempPath()` sin caracteres raros), `exit /b N`. `%*` expande
`plugin disable superpowers@test --scope user`. La aserción `-notlike '*plugin disable
superpowers@test --scope user*'` sobre `Get-Content -Raw` (wildcard de PS es Singleline). **OK.**

**Ningún test toca el `settings.json` real del runner.** `test-install.ps1:102` fija por defecto
`EXO_CLAUDE_SETTINGS` a un fichero inexistente ("nada que hacer" aunque el runner tuviera
`claude`); casos 8-11 lo apuntan a un JSON temporal y anteponen el stub al PATH (`:334`), así que
ni con un `claude` real se invocaría el real. `EXO_PATH_SCOPE=Process` en todos (`:95`) → HKCU
intacto. `:408-413` limpian el entorno. **OK.**

**Orden "verificar antes de copiar".** exo: baja+SHA (`:74-97`); jq: baja+SHA fijado (`:99-118`);
solo entonces `New-Item $Dir` (`:120`) y `Copy-Item` (`:122`, `:127`). Casos 5 y 6 asertan que
`$dest` ni existe tras un jq corrupto / contra el pin. **OK.**

**Hook `ce2f3fe` (`exo-recall.sh:163-191`).** Bucle `[[ =~ ]]` con
`resto="${resto#*"${BASH_REMATCH[0]}"}"`: termina (el match nunca es vacío), sin spawns, precedencia
user < project < local por "último fichero que menciona la clave". Bajo `set -uo pipefail` (sin
`-e`). 7/7 corridos por mí. **OK.**

### Clasificación: BLOQUEANTES — ninguno. Menores:

- **M1 (el que importa).** `install gate` verde **no verifica `install.ps1:143-169`**: todos los
  casos corren `EXO_PATH_SCOPE=Process` por diseño (`test-install.ps1:80-83`, `:95`), así que
  `GetValue(…DoNotExpandEnvironmentNames)`, `Set-ItemProperty -Type ExpandString` y el P/Invoke no
  tienen oráculo automático en ningún sitio. Es el espíritu exacto de la lección 1. Fallo posible:
  un `throw` en `:144` o `:160` tras copiar binarios (`:122-133`) → exo.exe y jq.exe instalados,
  PATH sin tocar, sin `--version`/doctor/superpowers; ruidoso y recuperable, por eso menor.
  Recomendación (no condición): caso 12 guardado por `$env:GITHUB_ACTIONS -eq 'true'` con
  `EXO_PATH_SCOPE=User` que aserte `(Get-Item HKCU:\Environment).GetValueKind('Path') -eq 'ExpandString'`
  y que una entrada `%VAR%` sobreviva, restaurando el valor al final. Hasta entonces el oráculo es
  el PAUL-STEP `irm … | iex` en W11, y así debe leerse "install-gate verde".
- **M2.** El bloque superpowers (`:210-258`) va DESPUÉS de `& $destino doctor` (`:205`): doctor
  informa del estado previo al disable. Cosmético; mover el bloque antes de doctor.
- **M3.** Caso 11 asierta el literal `'WARNING'` (`test-install.ps1:402`): prefijo localizable
  ("ADVERTENCIA:" en es-ES). windows-latest es en-US; frágil fuera de CI.
- **M4.** Pre-check con `settings.json` no parseable → `catch` → "superpowers no está habilitado —
  nada que hacer" (`install.ps1:233-238`), mientras `doctor` da `Warn` en el mismo caso
  (`doctor.rs:997-1004`). Asimetría silenciosa; un `Write-Warning` en el `catch` basta.
- **M5.** En un W11 corporativo con ConstrainedLanguage, `Add-Type` (try/catch, degrada) y
  `.GetValue()` (sin catch: ahí lanzaría) están bloqueados. Superficie pre-existente
  (`[System.IO.Path]::GetTempPath()` `:67`, `[Environment]::SetEnvironmentVariable` de `c107ca4`);
  Q la amplía, no la crea.
- **M6.** Hook: `{"superpowers@a":true}` en user y `{"superpowers@b":false}` en project → no avisa
  (claves distintas se pisan). Falso negativo de borde; aceptado como el resto de la heurística.

## Deuda contra el criterio KB:45 (no bloquea, no debe quedar en silencio)

- `:45` pide aviso de SessionStart para "scope project/**managed**"; el hook cubre user/project/local
  y declara no mirar managed (`exo-recall.sh:164-167`). El package lo llama "riesgo aceptado": es
  aceptación del executor, no de Paul. Queda como deuda citable.
- `:45` dice "hace `exo init`": el instalador solo lo encadena con `EXO_INIT_KB`+`EXO_INIT_NAME`
  (`install.ps1:190-199`, paridad pre-existente con `install.sh`). Parcial, no regresión de Q.
- `install.sh` no toca superpowers (ya anotado en el verdict B'). Paridad Linux fuera de Q.

## Vía de merge y condiciones para el orquestador

1. Commitear este verdict en `q-w11` (condición 4). Después, PAUL-STEP: `git push -u origin q-w11 && gh pr create`.
2. Mergear SOLO con `install gate (windows-latest)` verde (required check, cita 3). Si sale rojo,
   el fix entra en la misma rama y se re-adjudica; no se mergea "y luego se arregla".
3. Colisión de versión: `q-w11` y `o-deuda` suben ambas a 1.5.14. La que entre segunda
   re-bumpea a 1.5.15; `plugin-bump-gate` solo lo detecta si se re-corre con `BASE=main` DESPUÉS
   de que `main` se mueva.
4. Post-merge, PAUL-STEP (ya en el package): release del engine para que `superpowers_disabled`
   llegue a W11, y `irm … | iex` real en W11 — ese es el oráculo de M1.

## Qué busqué para objetar

- **Que la vía pudiera ser merge local.** No: la lección 1 es literal y el caso es idéntico
  (código para un SO sin oráculo local). Además confirmé que el check de Windows es requerido;
  un merge local se saltaría exactamente la verificación que la lección declara parte del fix.
- **Un bug de PS 5.1 en los dos commits sin review.** Repasé cada construcción nueva contra la
  semántica de 5.1 (enum por string, parámetro dinámico `-Type`, `[ref]`/`out`, `NativeCommandError`
  con stderr redirigido, `cmd /c` + `exit /b`, `-like` Singleline, PATHEXT para `claude.cmd`). No
  encontré ninguna que falle; lo que encontré es que la mitad no la ejecuta CI (M1).
- **Que un test tocara el entorno real del runner o de Paul.** `:95`, `:102`, `:334`: no.
- **Que B' no se hubiera implementado como dictó el verdict previo.** Comparé los 7 puntos de la
  spec del verdict con `:210-258` y los casos 8-11: coinciden, incluido el desvío declarado
  ("claude ausente" imprime en vez de callar, para el caso 4).
- **Que el README mintiera sobre `hook_ms`.** Cifras y fecha coinciden con KB:48.
- **Que la rama rompiera el contrato de `doctor --json`.** `doctor_cli.rs` extiende la lista de
  checks (no encoge) y `docs/arquitectura.md`/`instalacion.md` pasan de doce a trece; `test-docs-vivos` verde.
- **Que el BOM de `install.ps1` fuera nuevo.** `git show main:install.ps1 | head -c 3` = `ef bb bf`:
  pre-existente, ya probado en W11 según los comentarios del propio script (2026-09-11).
- **Que hubiera un PR abierto o `main` movido** que invalidara la base: no (`gh pr list` vacío,
  `main`=`beecead`).
- No encontré nada que me hiciera rechazar. Lo más cerca de una objeción es M1: por eso el
  veredicto es "con condición" y no un MERGED liso, y por eso la deuda de M1 queda escrita aquí
  y no en un "riesgo aceptado" del package.
