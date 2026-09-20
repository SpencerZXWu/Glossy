import assert from "node:assert/strict";
import test from "node:test";

import { createHandler } from "../src/handler.js";
import { isLanguageTag } from "../src/upstream.js";

const NOW = Date.parse("2025-09-01T10:00:00Z");

function memoryStore() {
  const usage = new Map();
  const minutes = new Map();
  const key = (...parts) => parts.join("|");
  const read = (scope, id) => usage.get(key(scope, id)) ?? 0;

  return {
    calls: { reserve: 0, refund: 0 },
    async reserve(day, { clientId, ipHash, chars, limits, now, minute }) {
      this.calls.reserve += 1;
      const bucket = key(ipHash, minute);
      if (limits.requestsPerMinute > 0 && (minutes.get(bucket) ?? 0) >= limits.requestsPerMinute) {
        return { ok: false, code: "rate_limited", message: "too fast", retryAfter: 30 };
      }
      const checks = [
        [key("client", clientId), limits.charsPerClient, "client_quota_exceeded"],
        [key("ip", ipHash), limits.charsPerIp, "ip_quota_exceeded"],
        [key("total", "all"), limits.charsTotal, "global_quota_exceeded"],
      ];
      for (const [storeKey, limit, code] of checks) {
        const used = usage.get(storeKey) ?? 0;
        if (limit > 0 && used + chars > limit) {
          return { ok: false, code, message: "no quota", limit, remaining: Math.max(0, limit - used) };
        }
      }
      for (const [storeKey] of checks) usage.set(storeKey, (usage.get(storeKey) ?? 0) + chars);
      minutes.set(bucket, (minutes.get(bucket) ?? 0) + 1);
      const used = read("client", clientId);
      return { ok: true, used, remaining: Math.max(0, limits.charsPerClient - used) };
    },
    async refund(day, { clientId, ipHash, chars }) {
      this.calls.refund += 1;
      for (const storeKey of [key("client", clientId), key("ip", ipHash), key("total", "all")]) {
        usage.set(storeKey, Math.max(0, (usage.get(storeKey) ?? 0) - chars));
      }
    },
    async peek(day, { clientId, ipHash }) {
      return {
        client: read("client", clientId),
        ip: read("ip", ipHash),
        total: read("total", "all"),
      };
    },
  };
}

function setup({ translateImpl, env = {}, configured = true } = {}) {
  const store = memoryStore();
  const calls = [];
  const upstream = {
    isLanguageTag,
    configured,
    translate: async (input) => {
      calls.push(input);
      return translateImpl ? translateImpl(input) : { ok: true, from: "en", to: "zh", translation: "你好" };
    },
  };
  const config = {
    BAIDU_APP_ID: "app",
    BAIDU_KEY: "key",
    IP_SALT: "salt",
    DAILY_CHARS_PER_CLIENT: "100",
    DAILY_CHARS_PER_IP: "150",
    DAILY_CHARS_TOTAL: "1000",
    MAX_CHARS_PER_REQUEST: "50",
    MAX_REQUESTS_PER_MINUTE: "3",
    ...env,
  };
  const handler = createHandler({ store, upstream, config, now: () => NOW });
  const call = (path, init = {}) =>
    handler(
      new Request(`https://glossy.example${path}`, {
        headers: {
          "CF-Connecting-IP": "203.0.113.7",
          "Content-Type": "application/json",
          ...(init.headers ?? {}),
        },
        ...init,
      }),
    );
  const translate = (body) =>
    call("/v1/translate", { method: "POST", body: JSON.stringify({ clientId: "install-0001", ...body }) });
  return { store, calls, call, translate, upstream };
}

test("health reports whether the upstream has credentials", async () => {
  const { call } = setup();
  const response = await call("/v1/health");
  assert.equal(response.status, 200);
  assert.equal(response.headers.get("Cache-Control"), "no-store");
  const body = await response.json();
  assert.equal(body.ok, true);
  assert.equal(body.configured, true);
  assert.equal(body.day, "2025-09-01");

  const bare = setup({ configured: false });
  assert.equal((await (await bare.call("/v1/health")).json()).configured, false);
});

test("translates and reports usage", async () => {
  const { translate, calls } = setup();
  const response = await translate({ text: "hello world", from: "en", to: "zh-CN" });
  assert.equal(response.status, 200);
  const body = await response.json();
  assert.equal(body.ok, true);
  assert.equal(body.translation, "你好");
  assert.equal(body.chars, 11);
  assert.equal(body.usage.client, 11);
  assert.equal(body.usage.remaining, 89);
  assert.deepEqual(calls, [{ text: "hello world", from: "en", to: "zh-CN" }]);
});

test("counts characters, not bytes", async () => {
  const { translate } = setup();
  const body = await (await translate({ text: "你好世界", to: "en" })).json();
  assert.equal(body.chars, 4);
});

test("quota endpoint mirrors the counters", async () => {
  const { call, translate } = setup();
  await translate({ text: "hello", to: "zh" });
  const body = await (await call("/v1/quota?client=install-0001")).json();
  assert.equal(body.usage.client, 5);
  assert.equal(body.remaining, 95);
  assert.equal(body.limits.charsPerClient, 100);
  assert.equal(body.limits.charsTotal, 1000);
  assert.equal(body.limits.maxCharsPerRequest, 50);
});

test("stops a client that used up its daily characters", async () => {
  const { translate } = setup();
  await translate({ text: "x".repeat(40), to: "zh" });
  await translate({ text: "y".repeat(40), to: "zh" });
  const response = await translate({ text: "z".repeat(40), to: "zh" });
  assert.equal(response.status, 429);
  const body = await response.json();
  assert.equal(body.code, "client_quota_exceeded");
  assert.equal(body.remaining, 20);
});

test("stops a second install id behind the same ip", async () => {
  const { call } = setup({ env: { DAILY_CHARS_PER_IP: "20" } });
  const send = (clientId, text) =>
    call("/v1/translate", { method: "POST", body: JSON.stringify({ clientId, text, to: "zh" }) });
  await send("install-aaaa", "x".repeat(15));
  const response = await send("install-bbbb", "y".repeat(15));
  assert.equal((await response.json()).code, "ip_quota_exceeded");
});

test("stops everyone once the global ceiling is reached", async () => {
  const { call } = setup({ env: { DAILY_CHARS_TOTAL: "10", DAILY_CHARS_PER_CLIENT: "0", DAILY_CHARS_PER_IP: "0" } });
  const send = (clientId, text) =>
    call("/v1/translate", { method: "POST", body: JSON.stringify({ clientId, text, to: "zh" }) });
  await send("install-aaaa", "x".repeat(10));
  const response = await send("install-bbbb", "yyyyyyyyyy");
  assert.equal((await response.json()).code, "global_quota_exceeded");
});

test("rate limits a burst from one ip", async () => {
  const { translate } = setup();
  for (let i = 0; i < 3; i += 1) assert.equal((await translate({ text: "hi", to: "zh" })).status, 200);
  const response = await translate({ text: "hi", to: "zh" });
  assert.equal(response.status, 429);
  const body = await response.json();
  assert.equal(body.code, "rate_limited");
  assert.equal(body.retryAfter, 30);
});

test("gives characters back when the upstream call fails", async () => {
  const { translate, store, call } = setup({ translateImpl: async () => ({ ok: false, code: "upstream_limit", message: "额度用尽" }),
  });
  const response = await translate({ text: "hello world", to: "zh" });
  assert.equal(response.status, 502);
  assert.equal((await response.json()).code, "upstream_limit");
  assert.equal(store.calls.refund, 1);
  assert.equal((await (await call("/v1/quota?client=install-0001")).json()).usage.client, 0);
});

test("gives characters back when the upstream call throws", async () => {
  const { translate, store } = setup({ translateImpl: async () => {
      throw new Error("boom");
    },
  });
  const response = await translate({ text: "hello", to: "zh" });
  assert.equal(response.status, 502);
  assert.equal((await response.json()).code, "upstream_error");
  assert.equal(store.calls.refund, 1);
});

test("reports a server without credentials instead of charging", async () => {
  const { translate, store } = setup({ translateImpl: async () => ({ ok: false, code: "not_configured", message: "没有密钥" }),
  });
  const response = await translate({ text: "hello", to: "zh" });
  assert.equal(response.status, 503);
  assert.equal(store.calls.refund, 1);
});

test("rejects bad requests before touching the counters", async () => {
  const { call, translate, store, calls } = setup();

  assert.equal((await translate({})).status, 400);
  assert.equal((await translate({ text: "   " })).status, 400);
  assert.equal((await translate({ text: "hi", to: "not a language" })).status, 400);
  assert.equal((await call("/v1/translate", { method: "POST", body: "{" })).status, 400);
  assert.equal(
    (await call("/v1/translate", { method: "POST", body: JSON.stringify({ text: "hi", clientId: "short", to: "zh" }) })).status,
    400,
  );
  assert.equal((await translate({ text: "z".repeat(51), to: "zh" })).status, 413);
  assert.equal((await call("/v1/quota?client=nope")).status, 400);
  assert.equal((await call("/v1/translate")).status, 405);
  assert.equal((await call("/v1/health", { method: "POST" })).status, 405);
  assert.equal((await call("/v1/nothing")).status, 404);
  assert.equal((await call("/v1/nothing")).headers.get("Content-Type"), "application/json; charset=utf-8");

  assert.equal(store.calls.reserve, 0);
  assert.equal(calls.length, 0);
});

test("counts characters by code point", async () => {
  const { translate } = setup();
  const body = await (await translate({ text: "😀😀", to: "zh" })).json();
  assert.equal(body.chars, 2);
});
