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
  const pinButton = document.getElementById("pin");
  const starButton = document.getElementById("star");
  const shotButton = document.getElementById("shot");
  const settingsButton = document.getElementById("settings");
  const badge = document.getElementById("badge");

  /** Source value that lets the provider detect the language itself. */
  const AUTO = "auto";

  /** Default `.card` width in CSS pixels, kept in sync with popup.css. */
  const CARD_WIDTH = 356;
  /** Transparent margin the body keeps around the card so its shadow shows. */
  const WINDOW_PADDING = 20;
  /** Same margin around the badge, which keeps its own, smaller one (2×8px). */
  const BADGE_PADDING = 16;
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
  let closeTimer = 0;
  /** True while the card is pinned: clicks elsewhere leave it alone and the
      "close by itself" countdown stands still. */
  let pinned = false;
  /** True while the card on screen is in the wordbook. */
  let starred = false;
  /** True while the card answers a selection that is still in place behind it,
      which is what a translation can be written back over. A card showing an
      old translation from the history has nothing behind it. */
  let canOverwrite = false;

  /**
   * Records whether the card on screen can be written back over, and tells the
   * backend so: Ctrl+Enter belongs to the card only while there is a translation
   * of a selection still in place to write back, and it has to go back to the
   * program in front the moment there is not.
   */
  function showWriteBack(value) {
    canOverwrite = value;
    Glossy.invoke("popup_set_replace", { armed: value }).catch(() => {});
  }
  /** The selection the badge is holding, waiting for its click. */
  let waiting = "";

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

  /**
   * Resolves once the layout just written has reached the screen.
   *
   * Two frames are waited out because the first one only starts the paint the
   * placement asked for; the second one runs after it, which is what "the popup
   * is on screen" means to whoever is measuring the road from the selection.
   */
  function painted() {
    return new Promise((resolve) => {
      requestAnimationFrame(() => requestAnimationFrame(resolve));
    });
  }

  function measure() {
    // The badge is the whole window while it is up, and it is not the card, so
    // it is measured instead of it.
    if (document.body.dataset.state === "badge") {
      const icon = badge.getBoundingClientRect();
      return {
        width: Math.ceil(icon.width) + BADGE_PADDING,
        height: Math.ceil(icon.height) + BADGE_PADDING,
      };
    }
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
    settingsButton.setAttribute("aria-label", Glossy.i18n.t("popup.settings"));
    badge.setAttribute("aria-label", Glossy.i18n.t("popup.translate"));
    pinButton.setAttribute("aria-label", pinLabel());
    langFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    langTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    langSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
    nameWithShortcut(shotButton, "popup.shot", preferences.hotkeyOcr);
    nameWithShortcut(settingsButton, "popup.settings", preferences.hotkeySettings);
  }

  /**
   * Names a header button after what it does, with the shortcut that does the
   * same thing in brackets.
   *
   * The combination is read from the settings on every pass rather than written
   * into the markup, because the user is the one who records it: a tooltip has to
   * name the key that works now, and a field the user cleared means no key at all.
   */
  function nameWithShortcut(button, key, shortcut) {
    const label = Glossy.i18n.withShortcut(Glossy.i18n.t(key), shortcut);
    button.title = label;
    button.setAttribute("aria-label", label);
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

  /** Writes the wordbook state into the star that shows it. */
  function showStar() {
    starButton.dataset.state = starred ? "on" : "off";
    starButton.setAttribute("aria-pressed", starred ? "true" : "false");
    starButton.setAttribute("data-i18n-title", starred ? "popup.unstar" : "popup.star");
    starButton.title = Glossy.i18n.t(starred ? "popup.unstar" : "popup.star");
    starButton.setAttribute("aria-label", starButton.title);
  }

  /**
   * Asks whether the card on screen is kept already, and shows the answer on
   * its star. The question is asked every time a card is drawn, because a card
   * is drawn again when the lookups fill it in, and it belongs to that card: the
   * answer is dropped when a newer card has taken over.
   */
  async function readStar(result) {
    const mine = ticket;
    let kept = false;
    if (result && result.sourceText) {
      kept = await Glossy.invoke("vocabulary_keeps", {
        sourceText: String(result.sourceText),
        targetLang: String(result.targetLang || ""),
      }).catch(() => false);
    }
    if (mine !== ticket) return;
    starred = kept === true;
    showStar();
  }

  /** Keeps the card in the wordbook, or takes it back out of it. */
  async function toggleStar() {
    if (!current) return;
    let kept = starred;
    try {
      kept = await Glossy.invoke("vocabulary_toggle", { result: current });
    } catch (error) {
      // The star keeps the state it had; nothing was written.
      return;
    }
    starred = kept === true;
    showStar();
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

  /**
   * Shows the badge for a fresh selection instead of the card. Nothing is
   * translated — and nothing is billed — until the badge is clicked, so this is
   * also the state the daily allowance is still untouched in.
   */
  async function showBadge(selection) {
    const value = String(selection || "").trim();
    if (value.length < minimumLength()) return;

    waiting = value;
    ticket += 1;
    clearTimeout(closeTimer);
    // A badge belongs to a new selection and has to answer to a click anywhere
    // else; a pin left over from the card before would swallow that click.
    pinned = false;
    showPin();
    Glossy.invoke("popup_set_pinned", { pinned: false }).catch(() => {});
    serviceMenu = null;
    current = null;
    text = value;
    // A badge holds a selection, but it is not a translation yet.
    showWriteBack(false);
    Glossy.render.stopSpeaking();
    document.body.dataset.state = "badge";
    size = { width: 0, height: 0 };
    await place(true);
    await painted();
    // Free unless the run was started with `GLOSSY_TIMING`.
    Glossy.invoke("popup_painted").catch(() => {});
  }

  async function run(selection) {
    const value = String(selection || "").trim();
    if (value.length < minimumLength()) return;

    text = value;
    current = null;
    // The card is the answer to a selection the program in front still has,
    // so the translation can take its place.
    showWriteBack(true);
    // Redrawing the card throws the list away with it, so it is forgotten here
    // rather than removed.
    serviceMenu = null;
    const mine = ++ticket;
    clearTimeout(closeTimer);

    headword.textContent = value;
    langbar.hidden = true;
    document.body.dataset.state = "loading";
    // The card is the shape of the answer that is coming, and the engine can be
    // changed from it while it is on its way.
    Glossy.render.loading(content, {
      provider: SERVICE_ENGINES[service],
      onChooseService: toggleServiceMenu,
    });
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
      // The card that held the list is gone with the loading state.
      serviceMenu = null;
      document.body.dataset.state = "error";
      // The engine that refused stays on the card, so another one can be picked
      // from it and the same text asked again — which is what a reader wants
      // after a failure, rather than a trip to the settings window.
      Glossy.render.error(content, Glossy.errorMessage(error), () => run(text), {
        provider: SERVICE_ENGINES[service],
        onChooseService: toggleServiceMenu,
      });
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
      overwrite: canOverwrite,
      pending: true,
      onChooseService: toggleServiceMenu,
      onCopied: cardCopied,
      onRetranslate: run,
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
    // The list goes with the card that held it.
    serviceMenu = null;
    Glossy.render.result(content, result, {
      showOriginal: preferences.showOriginal,
      compact: preferences.compactPopup === true,
      overwrite: canOverwrite,
      onChooseService: toggleServiceMenu,
      onCopied: cardCopied,
      onRetranslate: run,
    });
    readStar(result);
  }

  /**
   * The engines the card can switch between, in the order the settings window
   * lists them. The stored choice is spread over four fields, so the backend is
   * asked which entry it adds up to instead of deriving it again here.
   */
  const SERVICES = ["cloud-baidu", "cloud-youdao", "google", "offline"];

  /** The name each of them answers under, as the backend spells it. */
  const SERVICE_ENGINES = {
    "cloud-baidu": "baidu",
    "cloud-youdao": "youdao",
    google: "google",
    offline: "offline",
  };

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
    keyboardMenu(list, button);
    await place(false);
  }

  /** The arrows, Home, End and Tab of an open menu.
   *
   * The list is a `menu` of `menuitemradio`s, so the keys are the ones that
   * pattern has: the arrows walk it, Home and End go to its ends, Tab leaves it
   * and Escape - handled with the card's own Escape - closes it. A menu that
   * opens without saying where the keyboard is would make the arrows a guess,
   * so the item the engine is currently on takes the focus.
   */
  function keyboardMenu(list, button) {
    const items = Array.from(list.children);
    if (!items.length) return;
    const at = (index) => {
      const item = items[(index + items.length) % items.length];
      if (item) item.focus();
    };
    const chosen = items.findIndex((item) => item.getAttribute("aria-checked") === "true");
    at(chosen === -1 ? 0 : chosen);

    list.addEventListener("keydown", (event) => {
      const here = items.indexOf(document.activeElement);
      if (event.key === "ArrowDown") {
        event.preventDefault();
        at(here + 1);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        at(here < 0 ? items.length - 1 : here - 1);
      } else if (event.key === "Home") {
        event.preventDefault();
        at(0);
      } else if (event.key === "End") {
        event.preventDefault();
        at(items.length - 1);
      } else if (event.key === "Tab") {
        // Tab belongs to the card, not to the list: the list closes and the
        // caret stays on the button it was opened from.
        closeServiceMenu();
      }
    });
  }

  /** Puts the card back the way it was before the list was opened. */
  function closeServiceMenu() {
    if (!serviceMenu) return;
    const { list, button } = serviceMenu;
    serviceMenu = null;
    button.setAttribute("aria-expanded", "false");
    // A caret inside a list that is about to be removed would go with it, and
    // the tab stop it was on is the button that opened the list.
    const held = list.contains(document.activeElement);
    if (list.parentNode) list.parentNode.removeChild(list);
    if (held) button.focus();
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
    // An old translation has no selection behind it to be written over.
    showWriteBack(false);
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

  /** Called when one side of the card reaches the clipboard: the preference
      asks for the card to close itself after a copy. */
  function cardCopied() {
    if (!preferences.closeAfterCopy) return;
    clearTimeout(closeTimer);
    closeTimer = setTimeout(dismiss, 420);
  }

  function dismiss() {
    ticket += 1;
    clearTimeout(closeTimer);
    // The card is about to be hidden with whatever is on it.
    serviceMenu = null;
    waiting = "";
    // Unpinning here keeps the button in step with the card that is about to
    // vanish; the backend forgets the pin with the card.
    pinned = false;
    showPin();
    // A card that is gone must not keep talking.
    Glossy.render.stopSpeaking();
    document.body.dataset.state = "idle";
    Glossy.invoke("popup_close").catch(() => {});
  }

  pinButton.addEventListener("click", () => setPinned(!pinned));
  starButton.addEventListener("click", toggleStar);
  // The badge is the only thing that starts a translation: the selection alone
  // only puts it on screen.
  badge.addEventListener("click", () => {
    const value = waiting;
    waiting = "";
    run(value);
  });
  settingsButton.addEventListener("click", () => {
    // A pinned card was asked to stay where it is, so it outlives the errand
    // the settings button sends the user on.
    if (!pinned) dismiss();
    Glossy.invoke("open_settings").catch(() => {});
  });
  // The screenshot is taken by the backend, which owns the overlay the rectangle
  // is drawn on. It puts the card away first — the card is what is in the way of
  // the thing the user wants to read — and the answer comes back to it.
  shotButton.addEventListener("click", () => {
    Glossy.render.stopSpeaking();
    Glossy.invoke("ocr_start_from_card").catch(() => {});
  });
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
  // The card is shown without the keyboard so that clicking the badge leaves the
  // text selected in the program behind it alone; the space the original is
  // written in is editable, though, and the rest of the card answers the
  // keyboard too, so the focus going into it is what asks for the front.
  //
  // It has to be a keyboard focus and not a click on a button: giving the card
  // the keyboard takes it away from the program the selection came from, and
  // most of them drop the selection the moment that happens. `:focus-visible`
  // is the difference - the browser matches it for the caret going into a text
  // field and for a Tab, and not for a click on a button.
  document.addEventListener("focusin", (event) => {
    const field = event.target;
    if (!field || typeof field.matches !== "function") return;
    if (field.matches(":focus-visible")) Glossy.invoke("popup_take_focus").catch(() => {});
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
    const payload = (event && event.payload) || {};
    const value = payload.text;
    // A selection on its own only puts the badge on screen; the click is what
    // buys a translation. A card that is pinned, and the shortcut, which the
    // user presses on purpose, translate straight away.
    if (payload.immediate === true || pinned) run(value);
    else showBadge(value);
  });

  Glossy.listen("glossy://result", (event) => {
    showStored(event && event.payload);
  });

  // Ctrl+Enter writes the translation back over the text it came from. The
  // card holds the translation, so the accelerator only says that the key was
  // pressed, and the card answers with the same write its own button makes.
  Glossy.listen("glossy://replace", () => {
    if (!canOverwrite || !current || !current.translation) return;
    Glossy.render.overwrite(String(current.translation));
  });

  // The first screenshot has to download the recognition engine before it can
  // read anything, so the card stands where the translation will appear and says
  // what is being waited for.
  Glossy.listen("glossy://popup-wait", async (event) => {
    // The card goes up the moment a rectangle is accepted, before anything is
    // read: what is coming is either a first-use download of the text reader or
    // the recognition itself.
    const waitingFor = String((event && event.payload) || "");
    resetLanguages();
    clearTimeout(closeTimer);
    Glossy.render.stopSpeaking();
    current = null;
    text = "";
    showWriteBack(false);
    serviceMenu = null;
    ticket += 1;
    headword.textContent = "";
    langbar.hidden = true;
    document.body.dataset.state = "loading";
    if (waitingFor === "engine") {
      Glossy.render.loading(content, { message: Glossy.i18n.t("popup.engineWait") });
    } else {
      // Reading the picture: the card is the shape of the answer that is
      // coming, exactly as it is while a selection is being translated.
      Glossy.render.loading(content, {
        provider: SERVICE_ENGINES[service],
        onChooseService: toggleServiceMenu,
      });
    }
    size = { width: 0, height: 0 };
    await place(true);
  });

  // A screenshot that could not be read has no text to translate again, so its
  // card only says what went wrong — with the engine named under it, because a
  // screenshot that was refused is often refused by the engine's allowance and
  // another one answers it.
  Glossy.listen("glossy://popup-error", async (event) => {
    resetLanguages();
    clearTimeout(closeTimer);
    Glossy.render.stopSpeaking();
    current = null;
    text = "";
    document.body.dataset.state = "error";
    Glossy.render.error(content, String((event && event.payload) || ""), null, {
      provider: SERVICE_ENGINES[service],
      onChooseService: toggleServiceMenu,
    });
    size = { width: 0, height: 0 };
    await place(true);
    scheduleAutoClose();
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

    if (sample === "badge") {
      // What a selection looks like before it is clicked: the icon alone.
      showBadge("running");
      return;
    }
    if (sample === "loading") {
      Glossy.render.loading(content, {
        provider: SERVICE_ENGINES[service],
        onChooseService: toggleServiceMenu,
      });
      return;
    }
    if (sample === "engine") {
      // The wait before the first screenshot, while the recognition engine is
      // being fetched.
      document.body.dataset.state = "loading";
      langbar.hidden = true;
      Glossy.render.loading(content, { message: Glossy.i18n.t("popup.engineWait") });
      return;
    }
    if (sample === "error") {
      Glossy.render.error(content, "Translation failed: the provider is unreachable.", () => run("running"), {
        provider: SERVICE_ENGINES[service],
        onChooseService: toggleServiceMenu,
      });
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
    showStar();
    if (!Glossy.live) preview();
  })();
})(window.Glossy);
