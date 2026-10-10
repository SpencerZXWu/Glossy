# Answers one question about the live relay: can an old build still translate?
#
#   powershell -ExecutionPolicy Bypass -File scripts\relay.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\relay.ps1 -Endpoint https://...
#
# It asks /v1/health what the deployment says about the builds it serves, then
# sends two translations the way the two ends of the range would: one with no
# version at all, which is what every build before 2.1.1 looks like, and one from
# a build that reports a version. That is enough to tell an operator four things
# apart, which is why it exists:
#
#   * the new code is not deployed yet (health has no `version` block at all, so
#     every switch below it does nothing);
#   * the new code is deployed and old builds are being allowed through;
#   * the new code is deployed and old builds are being refused;
#   * and, from the answer a real App would get, which notices are being sent.
#
# It reads no settings and writes nothing: two requests, one of them refused or
# one short translation, booked under a client id of its own so it cannot take
# anything from a real installation.
param(
  [string]$Endpoint = "https://1492303375-cwrw0pdztr.ap-guangzhou.tencentscf.com",
  [string]$OldBuild = "",
  [string]$NewBuild = "2.1.1"
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$Endpoint = $Endpoint.TrimEnd("/")

# PowerShell 5.1 has no `??`, and an unset switch is the common case here.
function Show-Switch($Value) {
  if ([string]::IsNullOrWhiteSpace([string]$Value)) { return "（未设）" }
  return [string]$Value
}

function Send-Translate([string]$AppVersion) {
  $body = @{ clientId = "relay-check-0001"; text = "hello"; from = "en"; to = "zh" }
  if ($AppVersion) { $body["appVersion"] = $AppVersion }
  $json = $body | ConvertTo-Json -Compress
  try {
    $answer = Invoke-WebRequest -Uri "$Endpoint/v1/translate" -Method Post -Body $json `
      -ContentType "application/json" -UseBasicParsing
    return @{ Status = [int]$answer.StatusCode; Body = ($answer.Content | ConvertFrom-Json) }
  } catch {
    # Windows PowerShell throws on 4xx, which is exactly the answer being looked
    # for here, so the response is read off the exception.
    $response = $_.Exception.Response
    if (-not $response) { throw }
    $reader = New-Object IO.StreamReader($response.GetResponseStream())
    return @{ Status = [int]$response.StatusCode; Body = ($reader.ReadToEnd() | ConvertFrom-Json) }
  }
}

function Show-Result([string]$Label, $Result) {
  $body = $Result.Body
  Write-Host ("{0}  HTTP {1}" -f $Label, $Result.Status)
  if ($body.code) { Write-Host ("           code = {0}" -f $body.code) }
  if ($body.message) { Write-Host ("           message = {0}" -f $body.message) }
  if ($body.notice) {
    Write-Host ("           notice = {0}" -f ($body.notice | ConvertTo-Json -Compress))
  } elseif ($Result.Status -eq 200) {
    Write-Host "           notice = 无（没有下发升级提示或公告）"
  }
}

Write-Host "服务端：$Endpoint"
$health = Invoke-RestMethod -Uri "$Endpoint/v1/health" -Method Get
Write-Host ("health：service={0} configured={1} ocr={2}" -f $health.service, $health.configured, $health.ocr)
if ($health.version) {
  Write-Host ("        开关：blockBelow={0} minVersion={1} announcement={2}" -f `
    (Show-Switch $health.version.blockBelow), (Show-Switch $health.version.minVersion), $health.version.announcement)
} else {
  Write-Host "        开关：服务端还没有这批新代码 —— 下面三个开关现在都不起作用" -ForegroundColor Yellow
}
Write-Host ""

$old = Send-Translate $OldBuild
Show-Result "【旧版本（不发版本号）】" $old
$new = Send-Translate $NewBuild
Show-Result ("【新版 {0}】" -f $NewBuild) $new
Write-Host ""

if ($old.Status -eq 403 -and $old.Body.code -eq "version_too_old") {
  Write-Host "结论：旧版本已被拒绝，用户会看到上面那句 message。" -ForegroundColor Yellow
} elseif ($old.Status -eq 200) {
  Write-Host "结论：旧版本目前仍然可以正常翻译。"
} else {
  Write-Host ("结论：旧版本收到了意外回答（HTTP {0}，code={1}），先查这个。" -f $old.Status, $old.Body.code) -ForegroundColor Yellow
}
Write-Host "（这个脚本用 clientId=relay-check-0001 发请求，最多占用 5 个字符的每日额度，不会动真实安装的额度。）"
