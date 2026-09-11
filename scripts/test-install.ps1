#Requires -Version 5.1
<#
.SYNOPSIS
  Gate de install.ps1, sin red: fabrica una "release" falsa en disco y la
  sirve por file:///, que Invoke-WebRequest sabe leer.
.DESCRIPTION
  Mismos dos casos que scripts/test-install.sh:
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
    # Mismo formato que `shasum -a 256`: "<hash>  <fichero>", que es lo que
    # install.ps1 parsea (campos[0] = hash, campos[-1] = nombre de fichero).
    param([string]$Dir)
    New-Item -ItemType Directory -Force -Path $Dir | Out-Null
    $destAsset = Join-Path $Dir $asset
    [System.IO.File]::WriteAllBytes($destAsset, $payloadBytes)
    $hash = (Get-FileHash -Path $destAsset -Algorithm SHA256).Hash.ToLower()
    Set-Content -Path "$destAsset.sha256" -Value "$hash  $asset" -Encoding ascii -NoNewline
}

function Get-FileUrl {
    param([string]$Dir)
    'file:///' + $Dir.Replace('\', '/')
}

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

Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue

if ($fallos -ne 0) {
    Write-Output "test-install: hay fallos"
    exit 1
}
Write-Output "test-install: OK - los dos casos"
exit 0
