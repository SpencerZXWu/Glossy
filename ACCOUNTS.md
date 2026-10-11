# Glossy 账号与用户数据 · 设计文档

> **状态：草案，未实现。** 本文只写方案，不含任何代码改动。审完再动手。
> 作者视角：本文件由 AI 助手起草，所有「现状」都标了代码位置，可以逐条核对。

---

## 0. 一页摘要

你要的三件事，其实是三个不同层次的问题：

| 你的诉求 | 真正的问题 | 解决它的最小改动 |
| --- | --- | --- |
| 「剩余额度本地能改」 | **身份可伪造**：额度按客户端自称的安装 ID 计数，改 `settings.json` 里的 `cloudId` 就等于换了一台新设备 | 把计数的键从「安装 ID」换成「服务器签发的账号」——**这一条本身就能堵住洞，不需要先做登录界面** |
| 「用户数据上传到数据库」 | 现在**服务器根本没有数据库**（计数在函数实例 `/tmp` 的 JSON 里），也没有任何用户记录 | 加一格真正的库：免费起步用 **CloudBase 免费体验版**（≈¥3/月资源点），长期方案是腾讯云 PostgreSQL（¥30–60/月），两者都存账号、设备、额度账本、版本 |
| 「方便以后告知用户信息」 | 客户端**连自己的版本号都不上报**，服务器没有任何触达通道 | 每次请求带上 `appVersion`，服务端响应里回带 `announcement` |

**能不能用邮箱注册？能。** 但项目现在没有任何发信能力（没有域名邮箱、没有发信服务），而发信不只是注册验证码——改密码、换邮箱、通知用户都要它。所以「邮箱注册」= 同时引入一家邮件服务商 + 一个配好 SPF/DKIM/DMARC 的域名。

**代价（必须先知道）**：这会推翻仓库里已经写下的承诺——[PRODUCT.md](./PRODUCT.md) 的「There is no account and no login」、[PRIVACY.md](./PRIVACY.md) 开头的「no account, no analytics and no telemetry」、[FAQ.md](./FAQ.md) 的「我需要账号吗？不需要」。上线登录就必须同步改这些文档，并补上隐私政策的收集目的、保留期限、删除账号入口，以及首次启动的同意流程。

**建议节奏**：先定下面「15. 待你拍板的决策」里的 4 件事 → 后端账号 + 额度按账号（1–2 天）→ 客户端登录界面（2–3 天）→ 版本上报与公告（1 天）→ 文档与合规（1 天）。

---

## 1. 目标与非目标

**目标**

- **G1** 堵住额度可绕过：额度只能按服务器认可的账号计数，改本地文件不能换来新额度。
- **G2** 建立最小用户数据：账号、设备、额度账本、客户端版本，能回答「谁在用、用什么版本、还剩多少」。
- **G3** 有一条主动触达通道：能对指定版本区间推一条公告（升级提示、维护通知、条款变更）。
- **G4** 不破坏现有体验：不登录时，不花项目钱的渠道（`google`、`offline`、自填 API）仍然可用。

**非目标（本次不做）**

- **N1** 不做云同步历史/生词本/设置（那是另一件事，需要冲突合并策略）。
- **N2** 不上报原文、译文、选区内容、截图——一个字都不存，这是 [PRIVACY.md](./PRIVACY.md) 的核心承诺。
- **N3** 不做第三方登录（微信/QQ 之后再说；Google/GitHub 在大陆不可达，直接排除）。
- **N4** 不做付费/订阅体系，`tier` 字段先留空位。

---

## 2. 现状（代码事实）

### 2.1 额度是怎么算的

- 客户端取额度：`GET /v1/quota?client=<安装ID>` → 服务端返回 `limits{...}`、`usage{client,ip,total}`、`remaining`、`remainingOcrMonth`（[server/src/handler.js](./server/src/handler.js) 的 `/v1/quota` 分支）。
- 计数维度与默认上限（[server/wrangler.toml](./server/wrangler.toml) 的 `[vars]`，与 `handler.js` 的默认值一致）：

| 维度 | 键 | 默认上限 | 存哪 |
| --- | --- | --- | --- |
| 每安装 ID / 天 | `clientId` | 20,000 字符 | 当天桶 |
| 每 IP / 天 | IP 哈希 | 30,000 字符 | 当天桶（挡「换 ID 继续刷」） |
| 全局 / 天 | `TOTAL_SCOPE \| TOTAL_ID` | 30,000 字符 | 当天桶（保险丝） |
| 每分钟请求 | IP 哈希 | 30 次/分 | 分钟桶 |
| OCR / 每月 | `clientId` | 100 次 | 月度桶 |

- 存储位置随部署方式不同：Cloudflare 用 Durable Object + SQLite（`usage(scope,id,chars,requests)`，[server/src/quota-object.js](./server/src/quota-object.js)）；**线上的那套是腾讯云 SCF**（[src-tauri/src/translate/cloud.rs](./src-tauri/src/translate/cloud.rs) 里 `DEFAULT_ENDPOINT = https://1492303375-cwrw0pdztr.ap-guangzhou.tencentscf.com`），而 **SCF 上没有数据库**，计数写在函数实例 `/tmp` 的 JSON 文件里（`STATE_FILE`，加旁边的 `.month`，见 [server/README.md](./server/README.md) 的说明与 `store-file.js`）。
- 记账逻辑与存储解耦，共用 `policy.js`；桶键是 `scope|id` 字符串，**新增一个 `account` scope 是很小的改动**。

### 2.2 身份与可伪造点

- 身份就是 `settings.json` 里的 `cloudId`（[src-tauri/src/settings.rs](./src-tauri/src/settings.rs)），由本机 + Windows 账户哈希得出，**存在本机文件里，用户可编辑**。
- 因此「每安装 ID 20k/天」可被 `换一个 cloudId` 绕过；但每 IP 30k/天 与全局 30k/天 仍然拦得住，所以是「换 ID 继续用」而不是「无限刷」。`policy.js` 里还有一条保护：桶数量有上限，超了会塌缩成共享行，所以伪造 ID 不能把状态撑爆。
- 服务器**不知道**调用者是谁：`POST /v1/translate` 的体是 `{clientId, text, from, to, vendor}`，没有令牌、没有版本（[src-tauri/src/translate/cloud.rs](./src-tauri/src/translate/cloud.rs)）。

### 2.3 版本与触达现状

- 客户端只在「检查更新」时问 GitHub `releases/latest`（[src/index.html](./src/index.html) 更新页 + [src/js/app.js](./src/js/app.js) 的 `checkForUpdate`），**平时不给中转服务报版本**。
- 服务器没有任何向客户端推消息的机制：响应体里没有公告字段。
- 已有一条可用的事件通道：`glossy://update`（设置窗口里已经有人监听）。

### 2.4 现有接口清单（服务器）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/v1/health` | 健康检查、上游是否配置、当天日期 |
| GET | `/v1/quota?client=` | 额度与剩余 |
| GET | `/v1/rates?client=&base=` | 汇率（不计字符额度，只计每分钟） |
| POST | `/v1/translate` | 翻译 |
| POST | `/v1/ocr` | 截图取字（按每月次数计） |

---

## 3. 方案总览

```
┌──────────────┐   邮箱+密码+验证码          ┌─────────────────────────┐
│ Glossy 客户端 │ ─────────────────────────▶ │ 账号服务（SCF 同区）      │
│  (Tauri)     │ ◀───────────────────────── │  /v1/auth/*             │
│              │   access token (1h)        │                         │
│ 设置里的 token│                            └───────┬─────────────────┘
│  ← DPAPI 保护 │                                    │ 读写
└──────┬───────┘                            ┌───────▼─────────────────┐
       │  Authorization: Bearer + appVersion│  数据库（PostgreSQL）    │
       │                                    │  users / sessions /     │
       └──────────────────────────────────▶ │  devices / quota_usage /│
          /v1/translate /v1/ocr /v1/quota   │  announcements          │
          （额度按账号计数）                  └─────────────────────────┘
                                                     ▲
                                       邮件服务（阿里云 DirectMail / 腾讯云 SES）
```

三句话概括：

1. **账号服务和中转服务是同一个部署**（现在就是腾讯云 SCF 广州），账号库放在同一个区的云数据库里，走内网；不引第二套基础设施。
2. **额度计数的键从 `clientId` 换成 `userId`**，`ip` / `total` 两个桶原样保留作为兜底——这才是修漏洞的那一步。
3. **令牌只存在本机**，用仓库里已有的 DPAPI 模块（[src-tauri/src/platform/windows/secrets.rs](./src-tauri/src/platform/windows/secrets.rs)）保护后写进设置文件，和自填 API 的密钥同一套机制。

---

## 4. 认证设计

### 4.1 为什么是「邮箱 + 密码 + 6 位验证码」

| 方案 | 判断 |
| --- | --- |
| **邮箱 + 密码 + 验证码**（推荐） | 不依赖浏览器跳转，桌面应用最省事；发信能力早晚要有（改密码也要），一次投入两处受益 |
| 魔法链接（点邮件里的链接回来） | 需要 deep-link 插件 + 默认浏览器往返，桌面端体验更绕，且邮件客户端预取链接会误触发 |
| 邮箱 + 一次性验证码（无密码） | 体验最好，但**每次登录都要发信**，发信成本与到达率风险全压在登录上；可作为后续升级项 |
| 微信 / QQ 登录 | 国内可达，但需要企业主体/开放平台审核，周期长，放到 N3 |
| Google / GitHub 登录 | 大陆不可达，直接排除 |

### 4.2 密码怎么存

- 算法：**scrypt**（`node:crypto` 内置，SCF 直接可用）或 **PBKDF2-SHA256**（WebCrypto 与 node:crypto 都支持）。参数起步：scrypt `N=2^15, r=8, p=1`（约 100ms），PBKDF2 600,000 次。
- **不用 argon2**：需要原生模块，SCF 的 Node 运行时不好带；除非走 `hash-wasm`（纯 JS，两边都能跑），如果你更看重抗 GPU 强度，就选它，代价是多一个依赖。
- 每个用户独立 salt（16 字节），哈希与参数一起存，参数变更时可透明升级。

### 4.3 令牌

| 令牌 | 内容 | 寿命 | 存哪 |
| --- | --- | --- | --- |
| access | `{sub: userId, exp, dev: deviceId}`，HMAC-SHA256 签名 | 1 小时 | **只在内存**，不落盘 |
| refresh | 随机 32 字节，服务端只存哈希 | 30 天，每次刷新轮换 | 设置文件里，`DPAPI` 保护（复用 `secrets.rs`） |

- 轮换 + 复用检测：一个已被换掉的 refresh 再次出现，说明令牌被复制，撤销整条会话链。
- 退出登录 = 删除本地 refresh + 服务端 `revoked_at`。
- 服务端签名密钥走 `wrangler secret` / SCF 环境变量，和上游厂商密钥一样**不进仓库**。

### 4.4 接口

| 方法 | 路径 | 入参 | 出参 | 备注 |
| --- | --- | --- | --- | --- |
| POST | `/v1/auth/register` | `email, password` | `{ok, verifySent:true}` | 不立即发令牌；同邮箱重复注册返回同样的成功响应（防枚举） |
| POST | `/v1/auth/verify` | `email, code` | `{ok, account:{id,email}, accessToken, refreshToken, quota:{...}}` | 验证成功后即可用 |
| POST | `/v1/auth/resend` | `email` | `{ok}` | 60 秒内不重发 |
| POST | `/v1/auth/login` | `email, password, device:{id,platform,os,appVersion}` | 同 verify | 未验证邮箱 → `email_unverified`，客户端跳到验证码页 |
| POST | `/v1/auth/refresh` | `refreshToken` | 新的一对令牌 | 轮换 |
| POST | `/v1/auth/logout` | `refreshToken` | `{ok}` | 撤销当前会话 |
| POST | `/v1/auth/password/reset` | `email` | `{ok}` | 发重置验证码 |
| POST | `/v1/auth/password/confirm` | `email, code, password` | `{ok}` | 改完撤销所有会话 |
| GET | `/v1/me` | Bearer | `{account, devices[], quota}` | 账号页用 |
| DELETE | `/v1/me` | Bearer + 密码确认 | `{ok}` | 删除账号：级联删除设备/会话/账本（合规要求） |

**所有现有接口**（`/v1/translate`、`/v1/ocr`、`/v1/quota`、`/v1/rates`）增加：
- 请求头 `Authorization: Bearer <access>`（可选，缺失时按「未登录」处理）；
- 请求头或 body 里带 `appVersion`、`deviceId`；
- 未登录时 `/v1/translate`、`/v1/ocr` 直接 `401 login_required`（**只在花项目钱的渠道上强制**，见 6.3）。

错误码沿用现有风格（`code` + 中文 `message`，见 `handler.js` 的 `fail()`）。

### 4.5 邮件流程

| 模板 | 触发 | 内容 | 有效期 |
| --- | --- | --- | --- |
| 注册验证码 | `register` / `resend` | 6 位数字 + 10 分钟有效 + 「不是你是谁在注册」提示 | 10 分钟 |
| 重置密码验证码 | `password/reset` | 同上 | 10 分钟 |
| 邮箱变更确认 | 改邮箱 | 双确认：旧邮箱收「即将变更」，新邮箱收「确认」 | 24 小时 |
| 安全通知 | 新设备登录 | 时间、设备、IP 归属地（不存明文 IP） | 即时 |

- 验证码存**哈希**（不是明文），带尝试次数上限（5 次错误作废）。
- 域名要配 **SPF + DKIM + DMARC**；发件人用 `no-reply@你的域名`；退信/投诉要能处理（否则会被拉黑）。
- 国内收件（QQ/163/企业微信邮）优先用国内服务商，见下。

### 4.6 发信服务选型

| 服务 | 大陆到达率 | 免费额度 | 之后的价格 | 备注 |
| --- | --- | --- | --- | --- |
| **阿里云邮件推送 DirectMail**（**长期最省**） | 好 | **共 2000 封**，且**每天最多免费 200 封** | **¥0.002/封** | 只需在发信域名上配 SPF/DKIM；**不需要 ICP 备案**（只有「邮件追踪」功能要求域名已备案）。支持 API 与 SMTP |
| **腾讯云 SES 邮件推送**（和中转同厂） | 好 | **1000 封，终身一次性**（不是每月） | ¥0.0019/封 | 发信域名要配 SPF/**DKIM**/DMARC（广州区用 `qcloudgz2048._domainkey`）；发信同样**不需要备案** |
| Resend | 一般（境外） | 3000 封/月（每天 100 封封顶） | $20/月起 | 代码最舒服，但发 QQ/163 到达率差、大陆访问其控制台也不稳 |
| Mailjet / Brevo | 一般 | 6000 封/月（≤200/天）/ 300 封/天 | 按量 | 同上，境外 |
| SendGrid | 一般 | **已无长期免费档**（只剩 60 天试用） | $19.95/月起 | 老账号可能还留着 100/天 |
| 个人 QQ 邮箱 SMTP | 好 | — | — | **不推荐**：每日上限腾讯官方不公开、属个人用途条款、发 163 容易被判垃圾 |
| 企业微信/腾讯企业邮 SMTP | 好 | 域内与联系人无限，外部按域名动态限量 | 付费坐席约 ¥100/账号/年 | 限额不公开，不适合做注册验证码的主力 |
| Cloudflare Email Routing | — | 免费 | — | **只能收，不能发** |

### 4.7 防滥用

| 动作 | 限制 |
| --- | --- |
| 注册（同 IP） | 每小时 5 次、每天 20 次 |
| 邮件发送（同邮箱） | 60 秒 1 封、每天 10 封 |
| 登录失败 | 同邮箱 5 次/15 分钟后要求验证码；同 IP 100 次/小时 |
| 验证码校验 | 5 次错误即作废，需重新发送 |
| 账号创建总量 | 每日上限 + 异常告警（防刷库） |

注册接口同样要走现有的每分钟限流（`requestsPerMinute`）。

### 4.8 删除账号与数据导出

- `DELETE /v1/me` 需要密码确认，删除后：`sessions`、`devices`、`quota_usage` 立即级联删除；`users` 行把 `email` 换成不可逆哈希、`status='deleted'`，保留 30 天以备争议，之后物理删除。
- 客户端账号页提供「删除账号」按钮（合规需要，别等用户来邮件要）。

---

## 5. 数据模型

```sql
CREATE TABLE users (
  id            BIGSERIAL PRIMARY KEY,
  email         TEXT NOT NULL UNIQUE,          -- 存小写
  password_hash TEXT NOT NULL,                 -- "scrypt$N$r$p$salt$hash"
  email_verified_at TIMESTAMPTZ,
  status        TEXT NOT NULL DEFAULT 'active',-- active | suspended | deleted
  tier          TEXT NOT NULL DEFAULT 'free',  -- 预留：free | plus
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_login_at TIMESTAMPTZ
);

CREATE TABLE sessions (
  id           BIGSERIAL PRIMARY KEY,
  user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  device_id    TEXT NOT NULL,
  refresh_hash TEXT NOT NULL,                  -- 只存哈希
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at TIMESTAMPTZ,
  expires_at   TIMESTAMPTZ NOT NULL,
  revoked_at   TIMESTAMPTZ
);
CREATE INDEX ON sessions (user_id, expires_at);

CREATE TABLE devices (
  id           BIGSERIAL PRIMARY KEY,
  user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  install_hash TEXT,                           -- 老 cloudId 的哈希，用于迁移绑定
  platform     TEXT,                           -- windows
  os           TEXT,                           -- Windows 11 26100
  app_version  TEXT,
  first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at  TIMESTAMPTZ,
  last_ip_hash  TEXT                           -- 只存哈希，不存明文 IP
);
CREATE UNIQUE INDEX ON devices (user_id, install_hash);

-- 额度账本：和现有桶键一一对应，只是多了 account 作用域
CREATE TABLE quota_usage (
  scope    TEXT NOT NULL,                      -- account | client | ip | total
  key      TEXT NOT NULL,                      -- userId / installId / ipHash
  period   TEXT NOT NULL,                      -- 2026-10-05（天）或 2026-10（月）
  chars    BIGINT NOT NULL DEFAULT 0,
  requests BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (scope, key, period)
);

-- 验证码（也可以放内存/KV，重启会丢，落库更稳）
CREATE TABLE email_codes (
  email      TEXT NOT NULL,
  purpose    TEXT NOT NULL,                    -- register | reset | change
  code_hash  TEXT NOT NULL,
  attempts   INT  NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (email, purpose)
);

CREATE TABLE announcements (
  id          BIGSERIAL PRIMARY KEY,
  severity    TEXT NOT NULL,                   -- info | warning | critical
  min_version TEXT, max_version TEXT,          -- 语义化区间，NULL 表示不限
  title_key   TEXT NOT NULL,                   -- 客户端 i18n key
  body_key    TEXT,
  url         TEXT,
  starts_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  ends_at     TIMESTAMPTZ
);
```

保留策略：`sessions` 过期即清；`quota_usage` 保留 90 天（对账用），之后归档或删除；`email_codes` 用完即删。

---

## 6. 额度重构

### 6.1 键从「安装 ID」换成「账号」

`policy.js` 的 `peek/reserve` 现在读 `client` 桶，加一个 `account` 桶即可：

```
已登录： buckets = { account: userId, ip: ipHash, total: TOTAL }
未登录： 拒绝（if 该渠道要求登录）
```

- 每账号每天多少字符：建议**沿用 20,000**（先和现在一致，观察一周真实用量再调）。
- `ip` 30k / 全局 30k 保留，唯一的作用是「同一个账号被多台机器共享」和「有人批量注册」时的兜底。
- **不需要**再按设备分额度：设备只是一个显示维度（账号页里能看到「这台机器」），不参与额度计算。

### 6.2 过渡与迁移

| 场景 | 处理 |
| --- | --- |
| 老用户升级后未登录 | 中转渠道提示「需要登录后继续使用」，`google` / `offline` / 自填 API 照常 |
| 登录时带一个已有 `cloudId` | 服务端把 `install_hash` 绑到账号上（`devices` 表），不继承历史用量（更简单也更安全） |
| 双计期间（可选，30 天） | 同时记 `account` 与 `client` 两个桶，方便对比「账号额度」和「老的设备额度」是否一致；之后只留 `account` |
| 想回退 | 老版本客户端仍带 `clientId`：给未带令牌的请求一个极小的额度（例如 2,000/天）或直接拒绝，两者都要在灰度里验证 |

### 6.3 哪些渠道要求登录

| 渠道 | 是否要求登录 | 理由 |
| --- | --- | --- |
| `cloud-baidu` / `cloud-youdao` | **要求** | 花的是项目的钱，也是额度的所在 |
| `/v1/ocr`（截图取字） | **要求** | 同上（截图取字也是项目付给厂商的） |
| `google` | 不要求 | 直连 Google，不花项目钱 |
| `offline` | 不要求 | 本机模型 |
| `api-baidu` / `api-openai` | 不要求 | 用户自己的账号 |

这样「不注册就不能用」只落在真正花钱的渠道上，不会让整个应用变成必须注册才能打开。

### 6.4 OCR 月度

`ocrPerClientMonth` 同样换成按账号（`account` 作用域的月度桶）。账号页显示「本月截图取字剩余 N 次」。

---

## 7. 用户数据、版本与触达

### 7.1 上报字段（最小集）

每次请求的请求头（或顺带在 body 里）：

| 字段 | 例子 | 用途 | 是否敏感 |
| --- | --- | --- | --- |
| `Authorization` | `Bearer …` | 身份 | 是（只存哈希/短命） |
| `X-Glossy-Version` | `2.1.0` | 版本分布、最低支持版本 | 否 |
| `X-Glossy-Device` | `dev_7f3a…` | 设备维度 | 否（随机 ID） |
| `X-Glossy-OS` | `Windows 11 26100` | 兼容性排障 | 否 |
| IP | — | 风控，**只存哈希** | 是 |

**明确不上报**：原文、译文、选区文本、截图、历史记录、生词本、设置内容、自填 API 的密钥。

### 7.2 公告怎么送到用户面前

- 服务端在 `/v1/quota`、`/v1/translate` 的响应里附一个可选字段：`announcement: {severity, titleKey, bodyKey, url}`（服务端按 `min_version/max_version` 和当前 `appVersion` 筛，带 60 秒缓存）。
- 客户端：`info` → 窗口里一条 toast；`warning` → 更新页一行 + toast；`critical` → 启动时弹一次不可忽略的提示（引导样式复用现有 `confirmAction` 渲染）。
- 文案用 **i18n key**，不直接下发中文/英文，这样公告在两个界面语言下都对，也避免下发 HTML。

### 7.3 最低支持版本

- 加一条服务端配置 `MIN_SUPPORTED_VERSION`；低于它的 `appVersion` 在 `/v1/translate` 上返回 `426 upgrade_required`，客户端把它渲染成「这一版太旧，请下载 2.x.x」+ 一个跳转按钮（`releases/latest` 已经在更新页里用了）。
- 好处：以后改了中转协议，不需要让老客户端「发出去才报错」。

---

## 8. 客户端改动清单

| 文件 | 改动 |
| --- | --- |
| `src-tauri/src/auth.rs`（新） | `register/login/verify/resend/reset/refresh/logout/me/delete` 的 HTTP 客户端 + 令牌刷新（带并发去重、过期前 5 分钟续期） |
| `src-tauri/src/settings.rs` | `Settings` 加 `account { id, email, refreshToken, verified, tier }`；`refreshToken` 与自填 API 的密钥一样走 `protected_for_storage()`；`sanitized()` 里对邮箱做 trim/小写 |
| `src-tauri/src/platform/windows/secrets.rs` | 直接复用，无需改动（新字段自动被保护，因为 `stored_credentials()` 是列表驱动的——**要记得把新字段加进那张表**） |
| `src-tauri/src/lib.rs` | 新命令：`auth_register`、`auth_login`、`auth_verify`、`auth_logout`、`account_status`、`account_delete`；`save_settings` 里不要把 `refreshToken` 回传给窗口（**只回 `{id,email,verified}`**，否则密钥会出现在 WebView 里） |
| `src-tauri/src/translate/cloud.rs` | 请求加 `Authorization`、版本头；`/v1/quota` 改成带令牌调用；把 `login_required` / `upgrade_required` 两个错误码翻译成中文提示 |
| `src/index.html` | 新页面 `page-account`（未登录=注册/登录表单 + 验证码；已登录=邮箱、额度、设备列表、「退出登录」「删除账号」）；General 页加一条「账号」入口 |
| `src/js/app.js` | `login`/`register` 流程、验证码倒计时、额度行改成「账号额度」、未登录时在渠道下拉框上给出提示、公告渲染（复用 `showToast`/`confirmAction`） |
| `src/js/i18n.js` | 新增 `account.*`、`announcement.*` 两个命名空间（中英各一份，key 必须一一对应） |
| `tests/markup.test.js` / `tests/i18n.test.js` | 新页面的 `data-i18n` 键、控件命名、两个字典的 key 对齐（现有测试会自动覆盖一部分） |

**安全细节**：令牌绝不能回传到窗口（WebView 里能看到的一切都等于明文）；`account_status` 只返回展示所需字段。

---

## 9. 服务器改动清单

| 文件 | 改动 |
| --- | --- |
| `server/src/auth.js`（新） | 注册/登录/验证码/令牌签发与轮换；密码哈希；限流（复用 `policy.js` 的分钟桶，新增注册桶） |
| `server/src/db.js`（新） | 数据库访问层（`pg` 或 `mysql2`），连接池 + 语句超时；**必须要能注入内存实现**，否则现有 143 个单测跑不动 |
| `server/src/mail.js`（新） | 发信适配器（阿里云 DirectMail / 腾讯云 SES 两个实现 + 本地「打印到控制台」实现用于测试） |
| `server/src/handler.js` | 新增 `/v1/auth/*`、`/v1/me`；现有接口加令牌校验与 `appVersion` 记录；`/v1/quota` 改为按账号；错误码补 `login_required`、`upgrade_required`、`email_unverified` |
| `server/src/policy.js` | 加 `account` 作用域；`limits` 增加 `charsPerAccount`、`minSupportedVersion` |
| `server/src/store-file.js`、`quota-object.js` | 桶作用域新的 `account` 键在两边都要能存（DO 的 SQL 表不用改结构，`scope` 已经是列） |
| `server/test/auth.test.js`（新） | 用内存 DB + 假发信器，覆盖：注册→验证→登录→刷新→登出、验证码过期、错误次数上限、重放被撤销的 refresh |
| `server/README.md` | 部署章节加数据库与邮件环境变量；新增「账号」一节 |

**部署要点**：SCF 要开 **VPC** 才能连云数据库；数据库只允许内网访问；`STATE_FILE` 那套文件计数可以保留（额度兜底用），但账号数据必须在库里。

---

## 10. 安全清单

- [ ] HTTPS 全程（现有 SCF 域名已是 https）
- [ ] 令牌不进 WebView、不进日志、不进崩溃报告
- [ ] 密码哈希参数可升级；错误信息不区分「邮箱不存在」和「密码错误」（统一 `invalid_credentials`）
- [ ] 注册/重置接口响应不泄露「邮箱是否已注册」
- [ ] 数据库只存 IP 哈希；日志里对邮箱/令牌打码
- [ ] 服务端签名密钥与上游厂商密钥一样走 secret 管理，不入库不入仓库
- [ ] 删除账号后真删关联数据；提供导出
- [ ] 依赖审计：SCF 包体保持小（现在用 esbuild 打成单文件），`pg` 之类要么打进包要么用 HTTP 数据库 API

---

## 11. 隐私与合规

**文档必须同步改（中英各一份）**

| 文件 | 改动 |
| --- | --- |
| [PRODUCT.md](./PRODUCT.md) | 删掉「There is no account and no login」这条原则，改成「账号可选，仅花钱的渠道需要」 |
| [PRIVACY.md](./PRIVACY.md) | 加「我们收集什么」（邮箱、设备、版本、额度账本、IP 哈希）、「不收集什么」（原文/译文/截图/历史）、用途、保留期限、删除方式、发信服务商是谁 |
| [FAQ.md](./FAQ.md) | 「我需要账号吗？」改成「要用 Glossy 自己的中转渠道需要；用 Google/离线/自填 API 不需要」 |
| [README.md](./README.md) | 两份：安装与首次使用流程、账号页说明、回归清单加注册/登录/额度/公告几条 |
| `ACCOUNTS.md`（本文） | 定稿后从「草案」改为设计记录，并在 README 的 Layout 一节登记 |

**合规要点（不是技术问题，请按你的实际情况判断）**

1. 腾讯云账号需**实名认证**；自建发信域名要**ICP 备案**（SCF 默认域名不用备案，所以中转一直能用）。
2. 邮箱属于个人信息：要有**告知同意**（首次启动的登录屏里给一条「隐私政策」链接）、**目的正当与最小化**、**保留期限**、**可删除/可导出**。
3. 若数据库放境外（比如 Cloudflare D1），会涉及**数据出境**，个保法有额外要求；所以建议账号库就放在腾讯云广州，和中转同区。
4. 面向公众提供服务的站点/域名，按现行规定可能需要完成备案与公安备案。
5. 未成年人：如面向未成年人需额外提示；先按「不主动收集年龄」处理。

---

## 12. 成本估算

### 12.1 免费路径（2026-10-05 核对过官方页面）

| 项目 | 免费方案 | 免费额度 | 超了之后 | 备注 |
| --- | --- | --- | --- | --- |
| **数据库 + 后端** | **腾讯云开发 CloudBase 免费体验版** | 每账号一个免费环境，**3000 资源点/月（1000 点 = ¥1，≈ ¥3/月）** | 付费套餐或迁到独立数据库 | 唯一「大陆原生 + 免费用」的组合；自带**用户名密码 / 邮箱登录** |
| **发信** | **阿里云 DirectMail**（长期）或 **腾讯云 SES** | DirectMail **2000 封**（≤200 封/天）；SES **1000 封终身** | ¥0.002 / ¥0.0019 每封 | 几百个用户、5000 封邮件 ≈ **¥6–7.6** |
| 域名 | 已有域名则 0 | — | ¥50–100/年 | 发信只需要在 DNS 上加 SPF/DKIM/DMARC，**不需要 ICP 备案** |
| SCF 调用 | 现有函数 | 现有量在免费额度内 | — | — |

**CloudBase 免费版的三个坑（必须知道）**

1. 是「**体验版**」：**单次 6 个月，到期前 1 个月内手动续期，不支持自动续费**。
2. 如果以后把它用于**发布微信小程序**，免费环境会在**上线后第 15 天到期**（对桌面端纯 Web/H5 不适用）。
3. 3000 资源点/月 ≈ ¥3 的资源，**不适合长跑**；真实用户量上来就要迁到独立数据库。所以数据库访问层要**可替换**（见 §9 的 `db.js`），别把 CloudBase 的调用散落在业务代码里。
4. 它的**邮箱登录是否代发验证邮件需要实测**；如果不代发，就用上面的 SES/DirectMail 自己发，成本几乎一样。

**已经排除的选项（别再考虑）**

| 选项 | 结论 |
| --- | --- |
| LeanCloud 国内版 | **已死**：2026-01-12 起功能冻结（新用户不能注册、不能新建应用），**2027-01-12 全量停服、数据销毁** |
| Cloudflare Workers / D1 / Durable Objects | `*.workers.dev` **在 gfwlist 里，大陆不可直连**；Cloudflare 中国网络是企业版且要备案 |
| Supabase / Neon / Turso 免费版 | 免费额度很好看（Supabase 500 MB + 5 万 MAU，Neon 1 GB，Turso 5 GB），但**都没有中国区，大陆可达性未验证**，别拿账号体系赌这个 |
| 腾讯云 / 阿里云 独立 RDS | **没有长期免费档**，只有新用户试用（约 1 个月） |

### 12.2 付费路径（用户量上来之后）

| 项目 | 规格 | 参考成本（人民币） |
| --- | --- | --- |
| 云数据库 PostgreSQL/MySQL | 1 核 2G 基础版，广州 | ¥30–60 / 月 |
| 邮件服务 | DirectMail / SES 按量 | 约 ¥2 / 千封 |
| 域名 | 已有则 0 | ¥50–100 / 年 |
| SCF 调用 | 现有函数调用量以内 | 基本在免费额度里 |
| 时间 | 后端 1–2 天、客户端 2–3 天、文档 1 天、预留 1 天联调 | — |

对比不做账号的代价：额度可以靠换 ID 绕过，且没有任何触达用户的手段。

---

## 13. 分期计划与验收标准

| 阶段 | 内容 | 验收标准 |
| --- | --- | --- |
| **0 决策** | 定 4 件事（见第 15 节） | 你能一句话回答完 |
| **1 后端**（1–2 天） | 数据库 + `/v1/auth/*` + 令牌校验 + 额度按账号 + `appVersion` 记录 | 单测覆盖注册→验证→登录→刷新→登出；把一个 `cloudId` 改成新的也拿不到新额度（`account` 桶不变）；老客户端带 `clientId` 时的策略经验证生效 |
| **2 客户端**（2–3 天） | 登录/注册界面、令牌 DPAPI 落库、账号页、未登录降级提示 | 真实跑通：注册→收码→验证→翻译→额度正确；清空设置后需要重新登录；令牌不出现在窗口、日志和导出文件里 |
| **3 触达**（1 天） | 版本上报 + 公告接口 + 三种严重度的展示 + 最低版本拦截 | 造一条公告，指定版本区间，只有符合条件的客户端看得到；低于最低版本时翻译被拦住并给出下载入口 |
| **4 合规**（1 天） | 中英文档、隐私政策、删除账号、注册限流 | 逐份文档改完；删除账号后数据库里查不到关联行；注册接口限流生效 |

每个阶段的验证都按仓库现有习惯走：`node --test tests/*.test.js`、`cd server && npm test`、`cargo test --lib`、`cargo fmt --check`、`clippy --all-targets -- -D warnings`，加真实应用里手工走一遍（README 的回归清单里加条目）。

---

## 14. 风险与备选

| 风险 | 影响 | 对策 |
| --- | --- | --- |
| 强制登录掉用户 | 一部分人只是想「划一下看意思」 | 只对花钱的渠道要求登录（6.3）；未登录时给出「原因 + 一键注册」而不是硬报错 |
| 邮件到达率差 | 注册卡住 | 选国内服务商；提供「重新发送」；后续可加短信或第三方登录 |
| SCF 冷启动 + 数据库连接 | 首次请求慢 | 连接池 + 语句超时；必要时把账号服务拆成常驻实例 |
| 数据库成为单点 | 挂了全站不能用 | 额度兜底逻辑保留（`STATE_FILE` 计数），库挂了只降级「不能登录/不能查账号」，不让翻译整体挂掉 |
| 一个账号被多人共用 | 20k/账号 被摊薄 | `ip` 与「同 IP 多账号」的告警；必要时把每账号额度按设备数微调 |
| 密钥管理 | 令牌签名密钥泄露 | 只放 secret 管理；支持双密钥轮换 |

**备选：只做「最小堵漏」（0.5 天）**
服务端给每个安装 ID 签发一个 HMAC 设备令牌，首次见到新 ID 时要求一次「注册」（邀请码，或全新 ID 每日额度 = 0）。这能立刻堵住「改 ID 换额度」，但**不解决触达和版本统计**——那两件事仍然需要账号。可以把它当作阶段 1 的过渡。

---

## 15. 待你拍板的决策

1. **数据库**：走免费路径（**腾讯云开发 CloudBase 免费体验版**，自带邮箱/用户名登录，但只是 6 个月一续的体验版）还是直接上收费的**腾讯云 PostgreSQL**（¥30–60/月，是长期方案，迁移成本为零）？
2. **邮件服务**：**阿里云 DirectMail**（长期最省：2000 封免费、之后 ¥0.002/封）还是**腾讯云 SES**（和中转同厂：1000 封终身免费、之后 ¥0.0019/封）？发信域名用哪个域名（只需 DNS 验证，不需要备案）？
3. **强制范围**：确认「只有中转渠道 + 截图取字要求登录，Google/离线/自填 API 不要求」这个口径？
4. **额度**：每账号每天仍是 20,000 字符吗？OCR 仍是每月 100 次吗？（先沿用现有数字，观察一周再调也行）
5. **老客户端策略**：未带令牌的旧版本给「极小额度（如 2,000/天）」还是直接拒绝？（前者更温和，能留住还没升级的用户）
6. **登录方式**：确认「邮箱 + 密码 + 验证码」？要不要同时预留「邮箱 + 纯验证码（无密码）」的入口？
7. **公告与最低版本**：要不要一开始就做 `critical` 级别的强制弹窗？还是先只做「更新页里一行提示」？
8. **合规**：隐私政策、删除账号、首次启动同意流程，是这一版就做，还是分到下一次发布？
