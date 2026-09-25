"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");

const fs = require("node:fs");
const path = require("node:path");

const { JS_DIR, loadFrontend } = require("./helpers/load-scripts.js");

/** The Rust module that turns the recorded spelling back into a hotkey. */
const RUST_HOTKEY = path.resolve(
  JS_DIR,
  "..",
  "..",
  "src-tauri",
  "src",
  "platform",
  "windows",
  "hotkey.rs",
);

const env = loadFrontend({ files: ["keycombo.js"] });
const keycombo = env.Glossy.keycombo;

/** A keyboard event, with the modifier flags the recorder reads. */
function press(key, options) {
  const held = options || {};
  return {
    key,
    code: held.code || "",
    ctrlKey: !!held.ctrl,
    altKey: !!held.alt,
    shiftKey: !!held.shift,
    metaKey: !!held.win,
  };
}

test("the module exports what the settings window reads", () => {
  assert.equal(typeof keycombo.capture, "function");
  assert.equal(typeof keycombo.conflict, "function");
  assert.ok(Array.isArray(keycombo.BLOCKED));
  assert.ok(Array.isArray(keycombo.BUSY));
});

test("capture spells a combination the way the backend parses it", () => {
  assert.equal(keycombo.capture(press("c", { ctrl: true, alt: true })).spec, "Ctrl+Alt+C");
  assert.equal(keycombo.capture(press("1", { ctrl: true, shift: true })).spec, "Ctrl+Shift+1");
  assert.equal(keycombo.capture(press("F5", { win: true })).spec, "Win+F5");
  assert.equal(keycombo.capture(press("ArrowUp", { ctrl: true })).spec, "Ctrl+Up");
  assert.equal(keycombo.capture(press(" ", { ctrl: true, alt: true })).spec, "Ctrl+Alt+Space");
  assert.equal(keycombo.capture(press("Enter", { ctrl: true })).spec, "Ctrl+Enter");
});

test("capture keeps the modifier order stable however the keys are held", () => {
  assert.equal(keycombo.capture(press("k", { win: true, shift: true, alt: true, ctrl: true })).spec, "Ctrl+Alt+Shift+Win+K");
});

test("capture reports a lone modifier as unfinished rather than a key", () => {
  for (const key of ["Control", "Alt", "Shift", "Meta", "AltGraph"]) {
    const read = keycombo.capture(press(key, { ctrl: true }));
    assert.equal(read.problem, "modifiers", key);
    assert.equal(read.spec, "");
  }
});

test("capture refuses a key held without a modifier", () => {
  const read = keycombo.capture(press("c"));
  assert.equal(read.problem, "nomod");
  assert.equal(read.spec, "C");
});

test("capture names the key it could not use", () => {
  const read = keycombo.capture(press("Dead", { ctrl: true }));
  assert.equal(read.problem, "unknown");
  assert.equal(read.key, "Dead");
});

test("capture falls back to the physical key when the layout hides the letter", () => {
  assert.equal(keycombo.capture(press("é", { code: "KeyE", ctrl: true })).spec, "Ctrl+E");
  assert.equal(keycombo.capture(press("&", { code: "Digit1", ctrl: true })).spec, "Ctrl+1");
});

test("capture knows the same named keys the backend does", () => {
  const rust = fs.readFileSync(RUST_HOTKEY, "utf8");
  const block = /fn key_code[\s\S]*?\n\}/.exec(rust);
  assert.ok(block, "key_code is not in hotkey.rs");
  const labels = new Set(
    [...block[0].matchAll(/=> Some\(\(VK_[A-Z0-9_]+, "([A-Za-z0-9]+)"\)\)/g)].map(
      (match) => match[1],
    ),
  );
  assert.ok(labels.size > 10, "no named keys were read out of key_code");
  for (const label of labels) {
    const read = keycombo.capture(press(label, { ctrl: true }));
    assert.equal(read.problem, "", label);
    assert.equal(read.spec, "Ctrl+" + label, label);
  }
});

test("conflict refuses the combinations Windows keeps for itself", () => {
  assert.equal(keycombo.conflict("Ctrl+Alt+Delete"), "blocked");
  assert.equal(keycombo.conflict("Win+L"), "blocked");
  assert.equal(keycombo.conflict("Alt+Tab"), "blocked");
  assert.equal(keycombo.conflict("ctrl + shift + esc"), "blocked");
});

test("conflict warns about the ones something else usually owns", () => {
  assert.equal(keycombo.conflict("Win+D"), "busy");
  assert.equal(keycombo.conflict("Alt+F4"), "busy");
});

test("conflict leaves a free combination alone", () => {
  assert.equal(keycombo.conflict("Ctrl+Alt+C"), "");
  assert.equal(keycombo.conflict(""), "");
});

test("capture and conflict agree on what was pressed", () => {
  const read = keycombo.capture(press("Delete", { ctrl: true, alt: true }));
  assert.equal(keycombo.conflict(read.spec), "blocked");
});
