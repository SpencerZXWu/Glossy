"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const { extractDictionaryKeys, readI18nSource } = require("./helpers/load-scripts.js");

const SRC = path.resolve(__dirname, "..", "src");
const PAGES = ["index.html", "popup.html", "notice.html"];
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
  // The visible text of these two is nothing at all, so the tooltip and the
  // accessible name are the whole label and both have to follow the interface
  // language. The static English value is what shows before the script runs.
  const uiLang = tagOf(readPage("index.html"), "uiLang");
  assert.ok(uiLang.includes('title="Interface language"'), uiLang);
  assert.ok(uiLang.includes('data-i18n-title="ui.lang"'), uiLang);
  assert.ok(uiLang.includes('data-i18n-aria-label="ui.lang"'), uiLang);

  const close = tagOf(readPage("notice.html"), "close");
  assert.ok(close.includes('title="Dismiss"'), close);
  assert.ok(close.includes('data-i18n-title="notice.close"'), close);
  assert.ok(close.includes('data-i18n-aria-label="notice.close"'), close);
});
