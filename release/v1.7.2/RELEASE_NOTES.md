[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that puts the other recognition languages back, after checking each one on a
real screen — where more than one of them can be checked at once and the best reading wins,
and where reading a screenshot got three times quicker.

### Added

- **The notification area menu carries the two actions that need no window.** It had
  **Open Glossy** and **Quit** — but closing the settings window hides it, and that menu is
  then the only place the shortcut keys are written down at all, so **Translate a screenshot**
  and **Translate the clipboard** are on it now as well. Each entry names the combination
  doing the same thing, taken from the settings, and the menu is rebuilt whenever the settings
  are saved, so a shortcut the user records is in it before the next launch. The clipboard
  entry translates what is on the clipboard when it is clicked, rather than pressing `Ctrl+C`
  first the way the shortcut does: a menu click cannot promise the program the user meant is
  still in front. When there is nothing to translate, or translations are switched off, that
  is said in the card instead of the click doing nothing.
- **A screenshot can be read with several languages at once.** The rows on the **Resources**
  page are checkboxes now rather than a single choice: every language that is checked reads
  the same screenshot, and each line is kept from the language that read it best. A picture
  holding Japanese and English no longer needs the user to say which it is, and the choice
  costs almost nothing — measured on a real machine, a screenshot read with two languages
  takes about 0.7 s against 0.5 s for one, and the extra work grows with the languages
  checked rather than with the size of the picture. The order is the one the page shows, at
  least one language always stays checked, and a language the settings file names that this
  build does not offer is dropped rather than carried into a download of a model that does
  not exist.

### Changed

- **The detector stops stretching a wide, short screenshot.** The region is brought up to the
  detector's size on its shorter side, which is what makes small text readable — but on its own
  that turned a 900×140 region into 4736×736, fifteen times the pixels of the picture it came
  from. The longer side is held at 1440 now, so the detector's work stays near the size of the
  region whatever its shape: reading the same picture went from 1.3 s to 0.17 s, and a
  screenshot translation from about 1.4 s to 0.5 s. The recognisers also share one detector
  instead of loading a copy each, which is what made reading with two languages cost 2.3 s
  before it cost 0.7 s.
- **The recognition languages are six again, and each one says what it reads.** The
  **Resources** page offers Chinese and English, Japanese, Traditional Chinese, the Latin-script
  languages, the Cyrillic ones and Korean, as it did before 1.6.1 — each is downloaded, checked
  and removed on its own, and a language already on the disk shows up as installed rather than
  being fetched a second time. The hint above them says the thing that made this look broken:
  a pack reads its own script and nothing else, so the language has to be picked rather than
  guessed.

### Fixed

- **Japanese screenshots were read with the Chinese recogniser.** Since the language packs
  were taken off the **Resources** page in 1.6.1 only Chinese and English could be picked, and
  a settings file still naming another language was quietly read as Chinese and English rather
  than reported — the Chinese pack's dictionary holds no kana at all, so a Japanese screenshot
  came back with every kana missing and the kanji guessed, which is what "Japanese recognition
  is completely wrong" was. Each of the six languages was then read over a picture written in
  its own script — `ch`, `ja`, `cht`, `latin`, `cyrillic` and `ko` — and each one read its own
  script back: Japanese at 0.998, Traditional Chinese at 1.000, English and French at 1.000 and
  0.984, Russian at 1.000, Korean at 0.997. Nothing in the models, the dictionaries or the
  engine was wrong; only the page was, and all six are offered again.
- **The source language a card detected showed up as a code.** Youdao names the pair it
  translated rather than the source alone — `l: "en2zh-CHS"` — and the relay passed the whole
  of it on as `from`, so a card that had asked for the language to be detected read
  "Detect language · EN2ZH-CHS" where "Detect language · English" belongs. The relay now
  answers with the left half of the pair, and `normalize_lang_code` reads a pair that way too,
  so a deployment older than this build is fixed by the app rather than by a redeploy — and a
  pair can no longer reach the next request as a source the provider would reject.
- **The page renamed to *Files and logs* in 1.7.0 still had its old heading.** The sidebar
  said **Files and logs** while the heading above the page said *Settings file*, and the
  English list of pages in `README.md` said the same. Both say what the page is now.

<a id="zh-cn"></a>

## 中文

把其余识别语言找回来的一版 —— 每一种都在真机上核对过,现在可以同时勾选多种、取读得最好的那一种,而且截图识别快了三倍。

### 新增

- **通知区域菜单里加上了两个不需要窗口的动作。** 原来只有**打开 Glossy** 和**退出**——但设置窗口一关就藏起来了,托盘菜单就是唯一写着快捷键的地方,所以现在把**截图翻译**和**翻译剪贴板**也放了进去。每一项都写着做同一件事的组合键,取自设置文件;每次保存设置都会重建这个菜单,所以用户刚录制的快捷键不用重启就出现在里面。菜单里的「翻译剪贴板」翻译的是点击那一刻剪贴板里现有的文字,而不像快捷键那样先按一次 `Ctrl+C`:点菜单无法保证用户想要的那个程序还在最前面。没有可翻译的文字、或者翻译被关掉时,这些情况会在卡片里说明,而不是点了没反应。
- **一张截图可以同时用多种语言识别。** 资源页的语言行从单选改成了复选框:勾上的每一种语言都会读同一张截图,每一行保留读得最好的那一种语言的结果。中英混排的图不用再先说是哪种语言;而且这点代价几乎可以忽略——真机实测,两种语言读一张截图约 0.7 秒,一种约 0.5 秒,多出来的开销随勾选的语言数增长,而与图片大小无关。顺序就是页面上的顺序,始终至少保留一种语言;设置文件里写着、而这一版并不提供的语言会被丢掉,而不是拿去下载一个并不存在的模型。

### 变更

- **检测模型不再把又宽又矮的截图拉伸了。** 截图会按短边放大到检测模型的尺寸,这是小字能认出来的原因;但单靠这一条会把 900×140 的框选区域变成 4736×736——原图的十五倍像素。现在长边被限制在 1440,检测模型的运算量无论区域什么形状都接近原图大小:同一张图从 1.3 秒降到 0.17 秒,一次截图翻译从约 1.4 秒降到约 0.5 秒。另外,各个识别模型现在**共用一份检测模型**,而不是每种语言各加载一份——这正是双语言从 2.3 秒降到 0.7 秒的原因。
- **识别语言恢复为六种,并且各自说明它能认什么。** 资源页重新提供中文与英文、日语、繁体中文、拉丁字母语言、西里尔字母语言和韩语,与 1.6.1 之前一样——各自下载、勾选、删除,已在磁盘上的语言会直接显示为已安装,不会重复下载。页面上方的说明写出了让这件事看起来像坏掉的那一点:一种语言只认得它自己那一种文字,所以语言要自己勾选,而不是由程序猜。

### 修复

- **日文截图此前是用中文识别模型读的。** 自 1.6.1 把语言包从资源页撤下之后,只有中文与英文可以选,而设置文件里仍然写着其它语言时会被静默地当成中文与英文——中文包的字典里根本没有假名,所以日文截图读回来假名全丢、只剩猜出来的汉字,这就是"日文识别完全是错的"的来由。随后六种语言各自用写有本语言文字的样张核对过一遍(`ch`、`ja`、`cht`、`latin`、`cyrillic`、`ko`),每一种都读回了自己的文字:日文 0.998、繁体中文 1.000、英文与法文 1.000 与 0.984、俄文 1.000、韩文 0.997。模型、字典、引擎本身都没有问题,坏的只是那个页面,现在六种都重新提供了。
- **卡片上自动识别出的源语言显示成了一串代码。** 有道把翻译的语言对整串放在 `l` 里(`en2zh-CHS`),而不是只放源语言,中转原样当成 `from` 发回,于是选择"自动检测"的卡片上显示的是「自动检测 · EN2ZH-CHS」,而那里本该是「自动检测 · 英语」。中转现在只回传语言对的左半,`normalize_lang_code` 也按同样方式读语言对,所以**比这一版旧的中转部署由 App 自己纠正**,不必等重新部署;语言对也不会再作为源语言发给下一个请求而被服务拒绝。
- **1.7.0 改名为 *Files and logs* 的那一页,标题还是旧的。** 侧栏写着 **Files and logs**,页面标题却还是 *Settings file*,`README.md` 里的英文页面清单也是同样的问题。现在都改成了这一页实际的名字。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
