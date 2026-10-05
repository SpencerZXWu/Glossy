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

  /** What `ocr_model_status` answers in the preview: the engine and the six
   * languages, two of them already downloaded. */
  const ENGINE_INSTALLED = {
    installed: true,
    downloading: false,
    bytes: 33396436,
    total: 33396436,
    error: null,
    size: 22512229 + 10884207 + 9770667,
    coreSize: 22512229,
    folder: "C:\\Users\\you\\AppData\\Roaming\\com.glossy.app\\ocr",
    packs: [
      { id: "ch", installed: true, size: 10884207, downloading: false },
      { id: "ja", installed: true, size: 9770667, downloading: false },
      { id: "cht", installed: false, size: 11185979, downloading: false },
      { id: "latin", installed: false, size: 8978659, downloading: false },
      { id: "cyrillic", installed: false, size: 8972823, downloading: false },
      { id: "ko", installed: false, size: 24082260, downloading: false },
    ],
    target: null,
  };

  const MOCK_SETTINGS = {
    enabled: true,
    triggerOnDrag: true,
    triggerOnDoubleClick: true,
    targetLang: "zh-CN",
    service: "cloud-baidu",
    palette: "default",
    accent: "",
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
    fallbackOrder: ["google", "cloud-youdao"],
    hotkey: "Ctrl+Alt+C",
    hotkeySettings: "Ctrl+Alt+G",
    hotkeyOcr: "Ctrl+Alt+Q",
    ocrPacks: ["ch", "ja"],
    developerMode: false,
    subtitleSourceLang: "ja",
    subtitleTargetLang: "zh-CN",
    subtitlePack: "ja",
    subtitleFontSize: 26,
    uiLang: "system",
  };

  /** The subtitle reading the preview pretends is running, if any. */
  let previewSubtitle = { running: false };

  /** What `offline_model_status` answers in the preview: the pack before it has
   * been downloaded, which is what a fresh install sees. */
  let previewOffline = {
    installed: false,
    downloading: false,
    bytes: 0,
    total: 0,
    error: null,
    size: 0,
    full: 256_012_691,
    folder: "C:\\Users\\you\\AppData\\Roaming\\com.glossy.translator\\translate",
    target: null,
    pairs: [
      { id: "zh-en", source: "zh-CN", target: "en", installed: false, size: 119_123_183, downloading: false },
      { id: "en-zh", source: "en", target: "zh-CN", installed: false, size: 119_122_796, downloading: false },
    ],
  };

  let settings = { ...MOCK_SETTINGS };

  /** The engine each service reports as the one that answered. */
  const SERVICE_ENGINES = {
    "cloud-baidu": "baidu",
    "cloud-youdao": "youdao",
    google: "google",
    offline: "offline",
  };

  /** The service the stored settings name, the way the backend reads it. */
  function storedService() {
    const wanted = String(settings.service || "");
    return SERVICE_ENGINES[wanted] ? wanted : "cloud-baidu";
  }

  /**
   * Whether the preview's imaginary voice is still reading. The real backend
   * reports this itself; here the button would go quiet a frame after it lit up,
   * so a reading is held for as long as a short sentence would take.
   */
  let previewSpeaking = false;

  /** The timer that ends the preview's imaginary reading. */
  let previewSpeechTimer = 0;

  /** The words the preview pretends were kept in the wordbook, newest first. */
  let previewVocabulary = [];

  /** The log the preview pretends Glossy has been writing. */
  let previewLog = {
    path: "C:\\Users\\you\\AppData\\Roaming\\com.glossy.translator\\glossy.log",
    size: 26112,
    cap: 262144,
  };
  let previewWordId = 0;

  function previewKeeps(sourceText, targetLang) {
    return previewVocabulary.some(
      (entry) =>
        entry.result.sourceText === sourceText &&
        (!targetLang || entry.result.targetLang === targetLang)
    );
  }

  function previewToggle(result) {
    const index = previewVocabulary.findIndex(
      (entry) =>
        entry.result.sourceText === result.sourceText &&
        entry.result.targetLang === result.targetLang
    );
    const kept = index === -1;
    if (kept) {
      previewWordId += 1;
      previewVocabulary.unshift({ id: previewWordId, at: Date.now(), result });
    } else {
      previewVocabulary.splice(index, 1);
    }
    previewEmit("glossy://vocabulary", {});
    return kept;
  }

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

  /**
   * Whether the preview's offline pack has the direction a call asks for.
   *
   * The real channel is chosen by hand and answers with an error when the pack
   * for that direction was never downloaded, so the preview has to know which
   * of its two rows are on its imaginary disk before it can answer at all.
   */
  function offlinePairReady(input) {
    const asked = (value) => String(value || "").trim().toLowerCase();
    const source = asked(input && input.sourceLang);
    const target = asked(input && input.targetLang);
    return previewOffline.pairs.some((pair) => {
      if (source && asked(pair.source) !== source) return false;
      if (target && asked(pair.target) !== target) return false;
      return pair.installed;
    });
  }

  function mockTranslate(text, overrides) {
    const source = String(text || "").trim();
    const forced = overrides || {};
    const isWord = source.length <= 32 && source.split(/\s+/).length <= 4;
    const base = {
      sourceText: source,
      sourceLang: forced.sourceLang || (/[\u4e00-\u9fff]/.test(source) ? "zh-CN" : "en"),
      targetLang: forced.targetLang || settings.targetLang,
      provider: SERVICE_ENGINES[storedService()] || "google",
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
    if (previewWantsFallback()) {
      // The case the card exists for: the service that was asked for did not
      // answer, so the card names the one that did and the one that did not.
      sentence.provider = "youdao";
      sentence.fallbackFrom = "baidu";
      // The relay walked past the engine the app asked for and says why, which
      // is what the card reads to name the reason.
      if (/(^|[?&])fallbackwhy(=|&|$)/.test(window.location.search || "")) {
        sentence.fallbackCode = "upstream_limit";
      }
    }
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
      case "log_status":
        return { ...previewLog };
      case "log_open_folder":
        return null;
      case "log_copy":
        return null;
      case "log_export":
        return "C:\\Users\\you\\Documents\\glossy-log-1790000000.txt";
      case "log_clear":
        previewLog = { ...previewLog, size: 0 };
        return null;
      case "open_releases_page":
        return null;
      case "history_list":
        return [];
      case "vocabulary_list":
        return previewVocabulary.map((entry) => ({ ...entry }));
      case "vocabulary_toggle":
        return previewToggle(input.result || {});
      case "vocabulary_keeps":
        return previewKeeps(input.sourceText, input.targetLang);
      case "vocabulary_remove":
        previewVocabulary = previewVocabulary.filter((entry) => entry.id !== input.id);
        previewEmit("glossy://vocabulary", {});
        return null;
      case "vocabulary_clear":
        previewVocabulary = [];
        previewEmit("glossy://vocabulary", {});
        return null;
      case "vocabulary_reopen":
        return null;
      case "history_clear":
      case "history_remove":
      case "history_reopen":
        return null;
      case "update_capability":
        // A build without an update signing key, which is what every release is
        // until the key pair exists — and the reason the releases page is on the
        // Updates section at all.
        return false;
      case "app_version":
        // The preview is a page, not a build: there is no version behind it.
        return "preview";
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
        // The offline channel is the one entry the app cannot quietly replace
        // with another: it is not in the fallback order, so a pack that was
        // never downloaded has to say so rather than answer with somebody
        // else's translation. The wording matches the backend's.
        if (storedService() === "offline" && !offlinePairReady(input)) {
          throw new Error(
            "The offline translation pack is not downloaded yet. It is on the Resources page of the settings window.",
          );
        }
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
        // The browser preview has no voice; the button is kept lit long enough
        // to be looked at, then the reading ends by itself.
        previewSpeaking = true;
        clearTimeout(previewSpeechTimer);
        previewSpeechTimer = setTimeout(() => {
          previewSpeaking = false;
        }, 2400);
        return true;
      case "stop_speaking":
        previewSpeaking = false;
        clearTimeout(previewSpeechTimer);
        return true;
      case "speaking":
        return previewSpeaking;
      case "current_service":
        return storedService();
      case "service_languages":
        // Stand-in for the backend's table: enough of a difference between the
        // two engines to see the language lists change with them.
        return [
          {
            id: "cloud-baidu",
            languages: ["en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "pt", "it", "ru", "ar"],
          },
          {
            id: "cloud-youdao",
            languages: ["en", "zh-CN", "zh-TW", "ja", "ko", "fr", "de", "es", "pt", "it", "ru", "ar", "nl", "pl", "tr"],
          },
          {
            id: "offline",
            languages: ["en", "zh-CN"],
          },
        ];
      case "set_target_lang":
        settings = { ...settings, targetLang: String(input.code || settings.targetLang) };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "set_service":
        if (!SERVICE_ENGINES[input.id]) throw new Error("unknown service");
        settings = { ...settings, service: input.id };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "capture_status":
        // One entry per slot the backend reports, built from the settings the
        // preview holds: a row whose combination the page has just recorded
        // must read back as active, the way it does in the app. A slot left out
        // makes the page say "no global hotkey" under a box that shows one.
        return {
          hooked: true,
          error: null,
          hotkeys: [
            { slot: "translate", spec: String(settings.hotkey || ""), error: null },
            { slot: "settings", spec: String(settings.hotkeySettings || ""), error: null },
            { slot: "ocr", spec: String(settings.hotkeyOcr || ""), error: null },
          ],
        };
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
      case "ocr_start":
      case "ocr_region":
      case "ocr_cancel":
      case "document_cancel":
        // Nothing is drawn behind a browser tab, so there is no screen to
        // capture; the region picker itself is still usable in the preview.
        return null;
      case "ocr_model_status":
        // The preview reads no picture, so the engine is reported as installed
        // rather than offering a download that would go nowhere.
        return ENGINE_INSTALLED;
      case "ocr_model_download": {
        // The shared files are already on the machine here, so the download is
        // the language alone: it is announced with the size of its own files,
        // which is the number its row counts.
        const wanted =
          ENGINE_INSTALLED.packs.find((entry) => entry.id === input.pack) ||
          ENGINE_INSTALLED.packs[0];
        const packs = ENGINE_INSTALLED.packs.map((entry) => ({
          ...entry,
          downloading: entry.id === wanted.id,
        }));
        const report = (bytes) =>
          previewEmit("glossy://ocr-model", {
            ...ENGINE_INSTALLED,
            packs,
            downloading: true,
            bytes,
            total: wanted.size,
            target: wanted.id,
          });
        report(0);
        report(Math.round(wanted.size / 3));
        return ENGINE_INSTALLED;
      }
      case "ocr_model_remove":
        return { ...ENGINE_INSTALLED, installed: false, size: 0, packs: [] };
      case "offline_model_status":
        // The preview has no models on its disk, so the pack is offered rather
        // than reported ready: the buttons are what the browser can show.
        return previewOffline;
      case "offline_model_download": {
        // One direction, or both when the page names none.
        const wanted = input.pack
          ? previewOffline.pairs.filter((pair) => pair.id === input.pack)
          : previewOffline.pairs;
        const span = wanted.reduce((total, pair) => total + pair.size, 0);
        previewEmit("glossy://offline-model", {
          ...previewOffline,
          downloading: true,
          bytes: Math.round(span / 3),
          total: span,
          target: wanted.length ? wanted[0].id : null,
          pairs: previewOffline.pairs.map((pair) => ({
            ...pair,
            downloading: wanted.some((entry) => entry.id === pair.id),
          })),
        });
        const done = wanted.map((pair) => pair.id);
        previewOffline = {
          ...previewOffline,
          installed: previewOffline.pairs.every(
            (pair) => pair.installed || done.indexOf(pair.id) !== -1
          ),
          size: previewOffline.size + span,
          pairs: previewOffline.pairs.map((pair) => ({
            ...pair,
            installed: pair.installed || done.indexOf(pair.id) !== -1,
          })),
        };
        return previewOffline;
      }
      case "offline_model_remove": {
        const gone = input.pack
          ? previewOffline.pairs.filter((pair) => pair.id === input.pack)
          : previewOffline.pairs;
        const freed = gone.reduce((total, pair) => total + pair.size, 0);
        const left = gone.map((pair) => pair.id);
        previewOffline = {
          ...previewOffline,
          installed: false,
          size: Math.max(0, previewOffline.size - freed),
          pairs: previewOffline.pairs.map((pair) => ({
            ...pair,
            installed: pair.installed && left.indexOf(pair.id) === -1,
          })),
        };
        return previewOffline;
      }
      case "developer_unlock":
        // The preview has no key of its own — the real one is checked in the
        // process and never reaches the window — so an empty field is the one
        // answer it refuses and anything else turns the mode on.
        if (String(input.key || "").trim() === "") throw "dev.key";
        settings = { ...settings, developerMode: true };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "developer_lock":
        settings = { ...settings, developerMode: false };
        previewEmit("glossy://settings", { ...settings });
        return { ...settings };
      case "subtitle_start":
        // The two rectangles are drawn behind a browser tab, so the preview
        // reports the reading as running rather than asking for them.
        previewSubtitle = { running: true };
        return null;
      case "subtitle_stop":
        previewSubtitle = { running: false };
        return null;
      case "subtitle_status":
        return { ...previewSubtitle };
      case "subtitle_edit_start":
      case "subtitle_edit_cancel":
      case "subtitle_edit_apply":
        // Moving the boxes means dragging them over a screen, and a browser tab
        // has none; the editor's own page is still usable in the preview.
        return null;
      case "document_open":
        // The preview splits nothing: the page is being looked at, not the
        // translation, so a picked file is reported as one piece.
        return {
          name: input.name,
          format: String(input.name).split(".").pop().toLowerCase(),
          chars: 120,
          segments: 1,
          sample: "(the preview reads the file but does not translate it)",
        };
      case "document_pick_directory":
        // The preview has no system dialog to open, so it answers with a
        // folder the browser can show.
        return "C:\\Users\\you\\Desktop";
      case "replace_selection":
        // There is nothing to write into behind a browser tab, so the preview
        // only reports that the request arrived.
        return true;
      case "document_start":
        previewEmit("glossy://document", { state: "progress", done: 0, total: 1 });
        previewEmit("glossy://document", {
          state: "done",
          done: 1,
          total: 1,
          preview: "(the preview does not translate anything)",
        });
        return null;
      case "document_save":
        return "C:\\Users\\you\\Documents\\preview.translated.txt";
      case "popup_present":
      case "popup_resize":
        // The real backend reports the usable height of the monitor the popup
        // landed on; in the preview the single browser window is that monitor.
        return Number(window.screen && window.screen.availHeight) || null;
      case "popup_sync_anchor":
      case "popup_close":
      case "popup_set_pinned":
      case "popup_set_replace":
      case "set_window_surface":
      case "show_popup":
      case "open_settings":
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

  /**
   * Draws a line into the subtitle page in the preview.
   *
   * Nothing is read behind a browser tab and the line is the whole of that
   * page, so the preview puts one of its own up to work on the look of it.
   */
  function previewSubtitleLine() {
    if (live || !document.getElementById("line")) return;
    window.addEventListener("load", () => {
      setTimeout(() => {
        previewEmit("glossy://subtitle", {
          text: "This is what a translated subtitle line looks like in the box you picked, shrunk to fit it.",
          error: null,
        });
      }, 300);
    });
  }

  /**
   * Draws two boxes into the box editor in the preview.
   *
   * The editor is opened over a screen that the boxes are moved on, and a
   * browser tab has none: it is given a pair of boxes of its own instead.
   */
  function previewSubtitleBoxes() {
    if (live || !document.getElementById("area")) return;
    window.addEventListener("load", () => {
      setTimeout(() => {
        previewEmit("glossy://subtitle-edit", {
          area: { x: 180, y: 420, width: 520, height: 84 },
          place: { x: 220, y: 300, width: 440, height: 64 },
        });
      }, 300);
    });
  }

  window.Glossy = { live, invoke, listen, readSettings, stripTags };
  window.Glossy.preview = !live;
  previewSubtitleLine();
  previewSubtitleBoxes();
})();
