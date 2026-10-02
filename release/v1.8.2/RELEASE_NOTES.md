[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that makes a subtitle reading actually start.

### Fixed

- **A subtitle reading that started and then read nothing.** Starting a run ends the one
  before it, and the way a run was ended took the two boxes with it — so the run that
  followed was left with no area to read and no place to draw in: the loop returned before
  its first tick, the log said the reading had started, nothing was translated, and neither
  dashed box ever appeared. Ending a run and forgetting the boxes are two different things
  now, and a start stores both of its own rectangles after the run before it has been ended.

### Changed

- **The settings window stays where it is when a reading starts.** It used to hide itself,
  because it is in front of the video — but a window that vanished the moment the second
  rectangle was released read as the feature closing itself. It is yours to move, minimise
  or close, and closing it still only hides it, as it does everywhere else in Glossy.

<a id="zh-cn"></a>

## 中文

这一版让字幕识别真正跑得起来。

### 修复

- **识别「开始了」却什么都读不到。**开始一次识别会先结束上一次，而结束的方式把两个矩形一起清掉了——于是
  紧接着开始的那一次既没有区域可读、也没有位置可画：循环在第一次 tick 之前就返回了，日志写着识别已经开始，
  却什么都不翻译，两个虚线框也一个都不出现。现在「结束一次运行」和「忘掉两个框」是两件事，开始识别时会在
  结束上一次之后存入它自己的两个矩形。

### 变更

- **开始识别时设置窗口留在原地。**它以前会自己藏起来（因为挡在视频前面），但第二个矩形一松开窗口就消失，
  看起来像是功能自己关掉了。现在它由你决定挪走、最小化还是关掉；关闭依旧是隐藏，和 Glossy 其它地方一样。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
