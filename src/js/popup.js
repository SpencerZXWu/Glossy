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
  const pinButton = document.getElementById("pin");
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
    compactPopup: false,
    uiLang: "system",
  };
  let size = { width: 0, height: 0 };
  /** Usable height in CSS pixels of the monitor the popup currently sits on.
      Reported by the backend, which knows where the window really landed. */
  let availableHeight = null;
  /** Height cap currently written into the stylesheet. */
  let screenCap = 0;
  let ticket = 0;
  let copyTimer = 0;
  let closeTimer = 0;
  /** True while the card is pinned: clicks elsewhere leave it alone and the
      "close by itself" countdown stands still. */
  let pinned = false;

  /** Pair shown in the language bar. `source: AUTO` asks the provider to detect
      the language and `target: null` follows the configured target. */
  let pair = { source: AUTO, target: null };
  /** Language the provider reported for the current result. */
  let detected = "";
  /** Engine that translates right now; the language bar offers its languages. */
  let service = "";

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
    pinButton.setAttribute("aria-label", pinLabel());
    langFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    langTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    langSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
  }

  /** Pushes the look-and-feel settings into the stylesheet. */
  function applyAppearance() {
    const root = document.documentElement;

    GlossyTheme.apply({ theme: preferences.theme });

    const scale = Number(preferences.fontScale);
    root.style.setProperty("--popup-font", String((Number.isFinite(scale) && scale > 0 ? scale : 100) / 100));
    const opacity = Number(preferences.popupOpacity);
    root.style.setProperty(
      "--popup-opacity",
      String((Number.isFinite(opacity) ? Math.min(Math.max(opacity, 50), 100) : 100) / 100),
    );
    applyMetrics();
  }

  /**
   * Writes the size limits into the stylesheet. The cap depends on the monitor
   * the popup is on, so it is re-applied once the backend reports where the
   * window was placed. Returns true when the cap actually changed.
   */
  function applyMetrics() {
    const cap = maxCardHeight();
    const changed = cap !== screenCap;
    screenCap = cap;
    document.documentElement.style.setProperty("--card-width", cardWidth() + "px");
    document.documentElement.style.setProperty("--max-card-height", cap + "px");
    return changed;
  }

  /** Tallest the card may grow: as much of the screen as the popup can use. */
  function maxCardHeight() {
    const available =
      Number.isFinite(availableHeight) && availableHeight > 0
        ? availableHeight
        : Number(window.screen && window.screen.availHeight);
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
    if (pinned) return;
    const seconds = Number(preferences.autoCloseSecs);
    if (!Number.isFinite(seconds) || seconds <= 0) return;
    closeTimer = setTimeout(dismiss, Math.min(seconds, 600) * 1000);
  }

  function pinLabel() {
    return Glossy.i18n.t(pinned ? "popup.unpin" : "popup.pin");
  }

  /** Writes the pin state into the button that shows it. */
  function showPin() {
    pinButton.dataset.state = pinned ? "on" : "off";
    pinButton.setAttribute("aria-pressed", pinned ? "true" : "false");
    pinButton.setAttribute("data-i18n-title", pinned ? "popup.unpin" : "popup.pin");
    pinButton.title = pinLabel();
    pinButton.setAttribute("aria-label", pinButton.title);
  }

  /**
   * Pins or unpins the card. The backend has to know as well: the click that
   * would dismiss the card is caught by the mouse hook, not by this window.
   */
  function setPinned(next) {
    if (next === pinned) return;
    pinned = next;
    showPin();
    Glossy.invoke("popup_set_pinned", { pinned }).catch(() => {});
    if (pinned) clearTimeout(closeTimer);
    else scheduleAutoClose();
  }

  /**
   * Reports the measured card size to the backend, which resizes and
   * repositions the window and answers with the height the monitor it landed on
   * offers. The card is capped to that height, so the limit always belongs to
   * the screen the popup is really on.
   */
  async function place(reveal) {
    const next = measure();
    if (!reveal && next.height === size.height && next.width === size.width) {
      size = next;
      return;
    }
    size = next;
    const command = reveal ? "popup_present" : "popup_resize";
    let available = null;
    try {
      available = await Glossy.invoke(command, next);
    } catch (error) {
      console.warn("glossy: unable to place popup", error);
    }

    // The window may have moved to another monitor, which changes how tall the
    // card is allowed to be. That is only known now, so re-measure once and
    // resize when the cap changed.
    if (Number.isFinite(available) && available > 0) availableHeight = available;
    if (!applyMetrics()) return;
    size = measure();
    try {
      await Glossy.invoke("popup_resize", size);
    } catch (error) {
      console.warn("glossy: unable to re-measure popup", error);
    }
  }

  /** Label of the entry that hands the source language over to the provider. */
  function autoLabel() {
    const name = Glossy.languageName(detected);
    const label = Glossy.i18n.t("popup.autoDetected");
    return name ? `${label} · ${name}` : label;
  }

  /**
   * The entries an engine can offer, plus the wanted code when the shared list
   * never had it: a detected language comes from the provider rather than from
   * the menu, so it stays selectable instead of leaving the bar blank.
   *
   * A language of the menu that the engine does not translate is left out, so
   * the bar only offers what the translation through this engine can be.
   */
  function entries(selected) {
    const codes = Glossy.languagesFor(service);
    if (
      selected &&
      selected !== AUTO &&
      codes.indexOf(selected) === -1 &&
      Glossy.languageCodes.indexOf(selected) === -1
    ) {
      codes.unshift(selected);
    }
    return codes;
  }

  /**
   * Rebuilds one language dropdown, and settles on the wanted value when the
   * engine still offers it: otherwise on "detect it" (the source bar) or on the
   * first language the engine does translate (the target bar).
   */
  function fillLanguageSelect(select, selected, withAuto) {
    const codes = entries(selected);

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
    // Redrawing the card throws the list away with it, so it is forgotten here
    // rather than removed.
    serviceMenu = null;
    const mine = ++ticket;
    clearTimeout(closeTimer);

    headword.textContent = value;
    langbar.hidden = true;
    document.body.dataset.state = "loading";
    Glossy.render.loading(content);
    size = { width: 0, height: 0 };
    // A pinned card stays where it is; only a fresh, unpinned one follows the
    // cursor to the new selection.
    await place(!pinned);
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
      draw(result);
      if (result.sourceText) headword.textContent = String(result.sourceText);

      showLanguages(result);
      await place(false);
      scheduleAutoClose();
      refine(result, mine);
    } catch (error) {
      if (mine !== ticket) return;
      document.body.dataset.state = "error";
      Glossy.render.error(content, Glossy.errorMessage(error), () => run(text));
      await place(false);
      scheduleAutoClose();
    }
  }

  /** Whether a word card is still missing something the lookups can add. */
  function needsDetails(result) {
    return !result.phonetic || !(result.meanings || []).length || !result.example;
  }

  /** Grows a word card into the full entry once the lookups have answered.
   *
   * The translation comes from the provider and is on screen already; phonetic
   * symbols, meanings and an example need the dictionary and the free endpoint,
   * which are slow and sometimes have nothing. The card therefore says that the
   * lookups are running and updates in place when they answer — staying a bare
   * translation when they have nothing to add.
   *
   * Whatever the provider already returned stays on screen while that happens:
   * emptying the card and filling it again made the whole entry slide up and
   * down twice for a single selection. */
  async function refine(result, mine) {
    if (!result || result.kind !== "word" || !needsDetails(result)) return;

    Glossy.render.result(content, result, {
      showOriginal: preferences.showOriginal,
      compact: preferences.compactPopup === true,
      pending: true,
    });
    await place(false);

    let details = null;
    try {
      details = await Glossy.invoke("word_details", {
        text,
        sourceLang: pair.source === AUTO ? null : pair.source,
        targetLang: result.targetLang || null,
      });
    } catch (error) {
      details = null;
    }
    // A newer selection, or a card the user closed, has taken over.
    if (mine !== ticket || !current) return;

    current = {
      ...current,
      phonetic: current.phonetic || (details && details.phonetic) || null,
      meanings: (current.meanings || []).length
        ? current.meanings
        : (details && details.meanings) || [],
      example: current.example || (details && details.example) || null,
      synonyms: (current.synonyms || []).length
        ? current.synonyms
        : (details && details.synonyms) || [],
      forms: (current.forms || []).length ? current.forms : (details && details.forms) || [],
      context: current.context || (details && details.context) || null,
    };
    // Drawn either way: the card has to lose its placeholder even when the
    // lookups came back with nothing.
    draw(current);
    await place(false);
  }

  /** Draws the card the way the reading and compactness settings ask for. */
  function draw(result) {
    Glossy.render.result(content, result, {
      showOriginal: preferences.showOriginal,
      compact: preferences.compactPopup === true,
      onChooseService: toggleServiceMenu,
    });
  }

  /**
   * The engines the card can switch between, in the order the settings window
   * lists them. The stored choice is spread over four fields, so the backend is
   * asked which entry it adds up to instead of deriving it again here.
   */
  const SERVICES = ["cloud-baidu", "cloud-youdao", "google"];

  /** The service list while it is open, and the name it was opened from. */
  let serviceMenu = null;

  /**
   * Opens the list of engines under the name of the one that answered, or
   * closes it again when the name is clicked twice.
   *
   * The list is drawn inside the card rather than floating over it: the card is
   * resized to whatever it holds, so growing it is a layout the window already
   * knows how to place, and the list can never end up off the screen.
   */
  async function toggleServiceMenu(button) {
    if (serviceMenu) {
      closeServiceMenu();
      return;
    }
    let chosen = "";
    try {
      chosen = String((await Glossy.invoke("current_service")) || "");
    } catch (error) {
      chosen = "";
    }
    // The answer took a round trip; a click elsewhere may have closed the card.
    if (document.body.dataset.state === "idle") return;

    const list = document.createElement("div");
    list.className = "service-menu";
    list.setAttribute("role", "menu");
    SERVICES.forEach((id) => {
      const item = document.createElement("button");
      item.type = "button";
      item.className = "service-item";
      item.setAttribute("role", "menuitemradio");
      item.setAttribute("aria-checked", String(id === chosen));
      const name = document.createElement("span");
      name.className = "service-name";
      name.textContent = Glossy.i18n.t(`service.${id}`);
      item.appendChild(name);
      if (id === chosen) {
        const mark = document.createElement("span");
        mark.className = "service-chosen";
        mark.textContent = "✓";
        item.appendChild(mark);
      }
      item.addEventListener("click", (event) => {
        if (event && typeof event.stopPropagation === "function") event.stopPropagation();
        chooseService(id);
      });
      list.appendChild(item);
    });

    button.setAttribute("aria-expanded", "true");
    content.appendChild(list);
    serviceMenu = { list, button };
    await place(false);
  }

  /** Puts the card back the way it was before the list was opened. */
  function closeServiceMenu() {
    if (!serviceMenu) return;
    const { list, button } = serviceMenu;
    serviceMenu = null;
    button.setAttribute("aria-expanded", "false");
    if (list.parentNode) list.parentNode.removeChild(list);
    place(false);
  }

  /** Asks the backend which engine translates, and which languages it takes. */
  async function loadService() {
    try {
      service = String((await Glossy.invoke("current_service")) || "");
    } catch (error) {
      service = "";
    }
  }

  /** Reads the per-engine language tables once; the language bar filters with
      them, and goes on offering everything if they cannot be read. */
  async function loadLanguages() {
    try {
      Glossy.setServiceLanguages(await Glossy.invoke("service_languages"));
    } catch (error) {
      console.warn("glossy: cannot read the language tables", error);
    }
  }

  /** Switches the engine and translates the same text through it. */
  async function chooseService(id) {
    closeServiceMenu();
    try {
      await Glossy.invoke("set_service", { id });
    } catch (error) {
      // The card keeps the translation it already has; the settings window is
      // where the choice is explained, and nothing here can improve on that.
      console.warn("glossy: could not switch the translation service", error);
      return;
    }
    service = id;
    // A language the engine just left cannot stand in the bar any more, so the
    // next run goes back to the configured pair instead of asking for it.
    if (unserved(pair.source)) pair.source = AUTO;
    if (unserved(pair.target)) pair.target = null;
    if (text) run(text);
  }

  /** Whether the chosen engine refuses a language the bar is holding. */
  function unserved(code) {
    return !!code && code !== AUTO && !Glossy.servesLanguage(service, code);
  }

  /**
   * Remembers the language the card is translated into, so the next selection
   * starts from it rather than from the target configured in the settings.
   */
  function rememberTarget(code) {
    if (!code || code === AUTO) return;
    Glossy.invoke("set_target_lang", { code }).catch((error) => {
      console.warn("glossy: could not remember the target language", error);
    });
  }

  /** Shows a translation the history already has, without asking the provider
      again: the stored card keeps its phonetic symbols, meanings and example. */
  async function showStored(result) {
    if (!result || !result.translation) return;
    const mine = ++ticket;
    clearTimeout(closeTimer);
    serviceMenu = null;
    resetLanguages();

    text = String(result.sourceText || "");
    current = result;
    detected = result.sourceLang || "";
    headword.textContent = text;
    document.body.dataset.state = result.kind === "sentence" ? "sentence" : "word";
    draw(result);
    showLanguages(result);

    size = { width: 0, height: 0 };
    await place(!pinned);
    if (mine !== ticket) return;
    scheduleAutoClose();
    // An entry stored before the details were looked up grows into the full
    // card here as well.
    refine(result, mine);
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
    // The card is about to be hidden with whatever is on it.
    serviceMenu = null;
    // Unpinning here keeps the button in step with the card that is about to
    // vanish; the backend forgets the pin with the card.
    pinned = false;
    showPin();
    // A card that is gone must not keep talking.
    Glossy.render.stopSpeaking();
    document.body.dataset.state = "idle";
    Glossy.invoke("popup_close").catch(() => {});
  }

  copyButton.addEventListener("click", copyResult);
  pinButton.addEventListener("click", () => setPinned(!pinned));
  closeButton.addEventListener("click", dismiss);
  langFrom.addEventListener("change", () => {
    pair.source = langFrom.value;
    detected = "";
    run(text);
  });
  langTo.addEventListener("change", () => {
    pair.target = langTo.value;
    rememberTarget(pair.target);
    run(text);
  });
  langSwap.addEventListener("click", swapLanguages);
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    // Escape belongs to the innermost thing that is open: the list closes
    // first, and the card goes on the next press.
    if (serviceMenu) closeServiceMenu();
    else dismiss();
  });
  // A click anywhere else on the card means the list is not wanted; the click
  // that opens it is stopped from reaching here by the name it started on, and
  // the list's own items are handled before it too.
  document.addEventListener("click", (event) => {
    if (!serviceMenu) return;
    const target = event.target;
    if (target === serviceMenu.button) return;
    if (target && typeof serviceMenu.list.contains === "function") {
      if (serviceMenu.list.contains(target)) return;
    }
    closeServiceMenu();
  });

  Glossy.listen("glossy://selection", (event) => {
    resetLanguages();
    run(event && event.payload ? event.payload.text : "");
  });

  Glossy.listen("glossy://result", (event) => {
    showStored(event && event.payload);
  });

  Glossy.listen("glossy://settings", async (event) => {
    preferences = { ...preferences, ...(event.payload || {}) };
    applyAppearance();
    applyLanguage();
    // The engine may have been switched in the settings window, and another
    // engine offers other languages; the labels of the bar are translated too.
    await loadService();
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
    // The sample stands in for a real selection, so the card that switches
    // services has the text to translate again.
    text = value;
    headword.textContent = value;
    Glossy.invoke("translate_text", { text: value }).then((result) => {
      current = result;
      detected = result.sourceLang || "";
      draw(result);
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
    // The engine and its languages are known before the card is filled, so the
    // bar of the first result already leaves out what the engine cannot do.
    await loadLanguages();
    await loadService();
    applyAppearance();
    applyLanguage();
    showPin();
    if (!Glossy.live) preview();
  })();
})(window.Glossy);
