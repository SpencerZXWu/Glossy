[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

### Added

- **Two arrows that walk the translations the card has already shown.** In the
  blank at the right of the engine's name at the foot of the card, a pair of
  arrows steps to the translation made before this one and back to the one made
  after it, without asking any provider again: the list is the history the
  settings window already keeps. An arrow with nothing behind it is disabled, a
  translation brought back this way has no selection behind it and so cannot be
  written back over, and the card keeps its place on screen while the arrows walk.
- **A build the relay has moved past says so on the card, and a message meant for
  everyone arrives in the corner of the screen.** A translation asked of the
  built-in engines now carries the app's own version number, and the relay may
  answer with a line of its own beside the translation: when this build is older
  than the oldest one the deployment still serves, a marked line appears under
  the card with the version to move to and a link to the download page; a
  one-line announcement is shown once per run in the card in the corner of the
  screen. Both are decided by the deployment — a relay that says nothing, and one
  that has never heard of either, change nothing — and the version number is the
  only thing about the app that goes with a request.

### Fixed

- **Dragging the card while it is still being filled no longer cuts off what
  sits below.** The card sizes the window to itself, but a drag runs a modal
  loop in Windows that owns the window's geometry for as long as it lasts, so a
  resize asked for while the user was dragging the popup during a reading could
  be dropped: the card was then taller than its window, and the bottom of it —
  the translation of a long reading, say — was cut off with no scrollbar to
  reach it. The window now hears about every move and is asked for the size the
  card needs once the moves have stopped.
- **A translation no longer stays broken until the app is restarted.** The whole
  app shares one HTTP client, and a client is a pool of connections: a network
  change, a proxy that went away or a machine that woke from sleep can leave that
  pool holding connections that go nowhere, after which every service is
  reported as unreachable for as long as the process lives. A request that cannot
  be sent now throws the shared client away, so the next translation begins from
  a fresh pool. The log also carries the whole chain of reasons a request failed
  now, rather than only that it did.
- **A card dragged half off the screen comes back.** A drag was the one thing
  that could leave the card hanging over the edge: the window is clamped onto
  its monitor when it is placed, and a drag placed nothing. Once the moves stop
  the card is placed again, so the window is put back onto the screen — which is
  also the moment the size the card needs is asked for again, so the two are one
  pass rather than two.
- **A card reopened from the history walks the list from the entry it is showing.**
  The entry it came from travels with it now, so the arrows start there instead of
  from the newest entry that happens to hold the same text and target.

### Changed

- The arrows that walk the history ask the backend once per card rather than on
  every redraw, and a card whose row of arrows the settings just turned on or off
  is drawn again rather than waiting for the next one.

<a id="zh-cn"></a>

## 中文

### 新增

- **卡片底部多了两个箭头，可以在已经翻过的译文之间来回走。** 在卡片底部引擎名
  右侧的空白处，一对箭头可以退回上一次翻译、再回到下一次，全程不再向任何服务发
  请求：这份列表就是设置窗口里早已在保存的历史记录。身后没有内容的箭头是禁用状态；
  用这种方式调出来的译文背后没有选中文本，所以不能写回原文；箭头来回走的时候，
  卡片在屏幕上保持原位。
- **中转服务认为已经过时的版本，会在卡片上说明；面向所有人的一句话，会出现在屏幕
  右下角。** 走内置引擎的翻译现在会带上应用自己的版本号，中转服务可以在译文旁边
  回一句它自己的话：当这个版本比部署方还愿意服务的最低版本更旧时，卡片下方会出现
  一条醒目的提示，写上该升到哪个版本以及下载页面；一句话公告则在每轮运行里只在
  右下角的小卡片里显示一次。两者都由部署方决定——什么都不说的中转服务，以及从没
  听说过这两个开关的中转服务，都不会带来任何变化；而版本号是与请求一同发出的、
  关于应用本身的唯一信息。

### 修复

- **卡片还在填充内容时拖动它，不再切掉下面的部分。** 卡片会让窗口跟自己一样大，
  但拖动在 Windows 里是一段模态循环，这期间窗口的几何由系统掌握，所以在一段较长
  译文还在陆续填充时拖动弹窗，中途请求的尺寸调整可能被丢掉：卡片于是比窗口更高，
  底部那部分——比如长句的后半段译文——就被切掉了，而且没有滚动条能够到。现在窗口
  会收到每一次移动，并在移动停下来之后按卡片的需要再要一次尺寸。
- **翻译不会再一直坏到重启应用为止。** 整个应用共用一个 HTTP 客户端，而客户端就是
  一组连接：网络发生变化、代理消失、机器从睡眠中醒来，都可能让这组连接里留下一批
  已经不通的连接，此后只要进程还在，每个服务都会被报成"无法连接"。现在一个发不
  出去的请求会把共用客户端整个丢掉，下一次翻译从一个全新的连接池开始。日志里也会
  记下整条失败原因链，而不只是"失败了"。
- **被拖到屏幕外的卡片会自己回来。** 拖动是唯一能把卡片留在屏幕边缘外的事情：
  窗口在摆放时会被夹回所在显示器，而拖动期间不会做这件摆放。移动停止后卡片会重新
  摆放，窗口因此回到屏幕内——这也正是重新询问卡片需要多大尺寸的时刻，两件事合成
  一趟，而不是两趟。
- **从历史里重新打开的卡片，会从它当前显示的那一条开始走列表。** 它来自哪一条现在
  会跟着它一起过来，所以箭头从那里出发，而不是从"文本和目标语言刚好相同的最新一条"
  出发。

### 变更

- 走历史的箭头现在每张卡片只向后端问一次，而不是每次重绘都问；设置里刚把箭头那一行
  打开或关掉之后，卡片会立刻重绘，而不是等下一次。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
