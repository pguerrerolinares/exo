#Requires -Version 5.1
<#
.SYNOPSIS
  Instala exo desde GitHub Releases en Windows.
.DESCRIPTION
  Baja el binario de la release, verifica su SHA256 y lo deja en
  $HOME\.local\bin\exo.exe — la misma ruta que ve Git Bash como
  ~/.local/bin/exo. Si ese directorio está en el PATH, kb-precommit.sh lo
  resuelve por `command -v exo` (mismo orden que los hooks); si no, cae al
  literal $HOME/.local/bin/exo(.exe). Si ninguno de los dos resuelve un exo
  ejecutable, ese gate sale 1 y BLOQUEA el commit (fail-closed); el escape
  consciente es git commit --no-verify.

  Además deja un jq.exe real (el binario oficial de jqlang, con su SHA256
  verificado) junto a exo.exe, y antepone ese directorio al PATH de USUARIO
  para que gane al alias de la Store (WindowsApps\jq.exe, que no es un jq).
  Si ya hay un jq real en el PATH, no lo toca.

  No desactiva superpowers por su cuenta: es una acción sobre la config de
  Claude Code del usuario. Imprime el comando; con -DisableSuperpowers lo
  ejecuta (disable, nunca uninstall).
#>
[CmdletBinding()]
param(
    [string]$Version = $(if ($env:EXO_VERSION) { $env:EXO_VERSION } else { 'latest' }),
    # OJO: `$HOME` de PowerShell sale de USERPROFILE/HOMEDRIVE+HOMEPATH y NO
    # mira `$env:HOME`, que Git Bash sí hereda. En una máquina con HOME puesto
    # a mano, los dos instaladores escribirían en sitios distintos. Mientras
    # ese directorio siga en el PATH da igual, porque kb-precommit.sh resuelve
    # por `command -v exo` antes que por el literal — pero si no está en el
    # PATH, el literal `$HOME/.local/bin/exo` que busca el gate quedaría
    # vacío, y si tampoco encuentra un exo ejecutable ahí, BLOQUEA el commit
    # (fail-closed) en vez de pasarlo en silencio. Se prefiere $env:HOME
    # cuando existe.
    [string]$Dir     = $(if ($env:EXO_DIR)     { $env:EXO_DIR }     else { Join-Path $(if ($env:HOME) { $env:HOME } else { $HOME }) '.local\bin' }),
    [string]$Repo    = $(if ($env:EXO_REPO)    { $env:EXO_REPO }    else { 'pguerrerolinares/exo' }),
    [string]$BaseUrl = $env:EXO_BASE_URL,
    [string]$JqBaseUrl = $(if ($env:EXO_JQ_BASE_URL) { $env:EXO_JQ_BASE_URL } else { 'https://github.com/jqlang/jq/releases/download/jq-1.7.1' }),
    # 'Process' es el seam de test: evita escribir el PATH real del usuario.
    [string]$PathScope = $(if ($env:EXO_PATH_SCOPE) { $env:EXO_PATH_SCOPE } else { 'User' }),
    [switch]$DisableSuperpowers
)

$ErrorActionPreference = 'Stop'

$asset = 'exo-x86_64-pc-windows-msvc.exe'

if (-not $BaseUrl) {
    if ($Version -eq 'latest') {
        $BaseUrl = "https://github.com/$Repo/releases/latest/download"
    } else {
        $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
    }
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("exo-install-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $binTmp = Join-Path $tmp $asset
    $shaTmp = "$binTmp.sha256"

    Write-Host "install: bajando $asset de $BaseUrl"
    Invoke-WebRequest -Uri "$BaseUrl/$asset"        -OutFile $binTmp -UseBasicParsing
    Invoke-WebRequest -Uri "$BaseUrl/$asset.sha256" -OutFile $shaTmp -UseBasicParsing

    # El fichero viene en uno de los DOS formatos estandar, y la release
    # publica los dos a la vez: `shasum -a 256` escribe "<hash>  <fichero>" y
    # `sha256sum` escribe "<hash> *<fichero>", donde el `*` marca modo binario
    # y NO es parte del nombre. En v0.1.0 el asset de Windows lo firmo
    # `sha256sum` (el fallback del workflow, porque `shasum` no existe en el
    # bash de windows-latest) y los de Linux y macOS `shasum`. Sin quitar ese
    # `*`, este instalador rechaza su propio binario — medido contra la
    # release real el 2026-09-11. `shasum -c` del camino bash lo entiende
    # nativamente; aqui hay que decirlo.
    # Se compara ANTES de copiar nada: un fallo no deja binario a medias.
    $campos   = (Get-Content $shaTmp -Raw).Trim() -split '\s+'
    $esperado = $campos[0]
    $firmado  = $campos[-1] -replace '^\*', ''
    if ($campos.Count -ge 2 -and $firmado -ne $asset) {
        throw "install: el .sha256 es de '$firmado', no de '$asset'. NO se ha instalado nada."
    }
    $real     = (Get-FileHash -Path $binTmp -Algorithm SHA256).Hash.ToLower()
    if ($real -ne $esperado.ToLower()) {
        throw "install: el SHA256 no cuadra. Esperado $esperado, calculado $real. NO se ha instalado nada."
    }
    Write-Host "install: sha256 verificado"

    # jq se baja y verifica ANTES de copiar nada, igual que exo: un fallo no
    # deja binarios a medias. Un alias de WindowsApps NO cuenta como jq.
    $jqAsset = 'jq-windows-amd64.exe'
    $jqPrevio = Get-Command jq -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    $jqReal = $jqPrevio -and ($jqPrevio.Source -notlike '*WindowsApps*')
    $jqTmp = Join-Path $tmp $jqAsset
    if ($jqReal) {
        Write-Host "install: jq ya presente en $($jqPrevio.Source) — no se toca"
    } else {
        Write-Host "install: bajando $jqAsset de $JqBaseUrl"
        Invoke-WebRequest -Uri "$JqBaseUrl/$jqAsset"        -OutFile $jqTmp -UseBasicParsing
        Invoke-WebRequest -Uri "$JqBaseUrl/sha256sum.txt"   -OutFile "$jqTmp.sums" -UseBasicParsing
        # sha256sum.txt lista todos los assets de la release: "<hash>  <fichero>".
        $jqEsperado = $null
        foreach ($linea in (Get-Content "$jqTmp.sums")) {
            $c = $linea.Trim() -split '\s+'
            if ($c.Count -ge 2 -and ($c[-1] -replace '^\*', '') -eq $jqAsset) { $jqEsperado = $c[0]; break }
        }
        if (-not $jqEsperado) { throw "install: sha256sum.txt de jq no lista $jqAsset. NO se ha instalado nada." }
        $jqReal256 = (Get-FileHash -Path $jqTmp -Algorithm SHA256).Hash.ToLower()
        if ($jqReal256 -ne $jqEsperado.ToLower()) {
            throw "install: el SHA256 de jq no cuadra. Esperado $jqEsperado, calculado $jqReal256. NO se ha instalado nada."
        }
        Write-Host "install: sha256 de jq verificado"
    }

    if (-not (Test-Path $Dir)) { New-Item -ItemType Directory -Path $Dir -Force | Out-Null }
    $destino = Join-Path $Dir 'exo.exe'
    Copy-Item -Path $binTmp -Destination $destino -Force
    Write-Host "install: instalado en $destino"

    if (-not $jqReal) {
        $jqDestino = Join-Path $Dir 'jq.exe'
        Copy-Item -Path $jqTmp -Destination $jqDestino -Force
        & $jqDestino --version
        if ($LASTEXITCODE -ne 0) {
            throw "install: el jq instalado en $jqDestino no responde a --version (exit $LASTEXITCODE)."
        }
        Write-Host "install: jq instalado en $jqDestino"
    }

    # PATH de usuario, no de proceso: tiene que sobrevivir a esta consola. Se
    # ANTEPONE (no se añade al final) para que este jq gane al alias de la
    # Store. Idempotente: si $Dir ya está, no se reescribe.
    $pathActual = [Environment]::GetEnvironmentVariable('Path', $PathScope)
    $yaEsta = ($pathActual -split ';') -contains $Dir
    if (-not $yaEsta) {
        [Environment]::SetEnvironmentVariable('Path', "$Dir;$pathActual", $PathScope)
        Write-Host "install: $Dir antepuesto al PATH ($PathScope); abre una consola nueva para que lo vea"
    }
    if (($env:PATH -split ';') -notcontains $Dir) { $env:PATH = "$Dir;$env:PATH" }

    # El exit code de un proceso NATIVO no lanza excepcion en PowerShell, ni
    # siquiera con $ErrorActionPreference='Stop': hay que mirar $LASTEXITCODE a
    # mano. Sin esto, un binario que arranca pero devuelve != 0 dejaria la
    # instalacion "en verde" sin senal alguna — y `install.sh`, bajo
    # `set -euo pipefail`, si aborta en ese mismo caso.
    & $destino --version
    if ($LASTEXITCODE -ne 0) {
        throw "install: el binario instalado en $destino no responde a --version (exit $LASTEXITCODE)."
    }

    $cfg = if ($env:EXO_CONFIG) { $env:EXO_CONFIG } else { Join-Path $HOME '.exo\config.toml' }
    if (Test-Path $cfg) {
        Write-Host "install: config ya existente en $cfg — no se toca"
    } elseif ($env:EXO_INIT_KB -and $env:EXO_INIT_NAME) {
        # Paridad con install.sh: si el usuario da KB y nombre por entorno, se
        # encadena `exo init`. Sin eso no se inventa una KB por defecto.
        & $destino init --kb $env:EXO_INIT_KB --name $env:EXO_INIT_NAME
        if ($LASTEXITCODE -ne 0) {
            throw "install: exo init fallo (exit $LASTEXITCODE)."
        }
    } else {
        Write-Host "install: no hay config en $cfg. Créala con:"
        Write-Host "  $destino init --kb <ruta-de-tu-kb> --name <nombre>"
    }

    # doctor cierra la instalación. Su exit 3 significa "esta máquina tiene
    # deuda", no "la instalación falló": se imprime el informe y no se
    # propaga el código.
    & $destino doctor
    if ($LASTEXITCODE -ne 0) {
        Write-Host "install: doctor ha marcado deuda (exit $LASTEXITCODE) — mira las filas 'fail' de arriba"
    }

    # exo sustituye a superpowers; con los dos habilitados se duplican skills y
    # hooks. `disable`, nunca `uninstall`: se puede revertir.
    if ($DisableSuperpowers -or $env:EXO_DISABLE_SUPERPOWERS) {
        & claude plugin disable superpowers
        if ($LASTEXITCODE -ne 0) { Write-Warning "install: 'claude plugin disable superpowers' salió con $LASTEXITCODE" }
    } else {
        Write-Host "install: si tienes superpowers instalado, desactívalo (exo lo sustituye):"
        Write-Host "  claude plugin disable superpowers"
    }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
