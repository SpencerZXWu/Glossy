[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where the machine reads more than Chinese and English, and where the
settings say where every file it downloads comes from. The card for a screenshot is also up
before the picture is read rather than after it, and the engine the card switches to is no
longer at the mercy of the settings window.

### Added

- **The reading engine is a shared half plus a language.** The ONNX Runtime and the detector
  — about 21 MB, shared by every language — are now separate files from a language's
  recogniser and its dictionary, about 10 MB, which is what the app has always downloaded
  for Chinese and English. A language is a pack of its own on the **Resources** page, where
  it can be downloaded, picked and deleted on its own; the engine and the languages already
  on the disk are not fetched again. **Chinese and English is the language this build
  offers**: Japanese, Traditional Chinese, the Latin-script languages, the Cyrillic ones and
  Korean are built the same way and are already in the table the downloader and the notices
  file use, but they are not offered yet — their recognition has to be checked on a real
  screen first, and one word in `ocr/models.rs` offers them.
- **Every downloaded file says where it comes from.** Each row on the Resources page names
  its source and its licence, and [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) — put
  next to the app by the installer and into the portable archive — carries the full licence
  texts together with the address and SHA-256 of everything the app downloads. The list is
  checked against the table the downloader uses, so it cannot quietly go stale. All of it is
  Apache-2.0: PaddleOCR's PP-OCR models, distributed as ONNX by RapidOCR.

### Changed

- **The Resources page is a list of files, not two paragraphs.** Everything that can be
  downloaded is a row of its own with its size, its state and the button that acts on it,
  the languages it can read are one list with the one in use marked, and the sources are
  folded away at the bottom for whoever wants them.
- **The first-use prompt names the size of what it is about to fetch.** A second language no
  longer reads as a second whole engine.

### Fixed

- **The keyboard stays where it was.** Every list in the settings window is drawn
  again rather than updated in place — by a save, by a download's progress, or by the window
  taking the focus back — which used to throw the focus away with the element it was on, so
  picking a language with Space moved the reader to the top of the page. The control that had
  it is found again by what it says about itself, and the row it belonged to stands in when
  the control itself is what the change removed.
- **A language reports its own progress on its own row.** The bar and its numbers used to
  appear under the shared engine and count the runtime and the detector together with the
  language, so a 10 MB download read as a 33 MB one. Each row now counts the files it is
  fetching — the engine row the runtime and the detector, a language's row that language —
  and a failure is reported on the row it belongs to.
- **The card for a screenshot is up while the picture is being read.** Taking a screenshot
  showed nothing at all until the text had been recognised and translated, because the
  waiting card was only put up when the reading engine still had to be downloaded. It now
  appears the moment the rectangle is accepted — with the name of the engine that is about
  to be asked, or saying that the reading engine is being fetched — and the translation
  takes its place.
- **A choice made in the card is no longer undone by the settings window.** The settings
  window writes the whole settings file on every save, engine and target language included,
  taken from its own controls; a window that was not on screen while the card switched its
  engine could write the older choice back, which puts the app back on Baidu. Those two are
  only written back when the pick was made on that screen, and the window re-reads the
  stored settings whenever it comes back into view.

<a id="zh-cn"></a>

## 中文

这一版让设置页说清它下载的每个文件来自哪里，也让识别引擎变成「共用的一半 + 一门语言」。此外，截图之后卡片在读图之前就立起来，而不是读完之后；卡片切到的引擎也不会再被设置窗口左右。

### 新增

- **识别引擎是「共用的一半 + 一门语言」。** ONNX Runtime 和检测模型（约 21 MB，所有语言共用）现在和一门语言的识别模型加字典（约 10 MB，也就是一直以来为中文与英文下载的那一套）分成独立的文件。一门语言在**资源**页上是一个独立的包，可以单独下载、选用和删除；已经在本地的引擎和语言不会重复下载。**这一版提供的是中文与英文**：日语、繁体中文、拉丁字母各语言、西里尔字母各语言和韩语都是同样的做法，也已经写进下载器和许可证清单使用的那张表里，但暂时没有放出来——要先在真实屏幕上确认它们的识别效果，此后在 `ocr/models.rs` 里改一个词即可放出。
- **每个下载来的文件都写明出处。** 资源页的每一行都写着它的来源与许可证，安装程序会放到应用旁边的 [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)（便携版压缩包里也有）收录了完整的许可证全文，以及应用会下载的每个文件的地址与 SHA-256。这份清单会跟下载器使用的那张表核对，所以不会悄悄过期。它们全都是 Apache-2.0：PaddleOCR 的 PP-OCR 模型，由 RapidOCR 以 ONNX 形式分发。

### 变更

- **资源页是一份文件清单，不再是两段文字。** 每个能下载的东西都自占一行，写着自己的大小、状态和对应的按钮；它能识别的语言合成一张列表，并标出正在使用的那一种；来源信息折叠在底部，留给想看的人。
- **首次使用的提示会写明即将下载的体积。** 再加一种语言不再读起来像要再下一整套引擎。

### 修复

- **键盘留在原处。** 设置窗口里的每一张列表都是整体重绘而不是原地更新——一次保存、一次下载的进度、或者窗口重新拿回焦点都会触发重绘——以前焦点会跟着被替换掉的元素一起丢掉，于是用空格选一种语言就把人送回了页面顶部。现在会按控件自己的标识把它重新找回来；如果被重绘掉的正是那个控件本身，就落在它所在的那一行上。
- **一门语言在自己的那一行上报自己的进度。** 以前进度条和数字出现在共用的引擎那一段下面，而且把运行时、检测模型和这门语言加在一起算，于是一个 10 MB 的下载读起来像 33 MB。现在每一行只数它自己在取的文件——引擎那行是运行时与检测模型，语言那行是这门语言——失败时的消息也出现在它所属的那一行。
- **截图后卡片在读图期间就立起来。** 以前截图后屏幕上什么都没有，要等文字识别和翻译都完成——因为等待卡片只在识别引擎还需要下载时才会出现。现在松开矩形的那一刻卡片就会出现：写着即将使用的引擎，或者说明正在准备文字识别；译文随后接替它。
- **卡片里做的选择不会再被设置窗口改回去。** 设置窗口每次保存都会写回整份设置文件，其中引擎和目标语言取的是它自己控件里的值；一个在卡片切换引擎时不在屏幕上的窗口，会把旧的选择写回去，于是应用又回到百度。现在只有在那个界面上真的改过这两项时才会写回，并且窗口每次回到前台都会重新读一遍已存设置。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
