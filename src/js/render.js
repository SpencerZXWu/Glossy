/**
 * Shared result renderers for the popup window and the in-app demo pane.
 */
(function (Glossy) {
  /** Language codes offered as a target, in menu order. */
  const LANGUAGE_CODES = [
    "en",
    "zh-CN",
    "zh-TW",
    "ja",
    "ko",
    "fr",
    "de",
    "es",
    "pt",
    "it",
    "ru",
    "uk",
    "nl",
    "pl",
    "tr",
    "ar",
    "hi",
    "th",
    "vi",
    "id",
    "ms",
    "cs",
    "da",
    "fi",
    "el",
    "he",
    "hu",
    "no",
    "ro",
    "sk",
    "sv",
  ];

  const LANGUAGES = {
    auto: "Auto detect",
    en: "English",
    zh: "Chinese",
    "zh-CN": "Chinese (Simplified)",
    "zh-TW": "Chinese (Traditional)",
    ja: "Japanese",
    ko: "Korean",
    fr: "French",
    de: "German",
    es: "Spanish",
    pt: "Portuguese",
    it: "Italian",
    ru: "Russian",
    uk: "Ukrainian",
    nl: "Dutch",
    pl: "Polish",
    tr: "Turkish",
    ar: "Arabic",
    hi: "Hindi",
    th: "Thai",
    vi: "Vietnamese",
    id: "Indonesian",
    ms: "Malay",
    cs: "Czech",
    da: "Danish",
    fi: "Finnish",
    el: "Greek",
    he: "Hebrew",
    hu: "Hungarian",
    no: "Norwegian",
    ro: "Romanian",
    sk: "Slovak",
    sv: "Swedish",
  };

  function languageName(code) {
    if (!code) return "";
    const value = String(code);
    return Glossy.i18n.languageName(value, englishName);
  }

  /** English spelling, used when the interface is not in Chinese. */
  function englishName(code) {
    const value = String(code);
    const exact = lookup(value);
    if (exact) return exact;
    const base = value.split("-")[0].toLowerCase();
    const name = lookup(base);
    if (name) {
      const suffix = value.slice(base.length + 1).toUpperCase();
      return suffix ? `${name} (${suffix})` : name;
    }
    return value.toUpperCase();
  }

  /** Looks a language up ignoring case, since providers spell codes differently. */
  function lookup(code) {
    if (LANGUAGES[code]) return LANGUAGES[code];
    const needle = String(code).toLowerCase();
    const match = Object.keys(LANGUAGES).find((key) => key.toLowerCase() === needle);
    return match ? LANGUAGES[match] : "";
  }

  function node(tag, className, text) {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (text !== undefined && text !== null) element.textContent = text;
    return element;
  }

  function clear(target) {
    // A card that is being replaced must not keep talking behind the new one,
    // and must not leave a tick behind on a button nobody can see.
    if (reading) stopSpeaking();
    if (copied) resetCopy();
    // The card that held the button is going away with it.
    overwriteButton = null;
    while (target.firstChild) target.removeChild(target.firstChild);
  }

  function phoneticText(value) {
    const text = String(value).trim();
    if (!text) return "";
    return text.startsWith("/") || text.startsWith("[") ? text : `/${text}/`;
  }

  /**
   * The card while an answer is on its way. It keeps the bottom row of a
   * finished one, so the engine can be changed without waiting for the first
   * answer: the name is the service that was asked, not the one that answered,
   * which is not known yet.
   */
  function loading(target, options) {
    const opts = options || {};
    clear(target);
    const wrap = node("div", "skeleton");
    wrap.setAttribute("aria-busy", "true");
    wrap.appendChild(node("span"));
    wrap.appendChild(node("span"));
    wrap.appendChild(node("span"));
    // A wait that has nothing to do with translating, such as the one the first
    // screenshot spends on the recognition engine, says what it is waiting for.
    if (opts.message) wrap.appendChild(node("div", "note", opts.message));
    target.appendChild(wrap);
    foot(target, opts.provider, opts);
  }

  /**
   * Puts the name of an engine at the left of the bottom row. It is a button when
   * the caller offers a way to choose another one, plain text otherwise.
   */
  function engineName(row, provider, options) {
    if (!provider) return;
    const opts = options || {};
    const name = Glossy.i18n.providerName(provider);
    if (typeof opts.onChooseService === "function") {
      const choose = node("button", "service", name);
      choose.type = "button";
      choose.setAttribute("aria-haspopup", "true");
      choose.setAttribute("aria-expanded", "false");
      choose.setAttribute("title", Glossy.i18n.t("popup.chooseService"));
      choose.addEventListener("click", (event) => {
        if (event && typeof event.stopPropagation === "function") event.stopPropagation();
        opts.onChooseService(choose);
      });
      row.appendChild(choose);
    } else {
      row.appendChild(node("span", "engine", name));
    }
  }

  /** The bottom row of the card, added only when it has something to hold. */
  function foot(target, provider, options) {
    const opts = options || {};
    const row = node("div", "foot");
    engineName(row, provider, opts);
    // The row names the engine that answered; the mark of the app that asked it
    // closes the line. Without an engine to name there is no row at all.
    if (!row.childNodes.length) return row;
    const steps = historyNav(opts);
    if (steps) row.appendChild(steps);
    row.appendChild(brand());
    target.appendChild(row);
    return row;
  }

  /**
   * The pair of arrows that walk the translations the history holds, drawn in
   * the blank at the right of the name of the engine.
   *
   * They are only drawn when the caller knows how to move — a card in the demo
   * pane, or one in the settings window, has no list behind it — and both start
   * disabled, because which way a card can go is only known once the list has
   * been asked: the caller settles them the moment the answer is in.
   */
  function historyNav(options) {
    const opts = options || {};
    if (typeof opts.onHistory !== "function") return null;
    const group = node("div", "history-nav");
    group.appendChild(historyStep(-1, BACK_GLYPH, "popup.historyPrev", opts.onHistory));
    group.appendChild(historyStep(1, FORWARD_GLYPH, "popup.historyNext", opts.onHistory));
    return group;
  }

  /** One arrow of the pair: a button that asks the caller to step that way. */
  function historyStep(direction, glyph, key, onHistory) {
    const step = node("button", "history-step");
    step.type = "button";
    step.dataset.step = String(direction);
    step.disabled = true;
    step.innerHTML = glyph;
    name(step, key);
    step.addEventListener("click", (event) => {
      if (event && typeof event.stopPropagation === "function") event.stopPropagation();
      onHistory(direction);
    });
    return step;
  }

  /** The translucent Glossy mark at the right of the row that names the engine. */
  function brand() {
    const mark = node("span", "brand");
    const logo = node("img", "brand-logo");
    logo.src = "images/logo.png";
    logo.alt = "";
    mark.appendChild(logo);
    mark.appendChild(node("span", "brand-name", "Glossy"));
    return mark;
  }

  /**
   * The grey block holding the text that was translated. It sits under the
   * language row on every card, word cards included, and it is editable when
   * the caller offers a way to translate again: the text here is what the answer
   * is an answer to, so correcting it costs no more than one more translation.
   */
  function originalBlock(target, value, options) {
    const opts = options || {};
    const committedAtStart = String(value);
    const block = node("div", "original", committedAtStart);
    let committed = committedAtStart;

    if (typeof opts.onRetranslate === "function") {
      block.setAttribute("contenteditable", "plaintext-only");
      block.setAttribute("spellcheck", "false");
      block.setAttribute("role", "textbox");
      block.setAttribute("aria-label", Glossy.i18n.t("render.originalEdit"));
      block.setAttribute("title", Glossy.i18n.t("render.originalHint"));

      const read = () => String(block.innerText || block.textContent || "").trim();

      // The one place an edit becomes a translation. It ignores text that is
      // empty or unchanged, so asking twice for the same text — a Ctrl+Enter
      // followed by the blur it causes — costs one translation, not two.
      const commit = () => {
        const edited = read();
        if (!edited || edited === committed) {
          block.textContent = committed;
          return;
        }
        committed = edited;
        block.textContent = edited;
        opts.onRetranslate(edited);
      };

      block.addEventListener("keydown", (event) => {
        if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
          // Enter alone still breaks a line; with the modifier it translates.
          event.preventDefault();
          commit();
          block.blur();
          return;
        }
        if (event.key === "Escape") {
          // Escape belongs to the edit while the caret is in it: the text goes
          // back to what was translated instead of the card being closed.
          event.preventDefault();
          event.stopPropagation();
          block.textContent = committed;
          block.blur();
        }
      });

      // Leaving the field translates what is in it, so a correction is never
      // quietly dropped when the reader clicks elsewhere.
      block.addEventListener("blur", commit);

      // Ctrl+Enter belongs to the field while the caret is in it, and to the
      // card while it is not: the card's own is registered with Windows, so it
      // reaches the card before the field the key was pressed in. The field
      // therefore says when it holds the caret, and the card lets go of the
      // accelerator for as long as it does.
      if (typeof opts.onEditing === "function") {
        block.addEventListener("focus", () => opts.onEditing(true));
        block.addEventListener("blur", () => opts.onEditing(false));
      }
    }

    target.appendChild(block);
    return block;
  }

  /**
   * Draws what went wrong.
   *
   * The bottom row is the same one a card that answered wears, because a
   * failure is exactly when a reader wants another engine: the one that just
   * refused is named there and, when the caller offers a way to choose, the
   * name opens the list. Leaving the row off the failure card would make the
   * only way out the settings window, which is not where the reader is.
   */
  function error(target, message, onRetry, options) {
    clear(target);
    const opts = options || {};
    const wrap = node("div", "error");
    const detail = String(message || "").trim();
    wrap.appendChild(node("div", "message", Glossy.i18n.t("render.failed")));
    // The failure text from the backend is written in English and names things
    // like sockets; the reader gets their own language as the headline with the
    // original underneath, where it is useful for a bug report and harmless
    // otherwise.
    if (detail && detail !== Glossy.i18n.t("render.failed")) {
      wrap.appendChild(node("div", "note", detail));
    }
    if (typeof onRetry === "function") {
      const retry = node("button", "ghost-button", Glossy.i18n.t("render.retry"));
      retry.type = "button";
      retry.addEventListener("click", onRetry);
      wrap.appendChild(retry);
    }
    target.appendChild(wrap);
    foot(target, opts.provider, opts);
  }

  function result(target, value, options) {
    const opts = options || {};
    const data = value || {};
    // The compact card keeps what the selection was translated into, and drops
    // the blocks that only add context.
    const extras = opts.compact !== true;
    clear(target);

    const translation = String(data.translation || "").trim();
    const sourceText = String(data.sourceText || "").trim();
    const isSentence = data.kind === "sentence";
    // An empty answer is a message about the result, not the result: it gets its
    // own styling so it does not sit in the card with the weight of a translation.
    const answer = () =>
      node(
        "div",
        translation ? "translation" : "translation empty",
        translation || Glossy.i18n.t("render.empty"),
      );

    // The two sides of the card, each with the text it holds and the language
    // that text is read out loud in. A word that was looked up stands in the
    // header above the body, so its buttons are the first thing the body holds.
    const original = sourceText
      ? { key: "Original", text: sourceText, language: data.sourceLang }
      : null;
    const answerSide = translation
      ? { key: "Translation", text: translation, language: data.targetLang }
      : null;

    // Both shapes of card open with the grey original, under the row that picks
    // the languages: it is what the translation is of, and the one place a
    // wrongly selected or misspelled source can be corrected.
    if (original && opts.showOriginal !== false) {
      originalBlock(target, original.text, opts);
    }

    if (isSentence) {
      if (original) sideTools(target, original, opts);
      target.appendChild(answer());
      if (answerSide) sideTools(target, answerSide, opts);
      if (extras) pairs(target, data.pairs);
    } else {
      if (original) sideTools(target, original, opts);
      target.appendChild(answer());
      if (answerSide) sideTools(target, answerSide, opts);

      const phonetic = phoneticText(data.phonetic || "");
      if (phonetic) {
        target.appendChild(node("div", "phonetic", phonetic));
      }

      const meanings = Array.isArray(data.meanings) ? data.meanings : [];
      let shownMeanings = false;
      if (meanings.length) {
        const list = node("div", "meanings");
        meanings.forEach((meaning) => {
          const definitions = (meaning.definitions || []).filter(Boolean);
          if (!definitions.length) return;
          const row = node("div", "meaning");
          row.appendChild(node("span", "pos", meaning.partOfSpeech || "def"));
          row.appendChild(node("span", "defs", definitions.join(" · ")));
          list.appendChild(row);
        });
        if (list.childNodes.length) {
          target.appendChild(list);
          shownMeanings = true;
        }
      }

      if (data.example && extras) {
        target.appendChild(node("div", "example", String(data.example)));
      }

      if (extras) {
        forms(target, data.forms);
        synonyms(target, data.synonyms);
        context(target, data.context);
      }

      // The dictionary lookups a word card needs answer seconds later, so the
      // card says that they are running instead of looking finished.
      if (opts.pending && !phonetic && !data.example && !shownMeanings) {
        const waiting = node("div", "pending", Glossy.i18n.t("render.lookup"));
        waiting.setAttribute("aria-live", "polite");
        target.appendChild(waiting);
      }
    }

    if (extras) units(target, data.conversions);

    // The bottom line of the card: the name of the engine that answered, with
    // the Glossy mark at the far right.
    foot(target, data.provider, opts);

    // The line under the engine names the one that did *not* answer, which is
    // the service the settings asked for: the engine in the footer is the one
    // that answered, and a line that named it again would say nothing. The
    // reason is told when the relay walked past it for a reason it named.
    if (data.fallbackFrom && data.provider) {
      const failed = Glossy.i18n.providerName(data.fallbackFrom);
      const reason = reasonText(data.fallbackCode);
      const line = reason
        ? Glossy.i18n.t("render.fallbackWhy", failed, reason)
        : Glossy.i18n.t("render.fallback", failed);
      target.appendChild(node("div", "foot fallback", line));
    }

    // The relay can tell the App something about itself as well as translate:
    // that this build is older than the one it serves. It goes last, so the
    // translation keeps the top of the card.
    updateLine(target, data.notice, opts);
  }

  /**
   * The line a relay that has moved on puts under a card from before it moved.
   *
   * Only `update_available` is drawn here. Every other code is the caller's to
   * deal with — an announcement belongs in the corner of the screen rather than
   * on top of somebody's translation — and a code this build has never heard of
   * is not drawn at all, because there is nothing here to draw it with.
   */
  function updateLine(target, notice, options) {
    if (!notice || String(notice.code || "") !== "update_available") return;
    const opts = options || {};
    const line = node("div", "notice update");
    line.setAttribute("role", "status");
    const version = String(notice.minVersion || "").trim();
    const text = version
      ? Glossy.i18n.t("render.updateAvailable", version)
      : Glossy.i18n.t("render.updateAvailableAny");
    line.appendChild(node("span", "notice-text", text));
    if (typeof opts.onUpdate === "function") {
      const go = node("button", "notice-go", Glossy.i18n.t("render.updateAction"));
      go.type = "button";
      go.addEventListener("click", (event) => {
        if (event && typeof event.stopPropagation === "function") event.stopPropagation();
        opts.onUpdate();
      });
      line.appendChild(go);
    }
    target.appendChild(line);
  }

  /**
   * What a vendor's own refusal code says, in the reader's language.
   *
   * A code this build does not know says nothing here: the line already says
   * that the engine did not answer, and "it did not say why" after it would only
   * repeat that. The code itself still reaches `glossy.log`.
   */
  function reasonText(code) {
    // The server writes its codes with underscores (`upstream_limit`); the
    // dictionary spells its keys without them.
    const name = String(code || "")
      .trim()
      .toLowerCase()
      .replace(/_([a-z0-9])/g, (_, letter) => letter.toUpperCase());
    if (!name) return "";
    const key = `reason.${name}`;
    const text = Glossy.i18n.t(key);
    return text === key ? "" : text;
  }

  /** The inflections of a word, labelled by the tag the backend wrote. */
  function forms(target, value) {
    const rows = (Array.isArray(value) ? value : []).filter(
      (form) => form && String(form.text || "").trim(),
    );
    if (!rows.length) return;

    const block = node("div", "forms");
    block.appendChild(node("div", "block-title", Glossy.i18n.t("render.forms")));
    rows.forEach((form) => {
      const row = node("div", "form");
      row.appendChild(
        node("span", "form-tag", Glossy.i18n.t(`form.${form.tag || "other"}`)),
      );
      row.appendChild(node("span", "form-text", String(form.text)));
      block.appendChild(row);
    });
    target.appendChild(block);
  }

  /** Words that mean roughly the same, in the language of the original. */
  function synonyms(target, value) {
    const rows = (Array.isArray(value) ? value : []).filter((word) =>
      String(word || "").trim(),
    );
    if (!rows.length) return;

    const block = node("div", "synonyms");
    block.appendChild(node("div", "block-title", Glossy.i18n.t("render.synonyms")));
    block.appendChild(node("div", "synonym-list", rows.join(" · ")));
    target.appendChild(block);
  }

  /** The sentence a word was selected from, next to its own translation. */
  function context(target, value) {
    const source = String((value && value.text) || "").trim();
    if (!source) return;

    const block = node("div", "context");
    block.appendChild(node("div", "block-title", Glossy.i18n.t("render.context")));
    block.appendChild(node("div", "context-source", source));
    const translation = String((value && value.translation) || "").trim();
    if (translation) block.appendChild(node("div", "context-translation", translation));
    target.appendChild(block);
  }

  /** The original and the translation, one sentence per row. */
  function pairs(target, value) {
    const rows = (Array.isArray(value) ? value : []).filter(
      (pair) => pair && (String(pair.source || "").trim() || String(pair.translation || "").trim()),
    );
    // A translation that divides differently comes back as one row, which is
    // the whole text again and says nothing the card does not say already.
    if (rows.length < 2) return;

    const block = node("div", "pairs");
    block.appendChild(node("div", "block-title", Glossy.i18n.t("render.pairs")));
    rows.forEach((pair) => {
      const row = node("div", "pair");
      row.appendChild(node("div", "pair-source", String(pair.source || "")));
      row.appendChild(node("div", "pair-translation", String(pair.translation || "")));
      block.appendChild(row);
    });
    target.appendChild(block);
  }

  /**
   * The two faces of a pronunciation button: a speaker at rest, and the stop
   * square it wears while its text is being read.
   */
  const SPEAK_GLYPH =
    '<svg class="glyph glyph-speak" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M11 4.5 6 8.5H3v7h3l5 4v-15Z" /><path d="M15.5 9.5a4 4 0 0 1 0 5" /><path d="M18.5 7a8 8 0 0 1 0 10" /></svg>';
  const STOP_GLYPH =
    '<svg class="glyph glyph-stop" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="7.5" y="7.5" width="9" height="9" rx="1.5" /></svg>';

  /**
   * The two faces of a copy button: the two sheets at rest, and the tick it
   * wears for a moment after the text reached the clipboard.
   */
  const COPY_GLYPH =
    '<svg class="glyph glyph-copy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2.5" /><path d="M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1" /></svg>';
  const DONE_GLYPH =
    '<svg class="glyph glyph-done" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6L9 17l-5-5" /></svg>';
  /** The two arrows of the button that writes the translation back over the
      original, which is the one thing it does. */
  const REPLACE_GLYPH =
    '<svg class="glyph glyph-replace" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 8h12.5l-3-3" /><path d="M20 16H7.5l3 3" /></svg>';

  /** The two arrows that walk the translations the history holds. */
  const BACK_GLYPH =
    '<svg class="glyph glyph-back" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14.5 6 8.5 12l6 6" /></svg>';
  const FORWARD_GLYPH =
    '<svg class="glyph glyph-forward" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9.5 6l6 6-6 6" /></svg>';

  /** How often the card asks whether the voice is still reading. */
  const SPEECH_POLL_MS = 250;

  /** How long a reading that failed shows on its button before going back. */
  const FAILED_HOLD_MS = 2600;

  /** How long the tick of a copied text stays on its button. */
  const COPIED_HOLD_MS = 1100;

  /** The button that is currently reading something out loud, if any. */
  let reading = null;

  /** The timer that waits for the voice to fall silent, while one is reading. */
  let watching = null;

  /** The button that copied last, and the timer that puts it back to rest. */
  let copied = null;
  let copiedTimer = null;
  /** The card's own button for writing a translation back over its original. */
  let overwriteButton = null;

  /**
   * The buttons of one side of the card: the one that copies that text and the
   * one that reads it out loud. Both sit under the text they act on, so which
   * side each of them belongs to is never something the reader has to remember.
   *
   * `side` is `{ key: "Original" | "Translation", text, language }`; the key
   * names the two labels of the side in the dictionary.
   */
  function sideTools(target, side, options) {
    const opts = options || {};
    const row = node("div", "tools");

    const copy = node("button", "tool");
    copy.type = "button";
    copy.setAttribute("data-copy", `render.copy${side.key}`);
    copy.innerHTML = `${COPY_GLYPH}${DONE_GLYPH}`;
    name(copy, copy.getAttribute("data-copy"));
    copy.addEventListener("click", () => copySide(copy, side, opts));
    row.appendChild(copy);

    if (opts.speak !== false) {
      const say = node("button", "tool");
      say.type = "button";
      say.setAttribute("data-say", `render.speak${side.key}`);
      say.setAttribute("aria-pressed", "false");
      say.innerHTML = `${SPEAK_GLYPH}${STOP_GLYPH}`;
      name(say, say.getAttribute("data-say"));
      say.addEventListener("click", () => toggleReading(say, side));
      row.appendChild(say);
    }

    // Only the translation has an original to take the place of, and it only
    // has one when the card is the answer to a selection: a card that is only
    // being looked at has nothing behind it to write into.
    if (opts.overwrite === true && side.key === "Translation") {
      const write = node("button", "tool");
      write.type = "button";
      write.setAttribute("data-overwrite", "render.overwrite");
      write.innerHTML = `${REPLACE_GLYPH}${DONE_GLYPH}`;
      name(write, write.getAttribute("data-overwrite"));
      write.addEventListener("click", () => overwrite(side.text, write));
      row.appendChild(write);
      overwriteButton = write;
    }

    target.appendChild(row);
  }

  /**
   * Writes `text` over the selection it was translated from.
   *
   * The writing itself belongs to the backend, which owns the clipboard and the
   * paste; the card only names the text and shows the answer. `button` is the
   * one that asked, when one did: the accelerator arrives without a button, so
   * the card's own is used, and it may be gone along with the card.
   */
  function overwrite(text, button) {
    if (typeof Glossy.invoke !== "function" || !text) return;
    const target = button || overwriteButton;
    Glossy.invoke("replace_selection", { text })
      .then((written) => report(target, written !== false))
      .catch(() => report(target, false));
  }

  /** Says on a button how a write back ended, for as long as a copy is shown. */
  function report(button, ok) {
    if (!button) return;
    if (copied && copied !== button) resetCopy();
    copied = button;
    button.dataset.state = ok ? "copied" : "failed";
    name(button, ok ? "render.overwritten" : "render.overwriteFailed");
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(resetCopy, COPIED_HOLD_MS);
  }

  /** Sends one side of the card to the clipboard and says so on its button. */
  function copySide(button, side, options) {
    if (typeof Glossy.invoke !== "function") return;
    Glossy.invoke("copy_text", { text: side.text }).catch(() => false);
    announceCopy(button);
    if (typeof options.onCopied === "function") options.onCopied(side.key.toLowerCase());
  }

  /** Shows the tick on the button that just copied, and takes it back later. */
  function announceCopy(button) {
    if (copied && copied !== button) resetCopy();
    copied = button;
    button.dataset.state = "copied";
    name(button, "render.copied");
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(resetCopy, COPIED_HOLD_MS);
  }

  /** Puts the button that copied back to its resting look. */
  function resetCopy() {
    clearTimeout(copiedTimer);
    copiedTimer = null;
    if (copied) {
      copied.dataset.state = "off";
      const resting =
        copied.getAttribute("data-copy") || copied.getAttribute("data-overwrite");
      name(copied, resting);
    }
    copied = null;
  }

  /** Names a button after an i18n key, for its tooltip and its screen reader. */
  function name(button, key) {
    const text = Glossy.i18n.t(key);
    button.setAttribute("title", text);
    button.setAttribute("aria-label", text);
  }

  /** Reads one text out loud, or stops the one that is already being read. */
  function toggleReading(button, row) {
    if (typeof Glossy.invoke !== "function") return;
    if (reading === button) {
      stopSpeaking();
      return;
    }
    stopSpeaking();
    reading = button;
    button.dataset.state = "on";
    button.setAttribute("aria-pressed", "true");
    name(button, "render.stop");
    watch();
    // The voice says the words on a thread of its own, so this promise answers
    // long before the reading is over; whether it is still going is asked
    // separately, in `watch`.
    Glossy.invoke("say", { text: row.text, language: row.language || null }).catch((error) => {
      console.warn("glossy: unable to read the text out loud", error);
      if (reading === button) failReading(button);
    });
  }

  /** Follows a reading until the voice reports that it has fallen silent. */
  function watch() {
    if (watching !== null) return;
    watching = setInterval(async () => {
      let busy;
      try {
        busy = await Glossy.invoke("speaking");
      } catch (error) {
        // A backend that will not answer is not one to keep asking.
        console.warn("glossy: unable to ask whether the voice is busy", error);
        stopWatching();
        return;
      }
      if (!busy) stopReading();
    }, SPEECH_POLL_MS);
  }

  function stopWatching() {
    if (watching === null) return;
    clearInterval(watching);
    watching = null;
  }

  /** Puts the button that was reading back to its resting look. */
  function stopReading() {
    if (reading) {
      reading.dataset.state = "off";
      reading.setAttribute("aria-pressed", "false");
      name(reading, reading.getAttribute("data-say"));
    }
    reading = null;
    stopWatching();
  }

  /** Says on the button that its text could not be read, then goes back. */
  function failReading(button) {
    stopReading();
    button.dataset.state = "failed";
    name(button, "render.speakFailed");
    setTimeout(() => {
      if (button.dataset.state !== "failed") return;
      button.dataset.state = "off";
      name(button, button.getAttribute("data-say"));
    }, FAILED_HOLD_MS);
  }

  /** Stops whatever is being read out loud and puts the buttons back. */
  function stopSpeaking() {
    stopReading();
    if (typeof Glossy.invoke !== "function") return;
    Glossy.invoke("stop_speaking").catch(() => {});
  }

  /** The source of a live currency rate, spelled the way the interface does. */
  function rateSource(name) {
    if (name === "exchangerate-api.com") return Glossy.i18n.t("units.source.exchangerateApi");
    if (name === "frankfurter.app") return Glossy.i18n.t("units.source.frankfurter");
    return String(name);
  }

  /** One line of unit conversions, or nothing when the result carries none. */
  function units(target, value) {
    const conversions = Array.isArray(value) ? value : [];
    const rows = conversions.filter(
      (item) => item && (item.original || item.converted),
    );
    if (!rows.length) return;

    const block = node("div", "units");
    block.appendChild(node("div", "units-title", Glossy.i18n.t("units.title")));
    let previousNote = "";
    rows.forEach((item) => {
      const row = node("div", "unit");
      row.appendChild(node("span", "unit-from", String(item.original || "")));
      row.appendChild(node("span", "unit-arrow", Glossy.i18n.t("units.approx")));
      row.appendChild(node("span", "unit-to", String(item.converted || "")));
      block.appendChild(row);
      if (item.rate) {
        block.appendChild(node("div", "unit-rate", String(item.rate)));
      }
      // Every amount in one card shares a single exchange rate, so the note
      // under the first row is the note for all of them.
      const note = rateNote(item);
      if (note && note !== previousNote) {
        previousNote = note;
        block.appendChild(node("div", "unit-note", note));
      }
    });
    target.appendChild(block);
  }

  function rateNote(item) {
    if (!item.rateSource && !item.rateDate) return "";
    const label = item.stale
      ? Glossy.i18n.t("units.stale")
      : Glossy.i18n.t("units.rate");
    return [label, rateSource(item.rateSource || ""), item.rateDate || ""]
      .filter(Boolean)
      .join(" · ");
  }

  /**
   * Languages each engine offers, as the backend reported them.
   *
   * The tables are read once at start-up (`service_languages`); until they
   * arrive every list is offered in full, so a backend that cannot answer costs
   * the filtering and nothing else.
   */
  let serviceLanguages = {};

  /** Records the per-engine language tables the backend sent. */
  function setServiceLanguages(list) {
    const table = {};
    (Array.isArray(list) ? list : []).forEach((entry) => {
      if (!entry || !entry.id) return;
      const codes = (Array.isArray(entry.languages) ? entry.languages : [])
        .map((code) => String(code))
        .filter(Boolean);
      // An engine the backend knows no language for keeps the full list, which
      // is the same answer as an engine it never mentioned.
      if (codes.length) table[String(entry.id)] = codes;
    });
    serviceLanguages = table;
  }

  /** Codes of the shared list the engine offers, in menu order. */
  function languagesFor(service) {
    const codes = serviceLanguages[service];
    if (!codes) return LANGUAGE_CODES.slice();
    return LANGUAGE_CODES.filter((code) => codes.indexOf(code) !== -1);
  }

  /** Whether the engine takes one language code; unknown engines take any. */
  function servesLanguage(service, code) {
    const codes = serviceLanguages[service];
    if (!codes) return true;
    return codes.indexOf(String(code)) !== -1;
  }

  Glossy.languageName = languageName;
  Glossy.languageCodes = LANGUAGE_CODES;
  Glossy.setServiceLanguages = setServiceLanguages;
  Glossy.languagesFor = languagesFor;
  Glossy.servesLanguage = servesLanguage;
  Glossy.errorMessage = function (error) {
    if (typeof error === "string") return error;
    if (error && typeof error.message === "string") return error.message;
    return Glossy.i18n.t("render.failed");
  };
  Glossy.render = { loading, error, result, clear, stopSpeaking, overwrite };
})(window.Glossy);
