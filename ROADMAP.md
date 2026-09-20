# Glossy roadmap

[English](#en) · [中文](#zh-cn)

<a id="en"></a>

Planned work, grouped into releases. Every release listed here maps to a GitHub
milestone of the same name; the acceptance criteria are what has to be true before
the milestone is closed and the tag is pushed.

## Where the project stands

| Area | State |
| --- | --- |
| Version | `1.0.1`. `src-tauri/tauri.conf.json` is authoritative; `scripts/version.ps1` keeps the five other locations in step and CI fails when one drifts |
| Size | ~15,700 lines: ~8,300 Rust, ~4,600 frontend (plain HTML/CSS/JS), ~1,000 frontend test lines and ~1,900 in `server/`, comments included |
| Tests | 157 Rust tests, 155 frontend tests (`node --test`) and 74 tests for `server/`; `cargo fmt`, `cargo clippy`, `cargo test` and the frontend suite run in CI on `windows-latest` |
| Platform | Windows only — no `cfg(target_os)` gating, the `windows` crate is used unconditionally |
| Distribution | NSIS installer only; no code signing, a self-update skeleton that stays inert until a signing key pair exists, optional start with Windows |
| Backend | `server/` holds a translation proxy that keeps the provider credentials server side, so the app needs no key of its own; it runs on Cloudflare Workers and on Tencent Cloud SCF Web 函数, and one deployment is live. It speaks to an OpenAI-compatible model, to Baidu and to Youdao, and a request can name the one it wants |
| Repository | MIT licensed, changelog and roadmap in place, every release from `v0.1.0` to `v1.0.1` tagged and published with its NSIS installer, and the staged installer kept in `release/vX.Y.Z/` |

No defect is carried into the plan below. The last one — API keys sitting in
`%APPDATA%\com.glossy.translator\settings.json` as readable text — is fixed by the
first item of v0.2.0, which is already in the tree.

## Versioning policy

Semantic Versioning. The `0.x` range was used the way it is meant to be used — every
release could change the settings file format or an IPC command name — and `1.0.0`
ends that:

- `0.1.x` — patches. Bug fixes only: no new settings, no changes to the settings file format.
- `0.x.0` (x ≥ 2) — feature releases. May add settings; `Settings` is
  `#[serde(default)]`, so files written by older versions keep loading.
- `1.x` — feature releases. Settings are only added, never removed or renamed, and IPC
  command names and their arguments stay as they are; a file written before an addition
  keeps loading. Anything that does break either of those waits for `2.0.0`.
- `2.0.0` — the freeze. The settings JSON format and the IPC command names stop
  changing for the whole `2.x` line; later additions go through a migration function.
- Every `X.Y.0` gets a GitHub milestone, and every release is a `vX.Y.Z` tag with the
  NSIS installer attached.

## v0.1.1 — Open-source ready (patch)

Goal: a stranger can clone the repository, build it and trust the result.

| Work item | Acceptance criteria | State |
| --- | --- | --- |
| Add `LICENSE` | File present at the root, referenced from the README | done |
| Single-source the version | Only `tauri.conf.json` carries the number; `scripts/version.ps1` writes the other five locations, and CI fails when one of them disagrees with it | done |
| Add `CHANGELOG.md` | Keep a Changelog format, with a retrospective `0.1.0` section | done |
| CI workflow | `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` on a Windows runner for every push and pull request | done, in `.github/workflows/ci.yml` |
| Build documentation | README covers the required Rust (GNU toolchain) and Node versions, `npm run dev`, `npm run build` and where the installer lands | done |

All five items are in the tree; the release is waiting for its tag. The work needed
to get `clippy` and `rustfmt` to pass cleanly — a derived `Default` for the three
enums in `settings.rs`, one unreachable branch in `default_target_lang`, and a
`cargo fmt` pass over ten files — is behaviour-preserving.

Estimated effort: 0.5–1 day.

## v0.2.0 — Daily driver

Goal: from "it runs" to "I leave it running".

| Work item | Details | State |
| --- | --- | --- |
| Encrypt stored credentials | API keys are protected with DPAPI (`CryptProtectData`, current-user scope) before writing. A settings file written by an older version is migrated and rewritten on the first launch of this build, and a key that belongs to another Windows login is dropped instead of being sent on | done, in `src-tauri/src/secrets.rs` |
| Autostart | `tauri-plugin-autostart` backs a settings toggle. The single-instance guard already ships: a second launch says so in the notification area instead of installing a second mouse hook | done, in `src-tauri/src/autostart.rs` |
| Auto-update skeleton | `tauri-plugin-updater` with a signing key, fed from GitHub releases; can be switched off in the settings | done, in `src-tauri/src/updater.rs`; waiting for a key pair |
| Translation history | A list in the settings window (in memory, optional persistence, configurable cap) with search, reopen-in-card and copy | done, in `src-tauri/src/history.rs` |
| Pin the popup | A button in the header that keeps the card open through clicks outside and disables auto-close until it is closed | done |
| Import and export settings | JSON file out and in, validating through `sanitized()`. Now that credentials are protected, an export has to confirm before it includes them, and an import has to protect what it brings | done, in `export_settings` / `import_settings` |
| Frontend tests | `node --test` coverage for the logic behind `i18n.js`, `render.js` and the classification in `classify.rs`; at least 25 cases | done, 71 cases in `tests/`, wired into CI |
| Manual regression list | A pre-release checklist in the README: multiple monitors, 150% scaling, both colour schemes, every provider, the ignore list | done, 17 items in the README |

All eight items are in the tree. The update check works end to end except for the
signing key: `tauri.conf.json` carries a placeholder public key, so the check
reports that this build cannot update itself, and the uploader needs a real key
pair before a release can be published as an update.

Estimated effort: 3–5 days.

## v0.3.0 — One visual language, and the reading card

Goal: the three windows look like one product, and a translated passage carries
the numbers a reader of the target language needs.

| Work item | Details | State |
| --- | --- | --- |
| Design tokens | `src/styles/tokens.css` holds every colour, radius, shadow, font size and duration; the WinUI palette; `app.css`, `popup.css` and `notice.css` alias the tokens instead of repeating literals | done |
| Theme resolution | `js/theme.js` resolves `system`/`light`/`dark` in one place for all three windows | done |
| Window material | Mica behind the settings window (`window-vibrancy`), a theme-matched title bar colour, and a `SurfaceInfo.ready` handshake so the stylesheet only turns transparent once the backdrop is confirmed | done, in `src-tauri/src/surface.rs` |
| Unit and currency conversion | A measurement or amount the target language does not use is shown converted, with its rate, under the card: length, mass, volume, speed, area, temperature, and money through a live rate table. The numbers are read from the translation rather than the selection — the translator is what decides whether a symbol is a unit, and it writes the measurement in a language the tables know — with the original as the fallback when the translation holds none | done, in `src-tauri/src/units/`; reading the translation from 0.3.1 |
| Conversion switch | `Convert units and currency`, on by default; off means no conversion and no rate request | done |
| Word-card placeholder | The card says the dictionary is being looked up instead of changing under the reader, and the lookups share a short budget | done |

Remaining visual work — the navigation and settings-panel rework, the popup card
rebuild, icon and motion polish, and the accessibility pass — is listed in v1.1.0.
It is behaviour-preserving and ships as its own step.

Estimated effort: 3–4 days.

## v1.0.0 — Glossy Cloud, and the Windows line called stable

Goal: install it and use it. Until now the first thing a new user had to do was
get an API key from a vendor; this release removes that step, and with it the
reason the version stayed in `0.x`.

| Work item | Details | State |
| --- | --- | --- |
| Glossy Cloud provider | A sixth provider that talks to a server of our own instead of to a vendor. The app sends the text and an install id; the server holds the provider credentials, so nothing has to be filled in and no key ever reaches a copy of the app. The address is a setting, validated for `https://`, and a build made from this source already carries the address of the live deployment | done, in `src-tauri/src/translate/cloud.rs` |
| Today's allowance, in the window | The settings window asks the server what is left of today and shows it under the provider row, with a button to ask again; an unreachable server says so in the same line | done, `cloud_status` and `/v1/quota` |
| Install id | One random `cloudId` per installation, kept on disk and regenerated only when a settings file written before this release has none, so the allowance is counted per device rather than per launch | done, in `settings.rs` |
| Server-side quota | Per installation, per caller address and global daily character caps, a per-minute request cap and a per-request size cap, all configurable through environment variables; the caller address comes from the end of `X-Forwarded-For` minus the hops the host's gateway appends, so a client cannot claim a fresh bucket | done, in `server/src/handler.js`; limits documented in `server/README.md` |
| Upstream resilience | The providers are asked again — twice, 600 ms and 1400 ms apart — when they answer with their per-second throttling code or with a temporary failure, because a paid tier with a one-request-per-second limit is the plan the server runs on | done, in `server/src/upstream.js` |
| Deployable two ways | The same source runs on a Cloudflare Worker (a Durable Object makes the count exact) and on a Tencent Cloud SCF Web 函数 through `node-server.js`, with the deployment steps for both in `server/README.md` | done |
| Dark-mode selectors | The language selectors in the popup and in the settings window are readable in dark mode: Windows paints a `select` and its option list with the control's own background, so the translucent fill showed the desktop through the names | done |
| Documentation | The README's provider tables carry the new provider in all three languages, and the server has its own deployment guide | done |

Estimated effort: 4–6 days, spread over several sessions.

## v1.1.0 — Translation quality and the rest of the appearance work

Goal: make the reading use case actually good.

| Work item | Details | State |
| --- | --- | --- |
| Settings window rework | A left navigation rail in place of one long scroll, a search box that filters the options, and grouped panels with the WinUI control look | planned |
| Translation service: one list | One dropdown holds every way a selection can be translated — our own server (nothing to fill in), a model running on this machine over any OpenAI-compatible endpoint, the free public Google endpoint, and the vendors that take the user's key. The credentials of the last group live in an **Extensions** panel at the bottom of the settings window, dimmed while a service that needs nothing is selected. Underneath, the stored shape stayed `channel` + `provider` (plus `cloudVendor` for the server-backed entries), so a file written earlier keeps working and a new one starts on our server | done, in `settings.rs`, `translate/local.rs` and `server/src/llm.js` |
| Named vendor channels | Baidu and Youdao appear in the same dropdown as channels that need nothing set up, because they are our server with a vendor attached: the app sends `cloudVendor`, the server tries that upstream first and falls back to the others, and the card names the one that answered | done, in `server/src/upstream.js`, `server/src/youdao.js` and `src-tauri/src/translate/cloud.rs` |
| Server address is no longer a field | The one service that needs nothing set up should not come with a box that lets people break it, so the address travels with the build: the `DEFAULT_ENDPOINT` the app was compiled with wins, a leftover address in an older settings file is ignored, and the window keeps only the allowance line and its `Check again` button. Pointing the app at your own deployment means editing that one line in `src-tauri/src/translate/cloud.rs`; a build made without an address still reads the setting | done, in `src-tauri/src/translate/cloud.rs` |
| Popup rebuild | The card rebuilt on the token layer: header actions, original/translation hierarchy, dictionary and conversion blocks, and a compact mode | partly, the conversion block is in the tree |
| Icons, motion and accessibility | One icon set, transitions on the popup's appearance and dismissal, full keyboard operability, focus rings, high-contrast colours | planned |
| Provider fallback | When the active provider fails or rate-limits (Google answering `429`), fall back through a configurable order and name the provider that answered in the card footer | done server-side, in `server/src/upstream.js`; the app-side order is still planned |
| More providers | At least two more free tiers (Tencent, Volcengine, or a self-hosted LibreTranslate), plus a custom base URL per provider. Youdao is in: `server/src/youdao.js` serves it, and the app reaches it through the named vendor channels. Tencent was dropped — its machine translation API no longer has a text action, only `ImageTranslateLLM` | partly done |
| Richer word cards | Inflections, synonyms and merged definitions from several sources; an optional sentence-by-sentence view that pairs the original with the translation | planned |
| Word to sentence | Optionally translate the sentence the selected word sits in, alongside the word itself | planned |
| Text to speech | A pronunciation button for the original and the translation, via Windows SAPI or edge-tts | planned |
| Selection robustness | A dedicated pass over rich text, browsers, Office and terminals, where `stripTags` is currently a simple cleanup; backed by a set of real-world fixtures | planned |

Estimated effort: 6–10 days.

## v1.2.0 — Cross-platform and distribution

Goal: leave Windows behind and stop being flagged by SmartScreen.

| Work item | Details |
| --- | --- |
| Platform abstraction | Move the direct Win32 calls in `input.rs`, `platform.rs`, `clipboard.rs` and `hotkey.rs` behind `platform/windows.rs` with `cfg(target_os)` dispatch. This is a prerequisite — there is no gating today |
| macOS | Selection capture through the Accessibility API with a permission onboarding flow; the popup becomes an `NSPanel` that does not take focus; signing and notarisation |
| Linux | X11 first, where the `PRIMARY` selection maps naturally onto select-to-translate. On Wayland the global hook is not available, so it is documented as an unsupported combination rather than silently failing |
| Code signing | Azure Trusted Signing or an EV certificate, so a fresh install no longer shows a SmartScreen warning |
| Packaging matrix | NSIS, MSI and a portable zip; x64 and ARM64 |
| Content Security Policy | `tauri.conf.json` sets `"csp": null` today; replace it with an explicit whitelist (the frontend loads no remote scripts, so this is cheap) |

Estimated effort: 8–15 days.

## v2.0.0 — Format freeze and the long-run promises

Goal: turn a personal tool into something that can be promised.

| Work item | Details |
| --- | --- |
| Format freeze | Settings JSON and IPC command names frozen, with a migration function and a fallback that backs up the original file and starts from defaults |
| Performance budget | Idle CPU below 0.5%, memory below 80 MB, selection to popup under 150 ms, and no handle or GDI leak over long runs — the hook and the clipboard are the places to watch |
| Stability | Crash recovery and optional anonymous error reporting, off by default and asked about on first run |
| Accessibility | Full keyboard operability, a high-contrast theme, correct focus order and aria labels |
| Documentation | The README already carries all three languages; still to write: the FAQ and a note on provider quotas and terms of use |

Estimated effort: 5–10 days.

## Priority

When time is short, the order of return on effort is: **provider fallback in
v1.1.0 → the settings window rework → v1.2.0**.
Cross-platform support is the only item large enough that it may never be finished,
so it is deliberately scheduled last; decide on it after v1.1.0 rather than investing
in it early.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| The Google endpoint is unofficial | It can start rate-limiting or change protocol at any time, and its terms of use are unclear | A retry across two clients already lives in `google.rs`; provider fallback in v1.1.0 is the real fix |
| A currency rate service changes shape or goes down | An amount in the card loses its conversion | Two independent sources (exchangerate-api.com, and the ECB through `frankfurter.app`), a six-hour memory and disk cache, a stale table that stays usable for seven days and is labelled as such, and a two-minute quiet period after both fail |
| Antivirus flags the low-level mouse hook | Installs and runs get blocked | Code signing in v1.2.0, plus a README section explaining what the hook does and why |
| macOS and Linux permission models | The port costs more than expected | Kept as its own release, X11 first, Wayland explicitly unsupported |
| Credential leakage | A readable API key on disk | Fixed for v0.2.0: keys are encrypted with DPAPI and unreadable outside the Windows login that entered them |
| Hand-built releases | Installers cannot be reproduced | Version check, format, lints and tests run in CI since v0.1.1; the installer itself is still built locally by `scripts/release.ps1`, and a tagged release workflow is the next step |

## Release process

1. Set the version in its single source and update `CHANGELOG.md`:
   `powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set X.Y.Z` writes all
   six locations, and `-Check` lists any that drifted apart.
2. `cargo test`, the frontend tests and CI all pass.
3. `powershell -ExecutionPolicy Bypass -File scripts/release.ps1` builds the installer
   and stages it in `release/vX.Y.Z/` with `SHA256SUMS.txt` and the text for the
   release description. The notes it writes are trilingual: English from
   `CHANGELOG.md`, then the same text translated into Chinese and Spanish, each behind
   an anchor (`<a id="en">`, `<a id="zh-cn">`, `<a id="es">`) that the links at the top
   jump to, so one file serves all three languages. A freshly generated file still
   holds the translations as placeholders and the script warns about them; the two
   translations are written by hand (Google Translate is fine as the starting point)
   before the release goes out.
4. Tag `vX.Y.Z` on `main` and push the tag.
5. Publish a GitHub release at that tag, titled `Glossy X.Y.Z` — the tag carries the
   `v`, the title does not. Paste `release/vX.Y.Z/RELEASE_NOTES.md` into the
   description - all three languages, anchors included - and attach the installer
   together with the checksum. Check that the installer is the one this version staged,
   that its name carries the version, and that `SHA256SUMS.txt` lists it: v0.3.1 went
   out with v0.3.0's installer attached, so the release did not contain the fixes the
   notes described.
6. From v0.2.0 on, the release also serves as the auto-update feed.

## What is explicitly out of scope

- Browser extensions and mobile apps. Glossy is a desktop utility, and the selection
  capture it relies on has no equivalent there.
- OCR and screenshot translation. It shares the popup, but not the input path, and it
  would double the surface area.
- Bundling translation engines or models locally. It conflicts with the "lightweight"
  goal; users who need it can point a provider at their own endpoint.

<a id="zh-cn"></a>

# Glossy 路线图 · 中文

按版本分组的计划工作。这里列出的每个版本都对应一个同名的 GitHub 里程碑；验收标准
是关闭该里程碑、推送标签之前必须成立的条件。

## 项目现状

| 方面 | 状态 |
| --- | --- |
| 版本 | `1.0.1`。以 `src-tauri/tauri.conf.json` 为准；`scripts/version.ps1` 让其余五个位置保持一致，任何一处走样 CI 都会失败 |
| 规模 | 约 15,700 行：Rust 约 8,300 行，前端约 4,600 行（纯 HTML/CSS/JS），前端测试约 1,000 行，`server/` 约 1,900 行，含注释 |
| 测试 | Rust 157 个测试、前端 155 个测试（`node --test`）、`server/` 74 个测试；CI 在 `windows-latest` 上跑 `cargo fmt`、`cargo clippy`、`cargo test` 和前端测试 |
| 平台 | 仅 Windows —— 没有 `cfg(target_os)` 分支，`windows` crate 无条件使用 |
| 分发 | 只有 NSIS 安装包；没有代码签名；自更新框架在签名密钥对就位之前保持静默；可选开机自启 |
| 后端 | `server/` 是一个翻译代理，把服务商凭据留在服务端，所以 app 自己不需要任何密钥；可跑在 Cloudflare Workers 和腾讯云 SCF Web 函数上，已有一处在线部署。它对接 OpenAI 兼容模型、百度和有道，请求里可以点名要用哪一个 |
| 仓库 | MIT 许可，CHANGELOG 和路线图齐备，从 `v0.1.0` 到 `v1.0.1` 的每个版本都已打标签并连同 NSIS 安装包发布，暂存的安装包保存在 `release/vX.Y.Z/` |

下面的计划里没有遗留缺陷。最后一个 —— API 密钥以明文躺在
`%APPDATA%\com.glossy.translator\settings.json` 里 —— 由 v0.2.0 的第一项修复，
而那一项已经在代码库里。

## 版本策略

遵循语义化版本。`0.x` 阶段就是按它的本意用的 —— 每个版本都可能改动设置文件格式
或某个 IPC 命令名 —— `1.0.0` 结束了这一点：

- `0.1.x` —— 补丁。仅修 bug：不加设置项，不动设置文件格式。
- `0.x.0`（x ≥ 2）—— 功能版本。可以新增设置；`Settings` 是
  `#[serde(default)]`，所以旧版本写出的文件照样能加载。
- `1.x` —— 功能版本。设置只增不删、不改名，IPC 命令名及其参数保持原样；在新增项
  之前写出的文件仍然能加载。任何会破坏这两点的改动都要等到 `2.0.0`。
- `2.0.0` —— 冻结。设置 JSON 格式和 IPC 命令名在整个 `2.x` 线内不再变化；之后再
  新增要走迁移函数。
- 每个 `X.Y.0` 都配一个 GitHub 里程碑，每个版本都是一个带 NSIS 安装包的
  `vX.Y.Z` 标签。

## v0.1.1 —— 可开源（补丁）

目标：陌生人能克隆仓库、构建它，并信任构建结果。

| 工作项 | 验收标准 | 状态 |
| --- | --- | --- |
| 加入 `LICENSE` | 根目录存在该文件，README 里有引用 | 完成 |
| 版本单一来源 | 只有 `tauri.conf.json` 带版本号；`scripts/version.ps1` 写入其余五个位置，CI 在任一位置与它不一致时失败 | 完成 |
| 加入 `CHANGELOG.md` | 采用 Keep a Changelog 格式，并有一节回溯性的 `0.1.0` | 完成 |
| CI 工作流 | 每次推送和 PR 都在 Windows runner 上跑 `cargo fmt --check`、`cargo clippy -- -D warnings` 和 `cargo test` | 完成，见 `.github/workflows/ci.yml` |
| 构建文档 | README 覆盖所需的 Rust（GNU 工具链）和 Node 版本、`npm run dev`、`npm run build` 以及安装包落地的位置 | 完成 |

五项都在代码库里；这个版本只差标签。为了让 `clippy` 和 `rustfmt` 干净通过所做的
工作 —— 给 `settings.rs` 里三个枚举派生 `Default`、`default_target_lang` 里一个
不可达分支，以及对十个文件跑一遍 `cargo fmt` —— 都不改变行为。

预计工作量：0.5–1 天。

## v0.2.0 —— 日常可用

目标：从“能跑”到“我就让它一直开着”。

| 工作项 | 细节 | 状态 |
| --- | --- | --- |
| 加密已存凭据 | API 密钥在写入前用 DPAPI（`CryptProtectData`，当前用户范围）保护。旧版本写出的设置文件在本版本首次启动时迁移并重写；属于另一个 Windows 登录的密钥会被丢弃，而不是被发出去 | 完成，见 `src-tauri/src/secrets.rs` |
| 开机自启 | `tauri-plugin-autostart` 支撑一个设置开关。单实例保护已就位：第二次启动只在通知区域提示，不会装上第二个鼠标钩子 | 完成，见 `src-tauri/src/autostart.rs` |
| 自更新框架 | 带签名密钥的 `tauri-plugin-updater`，由 GitHub release 提供更新；可在设置里关闭 | 完成，见 `src-tauri/src/updater.rs`；等待密钥对 |
| 翻译历史 | 设置窗口里的一份列表（内存存储、可选持久化、可配置上限），支持搜索、在卡片中重开和复制 | 完成，见 `src-tauri/src/history.rs` |
| 固定弹窗 | 标题栏上一个按钮，固定后卡片在点击外部时保持打开，直到手动关闭前都不自动关闭 | 完成 |
| 设置导入导出 | JSON 文件导出与导入，导入时经 `sanitized()` 校验。既然凭据已被保护，导出时必须先确认才会带上它们，导入时必须保护带进来的内容 | 完成，见 `export_settings` / `import_settings` |
| 前端测试 | 对 `i18n.js`、`render.js` 背后的逻辑以及 `classify.rs` 的分类做 `node --test` 覆盖；至少 25 个用例 | 完成，`tests/` 里 71 个用例，已接入 CI |
| 手动回归清单 | README 里的一份发布前清单：多显示器、150% 缩放、两种配色方案、每个服务商、忽略列表 | 完成，README 里 17 项 |

八项都在代码库里。除了签名密钥，更新检查已经端到端可用：`tauri.conf.json` 里放的是
占位公钥，所以检查会报告本版本无法自更新；要把某个发布当作更新推送出去，上传器需要
一对真实的密钥。

预计工作量：3–5 天。

## v0.3.0 —— 统一的视觉语言，以及阅读卡片

目标：三个窗口看起来像同一个产品，并且一段译文会带上目标语言读者需要的数字。

| 工作项 | 细节 | 状态 |
| --- | --- | --- |
| 设计变量 | `src/styles/tokens.css` 收纳所有颜色、圆角、阴影、字号和时长；WinUI 调色板；`app.css`、`popup.css` 和 `notice.css` 引用变量，而不是重复字面量 | 完成 |
| 主题解析 | `js/theme.js` 为三个窗口统一解析 `system`/`light`/`dark` | 完成 |
| 窗口材质 | 设置窗口背后的 Mica（`window-vibrancy`）、跟随主题的标题栏颜色，以及 `SurfaceInfo.ready` 握手，让样式表只在确认背景材质后可透明 | 完成，见 `src-tauri/src/surface.rs` |
| 单位与货币换算 | 目标语言不使用的计量或金额会在卡片下方显示换算值及其汇率：长度、质量、体积、速度、面积、温度，以及通过实时汇率表的货币。数字从译文里读取，而不是从选区 —— 因为一个符号是不是单位由翻译器决定，而它会用表里认识的语种写出该计量 —— 译文里没有时回退到原文 | 完成，见 `src-tauri/src/units/`；从 0.3.1 起改为读译文 |
| 换算开关 | `换算单位与货币`，默认开启；关闭即不换算、也不请求汇率 | 完成 |
| 单词卡片占位 | 卡片会显示正在查词典，而不是在读者眼前变来变去，且多次查询共享一段很短的时间预算 | 完成 |

剩余的视觉工作 —— 导航与设置面板重做、弹窗卡片重建、图标与动效打磨、无障碍梳理
—— 列在 v1.1.0。它们不改变行为，作为独立一步发布。

预计工作量：3–4 天。

## v1.0.0 —— Glossy Cloud，以及正式稳定的 Windows 版本线

目标：装上就能用。此前新用户要做的第一件事是去厂商申请 API 密钥；这个版本去掉了
这一步，也随之去掉了版本停留在 `0.x` 的理由。

| 工作项 | 细节 | 状态 |
| --- | --- | --- |
| Glossy Cloud 服务商 | 第六个服务商，它对接我们自己的服务器而不是厂商。app 发送文本和一个安装 id；服务器持有服务商凭据，所以用户什么都不用填，密钥也永远不会进入任何一份 app 副本。地址是一个设置项，按 `https://` 校验，而由本源码构建出的版本已经带上在线部署的地址 | 完成，见 `src-tauri/src/translate/cloud.rs` |
| 窗口里显示今日额度 | 设置窗口向服务器询问今天还剩多少，并显示在服务商那一行下面，附一个再次询问的按钮；服务器不可达时在同一行说明 | 完成，`cloud_status` 与 `/v1/quota` |
| 安装 id | 每个安装一份随机 `cloudId`，落盘保存，仅当一个早于本版本的设置文件里没有时才重新生成，因此额度按设备而非按启动次数计算 | 完成，见 `settings.rs` |
| 服务端额度 | 按安装、按调用方地址和全局的每日字符上限，加上每分钟请求上限和单次请求大小上限，全部可通过环境变量配置；调用方地址取自 `X-Forwarded-For` 的末位、扣除宿主网关追加的跳数，因此客户端无法自称一个新的配额桶 | 完成，见 `server/src/handler.js`；限额记在 `server/README.md` |
| 上游容错 | 当服务商返回每秒限流的错误码或临时失败时，会再请求两次 —— 间隔 600 ms 和 1400 ms —— 因为服务器跑在的就是每秒一次请求的付费档 | 完成，见 `server/src/upstream.js` |
| 两种部署方式 | 同一份源码可跑在 Cloudflare Worker 上（用 Durable Object 让计数精确），也可通过 `node-server.js` 跑在腾讯云 SCF Web 函数上，两种部署步骤都在 `server/README.md` | 完成 |
| 深色模式下的选择器 | 弹窗和设置窗口里的语种选择器在深色模式下可读：Windows 会用控件自己的背景绘制 `select` 及其选项列表，于是半透明填充会让名字后面透出桌面 | 完成 |
| 文档 | README 的服务商表格在三种语言里都带上了新服务商，服务器也有自己的部署指南 | 完成 |

预计工作量：4–6 天，分散在若干次会话里。

## v1.1.0 —— 翻译质量与外观工作的剩余部分

目标：让阅读这个用法真正好用。

| 工作项 | 细节 | 状态 |
| --- | --- | --- |
| 设置窗口重做 | 用左侧导航栏取代一整条长滚动、一个能筛选选项的搜索框，以及符合 WinUI 控件观感的分组面板 | 计划中 |
| 翻译渠道：一个列表 | 一个下拉框就装下所有翻译方式——我们自己的服务器（无需填写）、通过任意 OpenAI 兼容接口跑在本机上的模型、免费的公开 Google 接口，以及需要用户密钥的服务。最后一类的凭据放在设置窗口最下面的 **扩展** 面板里，当前服务不需要凭据时会变暗。底层保存格式仍然是 `channel` + `provider`（走服务器的几个渠道另有 `cloudVendor`），因此旧文件照常可用，新文件默认走我们自己的服务器 | 已完成，见 `settings.rs`、`translate/local.rs` 与 `server/src/llm.js` |
| 点名上游的渠道 | 「百度翻译」「有道翻译」和「Glossy 翻译」并列在同一个下拉框里，同样是「无需配置」——因为它们就是我们自己的服务器，只是指定了用哪家上游：应用发送 `cloudVendor`，服务器先试点名的那个，答不上来再按顺序兜底，卡片页脚会注明这次是谁译的 | 已完成，见 `server/src/upstream.js`、`server/src/youdao.js` 与 `src-tauri/src/translate/cloud.rs` |
| 服务器地址不再是一个输入框 | 唯一一个「无需配置」的服务不该给用户留下把它填坏的机会，所以地址改为跟着构建走：构建时的 `DEFAULT_ENDPOINT` 优先，旧设置文件里残留的地址被忽略，窗口只保留额度提示行与「重新检查」。想用自己的部署就改 `src-tauri/src/translate/cloud.rs` 里的那一行；不带地址的构建仍然读设置文件 | 已完成，见 `src-tauri/src/translate/cloud.rs` |
| 弹窗重建 | 在变量层上重建卡片：头部操作、原文/译文层级、词典与换算区块，以及一个紧凑模式 | 部分完成，换算区块已在代码库中 |
| 图标、动效与无障碍 | 统一图标集、弹窗出现与消失的过渡、完整键盘操作、焦点环、高对比配色 | 计划中 |
| 服务商回退 | 当前服务商失败或限流时（Google 返回 `429`），按可配置顺序回退，并在卡片页脚注明是哪家服务的 | 服务端已完成，见 `server/src/upstream.js`；客户端可配置顺序仍计划中 |
| 更多服务商 | 至少再接入两个免费档（腾讯、火山引擎，或自建 LibreTranslate），并支持为每个服务商自定义 base URL。有道已接入：`server/src/youdao.js` 提供上游，客户端通过点名上游的渠道使用它。腾讯已放弃——它的机器翻译接口只剩 `ImageTranslateLLM`，没有文本翻译动作了 | 部分完成 |
| 更丰富的单词卡片 | 词形变化、同义词，以及合并多个来源的释义；可选的逐句对照视图，把原文与译文配对 | 计划中 |
| 单词所在句 | 可选地，在显示单词本身的同时翻译它所在的句子 | 计划中 |
| 朗读 | 原文与译文各一个发音按钮，通过 Windows SAPI 或 edge-tts | 计划中 |
| 选区健壮性 | 针对富文本、浏览器、Office 和终端做一轮专门梳理，目前 `stripTags` 只是简单清理；并用一组真实场景的样本作支撑 | 计划中 |

预计工作量：6–10 天。

## v1.2.0 —— 跨平台与分发

目标：走出 Windows，并且不再被 SmartScreen 拦下。

| 工作项 | 细节 |
| --- | --- |
| 平台抽象 | 把 `input.rs`、`platform.rs`、`clipboard.rs` 和 `hotkey.rs` 里直接调用的 Win32 挪到 `platform/windows.rs` 之后，用 `cfg(target_os)` 分派。这是前置条件 —— 目前完全没有分派 |
| macOS | 通过 Accessibility API 捕捉选区，并配一个权限引导流程；弹窗改为不抢焦点的 `NSPanel`；签名与公证 |
| Linux | 先做 X11，那里的 `PRIMARY` 选区天然对应“划词翻译”。Wayland 上没有全局钩子，因此明确记为不支持的组合，而不是悄悄失效 |
| 代码签名 | Azure Trusted Signing 或 EV 证书，让全新安装不再弹 SmartScreen 警告 |
| 打包矩阵 | NSIS、MSI 和便携 zip；x64 与 ARM64 |
| 内容安全策略 | `tauri.conf.json` 目前是 `"csp": null`；换成显式白名单（前端不加载任何远程脚本，所以这一步很便宜） |

预计工作量：8–15 天。

## v2.0.0 —— 格式冻结与长期承诺

目标：把个人工具变成可以承诺的东西。

| 工作项 | 细节 |
| --- | --- |
| 格式冻结 | 冻结设置 JSON 与 IPC 命令名，并配迁移函数和一个兜底：备份原文件、从默认值重新开始 |
| 性能预算 | 空闲 CPU 低于 0.5%、内存低于 80 MB、从选区到弹窗低于 150 ms，长时间运行不泄漏句柄或 GDI —— 钩子和剪贴板是最需要盯的地方 |
| 稳定性 | 崩溃恢复，以及可选的匿名错误上报，默认关闭并在首次运行时询问 |
| 无障碍 | 完整键盘操作、高对比主题、正确的焦点顺序与 aria 标签 |
| 文档 | README 已含三种语言；还要写：FAQ，以及一篇关于服务商配额与使用条款的说明 |

预计工作量：5–10 天。

## 优先级

时间紧张时，投入产出比的顺序是：**v1.1.0 的服务商回退 → 设置窗口重做 →
v1.2.0**。
跨平台是大到可能永远做不完的一项，所以它有意排在最后；在 v1.1.0 之后再决定要不要
做，而不是提前投入。

## 风险

| 风险 | 影响 | 缓解 |
| --- | --- | --- |
| Google 接口并非官方 | 它随时可能开始限流或改变协议，使用条款也不清晰 | `google.rs` 里已有跨两个客户端的重试；v1.1.0 的服务商回退才是真正的解法 |
| 汇率服务改结构或宕机 | 卡片里的金额失去换算 | 两个独立来源（exchangerate-api.com，以及通过 `frankfurter.app` 的欧洲央行），六小时的内存与磁盘缓存，过期表在七天内仍可用并明确标注，以及两者都失败后的两分钟静默期 |
| 杀毒软件拦截底层鼠标钩子 | 安装与运行被阻止 | v1.2.0 的代码签名，外加 README 里一节说明这个钩子做什么、为什么需要 |
| macOS 与 Linux 的权限模型 | 移植成本超出预期 | 保持为独立版本，先 X11，明确不支持 Wayland |
| 凭据泄露 | 磁盘上有可读的 API 密钥 | v0.2.0 已修复：密钥用 DPAPI 加密，在输入它们的 Windows 登录之外不可读 |
| 手工构建的发布 | 安装包无法复现 | 自 v0.1.1 起版本检查、格式、lint 和测试都在 CI 里跑；安装包本身仍由 `scripts/release.ps1` 在本地构建，带标签的发布工作流是下一步 |

## 发布流程

1. 在唯一来源里设置版本号，并更新 `CHANGELOG.md`：
   `powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set X.Y.Z` 会写入全部
   六个位置，`-Check` 会列出任何走样的位置。
2. `cargo test`、前端测试和 CI 全部通过。
3. `powershell -ExecutionPolicy Bypass -File scripts/release.ps1` 构建安装包，并把
   它暂存到 `release/vX.Y.Z/`，附上 `SHA256SUMS.txt` 和发布说明文本。它写出的说明
   是三语的：英文取自 `CHANGELOG.md`，随后是同一段文字的中文和西班牙文译版，各自
   带一个锚点（`<a id="en">`、`<a id="zh-cn">`、`<a id="es">`），顶部链接跳转到这些
   锚点，因此一个文件服务三种语言。新生成的文件里译文还是占位符，脚本会就此警告；
   发布前需手工写好两份译文（拿 Google Translate 起稿即可）。
4. 在 `main` 上打 `vX.Y.Z` 标签并推送该标签。
5. 在该标签处发布 GitHub release，标题为 `Glossy X.Y.Z` —— 标签带 `v`，标题不带。
   把 `release/vX.Y.Z/RELEASE_NOTES.md` 粘进描述 —— 三种语言、锚点一并保留 —— 并
   上传安装包与校验和。确认安装包正是本版本暂存的那个、文件名带版本号、且
   `SHA256SUMS.txt` 里列出了它：v0.3.1 发布时附上的却是 v0.3.0 的安装包，于是发布里
   并不包含说明所描述的修复。
6. 从 v0.2.0 起，发布同时充当自动更新的更新源。

## 明确不在范围内

- 浏览器扩展和移动端 app。Glossy 是桌面工具，它依赖的选区捕捉在那些平台上没有对应
  能力。
- OCR 与截图翻译。它们与弹窗共用同一套展示，但输入路径完全不同，而且会让接触面翻
  一倍。
- 在本地捆绑翻译引擎或模型。这与“轻量”的目标冲突；有需要的用户可以把自己的服务商
  指向自建端点。
