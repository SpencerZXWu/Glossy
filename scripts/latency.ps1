# Measures the two numbers of the v1.4.0 performance budget on a release build:
# the idle cost (CPU and memory with the popup closed) and the latency from the
# mouse-up that ends a selection drag to the moment the card is painted.
#
#   powershell -ExecutionPolicy Bypass -File scripts\latency.ps1
#
# The build must exist first: cargo build --release (the script refuses to run
# against anything else, because a debug number is not a number anyone runs).
# It refuses to start while another glossy.exe is running - the second instance
# would only show "already running" and exit - and it stops the instance it
# started when it is done. Nothing else is touched.
param(
  [string]$Exe = (Join-Path $PSScriptRoot "..\src-tauri\target\release\glossy.exe"),
  [int]$Drags = 12,
  [int]$IntervalMs = 1400,
  [int]$IdleSeconds = 20,
  [switch]$Keep
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Exe)) {
  Write-Host "No release build at $Exe"
  Write-Host "Build it first: cargo build --release"
  exit 1
}
$Exe = (Resolve-Path $Exe).Path

$running = @(Get-Process -Name glossy -ErrorAction SilentlyContinue)
if ($running.Count -gt 0) {
  Write-Host "Glossy is already running (PID $($running.Id -join ', ')). Quit it from the notification area"
  Write-Host "first - this script must own the single instance it measures."
  exit 1
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class Inject {
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  public const uint MOVE = 0x0001, ABSOLUTE = 0x8000, LEFTDOWN = 0x0002, LEFTUP = 0x0004;
  static uint NX(int x) { return (uint)Math.Round(x * 65535.0 / (GetSystemMetrics(0) - 1)); }
  static uint NY(int y) { return (uint)Math.Round(y * 65535.0 / (GetSystemMetrics(1) - 1)); }
  public static void At(int x, int y) { mouse_event(MOVE | ABSOLUTE, NX(x), NY(y), 0, UIntPtr.Zero); }
  public static void Down() { mouse_event(LEFTDOWN, 0, 0, 0, UIntPtr.Zero); }
  public static void Up() { mouse_event(LEFTUP, 0, 0, 0, UIntPtr.Zero); }
  public static void Click(int x, int y) { At(x, y); System.Threading.Thread.Sleep(40); Down(); System.Threading.Thread.Sleep(40); Up(); }
  public static void Drag(int x1, int y1, int x2, int y2) {
    At(x1, y1); System.Threading.Thread.Sleep(50); Down();
    for (int i = 1; i <= 8; i++) { At(x1 + (x2 - x1) * i / 8, y1 + (y2 - y1) * i / 8); System.Threading.Thread.Sleep(30); }
    Up();
  }
}
'@

# The app prints its samples to GLOSSY_TIMING_LOG as well as to stderr: it is a
# GUI-subsystem binary, so a console started from a script does not stay alive
# to show them.
$log = Join-Path $env:TEMP "glossy-latency.log"
Remove-Item $log -ErrorAction SilentlyContinue
$env:GLOSSY_TIMING = "1"
$env:GLOSSY_TIMING_LOG = $log

Write-Host "Starting $Exe"
$app = Start-Process -FilePath $Exe -PassThru

try {
  Start-Sleep -Seconds 3   # hook installed, tray icon up, nothing else happening

  # Idle cost: the popup closed and the hook listening is exactly this state.
  Write-Host "Sampling idle cost for $IdleSeconds s ..."
  $app.Refresh()
  $t1 = $app.TotalProcessorTime
  $w1 = $app.WorkingSet64
  Start-Sleep -Seconds $IdleSeconds
  $app.Refresh()
  $t2 = $app.TotalProcessorTime
  $w2 = $app.WorkingSet64
  $cpuPct = (($t2 - $t1).TotalMilliseconds) / ($IdleSeconds * 1000 * [Environment]::ProcessorCount) * 100
  Write-Host ("idle cpu    : {0:N2} % over {1} s" -f $cpuPct, $IdleSeconds)
  Write-Host ("idle memory : {0:N1} MB working set ({1:N1} MB private)" -f ($w2 / 1MB), ($app.PrivateMemorySize64 / 1MB))

  # Latency: a real drag over real text, injected so that every run looks the
  # same. One click away from the text first, to take the previous badge down.
  Write-Host "Injecting $Drags selection drags ..."
  $form = New-Object System.Windows.Forms.Form
  $form.Text = "Glossy latency host"
  $form.StartPosition = "Manual"
  $form.Location = New-Object System.Drawing.Point(200, 200)
  $form.ClientSize = New-Object System.Drawing.Size(960, 420)
  $box = New-Object System.Windows.Forms.TextBox
  $box.Multiline = $true
  $box.Dock = "Fill"
  $box.ScrollBars = "Vertical"
  $box.Font = New-Object System.Drawing.Font("Consolas", 16)
  $box.Text = "The quick brown fox jumps over the lazy dog and keeps on running down the road." + [char]13 + [char]10 +
              "A second line stands here so that the popup has something to cover." + [char]13 + [char]10 +
              "And a third one, because a selection needs neighbours to be picked out of."
  $form.Controls.Add($box)
  $form.TopMost = $true
  $form.Add_Shown({
      $form.Activate()
      $box.Focus()
      # Injected drags go to whatever is in front, so a host that never took the
      # foreground would measure the wrong window and sample nothing at all.
      if ([Inject]::GetForegroundWindow() -ne $form.Handle) {
        Write-Host "warning: the host window is not in the foreground - run this from a normal"
        Write-Host "         desktop session, not from a background one."
      }
    })

  $drag = {
    $top = $box.PointToScreen($box.GetPositionFromCharIndex(0))
    $lineY = $top.Y + 14
    [Inject]::Click($top.X + 5, $top.Y + $box.ClientSize.Height - 30)
    Start-Sleep -Milliseconds 150
    [Inject]::Drag($top.X + 5, $lineY, $top.X + 260, $lineY)
  }

  # The first drag lands while the window is still settling and is regularly the
  # one that does not copy anything, so it is thrown away instead of being read
  # as a lost sample.
  $script:warm = $true
  $script:done = 0
  $timer = New-Object System.Windows.Forms.Timer
  $timer.Interval = $IntervalMs
  $timer.Add_Tick({
      if ($script:warm) {
        $script:warm = $false
        & $drag
        Remove-Item $log -ErrorAction SilentlyContinue
        return
      }
      & $drag
      $script:done++
      if ($script:done -ge $Drags) {
        $timer.Stop()
        $form.Close()
      }
    })
  $timer.Start()
  [void]$form.ShowDialog()
  Start-Sleep -Milliseconds 300

  if (-not (Test-Path $log)) {
    Write-Host "No samples were recorded. Glossy has to be enabled, tracking a selection"
    Write-Host "has to be on, and the selection has to be long enough to count; the script"
    Write-Host "also has to run on the interactive desktop, where a drag can reach the app."
  }
  else {
    $samples = @(Get-Content $log | ForEach-Object {
        if ($_ -match "latency\s+([\d.]+) ms") { [double]$matches[1] }
      })
    Write-Host ""
    Write-Host "--- samples ($($samples.Count) of $Drags drags) ---"
    Get-Content $log
    if ($samples.Count -gt 0) {
      $sorted = $samples | Sort-Object
      $at = { param($q) $sorted[[Math]::Min($sorted.Count - 1, [Math]::Floor($q * ($sorted.Count - 1)))] }
      Write-Host ""
      Write-Host ("latency     : p50 {0:N1} ms, p95 {1:N1} ms, max {2:N1} ms" -f
        (& $at 0.5), (& $at 0.95), $sorted[-1])
    }
  }
}
finally {
  if (-not $Keep) {
    if (-not $app.HasExited) { Stop-Process -Id $app.Id }
    Write-Host "Glossy stopped."
  }
}

Write-Host ""
Write-Host "--- machine ---"
$os = Get-CimInstance Win32_OperatingSystem
$cs = Get-CimInstance Win32_ComputerSystem
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$real = Get-CimInstance Win32_VideoController |
  Where-Object { $_.CurrentHorizontalResolution } |
  Select-Object -First 1
$scale = 100
if ($real) { $scale = [Math]::Round($real.CurrentHorizontalResolution / $screen.Width * 100) }
Write-Host ("{0} / {1:N0} GB RAM / {2}" -f
  $cpu.Name.Trim(), ($cs.TotalPhysicalMemory / 1GB), "$($os.Caption) build $($os.BuildNumber)")
Write-Host ("{0}x{1} at {2} %" -f
  $(if ($real) { $real.CurrentHorizontalResolution } else { $screen.Width }),
  $(if ($real) { $real.CurrentVerticalResolution } else { $screen.Height }), $scale)
Write-Host "Paste the lines above into the performance section of README.md."
