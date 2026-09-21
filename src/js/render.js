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
    while (target.firstChild) target.removeChild(target.firstChild);
  }

  function phoneticText(value) {
    const text = String(value).trim();
    if (!text) return "";
    return text.startsWith("/") || text.startsWith("[") ? text : `/${text}/`;
  }

  function loading(target) {
    clear(target);
    const wrap = node("div", "skeleton");
    wrap.setAttribute("aria-busy", "true");
    wrap.appendChild(node("span"));
    wrap.appendChild(node("span"));
    wrap.appendChild(node("span"));
    target.appendChild(wrap);
  }

  function error(target, message, onRetry) {
    clear(target);
    const wrap = node("div", "error");
    wrap.appendChild(
      node("div", "message", message || Glossy.i18n.t("render.failed")),
    );
    if (typeof onRetry === "function") {
      const retry = node("button", "ghost-button", Glossy.i18n.t("render.retry"));
      retry.type = "button";
      retry.addEventListener("click", onRetry);
      wrap.appendChild(retry);
    }
    target.appendChild(wrap);
  }

  function result(target, value, options) {
    const opts = options || {};
    const data = value || {};
    // The compact card keeps what the selection was translated into, and drops
    // the blocks that only add context.
    const extras = opts.compact !== true;
    clear(target);

    const translation = String(data.translation || "").trim();
    const isSentence = data.kind === "sentence";
    const empty = () => Glossy.i18n.t("render.empty");

    if (isSentence) {
      if (opts.showOriginal !== false && data.sourceText) {
        target.appendChild(node("div", "original", data.sourceText));
      }
      target.appendChild(node("div", "translation", translation || empty()));
      if (extras) pairs(target, data.pairs);
    } else {
      target.appendChild(node("div", "translation", translation || empty()));

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
    if (opts.speak !== false) readOut(target, data);

    const parts = [];
    if (data.provider) parts.push(Glossy.i18n.providerName(data.provider));
    if (parts.length) target.appendChild(node("div", "foot", parts.join(" · ")));

    // The card names the service that answered, and says so when that is not the
    // one the settings picked.
    if (data.fallbackFrom && data.provider) {
      target.appendChild(
        node(
          "div",
          "foot fallback",
          Glossy.i18n.t("render.fallback", Glossy.i18n.providerName(data.fallbackFrom)),
        ),
      );
    }
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

  /** The button that is currently reading something out loud, if any. */
  let reading = null;

  /** The pronunciation buttons: the original, the translation, or both. */
  function readOut(target, data) {
    const rows = [];
    const original = String(data.sourceText || "").trim();
    const translation = String(data.translation || "").trim();
    if (original) {
      rows.push({ key: "render.speakOriginal", text: original, language: data.sourceLang });
    }
    if (translation) {
      rows.push({
        key: "render.speakTranslation",
        text: translation,
        language: data.targetLang,
      });
    }
    if (!rows.length) return;

    const block = node("div", "says");
    rows.forEach((row) => {
      const button = node("button", "say", Glossy.i18n.t(row.key));
      button.type = "button";
      button.setAttribute("data-say", row.key);
      button.setAttribute("title", Glossy.i18n.t(row.key));
      button.addEventListener("click", () => toggleReading(button, row));
      block.appendChild(button);
    });
    target.appendChild(block);
  }

  /** Reads one text out loud, or stops the one already playing. */
  async function toggleReading(button, row) {
    if (typeof Glossy.invoke !== "function") return;
    const stop = reading === button;
    stopReading();
    if (stop) {
      Glossy.invoke("stop_speaking").catch(() => {});
      return;
    }

    reading = button;
    button.dataset.state = "on";
    button.textContent = Glossy.i18n.t("render.stop");
    try {
      await Glossy.invoke("say", {
        text: row.text,
        language: row.language || null,
      });
    } catch (error) {
      // A machine without a voice for that language answers with an error, and
      // there is nothing to show for it beyond the button going back.
      console.warn("glossy: unable to read the text out loud", error);
    }
    if (reading === button) stopReading();
  }

  /** Puts the button that is reading back to its resting look. */
  function stopReading() {
    if (!reading) return;
    reading.dataset.state = "off";
    reading.textContent = Glossy.i18n.t(reading.getAttribute("data-say"));
    reading = null;
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

  Glossy.languageName = languageName;
  Glossy.languageCodes = LANGUAGE_CODES;
  Glossy.errorMessage = function (error) {
    if (typeof error === "string") return error;
    if (error && typeof error.message === "string") return error.message;
    return Glossy.i18n.t("render.failed");
  };
  Glossy.render = { loading, error, result, clear };
})(window.Glossy);
