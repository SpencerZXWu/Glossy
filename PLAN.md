# Glossy — what comes next

Written 2026-10-12, with the tree on 2.2.1. [ROADMAP.md](./ROADMAP.md) says how the
project got here and stops at v2.0.0; this file is the forward list, in the order it
should be done. Every state below was checked on the day it was written — against the
tree, or against the live deployment with one request. A line that was not checked is
not in here.

## Where things actually stand

| Thing | State today | How to see it |
| --- | --- | --- |
| App | 2.2.1, packaged: NSIS, MSI, portable zip, both languages of notes, checksums | `release/v2.2.1/` |
| Live relay | answering, but running the bundle from **before 2.2.0** | `/v1/health` has no `counters` field |
| Word lookup on the relay | not deployed yet | `/v1/dict` is not there until the bundle above is replaced |
| The allowance's numbers | **one instance's**, not the deployment's | 20 asks in a row read `0/0/20000`; 11 characters booked by a probe were gone from the same question minutes later |
| The operator's report | **on** — a deployment has an `ADMIN_TOKEN` | a wrong token answers `403 forbidden`, not `403 not_configured` |
| `scripts/stats.ps1` | works as it is | it reaches the relay and prints the server's own message |
| Tags | 2.2.0 has a tag and a GitHub release; from now on the maintainer asks for packages without them | — |

The one sentence that matters: **the app side is ahead of the deployment side.** Until
the new bundle is uploaded, none of 2.2.0 or 2.2.1 exists on the relay the app talks to.

## P0 — Put the deployment where the app already is (no code)

| Work item | Acceptance | Effort |
| --- | --- | --- |
| Upload the 2.2.1 bundle: paste `server/dist/scf/index.mjs` into the function's editor | `/v1/health` answers with `counters: { shared: false }`, and `/v1/dict?client=<id>&word=draft` answers `200` with `found: true` | 10 minutes |
| Keep one copy of the uploaded bundle somewhere it can be compared later | the file that was pasted can be named by version and digest | 5 minutes |

Nothing else in this file can be judged until this is done: the allowance line, the
report page and the word card all depend on what the relay answers, and right now it
answers from an older build.

## P1 — Make the allowance exact, and prove it

Goal: two questions about the same allowance, asked a second apart, cannot disagree.

| Work item | Acceptance | Effort |
| --- | --- | --- |
| Point the deployment at a shared store: create a Tencent Cloud Redis (standard architecture, public address — see `server/README.md` for why the VPC route is a network rebuild) and set `REDIS_HOST`, `REDIS_PORT`, `REDIS_PASSWORD` | `/v1/health` answers `counters: { shared: true }` | 30 minutes, mostly waiting for the instance |
| Check it before trusting it: `cd server && npm run check:redis` | three ✓ — write, read back, lock and book | 5 minutes |
| Promote the throwaway probe into the tree as `scripts/quota-check.ps1`: ask `/v1/quota` N times for one client id and refuse to agree unless every answer matches | the script is in `scripts/`, takes `-Endpoint`/`-Client`/`-Rounds`, and today prints the two shapes it finds | 1 hour |
| Register the daily line so the history exists from now on: `scripts/stats.ps1 -SaveToken`, then `-Register daily` | one row per day in `Documents/glossy-server-report.csv`, token stored with DPAPI, nothing in the task | 10 minutes |
| Say which of the two a reader is looking at, in the window, with a sentence that helps: today the line says "counted on this instance alone" — it could add what to do about it | the line names the state and the fix, in both languages | 1 hour |

## P2 — The word card, now that its lookup is fast

| Work item | Acceptance | Effort |
| --- | --- | --- |
| Cache entries on the relay: the same word asked twice in a day is asked of the dictionary once | a test with a stub fetch asserts the second lookup does not reach the dictionary; the cache has a TTL and a cap | half a day |
| Survive a dictionary that is having a bad day: a served entry outlives a slow upstream | with the upstream answering `522`, a word already in the cache still answers `found: true` | 2 hours, with the item above |
| Decide what the card does for languages the dictionary has no entry for (it carries English explained in Chinese today) — either a second source or an honest gap | the rule is written in `PROVIDERS.md` and matches what the card does | half a day |
| Two phonetics, not one: the relay already sends `phonetics.uk` and `phonetics.us`; the card shows one | both are readable when the dictionary has both | 2 hours |
| Watch the new cost: a lookup is one request against the per-minute limit, so selecting words quickly can hit `30/min` | a burst of lookups is measured, and the limit or the client's behaviour is chosen on purpose | half a day |

## P3 — Releases without tags

The maintainer's rule since 2026-10-12: **packaging means the artefacts and
`release/v<version>/`, not a tag and not a GitHub release.**

| Work item | Acceptance | Effort |
| --- | --- | --- |
| Say the rule where the tooling can be read: `release/README.md` and the closing lines `scripts/release.ps1` prints | neither document tells the reader to create a tag as the next step | 30 minutes |
| Decide what `v2.2.0`'s tag and release are for now (leave, or undo with `gh release delete` + `git push origin :v2.2.0`) | the answer is written down so the next release does not have to guess | 5 minutes |
| The tag-triggered workflow, `.github/workflows/release.yml`, stays as it is but says when it is meant to be used | its header states that it is for a tagged release, which is now the exception | 15 minutes |
| Code signing (SignPath Foundation) is still waiting on their answer; nothing to do meanwhile | — | — |

## P4 — The upgrade path people actually take

| Work item | Acceptance | Effort |
| --- | --- | --- |
| Freeze a 1.1.2 settings file as a fixture and assert what it becomes: `service` folded from the four old fields, `ocrPack` → `ocrPacks`, `guideSeen` still absent | a test reads the fixture and checks each of the three, so a later migration cannot quietly break a jump of this size | 2 hours |
| Decide the guide's behaviour for someone who has used Glossy for a year: today `guideSeen` is missing from their file, so the tour plays once, the first time they open the settings window | the choice is written in the release notes and matches the code | 15 minutes, plus 1 hour if it changes |
| A one-page upgrade note for a version jump this large: what visibly changed in the window, and what did not | it is in the release notes for 2.2.1 or the next release, in both languages | 1 hour |

## P5 — Further out, from the roadmap

- The **settings search box**, and the icon, motion and accessibility polish that v1.1.0
  still lists as planned.
- **ARM64** packaging (needs an MSVC toolchain this machine does not have), then
  **macOS** and **Linux**, which `src/platform/mod.rs` was built for.
- **Code signing**, through the SignPath Foundation application that is already written
  in `README.md` and `release/README.md`.
- **Not planned**: anonymous reporting (v1.7.0 decided against it — the log is exported
  by hand instead), and any provider that would need a key typed into the window.

## Risks worth watching

| Risk | What it would look like | What to do about it |
| --- | --- | --- |
| SCF recycles instances, so a store in `/tmp` loses the day's numbers | the allowance reads full on a fresh instance; the report disagrees with itself | P1 — the shared store is the fix, not a patch |
| The dictionary endpoint is a public web interface, not a contracted API | entries stop arriving, or the shape changes | `YOUDAO_DICT_ENDPOINT` points at another one; the app falls back on the public endpoint it used before |
| The installers are unsigned | SmartScreen warns on a fresh download | SignPath; nothing else is honest to promise |
| The app and the relay can drift apart again | a feature is in the window and not on the deployment | P0's acceptance lines are the check: `/v1/health` and one request per route |

## How to use this file

Each item has an acceptance line that one command or one look can settle. When an item
is done, move its line into the release's `CHANGELOG.md` section and delete it here;
add a row to the state table above when the deployment changes. A list that is not
pruned stops being trusted, which is worse than no list.

---

# Glossy 接下来做什么 · 中文

写于 2026-10-12，树上是 2.2.1。[ROADMAP.md](./ROADMAP.md) 记录的是已经走过的路，写到 v2.0.0 为止；
这份文件是往前的清单，按该做的顺序排列。下面每一条的状态都在写它的那天核对过——对着代码树，或者对着
线上部署发一个请求。没核对过的话不写进来。

## 现在的真实情况

| 事项 | 今天的状态 | 怎么看 |
| --- | --- | --- |
| 应用 | 2.2.1，已打包：NSIS、MSI、便携 zip、中英说明、校验和 | `release/v2.2.1/` |
| 线上中转 | 能答，但跑的是 **2.2.0 之前**的包 | `/v1/health` 里没有 `counters` 字段 |
| 中转上的查词 | 还没部署 | 换掉上面的包之前，`/v1/dict` 不存在 |
| 额度的数字 | 是**某一台实例的**，不是整个部署的 | 连问 20 次都是 `0/0/20000`；探针打进去的 11 个字符，几分钟后再问就没了 |
| 运维报告 | **开着**——部署方设了 `ADMIN_TOKEN` | 口令错时返回 `403 forbidden`，而不是 `403 not_configured` |
| `scripts/stats.ps1` | 现状就能用 | 它能连上中转并打印服务端自己的话 |
| 标签 | 2.2.0 有 tag 和 GitHub Release；从现在起，维护者要的是"打包"，不建 tag | — |

一句话总结：**应用那一端跑在部署那一端前面。** 新包没上传之前，2.2.0 和 2.2.1 在应用真正对话的
那台中转上都不存在。

## P0 —— 把部署补到应用所在的位置（不用写代码）

| 工作项 | 验收标准 | 工时 |
| --- | --- | --- |
| 上传 2.2.1 的包：把 `server/dist/scf/index.mjs` 粘进函数的在线编辑器 | `/v1/health` 里出现 `counters: { shared: false }`，并且 `/v1/dict?client=<id>&word=draft` 返回 `200`、`found: true` | 10 分钟 |
| 留一份"传上去的到底是哪个包"的记录 | 那次粘贴的文件能按版本和摘要指名道姓 | 5 分钟 |

这件事做完之前，这份文件里别的条目都无从判断：额度那一行、报告页、单词卡片，全看中转怎么回答，
而它现在回答的是旧包。

## P1 —— 让额度变准，并且能证明它准

目标：同一个额度，间隔一秒问两次，不可能给出两个答案。

| 工作项 | 验收标准 | 工时 |
| --- | --- | --- |
| 让部署用上共享存储：开一个腾讯云 Redis（标准架构、公网地址——为什么内网那条路是"网络改造"而不是填两个变量，见 `server/README.md`），设 `REDIS_HOST`、`REDIS_PORT`、`REDIS_PASSWORD` | `/v1/health` 回答 `counters: { shared: true }` | 30 分钟，大多在等实例开通 |
| 信它之前先验它：`cd server && npm run check:redis` | 三个 ✓——写入、读回、加锁并记账 | 5 分钟 |
| 把一次性的探针提升成仓库脚本 `scripts/quota-check.ps1`：对同一个 client id 问 N 次 `/v1/quota`，只要有一次不一致就不算通过 | 脚本在 `scripts/` 里，支持 `-Endpoint`/`-Client`/`-Rounds`，并且在今天这种状态下能打印出它找到的两种形状 | 1 小时 |
| 从今天起留下历史：`scripts/stats.ps1 -SaveToken`，再 `-Register daily` | `文档/glossy-server-report.csv` 一天一行；口令用 DPAPI 存，任务里不含口令 | 10 分钟 |
| 窗口里那句话说得更有用：现在写的是"本部署按实例计数"，可以再补一句该怎么办 | 两种语言里，那一行既说明状态也给出出路 | 1 小时 |

## P2 —— 查词快了之后，单词卡片还缺什么

| 工作项 | 验收标准 | 工时 |
| --- | --- | --- |
| 中转上做词条缓存：同一天同一个词只去词典问一次 | 用假 fetch 的测试断言第二次查询不碰上上游；缓存有 TTL 和上限 | 半天 |
| 顶住词典服务抽风：已经拿到的词条不受一次慢上游影响 | 上游返回 `522` 时，缓存里已有的词仍然答 `found: true` | 2 小时，含上一条 |
| 给"词典没有这个词的语言"一个说法（它现在只带英汉词条）：要么接第二个源，要么老实说没有 | 规则写进 `PROVIDERS.md`，并且和卡片的行为一致 | 半天 |
| 两个音标都显示：中转已经同时给出 `phonetics.uk` 和 `phonetics.us`，卡片只显示一个 | 词典两个都有时，两个都看得到 | 2 小时 |
| 盯住新的成本：一次查词占用一次每分钟限额，快速连选几个词可能撞到 `30/min` | 实测一次连选，然后有意识地决定是调限额还是调客户端行为 | 半天 |

## P3 —— 不建 tag 的发布方式

维护者自 2026-10-12 起的规则：**"打包"指的是产物和 `release/v<版本>/`，不是 tag，也不是 GitHub Release。**

| 工作项 | 验收标准 | 工时 |
| --- | --- | --- |
| 把规则写在能被读到的地方：`release/README.md`，以及 `scripts/release.ps1` 结尾打印的那几行 | 两份文档都不再把"创建 tag"说成下一步 | 30 分钟 |
| 决定 `v2.2.0` 的 tag 和 Release 现在算什么（留着，或者用 `gh release delete` + `git push origin :v2.2.0` 撤掉） | 结论写下来，下一次发版不必再猜 | 5 分钟 |
| 由 tag 触发的 `.github/workflows/release.yml` 保持原样，但写明它什么时候才用 | 文件开头说明它是给"打标签的发版"用的，而那是例外情况 | 15 分钟 |
| 代码签名（SignPath 基金会）还在等对方答复，眼下无事可做 | — | — |

## P4 —— 真实用户会走的那条升级路径

| 工作项 | 验收标准 | 工时 |
| --- | --- | --- |
| 把一份 1.1.2 的设置文件固化成测试夹具，断言它会被变成什么：四个旧字段折叠成 `service`、`ocrPack` → `ocrPacks`、`guideSeen` 仍然缺失 | 一个测试读夹具并逐条核对，以后任何迁移都不能悄悄破坏这种跨度的升级 | 2 小时 |
| 决定"用了一年的老用户"该不该看到引导：现在他们的文件里没有 `guideSeen`，所以打开设置窗口时会播一次 | 这个决定写进发布说明，并且和代码一致 | 15 分钟；如果要改行为再加 1 小时 |
| 为这么大跨度的升级写一页说明：窗口里可见的变化，以及没变的东西 | 落在 2.2.1 或下一版的发布说明里，两种语言 | 1 小时 |

## P5 —— 更远的事（来自路线图）

- **设置窗口的搜索框**，以及 v1.1.0 里仍标着 planned 的图标、动效、无障碍收尾。
- **ARM64** 打包（需要这台机器没有的 MSVC 工具链），之后是**macOS** 和 **Linux**——
  `src/platform/mod.rs` 就是为它们铺的。
- **代码签名**，走已经写在 `README.md` 和 `release/README.md` 里的 SignPath 基金会申请。
- **不做的事**：匿名上报（v1.7.0 明确不做——日志改为手动导出），以及任何要求用户在窗口里填密钥的渠道。

## 值得盯住的风险

| 风险 | 它会以什么样子出现 | 怎么办 |
| --- | --- | --- |
| SCF 会回收实例，`/tmp` 里的计数随之丢失 | 新实例上额度显示是满的；报告和自己打架 | P1：共享存储是解法，不是补丁 |
| 词典接口是公开网页接口，不是签约 API | 词条不来了，或者结构变了 | `YOUDAO_DICT_ENDPOINT` 换一个地址；应用会退回原来那个公共接口 |
| 安装包没有签名 | 新下载时 SmartScreen 报警 | SignPath；除此之外没有老实的承诺可给 |
| 应用和中转再次跑偏 | 功能在窗口里有、在部署上没有 | P0 的验收标准就是这道检查：`/v1/health` 加每个路由发一个请求 |

## 这份文件怎么用

每条都有一行"一个命令或一眼就能定论"的验收标准。做完了，就把那一行搬进对应版本的 `CHANGELOG.md`
并从这份文件里删掉；部署有变化时，更新上面那张状态表。一份不做删减的清单会失去信任，那比没有清单更糟。
