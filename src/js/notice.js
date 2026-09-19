/**
 * The card that announces a background start. It closes on its own, or when the
 * user clicks it (which opens the settings window) or its close button.
 */
(function (Glossy) {
  /** How long the card stays on screen before it hides itself. */
  const LINGER_MS = 6500;

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
    // The card is shown again on the next start, so the timer restarts with the
    // freshly loaded document rather than being reset from the backend.
    setTimeout(() => hide("notice_close"), LINGER_MS);
  })();
})(window.Glossy);
