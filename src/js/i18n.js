/**
 * Interface language: a small dictionary plus the helpers that push it into the
 * markup.
 *
 * Every element that carries text uses a `data-i18n` attribute naming its key;
 * `data-i18n-placeholder` / `data-i18n-title` cover the attributes. Values may
 * contain markup, so they are applied with `innerHTML`.
 */
(function (Glossy) {
  const ENGLISH = {
    "ui.lang": "Interface language",
    "ui.system": "Follow Windows",
    "ui.zh": "简体中文",
    "ui.en": "English",

    "brand.tagline": "Select text anywhere and translate it on the spot.",
    "notice.title": "Glossy is running in the background",
    "notice.body": "Click here to open the settings, or use the tray icon.",
    "notice.close": "Dismiss",
    "brand.trayHint": "Closing this window keeps Glossy in the notification area. The tray icon is the only way back — it opens this window on its next launch too.",
    "status.listening": "Listening",
    "status.listening.title": "Select text anywhere to translate it.",
    "status.paused": "Paused",
    "status.paused.title": "Selection translation is disabled.",
    "status.unavailable": "Unavailable",
    "status.hook": "Hook unavailable",
    "status.hook.title": "The global mouse hook is not running.",

    "master.title": "Enable text selection translation",
    "master.hint": "Translate a selection as soon as you release the mouse.",
    "option.drag": "Translate when the mouse drags across text",
    "option.doubleClick": "Translate a word on double click",
    "option.restoreClipboard": "Put the clipboard back after reading a selection",
    "option.showOriginal": "Show the original text in the popup",
    "option.autostart": "Start Glossy with Windows",

    "panel.trigger": "Trigger",
    "field.minLength": "Shortest selection to translate",
    "field.hotkey": "Global hotkey",
    "hotkey.values": "Translate the clipboard content without selecting anything.",
    "hotkey.none": "No global hotkey. Type one such as Ctrl+Alt+C.",
    "hotkey.active": "Active: {0}. Press it to translate the clipboard content.",
    "field.ignored": "Never translate in these programs",
    "ignored.empty": "Nothing ignored yet — every program triggers a translation.",
    "ignored.count": "{0} program(s) ignored.",
    "ignored.add": "Add",
    "ignored.manual": "Type a program name",
    "ignored.running": "Pick from running programs…",
    "ignored.pick": "Pick with the mouse",
    "ignored.picking": "Click the window you want to ignore.",
    "ignored.picked": "Added {0}.",
    "ignored.notFound": "That window does not belong to a program.",
    "ignored.remove": "Stop ignoring this program",
    "ignored.loading": "Reading the list of running programs…",
    "ignored.none": "No program with a visible window was found.",
    "field.sourceLangs": "Only translate these source languages",
    "source.empty": "Every language triggers a translation.",
    "source.count": "{0} source language(s) allowed. Anything written in another language is skipped.",
    "source.add": "Add a language…",
    "source.clear": "Any language",
    "source.remove": "Stop translating this language",

    "panel.popup": "Popup",
    "field.theme": "Colours",
    "theme.system": "Follow Windows",
    "theme.light": "Always light",
    "theme.dark": "Always dark",
    "field.fontScale": "Text size",
    "font.90": "90 %",
    "font.100": "100 % · default",
    "font.115": "115 %",
    "font.130": "130 %",
    "font.150": "150 %",
    "field.width": "Width",
    "width.300": "Compact · 300 px",
    "width.356": "Default · 356 px",
    "width.400": "Wide · 400 px",
    "width.460": "Wider · 460 px",
    "width.520": "Extra wide · 520 px",
    "field.autoClose": "Close by itself",
    "autoClose.0": "Never",
    "autoClose.3": "After 3 seconds",
    "autoClose.5": "After 5 seconds",
    "autoClose.10": "After 10 seconds",
    "autoClose.20": "After 20 seconds",
    "autoClose.30": "After 30 seconds",
    "field.opacity": "Opacity",
    "opacity.100": "100 % · solid",
    "opacity.95": "95 %",
    "opacity.90": "90 %",
    "opacity.80": "80 %",
    "opacity.70": "70 %",
    "opacity.60": "60 %",
    "opacity.50": "50 % · see-through",
    "option.closeAfterCopy": "Close the popup right after the translation is copied",

    "panel.units": "Units",
    "option.units": "Convert measurements and money the target language does not use",
    "units.hint": "A measurement or an amount of money that reads the way the source language writes it is shown again the way the target language writes it, with the conversion rate. Currency rates are looked up live.",

    "panel.language": "Language",
    "field.targetLang": "Translate into",
    "field.service": "Translation service",
    "field.localEndpoint": "Local service address",
    "field.localModel": "Model name",
    "provider.cloudBaidu": "Baidu Translate · nothing to set up",
    "provider.cloudYoudao": "Youdao Translate · nothing to set up",
    "provider.local": "Local translation · Ollama (recommended)",
    "provider.google": "Google · free, no key",
    "providerName.google": "Google",
    "providerName.baidu": "Baidu Translate",
    "providerName.youdao": "Youdao Translate",
    "providerName.local": "Local translation",
    "cloud.quota.checking": "Asking the server how much is left today…",
    "cloud.quota.remaining": "Free quota today: {0} of {1} characters left",
    "cloud.quota.used": "Free quota today: all {0} characters are used up, it resets at 00:00 UTC",
    "cloud.quota.retry": "Check again",
    "language.hint": "The source language is detected automatically.",

    "provider.hint.google":
      "The free public Google endpoint. Phonetics, parts of speech, definitions and an example come from the same call. No key needed.",
    "provider.hint.cloudBaidu":
      "Nothing to fill in: the translation runs on the account behind this build, so no key ever reaches the app and each device gets a daily character allowance. Another engine answers when Baidu cannot.",
    "provider.hint.cloudYoudao":
      "Nothing to fill in: the translation runs on the account behind this build, so no key ever reaches the app. Another engine answers when Youdao cannot.",
    "cloud.hint.local":
      "The text goes to a model running on this machine, so nothing leaves the computer and nothing is metered. Any OpenAI compatible service works — Ollama serves <code>http://127.0.0.1:11434/v1</code> — and the model name is the one you pulled. Bigger models translate better; a small one still keeps the whole thing offline.",

    "panel.fallback": "Fallback",
    "option.fallback": "Ask another service when the chosen one fails",
    "fallback.hint":
      "The service picked above is always tried first. The entries below are asked in this order when it fails, rate-limits or answers with nothing.",
    "fallback.up": "Move up",
    "fallback.down": "Move down",
    "fallback.first": "always first",
    "service.cloud-baidu": "Baidu Translate",
    "service.cloud-youdao": "Youdao Translate",
    "service.local": "Local model",
    "service.google": "Google",

    "panel.reading": "Reading",
    "option.wordSentence": "Show the sentence a word was selected from",
    "option.sentencePairs": "Pair the original and the translation sentence by sentence",
    "option.compactPopup": "Draw a compact card, without the extras",
    "field.speechRate": "Speaking rate",
    "speech.slow": "Slow",
    "speech.normal": "Normal",
    "speech.fast": "Fast",
    "speech.fastest": "Very fast",
    "reading.hint":
      "Pronunciation uses the voices Windows already has; the language of a text picks the voice. A word is only looked up in its sentence when the program in front lets Glossy read it.",

    "panel.demo": "Translate",
    "demo.hint":
      "Type or paste text below, then select a part of it — a word, a phrase or a sentence — or press Translate (Ctrl+Enter). A pasted text is translated as soon as it lands. The result appears in the same card the floating popup draws.",
    "demo.placeholder":
      "Type or paste the text to translate, then select a part of it…",
    "demo.run": "Translate",
    "demo.paste": "Paste",
    "demo.clear": "Clear",
    "demo.pasteEmpty": "The clipboard holds no text.",
    "demo.units": "Convert units",
    "demo.button": "Show in floating popup",
    "demo.nothing": "Nothing selected yet.",
    "demo.demoText":
      "The quick brown fox jumps over the lazy dog. 敏捷的棕色狐狸跳过了那只懒狗。Select any part of this paragraph to see Glossy at work.",

    "toast.saved": "Settings saved",
    "toast.saveFailed": "Could not save: ",
    "toast.loadFailed": "Could not load settings: ",
    "toast.copied": "Translation copied",
    "panel.history": "History",
    "field.historyLimit": "Remember",
    "field.historySearch": "Search",
    "history.search": "Text or translation",
    "history.hint": "Click an entry to show it in the floating card again.",
    "history.open": "Show this translation again",
    "history.copy": "Copy this translation",
    "history.remove": "Remove this entry",
    "history.clear": "Forget everything",
    "history.empty": "Nothing translated yet.",
    "history.count": "{0} entries",
    "history.matching": "matched {0} of {1}",
    "history.cleared": "History cleared",
    "history.limit.off": "Off",
    "history.limit.20": "The last 20",
    "history.limit.50": "The last 50",
    "history.limit.100": "The last 100",
    "history.limit.200": "The last 200",
    "history.limit.500": "The last 500",
    "panel.backup": "Settings file",
    "backup.hint": "Export writes Documents\\glossy-settings.json. Import reads back a file you pick.",
    "backup.export": "Export…",
    "backup.import": "Import…",
    "backup.exported": "Settings exported to {0}",
    "backup.exportFailed": "Could not export: ",
    "backup.imported": "Settings imported",
    "backup.importFailed": "Could not import: ",
    "panel.updates": "Updates",
    "option.checkUpdates": "Check for a new version when Glossy starts",
    "update.check": "Check now",
    "update.install": "Download and restart",
    "update.available": "Version {0} is ready to install.",
    "update.upToDate": "Glossy is up to date.",
    "update.found": "Version {0} is available.",
    "update.failed": "Could not update: ",
    "update.downloading": "Downloading…",
    "update.unavailable": "This build carries no update signing key, so it cannot update itself yet.",

    "popup.copy": "Copy translation",
    "popup.close": "Close",
    "popup.pin": "Pin the card, so it stays open",
    "popup.unpin": "Unpin the card",
    "popup.sourceLang": "Source language",
    "popup.targetLang": "Target language",
    "popup.swap": "Swap languages and translate back",
    "popup.autoDetected": "Detect language",

    "render.retry": "Try again",
    "render.empty": "No translation returned.",
    "render.failed": "Translation failed.",
    "render.lookup": "Looking up the dictionary…",
    "render.forms": "Forms",
    "render.synonyms": "Synonyms",
    "render.context": "In this sentence",
    "render.pairs": "Sentence by sentence",
    "render.speakOriginal": "Read the original out loud",
    "render.speakTranslation": "Read the translation out loud",
    "render.stop": "Stop reading",
    "render.fallback": "Answered by {0} after the chosen service failed",

    "form.plural": "plural",
    "form.thirdPerson": "third person",
    "form.presentParticiple": "present participle",
    "form.past": "past",
    "form.pastParticiple": "past participle",
    "form.comparative": "comparative",
    "form.superlative": "superlative",
    "form.other": "other form",

    "units.title": "Units",
    "units.approx": "≈",
    "units.rate": "Live rate",
    "units.stale": "Last known rate",
    "units.source.exchangerateApi": "exchangerate-api.com",
    "units.source.frankfurter": "frankfurter.app",
  };

  const CHINESE = {
    "ui.lang": "界面语言",
    "ui.system": "跟随系统",
    "ui.zh": "简体中文",
    "ui.en": "English",

    "brand.tagline": "在任何地方选中文字，立即翻译。",
    "notice.title": "Glossy 已在后台运行",
    "notice.body": "点这里打开设置，也可以使用托盘图标。",
    "notice.close": "关闭",
    "brand.trayHint": "关闭本窗口后 Glossy 会留在通知区域继续运行。托盘图标是重新打开本窗口的唯一入口，下次启动也不会再自动弹出。",
    "status.listening": "监听中",
    "status.listening.title": "在任意位置选中文字即可翻译。",
    "status.paused": "已暂停",
    "status.paused.title": "划词翻译已关闭。",
    "status.unavailable": "不可用",
    "status.hook": "钩子不可用",
    "status.hook.title": "全局鼠标钩子没有运行。",

    "master.title": "开启划词翻译",
    "master.hint": "松开鼠标即翻译选中的内容。",
    "option.drag": "拖动鼠标划过文字时翻译",
    "option.doubleClick": "双击单词时翻译",
    "option.restoreClipboard": "读取选区后恢复剪贴板",
    "option.showOriginal": "在弹窗中显示原文",
    "option.autostart": "随 Windows 启动",

    "panel.trigger": "触发",
    "field.minLength": "触发翻译的最短长度",
    "field.hotkey": "全局快捷键",
    "hotkey.values": "不用选中文字，直接翻译剪贴板内容。",
    "hotkey.none": "未设置全局快捷键。可填写 Ctrl+Alt+C。",
    "hotkey.active": "已生效：{0}。按下即可翻译剪贴板内容。",
    "field.ignored": "以下程序中不翻译",
    "ignored.empty": "暂未忽略任何程序，所有程序都会触发翻译。",
    "ignored.count": "已忽略 {0} 个程序。",
    "ignored.add": "添加",
    "ignored.manual": "输入程序名",
    "ignored.running": "从运行中的程序选择…",
    "ignored.pick": "用鼠标拾取",
    "ignored.picking": "请点击你想要忽略的程序窗口。",
    "ignored.picked": "已添加 {0}。",
    "ignored.notFound": "该窗口不属于任何程序。",
    "ignored.remove": "不再忽略该程序",
    "ignored.loading": "正在读取运行中的程序…",
    "ignored.none": "没有找到带可见窗口的程序。",
    "field.sourceLangs": "仅翻译以下原文语言",
    "source.empty": "所有语言都会触发翻译。",
    "source.count": "只翻译 {0} 种原文语言，其它语言会被跳过。",
    "source.add": "添加语言…",
    "source.clear": "不限语言",
    "source.remove": "不再翻译该语言",

    "panel.popup": "弹窗",
    "field.theme": "配色",
    "theme.system": "跟随系统",
    "theme.light": "始终浅色",
    "theme.dark": "始终深色",
    "field.fontScale": "文字大小",
    "font.90": "90 %",
    "font.100": "100 % · 默认",
    "font.115": "115 %",
    "font.130": "130 %",
    "font.150": "150 %",
    "field.width": "宽度",
    "width.300": "紧凑 · 300 像素",
    "width.356": "默认 · 356 像素",
    "width.400": "宽 · 400 像素",
    "width.460": "较宽 · 460 像素",
    "width.520": "超宽 · 520 像素",
    "field.autoClose": "自动关闭",
    "autoClose.0": "不自动关闭",
    "autoClose.3": "3 秒后",
    "autoClose.5": "5 秒后",
    "autoClose.10": "10 秒后",
    "autoClose.20": "20 秒后",
    "autoClose.30": "30 秒后",
    "field.opacity": "不透明度",
    "opacity.100": "100 % · 完全不透明",
    "opacity.95": "95 %",
    "opacity.90": "90 %",
    "opacity.80": "80 %",
    "opacity.70": "70 %",
    "opacity.60": "60 %",
    "opacity.50": "50 % · 半透明",
    "option.closeAfterCopy": "复制译文后立即关闭弹窗",

    "panel.units": "单位换算",
    "option.units": "把译文语言不常用的计量与货币换算过来",
    "units.hint": "原文中按源语言习惯书写、而译文语言不常用的计量或货币，会按译文语言的习惯再写一遍并注明换算率，货币汇率实时查询。",

    "panel.language": "语言",
    "field.targetLang": "翻译为",
    "field.service": "翻译渠道",
    "field.localEndpoint": "本地服务地址",
    "field.localModel": "模型名称",
    "provider.cloudBaidu": "百度翻译 · 无需配置",
    "provider.cloudYoudao": "有道翻译 · 无需配置",
    "provider.local": "本地翻译 · Ollama（推荐）",
    "provider.google": "Google · 免费，无需密钥",
    "providerName.google": "Google",
    "providerName.baidu": "百度翻译",
    "providerName.youdao": "有道翻译",
    "providerName.local": "本地翻译",
    "cloud.quota.checking": "正在向服务器查询今天还剩多少额度…",
    "cloud.quota.remaining": "今日免费额度：剩余 {0} / {1} 字符",
    "cloud.quota.used": "今日免费额度：{0} 字符已用完，UTC 时间 0 点恢复",
    "cloud.quota.retry": "重新查询",
    "language.hint": "源语言会自动识别。",

    "provider.hint.google":
      "免费的 Google 公共接口：音标、词性、释义和例句都在同一次请求里返回，无需密钥。",
    "provider.hint.cloudBaidu":
      "无需填写任何密钥：翻译由本软件内置的账号完成，密钥不会进入客户端，每台设备每天有字符额度；百度答不上来时服务器会自动改用别的引擎。",
    "provider.hint.cloudYoudao":
      "无需填写任何密钥：翻译由本软件内置的账号完成，密钥不会进入客户端；有道答不上来时服务器会自动改用别的引擎。",
    "cloud.hint.local":
      "文本会发给你自己电脑上运行的模型，不经过任何服务器，也不计费。任何 OpenAI 兼容服务都行——Ollama 默认是 <code>http://127.0.0.1:11434/v1</code>——模型名就填你 pull 下来的那个。模型越大译得越好，小模型则胜在完全离线。",

    "panel.fallback": "备用服务",
    "option.fallback": "所选服务失败时改用其他服务",
    "fallback.hint":
      "上面选中的服务总是先试。当它失败、被限流或没有返回内容时，按下面的顺序逐个尝试。",
    "fallback.up": "上移",
    "fallback.down": "下移",
    "fallback.first": "始终最先尝试",
    "service.cloud-baidu": "百度翻译",
    "service.cloud-youdao": "有道翻译",
    "service.local": "本地模型",
    "service.google": "Google",

    "panel.reading": "阅读",
    "option.wordSentence": "显示所查单词所在的句子",
    "option.sentencePairs": "原文与译文逐句对照",
    "option.compactPopup": "精简卡片，不显示附加信息",
    "field.speechRate": "朗读语速",
    "speech.slow": "慢",
    "speech.normal": "正常",
    "speech.fast": "快",
    "speech.fastest": "很快",
    "reading.hint":
      "朗读使用 Windows 自带的语音，文本的语言决定用哪一种嗓音。只有当光标所在的程序允许 Glossy 读取时，才会去查单词所在的句子。",

    "panel.demo": "翻译",
    "demo.hint":
      "在下面输入或粘贴文本，然后选中其中任意一部分——单词、短语或整句——也可以直接点「翻译」（Ctrl+Enter）。粘贴进来的整段文本会立即翻译。结果会用与悬浮窗完全相同的卡片就地显示。",
    "demo.placeholder": "输入或粘贴要翻译的文本，然后选中其中一部分…",
    "demo.run": "翻译",
    "demo.paste": "粘贴",
    "demo.clear": "清空",
    "demo.pasteEmpty": "剪贴板里没有文字。",
    "demo.units": "单位换算",
    "demo.button": "在悬浮窗中显示",
    "demo.nothing": "还没有选中内容。",
    "demo.demoText":
      "The quick brown fox jumps over the lazy dog. 敏捷的棕色狐狸跳过了那只懒狗。选中这段话中的任意部分，看看 Glossy 的效果。",

    "toast.saved": "设置已保存",
    "toast.saveFailed": "保存失败：",
    "toast.loadFailed": "读取设置失败：",
    "toast.copied": "译文已复制",
    "panel.history": "历史记录",
    "field.historyLimit": "保留",
    "field.historySearch": "搜索",
    "history.search": "原文或译文",
    "history.hint": "点击一条记录，可在浮动窗口中重新查看。",
    "history.open": "再次显示这条译文",
    "history.copy": "复制这条译文",
    "history.remove": "删除这条记录",
    "history.clear": "清空历史记录",
    "history.empty": "还没有翻译记录。",
    "history.count": "共 {0} 条",
    "history.matching": "匹配 {0} / {1} 条",
    "history.cleared": "历史记录已清空",
    "history.limit.off": "关闭",
    "history.limit.20": "最近 20 条",
    "history.limit.50": "最近 50 条",
    "history.limit.100": "最近 100 条",
    "history.limit.200": "最近 200 条",
    "history.limit.500": "最近 500 条",
    "panel.backup": "设置文件",
    "backup.hint": "导出会写入「文档」目录下的 glossy-settings.json，导入可读取你选择的文件。",
    "backup.export": "导出…",
    "backup.import": "导入…",
    "backup.exported": "设置已导出到 {0}",
    "backup.exportFailed": "导出失败：",
    "backup.imported": "设置已导入",
    "backup.importFailed": "导入失败：",
    "panel.updates": "更新",
    "option.checkUpdates": "启动 Glossy 时检查新版本",
    "update.check": "立即检查",
    "update.install": "下载并重启",
    "update.available": "版本 {0} 已可安装。",
    "update.upToDate": "Glossy 已是最新版本。",
    "update.found": "发现新版本 {0}。",
    "update.failed": "更新失败：",
    "update.downloading": "正在下载…",
    "update.unavailable": "此版本还没有更新签名公钥，暂时无法自动更新。",

    "popup.copy": "复制译文",
    "popup.close": "关闭",
    "popup.pin": "固定窗口，不随点击关闭",
    "popup.unpin": "取消固定",
    "popup.sourceLang": "源语言",
    "popup.targetLang": "目标语言",
    "popup.swap": "交换语言并反向翻译",
    "popup.autoDetected": "自动检测",

    "render.retry": "重试",
    "render.empty": "没有返回译文。",
    "render.failed": "翻译失败。",
    "render.lookup": "词典查询中…",
    "render.forms": "词形变化",
    "render.synonyms": "近义词",
    "render.context": "所在句子",
    "render.pairs": "逐句对照",
    "render.speakOriginal": "朗读原文",
    "render.speakTranslation": "朗读译文",
    "render.stop": "停止朗读",
    "render.fallback": "所选服务失败，由 {0} 回答",

    "form.plural": "复数",
    "form.thirdPerson": "第三人称单数",
    "form.presentParticiple": "现在分词",
    "form.past": "过去式",
    "form.pastParticiple": "过去分词",
    "form.comparative": "比较级",
    "form.superlative": "最高级",
    "form.other": "其他形式",

    "units.title": "单位换算",
    "units.approx": "≈",
    "units.rate": "实时汇率",
    "units.stale": "离线，最后一次已知汇率",
    "units.source.exchangerateApi": "exchangerate-api.com",
    "units.source.frankfurter": "frankfurter.app",
  };

  /** Language names shown in the target-language list. */
  const LANGUAGE_NAMES = {
    zh: {
      auto: "自动检测",
      en: "英语",
      zh: "中文",
      "zh-CN": "简体中文",
      "zh-TW": "繁体中文",
      ja: "日语",
      ko: "韩语",
      fr: "法语",
      de: "德语",
      es: "西班牙语",
      pt: "葡萄牙语",
      it: "意大利语",
      ru: "俄语",
      uk: "乌克兰语",
      nl: "荷兰语",
      pl: "波兰语",
      tr: "土耳其语",
      ar: "阿拉伯语",
      hi: "印地语",
      th: "泰语",
      vi: "越南语",
      id: "印尼语",
      ms: "马来语",
      cs: "捷克语",
      da: "丹麦语",
      fi: "芬兰语",
      el: "希腊语",
      he: "希伯来语",
      hu: "匈牙利语",
      no: "挪威语",
      ro: "罗马尼亚语",
      sk: "斯洛伐克语",
      sv: "瑞典语",
    },
  };

  const TABLES = { en: ENGLISH, zh: CHINESE };
  let language = "en";

  /** Turns the stored preference into the language actually used. */
  function resolve(preference) {
    if (preference === "zh" || preference === "en") return preference;
    const locales = navigator.languages || [navigator.language || "en"];
    return String(locales[0] || "en").toLowerCase().startsWith("zh") ? "zh" : "en";
  }

  function set(preference) {
    language = resolve(preference);
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
    return language;
  }

  /** Looks up `key`, replacing `{0}`, `{1}` … with the extra arguments. */
  function t(key, ...values) {
    const table = TABLES[language] || ENGLISH;
    const text = table[key] !== undefined ? table[key] : ENGLISH[key];
    if (text === undefined) return key;
    return text.replace(/\{(\d+)\}/g, (match, index) => {
      const value = values[Number(index)];
      return value === undefined ? match : String(value);
    });
  }

  /** Fills every `data-i18n*` element at or below `root`. */
  function apply(root) {
    const scope = root || document;
    const fill = (selector, setter) => {
      const found = Array.from(scope.querySelectorAll(selector));
      // A root element is not part of its own `querySelectorAll` result.
      if (scope.nodeType === 1 && scope.matches(selector)) found.unshift(scope);
      found.forEach(setter);
    };
    fill("[data-i18n]", (element) => {
      element.innerHTML = t(element.dataset.i18n);
    });
    fill("[data-i18n-placeholder]", (element) => {
      element.placeholder = t(element.dataset.i18nPlaceholder);
    });
    fill("[data-i18n-title]", (element) => {
      element.title = t(element.dataset.i18nTitle);
    });
  }

  /** Localized name of a language code, falling back to the code itself. */
  function languageName(code, fallback) {
    const table = LANGUAGE_NAMES[language];
    if (table && code) {
      const value = String(code);
      if (table[value]) return table[value];
      // Providers spell codes differently ("zh-CN" vs "zh-cn").
      const needle = value.toLowerCase();
      const match = Object.keys(table).find((key) => key.toLowerCase() === needle);
      if (match) return table[match];
    }
    return fallback ? fallback(code) : String(code || "");
  }

  /** Friendly name of a provider id (`google` → `Google`), falling back to the id. */
  function providerName(id) {
    const key = `providerName.${String(id || "").toLowerCase()}`;
    const name = t(key);
    return name === key ? String(id || "") : name;
  }

  Glossy.i18n = { set, resolve, t, apply, languageName, providerName, language: () => language };
})(window.Glossy);
