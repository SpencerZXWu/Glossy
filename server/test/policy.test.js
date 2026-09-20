import assert from "node:assert/strict";
import test from "node:test";

import { createState, peek, pruneMinutes, refund, reserve, usageKey } from "../src/policy.js";

const LIMITS = {
  charsPerClient: 100,
  charsPerIp: 150,
  charsTotal: 1000,
  maxCharsPerRequest: 50,
  requestsPerMinute: 3,
};

const NOW = Date.parse("2025-09-01T10:00:00Z");
const MINUTE = Math.floor(NOW / 60000);

function book(state, over = {}) {
  return reserve(state, {
    clientId: "client-a",
    ipHash: "ip-a",
    chars: 10,
    limits: LIMITS,
    now: NOW,
    minute: MINUTE,
    ...over,
  });
}

test("books the characters and reports what is left", () => {
  const state = createState();
  const result = book(state);

  assert.equal(result.ok, true);
  assert.equal(result.used, 10);
  assert.equal(result.remaining, 90);
  assert.deepEqual(peek(state, { clientId: "client-a", ipHash: "ip-a" }), { client: 10, ip: 10, total: 10 });
});

test("rejects a request that would pass the client quota and writes nothing", () => {
  const state = createState();
  assert.equal(book(state, { chars: 95 }).ok, true);

  const result = book(state, { chars: 10 });
  assert.equal(result.ok, false);
  assert.equal(result.code, "client_quota_exceeded");
  assert.equal(result.remaining, 5);
  assert.equal(peek(state, { clientId: "client-a", ipHash: "ip-a" }).client, 95);
});

test("stops everyone once the shared ceiling is reached", () => {
  const state = createState();
  const wide = { ...LIMITS, charsPerClient: 0, charsPerIp: 0 };
  assert.equal(book(state, { chars: 1000, limits: wide }).ok, true);

  const result = book(state, { chars: 1, limits: wide });
  assert.equal(result.ok, false);
  assert.equal(result.code, "global_quota_exceeded");
});

test("blocks a second install behind the same address", () => {
  const state = createState();
  const ipLimits = { ...LIMITS, charsPerClient: 0 };
  assert.equal(book(state, { chars: 150, limits: ipLimits }).ok, true);

  const result = book(state, { clientId: "client-b", chars: 1, limits: ipLimits });
  assert.equal(result.ok, false);
  assert.equal(result.code, "ip_quota_exceeded");
});

test("rate limits a burst and says when to come back", () => {
  const state = createState();
  for (let i = 0; i < 3; i += 1) assert.equal(book(state).ok, true);

  const result = book(state);
  assert.equal(result.ok, false);
  assert.equal(result.code, "rate_limited");
  assert.equal(result.retryAfter, 60);
});

test("counts the rate limit per address, not per request", () => {
  const state = createState();
  for (let i = 0; i < 3; i += 1) assert.equal(book(state).ok, true);

  assert.equal(book(state, { ipHash: "ip-b" }).ok, true);
});

test("a limit of zero switches that check off", () => {
  const state = createState();
  const off = { ...LIMITS, charsPerClient: 0, charsPerIp: 0, charsTotal: 0, requestsPerMinute: 0 };
  assert.equal(book(state, { chars: 5000, limits: off }).ok, true);
  assert.equal(book(state, { chars: 5000, limits: off }).ok, true);
});

test("reports the total for today, not the characters twice", () => {
  const state = createState();
  assert.equal(book(state, { chars: 30 }).used, 30);
  assert.equal(book(state, { chars: 30 }).used, 60);
  assert.equal(book(state, { chars: 30 }).remaining, 10);
});

test("gives characters back when the upstream call fails", () => {
  const state = createState();
  book(state, { chars: 40 });
  book(state, { chars: 60 });
  assert.equal(book(state, { chars: 1 }).ok, false);

  refund(state, { clientId: "client-a", ipHash: "ip-a", chars: 60 });
  assert.equal(peek(state, { clientId: "client-a", ipHash: "ip-a" }).client, 40);
  assert.equal(book(state, { chars: 10 }).ok, true);
});

test("never refunds below zero", () => {
  const state = createState();
  book(state, { chars: 10 });
  refund(state, { clientId: "client-a", ipHash: "ip-a", chars: 999 });

  assert.equal(peek(state, { clientId: "client-a", ipHash: "ip-a" }).client, 0);
  assert.equal(peek(state, { clientId: "client-a", ipHash: "ip-a" }).ip, 0);
  assert.equal(peek(state, { clientId: "client-a", ipHash: "ip-a" }).total, 0);
});

test("collapses invented ids into one bucket once the state is full", () => {
  const state = createState();
  const limits = { ...LIMITS, charsPerClient: 0, charsPerIp: 0, requestsPerMinute: 0, maxTrackedBuckets: 3 };

  for (let i = 0; i < 10; i += 1) {
    assert.equal(book(state, { clientId: `spoof-${i}`, chars: 1, limits }).ok, true);
  }

  // 前两个 id 各占一格，剩下的都挤进同一个 "*" 桶。
  assert.equal(state.usage.size, 4);
  assert.equal(state.usage.get(usageKey("client", "*")).chars, 9);
  assert.equal(state.usage.get(usageKey("total", "all")).chars, 10);
});

test("forgets minute buckets that are already over", () => {
  const state = createState();
  book(state);

  pruneMinutes(state, MINUTE + 1);
  assert.equal(state.minutes.size, 0);

  book(state, { minute: MINUTE + 1 });
  assert.equal(state.minutes.size, 1);
});
