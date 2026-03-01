$ErrorActionPreference = "Stop"
$stdoutLog = "$env:TEMP\aurotype-stdout.log"
$stderrLog = "$env:TEMP\aurotype-stderr.log"

# Pre-compile the type so it's ready before the app starts
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Threading;
public class KeySender {
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    [DllImport("user32.dll")] public static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);
    
    public const byte VK_CONTROL = 0x11;
    public const byte VK_MENU = 0x12;
    public const byte VK_SPACE = 0x20;
    public const uint KEYEVENTF_KEYDOWN = 0x0000;
    public const uint KEYEVENTF_KEYUP = 0x0002;
    
    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT {
        public uint type_;
        public KEYBDINPUT ki;
        public long padding;
    }
    
    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }
    
    public static void SendCtrlAltSpace() {
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        Thread.Sleep(50);
        keybd_event(VK_MENU, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        Thread.Sleep(50);
        keybd_event(VK_SPACE, 0, KEYEVENTF_KEYDOWN, UIntPtr.Zero);
        Thread.Sleep(800);
        keybd_event(VK_SPACE, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        Thread.Sleep(50);
        keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        Thread.Sleep(50);
        keybd_event(VK_CONTROL, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
    }
}
"@

Write-Host "[debug] Starting tauri-app.exe..."
$proc = Start-Process -FilePath "src-tauri\target\release\tauri-app.exe" `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -PassThru

Write-Host "[debug] Waiting 10s for full startup..."
Start-Sleep -Seconds 10

Write-Host "[debug] Simulating Ctrl+Alt+Space keypress (with delays)..."
[KeySender]::SendCtrlAltSpace()
Write-Host "[debug] Keypress sent, waiting 5s for processing..."
Start-Sleep -Seconds 5

# Send a second press to double-check
Write-Host "[debug] Sending second keypress..."
[KeySender]::SendCtrlAltSpace()
Start-Sleep -Seconds 3

if ($proc.HasExited) {
    Write-Host "[debug] Process EXITED with code: $($proc.ExitCode)"
} else {
    Write-Host "[debug] Process still running (PID: $($proc.Id)), stopping..."
    Stop-Process -Id $proc.Id -Force
}

Write-Host "--- STDERR ---"
if (Test-Path $stderrLog) { Get-Content $stderrLog } else { Write-Host "(no file)" }
