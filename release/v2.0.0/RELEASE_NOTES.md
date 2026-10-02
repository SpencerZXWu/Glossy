[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that writes the project's promises down, and uses the contract break it is
allowed to take to put the settings file in one piece.

### Added

- **`FAQ.md`, `PROVIDERS.md`, `COMPATIBILITY.md`.** The questions the README answered only
  in passing, the terms of every translation service the app offers, and what a new version
  may do to an installation that already exists — three documents a reader can check instead
  of trusting, each linked from the README.

### Changed

- **The settings file is format version `2`.** The choice of translation service used to be
  spelled across four keys, because the app once offered modes it no longer has; it is now
  one `service` value (`cloud-baidu`, `cloud-youdao`, `google` or `offline`). Every settings
  file from `1.3.0` onwards is migrated on first launch, and nothing else in it moves: the
  language, the shortcuts, the ignored programs and the history setting all arrive as they
  were. This is the one break a major version is allowed to make, and
  [`COMPATIBILITY.md`](https://github.com/SpencerZXWu/Glossy/blob/main/COMPATIBILITY.md) is
  the promise about it, key by key.
- **A file written by a newer version is set aside rather than misread.** The rule already
  existed; the compatibility statement is where it is written down now.

### Removed

- **The mode that took an API key of the user's own**, and the storage that went with it: the
  `credentials` map, the `apiKey` and `appId` fields, the DPAPI machinery that encrypted them
  and the module that held it. No service this build offers asks for a key — the two
  server-backed entries hold the vendor account on the relay, Google takes none and the
  offline models run here — so the mode had nothing left to configure. A key left in an old
  settings file is dropped the next time the settings are saved, and is neither used nor sent
  anywhere. Nothing in either window changes: the credential panel was already gone in `0.8.0`,
  and the dropdown had four entries before this release and has four after it.
- **The relay address as a setting.** It was already part of the build rather than a field in
  the window; the `cloudEndpoint` key a `1.x` file may hold is dropped with the rest.

### Fixed

- **Choosing the offline channel in the settings window did not stick.** The list the window
  validates the stored service against held only the three services that go through a network,
  so `offline` was read as an unknown value and the dropdown fell back to Baidu the moment the
  save came back — the choice was written to the file and then shown as something else, and
  reopening the window showed Baidu while the card translated offline. The list holds all four
  entries now, and the list of fallbacks is a second, shorter one that leaves `offline` out:
  the models on this machine are a choice the reader makes, never a substitution the app makes
  for them, which is what the README has promised all along. The backend now drops `offline`
  from a hand-edited fallback order too, so the promise holds for a file as well as for a click.
- **`PRIVACY.md` and the README described screenshot translation as going through the relay.**
  It does not: PP-OCRv4 reads the rectangle on this machine and only the recognised text is
  sent, which is what the code has done since the reader became local. The privacy document
  said otherwise, and the Chinese settings table promised a monthly screenshot allowance that
  no build has counted since then. Both now say what actually happens.

<a id="zh-cn"></a>

## 中文

这个版本把项目的承诺写了下来，并使用大版本允许的那一次契约打破，让设置文件只保留一种形状。

### 新增

- **`FAQ.md`、`PROVIDERS.md`、`COMPATIBILITY.md`。** README 只顺带回答过的问题、应用提供的每一项翻译服务的条款，以及新版本可以对已有的安装做什么——三份可以去核对而不必相信的文件，README 里都有链接。

### 变更

- **设置文件改为格式版本 `2`。** "用哪个服务翻译"过去分散在四个键里，因为当时的应用还提供现在已经没有的模式；现在它是唯一一个 `service` 值（`cloud-baidu`、`cloud-youdao`、`google` 或 `offline`）。`1.3.0` 及以后写出的所有设置文件都会在首次启动时迁移，文件里的其他内容不会移动：语言、快捷键、忽略的程序和历史记录设置都保持原样。这是大版本被允许做的唯一一次打破，[`COMPATIBILITY.md`](https://github.com/SpencerZXWu/Glossy/blob/main/COMPATIBILITY.md) 就是关于它的承诺，逐键说明。
- **由更新版本写出的文件会被搁置而不是被误读。** 这条规则本来就有，现在它写在了兼容性声明里。

### 移除

- **索要用户自有 API 密钥的那个模式**，以及随它一起消失的存储：`credentials` 映射、`apiKey` 与 `appId` 字段、加密它们的 DPAPI 机制，以及承载它的模块。本构建提供的服务都不需要密钥——两个走服务器的入口把厂商账号留在中转服务上，Google 不需要密钥，离线模型就跑在本机——所以这个模式已经没有任何东西可配。旧设置文件里残留的密钥会在下次保存设置时被丢弃，既不会被使用，也不会被发到任何地方。两个窗口都没有任何变化：凭据面板在 `0.8.0` 就已经消失，下拉框在本版本之前有四项，之后也是四项。
- **把中转地址做成设置项。** 它本来就是构建的一部分，而不是窗口里的输入框；`1.x` 文件中可能存在的 `cloudEndpoint` 键会和其他键一起被丢弃。

### 修复

- **在设置窗口里选择离线渠道无法保持。** 窗口用来校验已保存服务的那个列表只包含三个走网络的服务，于是 `offline` 被当作未知值，保存结果一回来下拉框就退回百度——选择被写进了文件，显示的却是别的东西；重新打开窗口显示百度，而卡片其实在用离线翻译。现在这个列表包含全部四项，而备用服务是另一个更短的列表，其中不含 `offline`：本机模型是读者主动做出的选择，从来不是应用替他做的替换——这正是 README 一直以来的承诺。后端现在也会把手工写进备用顺序的 `offline` 去掉，因此这条承诺对文件同样成立，而不只是对点击成立。
- **`PRIVACY.md` 和 README 把截图翻译描述成会经过中转服务。** 它并不会：PP-OCRv4 在本机读你框选的那块矩形，只把识别出来的文字发出去——自从识别改在本机进行之后，代码一直是这样。隐私文档写的却是另一回事，中文设置表格里还承诺了一份"本月截图次数"额度，而自那以后没有任何构建统计过它。两处现在都改成了实际发生的情况。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
