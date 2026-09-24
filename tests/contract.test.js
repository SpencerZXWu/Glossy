"use strict";

// The frozen contract: `contract/contract.json` publishes the names the app
// answers to and the format of its settings file, and this file is what keeps
// the code to it. The settings half is checked next to the settings themselves,
// in `src-tauri/src/settings.rs`, where the stored shape is defined.

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const CONTRACT = JSON.parse(
  fs.readFileSync(path.join(ROOT, "contract", "contract.json"), "utf8"),
);

/** Every `invoke("name")` of a page, whichever way it reaches the bridge. */
function invokedCommands() {
  const dir = path.join(ROOT, "src", "js");
  return fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".js"))
    .flatMap((name) => {
      const source = fs.readFileSync(path.join(dir, name), "utf8");
      return Array.from(source.matchAll(/(?<!\w)invoke\(\s*"([^"]+)"/g), (match) => match[1]);
    });
}

/** The commands the Rust side registers, read from the one list that defines them. */
function registeredCommands() {
  const lib = fs.readFileSync(path.join(ROOT, "src-tauri", "src", "lib.rs"), "utf8");
  const block = lib.split("generate_handler![", 2)[1].split("])", 1)[0];
  return Array.from(block.matchAll(/([A-Za-z_]\w*(?:::[A-Za-z_]\w*)*)\s*,/g), (match) =>
    match[1].split("::").pop(),
  );
}

test("the frozen command list is sorted and holds no repeat", () => {
  const frozen = CONTRACT.ipcCommands;

  assert.ok(frozen.length > 0);
  assert.deepEqual(frozen, [...frozen].sort());
  assert.equal(new Set(frozen).size, frozen.length);
});

test("every command the app registers is frozen", () => {
  const registered = [...new Set(registeredCommands())].sort();

  assert.deepEqual(registered, CONTRACT.ipcCommands);
});

test("every command a page invokes is frozen", () => {
  const unknown = invokedCommands().filter((name) => !CONTRACT.ipcCommands.includes(name));

  assert.deepEqual(unknown, []);
});

test("the frozen format version is the one the settings module writes", () => {
  const source = fs.readFileSync(path.join(ROOT, "src-tauri", "src", "settings.rs"), "utf8");
  const declared = /pub const FORMAT_VERSION: u32 = (\d+);/.exec(source);

  assert.ok(declared, "settings.rs declares FORMAT_VERSION");
  assert.equal(Number(declared[1]), CONTRACT.formatVersion);
});
