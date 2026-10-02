/**
 * The box editor: the two rectangles a subtitle reading is made of.
 *
 * The backend hands the boxes over in the CSS pixels of this window, which
 * covers one monitor exactly, so a box drawn here and a box on the screen are
 * the same numbers. A drag moves or resizes one of them, and the two are handed
 * back when the user is done with them.
 *
 * Nothing here reads a screen or talks to a service: the boxes are a pair of
 * rectangles, and the reading that uses them is already running behind this
 * window.
 */
(function (Glossy) {
  "use strict";

  /** The element of each box, and how small each of them may be drawn. */
  const BOXES = {
    area: { element: document.getElementById("area"), min: { width: 8, height: 8 } },
    place: { element: document.getElementById("place"), min: { width: 160, height: 44 } },
  };

  /** The eight points a box is resized from, named by the edges each holds. */
  const HANDLES = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

  /** Where the boxes are now, each as `{ x, y, width, height }`. */
  let boxes = { area: null, place: null };

  /** The drag in progress, or null. */
  let drag = null;

  function clamp(value, low, high) {
    return Math.min(Math.max(value, low), high);
  }

  /** Keeps a box whole and inside the monitor it is drawn on. */
  function fit(box, name) {
    const min = BOXES[name].min;
    const width = clamp(box.width, min.width, window.innerWidth);
    const height = clamp(box.height, min.height, window.innerHeight);
    return {
      x: clamp(box.x, 0, window.innerWidth - width),
      y: clamp(box.y, 0, window.innerHeight - height),
      width,
      height,
    };
  }

  function draw(name) {
    const box = boxes[name];
    const element = BOXES[name].element;
    if (!box) {
      element.hidden = true;
      return;
    }
    element.hidden = false;
    element.style.left = `${box.x}px`;
    element.style.top = `${box.y}px`;
    element.style.width = `${box.width}px`;
    element.style.height = `${box.height}px`;
    // A box against the top of the screen has no room above it for its tag.
    element.dataset.flip = box.y < 22 ? "down" : "up";
  }

  function render() {
    draw("area");
    draw("place");
  }

  /** The box a drag ends at, given how far the pointer has moved. */
  function dragged(start, mode, dx, dy, min) {
    if (mode === "move") {
      return { ...start, x: start.x + dx, y: start.y + dy };
    }
    // The mode is `resize-<edges>`, where the edges are the handle's own name:
    // "nw" holds two of them, "n" one. The name is what the letters are read
    // from, so a box can never be dragged by an edge it has no handle for.
    const edges = mode.replace("resize-", "");
    const box = { ...start };
    if (edges.includes("w")) {
      // Holding the west edge moves the east one nowhere, so a box that is
      // dragged past its own smallest size stops instead of turning inside out.
      const right = start.x + start.width;
      box.width = Math.max(start.width - dx, min.width);
      box.x = right - box.width;
    }
    if (edges.includes("e")) box.width = Math.max(start.width + dx, min.width);
    if (edges.includes("n")) {
      const bottom = start.y + start.height;
      box.height = Math.max(start.height - dy, min.height);
      box.y = bottom - box.height;
    }
    if (edges.includes("s")) box.height = Math.max(start.height + dy, min.height);
    return box;
  }

  function begin(event, name, mode) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    drag = {
      name,
      mode,
      from: { x: event.clientX, y: event.clientY },
      box: { ...boxes[name] },
      target: event.currentTarget,
    };
    if (drag.target.setPointerCapture) drag.target.setPointerCapture(event.pointerId);
  }

  for (const [name, { element }] of Object.entries(BOXES)) {
    element.addEventListener("pointerdown", (event) => begin(event, name, "move"));
    for (const at of HANDLES) {
      const handle = document.createElement("span");
      handle.className = "handle";
      handle.dataset.at = at;
      handle.addEventListener("pointerdown", (event) => begin(event, name, `resize-${at}`));
      element.appendChild(handle);
    }
  }

  // One move handler for whichever box is being dragged, wherever the pointer
  // is: the box under the pointer takes the capture, so the drag keeps working
  // when the pointer leaves the box — which is what resizing is.
  window.addEventListener("pointermove", (event) => {
    if (!drag) return;
    const dx = event.clientX - drag.from.x;
    const dy = event.clientY - drag.from.y;
    boxes[drag.name] = fit(
      dragged(drag.box, drag.mode, dx, dy, BOXES[drag.name].min),
      drag.name
    );
    draw(drag.name);
  });
  window.addEventListener("pointerup", () => {
    drag = null;
  });
  window.addEventListener("pointercancel", () => {
    drag = null;
  });

  async function apply() {
    if (!boxes.area || !boxes.place) return;
    try {
      await Glossy.invoke("subtitle_edit_apply", {
        area: boxes.area,
        place: boxes.place,
      });
    } catch (error) {
      console.warn("glossy: the boxes could not be moved", error);
    }
  }

  function cancel() {
    Glossy.invoke("subtitle_edit_cancel").catch((error) =>
      console.warn("glossy: the boxes were not put away", error)
    );
  }

  document.getElementById("apply").addEventListener("click", apply);
  document.getElementById("cancel").addEventListener("click", cancel);
  window.addEventListener("keydown", (event) => {
    if (event.key === "Escape") cancel();
    else if (event.key === "Enter") apply();
  });

  // The window is shown again for the next edit without being reloaded, so the
  // boxes are taken from the message rather than read once at startup.
  Glossy.listen("glossy://subtitle-edit", (event) => {
    const payload = (event && event.payload) || {};
    boxes = { area: payload.area || null, place: payload.place || null };
    if (boxes.area) boxes.area = fit({ ...boxes.area }, "area");
    if (boxes.place) boxes.place = fit({ ...boxes.place }, "place");
    drag = null;
    render();
    document.getElementById("apply").focus();
  }).catch((error) => console.warn("glossy: the boxes were not answered", error));

  (async function load() {
    try {
      const settings = await Glossy.readSettings();
      Glossy.i18n.set(settings.uiLang);
    } catch (error) {
      console.warn("glossy: cannot read the settings", error);
    }
    Glossy.i18n.apply(document);
  })();
})(window.Glossy);
