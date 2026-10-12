# 对同一个安装 id 连问几次额度，看答复是不是同一份。
#
#   powershell -ExecutionPolicy Bypass -File scripts\quota-check.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\quota-check.ps1 -Rounds 20
#
# 存在的理由：云函数是会被复制的，每个实例各记各的。一个部署如果没接共享存储
# （Redis），同一秒钟问两次额度可能得到两组不同的数字——用户看到的就是"刷新几次
# 又变满"。这个脚本不猜，它把两种形状都打出来，并且**只要出现过一次不一致就以
# 非零码退出**，所以能直接放进"改完部署先跑一下"的流程里。
#
# 它只读 /v1/health 和 /v1/quota，不花字符额度（读额度不记账），也不动任何计数。
param(
  # 部署的地址；默认就是这个 App 平时翻译用的那个
  [string]$Endpoint = "https://1492303375-crw0pdztr.ap-guangzhou.tencentscf.com",
  # 用来问的安装 id；默认是一个明显不属于任何真机的 id
  [string]$Client = "quota-check-0000000000000000",
  # 问几次
  [int]$Rounds = 10,
  # 两次之间停多久（毫秒）
  [int]$PauseMs = 300
)

$ErrorActionPreference = "Stop"
# 云函数只认 TLS 1.2 及以上，而 Windows PowerShell 5.1 的默认值不一定包含它。
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch { }

$Endpoint = $Endpoint.TrimEnd("/")
Write-Host "部署：$Endpoint"

# 先问一次健康检查：它顺带说明这套部署的计数是不是整个部署共用的。
$shared = "不知道（这套部署还没有这个字段）"
try {
  $health = Invoke-RestMethod -Uri "$Endpoint/v1/health" -Method Get -TimeoutSec 15
  if ($health.counters -and $null -ne $health.counters.shared) {
    $shared = if ($health.counters.shared) { "true（整个部署共用一份）" } else { "false（按实例计数）" }
  }
} catch {
  Write-Host "读不到 /v1/health：$($_.Exception.Message)" -ForegroundColor Yellow
}
Write-Host "counters.shared：$shared"
Write-Host ""

$shapes = @{}
$failed = 0
for ($round = 1; $round -le $Rounds; $round += 1) {
  # 一次请求失败不等于计数不一致：这个域名是 CNAME 到区域入口，A 记录 TTL 只有
  # 两秒，偶尔就会解析空一次。所以每个回合给自己三次机会，三次都不行才算失败，
  # 而且失败要单独报出来——把"没连上"当成一种"形状"是最糟的处理方式。
  $problem = ""
  for ($attempt = 1; $attempt -le 3; $attempt += 1) {
    try {
      $answer = Invoke-RestMethod -Uri "$Endpoint/v1/quota?client=$Client" -Method Get -TimeoutSec 15
      $problem = ""
      break
    } catch {
      $problem = $_.Exception.Message
      Start-Sleep -Milliseconds 400
    }
  }
  if ($problem) {
    $failed += 1
    # .NET 对"解析不出来"和"连不上"给的是同一句含糊的话，所以这里补一句人话，
    # 免得把网络问题当成额度问题去查。这个域名是 CNAME 到区域入口，A 记录的 TTL
    # 只有两秒，解析偶尔会空一次，重试通常就好。
    $hint = ""
    if ($problem -match "基础连接已经关闭|timed out|远程名称|resolve|resolve" ) {
      $hint = "（多半是解析或网络，不是 TLS：先试 curl $Endpoint/v1/health）"
    }
    Write-Host ("{0,3}. 连不上（试了三次）：{1}{2}" -f $round, $problem, $hint) -ForegroundColor Yellow
    continue
  }
  $usage = $answer.usage
  $shape = "client=$($usage.client) ip=$($usage.ip) total=$($usage.total) remaining=$($answer.remaining) ocrMonth=$($usage.ocrMonth)"
  if (-not $shapes.ContainsKey($shape)) { $shapes[$shape] = 0 }
  $shapes[$shape] += 1
  Write-Host ("{0,3}. {1}" -f $round, $shape)
  if ($round -lt $Rounds) { Start-Sleep -Milliseconds $PauseMs }
}

$answered = $Rounds - $failed
Write-Host ""
if ($answered -eq 0) {
  Write-Host "$Rounds 个回合一个都没连上——这不是「额度不一致」，是根本问不到部署。" -ForegroundColor Red
  Write-Host "先确认地址、网络和函数是否在跑（curl $Endpoint/v1/health 也要能通）。" -ForegroundColor Red
  exit 2
}
if ($shapes.Count -eq 1) {
  Write-Host "$answered 次答复完全一致。" -ForegroundColor Green
  if ($failed -gt 0) {
    Write-Host "但有 $failed 个回合连不上，所以这次只算半个结论，不能就此认定计数是共享的。" -ForegroundColor Yellow
  }
  if ($shared -like "false*" -or $shared -like "不知道*") {
    Write-Host "这套部署自己说它是按实例计数的，所以上面那次「一致」只是这几次恰好落到同一台实例。" -ForegroundColor Yellow
    Write-Host "要让它真的准，就得接共享存储（见 server/README.md 的「共享计数」一节）。" -ForegroundColor Yellow
  }
  if ($failed -gt 0) { exit 1 }
  exit 0
}

Write-Host "$answered 次答复出现了 $($shapes.Count) 种形状：" -ForegroundColor Yellow
foreach ($shape in $shapes.Keys) {
  Write-Host ("  {0,3} 次  {1}" -f $shapes[$shape], $shape)
}
Write-Host ""
Write-Host "不同形状说明这些数字不是一份：每台实例各记各的，谁接到请求就用谁那份。" -ForegroundColor Yellow
Write-Host "解法是让所有实例共用一份计数——server/README.md 的「共享计数」一节。" -ForegroundColor Yellow
exit 1
