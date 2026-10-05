"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const { extractDictionaryKeys, readI18nSource } = require("./helpers/load-scripts.js");

const SRC = path.resolve(__dirname, "..", "src");
const PAGES = ["index.html", "popup.html", "notice.html", "ocr.html"];
const KEYS = new Set(extractDictionaryKeys(readI18nSource(), "ENGLISH"));

function readPage(name) {
  return fs.readFileSync(path.join(SRC, name), "utf8");
}

/** Every `data-i18n*` attribute of a page, as `attribute=key` entries. */
function localizedAttributes(html) {
  const pattern = /(data-i18n(?:-placeholder|-title|-aria-label)?)="([^"]+)"/g;
  return Array.from(html.matchAll(pattern), (match) => `${match[1]}=${match[2]}`);
}

test("every key a page asks for exists in the dictionary", () => {
  const missing = PAGES.flatMap((name) =>
    localizedAttributes(readPage(name)).filter((entry) => !KEYS.has(entry.split("=")[1])),
  );
  assert.deepEqual(missing, []);
});

/** The whole opening tag of the element that carries `id`. */
function tagOf(html, id) {
  const match = new RegExp(`<[a-z]+[^>]*id="${id}"[^>]*>`).exec(html);
  assert.ok(match, `no element with id="${id}"`);
  return match[0];
}

test("a control named only by an attribute carries that name in both directions", () => {
  // The visible text of this one is nothing at all, so the tooltip and the
  // accessible name are the whole label and both have to follow the interface
  // language. The static English value is what shows before the script runs.
  const close = tagOf(readPage("notice.html"), "close");
  assert.ok(close.includes('title="Dismiss"'), close);
  assert.ok(close.includes('data-i18n-title="notice.close"'), close);
  assert.ok(close.includes('data-i18n-aria-label="notice.close"'), close);

  // The switch in the title bar is read out as the switch it is: the state comes
  // from `aria-checked` and the name from the attribute, which therefore needs
  // the same English fallback.
  const status = tagOf(readPage("index.html"), "status");
  assert.ok(status.includes('role="switch"'), status);
  assert.ok(status.includes('aria-checked="true"'), status);
  assert.ok(status.includes('aria-label="Selection translation"'), status);
  assert.ok(status.includes('data-i18n-aria-label="status.switch"'), status);
});

test("every control of a page is named, by a label or by an attribute", () => {
  // The interface language moved into the settings window, where it has a
  // visible label of its own rather than a tooltip in the title bar.
  const html = readPage("index.html");
  const label = /<label for="uiLang"[^>]*data-i18n="field\.uiLang"/.exec(html);

  assert.ok(label, "the interface language has no visible label");
  assert.ok(!/data-i18n-title="ui\.lang"/.test(html), "the tooltip is doing the label's work");
});

test("the guide offers one step per scene, and every scene says what it shows", () => {
  const html = readPage("index.html");
  const steps = Array.from(html.matchAll(/class="guide-step"/g));
  const scenes = Array.from(html.matchAll(/<section class="scene" id="guideScene(\d+)"/g), (m) => m[1]);

  // A step with no scene behind it is a button that opens nothing.
  assert.equal(steps.length, scenes.length, "the rail and the scenes disagree");
  assert.deepEqual(scenes, scenes.map((_, index) => String(index)));

  // The numbers the rail shows run in the order the scenes are written, which
  // is the order the arrows and the counter walk.
  const numbers = Array.from(html.matchAll(/<span class="guide-num">(\d+)<\/span>/g), (m) => m[1]);
  assert.deepEqual(numbers, scenes.map((_, index) => String(index + 1)));

  for (const id of scenes) {
    // Every scene carries a heading and both lines of copy, each localized:
    // a scene that shows an animation and explains nothing is not a step.
    const block = html.split(`id="guideScene${id}"`)[1].split("</section>")[0];
    for (const part of ["<h3 data-i18n=", 'class="guide-how" data-i18n=', 'class="hint" data-i18n=']) {
      assert.ok(block.includes(part), `scene ${id} is missing ${part}`);
    }
  }
});
