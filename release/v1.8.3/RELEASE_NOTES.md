[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that translates without a connection, and that stops handing out a fresh
allowance to anybody who installs the app again.

### Added

- **Offline translation, the fourth channel.** `opus-mt-en-zh` and `opus-mt-zh-en` — OPUS-MT's
  English and Chinese models, Helsinki-NLP, Apache-2.0 and CC-BY-4.0 — run as int8 ONNX through the same
  runtime the recogniser uses. The text never leaves the machine, there is no allowance to
  count, and a sentence takes about a second. Chinese and English both ways and nothing else.
  The two directions are downloaded one at a time on the **Resources** page — each is about
  114 MB and is a whole translator on its own, so one way round never waits for the other —
  and each file is checked against its digest before it is used. They come from ModelScope's
  mirror of the two `Xenova` repositories, which carries byte-for-byte the same artifacts and
  answers about ten times faster here than the Hugging Face mirrors do. The channel answers
  with a readable error while a direction is not there, which is also what the fallback order
  does with it.
- **The allowance is shown where it is spent.** Today's free characters used to appear only
  next to the service dropdown on the Language page; the same line is now written on the
  **Translate text** page and the **OCR** page too, because those are the pages a translation
  is started from.

### Fixed

- **A failed card offered no way to change the engine.** The bottom row of the popup — the one
  that names the engine and opens the list of the others — was drawn only on a card that had
  something to show, so a translation that failed left the reader with a retry button and
  nothing else: switching to another engine, the offline one included, meant opening the
  settings window. The row is on the failure card now, which is exactly where a switch is
  wanted, and the card that a failed screenshot draws carries it too.
- **The daily allowance started over after a reinstall.** It is counted per installation id,
  and that id was drawn at random when the settings file was first written — so installing
  Glossy again (or deleting the settings file) was a new device with a new allowance. The id
  is now derived from the machine it runs on together with the Windows account, so an install
  that comes back lands on the id it had. Nothing about the machine is sent anywhere: only the
  32 hex characters the id hashes to.

### Changed

- **Subtitle text size.** The Settings window's subtitle page now takes the size of the
  translated line directly, `13`–`40` CSS pixels, instead of the line being scaled to fit the
  box and nothing else. What is set is still stepped down when a long translation would not
  fit, and the box can still make it bigger than the setting.

<a id="zh-cn"></a>

## 中文

这个版本让 Glossy 不联网也能翻译，并且不再让每个重新安装的人都白拿一份新的免费额度。

### 新增

- **离线翻译，第四种渠道。** `opus-mt-en-zh` 与 `opus-mt-zh-en`——OPUS-MT 的英中与中英模型
  （Helsinki-NLP，分别为 Apache-2.0 与 CC-BY-4.0）——以 int8 ONNX 的形式，跑在识别引擎已经在用的同一个运行时上。
  文本永远不离开这台电脑，也没有额度可算，一句话大约一秒。只做中英互译，不做别的语言。离线包和
  识别引擎一样在**资源**页下载和删除：两个方向**各下各的**，每个约 114 MB，本身就是一台完整的
  翻译器，所以选一个方向不用等另一个下完；每个文件在被使用之前都会校验摘要。文件取自 ModelScope
  对两个 `Xenova` 仓库的镜像，字节与原始仓库完全一致，在这边比 Hugging Face 的几个镜像快十倍左右。
  某个方向还没下载就选它，会得到一句说明原因的报错；把它排进备用服务顺序、前面的渠道都失败时，也是
  同样的结果。

### 修复
- **翻译失败的卡片上没法换服务。** 弹窗底部那一行——写着当前引擎、点一下就能选其他引擎——
  以前只画在有内容的卡片上，于是翻译失败时只剩一个「重试」按钮，想改用别的引擎（包括离线）
  只能去设置窗口。现在失败卡片上也有这一行——失败正是最想换引擎的时候——截图识别失败的卡片同样带着它。


- **重新安装之后免费额度会重置。** 额度按安装 id 统计，而这个 id 是在设置文件第一次写入时随机
  生成的——于是重新安装 Glossy（或删掉设置文件）就等于一台新设备、一份新额度。现在这个 id 由
  所在机器和 Windows 账户推导得出，重新装回来还是原来那一个。机器的任何信息都不会被发出去：
  发出去的只是这个 id 哈希出来的 32 个十六进制字符。

### 变更

- **字幕字号。** 设置窗口的字幕页现在可以直接指定译文的字号，`13`–`40` CSS 像素，而不再只是把
  文字缩放到刚好塞进框里。设好的字号在长译文放不下时仍会往下缩，框本身也仍然可以让它更大。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
