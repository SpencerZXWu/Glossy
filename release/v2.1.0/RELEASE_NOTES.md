[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that lets the window be something other than grey, that opens with a
guided tour of what it does, that gives Ctrl+Enter back to the field it is typed in,
and that lets the text be translated with an account of your own.

### Added

- **Two translation channels that answer with an account of your own.** The **General**
  page has a **Your own API** panel: a Baidu APP ID and key from the 翻译开放平台 console,
  and the address, key and model of any service that speaks the OpenAI chat completions
  protocol. Each pair unlocks an entry in the **Translation service** dropdown —
  `api-baidu` and `api-openai` — and the entry is only there while its fields are filled
  in: emptying them takes it out of the list again, settles the two language lists on the
  engine that still works, and falls the choice back on `cloud-baidu`. The rule is applied
  in the window and in the backend, so the list that is shown and the engine that answers
  can never disagree. The credentials are DPAPI protected for the Windows login that typed
  them before they reach the disk, an export carries the same protected shape, and a key
  encrypted for another login is dropped rather than kept. Both channels send the text from
  this machine straight to the service, so Glossy's relay is not in the path at all.
- **The channel is chosen on the page the text is typed on.** The **Translate text** page
  now carries the same **Translation service** list as the **Language** page, filled from
  the same entries, so switching from a service that is failing to one that is not takes one
  click from the box the text is in.
- **A guided tour on the first launch.** The window that only ever opens once now
  explains itself: four steps — selecting a text, typing or pasting one, reading a
  screenshot, and downloading what has to be on this machine — each with its own
  short animation and two lines saying what to do and what happens. The scenes are
  drawn from the same tokens as the window behind them rather than recorded, so
  they follow the palette, the accent and the interface language, and a reader who
  asked Windows for less motion gets the last frame of each scene instead of no
  scene at all. The rail, the arrow keys, Escape and a play/pause button work the
  way the four steps deserve, and **Watch the guide again** on the **General**
  page plays it from the start.
- **The state in the title bar is a switch.** *Listening* used to be a label that only said
  what the app was doing; it now takes a click and turns the selection capture off and on
  from wherever the window is, and a second click brings it back. It is the same setting the
  master switch on **General** drives — the label, the dot's colour and the switch itself
  never disagree, whichever of the two is used — and it is named as the switch it is, so a
  screen reader reads its state rather than a colour.

- **Six palettes, and an accent colour of your own.** The **Colours** page used to offer one
  choice — follow Windows, always light, always dark — and now offers three: the mode, the
  palette, and the accent. The palettes are **WinUI** (what the app has always used), **warm
  paper**, **Nord**, **Solarized**, **Dracula** and **true black**, and each states both
  halves, so the mode still decides light or dark and the palette decides the hues. Every
  card in the picker previews itself: its two halves carry that palette's own tokens, so what
  a card shows is what choosing it does, and the light and dark halves are both visible
  whichever mode the window is in. The row under it takes an accent — hand the choice back to
  the palette, or name one of eight colours, or pick any colour at all with the system's own
  picker — and the whole app follows it, selection colour and caret included.
- **The window frame follows the palette.** Windows draws the title bar and the frame, so a
  palette states its window colour to them through DWM (`set_window_surface`), and choosing
  one takes the window off the Mica backdrop: Mica is tinted by the desktop, which is exactly
  what a palette with a window colour of its own cannot have. The default palette keeps both,
  as it always has.

### Changed

- **The Paste button is gone from the Translate text page.** It was a second way to do what
  Ctrl+V does, and the channel list deserves the room more than the button did.
- **Exporting the settings carries the credentials, protected.** `Export…` used to write a
  file with no key in it at all; it now writes the credentials as DPAPI blobs, which keeps
  the file readable, and keeps it safe to hand to somebody, without handing over a key that
  works. An import unlocks what it can and drops a key that belongs to another Windows login.
- **The interface language moved to the Language page.** It was a select in the title bar,
  where it sat next to the state of the capture and read as a translation setting rather than
  one of the window's own; it now has a labelled row on the page that is named after it, in
  both languages.
- **The palette layer is one layer of the token system.** Every colour in a palette lives in
  `tokens.css` beside the two the app already had, and a palette states only the colours that
  carry meaning — the three opaque surfaces, the four text tiers, the strokes, the accent and
  the three states. The layering fills stay the white and black alphas they always were, so a
  palette changes the colours and never the depth system. `tests/tokens.test.js` re-measures
  the file: every text tier and the label on an accent fill has to clear 4.5:1 in all twelve
  palette-and-mode pairs, the WinUI palette has to stay identical to the `:root` defaults it
  is stated beside, and the picker has to offer exactly the palettes the stylesheet defines.
- **Text selection, the caret and the accent of a checkbox come from the palette too.** They
  were WebView2's defaults, which belong to no design system, and a themed window that keeps
  them reads as half-finished.

### Fixed

- **Dragging across a picture no longer copied it and threw the selection away.** A drag was
  taken as *the user has selected text* without asking whether anything had been selected, so
  dragging across images to pick several of them in WeChat — or in any program that lets a
  drag move or select something that is not text — had Glossy press Ctrl+C into the program,
  copying the picture and cancelling the gesture the user was in the middle of. A drag or a
  double click now asks the focused element through UI Automation whether it has a text
  selection at all, and only presses the shortcut when the answer is yes or when the element
  cannot say: a control with text and nothing selected is left alone entirely, while the
  selection shortcut is never affected, because the user pressed it on purpose.
- **A Windows contrast theme is no longer painted over.** The `prefers-contrast` block was
  stated on `:root` alone, which every `:root[data-theme="dark"]` rule outranked — so a
  contrast scheme with a dark Windows theme behind it had its palette replaced by the app's
  dark one, which is the opposite of following the setting.
- **A custom accent takes the text colour that reads better on it.** The choice was made at a
  luminance of 0.5, which leaves every mid-tone accent — amber, sky, coral — with white text
  at about 2:1 on it. The crossover is 0.179, and both halves are compared rather than guessed.
- **Ctrl+Enter in the card's grey original replaced the text behind it instead of translating
  the correction.** The card's write-back key is registered with Windows rather than with the
  page, and a registered shortcut is delivered to Glossy before the window that has the caret —
  so with the caret in the editable original, Ctrl+Enter never reached the field that promises to
  translate what is typed in it. What happened instead was the translation being pasted back over
  the original text in the program behind the card. The field now tells the card when it holds the
  caret, and the card lets go of the accelerator for as long as it does, which is the pair the
  field's own hint and the button's label already described: Ctrl+Enter translates the correction
  inside the field, and writes the translation back anywhere else on the card.

<a id="zh-cn"></a>

## 中文

这个版本让窗口不再只有一种颜色，首次启动时会先用一段引导讲清楚它是做什么的，把 Ctrl+Enter 还给了它所在的那个输入框，并且让你可以用自己的账号来翻译。

### 新增

- **两个「自带账号」翻译渠道。** 「常规」页多了一块**自填 API**：百度翻译开放平台的 APP ID 与密钥，以及任何兼容 OpenAI chat completions 协议的服务的地址、密钥和模型。每一对填好，**翻译渠道**下拉框里就多出一个条目——`api-baidu` 和 `api-openai`；字段清空，它又离开列表，两个语言列表回落到还能用的那个引擎，当前选择回落到 `cloud-baidu`。这条规则在窗口和后端各写一遍，所以列表里显示的和真正去翻译的永远不会不一致。密钥在落盘前用 `DPAPI` 按当前 Windows 登录加密，导出设置带的是同样加密后的形状，属于另一个登录的密钥会被丢弃而不是留着。两个渠道都把原文从本机直接发往该服务，Glossy 的中转服务完全不在链路上。
- **在输入原文的那一页就能换渠道。** **文本翻译**页现在也有和**语言**页相同的**翻译渠道**列表，条目来自同一份数据，所以在原文框边上一次点击就能从正在失败的渠道换到能用的那个。
- **首次启动时的新手引导。** 那个一生只打开一次的窗口现在会自己解释自己：四步——划词翻译、输入或粘贴原文、截图翻译，以及把必须留在这台机器上的资源下载下来——每一步各有自己的小动画，以及两行说明：要做什么、会发生什么。场景不是录屏，而是用窗口自己那套 token 画出来的示意图，因此它们跟随配色方案、强调色和界面语言；而在 Windows 里要求「减少动态效果」的读者看到的是每个场景的最后一帧，而不是一片空白。步骤栏、方向键、Esc 和播放/暂停按钮都按这四步该有的样子工作，**常规**页的**再看一次引导**可以随时从第一步重放。
- **标题栏里的状态变成了开关。** "监听中"过去只是一个说明应用在做什么的标签；现在它可以点击，能在任何位置关掉或打开划词捕获，再点一次恢复。它和**常规**页的总开关是同一个设置——文字、圆点颜色和开关本身永远一致，无论用哪一个改——而且它按开关本身命名，所以读屏软件读的是状态而不是颜色。
- **六套配色，以及一个你自己的强调色。** 「配色」页过去只有一个选择——跟随 Windows、总是浅色、总是深色——现在有三个：模式、配色方案和强调色。配色方案有 **WinUI**（应用一直以来的颜色）、**暖纸**、**Nord**、**Solarized**、**Dracula** 和 **纯黑**，每套都写明两种模式，因此模式仍然决定浅色还是深色，配色方案决定色相。选择器里的每张卡片都预览自己：它的两半用的是该方案自己的 token，所以卡片显示的就是选中后的效果，而且无论窗口当前是哪种模式，浅色和深色两半都看得见。下面那一行用来选强调色——交还给配色方案、从八个颜色里挑一个，或者用系统取色器选任意颜色——整个应用都会跟着变，包括选中文字的颜色和光标。
- **窗口边框跟着配色方案走。** 标题栏和边框是 Windows 画的，因此配色方案通过 DWM（`set_window_surface`）把窗口颜色告诉它们；选中 WinUI 以外的方案还会让窗口不再使用 Mica 背景：Mica 的颜色来自桌面，而自带窗口颜色的方案没法有它。默认配色两者都照旧保留。

### 变更

- **「文本翻译」页的粘贴按钮已移除。** 它只是 Ctrl+V 的另一个入口，而那块位置更适合放翻译渠道列表。
- **导出设置会带上凭据，而且是加密后的。** 过去 `导出…` 写出的文件里完全没有密钥；现在它把凭据写成 `DPAPI` 加密块，文件仍然可读、可以放心交给别人，却不会交出一把能用的密钥。导入时会解锁能解锁的，属于另一个 Windows 登录的密钥则被丢弃。
- **界面语言移到了「语言」页。** 它原来是标题栏里的一个下拉框，紧挨着捕获状态，读起来像翻译设置而不是窗口自己的设置；现在它是那个以它命名的页面上一行带标签的设置，两种界面语言都有。
- **配色层是 token 体系里的一层。** 每套配色的颜色都和原有的两套一起住在 `tokens.css` 里，而且一套配色只写有含义的颜色——三个不透明表面、四级文字、描边、强调色和三种状态。叠加用的填充仍然是原来的黑白透明度，所以配色换的是颜色，从不换深度体系。`tests/tokens.test.js` 会重新测量这份文件：十二个「配色 × 模式」组合里，每一级文字和强调色填充上的标签都必须达到 4.5:1，WinUI 必须和它旁边写着的 `:root` 默认值逐字相同，选择器提供的方案必须和样式表定义的完全一致。
- **选中文字的颜色、光标和复选框的强调色也来自配色方案。** 它们原本是 WebView2 的默认值，不属于任何设计体系，而一个用了配色却留着这些的窗口看起来只做了一半。

### 修复

- **拖选图片不再把它复制走，也不再打断选中。** 过去一次拖动会被直接当成"用户选中了文字"，从不问到底有没有选中东西，于是在微信里拖选多张图片——或在任何允许拖动移动或选中非文字内容的程序里——Glossy 会向那个程序按下 Ctrl+C，于是图片被复制，用户正在做的操作被打断。现在拖动或双击会先通过 UI Automation 问当前焦点控件到底有没有文字选区，只有在答案是"有"、或者控件答不上来时才按那个快捷键：有文字但没选中的控件完全不碰，而划词快捷键从不受影响，因为那是用户主动按下的。
- **Windows 高对比主题不再被覆盖。** `prefers-contrast` 那段只写在 `:root` 上，而任何 `:root[data-theme="dark"]` 规则都能压过它——于是一个高对比方案配上 Windows 的深色主题时，它的配色会被应用自己的深色配色替换掉，正好和"跟随设置"相反。
- **自定义强调色会选在它上面读起来更好的文字颜色。** 过去这个判断用的是 0.5 的亮度，于是所有中间调的强调色——琥珀、天蓝、珊瑚——上面的白字只有大约 2:1。真正的分界是 0.179，而且是两边都算一遍而不是猜。
- **在卡片里那段灰色原文上按 Ctrl+Enter，过去会把译文粘回卡片背后的原文，而不是翻译这段修改。** 卡片用来写回原文的那个键是向 Windows 注册的，而不是注册在页面上；注册过的快捷键会在拥有光标的窗口之前先送到 Glossy——所以当光标在那个可编辑的原文里时，Ctrl+Enter 根本到不了那个承诺"翻译你输入的内容"的输入框，实际发生的是译文被粘回卡片背后那个程序的原文上。现在输入框会告诉卡片光标在它这里，卡片在光标停留期间放开那个快捷键，这正是输入框自己的提示和按钮标签一直描述的配对：框里按 Ctrl+Enter 翻译修改后的文字，卡片其他位置按则把译文写回原文。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
