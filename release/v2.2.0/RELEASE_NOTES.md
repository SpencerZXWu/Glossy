[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that lets a deployment read its own counters from the window and from
a command line, that can put a third engine last in the relay's chain, and that
says so on the allowance line when the number is one instance's.

### Added

- **The deployment's own report, in a window and on a command line.** A relay
  built from `server/` keeps the day's counters to hold the free allowance
  honest; `GET /v1/stats` is those same numbers read the other way round — how
  much of today is gone, whose it went to, how many screenshots the month has
  read, and what is left in total. It is answered from the counters themselves
  rather than from a second book, so it can never disagree with what an App is
  told, and it books no characters and moves nothing. It is off unless the
  deployment sets `ADMIN_TOKEN`, because the report names installations and what
  each of them spent; a long tail is rolled into a line of its own rather than
  dropped, so its rows always add up to the total beside them. Two things read
  it: `scripts/stats.ps1`, which keeps one row per day in a CSV — its token is
  stored with DPAPI, so neither the script nor the scheduled task holds it — and
  a page of the window's own behind developer mode, where the token is typed in
  and kept in memory for as long as the window is open.
- **A third engine behind the other two.** A deployment that has opened Tencent
  Cloud's machine translation can put it last in the relay's chain with
  `TENCENT_MT=1`, reusing the key pair it already reads screenshots with. It is
  not a service an App can pick: the account is the operator's, and the engine is
  there so that an answer keeps coming when the engines a reader asked for have
  nothing left. It stays out of the chain unless that switch is set, because
  reading pictures and translating text are separate products on that account.

### Changed

- **The allowance line says when it is one instance's.** A deployment of several
  instances with no shared store counts in each of them and answers with
  whichever copy took the call, so the same question twice can come back with two
  different numbers. The relay now says which of the two it is (`shared` on
  `/v1/quota`, `counters.shared` on `/v1/health`), and the window adds a word to
  the line instead of letting the number move under the reader.
- **The allowance is not asked for on every page change.** An answer stands for
  twenty seconds before another one is asked for, so walking through the window
  no longer means watching the same number redraw itself.

### Fixed

- **The shared-counter tests left their sockets open.** A link keeps a socket for
  as long as it exists, and a socket that is still open keeps the test process
  alive: that file never printed its summary and the whole suite read as a hang.
  The links are closed when the file is over.

<a id="zh-cn"></a>

## 中文

这一版把「这套部署自己记的那份账」搬到了窗口里和命令行里，让上游链上多了第三个引擎（客户端看不到也点不了），
并让额度那一行在按实例计数时把话说清楚。

### 新增

- **这套部署自己的报告，窗口里和命令行里各有一份。** 从 `server/` 部署出去的转发端本来就留着当天的计数，
  好让免费额度有个准；`GET /v1/stats` 就是同一份数字换个方向读——今天用掉多少、算在谁头上、这个月读了多少张
  截图、总共还剩多少。它直接读计数本身，不另立一本账，所以永远不会跟 App 被告知的数字打架，也不扣字符、
  不动任何东西。这套部署没设 `ADMIN_TOKEN` 时它就是关着的，因为报告会点出各台安装和它们各自花了多少；
  尾巴上那一长串会合成一行而不是丢掉，所以各行加起来永远等于旁边那个总数。读它的有两处：`scripts/stats.ps1`，
  每天在 CSV 里记一行——它的令牌用 DPAPI 存，脚本和计划任务里都不放明文——以及窗口自己藏在开发者模式后面的
  那一页，令牌现场输入、窗口关掉就没了。
- **另外两个引擎后面又跟了第三个。** 部署方如果开了腾讯云的机器翻译，用 `TENCENT_MT=1` 就能把它排在上游链的
  最后，复用读截图已经在用的那对密钥。它不是 App 能选的服务：账号是部署方自己的，它待在那里只是为了让读者
  点名的引擎都没额度时还能有答案。不打开那个开关它就不进链，因为在这个账号上「读图」和「翻译文字」是两个
  独立的产品。

### 变更

- **额度那一行会说明它是不是某一台实例的。** 多实例部署如果没有共享存储，就是各记各的，谁接到请求就用谁
  那份回答，同一个问题问两次可能得到两个数。现在转发端会说明自己是哪一种（`/v1/quota` 里的 `shared`、
  `/v1/health` 里的 `counters.shared`），窗口会在那一行后面加一句说清楚，而不是让数字在读者眼前自己变。
- **换页不再每次都去问额度。** 一次回答管二十秒，之后才会再问，所以在窗口里走一圈不用再看同一个数字重画一遍。

### 修复

- **共享计数的测试没关连接。** 连接只要还在，socket 就一直在，而没关的 socket 会让测试进程活着：那个文件
  永远打不出汇总，整个测试套件看起来像卡死了。现在文件跑完就把连接关掉。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
