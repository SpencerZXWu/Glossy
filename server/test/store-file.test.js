import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { createFileStore } from "../src/store-file.js";

const DAY = "2025-09-01";
const NEXT_DAY = "2025-09-02";
const LIMITS = {
  charsPerClient: 100,
  charsPerIp: 150,
  charsTotal: 1000,
  maxCharsPerRequest: 50,
  requestsPerMinute: 3,
};
const NOW = Date.parse("2025-09-01T10:00:00Z");

function tempDir(t) {
  const dir = mkdtempSync(join(tmpdir(), "glossy-cloud-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

function book(store, over = {}) {
  return store.reserve(DAY, {
    clientId: "client-a",
    ipHash: "ip-a",
    chars: 10,
    limits: LIMITS,
    now: NOW,
    minute: Math.floor(NOW / 60000),
    ...over,
  });
}

test("remembers the counters after a restart", (t) => {
  const file = join(tempDir(t), "quota.json");

  const first = createFileStore({ file });
  assert.equal(book(first, { chars: 30 }).ok, true);

  const second = createFileStore({ file });
  assert.deepEqual(second.peek(DAY, { clientId: "client-a", ipHash: "ip-a" }), {
    client: 30,
    ip: 30,
    total: 30,
  });
  assert.equal(book(second, { chars: 71 }).ok, false);
  assert.equal(book(second, { chars: 70 }).ok, true);
});

test("writes a file that reads back as plain JSON", (t) => {
  const file = join(tempDir(t), "quota.json");
  const store = createFileStore({ file });
  book(store);

  const raw = JSON.parse(readFileSync(file, "utf8"));
  assert.equal(raw.day, DAY);
  assert.equal(raw.usage["client|client-a"].chars, 10);
  assert.equal(raw.usage["total|all"].chars, 10);
});

test("a new UTC day starts from zero", (t) => {
  const file = join(tempDir(t), "quota.json");
  const store = createFileStore({ file });
  book(store, { chars: 100 });

  assert.equal(book(store, { chars: 10 }).ok, false);
  assert.equal(store.peek(NEXT_DAY, { clientId: "client-a", ipHash: "ip-a" }).client, 0);
  assert.equal(book(store, { chars: 10 }).ok, true);
});

test("starts empty when the counter file is unreadable or corrupt", (t) => {
  const dir = tempDir(t);
  const file = join(dir, "quota.json");
  writeFileSync(file, "{ this is not json", "utf8");

  const store = createFileStore({ file });
  assert.equal(store.peek(DAY, { clientId: "client-a", ipHash: "ip-a" }).client, 0);
  assert.equal(book(store).ok, true);
  assert.equal(JSON.parse(readFileSync(file, "utf8")).day, DAY);
});

test("keeps serving when the counter file cannot be written", (t) => {
  const file = join(tempDir(t), "missing", "quota.json");
  const errors = [];
  const original = console.error;
  console.error = (...args) => errors.push(args.join(" "));
  t.after(() => {
    console.error = original;
  });

  const store = createFileStore({ file });
  assert.equal(book(store).ok, true);
  assert.equal(store.peek(DAY, { clientId: "client-a", ipHash: "ip-a" }).client, 10);
  assert.equal(errors.length, 1, "只提醒一次");
});

test("works without a file at all", () => {
  const store = createFileStore({ file: "" });
  assert.equal(book(store).ok, true);
  assert.equal(store.peek(DAY, { clientId: "client-a", ipHash: "ip-a" }).client, 10);
});
