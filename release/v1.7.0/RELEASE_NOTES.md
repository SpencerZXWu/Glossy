[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where Glossy writes down what went wrong. A failure in the mouse hook or in the
card no longer ends the process in silence, and the messages the app used to print to a
console nobody was watching now land in a file beside its settings.

### Added

- **An error log beside the settings.** `glossy.log` sits next to `settings.json` and takes
  every failure the app reports, stamped in UTC. It stops at 256 KB by becoming
  `glossy.log.1` and starting a new file, so a machine that runs for months leaves two
  bounded files instead of one that grows without end; nothing else is ever written into it.
  The new **Files and logs** section of the settings window — the page that was *Settings
  file* — shows its path and how much it holds, and offers `Open the folder`, which opens
  Explorer with the file selected, `Export…`, which writes it to `Documents\glossy-log-<unix
  seconds>.txt` the way an exported settings file is written, `Copy`, which puts it on the
  clipboard to paste into a message, and `Clear`. The log exists because a release is linked
  as a Windows GUI application and has no console at all: everything the app printed with
  `eprintln!` reached nobody when it was started from Explorer or the notification area,
  which is how it is normally started.
- **The Updates section says where a new build can be downloaded by hand.** The line that
  already says this build cannot update itself now ends with the releases address —
  `github.com/SpencerZXWu/Glossy/releases/latest` — and clicking it opens that page in the
  browser, which is the way to a new build until a signing key pair exists. The window itself
  never navigates anywhere: the address is part of the build, and the command that opens it
  takes no argument, so nothing else can be opened this way.

### Changed

- **The log is handed over by the user, not sent anywhere.** The planned anonymous reporting
  — a payload shown before it was sent, described in `PRIVACY.md` — is not built: `Export…`
  and `Copy` give whoever is asked for the log exactly the same thing, and nothing about a
  failure leaves the machine unless the user does it themselves. The settings page that was
  called *Settings file* is now **Files and logs**, because it holds both.

### Fixed

- **A failure in the mouse hook or in the card no longer takes the notification area icon
  with it.** The hook callback is called by Windows, and a panic that unwound out of it
  would have left through an `extern` boundary and aborted the whole process. The click
  handler, the accelerator handlers and the thread that reads a selection are now guarded:
  a failure is written to the log and the next click is answered as if nothing had happened.
  A panic anywhere else is recorded with its location before the process goes, which is the
  only way a crash nobody was watching can still be read afterwards.

<a id="zh-cn"></a>

## 中文

这一版让 Glossy 把出错的地方写下来。鼠标钩子或卡片里的失败不再无声地结束进程，应用原本打印给「没人在看的控制台」的那些信息，现在会落到设置旁边的一个文件里。

### 新增

- **设置旁边的错误日志。** `glossy.log` 与 `settings.json` 放在同一个文件夹，记录应用上报的每一次失败，时间戳用 UTC。到 256 KB 就会变成 `glossy.log.1` 并另起一个新的，所以一台连续运行几个月的机器只会留下两个大小有限的文件，而不是一个无限增长的文件；里面不会写别的东西。设置窗口新增的**文件与日志**一节——原来叫「设置文件」的那一页——会显示它的位置和已有大小，并提供 `打开所在文件夹`（用资源管理器打开并选中该文件）、`导出…`（按导出设置文件的方式写到 `Documents\glossy-log-<Unix 秒数>.txt`）、`复制`（放进剪贴板，方便贴进消息里）和 `清空`。之所以要有这份日志，是因为发布的程序链接成 Windows GUI 应用，完全没有控制台：从资源管理器或通知区域启动——也就是通常的启动方式——应用用 `eprintln!` 打印的内容谁也看不到。
- **「更新」页写明了可以手动下载新版本的地方。** 原本就说明本构建无法自动更新的那一行，现在末尾给出了发布页地址——`github.com/SpencerZXWu/Glossy/releases/latest`——点击它会在浏览器里打开该页面；在签名密钥对存在之前，这就是拿到新版本的途径。窗口自身不会导航到任何地方：地址是构建的一部分，打开它的命令不接受参数，因此没有别的东西能通过这条路被打开。

### 变更

- **日志由用户自己交出去，不会被发送到任何地方。** 原计划的匿名上报——发送前展示将要发出的内容、并在 `PRIVACY.md` 里说明——没有做：`导出…` 和 `复制` 能给被问到日志的人完全相同的东西，而且除非用户自己动手，任何失败信息都不会离开这台机器。原来叫「设置文件」的设置页现在叫**文件与日志**，因为它同时承载这两者。

### 修复

- **鼠标钩子或卡片里的失败不再把通知区域图标一起带走。** 钩子回调由 Windows 调用，从这里逃逸的 panic 会穿过 `extern` 边界并中止整个进程。现在点击处理、快捷键处理和取词的线程都加了保护：失败会写进日志，下一次点击照常得到应答。其他位置的 panic 也会在进程退出前连位置一起记录下来——这是没人看着时发生的崩溃，事后唯一还能被读到的办法。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
