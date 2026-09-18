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
    minSelectionLen: $("minSelectionLen"),
    hotkey: $("hotkey"),
    hotkeyHint: $("hotkeyHint"),
    ignoredList: $("ignoredList"),
    ignoredRunning: $("ignoredRunning"),
    ignoredInput: $("ignoredInput"),
    ignoredAdd: $("ignoredAdd"),
    ignoredPick: $("ignoredPick"),
    ignoredHint: $("ignoredHint"),
    theme: $("theme"),
    fontScale: $("fontScale"),
    popupWidth: $("popupWidth"),
    popupOpacity: $("popupOpacity"),
    autoCloseSecs: $("autoCloseSecs"),
    closeAfterCopy: $("closeAfterCopy"),
    targetLang: $("targetLang"),
    provider: $("provider"),
    apiIdField: $("apiIdField"),
    apiId: $("apiId"),
    apiKeyField: $("apiKeyField"),
    apiKey: $("apiKey"),
    providerHint: $("providerHint"),
    uiLang: $("uiLang"),
    status: $("status"),
    statusText: $("statusText"),
    demoText: $("demoText"),
    demoRun: $("demoRun"),
    demoCard: $("demoCard"),
    demoHeadword: $("demoHeadword"),
    demoCopy: $("demoCopy"),
    demoLangbar: $("demoLangbar"),
    demoFrom: $("demoFrom"),
    demoTo: $("demoTo"),
    demoSwap: $("demoSwap"),
    demoPopup: $("demoPopup"),
    demoResult: $("demoResult"),
    selectionHint: $("selectionHint"),
    toast: $("toast"),
  };

  const LANGUAGES = Glossy.languageCodes;

  const PROVIDER_HINTS = {
    google: "provider.hint.google",
    baidu: "provider.hint.baidu",
    zhipu: "provider.hint.zhipu",
    deepl: "provider.hint.deepl",
    openai: "provider.hint.openai",
  };

  /** Providers that need a secret, and the one that also needs an APP ID. */
  const PROVIDERS_WITH_KEY = ["baidu", "zhipu", "deepl", "openai"];

  const POPUP_SIZES = [300, 356, 400, 460, 520];
  const FONT_SCALES = [90, 100, 115, 130, 150];
  const AUTO_CLOSE = [0, 3, 5, 10, 20, 30];
  const OPACITIES = [50, 60, 70, 80, 90, 95, 100];

  /** Source value that lets the provider detect the language itself. */
  const AUTO = "auto";

  let settings = null;
  let saveTimer = 0;
  /** True while a change was made but not written to the settings file yet. */
  let pending = false;
  let toastTimer = 0;
  let selection = "";
  /** Programs that never trigger a translation, as shown by the chip list. */
  let ignored = [];
  /** Provider whose credentials the two input fields currently show. */
  let shownProvider = null;
  /** Number of running programs in the dropdown; -1 while it is being read. */
  let runningApps = -1;
  let lastStatus = null;

  /** Text the translation card shows; reused when only the languages change. */
  let demoValue = "";
  let demoResult = null;
  /** Pair of the card. `source: AUTO` asks the provider to detect the
      language and `target: null` follows the configured target. */
  let demoPair = { source: AUTO, target: null };
  let demoDetected = "";
  let demoTicket = 0;
  let demoCopyTimer = 0;
  /** Sample text the card box is filled with until the user types something. */
  let sampleText = "";

  const COPY_ICON = els.demoCopy.innerHTML;
  const DONE_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>';

  /**
   * Applies the chosen colour scheme. "system" hands back to the CSS
   * `prefers-color-scheme` rules.
   */
  function applyTheme(theme) {
    const root = document.documentElement;
    if (theme === "light" || theme === "dark") root.dataset.theme = theme;
    else delete root.dataset.theme;
  }

  function themeOr(theme) {
    return theme === "light" || theme === "dark" ? theme : "system";
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

  function fillLanguages(current) {
    const codes = LANGUAGES.slice();
    if (current && codes.indexOf(current) === -1) codes.unshift(current);
    els.targetLang.innerHTML = "";
    codes.forEach((code) => {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = Glossy.languageName(code);
      els.targetLang.appendChild(option);
    });
  }

  function syncProvider() {
    const provider = els.provider.value;
    els.apiIdField.hidden = provider !== "baidu";
    els.apiKeyField.hidden = PROVIDERS_WITH_KEY.indexOf(provider) === -1;
    els.providerHint.innerHTML = Glossy.i18n.t(PROVIDER_HINTS[provider] || "");
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
      ? Glossy.i18n.t("ignored.count", ignored.length)
      : Glossy.i18n.t("ignored.empty");
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

  /** Credentials the user saved earlier for the selected provider. */
  function rememberedCredentials(provider) {
    const map = (settings && settings.credentials) || {};
    return map[provider] || {};
  }

  function showCredentials(provider) {
    shownProvider = provider;
    const saved = rememberedCredentials(provider);
    els.apiId.value = saved.appId || "";
    els.apiKey.value = saved.apiKey || "";
  }

  /**
   * Copies what is typed in the credential fields into the entry of the
   * provider they belong to, so every provider keeps its own key. The fields
   * always show the provider picked last, which is why the key comes from
   * `shownProvider` and not from the dropdown.
   */
  function rememberCredentials() {
    if (!settings) return;
    const map = Object.assign({}, settings.credentials || {});
    const provider = shownProvider || els.provider.value;
    const entry = { appId: els.apiId.value.trim(), apiKey: els.apiKey.value.trim() };
    if (entry.appId || entry.apiKey) map[provider] = entry;
    else delete map[provider];
    settings.credentials = map;
  }

  /** Re-applies the interface language to everything the script writes itself. */
  function applyLanguage() {
    Glossy.i18n.apply(document);
    renderIgnored();
    relabelRunningApps();
    syncProvider();
    showHotkey(lastStatus);
    fillSample();
    syncDemoLanguage();
    if (settings) {
      fillLanguages(settings.targetLang);
      els.targetLang.value = settings.targetLang;
    }
  }

  /** Re-labels the translation card, and rebuilds the language bar with it. */
  function syncDemoLanguage() {
    els.demoCopy.setAttribute("aria-label", Glossy.i18n.t("popup.copy"));
    els.demoFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    els.demoTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    els.demoSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
    if (!els.demoLangbar.hidden) showLanguages(demoResult);
  }

  function apply(next) {
    settings = next;
    els.enabled.checked = !!next.enabled;
    els.triggerOnDrag.checked = !!next.triggerOnDrag;
    els.triggerOnDoubleClick.checked = !!next.triggerOnDoubleClick;
    els.restoreClipboard.checked = !!next.restoreClipboard;
    els.showOriginal.checked = !!next.showOriginal;
    els.minSelectionLen.value = String(numberOr(next.minSelectionLen, 2));
    ignored = normalizeIgnored(next.ignoredApps);
    els.hotkey.value = next.hotkey || "";
    els.theme.value = themeOr(next.theme);
    els.fontScale.value = String(pick(FONT_SCALES, next.fontScale, 100));
    els.popupWidth.value = String(pick(POPUP_SIZES, next.popupWidth, 356));
    els.popupOpacity.value = String(pick(OPACITIES, next.popupOpacity, 100));
    els.autoCloseSecs.value = String(pick(AUTO_CLOSE, next.autoCloseSecs, 0));
    els.closeAfterCopy.checked = !!next.closeAfterCopy;
    applyTheme(els.theme.value);
    fillLanguages(next.targetLang);
    els.targetLang.value = next.targetLang;
    els.provider.value = next.provider || "google";
    els.uiLang.value = next.uiLang === "zh" || next.uiLang === "en" ? next.uiLang : "system";
    Glossy.i18n.set(els.uiLang.value);
    showCredentials(els.provider.value);
    els.options.dataset.disabled = String(!next.enabled);
    applyLanguage();
  }

  function collect() {
    rememberCredentials();
    return {
      ...settings,
      enabled: els.enabled.checked,
      triggerOnDrag: els.triggerOnDrag.checked,
      triggerOnDoubleClick: els.triggerOnDoubleClick.checked,
      restoreClipboard: els.restoreClipboard.checked,
      showOriginal: els.showOriginal.checked,
      minSelectionLen: numberOr(els.minSelectionLen.value, 2),
      ignoredApps: ignored,
      hotkey: els.hotkey.value.trim(),
      theme: els.theme.value,
      fontScale: numberOr(els.fontScale.value, 100),
      popupWidth: numberOr(els.popupWidth.value, 356),
      popupOpacity: numberOr(els.popupOpacity.value, 100),
      autoCloseSecs: numberOr(els.autoCloseSecs.value, 0),
      closeAfterCopy: els.closeAfterCopy.checked,
      targetLang: els.targetLang.value,
      provider: els.provider.value,
      uiLang: els.uiLang.value,
    };
  }

  async function save() {
    try {
      apply(await Glossy.invoke("save_settings", { settings: collect() }));
      showToast(Glossy.i18n.t("toast.saved"));
      await refreshStatus();
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
    showHotkey(status);
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

  function showHotkey(status) {
    if (status && status.hotkey) {
      els.hotkeyHint.dataset.tone = "ok";
      els.hotkeyHint.textContent = Glossy.i18n.t("hotkey.active", status.hotkey);
    } else if (status && status.hotkeyError) {
      els.hotkeyHint.dataset.tone = "bad";
      els.hotkeyHint.textContent = status.hotkeyError;
    } else {
      delete els.hotkeyHint.dataset.tone;
      els.hotkeyHint.textContent = Glossy.i18n.t("hotkey.none");
    }
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
      Glossy.render.result(els.demoResult, result, { showOriginal: els.showOriginal.checked });
      if (result.sourceText) els.demoHeadword.textContent = String(result.sourceText);
      showLanguages(result);
    } catch (error) {
      if (mine !== demoTicket) return;
      Glossy.render.error(els.demoResult, Glossy.errorMessage(error), () => runDemo(text));
    }
  }

  async function copyDemo() {
    const value = demoResult && demoResult.translation ? String(demoResult.translation) : "";
    if (!value) return;
    await Glossy.invoke("copy_text", { text: value }).catch(() => false);
    els.demoCopy.innerHTML = DONE_ICON;
    els.demoCopy.classList.add("done");
    clearTimeout(demoCopyTimer);
    demoCopyTimer = setTimeout(() => {
      els.demoCopy.innerHTML = COPY_ICON;
      els.demoCopy.classList.remove("done");
    }, 1100);
  }

  /** Label of the entry that hands the source language over to the provider. */
  function autoLabel() {
    const name = Glossy.languageName(demoDetected);
    const label = Glossy.i18n.t("popup.autoDetected");
    return name ? `${label} · ${name}` : label;
  }

  /**
   * Rebuilds one language dropdown. A code outside of the shared list (a
   * detected language, or the target configured in the settings) is kept as an
   * extra entry so the current value always has a matching option.
   */
  function fillLanguageSelect(select, selected, withAuto) {
    const codes = LANGUAGES.slice();
    if (selected && selected !== AUTO && codes.indexOf(selected) === -1) codes.unshift(selected);

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
    select.value = selected || (withAuto ? AUTO : codes[0]);
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
  els.provider.addEventListener("change", () => {
    // Keep what the previous provider had before showing the credentials of the
    // newly selected one.
    rememberCredentials();
    showCredentials(els.provider.value);
    syncProvider();
    scheduleSave();
  });
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
  [
    els.targetLang,
    els.apiId,
    els.apiKey,
    els.restoreClipboard,
    els.showOriginal,
    els.minSelectionLen,
    els.hotkey,
    els.fontScale,
    els.popupWidth,
    els.popupOpacity,
    els.autoCloseSecs,
    els.closeAfterCopy,
  ].forEach((element) => {
    element.addEventListener("change", scheduleSave);
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
  els.demoFrom.addEventListener("change", () => {
    demoPair.source = els.demoFrom.value;
    demoDetected = "";
    runDemo(demoValue);
  });
  els.demoTo.addEventListener("change", () => {
    demoPair.target = els.demoTo.value;
    runDemo(demoValue);
  });
  els.demoSwap.addEventListener("click", swapDemo);
  els.demoCopy.addEventListener("click", copyDemo);

  els.demoPopup.addEventListener("click", () => {
    const value = selection || els.demoText.value.replace(/\s+/g, " ").trim();
    if (value.length < 2) return;
    Glossy.invoke("show_popup", { text: value }).catch(() => {});
  });

  // A pending change is written before the window goes away, whichever way it
  // goes away: hidden by the close button, minimized, or unloaded.
  Glossy.listen("glossy://flush-settings", flushSave);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) flushSave();
  });
  window.addEventListener("blur", flushSave);
  window.addEventListener("pagehide", flushSave);
  window.addEventListener("beforeunload", flushSave);

  (async function start() {
    try {
      apply(await Glossy.readSettings());
    } catch (error) {
      showToast(Glossy.i18n.t("toast.loadFailed") + Glossy.errorMessage(error));
    }
    await refreshStatus();
    Glossy.listen("glossy://status", refreshStatus);
    setInterval(refreshStatus, 4000);
    loadRunningApps();
  })();
})(window.Glossy);
