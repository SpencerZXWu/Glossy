# Glossy 快速检查报告

## 一、测试基本信息

| 项目 | 内容 |
| --- | --- |
| 应用名称 | Glossy（选中即译桌面工具） |
| 版本号 | 1.1.2（`package.json` / `Cargo.toml` / `tauri.conf.json` 三处一致） |
| 测试环境 | Windows 本机；真实二进制 `src-tauri/target/release/glossy.exe`（构建时间 2026-09-22 19:54）+ 浏览器预览 `http://localhost:8913`（`src/js/bridge.js` 内存模拟层，无需 Tauri） |
| 测试类型 | 快速检查：自动化测试套件 + 三个窗口功能/布局 + 真实程序端到端链路 |
| 测试时间 | 2026-09-22 20:00—20:20 |
| 检查范围 | 设置窗口（`index.html`）、翻译悬浮窗（`popup.html`）、启动提示窗（`notice.html`）；全局热键 → 选中 → 翻译 → 悬浮窗真实链路；单实例约束；i18n 字典一致性 |

## 二、总体结果

**结论：基本通过。核心链路真实可用，未发现崩溃或数据错误；发现 1 个功能异常（正常级）和 3 个轻微问题。**

| 检查项 | 结果 |
| --- | --- |
| 前端逻辑测试 | ✅ 177 通过 / 0 失败（约 4.8 s） |
| Rust 逻辑测试 | ✅ 189 通过 / 0 失败 |
| 翻译代理服务测试（`server/`） | ✅ 74 通过 / 0 失败 |
| 自动化测试合计 | ✅ **440 项全部通过** |
| 设置窗口功能 | ✅ 翻译卡片、单位换算开关、清空、忽略名单增删、导出、检查更新、界面语言切换全部正常，控制台**零报错** |
| 悬浮窗功能 | ✅ 复制（状态变为已完成）、固定（`aria-pressed` 切换）、语言互换、朗读状态机均正常 |
| 启动提示窗渲染 | ✅ 340×104，内容 324×88，无裁切、无溢出 |
| 真实端到端链路 | ✅ 见下 |
| 单实例约束 | ✅ 第二个实例退出码 0，不产生重复进程，由运行中实例接管"显示自己" |
| 深色模式对比度 | ✅ 次要文字 `rgba(255,255,255,.565)` 落在 `rgb(32,32,32)` 上，约 6.6:1，满足 WCAG AA |
| i18n 字典完整性 | ✅ 中英文字典各 202 键，零缺键、零多余键 |
| 设置窗口渲染 | ✅ 11 个分区齐全，真实窗口内容尺寸 760×660（`GetWindowRect` 775×697 含 DWM 阴影边框） |

### 真实端到端链路验证（v1.1.2 二进制）

1. 启动 `glossy.exe`，进程存活，内存约 40 MB，49 线程。
2. 剪贴板写入 `The quick brown fox jumps over the lazy dog.`，发送全局热键 `Ctrl+Alt+C`。
3. 约 6 秒后窗口中新增 **`Glossy Popup` 376×230**（356 px 卡片 + 20 px 内边距，与 `popup.rs` 自调整逻辑一致）→ **悬浮窗真实弹出**。
4. `%APPDATA%\com.glossy.translator\history.json` 新增记录 `id: 255`：`The quick brown fox jumps over the lazy dog.` → `敏捷的棕色狐狸跳过了懒惰的狗。`，`provider: "baidu"`，`sourceLang: en`，`targetLang: zh-CN` → **真实翻译成功并写入历史**。

即"热键 → 读取选中 → 代理翻译 → 悬浮窗展示 → 历史落库"整条链路在真实程序中跑通。

## 三、问题清单

### 问题 1 ｜正常｜启动提示窗被再次唤起后不再自动关闭

**位置**：[notice.rs](e:/Glossy/src-tauri/src/notice.rs:46) 的 `show()`、[lib.rs](e:/Glossy/src-tauri/src/lib.rs:426) 的 `show_launch_surface()`、[notice.js](e:/Glossy/src/js/notice.js:7) 的 `LINGER_MS`（计时器在 [notice.js](e:/Glossy/src/js/notice.js:44) 启动）

**复现步骤**

1. 启动 `glossy.exe`，右下角出现"Glossy 正在后台运行"提示窗。
2. 不做任何操作，等待提示窗自行消失（实测约 7 秒，与 `LINGER_MS = 6500` 一致，行为正确）。
3. 在设置窗口处于关闭状态时，再次启动 `glossy.exe`（例如再次双击开始菜单/桌面快捷方式）。

**实际表现**：第二实例触发运行中实例执行 `show_launch_surface()` → 主窗口不可见 → 调用 `notice::show()`，该函数**只调用 `window.show()`**；而控制自动隐藏的 6.5 秒计时器位于前端 `notice.js`，且只在页面加载时启动一次，此时早已触发完毕。实测重新弹出的提示窗**跟踪 30 秒仍不消失**，上一次残留更是超过 7 分钟。

**预期表现**：提示窗每次显示都应在 6.5 秒后自动隐藏。

**影响**：无边框、置顶、且被设置为不可聚焦（`WS_EX_NOACTIVATE`）的提示卡片会**长期停留在屏幕右下角通知区域上方**，遮挡其它窗口内容，且不会获得焦点提示用户处理。用户只能通过点击它（会顺带打开设置窗口）才能让其消失。注意：通过通知区域图标打开设置走的是 `tray::show_main`，不受影响；本问题仅在"重复启动程序"这条路径上触发，而这条路径恰恰是普通用户想重新打开设置时最自然的操作。

**预期修复效果**：`notice::show()` 每次显示时重置前端计时器（例如后端 `emit` 一个"已显示"事件，由 `notice.js` 重新 `setTimeout`），或把自动隐藏职责整体移到后端定时器；修复后重复启动程序时提示窗同样在 6.5 秒后自动消失。建议同时补一条覆盖"二次显示"的回归测试（当前 `notice.rs` 仅有一条 `contains` 测试）。

### 问题 2 ｜轻微｜提示窗关闭按钮的无障碍名称未本地化

**位置**：[notice.html](e:/Glossy/src/notice.html:18)

```html
<button class="close" id="close" type="button" data-i18n-title="notice.close" title="Dismiss" aria-label="Dismiss">
```

**实际表现**：界面语言为中文时，`title` 被正确地改写为"关闭"，但 `aria-label` 仍为英文 `Dismiss`。原因是 `Glossy.i18n.apply()`（[i18n.js](e:/Glossy/src/js/i18n.js:524)）只处理 `data-i18n`、`data-i18n-placeholder`、`data-i18n-title` 三类属性，从不处理 `aria-label`，而 [notice.js](e:/Glossy/src/js/notice.js:16) 的 `applyLanguage()` 也没有像 [popup.js](e:/Glossy/src/js/popup.js:82)（第 82—87、154 行）那样手动补写 `aria-label`。

**预期表现**：无障碍名称随界面语言变化。

**影响**：中文界面下屏幕阅读器会念出英文 "Dismiss"；鼠标悬停提示为中文而辅助技术名称为英文，二者不一致。同类静态英文 `aria-label` 在 `popup.html`、`index.html` 中均已由 JS 在运行时改写，只有本处遗漏。

**预期修复效果**：中文界面下该按钮的辅助技术名称为"关闭"，与 `title` 一致。

### 问题 3 ｜轻微｜界面语言选择器的提示与无障碍名称未本地化，且字典键 `ui.lang` 成为死键

**位置**：[index.html](e:/Glossy/src/index.html:23)

```html
<select id="uiLang" aria-label="Interface language" title="Interface language">
```

**实际表现**：该 `<select>` 既没有可见的 `<label>`，也没有 `data-i18n-title`，因此它的两个名称来源（`title` 提示、`aria-label` 无障碍名称）在任何界面语言下都恒为英文 "Interface language"。字典里中英文均已存在的 `ui.lang`（`"Interface language"` / `"界面语言"`，见 [i18n.js](e:/Glossy/src/js/i18n.js:11) 与 [i18n.js](e:/Glossy/src/js/i18n.js:235)）在 HTML 与 JS 中**无任何引用**，属于死键——这也说明该控件原本就打算使用这个键。

**预期表现**：中文界面下提示与无障碍名称均为"界面语言"。

**影响**：中文界面下语言切换控件缺少中文提示，屏幕阅读器念英文；同时存在一个永不生效的字典键，容易在后续维护中被误认为已覆盖。

**预期修复效果**：为 `#uiLang` 增加 `data-i18n-title="ui.lang"`（并让 `apply()` 支持 `data-i18n-aria-label`，见建议 2），`ui.lang` 不再是无引用键。

### 问题 4 ｜轻微｜英文文案复数写法偷懒

**位置**：`"ignored.count"`（[i18n.js](e:/Glossy/src/js/i18n.js:45)）、`"source.count"`（[i18n.js](e:/Glossy/src/js/i18n.js:58)）

**实际表现**：英文使用 `"{0} program(s) ignored."`、`"{0} source language(s) allowed..."` 这种 `(s)` 写法，单个条目时也会显示 "1 program(s) ignored."。

**影响**：英文界面下的文案显得未经打磨，与项目其它部分（隧道式提示、完整徽标文案）的完成度不一致。

**预期修复效果**：改为按数量选择单/复数形式（如 `1 program ignored` / `2 programs ignored`），中文文案不受影响。

### 观察项（非缺陷）

1. **设置窗口为单列长滚动**：11 个 `.panel` 分区自上而下堆叠，在 620 px 最小宽度下无横向溢出，但内容高度约 3,100 px（约 4.7 个视口高）。`ROADMAP.md` 的 v1.1.0 已计划"设置窗口改版"（左侧导航/搜索），属**计划内待办**，本次不作为缺陷计。
2. **`tauri.conf.json` 仍为 `"csp": null`**：`ROADMAP.md` v1.2.0 已列为计划项，同样不计为缺陷。
3. **`ROADMAP.md` 的测试数量已过时**：文档写"Rust 160 个测试、前端 155 个测试"，实测为 Rust 189、前端 177（`server/` 74 与文档一致），建议更新文档中的两处表格（英文第 17 行、中文第 240 行）。
4. **`.check` 复选框为 15×15 px 原生控件**：已设置 `accent-color: var(--accent)` 且主题跟随，属有意为之的设计选择，非缺陷。

## 四、优化建议

1. **修复问题 1（优先级最高）**：让提示窗的自动隐藏与"显示"动作解耦——在 `notice::show()` 中向前端发一个事件，由 `notice.js` 在每次收到该事件时重新启动 `LINGER_MS` 计时器；或在后端持有隐藏定时器，避免"显示"与"隐藏"职责分处两端却只初始化一次。
2. **让 i18n 覆盖无障碍属性**：在 `i18n.apply()` 中增加 `data-i18n-aria-label` 支持，可一次性消除问题 2、3，并让 `popup.js`/`app.js` 中现有的 10 余处手动 `setAttribute("aria-label", ...)` 有统一替代方案。
3. **补齐字典键的使用或删除**：`ui.lang` 接上 `#uiLang`；如有其它无引用键，建议加一条"字典键必须被引用"的前端测试，防止再次出现死键。
4. **英文复数**：抽一个 `plural(count, one, other)` 辅助函数，替换全部 `(s)` 写法。
5. **设置窗口改版（已在路线图中）**：可将 11 个分区按"翻译 / 触发 / 外观 / 数据"分组并加左侧导航，缩短单次滚动距离。
6. **CI 输出噪音**：GNU 工具链链接测试二进制时会打印 `corrupt .drectve at end of def file`、`.rsrc merge failure: multiple non-default manifests / duplicate leaf: type: 10 (VERSION)` 等警告并导致 `cargo test` 退出码非 0（测试本身仍全部通过）。建议在 CI 中显式区分"链接警告"与"测试失败"，避免后续误判为红。

## 五、未覆盖项

以下内容本次未验证，如需完整测试请单独安排：

- 真实鼠标选中文字触发翻译（本次用热键路径覆盖了同一后端链路，但未模拟拖拽/双击选中）
- 剪贴板还原的实际效果（`restoreClipboard` 选项）
- 托盘图标菜单各项的交互（打开设置、退出等）
- 更新检查/安装的真实远端流程
- Windows 亚克力（Mica/Acrylic）材质在透明窗口上的实际观感与性能
- 多显示器、非 100% 缩放下提示窗定位与悬浮窗尺寸换算
- 语言包、字体缩放、悬浮窗不透明度等外观选项的极端取值
