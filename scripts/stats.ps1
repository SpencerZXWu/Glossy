<#
按天记录 Glossy 服务端的报告。

服务端的 /v1/stats 是给运营者自己看的：今天用了多少字符、多少请求、多少个安装、
多少个地址、还剩多少，以及这个月识图用了几次。这个脚本把它读下来，写进一份 CSV，
一天一行；同一天再跑是覆盖那一行，不会写出两行来，所以什么时候跑都只留最新的数字。

日报在每个实例里各自清零，跑过去的日期补不回来：想要一整月的历史，要么每天跑一次
（-Register daily 会替你做这件事），要么就别指望它。

口令就是部署端设置的 ADMIN_TOKEN，不写在脚本里，也不写进计划任务：
  -Token  这次用它；
  没给时看环境变量 GLOSSY_ADMIN_TOKEN；
  再没有就看 %APPDATA%\Glossy\admin-token.txt —— 那是 -SaveToken 用 Windows 自己的
  数据保护接口（DPAPI）存下来的，只有本机这个账户解的开，别的机器、别的账户都读不出来。

用法：
  .\stats.ps1                     # 用存好的口令，记下今天的数字
  .\stats.ps1 -Token xxxxx        # 这次用这个口令
  .\stats.ps1 -SaveToken          # 提示粘贴口令，加密存起来
  .\stats.ps1 -NoCsv              # 只在屏幕上看看，不写文件
  .\stats.ps1 -Register daily     # 登记每天 23:50 自动记一次
  .\stats.ps1 -Register weekly    # 每周一 23:50 记一次
  .\stats.ps1 -Unregister         # 取消自动记录
#>

[CmdletBinding()]
param(
  # 部署的地址；默认就是这个 App 平时翻译用的那个
  [string]$Endpoint = "https://1492303375-cwrw0pdztr.ap-guangzhou.tencentscf.com",
  # 这次要用的口令，给了就不看环境变量和存下来的那份
  [string]$Token = "",
  # 记录写到哪里；默认是「文档」里的一份 CSV
  [string]$Csv = "",
  # 提示输入口令并加密存下来
  [switch]$SaveToken,
  # 只在屏幕上看看，不写文件
  [switch]$NoCsv,
  # 登记计划任务：daily 每天一次，weekly 每周一次
  [ValidateSet("none", "daily", "weekly")]
  [string]$Register = "none",
  # 取消计划任务
  [switch]$Unregister,
  [int]$TimeoutSec = 20
)

$ErrorActionPreference = "Stop"
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch { }

$TaskName = "Glossy server report"
$TokenFile = Join-Path $env:APPDATA "Glossy\admin-token.txt"
if (-not $Csv) {
  $Csv = Join-Path ([Environment]::GetFolderPath("MyDocuments")) "glossy-server-report.csv"
}

function Write-Step([string]$text) { Write-Host $text }

# 口令：先问参数，再问环境变量，最后问那份加密文件。
function Get-Token {
  if ($Token.Trim()) { return $Token.Trim() }
  if ($env:GLOSSY_ADMIN_TOKEN -and $env:GLOSSY_ADMIN_TOKEN.Trim()) { return $env:GLOSSY_ADMIN_TOKEN.Trim() }
  if (Test-Path $TokenFile) {
    $stored = (Get-Content -Path $TokenFile -Raw).Trim()
    if ($stored) {
      # ConvertFrom-SecureString 写出来的是 DPAPI 密文，换台机器就解不开，
      # 这是有意的：文件本身可以随便放，里面的东西只有这台机器的这个账户能读。
      $secure = ConvertTo-SecureString $stored
      return (New-Object System.Net.NetworkCredential("", $secure)).Password
    }
  }
  return ""
}

function Save-Token {
  $secure = Read-Host -AsSecureString "粘贴管理员口令（ADMIN_TOKEN），不回显"
  $plain = (New-Object System.Net.NetworkCredential("", $secure)).Password
  if (-not $plain.Trim()) { throw "没有输入口令。" }
  $folder = Split-Path -Parent $TokenFile
  if (-not (Test-Path $folder)) { New-Item -ItemType Directory -Path $folder -Force | Out-Null }
  $secure | ConvertFrom-SecureString | Set-Content -Path $TokenFile -Encoding ASCII
  Write-Step "口令已加密存到 $TokenFile（只有本机这个账户能解开）。"
}

# 读一份报告。失败时把服务端说的那句话尽量带出来，看不懂也至少说清是哪一步。
function Read-Report {
  $uri = "$($Endpoint.TrimEnd('/'))/v1/stats"
  try {
    return Invoke-RestMethod -Method Get -Uri $uri -TimeoutSec $TimeoutSec -Headers @{
      Authorization = "Bearer $script:AdminToken"
    }
  } catch {
    $response = $_.Exception.Response
    $detail = ""
    if ($response) {
      try {
        $reader = New-Object System.IO.StreamReader($response.GetResponseStream())
        $body = $reader.ReadToEnd()
        $reader.Dispose()
        $parsed = $body | ConvertFrom-Json
        $detail = [string]$parsed.message
      } catch { $detail = "" }
    }
    $status = 0
    if ($response) { $status = [int]$response.StatusCode }
    if ($status -eq 403) {
      throw "服务端不接受这个口令。要么口令不对，要么部署那边还没有设置 ADMIN_TOKEN（没设置时这个接口是关的）。$detail"
    }
    if ($status -eq 404) { throw "服务端没有 /v1/stats 这个接口，可能是那边还没更新到这一版。" }
    if ($status) { throw "服务端返回 HTTP $status。$detail" }
    throw "读不到 $uri ：$($_.Exception.Message)"
  }
}

function Format-Number($value) { "{0:N0}" -f [double]$value }

function Set-Number($value, $total) {
  if (-not $total -or [double]$total -le 0) { return (Format-Number $value) }
  $share = [double]$value / [double]$total * 100
  return "$(Format-Number $value) / $(Format-Number $total)（$([Math]::Round($share))%）"
}

function Write-Summary($report) {
  $today = $report.today
  $limits = $report.limits
  $readings = $report.readings

  Write-Step ""
  Write-Step "Glossy 服务端 · $($report.day)（$($report.service)）"
  Write-Step "  地址：$($Endpoint.TrimEnd('/'))"
  Write-Step "  今天：$(Set-Number $today.chars $limits.charsTotal) 字符，$($today.requests) 次请求"
  if ($null -ne $today.remaining) {
    Write-Step "  还能用：$(Format-Number $today.remaining) 字符（整天共用的一份）"
  } else {
    Write-Step "  整天没有上限，所以没有「还剩多少」这回事。"
  }
  Write-Step "  来过：$($today.installations) 个安装，$($today.addresses) 个地址"
  Write-Step "  最近一分钟：$($today.rate.requests) 次请求（上限每分钟 $($limits.requestsPerMinute) 次，按地址算）"

  $busiest = @($today.busiestInstallations)
  if ($busiest.Count) {
    Write-Step "  用得最多的安装："
    foreach ($row in $busiest) {
      $name = if ($row.rolled) { "其余 $($row.rolled) 个" } else { [string]$row.id }
      Write-Step ("    {0,-22} {1,12} 字符 {2,6} 次" -f $name, (Format-Number $row.chars), $row.requests)
    }
  }

  Write-Step "  这个月识图：$($readings.total) 次，$($readings.installations) 个安装用过"
  if ($readings.perInstallation) {
    Write-Step "  （每个安装每月上限 $($readings.perInstallation) 次，是上游按次计费的额度）"
  }
  Write-Step ""
  Write-Step "  报告生成于 $($report.generatedAt)"
}

# 一天一行；同一天再跑就是覆盖那一行，不是再加一行。
function Write-Csv($report) {
  $today = $report.today
  $line = [pscustomobject]@{
    day           = [string]$report.day
    chars         = [string]$today.chars
    requests      = [string]$today.requests
    installations = [string]$today.installations
    addresses     = [string]$today.addresses
    remaining     = [string]$today.remaining
    limit         = [string]$report.limits.charsTotal
    readings      = [string]$report.readings.total
    readAt        = [string]$report.generatedAt
  }

  $kept = @()
  if (Test-Path $Csv) {
    $kept = @(Import-Csv -Path $Csv | Where-Object { $_.day -ne $line.day })
  }
  $all = @(@($kept) + @($line) | Sort-Object day)
  $where = Split-Path -Parent $Csv
  if ($where -and -not (Test-Path $where)) { New-Item -ItemType Directory -Path $where -Force | Out-Null }
  $all | Export-Csv -Path $Csv -NoTypeInformation -Encoding UTF8
  Write-Step "已记下 $($line.day) 这一行（共 $($all.Count) 天）：$Csv"
}

function Register-Task {
  $exe = (Get-Process -Id $PID).Path
  if (-not $exe) { $exe = "powershell.exe" }
  $arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  if ($Csv) { $arguments += " -Csv `"$Csv`"" }
  $action = New-ScheduledTaskAction -Execute $exe -Argument $arguments
  $at = (Get-Date -Hour 23 -Minute 50 -Second 0)
  if ($Register -eq "weekly") {
    $trigger = New-ScheduledTaskTrigger -Weekly -DaysOfWeek Monday -At $at
    $when = "每周一 23:50"
  } else {
    $trigger = New-ScheduledTaskTrigger -Daily -At $at
    $when = "每天 23:50"
  }
  $settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -AllowStartIfOnBatteries
  Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings `
    -Description "把 Glossy 服务端当天的用量记进 CSV。" -Force | Out-Null
  Write-Step "已登记自动记录：$when（任务名「$TaskName」）。"
  Write-Step "  跑的是 $PSCommandPath"
  Write-Step "  口令从 $TokenFile 读，任务里没有口令。"
}

function Unregister-Task {
  $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
  if (-not $task) { Write-Step "没有登记过「$TaskName」这个任务，不用取消。"; return }
  Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
  Write-Step "已取消「$TaskName」。"
}

# 计划任务模式下不问口令、不打印：屏幕上没人看，读不到就当这次没跑。
if ($Unregister) { Unregister-Task; return }
if ($SaveToken) { Save-Token; return }

$script:AdminToken = Get-Token
if (-not $script:AdminToken) {
  Write-Step "还没有口令。先跑一次 .\stats.ps1 -SaveToken 把 ADMIN_TOKEN 存下来，或者这次用 -Token 给一个。"
  exit 1
}

# 读不到就只说一句人话再退出：这个脚本也会被计划任务跑，那边没人看堆栈。
try {
  $report = Read-Report
} catch {
  Write-Step $_.Exception.Message
  exit 1
}

Write-Summary $report
if (-not $NoCsv) { Write-Csv $report }
if ($Register -ne "none") { Register-Task }
