# Gate de test-windows (spec exo-hook-codex §7.8): Codex pasa la línea de codex.json a
# `cmd /C` tal cual. Se ejecuta con cmd.exe real, no vía bash/MSYS.
# Primero exige que `exo` responda: si no, `|| exit 0` daría 0 sin probar nada.
$ErrorActionPreference = 'Stop'
$env:PATH = "$env:GITHUB_WORKSPACE\engine\target\debug;$env:PATH"
& exo --version
if ($LASTEXITCODE -ne 0) { throw "exo no responde ($LASTEXITCODE)" }
cmd.exe /C 'nope || exit 0'
if ($LASTEXITCODE -ne 0) { throw "cmd /C 'nope || exit 0' devolvió $LASTEXITCODE" }
$ev = '{"session_id":"s","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"},"cwd":"."}'
$out = $ev | cmd.exe /C 'exo hook codex PreToolUse || exit 0'
if ($LASTEXITCODE -ne 0) { throw "exo hook codex PreToolUse devolvió $LASTEXITCODE" }
if (($out -join '') -ne '{}') { throw "stdout inesperado: $out" }
Write-Host 'test-windows-cmd-hook: OK'
