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

    "panel.language": "Language and provider",
    "field.targetLang": "Translate into",
    "field.provider": "Translation provider",
    "provider.google": "Google · free, no key",
    "provider.baidu": "Baidu 翻译 · free monthly quota, APP ID + key",
    "provider.zhipu": "Zhipu GLM · free tier, API key",
    "provider.deepl": "DeepL · API key",
    "provider.openai": "OpenAI · API key",
    "providerName.google": "Google",
    "providerName.baidu": "Baidu Translate",
    "providerName.zhipu": "Zhipu GLM",
    "providerName.deepl": "DeepL",
    "providerName.openai": "OpenAI",
    "field.appId": "APP ID",
    "field.apiKey": "API key",
    "apiKey.placeholder": "Paste your key",
    "apiKey.keep": "The key you saved for this provider is already filled in.",
    "language.hint": "The source language is detected automatically.",

    "provider.hint.google":
      "The free public Google endpoint. Phonetics, parts of speech, definitions and an example come from the same call. No key needed.",
    "provider.hint.baidu":
      "Paste the APP ID and the 密钥 from fanyi-api.baidu.com. The free tier gives 50,000 characters per month, or 1,000,000 after the free 个人认证. Reachable from mainland China.",
    "provider.hint.zhipu":
      "Paste a Zhipu (open.bigmodel.cn) API key. The free <code>glm-4.7-flash</code> model returns the translation, phonetics, definitions and an example in one call, and it is reachable from mainland China.",
    "provider.hint.deepl":
      "Paste a DeepL API key. Keys ending in <code>:fx</code> use the free endpoint.",
    "provider.hint.openai":
      "Paste an OpenAI API key. Word details are filled in from the free provider.",

    "panel.demo": "Translate",
    "demo.hint":
      "Type or paste text below, then select a part of it — a word, a phrase or a sentence — or press Translate. The result appears in the same card the floating popup draws.",
    "demo.placeholder":
      "Type or paste the text to translate, then select a part of it…",
    "demo.run": "Translate",
    "demo.button": "Show in floating popup",
    "demo.nothing": "Nothing selected yet.",
    "demo.demoText":
      "The quick brown fox jumps over the lazy dog. 敏捷的棕色狐狸跳过了那只懒狗。Select any part of this paragraph to see Glossy at work.",

    "toast.saved": "Settings saved",
    "toast.saveFailed": "Could not save: ",
    "toast.loadFailed": "Could not load settings: ",

    "popup.copy": "Copy translation",
    "popup.close": "Close",
    "popup.sourceLang": "Source language",
    "popup.targetLang": "Target language",
    "popup.swap": "Swap languages and translate back",
    "popup.autoDetected": "Detect language",

    "render.retry": "Try again",
    "render.empty": "No translation returned.",
    "render.failed": "Translation failed.",
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

    "panel.language": "语言与翻译渠道",
    "field.targetLang": "翻译为",
    "field.provider": "翻译渠道",
    "provider.google": "Google · 免费，无需密钥",
    "provider.baidu": "百度翻译 · 每月免费额度，APP ID + 密钥",
    "provider.zhipu": "智谱 GLM · 免费额度，API Key",
    "provider.deepl": "DeepL · API Key",
    "provider.openai": "OpenAI · API Key",
    "providerName.google": "Google",
    "providerName.baidu": "百度翻译",
    "providerName.zhipu": "智谱 GLM",
    "providerName.deepl": "DeepL",
    "providerName.openai": "OpenAI",
    "field.appId": "APP ID",
    "field.apiKey": "API Key",
    "apiKey.placeholder": "粘贴你的密钥",
    "apiKey.keep": "已填入你为该渠道保存的密钥。",
    "language.hint": "源语言会自动识别。",

    "provider.hint.google":
      "免费的 Google 公共接口：音标、词性、释义和例句都在同一次请求里返回，无需密钥。",
    "provider.hint.baidu":
      "在 fanyi-api.baidu.com 申请后，把 APP ID 和密钥填在这里。免费版每月 5 万字符，完成个人认证后每月 100 万字符，国内可直接访问。",
    "provider.hint.zhipu":
      "填入智谱（open.bigmodel.cn）的 API Key。免费的 <code>glm-4.7-flash</code> 模型一次返回译文、音标、释义和例句，国内可直接访问。",
    "provider.hint.deepl":
      "填入 DeepL API Key。以 <code>:fx</code> 结尾的密钥会使用免费接口。",
    "provider.hint.openai":
      "填入 OpenAI API Key。单词的详细信息会从免费渠道补充。",

    "panel.demo": "翻译",
    "demo.hint":
      "在下面输入或粘贴文本，然后选中其中任意一部分——单词、短语或整句——也可以直接点「翻译」。结果会用与悬浮窗完全相同的卡片就地显示。",
    "demo.placeholder": "输入或粘贴要翻译的文本，然后选中其中一部分…",
    "demo.run": "翻译",
    "demo.button": "在悬浮窗中显示",
    "demo.nothing": "还没有选中内容。",
    "demo.demoText":
      "The quick brown fox jumps over the lazy dog. 敏捷的棕色狐狸跳过了那只懒狗。选中这段话中的任意部分，看看 Glossy 的效果。",

    "toast.saved": "设置已保存",
    "toast.saveFailed": "保存失败：",
    "toast.loadFailed": "读取设置失败：",

    "popup.copy": "复制译文",
    "popup.close": "关闭",
    "popup.sourceLang": "源语言",
    "popup.targetLang": "目标语言",
    "popup.swap": "交换语言并反向翻译",
    "popup.autoDetected": "自动检测",

    "render.retry": "重试",
    "render.empty": "没有返回译文。",
    "render.failed": "翻译失败。",
  };

  /** Language names shown in the target-language list. */
  const LANGUAGE_NAMES = {
    zh: {
      auto: "自动检测",
      en: "英语",
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

  /** Fills every `data-i18n*` element below `root`. */
  function apply(root) {
    const scope = root || document;
    scope.querySelectorAll("[data-i18n]").forEach((element) => {
      element.innerHTML = t(element.dataset.i18n);
    });
    scope.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
      element.placeholder = t(element.dataset.i18nPlaceholder);
    });
    scope.querySelectorAll("[data-i18n-title]").forEach((element) => {
      element.title = t(element.dataset.i18nTitle);
    });
  }

  /** Localized name of a language code, falling back to the code itself. */
  function languageName(code, fallback) {
    const table = LANGUAGE_NAMES[language];
    if (table && code && table[code]) return table[code];
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
