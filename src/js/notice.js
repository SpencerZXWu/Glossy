/**
 * The card in the corner of the screen. It closes on its own, or when the user
 * clicks it or its close button.
 *
 * The window carries two sentences, never both: the hint that says Glossy came
 * up in the background, and a line the relay asked every App to pass on. Which
 * one is on it is kept by the backend, which is the only side that knows when
 * the card is shown again — this document is loaded once and the window is then
 * reused for every later show, so the line is asked for rather than pushed at a
 * page that may not be listening yet.
 */
(function (Glossy) {
  const card = document.getElementById("toast");
  const close = document.getElementById("close");
  const body = card.querySelector(".text span");

  /** The line the relay asked for, or empty while the card is the start hint. */
  let line = "";

  function hide(command) {
    Glossy.invoke(command).catch((error) => console.warn("glossy: notice", error));
  }

  /** Draws whichever of the two sentences this card is showing. */
  function draw() {
    card.classList.toggle("announce", Boolean(line));
    // The line is written by a person, in the language they picked, and there is
    // nothing here to translate it with: it goes up exactly as it arrived. The
    // hint comes from the translation table, so it is re-applied instead.
    body.textContent = line;
    // A line longer than the card is clipped to it; hovering shows all of it.
    card.setAttribute("title", line);
    if (!line) Glossy.i18n.apply(document);
  }

  /** Asks the backend which sentence is on the card, and draws it. */
  async function read() {
    try {
      line = String((await Glossy.invoke("notice_text")) || "").trim();
    } catch (error) {
      console.warn("glossy: cannot read the notice", error);
      line = "";
    }
    draw();
  }

  function applyLanguage(uiLang) {
    Glossy.i18n.set(uiLang);
    draw();
  }

  /** Follows the colour scheme picked in the settings. */
  function applyTheme(theme) {
    GlossyTheme.apply({ theme });
  }

  close.addEventListener("click", (event) => {
    event.stopPropagation();
    hide("notice_close");
  });

  // Clicking the hint opens the settings window; clicking a line from the relay
  // is only ever an acknowledgement, which the backend decides.
  card.addEventListener("click", () => hide("notice_open"));

  Glossy.listen("glossy://notice", read);

  (async function start() {
    try {
      const settings = await Glossy.readSettings();
      applyLanguage(settings.uiLang);
      applyTheme(settings.theme);
      // The palette and the accent reach this window through the cache js/theme.js
      // keeps, which its boot path has already applied.
    } catch (error) {
      console.warn("glossy: cannot read settings", error);
      applyLanguage("system");
    }
    // A line that arrived while this document was still loading had nobody to
    // hear it; the backend still has it, so it is asked for once more.
    read();
  })();
})(window.Glossy);
