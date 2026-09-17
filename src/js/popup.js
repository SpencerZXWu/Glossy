/**
 * Floating popup window: renders the translation of the current selection,
 * follows the cursor and closes itself when the user clicks elsewhere.
 */
(function (Glossy) {
  const card = document.getElementById("card");
  const content = document.getElementById("content");
  const headword = document.getElementById("headword");
  const langbar = document.getElementById("langbar");
  const langFrom = document.getElementById("langFrom");
  const langTo = document.getElementById("langTo");
  const langSwap = document.getElementById("langSwap");
  const copyButton = document.getElementById("copy");
  const closeButton = document.getElementById("close");

  /** Source value that lets the provider detect the language itself. */
  const AUTO = "auto";

  const COPY_ICON = copyButton.innerHTML;
  const DONE_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>';
  /** Default `.card` width in CSS pixels, kept in sync with popup.css. */
  const CARD_WIDTH = 356;
  /** Transparent margin the body keeps around the card so its shadow shows. */
  const WINDOW_PADDING = 20;
  /** Screen space the card leaves free for the taskbar and the edges. */
  const CARD_MARGIN = 32;
  /** Used when the screen size cannot be read. */
  const FALLBACK_CARD_HEIGHT = 640;

  let text = "";
  let current = null;
  let preferences = {
    showOriginal: true,
    minSelectionLen: 2,
    theme: "system",
    fontScale: 100,
    popupWidth: CARD_WIDTH,
    popupOpacity: 100,
    autoCloseSecs: 0,
    closeAfterCopy: false,
    uiLang: "system",
  };
  let size = { width: 0, height: 0 };
  let ticket = 0;
  let copyTimer = 0;
  let closeTimer = 0;

  /** Pair shown in the language bar. `source: AUTO` asks the provider to detect
      the language and `target: null` follows the configured target. */
  let pair = { source: AUTO, target: null };
  /** Language the provider reported for the current result. */
  let detected = "";

  function cardWidth() {
    const width = Number(preferences.popupWidth);
    return Number.isFinite(width) && width > 0 ? width : CARD_WIDTH;
  }

  function measure() {
    const rect = card.getBoundingClientRect();
    return {
      width: cardWidth() + WINDOW_PADDING,
      height: Math.ceil(rect.height) + WINDOW_PADDING,
    };
  }

  /** Pushes the chosen interface language into the popup chrome. */
  function applyLanguage() {
    Glossy.i18n.set(preferences.uiLang);
    Glossy.i18n.apply(document);
    copyButton.setAttribute("aria-label", Glossy.i18n.t("popup.copy"));
    closeButton.setAttribute("aria-label", Glossy.i18n.t("popup.close"));
    langFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    langTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    langSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
  }

  /** Pushes the look-and-feel settings into the stylesheet. */
  function applyAppearance() {
    const root = document.documentElement;
    const theme = preferences.theme;
    if (theme === "light" || theme === "dark") root.dataset.theme = theme;
    else delete root.dataset.theme;

    const scale = Number(preferences.fontScale);
    root.style.setProperty("--popup-font", String((Number.isFinite(scale) && scale > 0 ? scale : 100) / 100));
    const opacity = Number(preferences.popupOpacity);
    root.style.setProperty(
      "--popup-opacity",
      String((Number.isFinite(opacity) ? Math.min(Math.max(opacity, 50), 100) : 100) / 100),
    );
    root.style.setProperty("--card-width", cardWidth() + "px");
    root.style.setProperty("--max-card-height", maxCardHeight() + "px");
  }

  /** Tallest the card may grow: as much of the screen as the popup can use. */
  function maxCardHeight() {
    const available = Number(window.screen && window.screen.availHeight);
    if (!Number.isFinite(available) || available < 200) return FALLBACK_CARD_HEIGHT;
    return Math.round(available - CARD_MARGIN);
  }

  function minimumLength() {
    const minimum = Number(preferences.minSelectionLen);
    return Number.isFinite(minimum) && minimum > 1 ? minimum : 2;
  }

  /** Restarts the "close by itself" countdown, if one is configured. */
  function scheduleAutoClose() {
    clearTimeout(closeTimer);
    const seconds = Number(preferences.autoCloseSecs);
    if (!Number.isFinite(seconds) || seconds <= 0) return;
    closeTimer = setTimeout(dismiss, Math.min(seconds, 600) * 1000);
  }

  /** Reports the measured card size to the backend, resizing and repositioning. */
  async function place(reveal) {
    const next = measure();
    if (!reveal && next.height === size.height && next.width === size.width) {
      size = next;
      return;
    }
    size = next;
    const command = reveal ? "popup_present" : "popup_resize";
    try {
      await Glossy.invoke(command, next);
    } catch (error) {
      console.warn("glossy: unable to place popup", error);
    }
  }

  /** Label of the entry that hands the source language over to the provider. */
  function autoLabel() {
    const name = Glossy.languageName(detected);
    const label = Glossy.i18n.t("popup.autoDetected");
    return name ? `${label} · ${name}` : label;
  }

  /**
   * Rebuilds one language dropdown. A code outside of the shared list (a
   * detected language, or the target configured in the settings) is kept as an
   * extra entry so the current value always has a matching option.
   */
  function fillLanguageSelect(select, selected, withAuto) {
    const codes = Glossy.languageCodes.slice();
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
    const target = pair.target || (result && result.targetLang) || "";
    if (!target) {
      langbar.hidden = true;
      return;
    }
    fillLanguageSelect(langFrom, pair.source, true);
    fillLanguageSelect(langTo, target, false);
    langbar.hidden = false;
  }

  /** Back to "detect the source and use the configured target". */
  function resetLanguages() {
    pair = { source: AUTO, target: null };
    detected = "";
  }

  /** Reverse-translates the card: the translation becomes the new selection. */
  function swapLanguages() {
    if (!current || !current.translation) return;
    const value = String(current.translation).trim();
    if (value.length < minimumLength()) return;

    const source = pair.source === AUTO ? current.sourceLang : pair.source;
    const target = pair.target || current.targetLang;
    // The text of the next run is the old translation, so the pair is reversed.
    // A source the provider could not name stays on "detect it".
    pair = { source: target || AUTO, target: source || null };
    detected = "";
    run(value);
  }

  async function run(selection) {
    const value = String(selection || "").trim();
    if (value.length < minimumLength()) return;

    text = value;
    current = null;
    const mine = ++ticket;
    clearTimeout(closeTimer);

    headword.textContent = value;
    langbar.hidden = true;
    document.body.dataset.state = "loading";
    Glossy.render.loading(content);
    size = { width: 0, height: 0 };
    await place(true);
    if (mine !== ticket) return;

    try {
      const result = await Glossy.invoke("translate_text", {
        text: value,
        sourceLang: pair.source === AUTO ? null : pair.source,
        targetLang: pair.target,
      });
      if (mine !== ticket) return;

      current = result;
      detected = result.sourceLang || "";
      document.body.dataset.state = result.kind === "sentence" ? "sentence" : "word";
      Glossy.render.result(content, result, { showOriginal: preferences.showOriginal });
      if (result.sourceText) headword.textContent = String(result.sourceText);

      showLanguages(result);
      await place(false);
      scheduleAutoClose();
    } catch (error) {
      if (mine !== ticket) return;
      document.body.dataset.state = "error";
      Glossy.render.error(content, Glossy.errorMessage(error), () => run(text));
      await place(false);
      scheduleAutoClose();
    }
  }

  async function copyResult() {
    const value = current && current.translation ? String(current.translation) : "";
    if (!value) return;
    await Glossy.invoke("copy_text", { text: value }).catch(() => false);
    copyButton.innerHTML = DONE_ICON;
    copyButton.classList.add("done");
    clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copyButton.innerHTML = COPY_ICON;
      copyButton.classList.remove("done");
    }, 1100);
    if (preferences.closeAfterCopy) {
      clearTimeout(closeTimer);
      closeTimer = setTimeout(dismiss, 420);
    }
  }

  function dismiss() {
    ticket += 1;
    clearTimeout(closeTimer);
    document.body.dataset.state = "idle";
    Glossy.invoke("popup_close").catch(() => {});
  }

  copyButton.addEventListener("click", copyResult);
  closeButton.addEventListener("click", dismiss);
  langFrom.addEventListener("change", () => {
    pair.source = langFrom.value;
    detected = "";
    run(text);
  });
  langTo.addEventListener("change", () => {
    pair.target = langTo.value;
    run(text);
  });
  langSwap.addEventListener("click", swapLanguages);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") dismiss();
  });

  Glossy.listen("glossy://selection", (event) => {
    resetLanguages();
    run(event && event.payload ? event.payload.text : "");
  });

  Glossy.listen("glossy://settings", (event) => {
    preferences = { ...preferences, ...(event.payload || {}) };
    applyAppearance();
    applyLanguage();
    // The labels of the language bar are translated too.
    if (!langbar.hidden) showLanguages(current);
    // A different width or text size also changes the card size.
    const state = document.body.dataset.state;
    if (state && state !== "idle") place(false);
  });

  /** Renders a sample card when the page is opened outside of Tauri. */
  function preview() {
    const sample = new URLSearchParams(location.search).get("sample") || "word";
    document.body.dataset.state = sample;

    if (sample === "loading") {
      Glossy.render.loading(content);
      return;
    }
    if (sample === "error") {
      Glossy.render.error(content, "Translation failed: the provider is unreachable.", () => run("running"));
      return;
    }

    const value =
      sample === "sentence"
        ? "The quick brown fox jumps over the lazy dog, then it keeps on running."
        : "running";
    headword.textContent = value;
    Glossy.invoke("translate_text", { text: value }).then((result) => {
      current = result;
      detected = result.sourceLang || "";
      Glossy.render.result(content, result, { showOriginal: preferences.showOriginal });
      showLanguages(result);
    });
  }

  (async function start() {
    try {
      const settings = await Glossy.readSettings();
      if (settings) preferences = { ...preferences, ...settings };
    } catch (error) {
      console.warn("glossy: cannot read settings", error);
    }
    applyAppearance();
    applyLanguage();
    if (!Glossy.live) preview();
  })();
})(window.Glossy);
