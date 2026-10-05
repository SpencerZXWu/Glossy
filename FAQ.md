[English](#glossy-faq) · [中文](#glossy-常见问题--中文)

# Glossy FAQ

Questions the README answers only in passing, gathered in one place. Everything below
describes Glossy 2.0.0, and where an older version behaved differently the answer says so.

### Why do I have to click the Glossy icon?

Glossy watches the mouse for a drag or a double click and, when the gesture really selected
something, puts a small Glossy icon under the selection. Nothing is translated, nothing is
sent and nothing is charged against the day's allowance until you click that icon, so a word
you merely swiped over costs nothing; clicking anywhere else makes the icon disappear. The
recorded shortcuts that translate — `Ctrl+Alt+C` for the clipboard and `Ctrl+Alt+Q` for a
rectangle drawn over the screen — are the exception: each one translates at once, with no
icon in between. (`Ctrl+Alt+G` does not translate; it only brings the settings window to the
front.)

### Do I need an account, an API key or a credit card?

No. Four of the entries in the service dropdown work without an account of your own:
`google` asks a public endpoint, `offline` asks this machine, and `cloud-baidu` and
`cloud-youdao` are answered by a Glossy relay that holds the vendor account, so no key is
entered into or stored by the app. You can go further if you want to: since 2.1.0 the
**General** page has a **Your own API** panel where you may fill in an APP ID and key from
Baidu's 翻译开放平台, or the address, key and model of any service that speaks the OpenAI
chat completions protocol. Those two channels — `api-baidu` and `api-openai` — only appear
in the dropdown while their fields are filled in, the request goes from this machine
straight to the service, and the keys are encrypted for this Windows login before they
reach the disk. Nothing has to be filled in, and nothing is asked for on a fresh install.

### What is sent where?

Only the text you asked about leaves the machine, and only to the service you picked:

| Service | What leaves the machine | Where it goes |
| --- | --- | --- |
| `cloud-baidu` | the text, the language pair, the vendor name and the install id | the project's relay (deployed from `server/`), which forwards the text to Baidu |
| `cloud-youdao` | the same | the same relay, which forwards the text to Youdao |
| `google` | the text | `translate.googleapis.com`, a public Google endpoint |
| `api-baidu` | the text and a signature made from your APP ID and key | Baidu directly, with your own account |
| `api-openai` | the text, and the key in the `Authorization` header | the address you typed, directly |
| `offline` | nothing | nothing — the models run on this machine |

The other calls Glossy makes over the network are:

- **Word definitions and phonetics** for a single word go to `api.dictionaryapi.dev`, which
  receives the word and nothing else.
- **Currency rates**, when a currency amount is recognised, come from the relay's rate
  endpoint or, when that cannot answer, directly from `open.er-api.com` or
  `api.frankfurter.app`; they carry a base currency code (and the install id when they go
  through the relay) and nothing about you.
- **Update checks** are off by default. Turned on, the app asks GitHub for
  `releases/latest` of this repository.
- **Downloads** on the Resources page fetch the recognition engine, its language packs and
  the offline translation pack from `modelscope.cn` (the ONNX runtime comes from
  `files.pythonhosted.org`). Nothing is downloaded until you press Download.
- **Screenshot and subtitle reading** stay on this machine, and only the text the recogniser
  produced is then sent on like a selection.

Nothing else is uploaded, and there is no account, no analytics and no telemetry.

### What happens when a service's free quota runs out?

The two server-backed entries count the characters they translate per device, per address
and in total, per day, and those counters start over at 00:00 UTC. When one of them hits its
cap the relay refuses the request, and the card shows "The free cloud translation quota for
today is used up". With **Ask another service when the chosen one fails** on — which is the
default — Glossy does not stop there: it tries the other engines in the order the settings
put them in, and when one of them answers, the card's footer names the engine that answered
while the line under it names the one that did not and why (`its allowance is used up`, …);
the same line is written to `glossy.log`. The choice itself does not move — the service you
picked stays the configured one. Turning the fallback off shows the failure as a failure.
The settings window also shows what is left of today's allowance next to the dropdown, with
a `Check again` button. To keep translating past the shared allowance, pick another entry,
wait for 00:00 UTC, or deploy your own relay from `server/`.

### What is stored on my computer, and where?

Everything local lives in `%APPDATA%\com.glossy.translator\`:

| File | What it holds |
| --- | --- |
| `settings.json` | your choices, written with `formatVersion` 2 and no credential of any kind |
| `history.json` | the translations the history remembers — the original, the translation, the provider and the time — up to the limit you chose (default: the last 50; `Off` writes none) |
| `rates.json` | a small cache of currency rates |
| `glossy.log` (and `glossy.log.1`) | the failures the app ran into, capped at 256 KB |
| `ocr\` | the recognition engine and the language packs you downloaded (about 33 MB for the engine and the first language) |
| `translate\` | the offline translation pack, if you downloaded it (about 244 MB) |

Nothing in that folder is uploaded, and deleting it removes the settings, the history, the
cached rates, the log and the downloaded models.

### Are my translations cached or logged?

Not by the relay: the server counts the characters of the day and stores nothing else — the
text is not logged and not kept. On the machine, the answer is kept in `history.json` as
long as the history is on (the last 50 by default, or from `Off` to `The last 500`), because
a card reopened from the History is shown again without asking the provider a second time;
that file is plain JSON and never leaves the machine. `glossy.log` records failures and the
reason a fallback moved on, not the text of a translation. The only other cache is
`rates.json`, which holds currency rates for six hours.

### Does Glossy track what I select?

No. Glossy has no account, no analytics and no telemetry, and it does not report what you
select. Text leaves the machine only when you ask for a translation, and then only the text
you asked about.

### Why does Glossy need an install id, and what is it made from?

The relay counts the day's allowance per device, so it has to be able to tell one
installation from another, and the id is what it counts by. It is derived rather than drawn
at random — an MD5 of the Windows installation id (the `MachineGuid`) together with your
account name — so installing Glossy again lands on the same id instead of starting the
allowance over, and it is always the same 32 hex characters. Nothing about the machine
itself is sent: the relay only ever sees those 32 characters, and only for `cloud-baidu` and
`cloud-youdao` (and the rate lookups that go through the relay). `google` and `offline`
never carry it.

### How do I get the offline pack, and how big is it?

Open **Resources** in the settings window and download the offline translation pack. It is
two directions — Chinese into English and English into Chinese — and each direction is
downloaded on its own, about 114 MB, for about 244 MB in total. They are OPUS-MT models
(`opus-mt-zh-en` and `opus-mt-en-zh`, from Helsinki-NLP, distributed as int8 ONNX by
Xenova) fetched from `modelscope.cn` and kept in
`%APPDATA%\com.glossy.translator\translate`. Once a direction is there, choosing `offline`
translates that way with no connection and no allowance; a sentence takes about a second,
which is the price of not asking anybody. Only Chinese and English are covered.

### Why do the language bars offer fewer languages on some services?

Because each engine translates a different set, and the bars list only the languages of the
engine the card is using, so a language it would refuse is never offered and never sent.
`google` and `cloud-youdao` take all 31 languages Glossy lists. `cloud-baidu` takes 23 of
them: eight — `uk`, `tr`, `hi`, `id`, `ms`, `he`, `no` and `sk` — are left out because a
standard Baidu account answers `58001` for them, and only an enterprise plan accepts them.
`offline` takes only `en` and `zh-CN`, which is what its two models were trained for;
`zh-TW` is deliberately not offered there rather than answered in the wrong script.

### Can I use Glossy for commercial work?

The app is MIT-licensed, so the app itself may be used commercially. What the free
translation tiers forbid is handing the raw quota on to other people or reselling it, and
Glossy does not do that — the credentials stay on the relay and no user ever holds a vendor
key. The shared relay, however, is a metered free service meant for ordinary use, so a
business that needs volume should deploy its own relay from `server/` with its own account.
The offline models are Apache-2.0 (`opus-mt-en-zh`) and CC-BY-4.0 (`opus-mt-zh-en`), both
fine for commercial use.

### How do I move my settings to another machine?

On the **Files and logs** page, **Export…** writes your choices to
`Documents\glossy-settings.json`; on the other machine, **Import…** reads that file back
into Glossy. The export is plain JSON and holds the choices plus the credentials you filled
in yourself — and those go in **encrypted** (`DPAPI`, for the Windows login that entered
them), so the file is safe to mail: it never carries a directly usable key. On the other
machine the key cannot be unlocked and is dropped there, while the rest of the settings are
applied as they are. An import is
validated before it is applied, and a file that holds no Glossy setting at all is refused
rather than wiping the current setup.

### What happens to a settings file written by an older version?

`formatVersion` is now 2, and any file written by a release from 1.3.0 onwards is migrated
on the first launch rather than discarded. Version 2 folded the four fields that used to
name the service — `channel`, `cloudProvider`, `cloudVendor` and `provider` — into the
single `service` field, and dropped the relay's address and the credential map, which this
build no longer reads. A file that is not a JSON object at all, or one written by a newer
format version than this build understands, is copied to
`settings.backup-<unix seconds>.json` next to it before the defaults are used, so a
deliberate break never costs anyone their settings. Since 1.3.0 settings are only ever
added, so a file from a slightly older release normally just loads.

### Can I use Glossy without a network connection?

Partly. With the offline translation pack downloaded, `offline` translates Chinese and
English both ways with no connection and no allowance, and screenshot recognition also runs
on this machine, so a picture can be read without a network. Everything else — `cloud-baidu`,
`cloud-youdao` and `google` — needs a connection. Currency rates need one too, and a stored
rate goes stale after seven days offline.

### Why does Glossy install a low-level mouse hook, and why does my antivirus complain?

Select-to-translate has to notice a mouse gesture in whatever program you are reading in,
and the only way to see that from outside the program is a `WH_MOUSE_LL` hook. The hook
callback itself only records coordinates; the decision — drag or double click — is made on
a worker thread, which then copies the selection with `Ctrl+C` (and restores your clipboard
afterwards when the setting asks for it). A low-level mouse hook is also a common shape for
malware, so antivirus tools sometimes flag it; code signing is the real mitigation, and the
installers are not signed yet, so SmartScreen will warn about an unknown publisher.

### Why is the free Google endpoint sometimes rate-limited or unreachable?

`google` calls `translate.googleapis.com`, a public endpoint run by Google that is not a
documented, keyed API. It can answer `429` to a desktop client even while a browser or
`curl` still works; Glossy first retries with a second client id, and if the message stays,
waiting a minute or switching to `cloud-baidu` or `cloud-youdao` is the way out. The
endpoint is also blocked on some networks, including much of mainland China, which is what
the two server-backed entries exist for.

---

[English](#glossy-faq) · [中文](#glossy-常见问题--中文)

# Glossy 常见问题 · 中文

README 里只是一笔带过的那些问题，集中放在这里。下面所有内容描述的都是 Glossy 2.0.0；
旧版本行为不同的地方，答案里会写明。

### 为什么我必须点击 Glossy 图标？

Glossy 会监听鼠标的拖动或双击，确认这个手势真的选中了文字之后，就在选中内容下面放一个
小小的 Glossy 图标。在你点击那个图标之前，什么都不会被翻译、不会被发出去，也不会扣掉
当天的任何额度，所以只是扫过的词不花任何东西；而点击别处会让图标消失。记录下来的快捷键里
会直接翻译的那两个——`Ctrl+Alt+C` 读取剪贴板、`Ctrl+Alt+Q` 读取屏幕上框选的矩形——是例外：
它们会立刻翻译，中间不需要点图标。（`Ctrl+Alt+G` 不会翻译，它只是把设置窗口带到前台。）

### 我需要账号、API 密钥或者信用卡吗？

都不需要。服务下拉框里靠前的四项都不需要你自己的账号：`google` 访问一个公共端点，`offline`
用这台机器，而 `cloud-baidu` 和 `cloud-youdao` 由 Glossy 自己的中转服务作答，厂商账号保存在
服务端，因此应用既不需要输入、也不会保存任何密钥。如果你想更进一步，从 2.1.0 起**常规**页多了
一块**自填 API**：可以填百度翻译开放平台的 APP ID 与密钥，也可以填任何兼容 OpenAI chat
completions 协议的服务地址、密钥和模型名。这两个渠道——`api-baidu` 和 `api-openai`——只在
字段填好之后才出现在下拉框里，请求从本机直接发往那个服务，而密钥在落盘前会按当前 Windows
登录加密。全新安装不填任何东西，也不会被要求填。

### 什么内容会被发到哪里？

只有你要求翻译的那段文字会离开本机，而且只发给你选中的那个服务：

| 服务 | 离开本机的内容 | 去向 |
| --- | --- | --- |
| `cloud-baidu` | 文字、语言对、厂商名和安装 ID | 项目自己的中转服务（由 `server/` 部署），它把文字转给百度 |
| `cloud-youdao` | 同上 | 同一个中转服务，它把文字转给有道 |
| `google` | 文字 | `translate.googleapis.com`，Google 的公共端点 |
| `api-baidu` | 文字，以及用你的 APP ID 和密钥算出的签名 | 直接发往百度，用你自己的账号 |
| `api-openai` | 文字，以及放在 `Authorization` 头里的密钥 | 直接发往你填写的地址 |
| `offline` | 什么都不发 | 什么都不发——模型在这台机器上运行 |

Glossy 其它会联网的地方还有：

- **单词的释义与音标**会发给 `api.dictionaryapi.dev`，它只收到那个单词，别的什么都没有。
- **汇率**在识别到货币金额时查询，走中转服务的汇率接口；它答不上时，应用会直接访问
  `open.er-api.com` 或 `api.frankfurter.app`。请求只带一个基准货币代码（经过中转服务时还会
  带上安装 ID），不含任何与你相关的信息。
- **更新检查**默认关闭。打开后，应用会向 GitHub 查询本仓库的 `releases/latest`。
- **资源页的下载**会从 `modelscope.cn` 取回识别引擎、语言包和离线翻译包（ONNX 运行时来自
  `files.pythonhosted.org`）。在你按下「下载」之前，什么都不会下载。
- **截图翻译和字幕取字**都在本机完成，只有识别出来的文字会像一次划词那样被发出去。

其它内容都不会上传，也没有账号、没有统计分析、没有遥测。

### 当某个服务的免费额度用完时会发生什么？

走服务器的那两个渠道会按设备、按地址、按总量分别统计每天翻译过的字符数，这些计数器会在
UTC 00:00 重新开始。其中任何一个碰到当天上限时，中转服务会拒绝这次请求，卡片上会显示
「今天的免费云端翻译额度已用完」。在**所选服务失败时改用其他服务**打开时（这是默认值），
Glossy 不会就此停下：它会按设置里排好的顺序去试其它引擎；某个引擎答上来之后，卡片底部会写明
是谁作答的，它下面那一行则写明是谁没有作答、以及原因（`它的额度已用尽`……），同一行也会写进
`glossy.log`。你的选择本身不会移动：你选中的服务仍然是配置里的那一个。把这个开关关掉，失败
就会原样显示成失败。设置窗口还会在下拉框旁边显示今天还剩多少额度，配一个「重新检查」按钮。
想越过这份共享额度继续翻译，可以换一个选项、等 UTC 00:00，或者用 `server/` 部署一份自己的
中转服务。

### 我的电脑上存了什么，存在哪里？

本机的文件都在 `%APPDATA%\com.glossy.translator\`：

| 文件 | 存放的内容 |
| --- | --- |
| `settings.json` | 你的各项选择，以 `formatVersion` 2 写出，不含任何形式的凭据 |
| `history.json` | 历史记录记住的翻译——原文、译文、提供方与时间——数量上限由你设定（默认最近 50 条；设为「关闭」则不写） |
| `rates.json` | 一个很小的汇率缓存 |
| `glossy.log`（以及 `glossy.log.1`） | 应用遇到过的失败，上限 256 KB |
| `ocr\` | 你下载的识别引擎和语言包（引擎加第一门语言约 33 MB） |
| `translate\` | 离线翻译包，如果你下载过（约 244 MB） |

这个目录里的内容都不会上传；删掉它就等于删掉了设置、历史、汇率缓存、日志和已下载的模型。

### 我的翻译会被缓存或记录吗？

中转服务不会：服务器只统计当天的字符数，其余什么都不存——文本不写日志、不落盘。在本机，
只要历史记录开着，译文就会留在 `history.json` 里（默认最近 50 条，可选「关闭」到
「最近 500 条」），因为从历史里重新打开一张卡片时，要不问提供方第二次就能原样显示；那个
文件是普通 JSON，从不离开本机。`glossy.log` 记录的是失败，以及某个引擎为什么被换掉，而不是
翻译的原文。唯一的另一处缓存是 `rates.json`，它保存汇率六个小时。

### Glossy 会记录我选了什么吗？

不会。Glossy 没有账号、没有统计分析、没有遥测，也不会记录你选了什么。只有当你要求翻译时，
文本才会离开本机，而且只发送你要求翻译的那段文字。

### 为什么 Glossy 需要一个安装 ID，它是怎么算出来的？

中转服务要按设备统计当天的额度，所以必须能分辨不同的安装，而这个 ID 就是它用来计数的东西。
它不是随机生成的，而是从本机推导出来的——把 Windows 的安装标识（`MachineGuid`）和你的
账户名一起做 MD5——因此重新安装 Glossy 会落在同一个 ID 上，额度不会重新开始，而它始终是
同样的 32 个十六进制字符。机器本身的信息不会被发出去：中转服务只会看到这 32 个字符，而且只在
`cloud-baidu`、`cloud-youdao`（以及经过中转服务的汇率查询）时带上它。`google` 和 `offline`
从不携带它。

### 离线包怎么获取，有多大？

在设置窗口打开**资源**页，下载离线翻译包。它是两个方向——中译英和英译中——每个方向单独下载，
约 114 MB，两个方向合计约 244 MB。它们是 OPUS-MT 模型（`opus-mt-zh-en` 和 `opus-mt-en-zh`，
出自 Helsinki-NLP，由 Xenova 做成 int8 ONNX 分发），从 `modelscope.cn` 取回，保存在
`%APPDATA%\com.glossy.translator\translate`。某个方向到位后，选中 `offline` 就能用那个方向
翻译，不需要联网，也不消耗任何额度；一句话大约要一秒，这就是不麻烦别人的代价。它只覆盖中文和
英文。

### 为什么有些服务的语言栏里语言更少？

因为每个引擎能翻译的语言集合不同，而语言栏只列出当前这张卡片所用引擎支持的语言，所以它不会
提供、也不会发送一个会被拒绝的语言。`google` 和 `cloud-youdao` 支持 Glossy 列出的全部 31 种
语言。`cloud-baidu` 支持其中 23 种：`uk`、`tr`、`hi`、`id`、`ms`、`he`、`no`、`sk` 这八种被
排除在外，因为标准版百度账号对它们会返回 `58001`，只有企业版才接受它们。`offline` 只支持
`en` 和 `zh-CN`，这正是它两个模型训练过的范围；繁体中文 `zh-TW` 是刻意不提供的，而不是用它
不擅长的那种字形作答。

### Glossy 可以用在商业用途吗？

应用本身是 MIT 许可证，所以应用本身可以商用。各免费翻译额度所禁止的，是把原始额度转手给别人
或转卖；Glossy 没有这么做——凭据保存在中转服务上，任何用户都拿不到厂商密钥。不过共享的那份
中转服务是一项计量用的免费服务，面向日常使用，所以需要规模的商业用途应当用 `server/` 部署
自己的中转服务，接自己的账号。离线模型是 Apache-2.0（`opus-mt-en-zh`）和 CC-BY-4.0
（`opus-mt-zh-en`），两者都可商用。

### 我怎么把设置搬到另一台机器？

在**文件与日志**页，**导出…**会把你的选择写到 `Documents\glossy-settings.json`；在另一台
机器上用**导入…**把那个文件读回 Glossy。导出的是普通 JSON，包含各项设置，以及你自己填写的凭据——
而凭据是加密后写进去的（`DPAPI`，按当前 Windows 登录），所以导出的文件可以放心地发出去，
里面没有一把能直接用的密钥；在另一台机器上，属于原来那个 Windows 登录的密钥解不开，会被丢弃，
其余设置照常应用。导入会先校验再应用，如果一个 Glossy 设置都没有的文件会被拒绝，而不是把现有配置清空。

### 旧版本写出的设置文件会怎样？

`formatVersion` 现在是 2，任何 1.3.0 及以后写出的文件都会在首次启动时被迁移，而不是被丢弃。
版本 2 把过去用来指明服务的四个字段——`channel`、`cloudProvider`、`cloudVendor` 和
`provider`——合并成了单一的 `service` 字段，并去掉了中转服务的地址和凭据表，这个版本已经不再
读取它们。完全不是 JSON 对象的文件，或者由比本版本更新的格式版本写出的文件，会先被复制成旁边
的 `settings.backup-<unix 秒>.json`，再使用默认值，所以一次有意的破坏也不会让人丢掉设置。
从 1.3.0 起设置只会新增，所以旧一点版本的文件通常直接就能加载。

### 断网时能用 Glossy 吗？

部分可以。下载了离线翻译包之后，`offline` 能在不联网、不消耗额度的情况下做中英互译，截图识别
也在本机完成，所以图片文字没有网络也能读。除此之外——`cloud-baidu`、`cloud-youdao` 和
`google`——都需要联网。汇率查询也需要，离线的汇率七天后就会过期。

### 为什么 Glossy 要装一个底层鼠标钩子，为什么杀毒软件会报警？

划词翻译必须在你正在阅读的任何程序里察觉到鼠标手势，而要从程序外面看到这一点，唯一的方法就是
`WH_MOUSE_LL` 钩子。钩子回调本身只记录坐标；是拖动还是双击的判断放在工作线程上，判断完再用
`Ctrl+C` 复制选区（如果设置要求，之后会还原你的剪贴板）。底层鼠标钩子也是恶意软件常见的一种
形态，所以杀毒软件有时会报警；真正的解决办法是代码签名，而安装包目前还没有签名，因此
SmartScreen 会提示未知发布者。

### 为什么免费的 Google 端点有时会被限流或访问不了？

`google` 访问的是 `translate.googleapis.com`，一个由 Google 提供、但没有文档、也不需要用
密钥的公共端点。它会对桌面客户端返回 `429`，即使浏览器或 `curl` 仍然正常；Glossy 会先用第二个
客户端 id 重试，如果提示还在，等一分钟，或者改用 `cloud-baidu`、`cloud-youdao`，就是出路。
这个端点在部分网络（包括中国大陆的大部分地区）也会被屏蔽，那正是走服务器的两个渠道存在的意义。
