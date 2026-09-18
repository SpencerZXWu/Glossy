"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");

const { loadFrontend } = require("./helpers/load-scripts.js");
const { findByClass, classesOf } = require("./helpers/fake-dom.js");

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
  for (const name of ["loading", "error", "result", "clear"]) {
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
  assert.equal(findByClass(node, "translation").textContent, "No translation returned.");
  assert.equal(findByClass(node, "phonetic"), null);
  assert.equal(findByClass(node, "foot"), null);
});

test("result survives a missing or null payload", () => {
  const node = target();
  Glossy.render.result(node, null);
  assert.equal(findByClass(node, "translation").textContent, "No translation returned.");
  assert.deepEqual(classesOf(node), ["translation"]);
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
  assert.deepEqual(classesOf(node), ["translation"]);
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
  assert.deepEqual(classesOf(node), ["translation"]);
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

test("error shows the message and a retry button that calls back", () => {
  const node = target();
  let clicks = 0;
  Glossy.render.error(node, "Network is down", () => {
    clicks += 1;
  });
  assert.equal(findByClass(node, "message").textContent, "Network is down");
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
