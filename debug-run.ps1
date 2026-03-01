$ErrorActionPreference = "Stop"
$stdoutLog = "$env:TEMP\aurotype-stdout.log"
$stderrLog = "$env:TEMP\aurotype-stderr.log"

Write-Host "[debug] Starting tauri-app.exe..."
$proc = Start-Process -FilePath "src-tauri\target\release\tauri-app.exe" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -PassThru

Start-Sleep -Seconds 8

if ($proc.HasExited) {
    Write-Host "[debug] Process EXITED with code: $($proc.ExitCode)"
} else {
    Write-Host "[debug] Process still running (PID: $($proc.Id)), stopping..."
    Stop-Process -Id $proc.Id -Force
}

Write-Host "--- STDOUT ---"
if (Test-Path $stdoutLog) { Get-Content $stdoutLog } else { Write-Host "(no file)" }
Write-Host "--- STDERR ---"
if (Test-Path $stderrLog) { Get-Content $stderrLog } else { Write-Host "(no file)" }
