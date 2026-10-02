"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");

const fs = require("node:fs");
const path = require("node:path");

const { JS_DIR, loadFrontend } = require("./helpers/load-scripts.js");
const { findByClass, classesOf, descendantsOf } = require("./helpers/fake-dom.js");

/** The Rust module that decides which languages each channel translates. */
const RUST_LANGUAGES = path.resolve(
  JS_DIR,
  "..",
  "..",
  "src-tauri",
  "src",
  "translate",
  "languages.rs"
);

function newEnv() {
  const env = loadFrontend();
  env.Glossy.i18n.set("en");
  return env;
}

const env = newEnv();
const Glossy = env.Glossy;
const i18n = Glossy.i18n;

function target() {
  return env.document.createElement("div");
}

/** The rows of buttons, one per side of the card, in the order they are drawn. */
function toolsOf(root) {
  return descendantsOf(root, []).filter((node) => node.className === "tools");
}

/** A key press that records what the handlers did with it. */
function keyEvent(key, options) {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    defaultPrevented: false,
    stopped: false,
    ...options,
    preventDefault() {
      this.defaultPrevented = true;
    },
    stopPropagation() {
      this.stopped = true;
    },
  };
}

test("the module exports the render helpers the windows rely on", () => {
  assert.equal(typeof Glossy.languageName, "function");
  assert.equal(typeof Glossy.errorMessage, "function");
  assert.ok(Array.isArray(Array.from(Glossy.languageCodes)));
  for (const name of ["loading", "error", "result", "clear", "stopSpeaking"]) {
    assert.equal(typeof Glossy.render[name], "function", `${name} is missing`);
  }
});

test("the language code list is non-empty, unique and excludes auto", () => {
  const codes = Array.from(Glossy.languageCodes);
  assert.ok(codes.length >= 20);
  assert.equal(new Set(codes).size, codes.length);
  assert.ok(!codes.includes("auto"));
  assert.ok(codes.includes("zh-CN"));
});

test("every offered language code has its own English name", () => {
  const fallbacks = Array.from(Glossy.languageCodes)
    .map((code) => [code, Glossy.languageName(code)])
    .filter(([code, name]) => name === code.toUpperCase());
  assert.deepEqual(fallbacks, []);
});

test("well known codes map to their English names", () => {
  const expected = {
    en: "English",
    "zh-CN": "Chinese (Simplified)",
    "zh-TW": "Chinese (Traditional)",
    ja: "Japanese",
    ko: "Korean",
    de: "German",
    sv: "Swedish",
  };
  for (const [code, name] of Object.entries(expected)) {
    assert.equal(Glossy.languageName(code), name);
  }
  assert.equal(Glossy.languageName("auto"), "Auto detect");
});

test("languageName localizes once Chinese is active", () => {
  i18n.set("zh");
  assert.equal(Glossy.languageName("auto"), "自动检测");
  assert.equal(Glossy.languageName("zh-CN"), "简体中文");
  assert.equal(Glossy.languageName("zh-TW"), "繁体中文");
  assert.equal(Glossy.languageName("ja"), "日语");
  i18n.set("en");
  assert.equal(Glossy.languageName("ja"), "Japanese");
});

test("languageName names a region variant the table does not list", () => {
  assert.equal(Glossy.languageName("en-GB"), "English (GB)");
  assert.equal(Glossy.languageName("pt-BR"), "Portuguese (BR)");
});

test("languageName falls back to an uppercased code for unknown values", () => {
  assert.equal(Glossy.languageName("xx"), "XX");
  assert.equal(Glossy.languageName("qq-CN"), "QQ-CN");
});

test("languageName upper-cases the region suffix", () => {
  assert.equal(Glossy.languageName("en-us"), "English (US)");
  assert.equal(Glossy.languageName("pt-br"), "Portuguese (BR)");
});

test("languageName matches a bare or lower-cased code the table spells differently", () => {
  assert.equal(Glossy.languageName("zh"), "Chinese");
  assert.equal(Glossy.languageName("zh-cn"), "Chinese (Simplified)");
  assert.equal(Glossy.languageName("ZH-TW"), "Chinese (Traditional)");
});

test("languageName returns an empty string for empty input", () => {
  assert.equal(Glossy.languageName(""), "");
  assert.equal(Glossy.languageName(undefined), "");
  assert.equal(Glossy.languageName(null), "");
});

test("languageName stringifies a non-string code", () => {
  assert.equal(Glossy.languageName(123), "123");
});

test("errorMessage passes a plain string through untouched", () => {
  assert.equal(Glossy.errorMessage("boom"), "boom");
  assert.equal(Glossy.errorMessage(""), "");
});

test("errorMessage unwraps an Error or a rejection value", () => {
  assert.equal(Glossy.errorMessage(new Error("nope")), "nope");
  assert.equal(Glossy.errorMessage({ message: "wrapped" }), "wrapped");
});

test("errorMessage falls back to the localized failure text", () => {
  i18n.set("en");
  assert.equal(Glossy.errorMessage({}), "Translation failed.");
  assert.equal(Glossy.errorMessage(null), "Translation failed.");
  assert.equal(Glossy.errorMessage(undefined), "Translation failed.");
  i18n.set("zh");
  assert.equal(Glossy.errorMessage({ message: 42 }), "翻译失败。");
  i18n.set("en");
});

test("result renders a sentence with its original above the translation", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "你好",
    translation: "  Hello  ",
    provider: "google",
  });
  assert.equal(findByClass(node, "original").textContent, "你好");
  assert.equal(findByClass(node, "translation").textContent, "Hello");
  assert.equal(findByClass(node, "engine").textContent, "Google");
  assert.equal(findByClass(node, "phonetic"), null);
});

test("result drops the original when showOriginal is false", () => {
  const node = target();
  Glossy.render.result(
    node,
    { kind: "sentence", sourceText: "你好", translation: "Hello" },
    { showOriginal: false },
  );
  assert.equal(findByClass(node, "original"), null);
  assert.equal(findByClass(node, "translation").textContent, "Hello");
});

test("result drops the original when there is no source text", () => {
  const node = target();
  Glossy.render.result(node, { kind: "sentence", sourceText: "", translation: "Hello" });
  assert.equal(findByClass(node, "original"), null);
});

test("result renders the original under a word card too", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", sourceText: "fox", translation: "狐狸" });
  assert.equal(findByClass(node, "original").textContent, "fox");
  assert.equal(findByClass(node, "translation").textContent, "狐狸");
});

test("the original is plain text when the card cannot translate again", () => {
  const node = target();
  Glossy.render.result(node, { kind: "sentence", sourceText: "你好", translation: "Hello" });
  const original = findByClass(node, "original");
  assert.equal(original.getAttribute("contenteditable"), null);
  assert.equal(original.getAttribute("role"), null);
});

test("the original is editable when the card can translate again", () => {
  const node = target();
  Glossy.render.result(
    node,
    { kind: "sentence", sourceText: "你好", translation: "Hello" },
    { onRetranslate: () => {} },
  );
  const original = findByClass(node, "original");
  assert.equal(original.getAttribute("contenteditable"), "plaintext-only");
  assert.equal(original.getAttribute("spellcheck"), "false");
  assert.equal(original.getAttribute("role"), "textbox");
  assert.equal(original.getAttribute("aria-label"), "The text that was translated, editable");
  assert.ok(original.getAttribute("title").includes("Ctrl+Enter"));
});

test("Ctrl+Enter in the original translates the corrected text once", () => {
  const node = target();
  const asked = [];
  Glossy.render.result(
    node,
    { kind: "word", sourceText: "fox", translation: "狐狸" },
    { onRetranslate: (value) => asked.push(value) },
  );
  const original = findByClass(node, "original");
  original.textContent = "  foxes  ";

  const event = keyEvent("Enter", { ctrlKey: true });
  original.dispatch("keydown", event);

  // The block loses the caret after the edit, and the translation it already
  // asked for must not be asked for a second time by that.
  assert.ok(event.defaultPrevented, "the newline was not swallowed");
  assert.deepEqual(asked, ["foxes"]);
  assert.deepEqual(original.textContent, "foxes");
});

test("Escape in the original puts the text back and keeps the card open", () => {
  const node = target();
  const asked = [];
  Glossy.render.result(
    node,
    { kind: "sentence", sourceText: "你好", translation: "Hello" },
    { onRetranslate: (value) => asked.push(value) },
  );
  const original = findByClass(node, "original");
  original.textContent = "你好呀";

  const event = keyEvent("Escape");
  original.dispatch("keydown", event);

  // The card is dismissed by Escape too, so the edit has to keep the key to
  // itself while it holds the caret.
  assert.ok(event.defaultPrevented);
  assert.ok(event.stopped, "Escape would have reached the card");
  assert.equal(original.textContent, "你好");
  assert.deepEqual(asked, []);
});

test("leaving the original translates the edit, and nothing else", () => {
  const node = target();
  const asked = [];
  Glossy.render.result(
    node,
    { kind: "word", sourceText: "fox", translation: "狐狸" },
    { onRetranslate: (value) => asked.push(value) },
  );
  const original = findByClass(node, "original");

  // Leaving without touching anything, and leaving an emptied field, both mean
  // the same text: nothing to translate and no reason to spend quota.
  original.dispatch("blur");
  original.textContent = "   ";
  original.dispatch("blur");
  assert.deepEqual(asked, []);
  assert.equal(original.textContent, "fox");

  original.textContent = "vixen";
  original.dispatch("blur");
  assert.deepEqual(asked, ["vixen"]);
});

test("result shows the empty-translation text when nothing came back", () => {
  const node = target();
  Glossy.render.result(node, { kind: "sentence", translation: "   " });
  const line = findByClass(node, "translation empty");
  assert.equal(line.textContent, "No translation returned.");
  assert.equal(findByClass(node, "phonetic"), null);
  assert.equal(findByClass(node, "foot"), null);
});

test("result survives a missing or null payload", () => {
  const node = target();
  Glossy.render.result(node, null);
  assert.equal(
    findByClass(node, "translation empty").textContent,
    "No translation returned.",
  );
  assert.deepEqual(classesOf(node), ["translation empty"]);
});

test("result wraps a bare phonetic in slashes and keeps an existing wrapper", () => {
  const bare = target();
  Glossy.render.result(bare, { kind: "word", translation: "fox", phonetic: " fɒks " });
  assert.equal(findByClass(bare, "phonetic").textContent, "/fɒks/");

  const slashed = target();
  Glossy.render.result(slashed, { kind: "word", translation: "fox", phonetic: "/fɒks/" });
  assert.equal(findByClass(slashed, "phonetic").textContent, "/fɒks/");

  const bracketed = target();
  Glossy.render.result(bracketed, { kind: "word", translation: "fox", phonetic: "[fɒks]" });
  assert.equal(findByClass(bracketed, "phonetic").textContent, "[fɒks]");
});

test("result skips the phonetic row for a whitespace-only phonetic", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", phonetic: "   " });
  assert.equal(findByClass(node, "phonetic"), null);
  assert.deepEqual(classesOf(node), ["translation", "tools", "tool", "tool"]);
});

test("result renders meanings with part of speech and joined definitions", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "fox",
    meanings: [
      { partOfSpeech: "noun", definitions: ["a wild animal", "a sly person"] },
      { partOfSpeech: "verb", definitions: ["to trick"] },
    ],
  });
  const meanings = findByClass(node, "meanings");
  assert.ok(meanings);
  assert.equal(meanings.childNodes.length, 2);
  assert.equal(findByClass(meanings.childNodes[0], "pos").textContent, "noun");
  assert.equal(findByClass(meanings.childNodes[0], "defs").textContent, "a wild animal · a sly person");
  assert.equal(findByClass(meanings.childNodes[1], "pos").textContent, "verb");
  assert.equal(findByClass(meanings.childNodes[1], "defs").textContent, "to trick");
});

test("result defaults the part of speech to def", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "fox",
    meanings: [{ definitions: ["a wild animal"] }],
  });
  assert.equal(findByClass(findByClass(node, "meanings"), "pos").textContent, "def");
});

test("result filters empty definitions and skips meanings without any", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "fox",
    meanings: [
      { definitions: ["", null, undefined, "kept"] },
      { definitions: ["", null] },
      { partOfSpeech: "noun" },
    ],
  });
  const meanings = findByClass(node, "meanings");
  assert.equal(meanings.childNodes.length, 1);
  assert.equal(findByClass(meanings, "defs").textContent, "kept");
});

test("result omits the meanings block when the field is not a list", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", meanings: "noun: fox" });
  assert.equal(findByClass(node, "meanings"), null);
});

test("result omits the meanings block when every meaning is empty", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", meanings: [] });
  assert.equal(findByClass(node, "meanings"), null);
});

test("result renders an example only when one was returned", () => {
  const withExample = target();
  Glossy.render.result(withExample, { kind: "word", translation: "fox", example: "The fox ran." });
  assert.equal(findByClass(withExample, "example").textContent, "The fox ran.");

  const without = target();
  Glossy.render.result(without, { kind: "word", translation: "fox" });
  assert.equal(findByClass(without, "example"), null);
});

test("result shows a lookup placeholder for a word card that is still waiting", () => {
  const node = target();
  Glossy.render.result(
    node,
    { kind: "word", translation: "狐狸", provider: "baidu" },
    { pending: true },
  );

  assert.equal(findByClass(node, "pending").textContent, "Looking up the dictionary…");
  assert.equal(findByClass(node, "pending").getAttribute("aria-live"), "polite");
  assert.equal(findByClass(node, "phonetic"), null);
  // The translation and the footer are still drawn around it.
  assert.equal(findByClass(node, "translation").textContent, "狐狸");
  assert.equal(findByClass(node, "engine").textContent, "Baidu Translate");
});

test("result localizes the lookup placeholder", () => {
  const node = target();
  i18n.set("zh");
  Glossy.render.result(node, { kind: "word", translation: "fox" }, { pending: true });
  assert.equal(findByClass(node, "pending").textContent, "词典查询中…");
  i18n.set("en");
});

test("result drops the placeholder once any detail arrived", () => {
  const phonetic = target();
  Glossy.render.result(
    phonetic,
    { kind: "word", translation: "fox", phonetic: "fɒks" },
    { pending: true },
  );
  assert.equal(findByClass(phonetic, "pending"), null);

  const meanings = target();
  Glossy.render.result(
    meanings,
    { kind: "word", translation: "fox", meanings: [{ partOfSpeech: "noun", definitions: ["a sly animal"] }] },
    { pending: true },
  );
  assert.equal(findByClass(meanings, "pending"), null);

  const example = target();
  Glossy.render.result(
    example,
    { kind: "word", translation: "fox", example: "The fox ran." },
    { pending: true },
  );
  assert.equal(findByClass(example, "pending"), null);

  // Meanings that carry no definition do not count as a detail.
  const hollowed = target();
  Glossy.render.result(
    hollowed,
    { kind: "word", translation: "fox", meanings: [{ definitions: [] }] },
    { pending: true },
  );
  assert.ok(findByClass(hollowed, "pending"));
});

test("result never shows the placeholder without the pending flag", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox" });
  assert.equal(findByClass(node, "pending"), null);

  const sentence = target();
  Glossy.render.result(
    sentence,
    { kind: "sentence", translation: "狐狸在跑。" },
    { pending: true },
  );
  assert.equal(findByClass(sentence, "pending"), null);
});

test("result shows the unit conversions of a sentence", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "这个房间有 3.66 米宽。",
    provider: "google",
    conversions: [
      {
        category: "length",
        original: "12 ft",
        converted: "3.66 m",
        rate: "1 ft = 0.3048 m",
      },
    ],
  });

  const units = findByClass(node, "units");
  assert.ok(units);
  assert.equal(findByClass(node, "units-title").textContent, "Units");
  assert.equal(findByClass(node, "unit-from").textContent, "12 ft");
  assert.equal(findByClass(node, "unit-arrow").textContent, "≈");
  assert.equal(findByClass(node, "unit-to").textContent, "3.66 m");
  assert.equal(findByClass(node, "unit-rate").textContent, "1 ft = 0.3048 m");
  // The block sits above the provider footer.
  const order = classesOf(node);
  assert.ok(order.indexOf("units") < order.indexOf("foot"));
});

test("result annotates a live currency rate once", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "一共 1430 元。",
    conversions: [
      {
        category: "currency",
        original: "$200",
        converted: "¥1,430.00",
        rate: "1 USD = 7.15 CNY",
        rateSource: "exchangerate-api.com",
        rateDate: "2026-02-05",
      },
      {
        category: "currency",
        original: "$20",
        converted: "¥143.00",
        rate: "1 USD = 7.15 CNY",
        rateSource: "exchangerate-api.com",
        rateDate: "2026-02-05",
      },
    ],
  });

  const notes = descendantsOf(node, []).filter((child) => child.className === "unit-note");
  assert.equal(notes.length, 1);
  assert.equal(notes[0].textContent, "Live rate · exchangerate-api.com · 2026-02-05");
  const rows = descendantsOf(node, []).filter((child) => child.className === "unit");
  assert.equal(rows.length, 2);
});

test("result marks a rate that came from the cache instead of the network", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "一共 1430 元。",
    conversions: [
      {
        original: "$200",
        converted: "¥1,430.00",
        rate: "1 USD = 7.15 CNY",
        rateSource: "frankfurter.app",
        rateDate: "2026-02-04",
        stale: true,
      },
    ],
  });

  assert.equal(
    findByClass(node, "unit-note").textContent,
    "Last known rate · frankfurter.app · 2026-02-04",
  );
});

test("result localizes the unit block", () => {
  const node = target();
  i18n.set("zh");
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "房间宽 3.66 米。",
    conversions: [{ original: "12 ft", converted: "3.66 m", rate: "1 ft = 0.3048 m" }],
  });
  assert.equal(findByClass(node, "units-title").textContent, "单位换算");
  assert.equal(findByClass(node, "unit-arrow").textContent, "≈");
  i18n.set("en");
});

test("result renders no unit block without conversions", () => {
  for (const conversions of [undefined, null, [], "nope", [{}], [{ original: "" }]]) {
    const node = target();
    Glossy.render.result(node, {
      kind: "sentence",
      translation: "Hello",
      conversions,
    });
    assert.equal(findByClass(node, "units"), null, `conversions=${String(conversions)}`);
  }
});

test("result writes unit conversions as text, not markup", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "Hello",
    conversions: [
      { original: "<b>12 ft</b>", converted: "<img src=x>", rate: "<script>alert(1)</script>" },
    ],
  });
  const html = node.outerHTML;
  assert.ok(html.includes("&lt;b&gt;12 ft&lt;/b&gt;"));
  assert.ok(html.includes("&lt;img src=x&gt;"));
  assert.ok(!html.includes("<script"));
});

test("result localizes the provider footer", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });
  assert.equal(findByClass(node, "engine").textContent, "Baidu Translate");
  i18n.set("zh");
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });
  assert.equal(findByClass(node, "engine").textContent, "百度翻译");
  i18n.set("en");
});

test("result leaves the footer off without a provider", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", provider: "" });
  assert.equal(findByClass(node, "foot"), null);
  assert.equal(findByClass(node, "engine"), null);
  assert.equal(findByClass(node, "service"), null);
});

test("result keeps the name of the engine on a footer row of its own", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });
  const foot = findByClass(node, "foot");
  assert.equal(foot.parentNode, node);
  assert.equal(findByClass(foot, "engine").textContent, "Baidu Translate");
  assert.deepEqual(classesOf(foot), ["engine", "brand", "brand-logo", "brand-name"]);
  // The buttons belong to the text they act on, not to the footer.
  assert.deepEqual(toolsOf(foot), []);
});

test("result signs the card with the app mark at the right of the footer", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });
  const foot = findByClass(node, "foot");
  const brand = findByClass(foot, "brand");

  // The mark closes the row that names the engine, and says nothing a reader
  // needs, so its logo carries no alternative text.
  assert.equal(foot.childNodes[foot.childNodes.length - 1], brand);
  assert.equal(brand.textContent, "Glossy");
  const logo = findByClass(brand, "brand-logo");
  assert.ok(logo.src.includes("logo.png"));
  assert.equal(logo.alt, "");
});

test("result writes translations, definitions and examples as text, not markup", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "<b>bold</b>",
    meanings: [{ definitions: ['<img src=x onerror="alert(1)">'] }],
    example: "<script>alert(1)</script>",
  });
  const html = node.outerHTML;
  assert.ok(html.includes("&lt;b&gt;bold&lt;/b&gt;"));
  assert.ok(html.includes("&lt;img src=x onerror=&quot;alert(1)&quot;&gt;"));
  assert.ok(html.includes("&lt;script&gt;alert(1)&lt;/script&gt;"));
  assert.ok(!html.includes("<b>"));
  assert.ok(!html.includes("<img"));
  assert.ok(!html.includes("<script"));
});

test("result clears whatever the target held before", () => {
  const node = target();
  node.appendChild(env.document.createElement("p"));
  Glossy.render.result(node, { kind: "sentence", translation: "Hello" });
  assert.deepEqual(classesOf(node), ["translation", "tools", "tool", "tool"]);
  assert.equal(node.firstChild.className, "translation");
});

test("loading replaces the target with a busy skeleton of three bars", () => {
  const node = target();
  node.appendChild(env.document.createElement("p"));
  Glossy.render.loading(node);
  const skeleton = findByClass(node, "skeleton");
  assert.equal(skeleton.getAttribute("aria-busy"), "true");
  assert.equal(skeleton.childNodes.length, 3);
  assert.equal(node.childNodes.length, 1);
  assert.ok(skeleton.childNodes.every((child) => child.tagName === "SPAN"));
});

test("error shows the localized headline over the raw detail and a retry button that calls back", () => {
  const node = target();
  let clicks = 0;
  Glossy.render.error(node, "Network is down", () => {
    clicks += 1;
  });
  assert.equal(findByClass(node, "message").textContent, "Translation failed.");
  assert.equal(findByClass(node, "note").textContent, "Network is down");
  const retry = findByClass(node, "ghost-button");
  assert.equal(retry.textContent, "Try again");
  assert.equal(retry.type, "button");
  assert.equal(retry.dispatch("click"), 1);
  assert.equal(clicks, 1);
});

test("error falls back to the failure text and hides retry without a callback", () => {
  const node = target();
  Glossy.render.error(node, "");
  assert.equal(findByClass(node, "message").textContent, "Translation failed.");
  assert.equal(findByClass(node, "ghost-button"), null);
  Glossy.render.error(node, "boom", "not a function");
  assert.equal(findByClass(node, "ghost-button"), null);
});

test("error never repeats the headline as its own detail", () => {
  const node = target();
  Glossy.render.error(node, "Translation failed.", () => {});
  assert.equal(findByClass(node, "message").textContent, "Translation failed.");
  assert.equal(findByClass(node, "note"), null);
});

test("error localizes its retry label", () => {
  i18n.set("zh");
  const node = target();
  Glossy.render.error(node, "", () => {});
  assert.equal(findByClass(node, "message").textContent, "翻译失败。");
  assert.equal(findByClass(node, "ghost-button").textContent, "重试");
  i18n.set("en");
});

test("error names the engine that refused, and lets another be chosen", () => {
  const node = target();
  const asked = [];
  Glossy.render.error(node, "boom", () => {}, {
    provider: "baidu",
    onChooseService: (button) => asked.push(button),
  });

  const choose = findByClass(node, "service");
  assert.ok(choose, "the failure card offers another engine");
  assert.equal(choose.textContent, "Baidu Translate");
  choose.dispatch("click");
  assert.equal(asked.length, 1);
});

test("error leaves the engine off a card that was not told one", () => {
  const node = target();
  Glossy.render.error(node, "boom", () => {});

  assert.equal(findByClass(node, "foot"), null);
});

test("clear removes every child and is safe on an empty target", () => {
  const node = target();
  node.appendChild(env.document.createElement("p"));
  node.appendChild(env.document.createElement("p"));
  Glossy.render.clear(node);
  assert.equal(node.childNodes.length, 0);
  assert.equal(node.firstChild, null);
  Glossy.render.clear(node);
  assert.equal(node.childNodes.length, 0);
});

test("result draws the inflections of a word", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "跑",
    forms: [
      { tag: "past", text: "ran" },
      { tag: "plural", text: "" },
      { tag: "other", text: "runs" },
    ],
  });
  const forms = findByClass(node, "forms");
  assert.ok(forms);
  assert.equal(findByClass(forms, "block-title").textContent, "Forms");
  const rows = descendantsOf(forms, []).filter((child) => child.className === "form");
  assert.equal(rows.length, 2);
  assert.equal(rows[0].childNodes[0].textContent, "past");
  assert.equal(rows[0].childNodes[1].textContent, "ran");
  assert.equal(rows[1].textContent, "other formruns");
});

test("result leaves the inflections out when there are none", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", forms: [] });
  assert.equal(findByClass(node, "forms"), null);
});

test("result draws the synonyms of a word", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", synonyms: ["tod", "reynard"] });
  assert.equal(findByClass(node, "synonym-list").textContent, "tod · reynard");
});

test("result draws the sentence a word came from", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    translation: "跑",
    context: { text: "He kept running.", translation: "他一直在跑。" },
  });
  const block = findByClass(node, "context");
  assert.ok(block);
  assert.equal(findByClass(block, "context-source").textContent, "He kept running.");
  assert.equal(findByClass(block, "context-translation").textContent, "他一直在跑。");
});

test("result leaves the context out when it has no original", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", context: { translation: "x" } });
  assert.equal(findByClass(node, "context"), null);
});

test("result draws the aligned pairs of a sentence", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "整段",
    pairs: [
      { source: "One.", translation: "一。" },
      { source: "Two.", translation: "二。" },
    ],
  });
  const pairs = findByClass(node, "pairs");
  assert.ok(pairs);
  const rows = descendantsOf(pairs, []).filter((child) => child.className === "pair");
  assert.equal(rows.length, 2);
  assert.equal(findByClass(rows[0], "pair-source").textContent, "One.");
  assert.equal(findByClass(rows[0], "pair-translation").textContent, "一。");
});

test("result ignores a single pair, which repeats the whole card", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "整段",
    pairs: [{ source: "One.", translation: "一。" }],
  });
  assert.equal(findByClass(node, "pairs"), null);
});

test("the compact card keeps the translation and drops the extras", () => {
  const node = target();
  Glossy.render.result(
    node,
    {
      kind: "word",
      translation: "跑",
      phonetic: "ˈrəniNG",
      meanings: [{ partOfSpeech: "noun", definitions: ["跑步"] }],
      example: "keep running",
      forms: [{ tag: "past", text: "ran" }],
      synonyms: ["jogging"],
      context: { text: "He kept running." },
      conversions: [{ category: "length", original: "1 ft", converted: "0.3 m" }],
    },
    { compact: true },
  );
  assert.equal(findByClass(node, "phonetic").textContent, "/ˈrəniNG/");
  assert.ok(findByClass(node, "meanings"));
  assert.equal(findByClass(node, "example"), null);
  assert.equal(findByClass(node, "forms"), null);
  assert.equal(findByClass(node, "synonyms"), null);
  assert.equal(findByClass(node, "context"), null);
  assert.equal(findByClass(node, "units"), null);
  assert.ok(findByClass(node, "translation"));
  assert.equal(toolsOf(node).length, 1);
});

test("result draws the buttons of each side under the text they act on", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });
  const rows = toolsOf(node);
  assert.equal(rows.length, 2);

  // Copy first, then pronunciation, so one row is one side's pair of actions.
  const original = rows[0].childNodes;
  assert.equal(original.length, 2);
  assert.equal(original[0].getAttribute("data-copy"), "render.copyOriginal");
  assert.equal(original[1].getAttribute("data-say"), "render.speakOriginal");

  const translation = rows[1].childNodes;
  assert.equal(translation.length, 2);
  assert.equal(translation[0].getAttribute("data-copy"), "render.copyTranslation");
  assert.equal(translation[1].getAttribute("data-say"), "render.speakTranslation");

  // Each row stands under the text it acts on, which is the row above it.
  const classes = classesOf(node);
  assert.equal(classes[classes.indexOf("original") + 1], "tools");
  assert.equal(classes[classes.indexOf("translation") + 1], "tools");
});

test("the buttons of a side are icons that carry their own accessible name", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });
  const buttons = toolsOf(node)[0].childNodes;

  // The buttons are icons, so the label lives on the accessible name instead of
  // on the face of the button.
  assert.equal(buttons[0].textContent, "");
  assert.equal(buttons[0].getAttribute("aria-label"), "Copy the original");
  assert.equal(buttons[0].getAttribute("title"), "Copy the original");
  assert.ok(buttons[0].innerHTML.includes("glyph-copy"));
  assert.ok(buttons[0].innerHTML.includes("glyph-done"));
  assert.equal(buttons[0].type, "button");

  assert.equal(buttons[1].textContent, "");
  assert.equal(buttons[1].getAttribute("aria-label"), "Read the original out loud");
  assert.equal(buttons[1].getAttribute("title"), "Read the original out loud");
  assert.equal(buttons[1].getAttribute("aria-pressed"), "false");
  assert.ok(buttons[1].innerHTML.includes("glyph-speak"));
  assert.ok(buttons[1].innerHTML.includes("glyph-stop"));
  assert.equal(buttons[1].type, "button");
});

test("a word card puts the buttons of the original above the translation", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "word",
    sourceText: "fox",
    translation: "狐狸",
    provider: "baidu",
  });
  // The word itself stands in the header, so the row of the original opens the
  // body of the card, ahead of the translation.
  assert.deepEqual(classesOf(node), [
    "original",
    "tools",
    "tool",
    "tool",
    "translation",
    "tools",
    "tool",
    "tool",
    "foot",
    "engine",
    "brand",
    "brand-logo",
    "brand-name",
  ]);
});

test("the buttons of a side keep to their own side of the card", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello.",
    translation: "你好。",
    provider: "baidu",
  });
  assert.deepEqual(classesOf(node), [
    "original",
    "tools",
    "tool",
    "tool",
    "translation",
    "tools",
    "tool",
    "tool",
    "foot",
    "engine",
    "brand",
    "brand-logo",
    "brand-name",
  ]);
});

test("a side with nothing to read out has no buttons", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "", sourceText: "" });
  assert.deepEqual(toolsOf(node), []);
});

/** An environment whose backend answers the way the test tells it to. */
function backend(invoke) {
  const fresh = newEnv();
  fresh.Glossy.invoke = invoke;
  return fresh;
}

/** A sentence card in its own environment, answered with its own buttons. */
function cardButtons(fresh) {
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });
  // Every row holds the copy button first and the speaker second, so the two
  // speakers come back in the order the sides sit in.
  return toolsOf(node).map((row) => row.childNodes[1]);
}

/** The copy buttons of the same card, in the same order. */
function cardCopyButtons(fresh) {
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });
  return toolsOf(node).map((row) => row.childNodes[0]);
}

test("pressing a pronunciation button reads that side out loud", () => {
  const calls = [];
  const fresh = backend((command, payload) => {
    calls.push({ command, payload });
    return Promise.resolve(command === "speaking");
  });
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");

  const said = calls.filter((call) => call.command === "say");
  assert.equal(said.length, 1);
  assert.equal(said[0].payload.text, "Hello there.");
  assert.equal(said[0].payload.language, "en");
  assert.equal(buttons[0].dataset.state, "on");
  assert.equal(buttons[0].getAttribute("aria-pressed"), "true");
  assert.equal(buttons[0].getAttribute("aria-label"), "Stop reading");
  // The buttons read in the same order they sit in, so the second one is the
  // translation's.
  assert.notEqual(buttons[1].dataset.state, "on");
});

test("pressing the button that is reading stops it instead of starting over", () => {
  const calls = [];
  const fresh = backend((command) => {
    calls.push(command);
    return Promise.resolve(command === "speaking");
  });
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");
  buttons[0].dispatch("click");

  assert.deepEqual(calls, ["stop_speaking", "say", "stop_speaking"]);
  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[0].getAttribute("aria-pressed"), "false");
  assert.equal(buttons[0].getAttribute("aria-label"), "Read the original out loud");
});

test("reading the other side of the card takes the highlight with it", () => {
  const fresh = backend((command) => Promise.resolve(command === "speaking"));
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");
  buttons[1].dispatch("click");

  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[1].dataset.state, "on");
  assert.equal(buttons[0].getAttribute("aria-label"), "Read the original out loud");
  assert.equal(buttons[1].getAttribute("aria-label"), "Stop reading");
});

test("the button goes back to rest once the voice falls silent", async () => {
  let busy = true;
  const fresh = backend((command) => Promise.resolve(command === "speaking" ? busy : undefined));
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");
  assert.equal(fresh.timers.intervals(), 1);

  await fresh.timers.fire();
  assert.equal(buttons[0].dataset.state, "on");

  busy = false;
  await fresh.timers.fire();
  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[0].getAttribute("aria-label"), "Read the original out loud");
  assert.equal(fresh.timers.intervals(), 0);
});

test("a reading that cannot start says so on its button", async () => {
  const fresh = backend((command) =>
    command === "say" ? Promise.reject(new Error("no voice")) : Promise.resolve(true),
  );
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");
  await Promise.resolve();
  await Promise.resolve();

  assert.equal(buttons[0].dataset.state, "failed");
  assert.equal(buttons[0].getAttribute("aria-label"), "Could not read this out loud");
  assert.equal(buttons[0].getAttribute("data-say"), "render.speakOriginal");
  assert.equal(fresh.timers.intervals(), 0);
});

test("a card that is replaced stops the reading behind it", () => {
  const calls = [];
  const fresh = backend((command) => {
    calls.push(command);
    return Promise.resolve(command === "speaking");
  });
  const buttons = cardButtons(fresh);

  buttons[0].dispatch("click");
  fresh.Glossy.render.result(fresh.document.createElement("div"), {
    kind: "sentence",
    translation: "Hello.",
  });

  assert.equal(calls[calls.length - 1], "stop_speaking");
  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(fresh.timers.intervals(), 0);
});

test("pressing a copy button sends that side of the card to the clipboard", () => {
  const calls = [];
  const fresh = backend((command, payload) => {
    calls.push({ command, payload });
    return Promise.resolve(true);
  });
  const buttons = cardCopyButtons(fresh);

  buttons[0].dispatch("click");

  const copied = calls.filter((call) => call.command === "copy_text");
  assert.equal(copied.length, 1);
  assert.equal(copied[0].payload.text, "Hello there.");
  assert.equal(buttons[0].dataset.state, "copied");
  assert.equal(buttons[0].getAttribute("aria-label"), "Copied");
  assert.equal(buttons[1].dataset.state, undefined);

  buttons[1].dispatch("click");
  assert.equal(calls.filter((call) => call.command === "copy_text")[1].payload.text, "你好。");
});

test("the tick goes back to the sheets once the moment has passed", async () => {
  const fresh = backend(() => Promise.resolve(true));
  const buttons = cardCopyButtons(fresh);

  buttons[0].dispatch("click");
  assert.equal(buttons[0].dataset.state, "copied");

  await fresh.timers.fireTimeouts();
  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[0].getAttribute("aria-label"), "Copy the original");
});

test("only the button that copied last wears the tick", async () => {
  const fresh = backend(() => Promise.resolve(true));
  const buttons = cardCopyButtons(fresh);

  buttons[0].dispatch("click");
  buttons[1].dispatch("click");

  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[1].dataset.state, "copied");
  assert.equal(buttons[0].getAttribute("aria-label"), "Copy the original");
});

test("a copy that reached the clipboard tells the caller which side it was", () => {
  const sides = [];
  const fresh = backend(() => Promise.resolve(true));
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(
    node,
    { kind: "sentence", sourceText: "Hello there.", translation: "你好。" },
    { onCopied: (side) => sides.push(side) },
  );
  const buttons = toolsOf(node).map((row) => row.childNodes[0]);

  buttons[1].dispatch("click");
  assert.deepEqual(sides, ["translation"]);
});

test("a card that can be written back over offers the button on the translation side", () => {
  const fresh = backend(() => Promise.resolve(true));
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(
    node,
    {
      kind: "sentence",
      sourceText: "Hello there.",
      translation: "你好。",
      sourceLang: "en",
      targetLang: "zh-CN",
    },
    { overwrite: true },
  );
  const rows = toolsOf(node);

  // The original has no place to be written into, so only the translation side
  // carries the third button.
  assert.equal(rows.length, 2);
  assert.equal(rows[0].childNodes.length, 2);
  assert.equal(rows[1].childNodes.length, 3);
  assert.equal(
    rows[1].childNodes[2].getAttribute("aria-label"),
    "Replace the original text (Ctrl+Enter)",
  );
});

test("a card with nothing behind it offers no button to write back", () => {
  const fresh = backend(() => Promise.resolve(true));
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });

  assert.equal(toolsOf(node)[1].childNodes.length, 2);
});

test("pressing that button sends the translation to be written over the original", async () => {
  const calls = [];
  const fresh = backend((command, payload) => {
    calls.push({ command, payload });
    return Promise.resolve(true);
  });
  const node = fresh.document.createElement("div");
  fresh.Glossy.render.result(
    node,
    {
      kind: "sentence",
      sourceText: "Hello there.",
      translation: "你好。",
      sourceLang: "en",
      targetLang: "zh-CN",
    },
    { overwrite: true },
  );
  const button = toolsOf(node)[1].childNodes[2];

  button.dispatch("click");
  await Promise.resolve();

  const written = calls.filter((call) => call.command === "replace_selection");
  assert.equal(written.length, 1);
  assert.equal(written[0].payload.text, "你好。");
  assert.equal(button.dataset.state, "copied");
  assert.equal(button.getAttribute("aria-label"), "Original replaced");
});

test("the accelerator writes the translation back without a button of its own", () => {
  const calls = [];
  const fresh = backend((command, payload) => {
    calls.push({ command, payload });
    return Promise.resolve(true);
  });
  fresh.Glossy.render.result(
    fresh.document.createElement("div"),
    {
      kind: "sentence",
      sourceText: "Hello there.",
      translation: "你好。",
      sourceLang: "en",
      targetLang: "zh-CN",
    },
    { overwrite: true },
  );

  fresh.Glossy.render.overwrite("你好。");

  const written = calls.filter((call) => call.command === "replace_selection");
  assert.equal(written.length, 1);
  assert.equal(written[0].payload.text, "你好。");
});

test("a card that is replaced takes the tick with it", () => {
  const fresh = backend(() => Promise.resolve(true));
  const buttons = cardCopyButtons(fresh);

  buttons[0].dispatch("click");
  fresh.Glossy.render.result(fresh.document.createElement("div"), {
    kind: "sentence",
    translation: "Hello.",
  });

  assert.equal(buttons[0].dataset.state, "off");
  assert.equal(buttons[0].getAttribute("aria-label"), "Copy the original");
  assert.equal(fresh.timers.timeouts(), 0);
});

test("result names the service that did not answer, not the one that did", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "你好。",
    provider: "Youdao",
    fallbackFrom: "Baidu",
  });
  // The footer names the engine that answered; the line under it names the one
  // that did not, which is the service the settings asked for.
  assert.equal(findByClass(node, "foot fallback").textContent, "Baidu Translate did not answer");
  assert.equal(findByClass(node, "engine").textContent, "Youdao Translate");
});

test("the line says why the engine stepped aside, when the relay told it", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "你好。",
    provider: "youdao",
    fallbackFrom: "baidu",
    fallbackCode: "upstream_limit",
  });
  assert.equal(
    findByClass(node, "foot fallback").textContent,
    "Baidu Translate did not answer — its allowance is used up",
  );
});

test("a refusal code this build does not know adds nothing to the line", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "你好。",
    provider: "youdao",
    fallbackFrom: "baidu",
    fallbackCode: "something_new",
  });
  assert.equal(findByClass(node, "foot fallback").textContent, "Baidu Translate did not answer");
});

test("a relay that never answered says so on the line", () => {
  // The app falls back by itself too, and the reason there is the relay's: the
  // sentence has to be about the relay rather than about the engine named in
  // the footer, which never got the request at all.
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "Hello.",
    provider: "google",
    fallbackFrom: "youdao",
    fallbackCode: "relay_unreachable",
  });
  assert.equal(
    findByClass(node, "foot fallback").textContent,
    "Youdao Translate did not answer — the Glossy relay could not be reached",
  );
});

test("result says nothing about a fallback when none happened", () => {
  const node = target();
  Glossy.render.result(node, { kind: "sentence", translation: "你好。", provider: "Baidu" });
  assert.equal(findByClass(node, "foot fallback"), null);
});

test("the name of the engine is a button when the card can switch services", () => {
  const node = target();
  const asked = [];
  Glossy.render.result(
    node,
    { kind: "word", translation: "狐狸", provider: "youdao" },
    { onChooseService: (button) => asked.push(button) },
  );

  const button = findByClass(node, "service");
  assert.ok(button, "the footer name is not a button");
  assert.equal(button.textContent, "Youdao Translate");
  assert.equal(button.getAttribute("aria-haspopup"), "true");
  // Collapsed until the list is really there, which is what a screen reader
  // reads out before the click.
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.getAttribute("title"), "Translation service");
  // The row holds the switchable name and the mark of the app, nothing else.
  assert.deepEqual(classesOf(findByClass(node, "foot")), [
    "service",
    "brand",
    "brand-logo",
    "brand-name",
  ]);

  button.dispatch("click");
  assert.equal(asked.length, 1);
  assert.equal(asked[0], button);
});

test("the name of the engine stays plain text without a way to switch", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });

  assert.equal(findByClass(node, "service"), null);
  assert.equal(findByClass(node, "engine").textContent, "Baidu Translate");
});

/** The codes of one table in the Rust module that decides them. */
function rustTable(name) {
  const pattern = new RegExp(
    `const ${name}: \\[&str; \\d+\\] = \\[([\\s\\S]*?)\\];`
  );
  const block = pattern.exec(fs.readFileSync(RUST_LANGUAGES, "utf8"));
  assert.ok(block, `languages.rs no longer declares ${name}`);
  return block[1]
    .split(",")
    .map((piece) => /"([^"]+)"/.exec(piece))
    .filter(Boolean)
    .map((match) => match[1]);
}

test("an engine the tables do not mention keeps every language", () => {
  const fresh = newEnv().Glossy;
  assert.deepEqual(
    Array.from(fresh.languagesFor("cloud-baidu")),
    Array.from(fresh.languageCodes)
  );
  assert.ok(fresh.servesLanguage("cloud-baidu", "tr"));
});

test("the languages of an engine are its table, in the order of the menu", () => {
  const fresh = newEnv().Glossy;
  // The table is written in the order a provider lists them, not the menu's.
  fresh.setServiceLanguages([
    { id: "cloud-baidu", languages: ["sv", "tr", "en", "zh-CN", "ceb"] },
  ]);
  assert.deepEqual(
    Array.from(fresh.languagesFor("cloud-baidu")),
    ["en", "zh-CN", "tr", "sv"]
  );
  assert.ok(fresh.servesLanguage("cloud-baidu", "en"));
  // A language the engine takes but the menu never had is still taken; it is
  // only the menu that holds no entry for it.
  assert.ok(fresh.servesLanguage("cloud-baidu", "ceb"));
  assert.ok(!fresh.servesLanguage("cloud-baidu", "de"));
  assert.deepEqual(
    Array.from(fresh.languagesFor("google")),
    Array.from(fresh.languageCodes)
  );
  assert.ok(fresh.servesLanguage("google", "tr"));
});

test("an engine with no languages in the tables keeps the full list", () => {
  const fresh = newEnv().Glossy;
  fresh.setServiceLanguages([
    { id: "cloud-youdao", languages: [] },
    { id: "", languages: ["en"] },
    { languages: ["en"] },
    null,
  ]);
  assert.deepEqual(
    Array.from(fresh.languagesFor("cloud-youdao")),
    Array.from(fresh.languageCodes)
  );
  assert.ok(fresh.servesLanguage("cloud-youdao", "tr"));
});

test("the language list of the backend is the list the menus offer", () => {
  assert.deepEqual(rustTable("ALL"), Array.from(newEnv().Glossy.languageCodes));
});

test("the languages Baidu leaves out are the eight the standard plan refuses", () => {
  // 有道 and Google both translate all 31, so Baidu alone decides this.
  const served = rustTable("BAIDU");
  const left = Array.from(newEnv().Glossy.languageCodes).filter(
    (code) => served.indexOf(code) === -1
  );
  assert.deepEqual(left, ["uk", "tr", "hi", "id", "ms", "he", "no", "sk"]);
});

