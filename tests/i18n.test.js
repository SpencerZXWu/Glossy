"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");

const {
  loadFrontend,
  extractDictionaryKeys,
  readI18nSource,
  placeholdersIn,
} = require("./helpers/load-scripts.js");

const SOURCE = readI18nSource();
const EN_KEYS = extractDictionaryKeys(SOURCE, "ENGLISH");
const ZH_KEYS = extractDictionaryKeys(SOURCE, "CHINESE");

const env = loadFrontend({ files: ["i18n.js"] });
const i18n = env.Glossy.i18n;

function dictionary(language) {
  i18n.set(language);
  const entries = {};
  for (const key of EN_KEYS) entries[key] = i18n.t(key);
  return entries;
}

const ENGLISH = dictionary("en");
const CHINESE = dictionary("zh");
i18n.set("en");

function valueReport(keys, values) {
  return keys
    .filter((key) => {
      const value = values[key];
      return typeof value !== "string" || value.trim() === "" || value === key;
    })
    .map((key) => `${key}=${JSON.stringify(values[key])}`);
}

test("the key extractor really sees both dictionaries", () => {
  assert.ok(EN_KEYS.length >= 100, `only extracted ${EN_KEYS.length} English keys`);
  assert.ok(ZH_KEYS.length >= 100, `only extracted ${ZH_KEYS.length} Chinese keys`);
  assert.ok(EN_KEYS.includes("render.failed"));
  assert.ok(ZH_KEYS.includes("render.failed"));
});

test("every English key also exists in the Chinese dictionary", () => {
  const chinese = new Set(ZH_KEYS);
  assert.deepEqual(EN_KEYS.filter((key) => !chinese.has(key)), []);
});

test("every Chinese key also exists in the English dictionary", () => {
  const english = new Set(EN_KEYS);
  assert.deepEqual(ZH_KEYS.filter((key) => !english.has(key)), []);
});

test("both dictionaries have the same size and no duplicate keys", () => {
  assert.equal(EN_KEYS.length, ZH_KEYS.length);
  assert.equal(new Set(EN_KEYS).size, EN_KEYS.length);
  assert.equal(new Set(ZH_KEYS).size, ZH_KEYS.length);
});

test("dictionary keys use the namespace.name convention", () => {
  // A trailing segment may carry dashes, since a service id is spelled that way
  // ("service.cloud-baidu").
  const malformed = EN_KEYS.filter((key) => !/^[a-z][a-zA-Z0-9]*(\.[a-zA-Z0-9]+[a-zA-Z0-9-]*)+$/.test(key));
  assert.deepEqual(malformed, []);
});

test("no English value is empty, whitespace or a bare key echo", () => {
  assert.deepEqual(valueReport(EN_KEYS, ENGLISH), []);
});

test("no Chinese value is empty, whitespace or a bare key echo", () => {
  assert.deepEqual(valueReport(EN_KEYS, CHINESE), []);
});

test("both languages agree on which keys take placeholders", () => {
  const mismatched = EN_KEYS.filter(
    (key) =>
      JSON.stringify(placeholdersIn(ENGLISH[key])) !== JSON.stringify(placeholdersIn(CHINESE[key])),
  ).map((key) => `${key}: en=${ENGLISH[key]} zh=${CHINESE[key]}`);
  assert.deepEqual(mismatched, []);
});

test("placeholder indices run from {0} without gaps", () => {
  for (const values of [ENGLISH, CHINESE]) {
    for (const key of EN_KEYS) {
      const indices = placeholdersIn(values[key]);
      const expected = indices.map((_, index) => index);
      assert.deepEqual(indices, expected, `placeholders of ${key} are ${indices}`);
    }
  }
});

test("no value contains a malformed placeholder", () => {
  const broken = [];
  for (const key of EN_KEYS) {
    for (const values of [ENGLISH, CHINESE]) {
      const withoutPlaceholders = values[key].replace(/\{\d+\}/g, "");
      if (withoutPlaceholders.includes("{") || withoutPlaceholders.includes("}")) {
        broken.push(`${key}=${values[key]}`);
      }
    }
  }
  assert.deepEqual(broken, []);
});

test("resolve passes an explicit en or zh preference straight through", () => {
  env.setLocale("en-US", undefined);
  assert.equal(i18n.resolve("zh"), "zh");
  env.setLocale("zh-CN", undefined);
  assert.equal(i18n.resolve("en"), "en");
});

test("resolve reads the system locale without a preference", () => {
  env.setLocale("zh-CN", undefined);
  assert.equal(i18n.resolve(undefined), "zh");
  env.setLocale("ZH-Hans", undefined);
  assert.equal(i18n.resolve(undefined), "zh");
  env.setLocale("en-GB", undefined);
  assert.equal(i18n.resolve(undefined), "en");
  env.setLocale("fr-FR", undefined);
  assert.equal(i18n.resolve(undefined), "en");
});

test("resolve prefers the first entry of navigator.languages", () => {
  env.setLocale("en-US", ["zh-TW", "en-US"]);
  assert.equal(i18n.resolve(undefined), "zh");
  env.setLocale("zh-CN", ["fr-FR", "zh-CN"]);
  assert.equal(i18n.resolve(undefined), "en");
});

test("resolve falls back to en when the navigator exposes nothing useful", () => {
  env.setLocale("", [""]);
  assert.equal(i18n.resolve(undefined), "en");
  env.setLocale("", undefined);
  assert.equal(i18n.resolve(undefined), "en");
});

test("resolve treats any other preference as follow-the-system", () => {
  env.setLocale("zh-CN", undefined);
  assert.equal(i18n.resolve("system"), "zh");
  env.setLocale("en-US", undefined);
  assert.equal(i18n.resolve("fr"), "en");
  env.setLocale("en-US", undefined);
});

test("a freshly loaded module starts in English", () => {
  const fresh = loadFrontend({ files: ["i18n.js"] });
  assert.equal(fresh.Glossy.i18n.language(), "en");
  assert.equal(fresh.Glossy.i18n.t("status.paused"), "Paused");
});

test("set switches the table used by t", () => {
  assert.equal(i18n.set("en"), "en");
  assert.equal(i18n.t("status.paused"), "Paused");
  assert.equal(i18n.set("zh"), "zh");
  assert.equal(i18n.t("status.paused"), "已暂停");
  assert.equal(i18n.t("popup.close"), "关闭");
  i18n.set("en");
  assert.equal(i18n.t("status.paused"), "Paused");
});

test("set maps a system preference through resolve", () => {
  env.setLocale("zh-TW", ["zh-TW"]);
  assert.equal(i18n.set("system"), "zh");
  assert.equal(i18n.t("status.listening"), "监听中");
  env.setLocale("en-US", undefined);
  assert.equal(i18n.set("system"), "en");
  assert.equal(i18n.t("status.listening"), "Listening");
});

test("set records the language on the document element", () => {
  i18n.set("zh");
  assert.equal(env.document.documentElement.lang, "zh-CN");
  i18n.set("en");
  assert.equal(env.document.documentElement.lang, "en");
});

test("t returns the key itself when the key is unknown", () => {
  assert.equal(i18n.t("does.not.exist"), "does.not.exist");
  i18n.set("zh");
  assert.equal(i18n.t("also.missing"), "also.missing");
  i18n.set("en");
});

test("t substitutes positional placeholders", () => {
  assert.equal(i18n.t("ignored.picked", "notepad.exe"), "Added notepad.exe.");
  assert.equal(i18n.t("ignored.count", 3), "3 program(s) ignored.");
  assert.equal(
    i18n.t("hotkey.active", "Ctrl+Alt+C"),
    "Active: Ctrl+Alt+C. Press it to translate the clipboard content.",
  );
  assert.equal(i18n.t("ignored.count", 0), "0 program(s) ignored.");
});

test("t leaves placeholders without a matching argument alone", () => {
  assert.equal(i18n.t("ignored.count"), "{0} program(s) ignored.");
  assert.equal(i18n.t("ignored.picked", "a", "b"), "Added a.");
});

test("t ignores extra arguments and substitutes inside Chinese text", () => {
  i18n.set("zh");
  assert.equal(i18n.t("ignored.picked", "notepad.exe"), "已添加 notepad.exe。");
  assert.equal(i18n.t("ignored.count", 2, 3, 4), "已忽略 2 个程序。");
  i18n.set("en");
});

test("providerName is case-insensitive and localized", () => {
  i18n.set("en");
  assert.equal(i18n.providerName("GOOGLE"), "Google");
  i18n.set("zh");
  assert.equal(i18n.providerName("baidu"), "百度翻译");
  assert.equal(i18n.providerName("Youdao"), "有道翻译");
  i18n.set("en");
});

test("providerName falls back to the raw id", () => {
  assert.equal(i18n.providerName("unknown"), "unknown");
  assert.equal(i18n.providerName(""), "");
  assert.equal(i18n.providerName(null), "");
  i18n.set("zh");
  assert.equal(i18n.providerName(undefined), "");
  assert.equal(i18n.providerName("unknown"), "unknown");
  i18n.set("en");
});

test("languageName uses the localized table when Chinese is active", () => {
  i18n.set("en");
  assert.equal(i18n.languageName("ja"), "ja");
  i18n.set("zh");
  assert.equal(i18n.languageName("ja"), "日语");
  assert.equal(i18n.languageName("zh-CN"), "简体中文");
  i18n.set("en");
});

test("languageName calls the fallback with the code when the table misses", () => {
  const fallback = (code) => `fallback:${String(code)}`;
  assert.equal(i18n.languageName("zz", fallback), "fallback:zz");
  assert.equal(i18n.languageName("", fallback), "fallback:");
  i18n.set("zh");
  assert.equal(i18n.languageName("zz", fallback), "fallback:zz");
  assert.equal(i18n.languageName("ja", fallback), "日语");
  i18n.set("en");
});

test("languageName returns the code as-is when no fallback is given", () => {
  assert.equal(i18n.languageName("zz"), "zz");
  assert.equal(i18n.languageName(""), "");
  assert.equal(i18n.languageName(null), "");
});

function attach(dataset) {
  const element = env.document.createElement("div");
  Object.assign(element.dataset, dataset);
  env.document.body.appendChild(element);
  return element;
}

test("apply fills text, placeholders and titles from the dictionary", () => {
  const text = attach({ i18n: "status.paused" });
  const placeholder = attach({ i18nPlaceholder: "history.search" });
  const title = attach({ i18nTitle: "popup.copy" });
  const plain = attach({ i18n: "render.retry" });

  i18n.set("en");
  i18n.apply();

  assert.equal(text.textContent, "Paused");
  assert.equal(placeholder.placeholder, "Text or translation");
  assert.equal(title.title, "Copy translation");
  assert.equal(plain.placeholder, "");
  assert.equal(plain.title, "");
});

test("apply follows a language change on a second pass", () => {
  const text = attach({ i18n: "status.paused" });
  i18n.set("en");
  i18n.apply();
  assert.equal(text.textContent, "Paused");
  i18n.set("zh");
  i18n.apply();
  assert.equal(text.textContent, "已暂停");
  i18n.set("en");
});

test("apply leaves the key in place for an unknown key", () => {
  const element = attach({ i18n: "nope.not.here" });
  i18n.apply();
  assert.equal(element.textContent, "nope.not.here");
});

test("apply writes a value as text, not as markup", () => {
  // The echo an unknown key leaves behind is the one value that reaches the
  // page with HTML in it, so it is where the escaping shows.
  const element = attach({ i18n: "<code>30 000</code> characters" });
  i18n.apply();
  assert.equal(element.textContent, "<code>30 000</code> characters");
  assert.ok(element.outerHTML.includes("&lt;code&gt;"));
});

test("apply only walks the subtree it is given", () => {
  const root = env.document.createElement("section");
  const inside = env.document.createElement("span");
  inside.dataset.i18n = "status.paused";
  root.appendChild(inside);
  env.document.body.appendChild(root);
  const outside = attach({ i18n: "status.paused" });
  outside.innerHTML = "untouched";

  i18n.set("en");
  i18n.apply(root);

  assert.equal(inside.textContent, "Paused");
  assert.equal(outside.innerHTML, "untouched");
});
