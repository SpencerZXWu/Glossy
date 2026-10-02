[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where a channel that stopped answering says so — and where the card can take the
picture itself. Nothing about which engine is asked first changed; what changed is that the
app no longer looks as if it had moved the choice by itself.

### Added

- **A camera button in the card.** Reading a rectangle off the screen no longer needs the
  keyboard: the button sits in the card's header, next to the pin and the wordbook star. The
  card itself is taken off the screen first — it is always on top and normally sits right next
  to what the user wants to read — and the answer, or the reason there is none, comes back to
  the same card, exactly as it does for `Ctrl+Alt+Q`.
- **The relay reports which backends it walked past.** `POST /v1/translate` now answers with
  `attempts`: one `{vendor, code}` per upstream that refused, in the order they were tried, so
  a deployment that holds both keys can say why the one that was asked for stepped aside
  instead of quietly answering with the other one's translation. The relay also writes it to
  its own log (`console.warn`), where the operator — the only one who can fix a vendor's
  credentials or quota — can see it. A deployment older than this build simply leaves the
  field out, which the app reads the same way.

### Changed

- **A button that has a shortcut says so in its tooltip.** The camera and the settings buttons
  in the card, and the screenshot button in the settings window, now name the combination that
  does the same thing. The three shortcuts are recorded by the user, so the tooltip is built
  from the stored value every time the window is drawn rather than written into the markup: it
  always names the key that works now, and a field the user cleared leaves the tooltip with the
  action alone.

### Fixed

- **The Glossy mark in the settings window's card was 128 pixels square.** Nothing in
  `app.css` gave the mark a size, so the card drawn on the Translate page used the size of the
  file itself while the floating popup drew the same mark at 13 pixels. It is 13 pixels in both
  windows now, faded in the same way, and a contrast theme lifts that fade in both.
- **The fallback list follows the service chosen above it.** The chosen service is always
  asked first, so a copy of it in the list below was a second attempt on the same backend —
  and it pushed one engine out of the list altogether: switching the first entry from Baidu
  to Youdao left Youdao listed twice and Baidu nowhere. The list is now rebuilt from the
  choice: every other engine once, in the order the user put them in, with an engine that
  was not in the list yet landing at the end.
- **A card whose answer came from another engine says which one — and why.** The relay in
  front of Baidu and Youdao walks its own list of backends when the one it was asked for
  fails, which is what made a card read "Youdao" while the settings still said Baidu, with
  the fallback list switched off and nothing to explain it. The card names the engine that
  answered in its footer, and the line under it names the engine that did *not*: "Baidu
  Translate did not answer — its allowance is used up", with the reason the relay gave when it
  gave one (a code this build does not know leaves the line at "Baidu Translate did not
  answer"). The name in that line is the engine (`Baidu Translate`) rather than the identifier
  of the entry of the channel list, which is what it used to print — and it used to say that
  the engine that *failed* was the one that answered. The same line goes into `glossy.log`.

<a id="zh-cn"></a>

## 中文

这一版让「不再应答的渠道」自己说明情况——卡片也能自己拍照了。哪个引擎先被询问没有变；变的是：应用不再看起来像是自己改了你的选择。

### 新增

- **卡片上多了一个相机按钮。** 框选屏幕取字不再需要键盘：按钮就在卡片的标题栏里，紧挨着「固定」和「收藏」。卡片本身会先离开屏幕——它始终置顶，而且通常就贴在你想读的那段文字旁边——译文，或者说没有译文的原因，仍然回到同一张卡片，和按 `Ctrl+Alt+Q` 完全一样。
- **中转服务会说明它跳过了哪些上游。** `POST /v1/translate` 现在会带上 `attempts`：每被跳过一个上游就有一项 `{vendor, code}`，按尝试顺序排列。这样同时配置了两家密钥的部署就能说清「被点名的那家为什么让开了」，而不是悄悄拿另一家的译文当答案。中转也会把它写进自己的日志（`console.warn`），这是运维的人——唯一能修密钥或额度的人——看得到它的地方。比本构建更旧的部署不会返回这个字段，应用按同样的方式读取。

### 变更

- **有快捷键的按钮会在悬停提示里写明快捷键。** 卡片上的相机按钮和设置按钮、以及本窗口里的截图按钮，都会写出做同一件事的那个组合。三个快捷键是用户自己录制的，所以提示是每次绘制时按已保存的值现算，而不是写死在标记里：它显示的永远是当前有效的组合，而字段被清空时提示里就不带组合键。

### 修复

- **本窗口卡片里的 Glossy 标记曾是 128 像素见方。** `app.css` 里从没给这个标记定过尺寸，于是「翻译」页里的卡片用了文件本身的大小，而悬浮卡片里的同一个标记是 13 像素。现在两个窗口里都是 13 像素、同样的淡化，对比度主题也都会把这点淡化去掉。
- **备用服务列表会跟着上面选中的服务改变。** 选中的服务永远第一个被询问，所以它出现在下面的列表里就是对同一个后端的第二次尝试——而且会把另一家引擎整个挤出列表：把第一项从百度改成有道后，列表里会有道出现两次、百度不见了。现在列表按上面的选择重建：其余每家引擎各一次，顺序沿用用户排过的，还不在列表里的那家补到末尾。
- **回答来自另一家引擎时，卡片会写明是哪一家——以及为什么。** 百度与有道前面那个中转在点名的上游失败时会走自己的后备列表，这正是「设置里写着百度、卡片却显示有道、备用服务还关着，而且没有任何解释」的原因。卡片页脚写的是**回答的那一家**，它下面那一行写的是**没答的那一家**：「百度翻译 未能应答——它的额度已用尽」；中转给出原因时就写出原因（本构建不认识的错误码只保留前半句）。那一行里的名字是引擎名（`百度翻译`），而不是渠道条目的内部 id——旧代码打印的正是 id，而且把「失败的那家」说成了「回答的那家」。同一行也会写进 `glossy.log`。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
