# Glossy 云端翻译服务

给你自己用的翻译代理：上游的密钥（大模型或百度）**只放在服务端**，Glossy 客户端不再需要让用户填密钥，直接调这个服务就能翻译。这就是客户端「翻译渠道」里的 **Glossy 翻译**。

上游可以配两个，配了就都留着：

| 上游 | 需要什么 | 说明 |
| --- | --- | --- |
| 大模型（OpenAI 兼容接口） | `LLM_API_KEY` | 翻译质量好，按量计费；先试它 |
| 百度翻译 | `BAIDU_APP_ID` + `BAIDU_KEY` | 认证版每月 100 万字符免费额度 |

大模型失败了（连不上、限流、额度用尽）会自动交回百度，用户那边不会看到报错。只配一个也能跑，一个都不配时 `/v1/translate` 返回 `not_configured`。

一套代码（`src/`）可以部署到两个地方：

| 部署方式 | 国内能否直连 | 要不要买域名 | 说明 |
| --- | --- | --- | --- |
| **腾讯云 SCF（Web 函数）** | ✅ 直连 | 不用，控制台自带公网地址 | **推荐** |
| Cloudflare Worker | ❌ `*.workers.dev` 被 SNI 阻断 | 要 | 代码已可用，留作备用 |

两边都实现了同一套额度规则（每台设备、每个 IP、全局每天的上限），计数逻辑写在
`src/policy.js` 里，两个平台共用。

---

## 一、准备

需要：

- **腾讯云账号并完成实名认证**（部署 SCF 用；个人认证即可，不花钱）
- 至少一个上游的密钥：大模型的 API Key，或百度翻译开放平台的 APP ID 和密钥
- Node.js 18 以上（本机已有）

## 二、部署到腾讯云 SCF（推荐，国内直连）

SCF 的「Web 函数」就是一个跑在云上的 Node 服务，控制台会白送一个国内能直连的访问地址，
**不需要买域名，也不需要备案**。

### 1. 打包

```bash
cd server
npm install
npm run build:scf
```

生成三个文件：

```
dist/scf/index.mjs        整个服务（一个文件，23 KB 左右）
dist/scf/scf_bootstrap    SCF 要求的启动脚本
dist/scf/package.json     {"type":"module"}
```

### 2. 新建函数

腾讯云控制台 → **云函数 SCF → 函数服务 → 新建**：

| 配置项 | 选什么 |
| --- | --- |
| 创建方式 | 从头开始 / 模板创建都行 |
| 函数类型 | **Web 函数** |
| 运行环境 | **Node.js 18.15**（更高版本没有的话就用这个） |
| 提交方法 | 在线编辑器（把 `dist/scf/index.mjs` 的内容整个粘进去）或上传 zip |
| 内存 / 超时 | 内存 128MB 即可，**超时时间改成 10 秒** |

> [!IMPORTANT]
> **超时时间一定要改。** SCF 默认只给 3 秒，而百度翻译一次往返经常要 0.5~2 秒（大模型更慢），
> 慢的时候直接用超时把请求掐掉。改成 10 秒最稳，用大模型的话建议 20 秒。

> [!TIP]
> **优先用「在线编辑器」直接粘贴 `index.mjs`**，不用上传 zip。因为在 Windows 上用
> PowerShell 的 `Compress-Archive` 打包会丢掉 `scf_bootstrap` 需要的 Unix 可执行权限（755），
> 传上去会报 `PortBindingFailed` 之类的错。非要用 zip 的话，就用控制台的「上传文件夹」并
> 在下面的启动命令里填好内容。

### 3. 填启动命令（关键）

进入 **函数配置 → 高级配置 → 启动命令**，把 `dist/scf/scf_bootstrap` 的内容原样填进去：

```bash
#!/bin/bash
export PORT=9000
export STATE_FILE=/tmp/glossy-cloud-quota.json
exec /var/lang/node18/bin/node index.mjs
```

几个要点，错了就直接起不来：

- 必须监听 **`0.0.0.0:9000`**（上面已经指定好了，别改成 127.0.0.1，也别改端口）；
- 启动脚本文件必须**恰好叫 `scf_bootstrap`**，名字不能变；
- 换行必须是 **LF**（Windows 记事本另存的 CRLF 会让它执行失败）；
- 函数实例里**只有 `/tmp` 能写**，所以计数文件放在 `/tmp`。

### 4. 配环境变量

**函数配置 → 环境变量**，至少配一组上游密钥（这就是密钥不放进代码里、客户端也不用填的原因）：

| 变量名 | 值 |
| --- | --- |
| `BAIDU_APP_ID` | 你的百度 APP ID |
| `BAIDU_KEY` | 你的百度密钥 |
| `LLM_API_KEY` | 大模型的 API Key（可选，配了就优先用它） |
| `LLM_ENDPOINT` | 大模型接口地址（可选，默认智谱 `https://open.bigmodel.cn/api/paas/v4/chat/completions`） |
| `LLM_MODEL` | 模型名（可选，默认 `glm-4-flash`） |
| `IP_SALT` | 随便一串长一点的随机字符，用来给 IP 做哈希 |

额度参数也可以在同一个地方配（不配就是用下面的默认值）：`DAILY_CHARS_PER_CLIENT`、
`DAILY_CHARS_PER_IP`、`DAILY_CHARS_TOTAL`、`MAX_CHARS_PER_REQUEST`、`MAX_REQUESTS_PER_MINUTE`。

### 5. 配触发器，拿到地址

**触发管理 → 创建触发器 → 函数 URL**，认证方式选 **免认证**（否则客户端调不通）。
创建完控制台会显示一个默认访问地址。

先自测一下：

```bash
curl https://<控制台给的地址>/v1/health
# {"ok":true,"service":"glossy-cloud","day":"2026-09-20","configured":true}
```

`configured: true` 就说明密钥配好了。

然后把这个地址填进 Glossy 设置里的「服务器地址」，**只填到域名，不要带 `/v1/translate`**。

> [!NOTE]
> **关于计数**：SCF 上没有数据库，计数存在 `/tmp` 的文件里，实例被冻结/重启后能恢复，
> 但**同时存在多个实例时各算各的**，所以真实的全局上限可能被放大到「实例数 × 设置值」。
> 个人自用一般只有一个实例，问题不大；`DAILY_CHARS_TOTAL` 的默认值是按百度认证版
> 100 万字符/月 摊到每天来定的，额度更小就再往保守了配。函数 URL 会把真实客户端地址带过来（`X-Forwarded-For` 末尾两段是网关自己
> 追加的边缘节点和内网跳转），所以 `DAILY_CHARS_PER_IP` 能正常按每个网络分桶；如果你的网关
> 拿不到客户端地址，这一项会退化成所有人共用一个桶——设成 `0` 可以关掉这一项检查。

## 三、部署到 Cloudflare Worker（备用）

Cloudflare Workers 免费版每天 10 万次请求，功能够用，但 `*.workers.dev` 在国内被阻断（见下文）。

在 `server` 目录下执行：


```bash
cd server
npm install

npx wrangler login          # 浏览器里点一下授权

# 密钥：依次执行，粘贴后回车即可（输入的内容不会显示）
npx wrangler secret put BAIDU_APP_ID     # 百度 APP ID
npx wrangler secret put BAIDU_KEY        # 百度密钥
# npx wrangler secret put LLM_API_KEY    # 可选：大模型的 Key，配了会优先用它
npx wrangler secret put IP_SALT          # 随便一串随机字符，用来给 IP 做哈希

npx wrangler deploy
```

`deploy` 成功后会打印一个地址，形如：

```
https://glossy-cloud.<你的子域>.workers.dev
```

这就是客户端的服务地址。之后客户端里选择「云端翻译」即可。

> [!IMPORTANT]
> **`*.workers.dev` 在中国大陆被 SNI 阻断，不是 DNS 污染，改 hosts 无效。**
>
> 症状：客户端报「无法连接到 Glossy 翻译服务器」，`Resolve-DnsName` 得到 `128.242.245.29`
> 这类污染地址，即使用真实 IP 强制 TLS 也报证书/SSL 错误。同一台机器访问
> `api.cloudflare.com`（wrangler 用的）却是正常的——所以「wrangler 能部署」并不能说明客户端能连上。
>
> **解决办法**（二选一）：
>
> - **改用腾讯云 SCF**，见本文档第二章。国内直连，不用买域名，代码同一套。**推荐这个**。
> - 给 Worker 绑一个自己的域名：
>   1. 买一个便宜域名（`.top` / `.xyz` 首年十几块），把域名的 NS 改成 Cloudflare 分配的两个
>      nameserver（Cloudflare 里「添加站点」会告诉你），免费版即可。
>   2. 等 Cloudflare 显示域名已激活，进入 **Workers & Pages → glossy-cloud → Settings →
>      Domains & Routes → Add → Custom Domain**，填 `api.你的域名.com`。Cloudflare 会自动加
>      DNS 记录和证书，一两分钟生效。
>   3. 客户端「服务器地址」改成 `https://api.你的域名.com`（**不要带 `/v1/translate`**）。
>
> 自定义域名的 SNI 是你自己的域名，不会被阻断；Cloudflare 免费版在国内通常能通，延迟
> 200~400ms，对翻译来说够用。

> 密钥不要写进 `wrangler.toml`，也不要用 `[vars]` —— 用 `secret put` 上传的才是加密存储的。配置文件里的 `[vars]` 只放额度参数。

## 四、额度怎么配

Cloudflare 上在 `wrangler.toml` 的 `[vars]` 里改（改完重新 `npx wrangler deploy`）；
腾讯云 SCF 上在控制台的「环境变量」里改（改完保存即生效）。名字和默认值两边一样：

| 参数 | 默认值 | 含义 |
| --- | --- | --- |
| `DAILY_CHARS_PER_CLIENT` | 20000 | 单个安装（一台设备）每天可翻译的字符数 |
| `DAILY_CHARS_PER_IP` | 30000 | 同一 IP 每天的上限，挡住「换个安装 ID 继续刷」 |
| `DAILY_CHARS_TOTAL` | 30000 | **全局每天上限**，真正的保险丝（≈ 百度认证版 100 万字符/月 的日均值）|
| `MAX_CHARS_PER_REQUEST` | 2000 | 单次请求字符上限 |
| `MAX_REQUESTS_PER_MINUTE` | 30 | 同一 IP 每分钟请求数上限 |

全局上限怎么定：**上游的月度字符额度 ÷ 30**，这样就算每天把额度用满，也刚好撑一个月而不会提前烧完（百度认证版 100 万字符/月 → `30000`）。额度更小就按比例往下压。换成按量计费的大模型时，这个上限直接等于**每天最多花多少钱**，定之前先算一下单价。

计数按 UTC 零点归零（北京时间早上 8 点）。

另外百度标准版是**每秒 1 次**的 QPS，连点两下就容易被判「调用过于频繁」。服务端遇到这种临时性错误会自动重试两次（0.6s、1.4s 后），所以 App 上一般不会看到这个报错。

## 五、接口

服务端不给浏览器页面用，因此**没有 CORS 头**，只有桌面客户端能调。

路径按**最后两段**匹配，所以腾讯云函数 URL 那种自带前缀的地址（`/release/v1/translate`）也能直接用。

### `GET /v1/health`

```json
{ "ok": true, "service": "glossy-cloud", "day": "2025-09-01", "configured": true }
```

`configured: false` 说明一个上游密钥都没配好。

### `GET /v1/quota?client=<安装ID>`

返回今天这个客户端的用量、剩余字符和各项上限。

### `POST /v1/translate`

```json
{ "clientId": "8-64位字母数字_-", "text": "被翻译的原文", "from": "auto", "to": "zh" }
```

成功：

```json
{
  "ok": true,
  "from": "en",
  "to": "zh",
  "translation": "你好",
  "chars": 5,
  "usage": { "client": 5, "remaining": 19995 }
}
```

失败统一是 `{ "ok": false, "code": "...", "message": "..." }`，HTTP 状态码配合：

| code | 状态 | 说明 |
| --- | --- | --- |
| `invalid_request` | 400 | 参数不对 |
| `unsupported_language` | 400 | 语言方向不支持 |
| `too_long` | 413 | 超过单次字符上限 |
| `rate_limited` | 429 | 每分钟请求太多，`retryAfter` 是建议等待秒数 |
| `client_quota_exceeded` | 429 | 这台设备今天用完了 |
| `ip_quota_exceeded` | 429 | 这个网络今天用完了 |
| `global_quota_exceeded` | 429 | 全局额度用完了 |
| `not_configured` | 503 | 服务端一个上游密钥都没配 |
| `upstream_error` / `upstream_timeout` / `upstream_unreachable` | 502 | 上游那边出问题 |
| `upstream_limit` | 502 | 上游额度用尽或被限流 |
| `upstream_credentials` | 502 | 上游认证失败（密钥不对） |

翻译失败时**占用的字符会退还**，不计入额度。

## 六、本地调试

### 跑单测（不需要网络）

```bash
npm test
```

### 本地起一个和 SCF 一模一样的服务

```bash
npm run dev:node        # 默认 http://0.0.0.0:9000，等价于 `node src/node.js`
```

环境变量（PowerShell 里用 `$env:BAIDU_APP_ID = "..."`）跟线上是同一套，另外支持：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `PORT` | 9000 | 监听端口，SCF 必须是 9000 |
| `HOST` | 0.0.0.0 | 监听地址，SCF 必须是 0.0.0.0 |
| `STATE_FILE` | `%TEMP%\glossy-cloud-quota.json` | 计数文件；设成空串则不落盘 |
| `CLIENT_IP_HEADERS` | `x-forwarded-for,x-real-ip,cf-connecting-ip` | 按顺序找客户端地址，取每个头里**倒数第三段**（末尾两段是网关追加的） |

### 打 SCF 的包

```bash
npm run build:scf       # 输出到 dist/scf/
```

### 跑 Cloudflare 那条路

```bash
cp .dev.vars.example .dev.vars   # Windows: Copy-Item .dev.vars.example .dev.vars
# 填好上游密钥（百度必填一组，或填 LLM_API_KEY）
npx wrangler dev                 # 默认 http://127.0.0.1:8787
```

`.dev.vars` 里还可以设 `BAIDU_ENDPOINT`、`LLM_ENDPOINT`（例如指向本地假接口），只影响本机调试，线上不设就用官方地址。

```bash
curl http://127.0.0.1:9000/v1/health
curl -X POST http://127.0.0.1:9000/v1/translate \
  -H 'Content-Type: application/json' \
  -d '{"clientId":"install-0001","text":"hello","to":"zh"}'
```

## 七、看日志 / 排错

Cloudflare：`npx wrangler tail`；腾讯云 SCF：控制台「日志查询」，或者函数详情页的实时日志。

常见情况：

- `upstream_credentials`：上游认证失败。百度是 APP ID 或密钥错了，或百度的「个人认证」没通过（个人认证才有免费额度）；大模型是 Key 失效或没权限。
- `upstream_limit`：上游额度用尽或被限流。两个上游都配了的话，出现这个说明两家都用尽了。
- SCF 上报 `PortBindingFailed` / 函数启动失败：启动命令没填对。检查是不是监听 `0.0.0.0:9000`、
  `scf_bootstrap` 内容是否 LF 换行、`index.mjs` 与启动命令是否在同一个目录。
- SCF 上调用超时：把函数**超时时间调到 10 秒**。
- 客户端报「无法连接」：先分清是哪一种——
  - 地址填错了（结尾不能带 `/v1/translate`，只填到域名即可）；
  - 域名是 `*.workers.dev` 且在国内网络 → 见上文「SNI 阻断」一节，换腾讯云 SCF 或绑自定义域名；
  - 用 `npx wrangler tail`（Cloudflare）或 SCF 日志看服务端有没有收到请求就能区分
    （收到=客户端到了，没收到=网络到不了）。

## 八、文件结构

```
server/
├── wrangler.toml         Cloudflare 部署配置 + 额度参数
├── scripts/
│   └── build-scf.mjs     打成 SCF 用的单文件
├── src/
│   ├── policy.js         额度规则（两边共用，纯逻辑）
│   ├── handler.js        三个接口、参数校验、额度扣减与退还
│   ├── node.js           Node 入口（SCF / 本地）
│   ├── node-server.js    Node http ↔ Request/Response 适配
│   ├── node-crypto.js    Node 18 没有全局 crypto，补上
│   ├── store-file.js     Node 侧的计数存储（JSON 文件）
│   ├── index.js          Cloudflare Worker 入口
│   ├── quota-object.js   按天计数的 Durable Object（SQLite）
│   ├── upstream.js       上游组装（大模型优先、百度兜底）、百度签名、语言代码映射
│   ├── llm.js            OpenAI 兼容的大模型调用与 JSON 解析
│   └── md5.js            Workers 里没有 MD5，自己带一个
└── test/                 node --test 单测
```
