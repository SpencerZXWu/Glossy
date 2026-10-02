/**
 * The region picker for the screenshot translation.
 *
 * The picture is already in the backend when this document comes up: the
 * overlay only collects a rectangle, in its own CSS pixels, and hands it over.
 * Nothing here has to be captured, which is why the page can draw freely over
 * the screen.
 *
 * A drag that selects nothing, the Escape key and a right click all abandon the
 * screenshot, so a stray press never costs a translation.
 */
(function (Glossy) {
  const band = document.getElementById("band");
  const size = document.getElementById("size");
  const hint = document.getElementById("hint");

  /**
   * Sentences the overlay can be up for, one per rectangle that is wanted: the
   * subtitles ask for two in a row, and the same overlay is used for both.
   */
  const HINTS = {
    once: "ocr.hint",
    area: "subtitle.pick.area",
    place: "subtitle.pick.place",
  };

  /** Smallest rectangle that is worth sending, in CSS pixels. The backend
      refuses anything smaller, so a click is turned away here instead. */
  const MIN_SIDE = 8;

  /** Corner the drag started from, or null when the pointer is not down. */
  let start = null;
  /** True once a rectangle was handed over; the window is on its way out and
      the leftover pointer events of that same click must be ignored. */
  let sent = false;

  function reset() {
    start = null;
    sent = false;
    document.body.classList.remove("dragging");
    size.hidden = true;
  }

  function clamp(value, limit) {
    return Math.min(Math.max(value, 0), limit);
  }

  /** The rectangle between the two corners, kept inside the screen. */
  function box(from, to) {
    const left = clamp(Math.min(from.x, to.x), window.innerWidth);
    const top = clamp(Math.min(from.y, to.y), window.innerHeight);
    return {
      x: left,
      y: top,
      width: clamp(Math.max(from.x, to.x), window.innerWidth) - left,
      height: clamp(Math.max(from.y, to.y), window.innerHeight) - top,
    };
  }

  function draw(rect) {
    band.style.left = `${rect.x}px`;
    band.style.top = `${rect.y}px`;
    band.style.width = `${rect.width}px`;
    band.style.height = `${rect.height}px`;

    // The readout follows the shorter edge of the rectangle, and stays on
    // screen when the rectangle touches the bottom of the monitor.
    size.hidden = false;
    size.textContent = `${Math.round(rect.width)} × ${Math.round(rect.height)}`;
    const below = rect.y + rect.height + 6;
    const [measured] = size.getClientRects();
    const height = measured ? measured.height : 20;
    size.style.left = `${clamp(rect.x, window.innerWidth - 80)}px`;
    size.style.top = `${below + height > window.innerHeight ? rect.y - height - 6 : below}px`;
  }

  function cancel() {
    reset();
    Glossy.invoke("ocr_cancel").catch((error) => console.warn("glossy: ocr", error));
  }

  async function send(rect) {
    sent = true;
    try {
      await Glossy.invoke("ocr_region", rect);
    } catch (error) {
      console.warn("glossy: ocr", error);
    }
  }

  window.addEventListener("pointerdown", (event) => {
    if (sent || event.button !== 0) return;
    reset();
    start = { x: event.clientX, y: event.clientY };
    document.body.classList.add("dragging");
    draw(box(start, start));
  });

  window.addEventListener("pointermove", (event) => {
    if (!start) return;
    draw(box(start, { x: event.clientX, y: event.clientY }));
  });

  window.addEventListener("pointerup", (event) => {
    if (!start) return;
    const rect = box(start, { x: event.clientX, y: event.clientY });
    reset();
    if (rect.width < MIN_SIDE || rect.height < MIN_SIDE) cancel();
    else send(rect);
  });

  window.addEventListener("contextmenu", (event) => {
    event.preventDefault();
    if (!sent) cancel();
  });

  window.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || sent) return;
    cancel();
  });

  // The window is put away by the backend and brought back for the next
  // screenshot, so the page starts each one from a clean slate. The overlay is
  // also the one window that is shown again and again without ever being
  // reloaded: should that message ever be missed, the page would stay stuck on
  // `sent` and ignore every later drag. Coming back to the front is a moment no
  // half-finished pick can be in, so it is a safe second chance to clear up.
  //
  // The message also names what the rectangle is for, which is the one thing
  // that changes about the overlay between the screenshot and the two picks of
  // the subtitle reading.
  Glossy.listen("glossy://ocr", (event) => {
    const want = (event && event.payload) || "once";
    hint.setAttribute("data-i18n", HINTS[want] || HINTS.once);
    hint.textContent = Glossy.i18n.t(HINTS[want] || HINTS.once);
    reset();
  }).catch((error) => console.warn("glossy: ocr", error));
  window.addEventListener("focus", () => {
    if (sent) reset();
  });

  (async function load() {
    try {
      const settings = await Glossy.readSettings();
      Glossy.i18n.set(settings.uiLang);
    } catch (error) {
      console.warn("glossy: cannot read settings", error);
      Glossy.i18n.set("system");
    }
    Glossy.i18n.apply(document);
  })();
})(window.Glossy);
