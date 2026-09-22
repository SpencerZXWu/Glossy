"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");

const { loadFrontend } = require("./helpers/load-scripts.js");
const { findByClass, classesOf, descendantsOf } = require("./helpers/fake-dom.js");

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
  assert.equal(findByClass(node, "foot").textContent, "Google");
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

test("result never renders the original for a word", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", sourceText: "fox", translation: "狐狸" });
  assert.equal(findByClass(node, "original"), null);
  assert.equal(findByClass(node, "translation").textContent, "狐狸");
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
  assert.deepEqual(classesOf(node), ["translation", "says", "say"]);
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
  assert.equal(findByClass(node, "foot").textContent, "Baidu Translate");
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
  assert.equal(findByClass(node, "foot").textContent, "Baidu Translate");
  i18n.set("zh");
  Glossy.render.result(node, { kind: "word", translation: "狐狸", provider: "baidu" });
  assert.equal(findByClass(node, "foot").textContent, "百度翻译");
  i18n.set("en");
});

test("result omits the footer without a provider", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "fox", provider: "" });
  assert.equal(findByClass(node, "foot"), null);
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
  assert.deepEqual(classesOf(node), ["translation", "says", "say"]);
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
  assert.ok(findByClass(node, "says"));
});

test("result draws a pronunciation button for each side of the card", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    sourceText: "Hello there.",
    translation: "你好。",
    sourceLang: "en",
    targetLang: "zh-CN",
  });
  const says = findByClass(node, "says");
  const buttons = says.childNodes;
  assert.equal(buttons.length, 2);
  // The buttons are icons, so the label lives on the accessible name instead of
  // on the face of the button.
  assert.equal(buttons[0].textContent, "");
  assert.equal(buttons[0].getAttribute("aria-label"), "Read the original out loud");
  assert.equal(buttons[0].getAttribute("title"), "Read the original out loud");
  assert.equal(buttons[0].getAttribute("aria-pressed"), "false");
  assert.ok(buttons[0].innerHTML.includes("glyph-speak"));
  assert.ok(buttons[0].innerHTML.includes("glyph-stop"));
  assert.equal(buttons[0].getAttribute("data-say"), "render.speakOriginal");
  assert.equal(buttons[1].getAttribute("data-say"), "render.speakTranslation");
  assert.equal(buttons[1].getAttribute("aria-label"), "Read the translation out loud");
  assert.equal(buttons[0].type, "button");
});

test("the pronunciation buttons come after the rest of the card", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "你好。",
    sourceText: "Hello.",
    provider: "Baidu",
  });
  const classes = classesOf(node);
  assert.ok(classes.indexOf("foot") < classes.indexOf("says"));
  assert.equal(classes[classes.length - 1], "say");
});

test("a card with nothing to read out has no pronunciation buttons", () => {
  const node = target();
  Glossy.render.result(node, { kind: "word", translation: "", sourceText: "" });
  assert.equal(findByClass(node, "says"), null);
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
  return findByClass(node, "says").childNodes;
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

test("result names the service that answered after a fallback", () => {
  const node = target();
  Glossy.render.result(node, {
    kind: "sentence",
    translation: "你好。",
    provider: "Youdao",
    fallbackFrom: "Baidu",
  });
  assert.equal(
    findByClass(node, "foot fallback").textContent,
    "Answered by Baidu Translate after the chosen service failed",
  );
});

test("result says nothing about a fallback when none happened", () => {
  const node = target();
  Glossy.render.result(node, { kind: "sentence", translation: "你好。", provider: "Baidu" });
  assert.equal(findByClass(node, "foot fallback"), null);
});
