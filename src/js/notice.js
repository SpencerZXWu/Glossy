/**
 * The card that announces a background start. It closes on its own, or when the
 * user clicks it (which opens the settings window) or its close button.
 *
 * The countdown that hides it is kept by the backend, which is the only side
 * that knows when the card is shown again: this document is loaded once and the
 * window is then reused for every later hint.
 */
(function (Glossy) {
  const card = document.getElementById("toast");
  const close = document.getElementById("close");

  function hide(command) {
    Glossy.invoke(command).catch((error) => console.warn("glossy: notice", error));
  }

  function applyLanguage(uiLang) {
    Glossy.i18n.set(uiLang);
    Glossy.i18n.apply(document);
  }

  /** Follows the colour scheme picked in the settings. */
  function applyTheme(theme) {
    GlossyTheme.apply({ theme });
  }

  close.addEventListener("click", (event) => {
    event.stopPropagation();
    hide("notice_close");
  });

  card.addEventListener("click", () => hide("notice_open"));

  (async function start() {
    try {
      const settings = await Glossy.readSettings();
      applyLanguage(settings.uiLang);
      applyTheme(settings.theme);
    } catch (error) {
      console.warn("glossy: cannot read settings", error);
      applyLanguage("system");
    }
  })();
})(window.Glossy);
