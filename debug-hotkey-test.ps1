$ErrorActionPreference = "Stop"
$stdoutLog = "$env:TEMP\aurotype-stdout.log"
$stderrLog = "$env:TEMP\aurotype-stderr.log"

Write-Host "[debug] Starting tauri-app.exe..."
$proc = Start-Process -FilePath "src-tauri\target\release\tauri-app.exe" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -PassThru

Write-Host "[debug] Waiting 8s for startup..."
Start-Sleep -Seconds 8

Write-Host "[debug] Simulating Ctrl+Alt+Space keypress..."
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class KeySender {
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    public const byte VK_CONTROL = 0x11;
    public const byte VK_MENU = 0x12;    // Alt
    public const byte VK_SPACE = 0x20;
    public const uint KEYEVENTF_KEYDOWN = 0x0000;
    public const uint KEYEVENTF_KEYUP = 0x0002;
    
    public static void SendCtrlAltSpace() {
        // Press Ctrl, Alt, Space
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        keybd_event(VK_MENU, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        keybd_event(VK_SPACE, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        System.Threading.Thread.Sleep(500);
        // Release Space, Alt, Ctrl
        keybd_event(VK_SPACE, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
    }
}
"@

[KeySender]::SendCtrlAltSpace()
Write-Host "[debug] Keypress sent, waiting 5s..."
Start-Sleep -Seconds 5

if ($proc.HasExited) {
    Write-Host "[debug] Process EXITED with code: $($proc.ExitCode)"
} else {
    Write-Host "[debug] Process still running (PID: $($proc.Id)), stopping..."
    Stop-Process -Id $proc.Id -Force
}

Write-Host "--- STDERR ---"
if (Test-Path $stderrLog) { Get-Content $stderrLog } else { Write-Host "(no file)" }
