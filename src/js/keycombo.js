/**
 * Key combinations: reading one off a keyboard event, and knowing which ones
 * Windows will not hand over.
 *
 * The spelling produced here is the one `hotkey::parse` in the backend reads
 * back, so a recorded combination is accepted without the user ever typing it.
 * Only keys that parse recognises are reported, which is why `label` is a fixed
 * table rather than `event.key` with the first letter capitalised.
 */
(function (Glossy) {
  const MODIFIERS = [
    ["ctrlKey", "Ctrl"],
    ["altKey", "Alt"],
    ["shiftKey", "Shift"],
    ["metaKey", "Win"],
  ];

  const NAMED = {
    " ": "Space",
    space: "Space",
    spacebar: "Space",
    enter: "Enter",
    return: "Enter",
    tab: "Tab",
    escape: "Esc",
    esc: "Esc",
    backspace: "Backspace",
    delete: "Delete",
    del: "Delete",
    insert: "Insert",
    ins: "Insert",
    home: "Home",
    end: "End",
    pageup: "PageUp",
    pagedown: "PageDown",
    arrowup: "Up",
    arrowdown: "Down",
    arrowleft: "Left",
    arrowright: "Right",
    up: "Up",
    down: "Down",
    left: "Left",
    right: "Right",
  };

  /** Keys that only modify: holding one is not yet a combination. */
  const PURE_MODIFIERS = ["Control", "Alt", "Shift", "Meta", "AltGraph", "OS", "Hyper"];

  /**
   * Combinations Windows answers itself, so `RegisterHotKey` never receives
   * them: recording one would leave the user with a hotkey that cannot work.
   */
  const BLOCKED = [
    "Ctrl+Alt+Delete",
    "Win+L",
    "Win+Tab",
    "Alt+Tab",
    "Alt+Esc",
    "Ctrl+Esc",
    "Ctrl+Shift+Esc",
  ];

  /**
   * Combinations Windows or Explorer usually owns already. They can sometimes
   * be registered, so these are a warning rather than a refusal — the line under
   * the field reports what the registration actually did.
   */
  const BUSY = [
    "Win+D",
    "Win+E",
    "Win+R",
    "Win+S",
    "Win+X",
    "Win+A",
    "Win+I",
    "Win+P",
    "Win+Space",
    "Win+Shift+S",
    "Alt+F4",
    "Alt+Space",
  ];

  /** The canonical name of the key in an event, or null when it has none. */
  function label(event) {
    const key = String(event.key || "");
    const code = String(event.code || "");

    if (/^[a-z]$/i.test(key)) return key.toUpperCase();
    if (/^[0-9]$/.test(key)) return key;
    if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key)) return key.toUpperCase();

    const named = NAMED[key.toLowerCase()];
    if (named) return named;

    // A layout can turn a key into a character Glossy has no name for ("é" for
    // the digit row, say); the physical code still says which key it was.
    const byCode = /^Key([A-Z])$/.exec(code);
    if (byCode) return byCode[1];
    const digit = /^Digit([0-9])$/.exec(code);
    if (digit) return digit[1];

    return null;
  }

  /**
   * Reads the combination an event describes.
   *
   * `problem` is empty when `spec` can be registered, and otherwise says why
   * not: "modifiers" while only modifiers are held, "nomod" when the key was
   * pressed on its own, "unknown" for a key the backend cannot name.
   */
  function capture(event) {
    const named = label(event);
    const held = MODIFIERS.filter(([flag]) => event[flag]).map(([, name]) => name);

    if (!named) {
      if (PURE_MODIFIERS.includes(String(event.key || ""))) {
        return { spec: "", problem: "modifiers", key: "" };
      }
      const pressed = String(event.key || "");
      return { spec: "", problem: "unknown", key: pressed === "Unidentified" ? "" : pressed };
    }
    if (!held.length) return { spec: named, problem: "nomod", key: "" };

    return { spec: held.concat(named).join("+"), problem: "", key: "" };
  }

  /** The spelled-out name of a key, whatever case and alias it was written in. */
  function canonicalKey(part) {
    const lower = part.toLowerCase();
    if (/^[a-z]$/.test(lower)) return lower.toUpperCase();
    if (NAMED[lower]) return NAMED[lower];
    if (/^f([1-9]|1[0-9]|2[0-4])$/.test(lower)) return lower.toUpperCase();
    return part;
  }

  /** Sorts a combination so two spellings of it compare equal. */
  function normalize(spec) {
    const parts = String(spec || "")
      .split("+")
      .map((part) => part.trim())
      .filter(Boolean);
    const order = MODIFIERS.map(([, name]) => name);
    const held = [];
    let key = "";
    parts.forEach((part) => {
      const name = order.find((candidate) => candidate.toLowerCase() === part.toLowerCase());
      if (name) {
        if (!held.includes(name)) held.push(name);
        return;
      }
      key = canonicalKey(part);
    });
    held.sort((a, b) => order.indexOf(a) - order.indexOf(b));
    return held.concat(key ? [key] : []).join("+");
  }

  /** "blocked", "busy", or "" when nothing is known against the combination. */
  function conflict(spec) {
    const wanted = normalize(spec);
    if (!wanted) return "";
    if (BLOCKED.some((blocked) => normalize(blocked) === wanted)) return "blocked";
    if (BUSY.some((busy) => normalize(busy) === wanted)) return "busy";
    return "";
  }

  Glossy.keycombo = { capture, conflict, label, normalize, BLOCKED, BUSY };
})(window.Glossy);
