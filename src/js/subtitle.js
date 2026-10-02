(() => {
  "use strict";

  /**
   * The subtitle line: what the reading has translated, drawn where the user
   * put the box.
   *
   * The page holds no controls on purpose. The window is as big as the box that
   * was picked for it and it is over a video that is being watched, so it takes
   * neither the mouse nor the keyboard: the reading is stopped from the settings
   * window, which is where it was started.
   */
  const line = document.getElementById("line");
  const note = document.getElementById("note");

  /** The largest and smallest the line is allowed to be drawn at. */
  const SMALLEST = 13;
  const LARGEST = 40;
  let preferredSize = 26;

  /**
   * Draws the line as large as it fits in the box.
   *
   * A translation is longer than what it translates more often than not, and
   * the box was picked for the original: the line is drawn at its largest size
   * and stepped down until it fits, which keeps a short line big and still
   * shows a long one whole.
   */
  function fit() {
    const height = document.documentElement.clientHeight - 12;
    const width = document.documentElement.clientWidth - 20;
    let size = Math.min(LARGEST, Math.max(SMALLEST, preferredSize));
    line.style.fontSize = `${size}px`;
    while (size > SMALLEST) {
      const box = line.getBoundingClientRect();
      if (box.height <= height && box.width <= width) return;
      size -= 1;
      line.style.fontSize = `${size}px`;
    }
  }

  function show(text) {
    line.textContent = text;
    document.body.dataset.state = "line";
    fit();
  }

  function failed(message) {
    note.textContent = message;
    document.body.dataset.state = "failed";
  }

  Glossy.listen("glossy://subtitle", (event) => {
    const payload = (event && event.payload) || {};
    if (payload.error) {
      failed(String(payload.error));
      return;
    }
    const text = String(payload.text || "").trim();
    if (text) show(text);
  });

  // The size of the box can change between two runs, and the line has to fit
  // again when it does — both when the window is resized and when the box is
  // moved under it while the reading runs.
  window.addEventListener("resize", () => {
    if (document.body.dataset.state === "line") fit();
  });
  Glossy.listen("glossy://subtitle-resize", () => {
    if (document.body.dataset.state === "line") fit();
  }).catch((error) => console.warn("glossy: subtitle", error));

  async function load() {
    try {
      const settings = await Glossy.readSettings();
      preferredSize = Math.min(LARGEST, Math.max(SMALLEST, Number(settings.subtitleFontSize || 26)));
      Glossy.i18n.set(settings.uiLang);
    } catch (error) {
      preferredSize = 26;
      console.warn("glossy: unable to read the settings", error);
    }
    Glossy.i18n.apply(document);
    document.body.dataset.state = "idle";
  }

  load();
})();
