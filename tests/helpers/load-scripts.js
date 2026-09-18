"use strict";

const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const { FakeDocument } = require("./fake-dom.js");

const JS_DIR = path.resolve(__dirname, "..", "..", "src", "js");
const DEFAULT_FILES = ["i18n.js", "render.js"];

function readScript(name) {
  const file = path.join(JS_DIR, name);
  return { name, file, code: fs.readFileSync(file, "utf8") };
}

function loadFrontend(options) {
  const settings = options || {};
  const scripts = (settings.files || DEFAULT_FILES).map(readScript);

  const document = new FakeDocument();
  const window = { Glossy: {} };
  const navigator = { language: settings.language || "en-US" };
  if (settings.languages) navigator.languages = settings.languages.slice();

  const context = vm.createContext({ window, document, navigator, console });

  function run() {
    for (const script of scripts) {
      vm.runInContext(script.code, context, { filename: script.file });
    }
  }

  run();

  return {
    Glossy: window.Glossy,
    window,
    document,
    navigator,
    setLocale(language, languages) {
      navigator.language = language;
      navigator.languages = languages ? languages.slice() : undefined;
    },
    rerun: run,
  };
}

function extractDictionaryKeys(source, name) {
  const lines = source.split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === `const ${name} = {`);
  if (start === -1) throw new Error(`Dictionary ${name} not found in the source`);
  const end = lines.findIndex((line, index) => index > start && line === "  };");
  if (end === -1) throw new Error(`Dictionary ${name} is not terminated by "  };"`);

  const keys = [];
  for (let index = start + 1; index < end; index += 1) {
    const match = /^\s{2,}"([^"]+)":/.exec(lines[index]);
    if (match) keys.push(match[1]);
  }
  return keys;
}

function readI18nSource() {
  return readScript("i18n.js").code;
}

function placeholdersIn(value) {
  const indices = [];
  const pattern = /\{(\d+)\}/g;
  let match = pattern.exec(String(value));
  while (match) {
    indices.push(Number(match[1]));
    match = pattern.exec(String(value));
  }
  return indices;
}

module.exports = {
  JS_DIR,
  loadFrontend,
  extractDictionaryKeys,
  readI18nSource,
  placeholdersIn,
};
