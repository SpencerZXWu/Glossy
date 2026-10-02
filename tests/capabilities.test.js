"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

// Every window of the app is a separate page, and Tauri only lets a page use a
// core command -- listening for an event included -- when a capability names
// that window. A window left out of the list fails at run time and not at build
// time, and only for the one call it was missing, so the gap is easy to miss
// and easy to reintroduce. These tests read the configuration the way Tauri
// does: which windows exist, which of them listen for events, and which of
// those a capability actually covers.

const TAURI_DIR = path.resolve(__dirname, "..", "src-tauri");
const SRC = path.resolve(__dirname, "..", "src");
const CAPABILITY_DIR = path.join(TAURI_DIR, "capabilities");

/** Permission names that let a window subscribe to a backend event. */
const LISTEN_PERMISSIONS = new Set([
  "core:event:default",
  "core:event:allow-listen",
  "core:default",
]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

const config = readJson(path.join(TAURI_DIR, "tauri.conf.json"));
const WINDOWS = config.app.windows.map((entry) => ({ label: entry.label, url: entry.url }));

const CAPABILITIES = fs
  .readdirSync(CAPABILITY_DIR)
  .filter((name) => name.endsWith(".json"))
  .map((name) => ({ name, ...readJson(path.join(CAPABILITY_DIR, name)) }));

/** True when a capability's `windows` list names this label, globs included. */
function names(patterns, label) {
  return patterns.some((pattern) => {
    if (!pattern.includes("*")) return pattern === label;
    const source = `^${pattern.split("*").map(escapeRegExp).join(".*")}$`;
    return new RegExp(source).test(label);
  });
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function covered(label) {
  return CAPABILITIES.some(
    (capability) =>
      names(capability.windows || [], label) &&
      (capability.permissions || []).some((permission) => LISTEN_PERMISSIONS.has(permission)),
  );
}

/** The page a window opens, since that is what decides which commands it needs. */
function pageOf(entry) {
  const base = path.basename(entry.url);
  return path.join(SRC, base === "" ? "index.html" : base);
}

/** The scripts a page loads, which is where its `Glossy.listen` calls live. */
function scriptsOf(entry) {
  const html = fs.readFileSync(pageOf(entry), "utf8");
  return Array.from(html.matchAll(/<script src="([^"]+)"/g), (match) =>
    fs.readFileSync(path.join(SRC, match[1]), "utf8"),
  );
}

function listensForEvents(entry) {
  return scriptsOf(entry).some((script) => script.includes("Glossy.listen("));
}

test("the configuration still describes the windows these tests read", () => {
  assert.deepEqual(
    WINDOWS.map((entry) => entry.label).sort(),
    ["main", "notice", "ocr", "popup", "subtitle", "subtitle-area", "subtitle-edit"],
  );
});

test("every page that listens for an event does so from a covered window", () => {
  const listeners = WINDOWS.filter(listensForEvents);
  assert.deepEqual(
    listeners.map((entry) => entry.label).sort(),
    ["main", "ocr", "popup", "subtitle", "subtitle-edit"],
  );

  const denied = listeners.filter((entry) => !covered(entry.label)).map((entry) => entry.label);
  assert.deepEqual(denied, [], `windows missing from a capability: ${denied.join(", ")}`);
});

test("the screenshot overlay is allowed to be told when a new screenshot starts", () => {
  // The overlay is the one window that is shown again and again without ever
  // being reloaded, so its page resets its state on that event. Denied, the
  // first region would be picked and every later one ignored.
  const ocr = WINDOWS.find((entry) => entry.label === "ocr");
  assert.ok(ocr, "no ocr window in tauri.conf.json");
  const listens = scriptsOf(ocr).some((script) => script.includes('Glossy.listen("glossy://ocr"'));
  assert.ok(listens, "the ocr page no longer resets itself on glossy://ocr");
  assert.ok(covered("ocr"), "the ocr window is not covered by any capability");
});
