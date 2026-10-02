# Glossy 全量缺陷检查报告

## 一、测试基本信息

| 项目 | 内容 |
| --- | --- |
| 应用名称 | Glossy（选中即译桌面工具） |
| 版本号 | 1.7.1（`package.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json` 三处一致） |
| 测试环境 | Windows 本机；源码工作区 `E:\Glossy`（有 90 项未提交改动，含 1.5.0–1.7.1 全部发布内容）；浏览器预览 `http://127.0.0.1:8791`（`src/js/bridge.js` 内存模拟层）；用户已安装的运行实例 `D:\Glossy\glossy.exe`（PID 33592，1.7.1） |
| 测试类型 | 全量检查：三套自动化测试 + 三道质量门禁 + 四个页面功能/布局/无障碍/i18n + 真实实例与中继侧证据核对 |
| 测试时间 | 2026-09-30 18:22—19:05 |
| 检查范围 | 设置窗口（`index.html`）、悬浮窗（`popup.html`）、截图覆盖层（`ocr.html`）、启动提示窗（`notice.html`）；Rust 侧 `ocr`/`document`/`docx`/`vocabulary`/`log`/`vitals` 等新模块；`server/` 中继；`contract/contract.json` 与 `capabilities` |

## 二、总体结果

**结论：功能面未发现缺陷，构建面有 3 项硬缺陷。** 用户可见的功能、布局、无障碍与本地化全部通过；但当前工作区**无法通过 CI**（格式、lint、测试三道门禁均不通过，其中测试为随机失败）。

> **后续更新（2026-09-30 19:05—19:45）：问题 1–5 已全部修复并复验通过，详见第六节。**

| 检查项 | 结果 | 修复后 |
| --- | --- | --- |
| 前端测试 `npm test` | ✅ 292 通过 / 0 失败 | ✅ 292 通过 |
| Rust 测试 `cargo test --locked` | ⚠️ 317 个用例本身全部正确，但**约 10–20% 的运行会失败**（见问题 1） | ✅ 317 通过，连跑 40 次（debug）＋40 次（release）**零失败** |
| 中继测试 `server/` `npm test` | ✅ 142 通过 / 0 失败 | ✅ 142 通过 |
| `cargo fmt --all --check` | ❌ **失败**，9 个文件 29 处（见问题 2） | ✅ 通过（退出码 0） |
| `cargo clippy --all-targets -- -D warnings` | ❌ **失败**，10 处警告被提升为错误（见问题 3） | ✅ 通过（退出码 0） |
| 设置窗口功能 | ✅ 12 个页面全部可切换；翻译卡片、单位换算、界面语言、快捷键录制、导出/导入、日志面板均正常，控制台**零报错** |
| 悬浮窗功能 | ✅ 复制分侧（每次只有一个按钮显示"已复制"并在 1.1 秒后归位）、固定、收藏、截图、设置、语言互换（`translate_text` 带 `zh-CN→en`）、就地改写原文（Ctrl+Enter 与失焦各提交一次，无改动不重复请求）均正确 |
| 截图覆盖层 | ✅ 拖拽生成矩形并回传 `ocr_region`，尺寸提示跟随，小于 8 px 的点击转为取消，交付后忽略后续拖拽，Esc/右键取消 |
| 启动提示窗 | ✅ 渲染与自隐藏正常（约 6.5 秒） |
| 布局（880×690 真实窗口尺寸） | ✅ 深浅两色 × 12 个页面：无横向溢出、无文本裁切、无越界元素；`main` 为 `overflow: auto`，长页面可滚动 |
| 无障碍 | ✅ 四个页面、深浅两色下**无任何无可访问名称的可交互元素**；`document.documentElement.lang` 在中文界面下为 `zh-CN` |
| 本地化 | ✅ 中英文字典各 378 键，零缺键/零多余键/零重复键；占位符 `{0}` 两个字典完全一致 |
| 契约与能力 | ✅ `contract.json` 与 `lib.rs` 注册的命令、与页面 `invoke` 的名称完全一致；`notice` 窗口不含 `listen`，故未被 capability 覆盖是自洽的 |
| 安全基线 | ✅ `tauri.conf.json` 已设置严格 CSP（`default-src 'self'`、`object-src 'none'`、`frame-ancestors 'none'` 等），此前 1.1.x 的 `csp: null` 已消除 |
| 数据文件 | ✅ `history.json`(50) / `vocabulary.json`(2) / `rates.json` / `settings.json`(39 字段，`formatVersion=1`) 全部为合法 JSON |
| 日志隐私 | ✅ `glossy.log` 只记录服务名与错误码，不含用户选中的文本，与 `PRIVACY.md` 一致 |

合计 **751 项自动化测试用例**（Rust 317 + 前端 292 + 中继 142）。

## 三、问题清单

### 问题 1 ｜正常｜Rust 测试随机失败，CI 会不可预测地变红

**位置**：[document.rs](e:/Glossy/src-tauri/src/document.rs:202) 的 `SKIP_PLAIN`/`SEGMENT_CHARS`、[document.rs](e:/Glossy/src-tauri/src/document.rs:194) 的 `PLAN`、[document.rs](e:/Glossy/src-tauri/src/document.rs:1199) 与 [document.rs](e:/Glossy/src-tauri/src/document.rs:1293) 两个测试

**复现与证据**

```
cargo test --release --lib a_word_document_is_opened_paragraph_by_paragraph
  → ok（单独运行必过）

循环运行同一测试二进制 25 次 → 失败 5 次
循环运行同一测试二进制 30 次 → 失败 3 次
失败信息：assertion `left == right` failed
            left: 3
           right: 2
         at src\document.rs:1304  →  assert_eq!(info.segments, 2);
```

**根因**：被检查的文件状态放在**进程级全局**里，测试之间互相覆盖。

- `open()` 每次调用都写入全局 `SKIP_PLAIN`（[document.rs](e:/Glossy/src-tauri/src/document.rs:286)）与 `SEGMENT_CHARS`，并把整份 `Plan` 放进全局 `PLAN`（[document.rs](e:/Glossy/src-tauri/src/document.rs:194)）。
- `a_word_document_...` 依赖"只含数字的段落被跳过"（`SKIP_PLAIN == true`）才得到 2 段，而 `pieces_that_hold_no_letters_are_left_alone_when_the_page_asks` 会调用 `opened_with(..., Some(false))`（[document.rs](e:/Glossy/src-tauri/src/document.rs:1207)）把 `SKIP_PLAIN` 置为 false。
- Rust 测试默认并行；只要那条 `store(false)` 恰好插进本用例的 `open()` 与 `open_word()` 之间，本用例就读到 3 段而失败。同一批全局还让 `PLAN.lock()` 断言存在读到**另一个测试的 Plan** 的隐患（当前表现为偶发，窗口更小）。

**影响**：CI 的 `cargo test --locked` 会间歇性失败（实测约 10–20%），发布流水线不可信；同一个提交重跑可能一红一绿，容易掩盖真实回归，也容易让人习惯性重跑。

**预期修复效果**：把"当前打开的文件"从进程级全局改为随 `Plan`/参数传递（或在命令层持有），使测试彼此隔离；修复后连续 30 次运行应全绿。若短期要止血，可让这批测试共用一个互斥锁串行执行。

### 问题 2 ｜正常｜`cargo fmt --all --check` 不通过

**位置**：[document.rs](e:/Glossy/src-tauri/src/document.rs)（11 处）、[docx.rs](e:/Glossy/src-tauri/src/docx.rs)（4 处）、[vocabulary.rs](e:/Glossy/src-tauri/src/vocabulary.rs)（4 处）、[desktop.rs](e:/Glossy/src-tauri/src/platform/windows/desktop.rs)（3 处）、[hotkey.rs](e:/Glossy/src-tauri/src/platform/windows/hotkey.rs)（3 处）、[lib.rs](e:/Glossy/src-tauri/src/lib.rs)（1 处）、[screen.rs](e:/Glossy/src-tauri/src/platform/screen.rs)（1 处）、[platform/windows/mod.rs](e:/Glossy/src-tauri/src/platform/windows/mod.rs)（1 处）、[selection.rs](e:/Glossy/src-tauri/src/selection.rs)（1 处）

**实际表现**：`cargo fmt --all --check` 报 **29 处**差异（多为长表达式需要换行缩进）。CI 的 `.github/workflows/ci.yml` 第二步就是 `cargo fmt --all --check`，因此**流水线在 lint 之前就中断**。

**影响**：无法通过 CI；后续提交的格式问题也会被这 29 处掩盖。

**预期修复效果**：`cargo fmt --all` 后 `--check` 无输出。

### 问题 3 ｜正常｜`cargo clippy -- -D warnings` 不通过

**位置**（lib 目标 7 条 + 测试目标新增 3 条，共 10 条）

| 文件 | 行 | 警告 |
| --- | --- | --- |
| `src/document.rs` | 234 | variant name ends with the enum's name |
| `src/docx.rs` | 68 | consider using `sort_by_key` |
| `src/ocr/vision.rs` | 70 / 256 | the loop variable is used to index the slice |
| `src/ocr.rs` | 385 | using `map_err` over `inspect_err` |
| `src/platform/screen.rs` | 119 | the loop variable `channel` is used to index `sum` |
| `src/platform/windows/screen.rs` | 68 | field assignment outside of initializer |
| `src/platform/windows/hotkey.rs` | 476 | field assignment outside of initializer |
| `src/vocabulary.rs` | 301 | useless use of `vec!` |

**实际表现**：CI 执行 `cargo clippy --all-targets --locked -- -D warnings`，10 条警告全部被提升为错误，构建失败（`--lib` 7 条、`--lib test` 3 条）。

**影响**：与问题 2 叠加，Rust 侧三道门禁（格式、lint、测试）目前**全部不通过**。

**预期修复效果**：`cargo clippy --all-targets -- -D warnings` 零警告退出。

### 问题 4 ｜轻微｜README 的测试数量已过时

**位置**：[README.md](e:/Glossy/README.md:544)、[README.md](e:/Glossy/README.md:548)、[README.md](e:/Glossy/README.md:1072)、[README.md](e:/Glossy/README.md:1076)

**实际表现**：README 在"快速开始"处两套语言各写一次 `node --test # 288 frontend tests` 与 `cargo test --locked # 260 Rust tests`。实测为**前端 292、Rust 317**（`ROADMAP.md` 第 17/366 行的 317 / 290 / 142 也只差前端 2 项）。同一文件第 720/1215 行另有 `cargo test # 234 tests`。

**影响**：读者按文档核对测试规模时会认为测试没有跑全。

**预期修复效果**：四处数字更新为 292 / 317，`ROADMAP.md` 前端数字更新为 292。

### 问题 5 ｜轻微｜浏览器预览层缺少"识别屏幕文字"的热键槽位，导致设置窗口在预览下自相矛盾

**位置**：[bridge.js](e:/Glossy/src/js/bridge.js:361) 的 `capture_status` 模拟

```js
hotkeys: [
  { slot: "translate", spec: "Ctrl+Alt+C", error: null },
  { slot: "settings",  spec: "Ctrl+Alt+G", error: null },
  // 缺 { slot: "ocr", ... }
],
```

**实际表现**：在浏览器里打开设置窗口 → 「常规」→ 快捷键区，「识别屏幕文字」一行的**输入框显示着 `Ctrl+Alt+Q`，下面一行却写着"未设置全局快捷键。点击「录制」来设置。"**。真实后端（[hotkey.rs](e:/Glossy/src-tauri/src/platform/windows/hotkey.rs:324) 的 `status()` 返回 `translate`/`settings`/`ocr` 三个槽位）不会出现该矛盾，故只在预览下发生。

**影响**：预览是项目自己提供的开发/验收面（`bridge.js` 的注释与 292 项测试都依赖它），这一处不一致会让人误以为 OCR 快捷键失效，也掩盖了真实回归——一项"预览里看起来正常"的改动可能让运行时三行提示全错。此外预览中 `Alt+Q` 之类用户实际设置值永远显示为默认值。

**预期修复效果**：`capture_status` 模拟补齐 `ocr` 槽位（并从 `settings.hotkeyOcr` 读取），预览下三行提示与输入框一致。

### 观察项（非缺陷，不计入问题清单）

1. **文档翻译按设计处于关闭状态**：侧栏该项 `disabled` 且带"开发中"角标，`app.js` 的 `CLOSED_PAGES = ["document"]` 会把它回落到「文本翻译」。而 `document.rs`/`docx.rs` 与相关 i18n 词条均已存在，`CHANGELOG.md` 也写明"Document translation is closed while it is rebuilt"。问题 1 的失败用例正属于这个重建中的模块，与"重建中"的说法一致。
2. **上游回退**：用户的 `settings.json` 配置 `provider = baidu`，而 `glossy.log` 在 2026-09-29 连续 5 次记录 `the relay answered with youdao instead of baidu`。本次探针显示内置中继 `/v1/health` 同时提供 `["baidu","youdao"]`，且以 `vendor: baidu` 请求 `/v1/translate` 时正常返回百度译文，说明是当日该上游故障、由中继走过了它。App 正确地把这一情况写进日志并在卡片底部标出"回答了的是有道"，即 1.7.1 新特性按预期工作；日志未带原因码是因为返回里没有 `attempts`（中继版本较旧），属于部署滞后而非应用缺陷。
3. **预览中的"所在句子"内容异常**（如"所在句子：A dog barked. keeps the whole street awake at night."）来自 `bridge.js` 的固定拼装，是模拟数据，非应用逻辑。
4. **测试数据文件**：`settings.json.bak` 只有 23 个字段而现文件有 39 个，是 2026-09-20 的旧备份，属正常。

## 四、优化建议

1. **优先修问题 1**：全局可变状态是这次随机失败的唯一来源。把 `SKIP_PLAIN`/`SEGMENT_CHARS` 变成 `open()` 的参数并随 `Plan` 携带，`PLAN`/`FINISHED` 改为由命令层持有的状态，测试即自然隔离；否则建议给这批用例加串行锁。修复后请连续跑 30 遍确认稳定。
2. **一条命令清掉问题 2、3**：`cargo fmt --all` 后按 clippy 建议逐条修（`sort_by_key`、`inspect_err`、循环改用迭代器、`#[derive(Default)]` 初始化、去掉多余的 `vec!`），再把 `cargo fmt --all --check` 与 `cargo clippy -- -D warnings` 纳入本地提交前检查，避免一次堆积 39 处。
3. **让 CI 的测试步骤重试一次并标注**：即便根因修好，Windows 上的鼠标钩子/语音/OCR 相关测试天然带时序性；`cargo test` 失败时自动重跑一次并打印两次结果，可以把"真回归"与"偶发"分开。
4. **预览层补齐**（问题 5）：`capture_status` 补齐 `ocr` 槽位，最好直接由模拟设置生成三个槽位，这样改设置后预览也能跟着变。
5. **同步文档**（问题 4）：README 四处与 ROADMAP 一处；建议在 CI 里加一步"README 声明的测试数 = 实际数"的校验，避免再次漂移。
6. **可选**：为悬浮窗的原文/译文块加 `lang` 属性（当前只有 `<html lang>`），屏幕阅读器可用正确语音朗读原文与译文——这是本次唯一发现的、语义层面的无障碍可改进点。

## 五、未覆盖项
以下内容本次未验证，如需完整测试请单独安排：

- **真实屏幕框选识别（OCR）整链路**：本次只验证了覆盖层交互（拖拽/取消/尺寸提示）与 74 MB 识别资源已安装，未在真实多窗口桌面上拖动框选并核对识别结果。用户已安装中/英/日识别包，说明该路径此前可用。
- **托盘图标菜单**（打开设置、退出等）与 `notice` 窗口被再次唤起后的表现（上次检查在该路径发现过缺陷）。
- **更新检查与安装的真实远端流程**（当前构建未放置更新公钥，界面已如实说明）。
- **真实网络下的翻译错误分支**：额度用尽、超时、上游不可达对应的卡片文案。
- **多显示器与非 100% 缩放**下的提示窗定位、悬浮窗尺寸换算与截图覆盖层坐标。
- **长时间运行的内存/句柄泄漏**：`vitals.rs` 的泄漏判定逻辑本次只通过单元测试确认，未做数小时实测。

## 六、修复记录（2026-09-30 19:05—19:45）

问题 1–5 已全部修复，修复后五项验证全部通过。

### 问题 1：把"当前打开的文件"的状态从进程级全局中取出

改动集中在 `src-tauri/src/document.rs`：

- 删除全局 `SEGMENT_CHARS`、`SKIP_PLAIN` 及其读取函数，改为一个 `Copy` 的 `Splitting { limit, skip_plain }` 结构，由 `Splitting::asked(...)` 从页面传来的两个选项算出，并作为参数沿 `Format::split` → `paragraphs`/`markdown`/`subtitles`/`open_word` → `flush` → `chunk` → `cut_point` 一路传递。**分片选项不再有任何全局状态**，两个文件同时打开也不会互相改变切分长度。
- `open()` 的返回值改为 `(Info, Plan)`，把刚打开的计划直接交给调用者。生产代码只取 `Info`；测试不再从全局 `PLAN` 里读别人的文件，而是用自己拿到的计划，`PLAN` 仅保留"应用当前打开的一份"这一真实职责。
- 测试侧：新增 `opened_plan()` 辅助函数返回 `(Info, Plan)`，`longest_piece(&plan)` 改为接收计划，原先四处 `PLAN.lock()` 断言全部改读本地计划；新增 `defaults()` 供只测切分的用例使用。

### 问题 2、3：格式化与 lint

- `cargo fmt --all` 修掉 9 个文件的 29 处格式差异。
- 逐条修掉 10 处 clippy 警告：`sort_by` → `sort_by_key`（`docx.rs`）、`map_err` → `inspect_err`（`ocr.rs`）、循环改用 `iter_mut().enumerate()`／`zip`（`ocr/vision.rs`、`platform/screen.rs`）、先赋值后赋字段改为结构体字面量加 `..Default::default()`（`platform/windows/screen.rs`、`platform/windows/hotkey.rs`）、去掉多余的 `vec!`（`vocabulary.rs`）、`Progress::Progress` 变体改名为 `Progress::Underway`（`serde` 的 `rename = "progress"` 保持不变，发给前端的 `state` 字段仍是 `progress`）。

### 问题 4：文档数字

README 四处、ROADMAP 两处：前端 288 → **292**、Rust 260/234 → **317**（ROADMAP 的前端 290 → 292）。

### 问题 5：预览层补齐热键槽位

`src/js/bridge.js` 的 `capture_status` 模拟改为按槽位返回三项，并直接读预览保存的设置：`translate` / `settings` / `ocr`。现在预览中录制或清除 OCR 快捷键后，该行提示会跟着变（实测 `Ctrl+Alt+Q` → 录制 `Ctrl+Shift+F9` → 清除后显示"未设置全局快捷键"）。

### 修复后的验证结果

| 验证项 | 命令 | 结果 |
| --- | --- | --- |
| 格式 | `cargo fmt --all --check` | ✅ 退出码 0 |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | ✅ 退出码 0（仅剩 `build.rs` 的 GNU 链接器警告，clippy 说明该 lint 不受 `-D warnings` 影响） |
| Rust 测试 | `cargo test --locked` | ✅ 317 通过 / 0 失败 |
| 稳定性 | 测试二进制连跑 40 次（debug）＋ 40 次（release） | ✅ **0 次失败**（修复前 25 次失败 5 次、30 次失败 3 次） |
| 前端测试 | `npm test` | ✅ 292 通过 / 0 失败 |
| 中继测试 | `server/` 的 `npm test` | ✅ 142 通过 / 0 失败 |
| 二进制构建 | `cargo build --locked` | ✅ 通过 |
| 预览复验 | 浏览器中录制/清除 OCR 快捷键 | ✅ 输入框与提示一致 |

