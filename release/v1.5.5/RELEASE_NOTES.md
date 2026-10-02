[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where the screen is read on the machine it is on, and where the settings stop
being a list of pages that each held one switch.

### Added

- **The screenshot is read here, not by a service.** Recognition runs on this machine
  through PP-OCRv4 on ONNX Runtime: the picture never leaves the machine, it works with no
  connection at all, and a reading costs nothing. The engine is fetched the first time a
  screenshot is asked for — a prompt gives its size, about 33 MB, before anything is
  downloaded — and lands next to the app's own files, where *Delete the engine* takes it
  off the disk again.
- **A Resources page.** Everything Glossy keeps on the machine is downloaded, sized and
  removed in one place: the recognition engine today, and the offline translation pack when
  it lands.
- **The version this copy is.** The Updates page prints it above the check button, so a
  report about a broken build starts with the number that identifies it.

### Changed

- **The settings are fewer pages.** *General* carries the shortcuts and the trigger rules
  as well as the master switch — the three were read together anyway — and *Wordbook* moved
  up into the features, where the rest of what Glossy does lives.
- **Document translation is closed while it is rebuilt.** The entry is disabled and marked
  *In development*, the page says so, and the commands behind it refuse instead of starting
  a translation that cannot be finished yet. Settings already saved are untouched.

### Removed

- **The monthly ceiling on screen readings.** Recognition happens on the machine now, so
  there is no allowance to stay inside: the screenshot page no longer reports how many
  readings the month has left, and the app no longer reads a remaining-allowance figure out
  of the shared server's answer.

<a id="zh-cn"></a>

## 中文

这一版里，屏幕是在它自己所在的机器上被读的；设置也不再是一堆「一页只放一个开关」的页面。

### 新增

- **截图在本地识别，不再交给服务。** 识别由本机的 PP-OCRv4（ONNX Runtime）完成：图片从
  不离开这台机器，完全断网也能用，读一次不花任何额度。第一次要求截图时会先弹窗说明它有多
  大（约 33 MB），得到同意后才下载，文件落在程序自己的文件旁边；设置里的「删除引擎」把它
  从磁盘上拿走。
- **新的「资源」页。** Glossy 需要在本机保留的东西都在这一处下载、显示占用和删除：今天是
  识别引擎，以后是离线翻译包。
- **这一个副本是哪一版。** 「更新」页把版本号写在检查按钮上方，报一个问题时不用再猜是哪
  个构建。

### 变更

- **设置页更少了。** 「常规」页现在一并放着快捷键和触发规则（这三件事本来也是一起读的），
  「生词本」挪到上面的功能区，和 Glossy 其它做的事放在一起。
- **文档翻译在重做期间关闭。** 入口被禁用并标为「开发中」，页面里有说明，背后的命令直接
  拒绝，而不是开始一个暂时没法做完的翻译。已经保存的设置不受影响。

### 移除

- **每月截图次数上限。** 识别已经在本机完成，也就没有需要守住的额度：截图页不再报告本月
  还剩多少次，程序也不再从中转服务器的应答里读取「剩余额度」这个数字。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
