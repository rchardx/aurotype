# Start Vite dev server bound to 127.0.0.1 for Playwright access
$env:TAURI_DEV_HOST = "127.0.0.1"
$proc = Start-Process -FilePath "C:\Users\rchar\.bun\bin\bun.exe" `
    -ArgumentList "run","dev","--","--host","127.0.0.1" `
    -WorkingDirectory "C:\Users\rchar\aurotype" `
    -WindowStyle Hidden `
    -RedirectStandardOutput "C:\Users\rchar\aurotype\vite-dev.log" `
    -RedirectStandardError "C:\Users\rchar\aurotype\vite-dev-err.log" `
    -PassThru
Write-Host "PID: $($proc.Id)"

# Wait for server to be ready
for ($i = 0; $i -lt 10; $i++) {
    Start-Sleep -Seconds 2
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:1420" -UseBasicParsing -TimeoutSec 3
        Write-Host "Vite dev server ready! Status: $($r.StatusCode)"
        exit 0
    } catch {
        Write-Host "Waiting... ($($i+1))"
    }
}
Write-Host "ERROR: Vite dev server did not start in time"
exit 1
