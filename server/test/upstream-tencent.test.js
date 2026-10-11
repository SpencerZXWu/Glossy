/**
 * Whether Tencent's machine translation is in the chain at all.
 *
 * It is the operator's own account and it is not one of the services an App can
 * pick, so the two things worth pinning down are that it stays out of the chain
 * unless a deployment asks for it, and that asking for it by name does nothing.
 */

import assert from "node:assert/strict";
import test from "node:test";

import { VENDORS, createUpstream, tencentMtWanted, upstreamConfigured } from "../src/upstream.js";

/** The pair the reader signs with, which is also this engine's. */
const KEYS = { TENCENT_SECRET_ID: "id-1", TENCENT_SECRET_KEY: "key-1" };

test("tencent machine translation is off unless a deployment asks for it", () => {
  // Reading pictures and translating text are separate products on that
  // account, so the keys being there says nothing about the second one. A
  // backend that answers every request with "not opened" is worse than one that
  // is not in the chain at all.
  assert.equal(tencentMtWanted(KEYS), false);
  assert.equal(tencentMtWanted({ ...KEYS, TENCENT_MT: "0" }), false);
  assert.equal(tencentMtWanted({ ...KEYS, TENCENT_MT: "off" }), false);
  assert.equal(tencentMtWanted({ ...KEYS, TENCENT_MT: "  " }), false);
  assert.equal(tencentMtWanted({ TENCENT_MT: "1" }), false, "no keys, no backend");
  assert.equal(tencentMtWanted({ ...KEYS, TENCENT_MT: "1" }), true);
  assert.equal(tencentMtWanted({ ...KEYS, TENCENT_MT: " on " }), true);
});

test("it is the last engine in the chain, and only when it is on", () => {
  const off = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key", ...KEYS });
  assert.deepEqual(off.vendors, ["baidu"]);
  assert.equal(upstreamConfigured(KEYS), false, "a reader alone is not a translator");

  const on = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key", ...KEYS, TENCENT_MT: "1" });
  assert.deepEqual(on.vendors, ["baidu", "tencent"]);
  // A deployment that opened nothing but this still has something to translate
  // with, which is what the health endpoint reports.
  assert.equal(upstreamConfigured({ ...KEYS, TENCENT_MT: "1" }), true);
});

test("an App cannot name it as the engine it wants", () => {
  assert.equal(VENDORS.includes("tencent"), false);
});
