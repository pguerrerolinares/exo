#Requires -Version 5.1
<#
.SYNOPSIS
  Gate de install.ps1, sin red: fabrica una "release" falsa en disco y la
  sirve por file:///, que Invoke-WebRequest sabe leer.
.DESCRIPTION
  Casos 1 y 2 como scripts/test-install.sh (3-11 son propios de Windows):
    1. checksum correcto -> instala, el binario queda en el -Dir pedido.
    2. checksum manipulado -> ABORTA y NO deja binario (ni crea el destino).
  El caso 2 es el que importa: un instalador que verifica el hash y sigue
  igual es peor que uno que no lo verifica, porque parece seguro.

  Payload: install.ps1 invoca "$destino --version" y desde el fix del
  2026-09-11 mira $LASTEXITCODE. Un fichero de texto con extension .exe no es
  un PE valido: PowerShell lanza un error de arranque de proceso ANTES de
  llegar a esa comprobacion, y el caso "bueno" fallaria por una razon que no
  tiene nada que ver con el gate que se quiere probar. Por eso el payload es
  un .exe real y minimo, compilado en el momento con csc.exe via Add-Type
  (preinstalado en los runners windows-latest de GitHub junto con el .NET
  Framework), que ignora sus argumentos y siempre sale 0. No usa el exo.exe
  del propio repo a proposito: este test no depende de que el engine este
  compilado, igual que test-install.sh no depende de tener un exo real.
#>
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$asset = 'exo-x86_64-pc-windows-msvc.exe'
$installScript = Join-Path $repoRoot 'install.ps1'
$fallos = 0

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('exo-test-install-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null

function New-PayloadReal {
    param([string]$Destino)
    $src = @'
using System;
class DummyExo {
    static int Main(string[] args) {
        Console.WriteLine("exo-falso " + string.Join(" ", args));
        return 0;
    }
}
'@
    Add-Type -TypeDefinition $src -OutputType ConsoleApplication -OutputAssembly $Destino
}

$payload = Join-Path $tmp 'dummy-exo.exe'
New-PayloadReal -Destino $payload
$payloadBytes = [System.IO.File]::ReadAllBytes($payload)

function New-Release {
    # Los DOS formatos estandar, porque la release de v0.1.0 publica los dos:
    #   `shasum -a 256` -> "<hash>  <fichero>"   (dos espacios)
    #   `sha256sum`     -> "<hash> *<fichero>"   (`*` = modo binario)
    # El asset de Windows lo firma `sha256sum` (el fallback del workflow,
    # porque `shasum` no existe en el bash de windows-latest) y los de Linux y
    # macOS los firma `shasum`. Un instalador que solo entienda uno rechaza su
    # propio binario: paso de verdad contra la release real el 2026-09-11.
    param([string]$Dir, [switch]$FormatoSha256sum)
    New-Item -ItemType Directory -Force -Path $Dir | Out-Null
    $destAsset = Join-Path $Dir $asset
    [System.IO.File]::WriteAllBytes($destAsset, $payloadBytes)
    $hash = (Get-FileHash -Path $destAsset -Algorithm SHA256).Hash.ToLower()
    if ($FormatoSha256sum) {
        $linea = "$hash *$asset"
    } else {
        $linea = "$hash  $asset"
    }
    Set-Content -Path "$destAsset.sha256" -Value $linea -Encoding ascii -NoNewline
}

function Get-FileUrl {
    param([string]$Dir)
    'file:///' + $Dir.Replace('\', '/')
}

# jq: todos los casos corren con un jq falso servido por file:// y con
# PathScope=Process. Sin lo primero, el instalador bajaria jq de GitHub (o lo
# saltaria si el runner ya trae uno); sin lo segundo escribiria el PATH de
# USUARIO real de quien corra este test.
function New-ReleaseJq {
    param([string]$Dir, [switch]$Corrupto)
    New-Item -ItemType Directory -Force -Path $Dir | Out-Null
    $f = Join-Path $Dir 'jq-windows-amd64.exe'
    [System.IO.File]::WriteAllBytes($f, $payloadBytes)
    $hash = (Get-FileHash -Path $f -Algorithm SHA256).Hash.ToLower()
    Set-Content -Path (Join-Path $Dir 'sha256sum.txt') -Value "$hash  jq-windows-amd64.exe" -Encoding ascii
    if ($Corrupto) { Add-Content -Path $f -Value 'basura' -Encoding ascii -NoNewline }
}
New-ReleaseJq -Dir (Join-Path $tmp 'jq-ok')
$env:EXO_JQ_BASE_URL = Get-FileUrl -Dir (Join-Path $tmp 'jq-ok')
$env:EXO_PATH_SCOPE = 'Process'
# El hash de jq esta fijado en install.ps1; este seam lo sustituye por el del
# jq falso. El caso 6 lo quita para probar que el pin manda.
$jqFalsoSha = (Get-FileHash -Path (Join-Path $tmp 'jq-ok\jq-windows-amd64.exe') -Algorithm SHA256).Hash.ToLower()
$env:EXO_JQ_SHA256 = $jqFalsoSha
# Sin settings de claude por defecto: ningun caso puede tocar el superpowers
# real de quien corra el test (en un runner con claude, el pre-check lo veria).
$env:EXO_CLAUDE_SETTINGS = Join-Path $tmp 'no-existe-settings.json'

# --- Caso 1: checksum correcto -> instala y el binario queda donde el gate mira
$rel = Join-Path $tmp 'release-ok'
$dest = Join-Path $tmp 'bin-ok'
New-Release -Dir $rel
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel
$env:EXO_DIR = $dest
$log = Join-Path $tmp 'ok.log'
# ErrorActionPreference='Continue' solo alrededor de la llamada: install.ps1
# hijo escribe a stderr (Write-Warning, etc.) y PowerShell promueve cada
# linea de stderr de un proceso nativo a un ErrorRecord. Bajo 'Stop' eso se
# convierte en excepcion terminante y aborta ESTE script aunque el hijo haya
# salido con exit 0 — no es el caso que se quiere probar.
$ErrorActionPreference = 'Continue'
& powershell -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log
$ec = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:EXO_BASE_URL = $null
$env:EXO_DIR = $null

if ($ec -eq 0) {
    if (Test-Path (Join-Path $dest 'exo.exe')) {
        Write-Output "test-install: OK - instalado en $dest\exo.exe"
    } else {
        Write-Output "test-install: FALLO - checksum-correcto salio 0 pero no hay binario en $dest\exo.exe"
        Get-Content $log
        $fallos = 1
    }
} else {
    Write-Output "test-install: FALLO - checksum-correcto no instalo (exit $ec)"
    Get-Content $log
    $fallos = 1
}

# --- Caso 2: checksum manipulado -> aborta y NO deja binario
$rel2 = Join-Path $tmp 'release-mala'
$dest2 = Join-Path $tmp 'bin-malo'
New-Release -Dir $rel2
# Se corrompe el binario DESPUES de firmar: el .sha256 ya no cuadra.
$assetPath2 = Join-Path $rel2 $asset
Add-Content -Path $assetPath2 -Value 'basura' -Encoding ascii -NoNewline

$env:EXO_BASE_URL = Get-FileUrl -Dir $rel2
$env:EXO_DIR = $dest2
$log2 = Join-Path $tmp 'malo.log'
$ErrorActionPreference = 'Continue'
& powershell -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log2
$ec2 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:EXO_BASE_URL = $null
$env:EXO_DIR = $null

if ($ec2 -eq 0) {
    Write-Output "test-install: FALLO - checksum-manipulado salio 0"
    Get-Content $log2
    $fallos = 1
} else {
    if (Test-Path $dest2) {
        Write-Output "test-install: FALLO - checksum-manipulado aborto pero dejo algo en $dest2"
        $fallos = 1
    } else {
        Write-Output "test-install: OK - checksum malo aborta y no instala nada"
    }
}

# --- Caso 3: el .sha256 en formato `sha256sum` ("<hash> *<fichero>") instala
# igual. Es regresion de un fallo REAL: install.ps1 tomaba el `*` como parte
# del nombre y rechazaba el binario que la propia release publica.
$rel3 = Join-Path $tmp 'release-sha256sum'
$dest3 = Join-Path $tmp 'bin-sha256sum'
New-Release -Dir $rel3 -FormatoSha256sum
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel3
$env:EXO_DIR = $dest3
$log3 = Join-Path $tmp 'sha256sum.log'
# Mismo blindaje que los otros dos casos: stderr del hijo no puede abortar
# este script bajo 'Stop'.
$ErrorActionPreference = 'Continue'
& powershell -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log3
$ec3 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($ec3 -ne 0) {
    Write-Output "test-install: FALLO - formato sha256sum rechazado (exit $ec3)"
    $fallos = 1
} elseif (-not (Test-Path (Join-Path $dest3 'exo.exe'))) {
    Write-Output "test-install: FALLO - formato sha256sum salio 0 pero no instalo"
    $fallos = 1
} else {
    Write-Output "test-install: OK - el formato de sha256sum tambien instala"
}

# --- Caso 4: jq. El PATH del hijo se reduce a System32 para que NO resuelva
# ningun jq previo (los runners windows-latest traen uno) y el instalador
# tenga que bajar el de la "release" falsa. PathScope=Process: no se toca el
# PATH de usuario real de quien corra este test.
$psExe = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$pathOriginal = $env:PATH

$rel4 = Join-Path $tmp 'release-jq'
$dest4 = Join-Path $tmp 'bin-jq'
New-Release -Dir $rel4
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel4
$env:EXO_DIR = $dest4
$env:PATH = Join-Path $env:SystemRoot 'System32'
$log4 = Join-Path $tmp 'jq.log'
$ErrorActionPreference = 'Continue'
& $psExe -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log4
$ec4 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:PATH = $pathOriginal
if ($ec4 -ne 0) {
    Write-Output "test-install: FALLO - caso jq no instalo (exit $ec4)"
    Get-Content $log4
    $fallos = 1
} elseif (-not (Test-Path (Join-Path $dest4 'jq.exe'))) {
    Write-Output "test-install: FALLO - caso jq salio 0 pero no hay jq.exe en $dest4"
    $fallos = 1
} elseif (-not (Select-String -Path $log4 -Pattern 'antepuesto al PATH' -Quiet)) {
    Write-Output "test-install: FALLO - caso jq no antepuso $dest4 al PATH"
    Get-Content $log4
    $fallos = 1
} elseif (-not (Select-String -Path $log4 -Pattern 'claude plugin disable superpowers' -Quiet)) {
    Write-Output "test-install: FALLO - el instalador no imprimio como desactivar superpowers"
    $fallos = 1
} else {
    Write-Output "test-install: OK - jq.exe instalado, PATH antepuesto, instruccion de superpowers impresa"
}

# --- Caso 5: jq con checksum manipulado aborta y NO deja ni exo ni jq
$rel5 = Join-Path $tmp 'release-jq-malo'
$dest5 = Join-Path $tmp 'bin-jq-malo'
New-Release -Dir $rel5
New-ReleaseJq -Dir (Join-Path $tmp 'jq-malo') -Corrupto
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel5
$env:EXO_JQ_BASE_URL = Get-FileUrl -Dir (Join-Path $tmp 'jq-malo')
$env:EXO_DIR = $dest5
$env:PATH = Join-Path $env:SystemRoot 'System32'
$log5 = Join-Path $tmp 'jq-malo.log'
$ErrorActionPreference = 'Continue'
& $psExe -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log5
$ec5 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:PATH = $pathOriginal
$env:EXO_BASE_URL = $null
$env:EXO_JQ_BASE_URL = $null
$env:EXO_DIR = $null
if ($ec5 -eq 0) {
    Write-Output "test-install: FALLO - jq con checksum malo salio 0"
    $fallos = 1
} elseif (Test-Path $dest5) {
    Write-Output "test-install: FALLO - jq malo aborto pero dejo algo en $dest5"
    $fallos = 1
} else {
    Write-Output "test-install: OK - jq con checksum malo aborta y no instala nada"
}

# --- Caso 6: el SHA256 de jq esta fijado en el script. Sin el seam, el jq
# falso (cuyo hash no es el de jq 1.7.1) tiene que rechazarse aunque este bien
# firmado en el servidor: un hash que viaja con el binario no verifica nada.
$rel6 = Join-Path $tmp 'release-jq-pin'
$dest6 = Join-Path $tmp 'bin-jq-pin'
New-Release -Dir $rel6
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel6
$env:EXO_DIR = $dest6
$env:EXO_JQ_BASE_URL = Get-FileUrl -Dir (Join-Path $tmp 'jq-ok')
$env:EXO_JQ_SHA256 = $null
$env:PATH = Join-Path $env:SystemRoot 'System32'
$log6 = Join-Path $tmp 'jq-pin.log'
$ErrorActionPreference = 'Continue'
& $psExe -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log6
$ec6 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:PATH = $pathOriginal
$env:EXO_JQ_SHA256 = $jqFalsoSha
if ($ec6 -eq 0) {
    Write-Output "test-install: FALLO - jq sin el hash fijado salio 0"
    $fallos = 1
} elseif (Test-Path $dest6) {
    Write-Output "test-install: FALLO - jq contra el pin aborto pero dejo algo en $dest6"
    $fallos = 1
} else {
    Write-Output "test-install: OK - el SHA256 de jq fijado en el script manda sobre lo que sirva el servidor"
}

# --- Caso 7: idempotencia del PATH con entradas normalizadas (mayusculas, `/`
# y `\` final). PathScope=Process, asi que se prueba la comparacion, no el
# registro.
$rel7 = Join-Path $tmp 'release-path'
$dest7 = Join-Path $tmp 'bin-path'
New-Release -Dir $rel7
$env:EXO_BASE_URL = Get-FileUrl -Dir $rel7
$env:EXO_DIR = $dest7
$env:PATH = (Join-Path $env:SystemRoot 'System32') + ';' + $dest7.ToUpper().Replace('\', '/') + '/'
$log7 = Join-Path $tmp 'path.log'
$ErrorActionPreference = 'Continue'
& $psExe -NoProfile -ExecutionPolicy Bypass -File $installScript *> $log7
$ec7 = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
$env:PATH = $pathOriginal
if ($ec7 -ne 0) {
    Write-Output "test-install: FALLO - caso PATH no instalo (exit $ec7)"
    Get-Content $log7
    $fallos = 1
} elseif (Select-String -Path $log7 -Pattern 'antepuesto al PATH' -Quiet) {
    Write-Output "test-install: FALLO - el PATH ya tenia $dest7 (otra grafia) y se reescribio"
    $fallos = 1
} else {
    Write-Output "test-install: OK - PATH idempotente con entradas en otra grafia"
}

# --- Casos 8-11: superpowers. Un `claude.cmd` falso al frente del PATH apunta
# sus argumentos a un fichero; EXO_CLAUDE_SETTINGS es el settings.json de
# usuario que lee el pre-check.
$stubDir = Join-Path $tmp 'stub'
New-Item -ItemType Directory -Force -Path $stubDir | Out-Null
$stubLog = Join-Path $tmp 'stub-args.log'
function New-StubClaude {
    param([int]$Salida)
    Set-Content -Path (Join-Path $stubDir 'claude.cmd') -Value @('@echo off', ('>>"{0}" echo %*' -f $stubLog), ('exit /b {0}' -f $Salida)) -Encoding ascii
}
function Set-SettingsSp {
    param([string]$Json)
    $f = Join-Path $tmp 'settings-sp.json'
    Set-Content -Path $f -Value $Json -Encoding ascii
    $env:EXO_CLAUDE_SETTINGS = $f
}
function Invoke-InstallSp {
    param([string]$Name)
    $relN = Join-Path $tmp "release-$Name"
    New-Release -Dir $relN
    $env:EXO_BASE_URL = Get-FileUrl -Dir $relN
    $env:EXO_DIR = Join-Path $tmp "bin-$Name"
    $env:PATH = "$stubDir;$pathOriginal"
    $logN = Join-Path $tmp "$Name.log"
    Remove-Item -Force $stubLog -ErrorAction SilentlyContinue
    $ErrorActionPreference = 'Continue'
    & $psExe -NoProfile -ExecutionPolicy Bypass -File $installScript *> $logN
    $ecN = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    $env:PATH = $pathOriginal
    return $ecN
}
function Get-StubArgs {
    if (Test-Path $stubLog) { return (Get-Content $stubLog -Raw) } else { return '' }
}

# 8: habilitado -> disable con la clave completa y --scope user
New-StubClaude -Salida 0
Set-SettingsSp -Json '{"enabledPlugins":{"superpowers@test":true}}'
$ec8 = Invoke-InstallSp -Name 'sp-on'
$args8 = Get-StubArgs
if ($ec8 -ne 0) {
    Write-Output "test-install: FALLO - caso superpowers habilitado salio $ec8"
    Get-Content (Join-Path $tmp 'sp-on.log')
    $fallos = 1
} elseif ($args8 -notlike '*plugin disable superpowers@test --scope user*') {
    Write-Output "test-install: FALLO - claude no recibio 'plugin disable superpowers@test --scope user' (recibio: $args8)"
    $fallos = 1
} elseif (-not (Select-String -Path (Join-Path $tmp 'sp-on.log') -Pattern 'claude plugin enable superpowers@test' -Quiet)) {
    Write-Output "test-install: FALLO - no aviso de como revertir"
    $fallos = 1
} else {
    Write-Output "test-install: OK - superpowers habilitado -> claude plugin disable <clave> --scope user"
}

# 9: opt-out por entorno
$env:EXO_DISABLE_SUPERPOWERS = '0'
$ec9 = Invoke-InstallSp -Name 'sp-keep'
$env:EXO_DISABLE_SUPERPOWERS = $null
if ($ec9 -ne 0) {
    Write-Output "test-install: FALLO - EXO_DISABLE_SUPERPOWERS=0 salio $ec9"
    $fallos = 1
} elseif (Test-Path $stubLog) {
    Write-Output "test-install: FALLO - EXO_DISABLE_SUPERPOWERS=0 pero se invoco claude"
    $fallos = 1
} else {
    Write-Output "test-install: OK - EXO_DISABLE_SUPERPOWERS=0 no toca superpowers"
}

# 10: settings sin superpowers habilitado -> no se invoca
Set-SettingsSp -Json '{"enabledPlugins":{"superpowers@test":false}}'
$ec10 = Invoke-InstallSp -Name 'sp-off'
if ($ec10 -ne 0) {
    Write-Output "test-install: FALLO - superpowers ya deshabilitado salio $ec10"
    $fallos = 1
} elseif (Test-Path $stubLog) {
    Write-Output "test-install: FALLO - superpowers ya deshabilitado pero se invoco claude"
    $fallos = 1
} else {
    Write-Output "test-install: OK - superpowers ya deshabilitado: nada que hacer"
}

# 11: claude falla -> aviso, la instalacion sigue en verde
New-StubClaude -Salida 1
Set-SettingsSp -Json '{"enabledPlugins":{"superpowers@test":true}}'
$ec11 = Invoke-InstallSp -Name 'sp-fail'
if ($ec11 -ne 0) {
    Write-Output "test-install: FALLO - un claude que falla tumbo la instalacion (exit $ec11)"
    Get-Content (Join-Path $tmp 'sp-fail.log')
    $fallos = 1
} elseif (-not (Select-String -Path (Join-Path $tmp 'sp-fail.log') -Pattern 'WARNING' -Quiet)) {
    Write-Output "test-install: FALLO - claude salio 1 y no hubo Write-Warning"
    $fallos = 1
} else {
    Write-Output "test-install: OK - claude plugin disable falla -> aviso, install exit 0"
}
$env:EXO_BASE_URL = $null
$env:EXO_DIR = $null
$env:EXO_JQ_BASE_URL = $null
$env:EXO_JQ_SHA256 = $null
$env:EXO_PATH_SCOPE = $null
$env:EXO_CLAUDE_SETTINGS = $null

Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue


if ($fallos -ne 0) {
    Write-Output "test-install: hay fallos"
    exit 1
}
Write-Output "test-install: OK - los once casos"
exit 0
