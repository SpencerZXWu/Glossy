/**
 * Settings window: feature toggle, provider configuration and an in-app
 * playground that translates a text selection without the global hook.
 */
(function (Glossy) {
  const $ = (id) => document.getElementById(id);

  const els = {
    enabled: $("enabled"),
    options: $("options"),
    triggerOnDrag: $("triggerOnDrag"),
    triggerOnDoubleClick: $("triggerOnDoubleClick"),
    restoreClipboard: $("restoreClipboard"),
    showOriginal: $("showOriginal"),
    autostart: $("autostart"),
    minSelectionLen: $("minSelectionLen"),
    hotkey: $("hotkey"),
    hotkeyRecord: $("hotkeyRecord"),
    hotkeyClear: $("hotkeyClear"),
    hotkeyHint: $("hotkeyHint"),
    hotkeySettings: $("hotkeySettings"),
    hotkeySettingsRecord: $("hotkeySettingsRecord"),
    hotkeySettingsClear: $("hotkeySettingsClear"),
    hotkeySettingsHint: $("hotkeySettingsHint"),
    hotkeyOcr: $("hotkeyOcr"),
    hotkeyOcrRecord: $("hotkeyOcrRecord"),
    hotkeyOcrClear: $("hotkeyOcrClear"),
    hotkeyOcrHint: $("hotkeyOcrHint"),
    ocrRun: $("ocrRun"),
    ocrEngineState: $("ocrEngineState"),
    ocrEngineProgress: $("ocrEngineProgress"),
    ocrEngineProgressBar: $("ocrEngineProgressBar"),
    ocrEngineDownload: $("ocrEngineDownload"),
    ocrEngineRemove: $("ocrEngineRemove"),
    ocrPackList: $("ocrPackList"),
    ocrPacksState: $("ocrPacksState"),
    offlineDownload: $("offlineDownload"),
    offlineRemove: $("offlineRemove"),
    offlineState: $("offlineState"),
    offlinePairList: $("offlinePairList"),
    offlineProgress: $("offlineProgress"),
    offlineProgressBar: $("offlineProgressBar"),
    subtitleSourceLang: $("subtitleSourceLang"),
    subtitleTargetLang: $("subtitleTargetLang"),
    subtitlePack: $("subtitlePack"),
    subtitleFontSize: $("subtitleFontSize"),
    subtitleRun: $("subtitleRun"),
    subtitleStop: $("subtitleStop"),
    subtitleAdjust: $("subtitleAdjust"),
    subtitleState: $("subtitleState"),
    devKey: $("devKey"),
    devUnlock: $("devUnlock"),
    devAsk: $("devAsk"),
    devOn: $("devOn"),
    devLock: $("devLock"),
    devState: $("devState"),
    ignoredList: $("ignoredList"),
    ignoredRunning: $("ignoredRunning"),
    ignoredInput: $("ignoredInput"),
    ignoredAdd: $("ignoredAdd"),
    ignoredPick: $("ignoredPick"),
    ignoredHint: $("ignoredHint"),
    sourceLangsList: $("sourceLangsList"),
    sourceLangsAdd: $("sourceLangsAdd"),
    sourceLangsClear: $("sourceLangsClear"),
    sourceLangsHint: $("sourceLangsHint"),
    theme: $("theme"),
    fontScale: $("fontScale"),
    popupWidth: $("popupWidth"),
    popupOpacity: $("popupOpacity"),
    autoCloseSecs: $("autoCloseSecs"),
    closeAfterCopy: $("closeAfterCopy"),
    unitsEnabled: $("unitsEnabled"),
    wordSentence: $("wordSentence"),
    sentencePairs: $("sentencePairs"),
    compactPopup: $("compactPopup"),
    speechRate: $("speechRate"),
    fallbackEnabled: $("fallbackEnabled"),
    fallbackList: $("fallbackList"),
    targetLang: $("targetLang"),
    service: $("service"),
    cloudBlock: $("cloudBlock"),
    cloudQuota: $("cloudQuota"),
    cloudQuotaRefresh: $("cloudQuotaRefresh"),
    cloudQuotaDemo: $("cloudQuotaDemo"),
    cloudQuotaOcr: $("cloudQuotaOcr"),
    confirm: $("confirm"),
    confirmTitle: $("confirmTitle"),
    confirmBody: $("confirmBody"),
    confirmOk: $("confirmOk"),
    confirmCancel: $("confirmCancel"),
    uiLang: $("uiLang"),
    status: $("status"),
    statusText: $("statusText"),
    demoText: $("demoText"),
    demoRun: $("demoRun"),
    demoPaste: $("demoPaste"),
    demoClear: $("demoClear"),
    demoUnits: $("demoUnits"),
    demoCard: $("demoCard"),
    demoHeadword: $("demoHeadword"),
    demoLangbar: $("demoLangbar"),
    demoFrom: $("demoFrom"),
    demoTo: $("demoTo"),
    demoSwap: $("demoSwap"),
    demoPopup: $("demoPopup"),
    demoResult: $("demoResult"),
    selectionHint: $("selectionHint"),
    historyLimit: $("historyLimit"),
    historySearch: $("historySearch"),
    historyList: $("historyList"),
    historyClear: $("historyClear"),
    historyCount: $("historyCount"),
    vocabularySearch: $("vocabularySearch"),
    vocabularyList: $("vocabularyList"),
    vocabularyClear: $("vocabularyClear"),
    vocabularyCount: $("vocabularyCount"),
    vocabularyHide: $("vocabularyHide"),
    settingsExport: $("settingsExport"),
    settingsImport: $("settingsImport"),
    settingsFile: $("settingsFile"),
    logPath: $("logPath"),
    logSize: $("logSize"),
    logOpen: $("logOpen"),
    logExport: $("logExport"),
    logCopy: $("logCopy"),
    logClear: $("logClear"),
    documentFile: $("documentFile"),
    documentDrop: $("documentDrop"),
    documentPick: $("documentPick"),
    documentLangbar: $("documentLangbar"),
    documentFrom: $("documentFrom"),
    documentTo: $("documentTo"),
    documentSwap: $("documentSwap"),
    documentSkipPlain: $("documentSkipPlain"),
    documentAutoSave: $("documentAutoSave"),
    documentSegmentLimit: $("documentSegmentLimit"),
    documentFolder: $("documentFolder"),
    documentFolderChange: $("documentFolderChange"),
    documentRun: $("documentRun"),
    documentCancel: $("documentCancel"),
    documentSave: $("documentSave"),
    documentState: $("documentState"),
    documentSample: $("documentSample"),
    documentProgress: $("documentProgress"),
    documentProgressBar: $("documentProgressBar"),
    documentPreview: $("documentPreview"),
    checkUpdates: $("checkUpdates"),
    updateCheck: $("updateCheck"),
    updateInstall: $("updateInstall"),
    updateStatus: $("updateStatus"),
    updateUnavailable: $("updateUnavailable"),
    updatePage: $("updatePage"),
    appVersion: $("appVersion"),
    toast: $("toast"),
  };

  const LANGUAGES = Glossy.languageCodes;

  /** The sidebar entries, in the order the sidebar lists them. */
  const NAV_ITEMS = Array.from(document.querySelectorAll(".nav-item[data-page]"));
  const PAGES = Array.from(document.querySelectorAll("main .page"));
  /** Which page the window was left on, so it reopens where it was. */
  const PAGE_KEY = "glossy.page";

  /** The lists that are drawn again rather than updated in place. */
  const REDRAWN_LISTS = ["#ocrPackList", "#fallbackList", "#historyList", "#vocabularyList"];

  /** The attributes a control or a row can be found again by after a redraw. */
  const IDENTIFYING = ["pack", "act", "id", "direction", "service"];

  /**
   * A selector for the element that stands in for `element` after a redraw.
   *
   * These pages are built from lists that are thrown away and written again, so
   * what had the keyboard after a save is gone and has to be found by what it
   * says about itself. Only an id or one of the attributes above can name it
   * well enough to be worth trying; the list it sits in comes first, because two
   * lists number their own entries the same way.
   */
  function describe(element) {
    const tag = element && element.tagName ? element.tagName.toLowerCase() : "";
    if (!tag || !element.id && !IDENTIFYING.some((name) => element.dataset && element.dataset[name])) {
      return "";
    }
    const parts = [tag];
    if (element.id) parts.push(`#${element.id}`);
    for (const name of IDENTIFYING) {
      const value = element.dataset ? element.dataset[name] : null;
      if (value) parts.push(`[data-${name}="${value}"]`);
    }
    const classes = String(element.className || "").trim().split(/\s+/).filter(Boolean);
    if (classes.length) parts.push("." + classes.join("."));
    const list = element.closest ? element.closest(REDRAWN_LISTS.join(", ")) : null;
    return (list && list.id ? `#${list.id} ` : "") + parts.join("");
  }

  /**
   * Puts the keyboard back after the page around it was drawn again.
   *
   * The row stands in when the control itself is gone — the button that opened
   * an answer, say, is what the redraw was about — so the reader stays where
   * they were instead of at the top of the document.
   */
  function restoreFocus(selector, row) {
    let again = document.querySelector(selector) || (row ? document.querySelector(row) : null);
    // A control that what it did has disabled — the arrow that pushed a row to
    // the end of the list — cannot take the keyboard back, so the row it belongs
    // to finds it somewhere else to sit.
    if (again && again.disabled && row) {
      const line = document.querySelector(row);
      again = (line && line.querySelector("button:not([disabled])")) || line || again;
    }
    if (again && typeof again.focus === "function") again.focus();
  }

  /**
   * Runs a draw that writes its elements from scratch, keeping the keyboard.
   *
   * Every list on these pages is rebuilt rather than updated in place, so a
   * control that has the focus is thrown away with the rest of its list — by a
   * save, by a download's progress, or by the window taking the focus back. The
   * decision of where the focus goes then belongs here rather than in each
   * caller, and a draw inside another draw is harmless: the inner one has
   * already put the focus back, so the outer one finds nothing to do.
   */
  function redrawing(draw) {
    const had = document.activeElement;
    const focused = had && had !== document.body ? describe(had) : "";
    const row = focused && had.closest ? describe(had.closest("li")) : "";
    draw();
    // The engine that drops a focused element reports it as detached until the
    // next task, so "the focus is gone" is the honest test rather than "the
    // document has it on its body".
    const now = document.activeElement;
    const lost = !now || now === document.body || !document.contains(now);
    if (focused && lost) restoreFocus(focused, row);
  }

  /** What the shared half of the engine is called in the progress it reports. */
  const CORE_TARGET = "core";

  /** The dropdown entries that go through one particular online engine. */
  const CLOUD_VENDORS = { "cloud-baidu": "baidu", "cloud-youdao": "youdao" };

  const POPUP_SIZES = [300, 356, 400, 460, 520];
  const FONT_SCALES = [90, 100, 115, 130, 150];
  const AUTO_CLOSE = [0, 3, 5, 10, 20, 30];
  const OPACITIES = [50, 60, 70, 80, 90, 95, 100];
  const SPEECH_RATES = [-4, 0, 4, 8];
  const HISTORY_LIMITS = [0, 20, 50, 100, 200, 500];

  /** Source value that lets the provider detect the language itself. */
  const AUTO = "auto";

  let settings = null;
  let saveTimer = 0;
  /** True while a change was made but not written to the settings file yet. */
  let pending = false;
  /**
   * Whether the engine and the target language were picked on this screen.
   *
   * The card picks both as well - it switches its own engine and its own
   * language - so a copy written back from a window that was not on screen
   * while that happened would undo the choice. Only a pick that was actually
   * made here is written back; see `collect`.
   */
  let pickedService = false;
  let pickedTarget = false;
  let toastTimer = 0;
  let selection = "";
  /** Programs that never trigger a translation, as shown by the chip list. */
  let ignored = [];
  /** Source languages that still trigger a translation; empty means all. */
  let sourceLangs = [];
  /** Services asked, in order, after the chosen one failed. */
  let fallbackOrder = [];
  /** Number of running programs in the dropdown; -1 while it is being read. */
  let runningApps = -1;
  let lastStatus = null;
  /** Languages the on-machine recogniser reads with, and the last state seen. */
  let ocrPacks = ["ch"];
  let ocrStatus = null;
  /** The last state of the offline translation pack, drawn again when the
      interface language changes: its rows are translated text. */
  let offlineStatus = null;
  /** Whether the pages that are still being built are on show. */
  let developerMode = false;

  /**
   * One row per global accelerator. `name` is the slot the backend reports,
   * `key` the settings key holding the combination.
   */
  const HOTKEYS = [
    {
      name: "translate",
      key: "hotkey",
      active: "hotkey.active",
      input: els.hotkey,
      record: els.hotkeyRecord,
      clear: els.hotkeyClear,
      hint: els.hotkeyHint,
      saved: "",
      recording: false,
    },
    {
      name: "settings",
      key: "hotkeySettings",
      active: "hotkey.active.settings",
      input: els.hotkeySettings,
      record: els.hotkeySettingsRecord,
      clear: els.hotkeySettingsClear,
      hint: els.hotkeySettingsHint,
      saved: "",
      recording: false,
    },
    {
      name: "ocr",
      key: "hotkeyOcr",
      active: "hotkey.active.ocr",
      input: els.hotkeyOcr,
      record: els.hotkeyOcrRecord,
      clear: els.hotkeyOcrClear,
      hint: els.hotkeyOcrHint,
      saved: "",
      recording: false,
    },
  ];

  /** Text the translation card shows; reused when only the languages change. */
  let demoValue = "";
  let demoResult = null;
  /** Pair of the card. `source: AUTO` asks the provider to detect the
      language and `target: null` follows the configured target. */
  let demoPair = { source: AUTO, target: null };
  let demoDetected = "";
  let demoTicket = 0;
  /** Sample text the card box is filled with until the user types something. */
  let sampleText = "";
  /** Translations the backend remembers, newest first. */
  let history = [];
  /** The words and sentences the user kept, newest first. */
  let vocabulary = [];
  /** Kept entries whose translation the self-test has already given away. */
  const revealedWords = new Set();
  /** `"mica"` once the backend reports a backdrop behind the window. */
  let backdrop = "none";

  const COPY_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2.5" /><path d="M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1"/></svg>';
  const REMOVE_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-12"/></svg>';
  /** The star that keeps a translation, filled in by `.history-button` while it
      is on. */
  const STAR_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4.4l2.4 4.9 5.4.8-3.9 3.8.9 5.4-4.8-2.6-4.8 2.6.9-5.4-3.9-3.8 5.4-.8z"/></svg>';

  /**
   * Applies the chosen colour scheme. "system" is resolved against the OS
   * setting by js/theme.js, which is the only place that touches
   * `data-theme`.
   */
  function applyTheme(theme) {
    const next = themeOr(theme);
    GlossyTheme.apply({ theme: next, backdrop });
    // Windows draws the title bar itself, so it needs telling separately.
    if (Glossy.live) {
      Glossy.invoke("set_window_theme", { label: "main", theme: next }).catch((error) => {
        console.warn("Glossy could not restyle its title bar:", error);
      });
    }
  }

  function themeOr(theme) {
    return theme === "light" || theme === "dark" ? theme : "system";
  }

  /**
   * Asks the backend whether a blurred backdrop sits behind the window and
   * remembers the answer for `applyTheme`.
   *
   * The window starts loading before the Rust `setup` hook has laid the effect
   * behind it, so the first answer only says "not decided yet"; the backend is
   * polled until it has an answer, which is normally within one attempt.
   */
  async function resolveBackdrop() {
    if (!Glossy.live) return;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      let info = null;
      try {
        info = await Glossy.invoke("surface_info");
      } catch (error) {
        info = null;
      }
      if (info && info.ready) {
        backdrop = info.backdrop ? "mica" : "none";
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }

  function numberOr(value, fallback) {
    const number = Number(value);
    return Number.isFinite(number) ? number : fallback;
  }

  /** Keeps a value inside the options the markup actually offers. */
  function pick(values, value, fallback) {
    const number = Number(value);
    return values.indexOf(number) === -1 ? fallback : number;
  }

  function setStatus(tone, text, title) {
    els.status.dataset.tone = tone;
    els.statusText.textContent = text;
    els.status.title = title || text;
  }

  function showToast(message) {
    els.toast.textContent = message;
    els.toast.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      els.toast.hidden = true;
    }, 1400);
  }

  /**
   * Asks before something is thrown away, and answers whether it may be.
   *
   * The dialog is the window's own rather than the system's, so it is drawn in
   * the language and the colour scheme the rest of the window is using.
   */
  function confirmAction(title, body, accept) {
    return new Promise((resolve) => {
      els.confirmTitle.textContent = title;
      els.confirmBody.textContent = body;
      els.confirmOk.textContent = accept;
      els.confirmCancel.textContent = Glossy.i18n.t("common.cancel");
      els.confirm.hidden = false;
      // The dialog is modal: whatever held the keyboard before it is what gets
      // it back, because the button that opened the dialog may be gone by the
      // time it is answered.
      const returnTo = document.activeElement;
      els.confirmOk.focus();

      const settle = (answer) => {
        els.confirm.hidden = true;
        els.confirmOk.removeEventListener("click", yes);
        els.confirmCancel.removeEventListener("click", no);
        document.removeEventListener("keydown", onKey, true);
        if (returnTo && document.contains(returnTo)) returnTo.focus();
        resolve(answer);
      };
      const yes = () => settle(true);
      const no = () => settle(false);
      const onKey = (event) => {
        if (event.key === "Escape") {
          event.stopPropagation();
          settle(false);
          return;
        }
        if (event.key !== "Tab") return;
        // Tab and Shift+Tab walk the two buttons and nothing behind them: a
        // dialog that says it is modal has to hold the keyboard while it is up.
        const stops = [els.confirmCancel, els.confirmOk];
        const here = stops.indexOf(document.activeElement);
        const step = event.shiftKey ? -1 : 1;
        event.preventDefault();
        stops[(here + step + stops.length) % stops.length].focus();
      };
      els.confirmOk.addEventListener("click", yes);
      els.confirmCancel.addEventListener("click", no);
      document.addEventListener("keydown", onKey, true);
    });
  }

  /**
   * Rebuilds the target list from the languages the chosen engine translates,
   * and answers with the code it settled on: what was wanted while the engine
   * takes it, and the first language it does take otherwise. A code the shared
   * list never had is kept, because it did not come from this menu.
   */
  function fillLanguages(current) {
    const codes = Glossy.languagesFor(els.service.value);
    if (current && codes.indexOf(current) === -1 && LANGUAGES.indexOf(current) === -1) {
      codes.unshift(current);
    }
    els.targetLang.innerHTML = "";
    codes.forEach((code) => {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = Glossy.languageName(code);
      els.targetLang.appendChild(option);
    });
    return current && codes.indexOf(current) !== -1 ? current : codes[0];
  }

  /** Whether the dropdown entry is served by Glossy's own server. */
  function isCloudService(service) {
    return Object.prototype.hasOwnProperty.call(CLOUD_VENDORS, service);
  }

  /**
   * Shows the fields that belong to the chosen service.
   *
   * The dropdown is one list and the entries are names and nothing else: what a
   * service needs is shown by the fields under it, and only the engines that go
   * through Glossy's server have anything to show.
   */
  function syncService() {
    const service = els.service.value;
    els.cloudBlock.hidden = !isCloudService(service);
    // The service chosen above is always asked first, so it cannot also sit in
    // the list below: a copy of it there would show the same engine twice and
    // leave another one out of the list entirely. Changing the first entry
    // therefore changes the list, and the change is saved like any other.
    const fixed = fallbackFor(service);
    if (fixed.join() !== fallbackOrder.join()) {
      fallbackOrder = fixed;
      scheduleSave();
    }
    renderFallbackOrder();
  }

  /** The dropdown value that matches what the settings file holds. */
  function serviceOf(stored) {
    const wanted = String(stored.service || "");
    return SERVICES.indexOf(wanted) === -1 ? "cloud-baidu" : wanted;
  }

  /**
   * Every page that shows today's allowance.
   *
   * The allowance belongs to the server rather than to a page, so the same line
   * is written on all three at once — the Language page it has always been on,
   * and the two pages a translation is actually started from, which is where a
   * reader wants to see what is left before spending it.
   */
  function quotaLines() {
    return [els.cloudQuota, els.cloudQuotaDemo, els.cloudQuotaOcr].filter(Boolean);
  }

  /** Writes one line of text on every page that shows the allowance. */
  function showQuota(text, tone) {
    quotaLines().forEach((line) => {
      if (tone) line.setAttribute("data-tone", tone);
      else line.removeAttribute("data-tone");
      line.textContent = text;
    });
  }

  /** Fetches the allowance again whenever the shared server comes into view. */
  function syncChannelQuota() {
    if (cloudQuotaVisible()) refreshCloudQuota();
    else showQuota("");
  }

  /**
   * Shows what the cloud provider still allows today.
   *
   * The allowance lives on the server, so it is asked for whenever the window
   * opens or another service is chosen; a server that cannot be reached says so
   * in the same line instead of blocking the rest of the window. Which server is
   * asked is decided by the backend, which carries the address it was built with.
   */
  async function refreshCloudQuota() {
    showQuota(Glossy.i18n.t("cloud.quota.checking"));
    try {
      const quota = await Glossy.invoke("cloud_status", {});
      if (!cloudQuotaVisible()) return;
      const used = quota.used || 0;
      const limit = quota.limit || 0;
      const remaining = quota.remaining === undefined ? Math.max(0, limit - used) : quota.remaining;
      if (remaining > 0) {
        showQuota(
          Glossy.i18n.t("cloud.quota.remaining", remaining.toLocaleString(), limit.toLocaleString())
        );
      } else {
        showQuota(Glossy.i18n.t("cloud.quota.used", limit.toLocaleString()), "bad");
      }
    } catch (error) {
      if (!cloudQuotaVisible()) return;
      showQuota(Glossy.errorMessage(error), "bad");
    }
  }

  /** Whether the shared server — and therefore its allowance — is on screen. */
  function cloudQuotaVisible() {
    return isCloudService(els.service.value);
  }

  /** Whether the page that shows the text recognition engine is on screen. */
  function ocrEngineVisible() {
    const page = document.getElementById("page-resources");
    return Boolean(page) && !page.hidden;
  }

  /**
   * Shows what the offline translation pack is doing, and offers the two things
   * the user can do about it: download it, or take it off the disk again.
   *
   * The pack is one row like the recognition engine is, so the whole of it is
   * announced as one download: the runtime, the piece tables, and a model for
   * each direction.
   */
  async function refreshOfflinePack() {
    if (!ocrEngineVisible()) return;
    els.offlineState.removeAttribute("data-tone");
    try {
      renderOfflinePack(await Glossy.invoke("offline_model_status", {}));
    } catch (error) {
      if (!ocrEngineVisible()) return;
      els.offlineState.setAttribute("data-tone", "bad");
      els.offlineState.textContent = Glossy.errorMessage(error);
    }
  }

  /** One button of a direction row: the way to download it, or to delete it. */
  function pairButton(act, id) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "ghost pack-action";
    button.dataset.act = act;
    button.dataset.pair = id;
    button.textContent = Glossy.i18n.t(
      act === "download" ? "resources.packs.download" : "resources.packs.remove"
    );
    return button;
  }

  /** Draws the pack and each direction it holds. */
  function renderOfflinePack(status) {
    if (!status) return;
    offlineStatus = status;
    const inst = Glossy.i18n.t.bind(Glossy.i18n);
    els.offlineDownload.hidden = status.installed || status.downloading;
    els.offlineRemove.hidden = !status.size;
    els.offlineProgress.hidden = !status.downloading;
    els.offlineProgressBar.style.width = status.downloading
      ? percent(status.bytes, status.total)
      : "0%";
    els.offlineState.removeAttribute("data-tone");

    if (status.downloading) {
      els.offlineState.textContent = inst(
        "resources.offline.downloading",
        megabytes(status.bytes),
        megabytes(status.total)
      );
    } else if (status.error) {
      els.offlineState.setAttribute("data-tone", "bad");
      els.offlineState.textContent = Glossy.errorMessage(status.error);
    } else if (status.installed) {
      els.offlineState.setAttribute("data-tone", "ok");
      els.offlineState.textContent = inst(
        "resources.offline.ready",
        wholeMegabytes(status.size)
      );
    } else if (status.size) {
      els.offlineState.textContent = inst(
        "resources.offline.partial",
        wholeMegabytes(status.size),
        wholeMegabytes(status.full)
      );
    } else {
      els.offlineState.textContent = inst(
        "resources.offline.missing",
        wholeMegabytes(status.full)
      );
    }

    els.offlinePairList.innerHTML = "";
    (status.pairs || []).forEach((pair) => {
      const item = document.createElement("li");
      item.className = "pack";
      item.dataset.pair = pair.id;
      const busy = !!pair.downloading && status.downloading;
      if (busy) item.dataset.state = "downloading";
      else if (pair.installed) item.dataset.state = "installed";

      const name = document.createElement("span");
      name.className = "pack-name";
      name.textContent = inst(`offline.pair.${pair.id}`);

      const state = document.createElement("span");
      state.className = "pack-state";
      state.textContent = busy
        ? inst(
            "resources.offline.downloading",
            megabytes(status.bytes),
            megabytes(status.total)
          )
        : pair.installed
          ? inst("resources.packs.installed", wholeMegabytes(pair.size))
          : inst("resources.packs.missing", wholeMegabytes(pair.size));

      const actions = document.createElement("span");
      actions.className = "pack-actions";
      if (busy) {
        const bar = document.createElement("span");
        bar.className = "progress pack-progress";
        bar.setAttribute("aria-hidden", "true");
        const fill = document.createElement("span");
        fill.className = "progress-bar";
        fill.style.width = percent(status.bytes, status.total);
        bar.appendChild(fill);
        actions.appendChild(bar);
      } else if (pair.installed) {
        actions.appendChild(pairButton("remove", pair.id));
      } else {
        actions.appendChild(pairButton("download", pair.id));
      }

      item.append(name, state, actions);
      els.offlinePairList.appendChild(item);
    });
  }

  /** Downloads the pack — one direction, or both when none is named. */
  async function downloadOfflinePack(pack) {
    if (!pack) {
      els.offlineDownload.hidden = true;
      els.offlineState.removeAttribute("data-tone");
      els.offlineState.textContent = Glossy.i18n.t("resources.offline.starting");
      els.offlineProgress.hidden = false;
    }
    try {
      renderOfflinePack(await Glossy.invoke("offline_model_download", pack ? { pack } : {}));
    } catch (error) {
      els.offlineProgress.hidden = true;
      els.offlineState.setAttribute("data-tone", "bad");
      els.offlineState.textContent = Glossy.errorMessage(error);
      els.offlineDownload.hidden = false;
    }
  }

  /** Takes one direction off the disk, or the whole pack, after asking. */
  async function removeOfflinePack(pack) {
    const agreed = await confirmAction(
      Glossy.i18n.t("resources.offline.remove"),
      Glossy.i18n.t(pack ? "resources.offline.removePairConfirm" : "resources.offline.removeConfirm"),
      Glossy.i18n.t("resources.offline.remove")
    );
    if (!agreed) return;
    try {
      renderOfflinePack(await Glossy.invoke("offline_model_remove", pack ? { pack } : {}));
    } catch (error) {
      els.offlineState.setAttribute("data-tone", "bad");
      els.offlineState.textContent = Glossy.errorMessage(error);
    }
  }

  /**
   * Shows what the text recognition engine is doing, and offers the two things
   * the user can do about it: download it, or take it off the disk again.
   *
   * The state is asked for when the page comes up and again whenever the window
   * is brought back to the front, because a download started from the first-use
   * prompt in the popup would otherwise leave a stale line behind. While a
   * download runs, the progress is drawn from the events the backend emits.
   */
  async function refreshOcrEngine() {
    if (!ocrEngineVisible()) return;
    els.ocrEngineState.removeAttribute("data-tone");
    try {
      renderOcrEngine(await Glossy.invoke("ocr_model_status", {}));
    } catch (error) {
      if (!ocrEngineVisible()) return;
      els.ocrEngineState.setAttribute("data-tone", "bad");
      els.ocrEngineState.textContent = Glossy.errorMessage(error);
    }
  }

  /** One decimal of a megabyte, for a size that is moving. */
  function megabytes(bytes) {
    return (Number(bytes || 0) / (1024 * 1024)).toFixed(1);
  }

  /** Whole megabytes, for a size that is not. */
  function wholeMegabytes(bytes) {
    return String(Math.round(Number(bytes || 0) / (1024 * 1024)));
  }

  /** What a pack is called in the interface, or its stored name if unnamed. */
  function packName(id) {
    const key = `pack.${id}`;
    const name = Glossy.i18n.t(key);
    return name === key ? id : name;
  }

  /** How far a download has come, as the width of a bar. */
  function percent(done, total) {
    const span = Number(total || 0);
    if (!(span > 0)) return "0%";
    return Math.min(100, (Number(done || 0) / span) * 100) + "%";
  }

  /** Draws one state of the engine and of every language it could read. */
  function renderOcrEngine(status) {
    redrawing(() => drawOcrEngine(status));
  }

  function drawOcrEngine(status) {
    if (!status) return;
    ocrStatus = status;
    const inst = Glossy.i18n.t.bind(Glossy.i18n);
    // The engine row is about the runtime and the detector alone, which is what
    // `core` names; a language's download says what it is on that language's own
    // row. Neither ever shows a number that mixes the two.
    const shared = status.downloading && status.target === CORE_TARGET;
    els.ocrEngineDownload.hidden = status.installed || status.downloading;
    els.ocrEngineRemove.hidden = !status.installed;
    els.ocrEngineState.removeAttribute("data-tone");
    els.ocrEngineProgress.hidden = !shared;
    els.ocrEngineProgressBar.style.width = shared ? percent(status.bytes, status.total) : "0%";

    if (shared) {
      els.ocrEngineState.textContent = inst(
        "ocr.engine.downloading",
        megabytes(status.bytes),
        megabytes(status.total)
      );
    } else if (status.error && status.target === CORE_TARGET) {
      els.ocrEngineState.setAttribute("data-tone", "bad");
      els.ocrEngineState.textContent = Glossy.errorMessage(status.error);
    } else if (status.installed) {
      els.ocrEngineState.setAttribute("data-tone", "ok");
      els.ocrEngineState.textContent = inst("ocr.engine.ready", wholeMegabytes(status.coreSize));
    } else {
      els.ocrEngineState.textContent = inst("ocr.engine.missing");
    }
    renderPacks(status);
  }

  /** Draws the language packs, and which ones the next screenshot is read with. */
  function renderPacks(status) {
    const inst = Glossy.i18n.t.bind(Glossy.i18n);
    const packs = status.packs || [];
    // The languages the settings checked, which is the list the backend reads
    // with; the ones this build does not offer are not drawn at all.
    const checked = ocrPacks.filter((id) => packs.some((pack) => pack.id === id));
    els.ocrPackList.innerHTML = "";

    packs.forEach((pack) => {
      const item = document.createElement("li");
      item.className = "pack";
      item.dataset.pack = pack.id;
      const downloading = pack.downloading && status.downloading;
      const failed = !status.downloading && !!status.error && status.target === pack.id;
      if (downloading) item.dataset.state = "downloading";
      else if (pack.installed) item.dataset.state = "installed";

      const isChecked = checked.indexOf(pack.id) !== -1;
      const pick = document.createElement("input");
      pick.type = "checkbox";
      pick.className = "pack-pick";
      pick.id = `ocrPack-${pack.id}`;
      pick.value = pack.id;
      pick.checked = isChecked;
      // A language that is not on the disk cannot be read with, so it cannot be
      // picked either: the button beside it is the way to it. The last one that
      // is left cannot be turned off either, because a screenshot has to be read
      // with something.
      const last = isChecked && checked.length === 1;
      pick.disabled = !pack.installed || last;
      pick.title = last ? inst("resources.packs.lastOne") : inst("resources.packs.pick");
      pick.addEventListener("change", () => {
        togglePack(pack.id, pick.checked);
      });

      const name = document.createElement("label");
      name.className = "pack-name";
      name.htmlFor = pick.id;
      name.textContent = packName(pack.id);

      // What the row says about itself: how big it is, how far its own download
      // has come, or why that download failed.
      const state = document.createElement("span");
      state.className = "pack-state";
      if (downloading) {
        state.textContent = inst(
          "resources.packs.progress",
          megabytes(status.bytes),
          megabytes(status.total)
        );
      } else if (failed) {
        state.setAttribute("data-tone", "bad");
        state.textContent = Glossy.errorMessage(status.error);
      } else {
        state.textContent = pack.installed
          ? inst("resources.packs.installed", wholeMegabytes(pack.size))
          : inst("resources.packs.missing", wholeMegabytes(pack.size));
      }

      const actions = document.createElement("span");
      actions.className = "pack-actions";
      // A row that is downloading offers nothing but its own progress, and the
      // rows that are still missing wait for it: one download at a time is what
      // the backend does, and a button that could only be refused is worse than
      // no button.
      if (downloading) {
        const bar = document.createElement("span");
        bar.className = "progress pack-progress";
        bar.setAttribute("aria-hidden", "true");
        const fill = document.createElement("span");
        fill.className = "progress-bar";
        fill.style.width = percent(status.bytes, status.total);
        bar.appendChild(fill);
        actions.appendChild(bar);
      } else if (pack.installed) {
        actions.appendChild(packButton("remove", pack.id));
        // The languages the next screenshot is read with say so on their own
        // row, which is what tells a checked language that is not installed
        // apart from one that is read with.
        if (isChecked) {
          const use = document.createElement("span");
          use.className = "pack-busy";
          use.setAttribute("data-tone", "ok");
          use.textContent = inst("resources.packs.use");
          actions.appendChild(use);
        }
      } else if (!status.downloading) {
        actions.appendChild(packButton("download", pack.id));
      }

      item.append(pick, name, state, actions);
      els.ocrPackList.appendChild(item);
    });

    const installed = packs.filter((pack) => pack.installed).length;
    els.ocrPacksState.textContent = packs.length
      ? inst(
          "resources.packs.total",
          String(installed),
          String(packs.length),
          wholeMegabytes((status.size || 0) - (status.coreSize || 0))
        )
      : "";
  }

  /** One button of a pack row: the way to download it, or to delete it. */
  function packButton(act, id) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "ghost pack-action";
    button.dataset.act = act;
    button.dataset.pack = id;
    button.textContent = Glossy.i18n.t(
      act === "download" ? "resources.packs.download" : "resources.packs.remove"
    );
    return button;
  }

  /** Downloads the engine, keeping the buttons honest while it runs. */
  async function downloadOcrEngine() {
    els.ocrEngineDownload.hidden = true;
    els.ocrEngineState.removeAttribute("data-tone");
    els.ocrEngineState.textContent = Glossy.i18n.t("ocr.engine.starting");
    els.ocrEngineProgress.hidden = false;
    try {
      renderOcrEngine(await Glossy.invoke("ocr_model_download", {}));
    } catch (error) {
      els.ocrEngineProgress.hidden = true;
      els.ocrEngineState.setAttribute("data-tone", "bad");
      els.ocrEngineState.textContent = Glossy.errorMessage(error);
      els.ocrEngineDownload.hidden = false;
    }
  }

  /** Downloads one language pack, along with the shared files if they are gone. */
  async function downloadPack(id) {
    try {
      renderOcrEngine(await Glossy.invoke("ocr_model_download", { pack: id }));
    } catch (error) {
      showPackError(id, Glossy.errorMessage(error));
    }
  }

  /**
   * Puts a failure on the row of the language it belongs to.
   *
   * Nothing about the shared files went wrong, so the reason does not belong
   * under them: the row is drawn again and the message is written on it.
   */
  function showPackError(id, message) {
    renderOcrEngine(ocrStatus);
    const row = els.ocrPackList.querySelector(`.pack[data-pack="${id}"]`);
    const state = row ? row.querySelector(".pack-state") : null;
    if (!state) return;
    state.setAttribute("data-tone", "bad");
    state.textContent = message;
  }

  /**
   * Takes the engine off the disk, after asking.
   *
   * Everything goes: the runtime, the detector and every language. One language
   * on its own is deleted from its own row.
   */
  async function removeOcrEngine() {
    const agreed = await confirmAction(
      Glossy.i18n.t("ocr.engine.remove"),
      Glossy.i18n.t("ocr.engine.removeConfirm"),
      Glossy.i18n.t("ocr.engine.remove")
    );
    if (!agreed) return;
    try {
      renderOcrEngine(await Glossy.invoke("ocr_model_remove", {}));
    } catch (error) {
      els.ocrEngineState.setAttribute("data-tone", "bad");
      els.ocrEngineState.textContent = Glossy.errorMessage(error);
    }
  }

  /** Deletes one language pack, after asking. */
  async function removePack(id) {
    const agreed = await confirmAction(
      packName(id),
      Glossy.i18n.t("resources.packs.removeConfirm"),
      Glossy.i18n.t("resources.packs.remove")
    );
    if (!agreed) return;
    try {
      const status = await Glossy.invoke("ocr_model_remove", { pack: id });
      // A language that was read with just left: the checked list keeps the
      // ones that are still on the disk — the rest could not be read with
      // anyway — and when nothing is left the language the app has always read
      // with takes over, so a screenshot is never left with no language at all.
      if (ocrPacks.indexOf(id) !== -1) {
        const installed = (status.packs || [])
          .filter((pack) => pack.installed)
          .map((pack) => pack.id);
        const rest = ocrPacks.filter((other) => installed.indexOf(other) !== -1);
        ocrPacks = rest.length ? rest : [installed.length ? installed[0] : "ch"];
        scheduleSave();
      }
      renderOcrEngine(status);
    } catch (error) {
      showPackError(id, Glossy.errorMessage(error));
    }
  }

  /**
   * The languages that are read with, as the settings hold them: blanks and
   * repeats dropped, and a list that is left with nothing set to the language
   * the app has always read with.
   *
   * The backend puts whatever it reads through the same shape, so a hand-written
   * settings file and a list that was drawn before a language was downloaded
   * both end up at the same place.
   */
  function normalizePacks(list) {
    const seen = Object.create(null);
    const kept = [];
    (Array.isArray(list) ? list : [list]).forEach((value) => {
      const id = String(value || "")
        .trim()
        .toLowerCase();
      if (!id || seen[id]) return;
      seen[id] = true;
      kept.push(id);
    });
    return kept.length ? kept : ["ch"];
  }

  /** Checks or unchecks a language the next screenshot is read with. */
  function togglePack(id, on) {
    const others = ocrPacks.filter((other) => other !== id);
    ocrPacks = on ? others.concat(id) : others;
    if (!ocrPacks.length) ocrPacks = [id];
    scheduleSave();
    // The row drawn again is what moves the button that cannot be turned off,
    // and what fills the checked set in again: the list is the settings, so it
    // is drawn from what was just changed rather than from the last state the
    // backend reported.
    if (ocrStatus) renderOcrEngine(ocrStatus);
    if (ocrStatus) fillSubtitlePacks(ocrStatus, els.subtitlePack.value);
  }

  /* ---- The subtitles, and the pages that hide behind developer mode ------ */

  /** How long a reading that was asked for is given to report itself running.
      The overlay of the picker closes itself after half a minute, so a run that
      was never started is over by then too. */
  const SUBTITLE_ASK = 33000;

  /** Whether a reading is running as far as this window knows, and since when
      one was asked for — the two rectangles are picked before it starts. */
  let subtitleRunning = false;
  let subtitleAskedAt = 0;
  /** The timer that follows the reading while its page is the one on show. */
  let subtitleWatch = 0;

  /**
   * Shows or hides the pages that are still being built.
   *
   * The key is checked in the backend and only the stored settings say whether
   * it was accepted, so this follows them rather than the field. What turns on
   * and off with the mode is the sidebar's own entry, and a subtitle page that
   * is open when the mode goes away is left behind with it.
   */
  function applyDeveloperMode() {
    const item = NAV_ITEMS.find((entry) => entry.dataset.page === "subtitle");
    if (item) item.hidden = !developerMode;
    els.devAsk.hidden = developerMode;
    els.devOn.hidden = !developerMode;
    els.devState.textContent = "";
    els.devState.removeAttribute("data-tone");
    const page = document.getElementById("page-subtitle");
    if (!developerMode && page && !page.hidden) showPage("translate");
  }

  /** Writes the subtitle settings into the page. */
  function fillSubtitleFields(next) {
    fillLanguageSelect(els.subtitleSourceLang, next.subtitleSourceLang, true);
    // The source bar says "detect it" without the name the demo found: nothing
    // has been read here yet.
    els.subtitleSourceLang.options[0].textContent = Glossy.i18n.t("popup.autoDetected");
    fillLanguageSelect(els.subtitleTargetLang, next.subtitleTargetLang, false);
    fillSubtitlePacks(ocrStatus, next.subtitlePack);
    const size = Math.min(40, Math.max(13, Number(next.subtitleFontSize || 26)));
    els.subtitleFontSize.value = String(size);
    drawSubtitleState();
  }

  /**
   * Offers the languages the region can be read with.
   *
   * Only the ones on the disk are offered: a reading runs every second, so a
   * language that would have to be downloaded first cannot be picked here — the
   * Resources page is the way to it. The one the settings name is kept even
   * when it is not there, because that is what a run would fall back from.
   */
  function fillSubtitlePacks(status, current) {
    const packs = (status && status.packs) || [];
    const installed = packs.filter((pack) => pack.installed).map((pack) => pack.id);
    const codes = installed.length ? installed.slice() : packs.map((pack) => pack.id);
    if (current && codes.indexOf(current) === -1) codes.unshift(current);

    els.subtitlePack.innerHTML = "";
    codes.forEach((id) => {
      const option = document.createElement("option");
      option.value = id;
      option.textContent = packName(id);
      els.subtitlePack.appendChild(option);
    });
    if (current && codes.indexOf(current) !== -1) els.subtitlePack.value = current;
  }

  /** Draws what the reading is doing on the page it was started from. */
  function drawSubtitleState() {
    const asked = subtitleAskedAt > 0 && Date.now() - subtitleAskedAt < SUBTITLE_ASK;
    if (subtitleRunning) subtitleAskedAt = 0;
    // One button at a time: a reading that is running, or being set up, is
    // stopped from here rather than started over again.
    els.subtitleRun.hidden = subtitleRunning || asked;
    els.subtitleStop.hidden = !subtitleRunning && !asked;
    els.subtitleState.removeAttribute("data-tone");
    if (subtitleRunning) els.subtitleState.setAttribute("data-tone", "ok");
    els.subtitleState.textContent = Glossy.i18n.t(
      subtitleRunning
        ? "subtitle.state.running"
        : asked
          ? "subtitle.state.starting"
          : "subtitle.state.stopped"
    );
  }

  /** Asks the backend whether a reading is running, and shows the answer. */
  async function refreshSubtitle() {
    try {
      const status = await Glossy.invoke("subtitle_status");
      subtitleRunning = !!(status && status.running);
    } catch (error) {
      // A status that could not be read is not a reading that stopped; the next
      // tick asks again.
      return;
    }
    drawSubtitleState();
  }

  /**
   * Follows the reading while its own page is the one on show.
   *
   * The reading is started by two picks and stopped from here, so the page has
   * no single answer to go on: it asks until the page is left, which is also
   * what tells a stopped reading from one that is still being set up.
   */
  function watchSubtitle(on) {
    if (on && !subtitleWatch) {
      refreshSubtitle();
      subtitleWatch = setInterval(refreshSubtitle, 1000);
    } else if (!on && subtitleWatch) {
      clearInterval(subtitleWatch);
      subtitleWatch = 0;
    }
  }

  /** Drops blanks and duplicates, ignoring a trailing `.exe`. */
  function normalizeIgnored(list) {
    const seen = Object.create(null);
    const result = [];
    (Array.isArray(list) ? list : [list]).forEach((entry) => {
      String(entry || "")
        .split(/[,;\s]+/)
        .forEach((part) => {
          const name = part.trim();
          if (!name) return;
          const key = name.replace(/\.exe$/i, "").toLowerCase();
          if (seen[key]) return;
          seen[key] = true;
          result.push(name);
        });
    });
    return result;
  }

  function addIgnored(name) {
    const next = normalizeIgnored([name]);
    if (!next.length) return false;
    const key = next[0].replace(/\.exe$/i, "").toLowerCase();
    const known = ignored.some((entry) => entry.replace(/\.exe$/i, "").toLowerCase() === key);
    if (known) return false;
    ignored = ignored.concat(next[0]);
    return true;
  }

  function renderIgnored() {
    els.ignoredList.innerHTML = "";
    ignored.forEach((name) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.setAttribute("role", "listitem");
      chip.appendChild(document.createTextNode(name));
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "chip-remove";
      remove.textContent = "×";
      remove.title = Glossy.i18n.t("ignored.remove");
      remove.setAttribute("aria-label", Glossy.i18n.t("ignored.remove"));
      remove.addEventListener("click", () => {
        ignored = ignored.filter((entry) => entry !== name);
        renderIgnored();
        scheduleSave();
      });
      chip.appendChild(remove);
      els.ignoredList.appendChild(chip);
    });
    els.ignoredHint.textContent = ignored.length
      ? Glossy.i18n.plural("ignored.count", ignored.length)
      : Glossy.i18n.t("ignored.empty");
  }

  /** Fills the dropdown with the languages that can still be added. */
  function fillSourceLangPicker() {
    els.sourceLangsAdd.innerHTML = "";
    els.sourceLangsAdd.appendChild(new Option(Glossy.i18n.t("source.add"), ""));
    LANGUAGES.filter((code) => !isSourceLang(code)).forEach((code) => {
      els.sourceLangsAdd.appendChild(new Option(Glossy.languageName(code), code));
    });
  }

  /** Drops blanks and duplicates from the source-language list. */
  function normalizeSourceLangs(list) {
    const seen = Object.create(null);
    const result = [];
    (Array.isArray(list) ? list : [list]).forEach((entry) => {
      String(entry || "")
        .split(/[,;\s]+/)
        .forEach((part) => {
          const code = part.trim();
          if (!code) return;
          const key = code.toLowerCase();
          if (seen[key]) return;
          seen[key] = true;
          result.push(code);
        });
    });
    return result;
  }

  function isSourceLang(code) {
    return sourceLangs.some((entry) => entry.toLowerCase() === code.toLowerCase());
  }

  function addSourceLang(code) {
    const next = normalizeSourceLangs([code]);
    if (!next.length || isSourceLang(next[0])) return false;
    sourceLangs = sourceLangs.concat(next[0]);
    return true;
  }

  function renderSourceLangs() {
    els.sourceLangsList.innerHTML = "";
    sourceLangs.forEach((code) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.setAttribute("role", "listitem");
      chip.appendChild(document.createTextNode(Glossy.languageName(code)));
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "chip-remove";
      remove.textContent = "×";
      remove.title = Glossy.i18n.t("source.remove");
      remove.setAttribute("aria-label", Glossy.i18n.t("source.remove"));
      remove.addEventListener("click", () => {
        sourceLangs = sourceLangs.filter((entry) => entry !== code);
        renderSourceLangs();
        scheduleSave();
      });
      chip.appendChild(remove);
      els.sourceLangsList.appendChild(chip);
    });
    els.sourceLangsClear.disabled = !sourceLangs.length;
    els.sourceLangsHint.textContent = sourceLangs.length
      ? Glossy.i18n.plural("source.count", sourceLangs.length)
      : Glossy.i18n.t("source.empty");
    fillSourceLangPicker();
  }

  /** The services of the dropdown, in the order it offers them. */
  const SERVICES = ["cloud-baidu", "cloud-youdao", "google", "offline"];

  /**
   * The services the list under the chosen one may hold.
   *
   * `offline` is left out of it: it is a choice the reader makes rather than a
   * substitution the app makes for them, so it is never offered as a fallback
   * and a file that names it there has it dropped on the next save — which is
   * what the backend's `service_order` is written to expect.
   */
  const FALLBACK_SERVICES = SERVICES.filter((id) => id !== "offline");

  /** Keeps every service once, and drops anything this build does not offer. */
  function normalizeFallbackOrder(list) {
    const seen = Object.create(null);
    return (Array.isArray(list) ? list : []).filter((service) => {
      if (FALLBACK_SERVICES.indexOf(service) === -1 || seen[service]) return false;
      seen[service] = true;
      return true;
    });
  }

  /**
   * The list under the chosen service: every other engine, once, in the order
   * the user put them in.
   *
   * The chosen service is asked first and cannot be moved, so a copy of it in the
   * list below would be a second attempt on the same backend and would push one
   * engine out of the list altogether — which is exactly what happened when the
   * first entry was changed. An engine the list does not have yet lands at the
   * end, where a newly added one belongs.
   */
  function fallbackFor(service) {
    const known = normalizeFallbackOrder(fallbackOrder);
    const others = FALLBACK_SERVICES.filter((id) => id !== service);
    return others
      .filter((id) => known.indexOf(id) !== -1)
      .concat(others.filter((id) => known.indexOf(id) === -1));
  }

  /** Short name of one service, the way the order list spells it. */
  function serviceLabel(service) {
    return Glossy.i18n.t(`service.${service}`);
  }

  /**
   * Draws the fallback order: the service chosen above always comes first and
   * cannot be moved, then the ones asked after it, each with the two buttons
   * that move it up or down.
   */
  function renderFallbackOrder() {
    redrawing(drawFallbackOrder);
  }

  function drawFallbackOrder() {
    els.fallbackList.innerHTML = "";
    els.fallbackList.dataset.disabled = String(!els.fallbackEnabled.checked);

    const chosen = document.createElement("li");
    chosen.className = "order-row chosen";
    chosen.setAttribute("role", "listitem");
    chosen.appendChild(serviceName(serviceLabel(els.service.value), "order-name"));
    chosen.appendChild(serviceName(Glossy.i18n.t("fallback.first"), "order-fixed"));
    els.fallbackList.appendChild(chosen);

    fallbackOrder.forEach((service, index) => {
      const row = document.createElement("li");
      row.className = "order-row";
      row.setAttribute("role", "listitem");
      // Which service the row is, so the arrow that moved it can be found again
      // — along with the row it belongs to when that arrow is the one the move
      // has just disabled.
      row.dataset.service = service;
      row.appendChild(serviceName(serviceLabel(service), "order-name"));
      row.appendChild(orderMove("up", index, service, index === 0));
      row.appendChild(orderMove("down", index, service, index === fallbackOrder.length - 1));
      els.fallbackList.appendChild(row);
    });
  }

  /** The text holder of one row of the order list. */
  function serviceName(text, className) {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = text;
    return span;
  }

  /** One of the two arrows that move an entry of the order list. */
  function orderMove(direction, index, service, disabled) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "order-move";
    button.dataset.direction = direction;
    // Which row the arrow belongs to, so the keyboard can find it again after
    // the list is drawn again — and the arrows of one row are two.
    button.dataset.service = service;
    const label = Glossy.i18n.t(direction === "up" ? "fallback.up" : "fallback.down");
    button.title = label;
    button.setAttribute("aria-label", label);
    button.textContent = direction === "up" ? "↑" : "↓";
    button.disabled = disabled || !els.fallbackEnabled.checked;
    button.addEventListener("click", () => moveFallback(index, direction === "up" ? -1 : 1));
    return button;
  }

  /** Moves one entry of the order list and saves the new order. */
  function moveFallback(index, step) {
    const target = index + step;
    if (target < 0 || target >= fallbackOrder.length) return;
    const next = fallbackOrder.slice();
    const moved = next.splice(index, 1)[0];
    next.splice(target, 0, moved);
    fallbackOrder = next;
    renderFallbackOrder();
    scheduleSave();
  }

  /** Reads the history the backend keeps and shows it. */
  async function loadHistory() {
    try {
      history = (await Glossy.invoke("history_list")) || [];
    } catch (error) {
      history = [];
    }
    renderHistory();
  }

  /** The entries that match what the search box holds. */
  function matchingHistory() {
    const needle = els.historySearch.value.trim().toLowerCase();
    if (!needle) return history;
    return history.filter((entry) => {
      const result = entry.result || {};
      return (
        String(result.sourceText || "").toLowerCase().includes(needle) ||
        String(result.translation || "").toLowerCase().includes(needle)
      );
    });
  }

  function historyButton(labelKey, svg, onClick) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "history-button";
    button.title = Glossy.i18n.t(labelKey);
    button.setAttribute("aria-label", button.title);
    button.innerHTML = svg;
    button.addEventListener("click", (event) => {
      // The click would otherwise reach the entry and reopen it.
      event.stopPropagation();
      onClick();
    });
    return button;
  }

  /** Draws the history the backend keeps, keeping the keyboard on its row. */
  function renderHistory() {
    redrawing(drawHistory);
  }

  function drawHistory() {
    const entries = matchingHistory();
    els.historyList.innerHTML = "";

    entries.forEach((entry) => {
      const result = entry.result || {};
      const item = document.createElement("li");
      item.className = "history-item";
      item.tabIndex = 0;
      item.setAttribute("role", "listitem");
      item.title = Glossy.i18n.t("history.open");
      // Which entry the row is, so the keyboard can find it again after the
      // list is drawn again.
      item.dataset.id = String(entry.id);

      const body = document.createElement("div");
      body.className = "history-body";
      const source = document.createElement("div");
      source.className = "history-source";
      source.textContent = result.sourceText || "";
      const translation = document.createElement("div");
      translation.className = "history-translation";
      translation.textContent = result.translation || "";
      const meta = document.createElement("div");
      meta.className = "history-meta";
      meta.textContent = [
        Glossy.i18n.providerName(result.provider),
        new Date(Number(entry.at) * 1000).toLocaleString(),
      ].join(" · ");
      body.append(source, translation, meta);

      const actions = document.createElement("div");
      actions.className = "history-actions";
      const kept = keptIn(result) !== null;
      const star = historyButton(
        kept ? "history.unstar" : "history.star",
        STAR_ICON,
        () => toggleKept(result),
      );
      if (kept) star.dataset.state = "on";
      actions.append(
        star,
        historyButton("history.copy", COPY_ICON, async () => {
          await Glossy.invoke("copy_text", { text: String(result.translation || "") }).catch(
            () => false,
          );
          showToast(Glossy.i18n.t("toast.copied"));
        }),
        historyButton("history.remove", REMOVE_ICON, async () => {
          await Glossy.invoke("history_remove", { id: entry.id }).catch(() => null);
          await loadHistory();
        }),
      );
      // The star and the two buttons belong to this entry, which is how the
      // keyboard finds the one it was on when the list is drawn again.
      for (const button of actions.children) button.dataset.id = String(entry.id);

      const open = () => Glossy.invoke("history_reopen", { id: entry.id }).catch(() => null);
      item.addEventListener("click", open);
      item.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          open();
        }
      });

      item.append(body, actions);
      els.historyList.appendChild(item);
    });

    els.historyList.hidden = entries.length === 0;
    els.historyCount.textContent = history.length
      ? Glossy.i18n.t(
          entries.length === history.length ? "history.count" : "history.matching",
          entries.length,
          history.length,
        )
      : Glossy.i18n.t("history.empty");
  }

  async function clearHistory() {
    await Glossy.invoke("history_clear").catch(() => null);
    await loadHistory();
    showToast(Glossy.i18n.t("history.cleared"));
  }

  /** Reads the wordbook the backend keeps and shows it. */
  async function loadVocabulary() {
    try {
      vocabulary = (await Glossy.invoke("vocabulary_list")) || [];
    } catch (error) {
      vocabulary = [];
    }
    renderVocabulary();
  }

  /** The key one kept card answers to: a text in one target language. */
  function keptKey(result) {
    return `${String((result && result.sourceText) || "")}\u0000${String(
      (result && result.targetLang) || "",
    )}`;
  }

  /** The wordbook answer under the same key, when it holds one. */
  function keptIn(result) {
    const key = keptKey(result);
    return vocabulary.find((entry) => keptKey(entry.result) === key) || null;
  }

  /** The kept cards that match what the search box holds. */
  function matchingVocabulary() {
    const needle = els.vocabularySearch.value.trim().toLowerCase();
    if (!needle) return vocabulary;
    return vocabulary.filter((entry) => {
      const result = entry.result || {};
      return (
        String(result.sourceText || "").toLowerCase().includes(needle) ||
        String(result.translation || "").toLowerCase().includes(needle)
      );
    });
  }

  /** Keeps a card, or takes it back out of the wordbook. */
  async function toggleKept(result) {
    let kept = false;
    try {
      kept = (await Glossy.invoke("vocabulary_toggle", { result })) === true;
    } catch (error) {
      showToast(Glossy.errorMessage(error));
      return;
    }
    showToast(Glossy.i18n.t(kept ? "vocabulary.kept" : "vocabulary.dropped"));
    await loadVocabulary();
  }

  /** Draws the wordbook, keeping the keyboard on the entry it was on. */
  function renderVocabulary() {
    redrawing(drawVocabulary);
  }

  function drawVocabulary() {
    const entries = matchingVocabulary();
    const hiding = els.vocabularyHide.checked;
    els.vocabularyList.innerHTML = "";

    entries.forEach((entry) => {
      const result = entry.result || {};
      const item = document.createElement("li");
      item.className = "history-item";
      item.tabIndex = 0;
      item.setAttribute("role", "listitem");
      item.title = Glossy.i18n.t("vocabulary.open");
      item.dataset.id = String(entry.id);

      const body = document.createElement("div");
      body.className = "history-body";
      const source = document.createElement("div");
      source.className = "history-source";
      source.textContent = result.sourceText || "";
      const translation = document.createElement("div");
      translation.className = "history-translation";
      // A word that is being tested on shows nothing until the answer is asked
      // for; the click that asks does not open the card as well.
      if (hiding && !revealedWords.has(entry.id)) {
        translation.classList.add("masked");
        const reveal = document.createElement("button");
        reveal.type = "button";
        reveal.className = "vocabulary-reveal";
        reveal.textContent = Glossy.i18n.t("vocabulary.reveal");
        // Named, so the keyboard can find the row it belongs to after the redraw
        // the reveal itself causes — the button is what disappears.
        reveal.dataset.act = "reveal";
        reveal.dataset.id = String(entry.id);
        reveal.addEventListener("click", (event) => {
          event.stopPropagation();
          revealedWords.add(entry.id);
          renderVocabulary();
        });
        translation.appendChild(reveal);
      } else {
        translation.textContent = result.translation || "";
      }
      const meta = document.createElement("div");
      meta.className = "history-meta";
      meta.textContent = [
        Glossy.i18n.languageName(result.targetLang),
        Glossy.i18n.providerName(result.provider),
        new Date(Number(entry.at) * 1000).toLocaleString(),
      ].join(" \u00b7 ");
      body.append(source, translation, meta);

      const actions = document.createElement("div");
      actions.className = "history-actions";
      actions.append(
        historyButton("vocabulary.copy", COPY_ICON, async () => {
          await Glossy.invoke("copy_text", { text: String(result.translation || "") }).catch(
            () => false,
          );
          showToast(Glossy.i18n.t("toast.copied"));
        }),
        historyButton("vocabulary.drop", REMOVE_ICON, async () => {
          await Glossy.invoke("vocabulary_remove", { id: entry.id }).catch(() => null);
          await loadVocabulary();
        }),
      );
      // Both buttons belong to this entry, which is how the keyboard finds the
      // one it was on after the list is drawn again.
      for (const button of actions.children) button.dataset.id = String(entry.id);

      const open = () => Glossy.invoke("vocabulary_reopen", { id: entry.id }).catch(() => null);
      item.addEventListener("click", open);
      item.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          open();
        }
      });

      item.append(body, actions);
      els.vocabularyList.appendChild(item);
    });

    els.vocabularyList.hidden = entries.length === 0;
    els.vocabularyCount.textContent = vocabulary.length
      ? Glossy.i18n.t(
          entries.length === vocabulary.length ? "vocabulary.count" : "vocabulary.matching",
          entries.length,
          vocabulary.length,
        )
      : Glossy.i18n.t("vocabulary.empty");
  }

  async function clearVocabulary() {
    await Glossy.invoke("vocabulary_clear").catch(() => null);
    // Nothing is left to give away.
    revealedWords.clear();
    await loadVocabulary();
    showToast(Glossy.i18n.t("vocabulary.cleared"));
  }

  /** Writes the settings to Documents and reports where they landed. */
  async function exportSettings() {
    try {
      const path = await Glossy.invoke("export_settings");
      showToast(Glossy.i18n.t("backup.exported", path));
    } catch (error) {
      showToast(Glossy.i18n.t("backup.exportFailed") + Glossy.errorMessage(error));
    }
  }

  /** Replaces the settings with those of a file the user picked. */
  async function importSettings(file) {
    try {
      const json = await file.text();
      apply(await Glossy.invoke("import_settings", { json }));
      await refreshStatus();
      await loadHistory();
      showToast(Glossy.i18n.t("backup.imported"));
    } catch (error) {
      showToast(Glossy.i18n.t("backup.importFailed") + Glossy.errorMessage(error));
    }
  }

  /** The log's path and size, as it was last read. */
  let logStatus = null;

  /** A size in kilobytes, which is the only size this scale needs. */
  function kilobytes(bytes) {
    const value = Number(bytes || 0) / 1024;
    return value >= 10 ? String(Math.round(value)) : value.toFixed(1);
  }

  /** Reads where the log is and how much it holds. */
  async function refreshLog() {
    try {
      logStatus = await Glossy.invoke("log_status");
    } catch (error) {
      // The page still says where the log would be, which is what a user who is
      // about to hand it over needs to know; only the size is missing.
      logStatus = null;
    }
    renderLog();
  }

  /** Writes the log's path and size into the page. */
  function renderLog() {
    if (!els.logPath) return;
    els.logPath.textContent =
      logStatus && logStatus.path ? logStatus.path : Glossy.i18n.t("log.noPath");
    els.logSize.textContent = logStatus
      ? Glossy.i18n.t("log.usage", kilobytes(logStatus.size), kilobytes(logStatus.cap))
      : "";
    els.logClear.disabled = !logStatus;
  }

  /** Whether this build can update at all; false without a signing key. */
  let canUpdate = true;

  /** The release waiting to be installed, if any. */
  let pendingUpdate = null;

  /** Shows the line under the update buttons, empty when there is nothing to say. */
  function setUpdateStatus(key, ...args) {
    els.updateStatus.textContent = key ? Glossy.i18n.t(key, ...args) : "";
  }

  /** The version this copy is, printed under the Updates heading. */
  let buildVersion = "";

  function renderAppVersion() {
    els.appVersion.textContent = buildVersion ? Glossy.i18n.t("app.version", buildVersion) : "";
  }

  async function loadAppVersion() {
    try {
      buildVersion = String(await Glossy.invoke("app_version"));
    } catch {
      buildVersion = "";
    }
    renderAppVersion();
  }

  function rememberUpdate(info) {
    pendingUpdate = info || null;
    els.updateInstall.hidden = !pendingUpdate;
    els.updateCheck.hidden = !!pendingUpdate;
    if (pendingUpdate) setUpdateStatus("update.available", pendingUpdate.version);
    return pendingUpdate;
  }

  /** Asks the release feed; only a click on the button reports "up to date". */
  async function checkForUpdate(announce) {
    if (!canUpdate) return;
    els.updateCheck.disabled = true;
    try {
      const info = rememberUpdate(await Glossy.invoke("check_for_update"));
      if (info) showToast(Glossy.i18n.t("update.found", info.version));
      else if (announce) showToast(Glossy.i18n.t("update.upToDate"));
    } catch (error) {
      if (announce) showToast(Glossy.i18n.t("update.failed") + Glossy.errorMessage(error));
    } finally {
      els.updateCheck.disabled = false;
    }
  }

  async function installUpdate() {
    els.updateInstall.disabled = true;
    setUpdateStatus("update.downloading");
    try {
      // The new version only runs after a restart, so this call does not return
      // before Glossy has closed.
      await Glossy.invoke("install_update");
    } catch (error) {
      els.updateInstall.disabled = false;
      setUpdateStatus("update.available", pendingUpdate ? pendingUpdate.version : "");
      showToast(Glossy.i18n.t("update.failed") + Glossy.errorMessage(error));
    }
  }

  /** Hides the whole section on a build that cannot update itself. */
  async function loadUpdateCapability() {
    try {
      canUpdate = !!(await Glossy.invoke("update_capability"));
    } catch {
      canUpdate = false;
    }
    els.updateUnavailable.hidden = canUpdate;
    els.updateCheck.hidden = !canUpdate;
    els.checkUpdates.disabled = !canUpdate;
  }

  /** Fills the dropdown with the programs that currently own a visible window. */
  async function loadRunningApps() {
    const loading = new Option(Glossy.i18n.t("ignored.loading"), "");
    els.ignoredRunning.innerHTML = "";
    els.ignoredRunning.appendChild(loading);
    let apps = [];
    try {
      apps = (await Glossy.invoke("running_apps")) || [];
    } catch (error) {
      apps = [];
    }
    runningApps = apps.length;
    els.ignoredRunning.innerHTML = "";
    els.ignoredRunning.appendChild(
      new Option(
        apps.length ? Glossy.i18n.t("ignored.running") : Glossy.i18n.t("ignored.none"),
        "",
      ),
    );
    apps.forEach((app) => {
      const title = String(app.title || "").trim();
      const label = title ? app.name + " · " + (title.length > 48 ? title.slice(0, 48) + "…" : title) : app.name;
      els.ignoredRunning.appendChild(new Option(label, app.name));
    });
  }

  /** Re-labels the dropdown header after a language change; -1 means "loading". */
  function relabelRunningApps() {
    const head = els.ignoredRunning.options[0];
    if (!head || head.value !== "") return;
    const key = runningApps < 0 ? "ignored.loading" : runningApps ? "ignored.running" : "ignored.none";
    head.textContent = Glossy.i18n.t(key);
  }

  /**
   * Writes the shortcuts into the tooltips of the buttons that have one.
   *
   * The three combinations are recorded by the user, so they are read from the
   * settings on every pass rather than written into the markup: a tooltip has to
   * name the key that works now, and a field the user cleared means no key at all.
   */
  function syncShortcutHints() {
    const stored = settings || {};
    const shot = Glossy.i18n.withShortcut(Glossy.i18n.t("ocr.run"), stored.hotkeyOcr);
    els.ocrRun.title = shot;
    els.ocrRun.setAttribute("aria-label", shot);
  }

  /** Re-applies the interface language to everything the script writes itself. */
  function applyLanguage() {
    Glossy.i18n.apply(document);
    syncShortcutHints();
    renderIgnored();
    renderSourceLangs();
    relabelRunningApps();
    syncService();
    renderAppVersion();
    showHotkeys(lastStatus);
    syncAllHotkeyLabels();
    fillSample();
    syncDemoLanguage();
    renderHistory();
    // The names of the language packs are translated text, so the list is drawn
    // again rather than relabelled in place.
    if (ocrStatus) renderOcrEngine(ocrStatus);
    if (offlineStatus) renderOfflinePack(offlineStatus);
    if (logStatus) renderLog();
    if (settings) {
      // A pick that is not saved yet keeps its place; one the engine does not
      // translate settles on the first language it does.
      els.targetLang.value = fillLanguages(els.targetLang.value || settings.targetLang);
      // A pick that is not saved yet keeps its place here too.
      fillSubtitleFields({
        subtitleSourceLang: els.subtitleSourceLang.value || settings.subtitleSourceLang,
        subtitleTargetLang: els.subtitleTargetLang.value || settings.subtitleTargetLang,
        subtitlePack: els.subtitlePack.value || settings.subtitlePack,
      });
    }
  }

  /** Re-labels the translation card, and rebuilds the language bar with it. */
  function syncDemoLanguage() {    els.demoFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    els.demoTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    els.demoSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
    if (!els.demoLangbar.hidden) showLanguages(demoResult);
  }

  /** Writes the stored settings into the window, keeping the keyboard. */
  function apply(next) {
    redrawing(() => drawSettings(next));
  }

  function drawSettings(next) {
    settings = next;
    // What is on screen now is what is stored, so nothing here is waiting to be
    // written back any more.
    pickedService = false;
    pickedTarget = false;
    els.enabled.checked = !!next.enabled;
    els.triggerOnDrag.checked = !!next.triggerOnDrag;
    els.triggerOnDoubleClick.checked = !!next.triggerOnDoubleClick;
    els.restoreClipboard.checked = !!next.restoreClipboard;
    els.showOriginal.checked = !!next.showOriginal;
    els.autostart.checked = !!next.autostart;
    els.minSelectionLen.value = String(numberOr(next.minSelectionLen, 2));
    ignored = normalizeIgnored(next.ignoredApps);
    sourceLangs = normalizeSourceLangs(next.sourceLangs);
    for (const row of HOTKEYS) {
      row.input.value = next[row.key] || "";
      row.saved = row.input.value;
    }
    els.theme.value = themeOr(next.theme);
    els.fontScale.value = String(pick(FONT_SCALES, next.fontScale, 100));
    els.popupWidth.value = String(pick(POPUP_SIZES, next.popupWidth, 356));
    els.popupOpacity.value = String(pick(OPACITIES, next.popupOpacity, 100));
    els.autoCloseSecs.value = String(pick(AUTO_CLOSE, next.autoCloseSecs, 0));
    els.closeAfterCopy.checked = !!next.closeAfterCopy;
    els.unitsEnabled.checked = next.unitsEnabled !== false;
    els.demoUnits.checked = els.unitsEnabled.checked;
    els.wordSentence.checked = !!next.wordSentence;
    els.sentencePairs.checked = !!next.sentencePairs;
    els.compactPopup.checked = !!next.compactPopup;
    els.speechRate.value = String(pick(SPEECH_RATES, next.speechRate, 0));
    els.fallbackEnabled.checked = next.fallbackEnabled !== false;
    fallbackOrder = normalizeFallbackOrder(next.fallbackOrder);
    els.historyLimit.value = String(pick(HISTORY_LIMITS, next.historyLimit, 50));
    els.checkUpdates.checked = !!next.checkUpdates;
    ocrPacks = normalizePacks(next.ocrPacks);
    developerMode = !!next.developerMode;
    applyDeveloperMode();
    fillSubtitleFields(next);
    if (els.subtitleFontSize) {
      els.subtitleFontSize.value = String(Math.min(40, Math.max(13, Number(next.subtitleFontSize || 26))));
    }
    applyTheme(els.theme.value);
    // The engine comes first: the two language lists are the languages it
    // takes, so what they offer depends on it.
    els.service.value = serviceOf(next);
    els.targetLang.value = fillLanguages(next.targetLang);
    els.uiLang.value = next.uiLang === "zh" || next.uiLang === "en" ? next.uiLang : "system";
    Glossy.i18n.set(els.uiLang.value);
    els.options.dataset.disabled = String(!next.enabled);
    applyLanguage();
    syncChannelQuota();
  }

  /** The recorded combinations, keyed the way the settings store them. */
  function hotkeyValues() {
    const values = {};
    for (const row of HOTKEYS) values[row.key] = row.saved;
    return values;
  }

  function collect() {
    const stored = settings || {};
    // The engine and the language it translates into are the card's to change
    // too, and this window may have missed that while it was hidden: what it
    // shows is only written back when the pick was made here.
    const service = pickedService ? els.service.value : serviceOf(stored);
    return {
      ...settings,
      ...hotkeyValues(),
      enabled: els.enabled.checked,
      triggerOnDrag: els.triggerOnDrag.checked,
      triggerOnDoubleClick: els.triggerOnDoubleClick.checked,
      restoreClipboard: els.restoreClipboard.checked,
      showOriginal: els.showOriginal.checked,
      autostart: els.autostart.checked,
      minSelectionLen: numberOr(els.minSelectionLen.value, 2),
      ignoredApps: ignored,
      sourceLangs: sourceLangs,
      theme: els.theme.value,
      fontScale: numberOr(els.fontScale.value, 100),
      popupWidth: numberOr(els.popupWidth.value, 356),
      popupOpacity: numberOr(els.popupOpacity.value, 100),
      autoCloseSecs: numberOr(els.autoCloseSecs.value, 0),
      closeAfterCopy: els.closeAfterCopy.checked,
      // One field for one choice: which service translates. The file used to
      // spread it over four, and a file that still does is folded into this by
      // the backend's migration before it is read.
      service,
      unitsEnabled: els.unitsEnabled.checked,
      wordSentence: els.wordSentence.checked,
      sentencePairs: els.sentencePairs.checked,
      compactPopup: els.compactPopup.checked,
      speechRate: numberOr(els.speechRate.value, 0),
      fallbackEnabled: els.fallbackEnabled.checked,
      fallbackOrder: fallbackOrder.slice(),
      historyLimit: numberOr(els.historyLimit.value, 50),
      checkUpdates: els.checkUpdates.checked,
      ocrPacks: ocrPacks.slice(),
      subtitleSourceLang: els.subtitleSourceLang.value || "auto",
      subtitleTargetLang: els.subtitleTargetLang.value || stored.subtitleTargetLang || "",
      subtitlePack: els.subtitlePack.value || stored.subtitlePack || "",
      subtitleFontSize: Math.min(40, Math.max(13, numberOr(els.subtitleFontSize.value, stored.subtitleFontSize || 26))),
      targetLang: pickedTarget ? els.targetLang.value : String(stored.targetLang || ""),
      uiLang: els.uiLang.value,
    };
  }

  async function save() {
    try {
      apply(await Glossy.invoke("save_settings", { settings: collect() }));
      showToast(Glossy.i18n.t("toast.saved"));
      await refreshStatus();
      // A change of the cap, or of nothing at all: the list stays right by
      // asking the backend what it remembers.
      await loadHistory();
    } catch (error) {
      showToast(Glossy.i18n.t("toast.saveFailed") + Glossy.errorMessage(error));
    }
  }

  function scheduleSave() {
    pending = true;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, 180);
  }

  /**
   * Writes a change the debounce is still holding.
   *
   * The settings window is closed (and the window is hidden rather than
   * destroyed) as soon as the user is done, which happens well inside the 180ms
   * the debounce waits, so the last edit would be dropped without this.
   */
  function flushSave() {
    clearTimeout(saveTimer);
    saveTimer = 0;
    if (!pending) return;
    pending = false;
    save();
  }

  async function refreshStatus() {
    let status = null;
    try {
      status = await Glossy.invoke("capture_status");
    } catch (error) {
      setStatus("bad", Glossy.i18n.t("status.unavailable"), Glossy.errorMessage(error));
      return;
    }
    lastStatus = status;
    showHotkeys(status);
    if (!els.enabled.checked) {
      setStatus("off", Glossy.i18n.t("status.paused"), Glossy.i18n.t("status.paused.title"));
    } else if (status && status.hooked) {
      setStatus("ok", Glossy.i18n.t("status.listening"), Glossy.i18n.t("status.listening.title"));
    } else {
      setStatus(
        "bad",
        Glossy.i18n.t("status.hook"),
        (status && status.error) || Glossy.i18n.t("status.hook.title"),
      );
    }
  }

  function showHotkey(row, status) {
    // A recording owns the line while it is running: the event that ends it
    // writes the verdict, and the registration behind it follows a moment later.
    if (row.recording) return;
    const reported = ((status && status.hotkeys) || []).find(
      (entry) => entry && entry.slot === row.name,
    );
    if (reported && reported.spec) {
      setHotkeyHint(row, "ok", Glossy.i18n.t(row.active, reported.spec));
    } else if (reported && reported.error) {
      setHotkeyHint(row, "bad", reported.error);
    } else {
      setHotkeyHint(row, "", Glossy.i18n.t("hotkey.none"));
    }
  }

  function showHotkeys(status) {
    for (const row of HOTKEYS) showHotkey(row, status);
  }

  function setHotkeyHint(row, tone, text) {
    if (tone) row.hint.dataset.tone = tone;
    else delete row.hint.dataset.tone;
    row.hint.textContent = text;
  }

  /** Keeps the field's placeholder and the record button in the set language. */
  function syncHotkeyLabels(row) {
    row.input.placeholder = Glossy.i18n.t(row.recording ? "hotkey.press" : "hotkey.empty");
    row.record.textContent = Glossy.i18n.t(row.recording ? "hotkey.cancel" : "hotkey.record");
  }

  function syncAllHotkeyLabels() {
    for (const row of HOTKEYS) syncHotkeyLabels(row);
  }

  function startHotkey(row) {
    if (row.recording) return;
    // Two recorders listening at once would fight over the same keystrokes.
    for (const other of HOTKEYS) if (other !== row) cancelHotkey(other);
    row.recording = true;
    row.input.value = "";
    row.input.dataset.recording = "true";
    syncHotkeyLabels(row);
    setHotkeyHint(row, "", Glossy.i18n.t("hotkey.press"));
    row.input.focus();
  }

  /** Leaves recording mode and puts the stored combination back on screen. */
  function stopHotkey(row) {
    row.recording = false;
    delete row.input.dataset.recording;
    row.input.value = row.saved;
    syncHotkeyLabels(row);
  }

  function cancelHotkey(row) {
    stopHotkey(row);
    showHotkey(row, lastStatus);
  }

  /** Reads a combination off a keypress and hands it to the backend. */
  function recordHotkey(row, event) {
    if (event.key === "Escape" && !event.altKey && !event.metaKey) {
      event.preventDefault();
      cancelHotkey(row);
      return;
    }

    // Tab is how the keyboard leaves a field, and a recorder that swallowed it
    // would hold the keyboard for good - the field is reached by Tab in the
    // first place, and focusing it is what starts the recording. Plain Tab and
    // Shift+Tab leave it unchanged; a modified one is still a combination to
    // record, so `Ctrl+Tab` and `Ctrl+Shift+Tab` are still spelled out here.
    if (event.key === "Tab" && !event.ctrlKey && !event.altKey && !event.metaKey) {
      cancelHotkey(row);
      return;
    }

    event.preventDefault();
    const read = Glossy.keycombo.capture(event);
    if (read.problem === "modifiers") {
      setHotkeyHint(row, "", Glossy.i18n.t("hotkey.press"));
      return;
    }
    if (read.problem === "unknown") {
      setHotkeyHint(row, "bad", Glossy.i18n.t("hotkey.unsupported", read.key || "?"));
      return;
    }
    if (read.problem === "nomod") {
      setHotkeyHint(row, "bad", Glossy.i18n.t("hotkey.hold"));
      return;
    }

    const clash = Glossy.keycombo.conflict(read.spec);
    if (clash === "blocked") {
      stopHotkey(row);
      setHotkeyHint(row, "bad", Glossy.i18n.t("hotkey.blocked", read.spec));
      return;
    }

    row.saved = read.spec;
    stopHotkey(row);
    setHotkeyHint(row, "", Glossy.i18n.t("hotkey.recorded", read.spec));
    // The warning outlives the line: the registration that follows replaces it
    // with the verdict either way.
    if (clash === "busy") showToast(Glossy.i18n.t("hotkey.busy", read.spec));
    scheduleSave();
  }

  async function runDemo(value) {
    const text = String(value || "").trim();
    if (text.length < 2) return;

    demoValue = text;
    demoResult = null;
    const mine = ++demoTicket;
    els.demoCard.hidden = false;
    els.demoHeadword.textContent = text;
    els.demoLangbar.hidden = true;
    Glossy.render.loading(els.demoResult);
    try {
      const result = await Glossy.invoke("translate_text", {
        text,
        sourceLang: demoPair.source === AUTO ? null : demoPair.source,
        targetLang: demoPair.target,
      });
      if (mine !== demoTicket) return;
      demoResult = result;
      demoDetected = result.sourceLang || "";
      Glossy.render.result(els.demoResult, result, {
        showOriginal: els.showOriginal.checked,
        compact: els.compactPopup.checked,
      });
      if (result.sourceText) els.demoHeadword.textContent = String(result.sourceText);
      showLanguages(result);
      refineDemo(result, mine);
    } catch (error) {
      if (mine !== demoTicket) return;
      Glossy.render.error(els.demoResult, Glossy.errorMessage(error), () => runDemo(text));
    }
    // A translation through the shared server eats into the allowance shown
    // under the address, and a refusal is exactly when it is worth looking at,
    // so what is left is read again either way.
    if (cloudQuotaVisible() && mine === demoTicket) refreshCloudQuota();
  }

  /** Grows the demo card into the full word entry once the lookups have
      answered, the same way the floating popup does. */
  async function refineDemo(result, mine) {
    if (!result || result.kind !== "word") return;
    if (result.phonetic && (result.meanings || []).length && result.example) return;

    const waiting = { ...result, phonetic: null, meanings: [], example: null };
    Glossy.render.result(els.demoResult, waiting, {
      showOriginal: els.showOriginal.checked,
      compact: els.compactPopup.checked,
      pending: true,
    });

    let details = null;
    try {
      details = await Glossy.invoke("word_details", {
        text: result.sourceText || text,
        sourceLang: demoPair.source === AUTO ? null : demoPair.source,
        targetLang: result.targetLang || null,
      });
    } catch (error) {
      details = null;
    }
    if (mine !== demoTicket || !demoResult) return;

    demoResult = {
      ...demoResult,
      phonetic: demoResult.phonetic || (details && details.phonetic) || null,
      meanings: (demoResult.meanings || []).length
        ? demoResult.meanings
        : (details && details.meanings) || [],
      example: demoResult.example || (details && details.example) || null,
      synonyms: (demoResult.synonyms || []).length
        ? demoResult.synonyms
        : (details && details.synonyms) || [],
      forms: (demoResult.forms || []).length ? demoResult.forms : (details && details.forms) || [],
      context: demoResult.context || (details && details.context) || null,
    };
    // Drawn either way, so the card loses its placeholder when the lookups have
    // nothing to add.
    drawDemo();
  }

  /** Draws the sample card, with the switches of the popup applied to it. */
  function drawDemo() {
    Glossy.render.result(els.demoResult, demoResult, {
      showOriginal: els.showOriginal.checked,
      compact: els.compactPopup.checked,
    });
  }

  /** Label of the entry that hands the source language over to the provider. */
  function autoLabel() {
    const name = Glossy.languageName(demoDetected);
    const label = Glossy.i18n.t("popup.autoDetected");
    return name ? `${label} · ${name}` : label;
  }

  /**
   * Rebuilds one language dropdown from the languages the chosen engine
   * translates, and settles on the wanted value while it is one of them: on
   * "detect it" (the source bar) or on the first language the engine takes (the
   * target bar) otherwise.
   *
   * A code the shared list never had (a language the provider detected) is kept
   * as an extra entry, because it did not come from the menu.
   */
  function fillLanguageSelect(select, selected, withAuto) {
    const codes = Glossy.languagesFor(els.service.value);
    if (
      selected &&
      selected !== AUTO &&
      codes.indexOf(selected) === -1 &&
      LANGUAGES.indexOf(selected) === -1
    ) {
      codes.unshift(selected);
    }

    select.innerHTML = "";
    if (withAuto) {
      const option = document.createElement("option");
      option.value = AUTO;
      option.textContent = autoLabel();
      select.appendChild(option);
    }
    codes.forEach((code) => {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = Glossy.languageName(code);
      select.appendChild(option);
    });
    const wanted = selected && codes.indexOf(selected) !== -1;
    select.value = wanted ? selected : withAuto ? AUTO : codes[0];
  }

  /** Reflects the pair of the current result in the language bar. */
  function showLanguages(result) {
    const target = demoPair.target || (result && result.targetLang) || "";
    if (!target) {
      els.demoLangbar.hidden = true;
      return;
    }
    fillLanguageSelect(els.demoFrom, demoPair.source, true);
    fillLanguageSelect(els.demoTo, target, false);
    els.demoLangbar.hidden = false;
  }

  /** Back to "detect the source and use the configured target". */
  function resetDemoLanguages() {
    demoPair = { source: AUTO, target: null };
    demoDetected = "";
  }

  /** Drops the demo languages the chosen engine does not translate, the same
      way the popup's bar does when the engine is switched under it. */
  function dropUnservedPair() {
    const unserved = (code) =>
      !!code && code !== AUTO && !Glossy.servesLanguage(els.service.value, code);
    if (unserved(demoPair.source)) demoPair.source = AUTO;
    if (unserved(demoPair.target)) demoPair.target = null;
  }

  /**
   * Remembers the language the card is translated into, so the next selection
   * starts from it rather than from the target the settings hold.
   */
  function rememberTarget(code) {
    if (!code || code === AUTO) return;
    Glossy.invoke("set_target_lang", { code }).catch((error) => {
      console.warn("glossy: could not remember the target language", error);
    });
  }

  /** Reverse-translates the card: the translation becomes the new selection. */
  function swapDemo() {
    if (!demoResult || !demoResult.translation) return;
    const value = String(demoResult.translation).trim();
    if (value.length < 2) return;

    const source = demoPair.source === AUTO ? demoResult.sourceLang : demoPair.source;
    const target = demoPair.target || demoResult.targetLang;
    demoPair = { source: target || AUTO, target: source || null };
    demoDetected = "";
    runDemo(value);
  }

  /** Text the user highlighted, in the box or anywhere else on the page. */
  function highlighted() {
    const onPage = String(window.getSelection() || "").trim();
    if (onPage) return onPage;
    const start = els.demoText.selectionStart;
    const end = els.demoText.selectionEnd;
    if (typeof start === "number" && typeof end === "number" && end > start) {
      return els.demoText.value.slice(start, end);
    }
    return "";
  }

  function translateHighlighted() {
    const picked = highlighted().replace(/\s+/g, " ").trim();
    if (picked.length < 2) return;
    selection = picked;
    els.selectionHint.textContent = picked.length > 42 ? picked.slice(0, 42) + "…" : picked;
    // A brand new selection starts from the configured languages again, exactly
    // like the popup does for every captured selection.
    resetDemoLanguages();
    runDemo(picked);
  }

  /** Keeps the sample text in the box while the user has not typed anything. */
  function fillSample() {
    const next = Glossy.i18n.t("demo.demoText");
    if (!els.demoText.value || els.demoText.value === sampleText) els.demoText.value = next;
    sampleText = next;
  }

  /** Draws the card again, so a units or a language change lands on the spot. */
  function refreshDemo() {
    if (demoValue) runDemo(demoValue);
  }

  /**
   * The translate area carries its own units switch. Both switches write the one
   * setting, so each mirrors the other. The backend attaches the conversions
   * from the settings it holds, so the write is awaited before the card is
   * redrawn — the conversions of the old switch would otherwise come back.
   */
  async function setUnits(enabled) {
    els.unitsEnabled.checked = enabled;
    els.demoUnits.checked = enabled;
    clearTimeout(saveTimer);
    saveTimer = 0;
    pending = false;
    await save();
    refreshDemo();
  }

  /** Puts the clipboard in the box and translates it right away. */
  async function pasteDemo() {
    let text = "";
    try {
      text = String((await Glossy.invoke("read_clipboard")) || "");
    } catch (error) {
      text = "";
    }
    if (!text.trim()) {
      showToast(Glossy.i18n.t("demo.pasteEmpty"));
      return;
    }
    els.demoText.value = text;
    resetDemoLanguages();
    runDemo(text);
  }

  /** Empties the box and takes the card away. */
  function clearDemo() {
    // A translation still in flight must not draw into the emptied card.
    demoTicket += 1;
    demoValue = "";
    demoResult = null;
    demoDetected = "";
    selection = "";
    resetDemoLanguages();
    els.demoText.value = "";
    els.demoCard.hidden = true;
    els.demoHeadword.textContent = "";
    els.demoResult.innerHTML = "";
    els.demoLangbar.hidden = true;
    els.selectionHint.textContent = Glossy.i18n.t("demo.nothing");
  }

  els.options.addEventListener("change", scheduleSave);
  els.enabled.addEventListener("change", () => {
    els.options.dataset.disabled = String(!els.enabled.checked);
    scheduleSave();
  });
  els.uiLang.addEventListener("change", () => {
    Glossy.i18n.set(els.uiLang.value);
    applyLanguage();
    scheduleSave();
  });
  els.service.addEventListener("change", async () => {
    pickedService = true;
    syncService();
    syncChannelQuota();
    // What the two language lists offer follows the engine, and so does the
    // answer of the card; the save is awaited because the backend only knows
    // the new engine once it has landed.
    els.targetLang.value = fillLanguages(els.targetLang.value);
    dropUnservedPair();
    refreshDocumentLanguages();
    clearTimeout(saveTimer);
    saveTimer = 0;
    pending = false;
    await save();
    refreshDemo();
  });
  els.cloudQuotaRefresh.addEventListener("click", () => refreshCloudQuota());
  els.ignoredRunning.addEventListener("change", () => {
    const picked = els.ignoredRunning.value;
    els.ignoredRunning.value = "";
    if (!picked) return;
    addIgnored(picked);
    renderIgnored();
    scheduleSave();
  });
  els.ignoredAdd.addEventListener("click", () => {
    const value = els.ignoredInput.value.trim();
    if (!value) return;
    addIgnored(value);
    els.ignoredInput.value = "";
    renderIgnored();
    scheduleSave();
  });
  els.ignoredInput.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    els.ignoredAdd.click();
  });
  els.sourceLangsAdd.addEventListener("change", () => {
    const code = els.sourceLangsAdd.value;
    els.sourceLangsAdd.value = "";
    if (!addSourceLang(code)) return;
    renderSourceLangs();
    scheduleSave();
  });
  els.sourceLangsClear.addEventListener("click", () => {
    if (!sourceLangs.length) return;
    sourceLangs = [];
    renderSourceLangs();
    scheduleSave();
  });
  els.ignoredPick.addEventListener("click", async () => {
    try {
      await Glossy.invoke("pick_app");
      showToast(Glossy.i18n.t("ignored.picking"));
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });
  Glossy.listen("glossy://picked-app", (event) => {
    const payload = (event && event.payload) || {};
    const name = payload.name || "";
    if (!name) {
      showToast(Glossy.i18n.t("ignored.notFound"));
      return;
    }
    addIgnored(name);
    renderIgnored();
    scheduleSave();
    showToast(Glossy.i18n.t("ignored.picked", name));
  });
  els.theme.addEventListener("change", () => {
    applyTheme(els.theme.value);
    scheduleSave();
  });
  // The engine and the target language can be picked from the card as well, so
  // a change made here is recorded as this screen's own choice.
  els.targetLang.addEventListener("change", () => {
    pickedTarget = true;
  });
  [
    els.targetLang,
    els.restoreClipboard,
    els.showOriginal,
    els.autostart,
    els.minSelectionLen,
    els.fontScale,
    els.popupWidth,
    els.popupOpacity,
    els.autoCloseSecs,
    els.closeAfterCopy,
    els.unitsEnabled,
    els.wordSentence,
    els.sentencePairs,
    els.compactPopup,
    els.speechRate,
    els.fallbackEnabled,
    els.historyLimit,
    els.checkUpdates,
    els.subtitleSourceLang,
    els.subtitleTargetLang,
    els.subtitlePack,
  ].forEach((element) => {
    element.addEventListener("change", scheduleSave);
  });
  // Registered after the loop, so the mirror of the area's switch runs with it.
  els.unitsEnabled.addEventListener("change", () => setUnits(els.unitsEnabled.checked));
  els.fallbackEnabled.addEventListener("change", renderFallbackOrder);
  els.compactPopup.addEventListener("change", () => {
    if (demoResult) drawDemo();
  });

  els.historySearch.addEventListener("input", renderHistory);
  els.historyClear.addEventListener("click", clearHistory);

  els.vocabularySearch.addEventListener("input", renderVocabulary);
  els.vocabularyClear.addEventListener("click", clearVocabulary);
  els.vocabularyHide.addEventListener("change", () => {
    // Turning the test on hides everything again, including what an earlier run
    // had already given away.
    if (els.vocabularyHide.checked) revealedWords.clear();
    renderVocabulary();
  });

  // The hotkey fields record instead of taking typed text: a combination has to
  // be spelled the way the backend reads it back, and a recorder cannot typo one.
  for (const row of HOTKEYS) {
    row.input.addEventListener("focus", () => startHotkey(row));
    row.input.addEventListener("keydown", (event) => {
      if (row.recording) recordHotkey(row, event);
      else if (event.key === "Enter" || event.key === " ") startHotkey(row);
    });
    row.input.addEventListener("blur", (event) => {
      // Clicking the two buttons beside the field is not leaving it.
      if (event.relatedTarget === row.record || event.relatedTarget === row.clear) return;
      if (row.recording) cancelHotkey(row);
    });
    row.record.addEventListener("click", () => {
      if (row.recording) cancelHotkey(row);
      else startHotkey(row);
    });
    row.clear.addEventListener("click", () => {
      row.saved = "";
      stopHotkey(row);
      setHotkeyHint(row, "", Glossy.i18n.t("hotkey.none"));
      scheduleSave();
    });
  }
  // Clicking away from the window takes the keyboard with it, and a recording
  // that cannot see the keys any more would sit there waiting forever.
  window.addEventListener("blur", () => {
    for (const row of HOTKEYS) if (row.recording) cancelHotkey(row);
  });

  els.updateCheck.addEventListener("click", () => checkForUpdate(true));
  els.updateInstall.addEventListener("click", installUpdate);

  // The address is opened by the backend, which knows the one page it may open:
  // left to itself the click would navigate this window away from the settings.
  els.updatePage.addEventListener("click", async (event) => {
    event.preventDefault();
    try {
      await Glossy.invoke("open_releases_page");
    } catch (error) {
      showToast(Glossy.i18n.t("update.pageFailed") + Glossy.errorMessage(error));
    }
  });

  // The screenshot is taken by the backend, which also owns the overlay the
  // region is drawn on; this window only starts it.
  els.ocrRun.addEventListener("click", async () => {
    try {
      await Glossy.invoke("ocr_start");
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });

  // The subtitle reading asks for its two rectangles itself, so this only asks
  // for the first one: whether there is a reading to stop is asked for, because
  // the reading lives in the process and outlives this page.
  els.subtitleRun.addEventListener("click", async () => {
    try {
      await Glossy.invoke("subtitle_start");
      subtitleAskedAt = Date.now();
      subtitleRunning = false;
      drawSubtitleState();
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });

  // The boxes are moved on the screen rather than here, so this only asks for
  // them: with nothing picked yet the same command asks for the two rectangles,
  // which is what the reading itself asks for.
  els.subtitleAdjust.addEventListener("click", async () => {
    try {
      await Glossy.invoke("subtitle_edit_start");
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });

  els.subtitleStop.addEventListener("click", async () => {
    // A pick that is still being made is cancelled as well: stopping has to
    // work from the moment the first rectangle is asked for.
    try {
      await Glossy.invoke("ocr_cancel");
      await Glossy.invoke("subtitle_stop");
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
    subtitleAskedAt = 0;
    subtitleRunning = false;
    drawSubtitleState();
  });

  // Developer mode: the key is checked in the backend, so the field is only a
  // way of asking and what was answered is what turns the pages on.
  els.devUnlock.addEventListener("click", () => unlockDeveloper());
  els.devKey.addEventListener("keydown", (event) => {
    if (event.key === "Enter") unlockDeveloper();
  });
  els.devLock.addEventListener("click", async () => {
    try {
      apply(await Glossy.invoke("developer_lock"));
    } catch (error) {
      devFailed(error);
    }
  });

  async function unlockDeveloper() {
    els.devState.textContent = "";
    els.devState.removeAttribute("data-tone");
    try {
      apply(await Glossy.invoke("developer_unlock", { key: els.devKey.value }));
      els.devKey.value = "";
    } catch (error) {
      devFailed(error);
    }
  }

  /** Shows why the mode could not be changed; a wrong key is its own answer. */
  function devFailed(error) {
    const wrong = Glossy.errorMessage(error) === "dev.key";
    els.devState.setAttribute("data-tone", "bad");
    els.devState.textContent = Glossy.i18n.t(wrong ? "dev.wrongKey" : "dev.failed");
  }

  // The recognition engine the screenshot translation runs on: the page can
  // download it or take it off the disk, and it follows a download started
  // anywhere else — the first-use prompt in the popup, for instance.
  els.ocrEngineDownload.addEventListener("click", downloadOcrEngine);
  els.ocrEngineRemove.addEventListener("click", removeOcrEngine);
  els.ocrPackList.addEventListener("click", (event) => {
    const button = event.target.closest ? event.target.closest(".pack-action") : null;
    if (!button) return;
    if (button.dataset.act === "download") downloadPack(button.dataset.pack);
    else removePack(button.dataset.pack);
  });
  Glossy.listen("glossy://ocr-model", (event) => {
    if (!ocrEngineVisible()) return;
    renderOcrEngine(event && event.payload);
  });

  // The offline translation pack: the same two buttons, on the same page.
  els.offlineDownload.addEventListener("click", () => downloadOfflinePack(""));
  els.offlineRemove.addEventListener("click", () => removeOfflinePack(""));
  els.offlinePairList.addEventListener("click", (event) => {
    const button = event.target.closest ? event.target.closest(".pack-action") : null;
    if (!button) return;
    if (button.dataset.act === "download") downloadOfflinePack(button.dataset.pair);
    else removeOfflinePack(button.dataset.pair);
  });
  Glossy.listen("glossy://offline-model", (event) => {
    if (!ocrEngineVisible()) return;
    renderOfflinePack(event && event.payload);
  });

  // A whole file: the page picks it, the backend splits and translates it, and
  // every piece it finishes arrives on `glossy://document`.

  /** The choices of this page, kept so the next file finds them again. */
  const DOCUMENT_KEY = "glossy.document";
  /** The piece limit the backend takes, and the one it uses by default. */
  const DOCUMENT_LIMIT = { min: 200, max: 1500, fallback: 1500 };

  function documentChoices() {
    return {
      from: els.documentFrom.value,
      to: els.documentTo.value,
      skipPlain: els.documentSkipPlain.checked,
      autoSave: els.documentAutoSave.checked,
      limit: documentLimit(),
      folder: els.documentFolder.dataset.path || "",
    };
  }

  function rememberDocument() {
    try {
      localStorage.setItem(DOCUMENT_KEY, JSON.stringify(documentChoices()));
    } catch (error) {
      // Without storage the choices still hold for this window.
    }
  }

  /** The piece limit the page asks for, kept inside what the backend takes. */
  function documentLimit() {
    const value = Number(els.documentSegmentLimit.value) || DOCUMENT_LIMIT.fallback;
    return Math.min(Math.max(value, DOCUMENT_LIMIT.min), DOCUMENT_LIMIT.max);
  }

  /** The folder translations are written into; the backend defaults to the
      Desktop, and says so here until another one is picked. */
  function showDocumentFolder() {
    els.documentFolder.textContent =
      els.documentFolder.dataset.path || Glossy.i18n.t("document.folderDefault");
  }

  /** Reads back what this page was left with, once the settings are known. */
  function setupDocumentPage() {
    let stored = {};
    try {
      stored = JSON.parse(localStorage.getItem(DOCUMENT_KEY) || "{}") || {};
    } catch (error) {
      stored = {};
    }
    fillLanguageSelect(els.documentFrom, stored.from || AUTO, true);
    fillLanguageSelect(els.documentTo, stored.to || els.targetLang.value, false);
    els.documentSkipPlain.checked = stored.skipPlain !== false;
    els.documentAutoSave.checked = stored.autoSave === true;
    els.documentSegmentLimit.value = stored.limit || DOCUMENT_LIMIT.fallback;
    els.documentFolder.dataset.path = stored.folder || "";
    showDocumentFolder();
  }

  /** The lists follow the engine, the way the lists of the popup do. */
  function refreshDocumentLanguages() {
    fillLanguageSelect(els.documentFrom, els.documentFrom.value || AUTO, true);
    fillLanguageSelect(els.documentTo, els.documentTo.value || els.targetLang.value, false);
  }

  /** The bytes of a picked file as base64, so the backend can decode an older
   * encoding instead of being handed text that is already broken. */
  async function base64Of(file) {
    const bytes = new Uint8Array(await file.arrayBuffer());
    let binary = "";
    // In chunks, because a spread of the whole array would overflow the stack
    // on a long document.
    for (let index = 0; index < bytes.length; index += 0x8000) {
      binary += String.fromCharCode(...bytes.subarray(index, index + 0x8000));
    }
    return btoa(binary);
  }

  /** The line under the buttons, and the bar above the preview. */
  function showDocumentState(message, tone) {
    els.documentState.textContent = message || "";
    els.documentState.dataset.tone = tone || "";
  }

  function showDocumentProgress(done, total) {
    const running = total > 0;
    els.documentProgress.hidden = !running;
    if (!running) return;
    const percent = Math.round((done / total) * 100);
    els.documentProgressBar.style.width = `${percent}%`;
  }

  function forgetDocument(open) {
    els.documentRun.disabled = !open;
    els.documentCancel.hidden = true;
    els.documentSave.hidden = true;
    els.documentSample.hidden = true;
    els.documentPreview.hidden = true;
    els.documentPreview.textContent = "";
    showDocumentProgress(0, 0);
  }

  async function openDocument(file) {
    try {
      const info = await Glossy.invoke("document_open", {
        name: file.name,
        data: await base64Of(file),
        segmentChars: documentLimit(),
        skipPlainParts: els.documentSkipPlain.checked,
      });
      forgetDocument(true);
      showDocumentState(
        Glossy.i18n.t("document.picked", info.name, info.segments, info.chars),
      );
      els.documentSample.textContent = info.sample;
      els.documentSample.hidden = !info.sample;
    } catch (error) {
      forgetDocument(false);
      showDocumentState(Glossy.errorMessage(error), "bad");
    }
  }

  /** Writes the finished translation where the page says, by hand or by itself
      as soon as the file is done. */
  async function saveDocument() {
    try {
      const path = await Glossy.invoke("document_save", {
        directory: els.documentFolder.dataset.path || null,
      });
      showToast(Glossy.i18n.t("document.saved", path));
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  }

  els.documentPick.addEventListener("click", () => els.documentFile.click());
  els.documentFile.addEventListener("change", () => {
    const file = els.documentFile.files && els.documentFile.files[0];
    // Clearing the input lets the same file be picked twice in a row.
    els.documentFile.value = "";
    if (file) openDocument(file);
  });

  els.documentDrop.addEventListener("dragover", (event) => {
    // Without this the file is handed to the webview instead of to the page.
    event.preventDefault();
    els.documentDrop.dataset.dragging = "true";
  });
  els.documentDrop.addEventListener("dragleave", () => {
    delete els.documentDrop.dataset.dragging;
  });
  els.documentDrop.addEventListener("drop", (event) => {
    event.preventDefault();
    delete els.documentDrop.dataset.dragging;
    const file = event.dataTransfer && event.dataTransfer.files && event.dataTransfer.files[0];
    if (file) openDocument(file);
    else showDocumentState(Glossy.i18n.t("document.dropUnsupported"), "bad");
  });

  // A file dropped anywhere else would otherwise make the window leave the app
  // to show the file itself.
  ["dragover", "drop"].forEach((type) =>
    window.addEventListener(type, (event) => event.preventDefault()),
  );

  els.documentSwap.addEventListener("click", () => {
    const from = els.documentFrom.value;
    // There is nothing to swap "detect it" with: a file is not translated into
    // a language that still has to be looked up.
    if (from === AUTO) return;
    const to = els.documentTo.value;
    fillLanguageSelect(els.documentFrom, to, true);
    fillLanguageSelect(els.documentTo, from, false);
    rememberDocument();
  });

  [
    "documentFrom",
    "documentTo",
    "documentSkipPlain",
    "documentAutoSave",
    "documentSegmentLimit",
  ].forEach((name) =>
    els[name].addEventListener("change", () => {
      els.documentSegmentLimit.value = documentLimit();
      rememberDocument();
    }),
  );

  els.documentFolderChange.addEventListener("click", async () => {
    try {
      const folder = await Glossy.invoke("document_pick_directory");
      // An empty answer means the picker was dismissed, which changes nothing.
      if (!folder) return;
      els.documentFolder.dataset.path = folder;
      showDocumentFolder();
      rememberDocument();
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });

  els.documentRun.addEventListener("click", async () => {
    els.documentRun.disabled = true;
    els.documentCancel.hidden = false;
    els.documentPreview.hidden = true;
    els.documentSave.hidden = true;
    // The language picked on this page answers; the one left to "detect it"
    // falls back to the pair the settings hold.
    const source = els.documentFrom.value;
    const target = els.documentTo.value;
    try {
      await Glossy.invoke("document_start", {
        source: source === AUTO ? null : source,
        target: target || null,
      });
    } catch (error) {
      forgetDocument(false);
      showDocumentState(Glossy.errorMessage(error), "bad");
    }
  });

  els.documentCancel.addEventListener("click", () => {
    Glossy.invoke("document_cancel").catch(() => {});
  });

  els.documentSave.addEventListener("click", () => saveDocument());

  Glossy.listen("glossy://document", (event) => {
    const progress = (event && event.payload) || {};
    const done = Number(progress.done) || 0;
    const total = Number(progress.total) || 0;
    if (progress.state === "progress") {
      showDocumentProgress(done, total);
      showDocumentState(Glossy.i18n.t("document.working", done, total));
      return;
    }
    els.documentCancel.hidden = true;
    els.documentRun.disabled = false;
    showDocumentProgress(done, total);
    if (progress.state === "done") {
      showDocumentState(Glossy.i18n.t("document.done", done), "ok");
      els.documentPreview.textContent = progress.preview || "";
      els.documentPreview.hidden = !progress.preview;
      els.documentSave.hidden = false;
      if (els.documentAutoSave.checked) saveDocument();
      return;
    }
    if (progress.state === "cancelled") {
      showDocumentState(Glossy.i18n.t("document.cancelled", done, total));
      return;
    }
    showDocumentState(Glossy.i18n.t("document.failed", progress.message || ""), "bad");
  });

  els.settingsExport.addEventListener("click", () => exportSettings());
  els.settingsImport.addEventListener("click", () => els.settingsFile.click());
  els.settingsFile.addEventListener("change", () => {
    const file = els.settingsFile.files && els.settingsFile.files[0];
    // Clearing the input lets the same file be picked twice in a row.
    els.settingsFile.value = "";
    if (file) importSettings(file);
  });

  els.logOpen.addEventListener("click", async () => {
    try {
      await Glossy.invoke("log_open_folder");
    } catch (error) {
      showToast(Glossy.i18n.t("log.openFailed") + Glossy.errorMessage(error));
    }
  });

  els.logClear.addEventListener("click", async () => {
    try {
      await Glossy.invoke("log_clear");
      showToast(Glossy.i18n.t("log.cleared"));
      await refreshLog();
    } catch (error) {
      showToast(Glossy.i18n.t("log.clearFailed") + Glossy.errorMessage(error));
    }
  });

  els.logExport.addEventListener("click", async () => {
    try {
      const path = await Glossy.invoke("log_export");
      showToast(Glossy.i18n.t("log.exported", path));
    } catch (error) {
      showToast(Glossy.i18n.t("log.exportFailed") + Glossy.errorMessage(error));
    }
  });

  els.logCopy.addEventListener("click", async () => {
    try {
      await Glossy.invoke("log_copy");
      showToast(Glossy.i18n.t("log.copied"));
    } catch (error) {
      showToast(Glossy.i18n.t("log.copyFailed") + Glossy.errorMessage(error));
    }
  });

  els.demoText.addEventListener("mouseup", () => setTimeout(translateHighlighted, 0));
  els.demoText.addEventListener("keyup", (event) => {
    // A selection made with the keyboard (Shift+arrows, Ctrl+A) translates too.
    if (event.shiftKey || event.key === "a" || event.key === "A") translateHighlighted();
  });
  els.demoRun.addEventListener("click", () => {
    const picked = highlighted().replace(/\s+/g, " ").trim();
    runDemo(picked.length >= 2 ? picked : els.demoText.value);
  });
  els.demoPaste.addEventListener("click", pasteDemo);
  els.demoClear.addEventListener("click", clearDemo);
  els.demoText.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    runDemo(els.demoText.value);
  });
  els.demoText.addEventListener("paste", () => {
    // The new text lands after the event, so the run waits a beat for it.
    setTimeout(() => {
      const value = els.demoText.value.trim();
      if (value.length < 2) return;
      resetDemoLanguages();
      runDemo(value);
    }, 0);
  });
  els.demoUnits.addEventListener("change", () => setUnits(els.demoUnits.checked));
  els.demoFrom.addEventListener("change", () => {
    demoPair.source = els.demoFrom.value;
    demoDetected = "";
    runDemo(demoValue);
  });
  els.demoTo.addEventListener("change", () => {
    demoPair.target = els.demoTo.value;
    rememberTarget(demoPair.target);
    runDemo(demoValue);
  });
  els.demoSwap.addEventListener("click", swapDemo);
  els.demoPopup.addEventListener("click", () => {
    const value = selection || els.demoText.value.replace(/\s+/g, " ").trim();
    if (value.length < 2) return;
    Glossy.invoke("show_popup", { text: value }).catch(() => {});
  });

  // A pending change is written before the window goes away, whichever way it
  // goes away: hidden by the close button, minimized, or unloaded.
  // A translation made anywhere else — the popup, the hotkey — lands in the
  // history while this window may be open, so the list reloads on the signal
  // instead of waiting for the next start.
  Glossy.listen("glossy://history", loadHistory);
  // A word kept from the card (or from the history) arrives here; the stars of
  // the history list are drawn from the same book, so that list follows.
  Glossy.listen("glossy://vocabulary", async () => {
    await loadVocabulary();
    renderHistory();
  });
  Glossy.listen("glossy://flush-settings", flushSave);
  // The card in the popup switches its own engine and picks its own target
  // language, and both are stored, so this window follows them — the fields it
  // does not own are left alone rather than saved back over the card's choice.
  Glossy.listen("glossy://settings", (event) => {
    const next = (event && event.payload) || null;
    if (!next) return;
    // The event carries the whole stored state, so a window that is still
    // reading its own copy takes this instead of dropping it: a dropped one
    // would leave the screen showing an engine the card has already replaced.
    if (!settings) {
      apply(next);
      return;
    }
    const service = serviceOf(next);
    const language = String(next.targetLang || settings.targetLang || "");
    const switched = els.service.value !== service;
    settings = {
      ...settings,
      targetLang: language,
      service: next.service,
    };
    if (switched) {
      els.service.value = service;
      syncService();
      syncChannelQuota();
      dropUnservedPair();
    }
    if (els.targetLang.value !== language) els.targetLang.value = fillLanguages(language);
    if (switched) refreshDemo();
  });
  document.addEventListener("visibilitychange", async () => {
    if (document.hidden) {
      flushSave();
      return;
    }
    // The card switches the engine and the target language while this window is
    // hidden, and a window that was still starting up or asleep may have missed
    // the event. What is on screen has to match what the backend holds before
    // the next save reads it.
    if (pending) {
      flushSave();
      return;
    }
    try {
      apply(await Glossy.readSettings());
    } catch (error) {
      console.warn("glossy: cannot re-read the settings", error);
    }
  });
  window.addEventListener("blur", flushSave);
  window.addEventListener("focus", () => {
    if (ocrEngineVisible()) refreshOcrEngine();
  });
  window.addEventListener("pagehide", flushSave);
  window.addEventListener("beforeunload", flushSave);

  /** The page this window was left on, or an empty string the first time. */
  function rememberedPage() {
    try {
      return localStorage.getItem(PAGE_KEY) || "";
    } catch (error) {
      return "";
    }
  }

  /** Pages that no longer stand on their own, and the page that holds them now. */
  const MERGED_PAGES = { shortcuts: "general", trigger: "general" };
  /** Pages that are closed for now; their sidebar entries are disabled. */
  const CLOSED_PAGES = ["document"];

  /** Whether a page's sidebar entry is on show; a page whose is not is shut. */
  function reachable(name) {
    const item = NAV_ITEMS.find((entry) => entry.dataset.page === name);
    return !item || !item.hidden;
  }

  /** Shows one page of the sidebar, marks it current, and remembers it. */
  function showPage(name) {
    const alias = MERGED_PAGES[name] || name;
    const open =
      reachable(alias) &&
      CLOSED_PAGES.indexOf(alias) < 0 &&
      Boolean(document.getElementById("page-" + alias));
    const wanted = open ? alias : NAV_ITEMS[0].dataset.page;
    PAGES.forEach((page) => {
      page.hidden = page.id !== "page-" + wanted;
    });
    NAV_ITEMS.forEach((item) => {
      if (item.dataset.page === wanted) item.setAttribute("aria-current", "page");
      else item.removeAttribute("aria-current");
    });
    try {
      localStorage.setItem(PAGE_KEY, wanted);
    } catch (error) {
      // Without storage the pages still switch, the window just does not recall.
    }
    watchSubtitle(wanted === "subtitle");
    if (wanted === "resources") {
      refreshOcrEngine();
      refreshOfflinePack();
    }
    if (wanted === "backup") refreshLog();
    syncChannelQuota();
  }

  NAV_ITEMS.forEach((item) => {
    item.addEventListener("click", () => showPage(item.dataset.page));
  });

  /**
   * Moves the keyboard inside the sidebar.
   *
   * The sidebar is one column of buttons, so the arrows walk it the way they
   * walk a list of them, and Home and End go to its ends. Only the focus moves:
   * a section is entered with Enter or Space, the way any button is - the entry
   * that is closed for now is skipped, because it cannot be focused at all.
   */
  /** The sidebar's entries that are on show, which are the ones it can focus. */
  function navStops() {
    return NAV_ITEMS.filter((item) => !item.disabled && !item.hidden);
  }

  function walkSidebar(step) {
    const stops = navStops();
    const here = stops.indexOf(document.activeElement);
    if (here < 0) return;
    stops[(here + step + stops.length) % stops.length].focus();
  }

  function endSidebar(end) {
    const stops = navStops();
    if (stops.length) stops[end === "first" ? 0 : stops.length - 1].focus();
  }

  NAV_ITEMS.forEach((item) => {
    item.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown" || event.key === "ArrowRight") {
        event.preventDefault();
        walkSidebar(1);
      } else if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
        event.preventDefault();
        walkSidebar(-1);
      } else if (event.key === "Home") {
        event.preventDefault();
        endSidebar("first");
      } else if (event.key === "End") {
        event.preventDefault();
        endSidebar("last");
      }
    });
  });

  showPage(rememberedPage());

  (async function start() {
    // Resolved before the settings paint the window, so the effect is in place
    // from the first frame on.
    await resolveBackdrop();
    // Which languages an engine translates is asked for first, so the lists
    // already leave out what the chosen engine cannot do.
    try {
      Glossy.setServiceLanguages(await Glossy.invoke("service_languages"));
    } catch (error) {
      console.warn("glossy: cannot read the language tables", error);
    }
    try {
      apply(await Glossy.readSettings());
    } catch (error) {
      showToast(Glossy.i18n.t("toast.loadFailed") + Glossy.errorMessage(error));
    }
    // The document page is set up once the settings are in, because the target
    // language it starts from is the one they hold.
    setupDocumentPage();
    await refreshStatus();
    Glossy.listen("glossy://status", refreshStatus);
    setInterval(refreshStatus, 4000);
    loadRunningApps();
    await loadHistory();
    await loadVocabulary();
    await loadUpdateCapability();
    await loadAppVersion();
    // The start-up check ran before this window existed, so it arrives as an
    // event rather than as the answer to a call.
    Glossy.listen("glossy://update", (event) => {
      rememberUpdate((event && event.payload) || null);
    });
  })();
})(window.Glossy);
