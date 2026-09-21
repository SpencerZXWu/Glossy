/**
 * Bridge between the UI and the Rust backend.
 *
 * Inside Tauri `invoke`/`listen` talk to the native commands. When the page is
 * opened in a normal browser (no `window.__TAURI__`) every call is answered by
 * an in-memory mock so the UI can be worked on without a full build.
 */
(function () {
  const internals = window.__TAURI__ && window.__TAURI__.core ? window.__TAURI__.core : null;
  const eventsApi = window.__TAURI__ && window.__TAURI__.event ? window.__TAURI__.event : null;
  const live = !!(internals && typeof internals.invoke === "function");

  /** event name -> Set<handler>, used to dispatch in preview mode only. */
  const previewListeners = new Map();

  const MOCK_SETTINGS = {
    enabled: true,
    triggerOnDrag: true,
    triggerOnDoubleClick: true,
    targetLang: "zh-CN",
    channel: "cloud",
    cloudProvider: "builtin",
    cloudVendor: "baidu",
    localEndpoint: "http://127.0.0.1:11434/v1",
    localModel: "qwen2.5:7b",
    provider: "baidu",
    credentials: {},
    restoreClipboard: true,
    showOriginal: true,
    minSelectionLen: 2,
    ignoredApps: [],
    theme: "system",
    fontScale: 100,
    popupWidth: 356,
    popupOpacity: 100,
    autoCloseSecs: 0,
    closeAfterCopy: false,
    unitsEnabled: true,
    wordSentence: true,
    sentencePairs: true,
    compactPopup: false,
    speechRate: 0,
    fallbackEnabled: true,
    fallbackOrder: ["google", "local"],
    hotkey: "Ctrl+Alt+C",
    uiLang: "system",
  };

  let settings = { ...MOCK_SETTINGS };

  function previewEmit(event, payload) {
    const set = previewListeners.get(event);
    if (!set) return;
    set.forEach((handler) => handler({ event, payload }));
  }

  function stripTags(value) {
    return String(value || "").replace(/<\/?[^>]+>/g, "");
  }

  /** A conversion of each kind, the way the backend sends them. */
  const MOCK_CONVERSIONS = [
    {
      category: "length",
      original: "12 ft",
      converted: "3.66 m",
      rate: "1 ft = 0.3048 m",
      stale: false,
    },
    {
      category: "currency",
      original: "$200",
      converted: "¥1,430.00",
      rate: "1 USD = 7.15 CNY",
      rateSource: "exchangerate-api.com",
      rateDate: "2026-02-05",
      stale: false,
    },
  ];

  function mockTranslate(text, overrides) {
    const source = String(text || "").trim();
    const forced = overrides || {};
    const isWord = source.length <= 32 && source.split(/\s+/).length <= 4;
    const base = {
      sourceText: source,
      sourceLang: forced.sourceLang || (/[\u4e00-\u9fff]/.test(source) ? "zh-CN" : "en"),
      targetLang: forced.targetLang || settings.targetLang,
      provider: "Google · preview",
    };
    if (isWord) {
      return {
        ...base,
        kind: "word",
        translation: "跑步",
        phonetic: "ˈrəniNG",
        meanings: [
          { partOfSpeech: "noun", definitions: ["赛跑", "跑步"] },
          { partOfSpeech: "adverb", definitions: ["连续地", "不断地"] },
          { partOfSpeech: "adjective", definitions: ["跑动的", "流动的"] },
        ],
        example: "marathon " + source,
        synonyms: ["jogging", "sprinting", "dashing"],
        forms: [
          { tag: "plural", text: "runnings" },
          { tag: "thirdPerson", text: "runs" },
          { tag: "presentParticiple", text: "running" },
          { tag: "past", text: "ran" },
          { tag: "pastParticiple", text: "run" },
        ],
        context: settings.wordSentence === false
          ? null
          : {
              text: `${source} keeps the whole street awake at night.`,
              translation: `这个${source}整夜吵得整条街都睡不着。`,
            },
        conversions: [],
      };
    }
    const sentence = {
      ...base,
      kind: "sentence",
      translation: "敏捷的棕色狐狸跳过了那只懒狗。这句话包含了英文里所有字母，常被用来测试字体和键盘。",
      phonetic: null,
      meanings: [],
      example: null,
      // What the backend annotates on a sentence that mixes units and money.
      conversions: settings.unitsEnabled === false ? [] : MOCK_CONVERSIONS,
    };
    if (settings.sentencePairs !== false) {
      sentence.pairs = [
        { source: "The quick brown fox jumps over the lazy dog.", translation: "敏捷的棕色狐狸跳过了那只懒狗。" },
        { source: "Then it keeps on running.", translation: "然后它继续跑着。" },
      ];
    }
    if (previewWantsFallback()) sentence.fallbackFrom = "Baidu Cloud";
    return sentence;
  }

  /** Whether the preview was asked to show a card answered by a fallback. */
  function previewWantsFallback() {
    return /(^|[?&])fallback(=|&|$)/.test(window.location.search || "");
  }

  async function mockInvoke(command, args) {
    const input = args || {};
    switch (command) {
      case "get_settings":
        return { ...settings };
      case "save_settings":
        settings = { ...settings, ...(input.settings || {}) };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "export_settings":
        return "C:\\Users\\you\\Documents\\glossy-settings.json";
      case "import_settings":
        settings = { ...settings, ...(JSON.parse(input.json || "{}") || {}) };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "history_list":
        return [];
      case "history_clear":
      case "history_remove":
      case "history_reopen":
        return null;
      case "update_capability":
        return true;
      case "check_for_update":
        // The browser preview has no release feed to ask.
        return null;
      case "surface_info":
        // Nothing is drawn behind a browser tab.
        return { backdrop: false, ready: true };
      case "install_update":
        return null;
      case "translate_text":
        await new Promise((resolve) => setTimeout(resolve, 350));
        if (!String(input.text || "").trim()) throw new Error("nothing selected");
        return mockTranslate(input.text, input);
      case "word_details":
        return {
          phonetic: "ˈrəniNG",
          meanings: [{ partOfSpeech: "noun", definitions: ["赛跑", "跑步"] }],
          example: "marathon " + String(input.text || "").trim(),
          synonyms: ["jogging", "sprinting"],
          forms: [
            { tag: "past", text: "ran" },
            { tag: "pastParticiple", text: "run" },
          ],
          context: settings.wordSentence === false
            ? null
            : {
                text: `${String(input.text || "").trim()} keeps the whole street awake at night.`,
                translation: "它整夜吵得整条街都睡不着。",
              },
        };
      case "say":
        // The browser preview has no voice; answering keeps the buttons alive.
        await new Promise((resolve) => setTimeout(resolve, 120));
        return true;
      case "stop_speaking":
        return true;
      case "capture_status":
        return { hooked: true, error: null, hotkey: "Ctrl+Alt+C", hotkeyError: null };
      case "cloud_status":
        // The preview has no server behind it, so it reports a plausible day.
        return { used: 1234, limit: 30000, remaining: 28766 };
      case "running_apps":
        return [
          { name: "explorer.exe", title: "File Explorer" },
          { name: "chrome.exe", title: "Preview · Glossy" },
          { name: "Code.exe", title: "app.js - Glossy - Visual Studio Code" },
        ];
      case "pick_app":
        previewEmit("glossy://picked-app", { name: "chrome.exe" });
        return null;
      case "copy_text":
        if (navigator.clipboard) {
          await navigator.clipboard.writeText(input.text || "").catch(() => {});
        }
        return true;
      case "read_clipboard":
        // The preview reads the browser clipboard; the app reads the Windows one.
        if (!navigator.clipboard || !navigator.clipboard.readText) return "";
        return await navigator.clipboard.readText().catch(() => "");
      case "popup_present":
      case "popup_resize":
        // The real backend reports the usable height of the monitor the popup
        // landed on; in the preview the single browser window is that monitor.
        return Number(window.screen && window.screen.availHeight) || null;
      case "popup_sync_anchor":
      case "popup_close":
      case "popup_set_pinned":
      case "show_popup":
        return null;
      default:
        return null;
    }
  }

  async function invoke(command, args) {
    if (live) return internals.invoke(command, args);
    return mockInvoke(command, args);
  }

  /**
   * Reads the persisted settings, retrying while the backend is still starting.
   *
   * The windows defined in `tauri.conf.json` begin loading before the Rust
   * `setup` hook has registered the application state, so the very first call
   * can arrive too early and be rejected with "state not managed".
   */
  async function readSettings(attempts = 25) {
    for (let attempt = 0; ; attempt += 1) {
      try {
        return await invoke("get_settings");
      } catch (error) {
        if (attempt >= attempts) throw error;
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
    }
  }

  /**
   * Subscribes to a backend event. Returns a promise of the unlisten function.
   */
  function listen(event, handler) {
    if (live && eventsApi) return eventsApi.listen(event, handler);
    if (!previewListeners.has(event)) previewListeners.set(event, new Set());
    previewListeners.get(event).add(handler);
    return Promise.resolve(() => previewListeners.get(event).delete(handler));
  }

  window.Glossy = { live, invoke, listen, readSettings, stripTags };
  window.Glossy.preview = !live;
})();
