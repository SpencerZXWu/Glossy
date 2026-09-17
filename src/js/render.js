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
    if (LANGUAGES[value]) return LANGUAGES[value];
    const base = value.split("-")[0].toLowerCase();
    const suffix = value.slice(base.length + 1).toUpperCase();
    if (LANGUAGES[base]) return suffix ? `${LANGUAGES[base]} (${suffix})` : LANGUAGES[base];
    return value.toUpperCase();
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
    clear(target);

    const translation = String(data.translation || "").trim();
    const isSentence = data.kind === "sentence";
    const empty = () => Glossy.i18n.t("render.empty");

    if (isSentence) {
      if (opts.showOriginal !== false && data.sourceText) {
        target.appendChild(node("div", "original", data.sourceText));
      }
      target.appendChild(node("div", "translation", translation || empty()));
    } else {
      target.appendChild(node("div", "translation", translation || empty()));

      if (data.phonetic) {
        target.appendChild(node("div", "phonetic", phoneticText(data.phonetic)));
      }

      const meanings = Array.isArray(data.meanings) ? data.meanings : [];
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
        if (list.childNodes.length) target.appendChild(list);
      }

      if (data.example) {
        target.appendChild(node("div", "example", String(data.example)));
      }
    }

    const parts = [];
    if (data.provider) parts.push(Glossy.i18n.providerName(data.provider));
    if (parts.length) target.appendChild(node("div", "foot", parts.join(" · ")));
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
