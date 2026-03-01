try {
    $r = Invoke-WebRequest -Uri "http://localhost:1420" -UseBasicParsing -TimeoutSec 5
    Write-Host "STATUS: $($r.StatusCode)"
} catch {
    Write-Host "ERROR: $($_.Exception.Message)"
}
