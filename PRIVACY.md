# Glossy · Privacy

<a id="en"></a>

## English

Glossy has no account, no analytics and no telemetry, and it does not report what you
select. Text leaves the machine only when you ask for a translation, and then only the
text you asked about.

**Where the text goes** — the target you picked in the settings:

- **Google Translate** (`translate.googleapis.com`) — a public endpoint run by Google.
- **The built-in engines** — the project's own relay, a Cloudflare Worker whose source is
  in [`server/`](./server/README.md), which passes the text on to Baidu, Youdao or Zhipu
  depending on how it is deployed. It counts the characters a day per installation and per
  address and stores nothing else: the text is not logged and not kept. **The app's own version
  number goes with each request**, so the relay can tell a build it no longer serves and can
  pass back a one-line announcement; the version is all it learns about the copy of Glossy on
  this machine. The installation is
  identified by an id derived from this computer and this Windows account (a hash of the two,
  so neither is sent), which is what keeps the daily allowance the same one after installing
  the app again.
- **A service of your own** (`api-baidu`, `api-openai`) — only if you filled its credentials
  in on the General page, and then the text goes from this machine **straight to that
  service**: Baidu's `fanyi-api.baidu.com` with your own APP ID and key, or the address you
  typed for the OpenAI compatible entry. The project's relay is not in the path, receives
  nothing, and cannot count anything. That service sees the text under its own privacy
  policy, as the account holder's request.
- **Nothing at all** — `offline` translates on this machine, with the models downloaded from
  the Resources page. No request is made and there is no allowance to count.

**Screenshots (OCR translation)** are taken on your machine, and the text in them is
recognised on your machine — PP-OCRv4, running through the ONNX Runtime the Resources page
downloads, reads the rectangle you dragged and nothing else. Only the recognised text is
sent, to the same relay and in the same way as a selection. The picture is never uploaded,
and neither it nor the text it produces is stored by the app.

**Documents (document translation)** are read and written on your machine, in your own
folder. Only the prose paragraphs are sent, to the same relay and in the same way as a
selection; the rest of the file — code blocks, front matter, heading marks, list bullets,
cue numbers and time codes — stays here. At most 1500 characters per request and 60 000
characters per document.

**Subtitles (subtitle translation, developer mode)** are read on your machine, from the one
rectangle you picked and nothing else, about once a second while a reading is running. Only
the text that comes back from that rectangle is sent, to the same relay and under the same
daily allowance as a selection, and only when what was read has changed. The picture is
never uploaded and nothing about it is stored; stopping the reading, quitting Glossy or
turning developer mode off ends it.

**Definitions** for single words come from `api.dictionaryapi.dev`, a public service that
just receives the word.

**Exchange rates** are fetched when a currency amount is recognised, from the relay or from
a public rates endpoint, and carry a base currency code and nothing about you.

**Update checks** are off by default. Turned on in the settings, the app asks GitHub for
`releases/latest` of this repository.

**What is stored locally**, in `%APPDATA%\com.glossy.translator\`: `settings.json` (settings
and the credentials you filled in yourself, each of those encrypted with Windows `DPAPI` for
this Windows login before it is written), `history.json`,
a small `rates.json` cache, and `glossy.log` — the failures the app ran into, capped at
256 KB, with the file before it kept as `glossy.log.1`. Nothing in that folder is uploaded;
deleting it removes the settings, the history and the log. A key encrypted for another
Windows login — a settings file copied from another machine, or from an older install — cannot
be unlocked here and is dropped rather than kept. `Export…` writes the same protected shape
into `Documents\glossy-settings.json`, so the exported file never carries a usable key. A
settings file written by an older version may still hold a key that version asked for: it is
dropped, unused, the next time the settings are saved.

The services above receive what they are sent under their own privacy policies.

<a id="zh-cn"></a>

## 中文

Glossy 没有账号、没有统计分析、没有遥测，也不会记录你选了什么。只有当你要求翻译时，文本才会离开
本机，而且只发送你要求翻译的那段文字。

**文本发给谁** —— 取决于你在设置里选的渠道：

- **Google 翻译**（`translate.googleapis.com`）—— Google 提供的公共端点。
- **内置引擎** —— 项目自己的中转服务，源码在 [`server/`](./server/README.md)，是一个
  Cloudflare Worker，按部署配置把文本转给百度、有道或智谱。它只按"每台设备 / 每个地址 / 每天"
  统计字符数，其余什么都不存：文本不写日志、不落盘。**每次请求还会带上应用自己的版本号**，
  中转服务据此判断某个版本是否还能继续提供服务，也可以回一句公告；除版本号外，它不会知道这台
  机器上这份 Glossy 的任何信息。设备用一个由本机与当前 Windows 账户推导出的
  安装 ID 标识（两者哈希后的值，二者本身都不会被发送），这样重新安装应用后每日额度仍然是同一份额度。
- **你自己的服务**（`api-baidu`、`api-openai`）—— 只有在「常规」页填过它的凭据时才会用到，此时文本
  从本机**直接发往那个服务**：你自己的 APP ID 与密钥对应的百度 `fanyi-api.baidu.com`，或者你为兼容
  OpenAI 的渠道填写的地址。项目自己的中转服务完全不在链路上，收不到任何东西，也无从统计。那个服务
  按自己的隐私政策处理收到的文本，看到的是账号持有者本人的请求。
- **什么都不发** —— `offline` 在本机翻译，用的是从「资源」页下载的模型。不发出任何请求，也没有额度可计。

**截图（OCR 翻译）**在本机完成：识别也用本机——PP-OCRv4 跑在「资源」页下载的 ONNX Runtime 上，
读的就是你框选的那块矩形，别的都不读。发送出去的只有识别出来的文字，走的是和划词同一个中转服务、同一条路径。
图片从不上传，应用也不保存这张图或它识别出的文字。

**文档（文档翻译）** 在本机读取、写回你自己的文件夹。发送出去的只有正文段落，走的是和划词
同一个中转服务、同一条路径；文件里其余的内容——代码块、前置元数据、标题符号、列表符号、字幕
序号与时间码——都留在这里。每次请求最多 1500 个字符，整个文档最多 60 000 个字符。

**字幕（字幕翻译，开发者模式）**在本机读取，读的就是你框选的那一块矩形，识别期间大约每秒读一次。
发送出去的只有那块矩形识别出来的文字，走的是和划词同一个中转服务、同一份每日额度，而且只在读到的
内容变化时才发送。图片从不上传，也不保存；停止识别、退出 Glossy 或关掉开发者模式都会结束它。

单词释义来自公共的 `api.dictionaryapi.dev`，它只会收到那个单词。

**汇率**在识别到货币金额时查询，走中转服务或公开的汇率端点，只带一个基准货币代码，不含任何与你
相关的信息。

**更新检查**默认关闭。在设置里打开后，应用会向 GitHub 查询本仓库的 `releases/latest`。

**本地存储**在 `%APPDATA%\com.glossy.translator\`：`settings.json`（各项设置，以及你自己填写的凭据——
每一条在落盘前都用 Windows `DPAPI` 按当前 Windows 登录加密）、`history.json`、一个小的 `rates.json`
缓存，以及 `glossy.log`——应用遇到过的失败，上限 256 KB，写满后旧的那份保留为 `glossy.log.1`。
该目录的内容不会上传；删掉它就等于删掉了设置、历史和这份日志。为另一个 Windows 登录加密过的密钥——
从别的机器或别的安装复制过来的设置文件——在这里解不开，会被丢弃而不是留着。`导出…` 写进
`Documents\glossy-settings.json` 的也是同样的加密形状，所以导出的文件永远不会带走一把能用的密钥。
旧版本写出的设置文件里可能还留着一个当时索要的密钥：它会在下次保存设置时被丢弃，且不会被使用。

上述服务按照它们各自的隐私政策处理收到的内容。
