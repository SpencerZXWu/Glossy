import assert from "node:assert/strict";
import test from "node:test";

import { createHandler } from "../src/handler.js";
import { isLanguageTag } from "../src/upstream.js";

const NOW = Date.parse("2025-09-01T10:00:00Z");

/** Stands in for a screenshot; only its length matters here. */
const PNG = Buffer.from("not really a png, but base64-clean").toString("base64");

function memoryStore() {
  const usage = new Map();
  const minutes = new Map();
  const readings = new Map();
  const key = (...parts) => parts.join("|");
  const read = (scope, id) => usage.get(key(scope, id)) ?? 0;

  return {
    readings,
    calls: { reserve: 0, refund: 0, reserveOcr: 0, refundOcr: 0 },
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
    async reserveOcr(month, { clientId, limit }) {
      this.calls.reserveOcr += 1;
      const readings = this.readings;
      const used = readings.get(clientId) ?? 0;
      if (limit > 0 && used >= limit) {
        return {
          ok: false,
          code: "ocr_month_quota_exceeded",
          message: "no readings left",
          limit,
          remaining: 0,
        };
      }
      readings.set(clientId, used + 1);
      return { ok: true, used: used + 1, remaining: limit > 0 ? Math.max(0, limit - used - 1) : null };
    },
    async refundOcr(month, { clientId }) {
      this.calls.refundOcr += 1;
      const readings = this.readings;
      readings.set(clientId, Math.max(0, (readings.get(clientId) ?? 0) - 1));
    },
    async peekOcr(month, { clientId }) {
      return { requests: this.readings.get(clientId) ?? 0 };
    },
  };
}

function setup({ translateImpl, ratesImpl, ocrImpl, env = {}, configured = true, ocrConfigured = true } = {}) {
  const store = memoryStore();
  const calls = [];
  const rateCalls = [];
  const ocrCalls = [];
  const upstream = {
    isLanguageTag,
    configured,
    ocrConfigured,
    rates: async (input) => {
      rateCalls.push(input);
      return ratesImpl
        ? ratesImpl(input)
        : { ok: true, base: "USD", source: "exchangerate-api.com", date: "2026-02-05", rates: { USD: 1, CNY: 7.12 } };
    },
    translate: async (input) => {
      calls.push(input);
      return translateImpl ? translateImpl(input) : { ok: true, from: "en", to: "zh", translation: "你好" };
    },
    ocr: async (input) => {
      ocrCalls.push(input);
      return ocrImpl ? ocrImpl(input) : { ok: true, text: "recognized", language: "CHN_ENG" };
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
  const handler = createHandler({ store, ocrStore: store, upstream, config, now: () => NOW });
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
  const ocr = (body) => call("/v1/ocr", { method: "POST", body: JSON.stringify({ clientId: "install-0001", image: PNG, ...body }) });
  return { store, calls, rateCalls, ocrCalls, call, translate, ocr, upstream };
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
test("health reports which version switches are on", async () => {
  // The operator's way to tell a live switch from one that was set on a
  // deployment that never took it: one request, no old build needed.
  const off = await (await setup().call("/v1/health")).json();
  assert.deepEqual(off.version, { blockBelow: null, minVersion: null, announcement: false });

  const on = await (
    await setup({
      env: { BLOCK_BELOW: " 2.1.1 ", MIN_VERSION: "2.2.0", ANNOUNCEMENT: "明天维护" },
    }).call("/v1/health")
  ).json();
  assert.equal(on.version.blockBelow, "2.1.1");
  assert.equal(on.version.minVersion, "2.2.0");
  assert.equal(on.version.announcement, true);
  // Whether there is one is all this needs to say; the text itself is for the
  // Apps, not for anyone who asks.
  assert.equal(JSON.stringify(on).includes("明天维护"), false);
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
  assert.deepEqual(calls, [{ text: "hello world", from: "en", to: "zh-CN", vendor: "" }]);
});

test("counts characters, not bytes", async () => {
  const { translate } = setup();
  const body = await (await translate({ text: "你好世界", to: "en" })).json();
  assert.equal(body.chars, 4);
});

test("says which vendors the answer was not from", async () => {
  const { translate } = setup({
    translateImpl: async () => ({
      ok: true,
      from: "en2zh-CHS",
      to: "zh-CHS",
      translation: "你好",
      vendor: "youdao",
      attempts: [{ vendor: "baidu", code: "upstream_limit" }],
    }),
  });
  const body = await (await translate({ text: "hello", to: "zh-CN", vendor: "baidu" })).json();

  assert.equal(body.vendor, "youdao");
  assert.deepEqual(body.attempts, [{ vendor: "baidu", code: "upstream_limit" }]);

  // A deployment whose translator says nothing about it answers with an empty
  // list rather than leaving the field out: the App reads both the same way.
  const plain = setup();
  const other = await (await plain.translate({ text: "hello", to: "zh-CN" })).json();
  assert.deepEqual(other.attempts, []);
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

test("serves exchange rates without spending characters", async () => {
  const { call, translate, rateCalls } = setup();
  await translate({ text: "hello", to: "zh" });

  const response = await call("/v1/rates?client=install-0001&base=usd");
  assert.equal(response.status, 200);
  assert.equal(response.headers.get("Cache-Control"), "no-store");
  const body = await response.json();
  assert.equal(body.ok, true);
  assert.equal(body.base, "USD");
  assert.equal(body.source, "exchangerate-api.com");
  assert.equal(body.date, "2026-02-05");
  assert.equal(body.rates.CNY, 7.12);
  assert.deepEqual(rateCalls, [{ base: "usd" }]);

  // A conversion is an annotation on the card, so the characters booked by the
  // translation are the only ones the counters hold.
  assert.equal((await (await call("/v1/quota?client=install-0001")).json()).usage.client, 5);
});

test("rejects a rate request before touching the counters", async () => {
  const { call, store, rateCalls } = setup();
  assert.equal((await call("/v1/rates?client=nope&base=USD")).status, 400);
  assert.equal((await call("/v1/rates?client=install-0001&base=US")).status, 400);
  assert.equal((await call("/v1/rates?client=install-0001")).status, 400);
  assert.equal((await call("/v1/rates?client=install-0001&base=USD", { method: "POST" })).status, 405);
  assert.equal(store.calls.reserve, 0);
  assert.equal(rateCalls.length, 0);
});

test("reports a rate vendor that cannot answer", async () => {
  const { call } = setup({
    ratesImpl: async () => ({ ok: false, code: "upstream_unreachable", message: "取不到汇率" }),
  });
  const response = await call("/v1/rates?client=install-0001&base=USD");
  assert.equal(response.status, 502);
  assert.equal((await response.json()).code, "upstream_unreachable");
});

test("rate limits rate requests as well", async () => {
  const { call } = setup();
  for (let i = 0; i < 3; i += 1) {
    assert.equal((await call("/v1/rates?client=install-0001&base=USD")).status, 200);
  }
  const response = await call("/v1/rates?client=install-0001&base=USD");
  assert.equal(response.status, 429);
  assert.equal((await response.json()).code, "rate_limited");
});

test("health says whether the server can read pictures", async () => {
  assert.equal((await (await setup().call("/v1/health")).json()).ocr, true);
  assert.equal((await (await setup({ ocrConfigured: false }).call("/v1/health")).json()).ocr, false);
});

test("reads a screenshot and books the fixed fee", async () => {
  const { ocr, ocrCalls } = setup({ env: { OCR_CHARS_PER_REQUEST: "5" } });
  const response = await ocr({ language: "en" });
  assert.equal(response.status, 200);

  const body = await response.json();
  assert.equal(body.text, "recognized");
  assert.equal(body.language, "CHN_ENG");
  assert.equal(body.chars, 5);
  assert.equal(body.usage.client, 5);
  assert.deepEqual(ocrCalls, [{ image: PNG, language: "en" }]);
});

test("strips a data url before handing the picture on", async () => {
  const { ocr, ocrCalls } = setup({ env: { OCR_CHARS_PER_REQUEST: "5" } });
  const response = await ocr({ image: `data:image/png;base64,${PNG}\n` });
  assert.equal(response.status, 200);
  assert.equal(ocrCalls[0].image, PNG);
});

test("refuses a request without a usable picture", async () => {
  const { ocr, store } = setup();
  assert.equal((await ocr({ image: "" })).status, 400);
  assert.equal((await ocr({ image: "data:image/png;base64," })).status, 400);
  assert.equal((await ocr({ image: "not base64 !!" })).status, 400);
  assert.equal((await ocr({ clientId: "short" })).status, 400);
  assert.equal(store.calls.reserve, 0);
});

test("refuses a picture that is larger than the limit", async () => {
  const { ocr, store } = setup({ env: { MAX_IMAGE_BYTES: "8" } });
  const response = await ocr({});
  assert.equal(response.status, 413);
  const body = await response.json();
  assert.equal(body.code, "too_long");
  assert.equal(body.limit, 8);
  assert.equal(store.calls.reserve, 0);
});

test("only accepts POST", async () => {
  const { call } = setup();
  assert.equal((await call("/v1/ocr")).status, 405);
});

test("says so when the server has no picture reader", async () => {
  const { ocr, ocrCalls, store } = setup({ ocrConfigured: false });
  const response = await ocr({});
  assert.equal(response.status, 503);
  assert.equal((await response.json()).code, "not_configured");
  assert.equal(store.calls.reserve, 0, "没配密钥就不该记额度");
  assert.equal(ocrCalls.length, 0);
});

test("hands the fee back when the picture holds no text", async () => {
  const { ocr, store } = setup({
    env: { OCR_CHARS_PER_REQUEST: "5" },
    ocrImpl: async () => ({ ok: false, code: "no_text", message: "没有文字" }),
  });
  const response = await ocr({});
  assert.equal(response.status, 422);
  assert.equal((await response.json()).code, "no_text");
  assert.equal(store.calls.refund, 1);
});

test("hands the fee back when the reader is out of quota", async () => {
  const { ocr, store } = setup({
    env: { OCR_CHARS_PER_REQUEST: "5" },
    ocrImpl: async () => ({ ok: false, code: "upstream_limit", message: "额度用完了" }),
  });
  const response = await ocr({});
  assert.equal(response.status, 429);
  assert.equal((await response.json()).code, "upstream_limit");
  assert.equal(store.calls.refund, 1);
});

test("says which vendor code the refusal came from", async () => {
  const { ocr } = setup({
    ocrImpl: async () => ({
      ok: false,
      code: "upstream_limit",
      message: "额度用完了",
      upstream: "ResourceUnavailable.NotExist",
    }),
  });
  const body = await (await ocr({})).json();
  assert.equal(body.upstream, "ResourceUnavailable.NotExist");
});

test("spends one flat fee per screenshot out of the daily allowance", async () => {
  const { ocr } = setup({ env: { OCR_CHARS_PER_REQUEST: "60" } });
  assert.equal((await ocr({})).status, 200);

  const response = await ocr({});
  assert.equal(response.status, 429);
  const body = await response.json();
  assert.equal(body.code, "client_quota_exceeded");
  assert.equal(body.limit, 100);
});

test("counts the readings of the month and stops at the limit", async () => {
  const { ocr, store, ocrCalls } = setup({ env: { OCR_PER_CLIENT_MONTH: "2", OCR_CHARS_PER_REQUEST: "1" } });

  assert.equal((await ocr({})).status, 200);
  assert.equal((await ocr({})).status, 200);

  const response = await ocr({});
  assert.equal(response.status, 429);
  const body = await response.json();
  assert.equal(body.code, "ocr_month_quota_exceeded");
  assert.equal(body.limit, 2);
  assert.equal(body.remaining, 0);
  // 被拦下的这一次既没有发给上游，也没有记账。
  assert.equal(ocrCalls.length, 2);
  assert.equal(store.calls.reserveOcr, 3);
  assert.equal(store.calls.reserve, 2);
});

test("a month limit of zero leaves the readings uncounted", async () => {
  const { ocr, store } = setup({ env: { OCR_PER_CLIENT_MONTH: "0", OCR_CHARS_PER_REQUEST: "1" } });
  for (let i = 0; i < 3; i += 1) assert.equal((await ocr({})).status, 200);

  assert.equal(store.calls.reserveOcr, 3);
  assert.equal(store.readings.get("install-0001"), 3);
});

test("hands the reading back when the day has no characters left", async () => {
  // 日额度先耗尽时，月度次数不该被记上。
  const { ocr, store } = setup({ env: { OCR_PER_CLIENT_MONTH: "2", OCR_CHARS_PER_REQUEST: "60" } });
  assert.equal((await ocr({})).status, 200);

  const response = await ocr({});
  assert.equal(response.status, 429);
  assert.equal((await response.json()).code, "client_quota_exceeded");
  assert.equal(store.calls.refundOcr, 1);
  assert.equal(store.readings.get("install-0001"), 1);
});

test("hands the reading back when the picture was never read", async () => {
  const { ocr, store, ocrCalls } = setup({
    env: { OCR_PER_CLIENT_MONTH: "1", OCR_CHARS_PER_REQUEST: "5" },
    ocrImpl: async () => ({ ok: false, code: "no_text", message: "没有文字" }),
  });
  assert.equal((await ocr({})).status, 422);
  assert.equal(store.calls.refundOcr, 1);
  assert.equal(store.readings.get("install-0001"), 0);

  // 读不到文字的这一次不算数，用户还能再截一次，而不是等一个月。
  assert.equal((await ocr({})).status, 422);
  assert.equal(ocrCalls.length, 2);
});

test("reports the month's readings and the day's characters apart", async () => {
  const { call, ocr } = setup({ env: { OCR_PER_CLIENT_MONTH: "5", OCR_CHARS_PER_REQUEST: "5" } });
  await ocr({});

  const body = await (await call("/v1/quota?client=install-0001")).json();
  assert.equal(body.month, "2025-09");
  assert.equal(body.limits.ocrPerClientMonth, 5);
  assert.equal(body.usage.ocrMonth, 1);
  assert.equal(body.usage.client, 5);
  assert.equal(body.remainingOcrMonth, 4);
});

test("a quota report without a month limit has nothing left to report", async () => {
  const { call } = setup({ env: { OCR_PER_CLIENT_MONTH: "0" } });
  const body = await (await call("/v1/quota?client=install-0001")).json();
  assert.equal(body.remainingOcrMonth, null);
  assert.equal(body.usage.ocrMonth, 0);
});

test("a deployment that has stopped an old build refuses it and books nothing", async () => {
  const { translate, ocr, calls, ocrCalls, store } = setup({ env: { BLOCK_BELOW: "2.1.1" } });

  // A build that sends no version is one from before 2.1.1, which is what a
  // floor is for.
  const old = await translate({ text: "hello world", to: "zh" });
  assert.equal(old.status, 403);
  const body = await old.json();
  assert.equal(body.ok, false);
  assert.equal(body.code, "version_too_old");
  // The App reading this is the one that has no sentence for the code, so the
  // text has to come from the relay.
  assert.match(body.message, /releases\/latest/);
  assert.equal(store.calls.reserve, 0, "被停用的版本不该占用额度");
  assert.deepEqual(calls, [], "被停用的版本不该调用上游");

  // A screenshot costs the operator a call to the vendor as well.
  const shot = await ocr({});
  assert.equal(shot.status, 403);
  assert.equal(store.calls.reserveOcr, 0);
  assert.deepEqual(ocrCalls, []);

  // At the floor, or above it, nothing changes.
  const fresh = await translate({ text: "hello world", to: "zh", appVersion: "2.1.1" });
  assert.equal(fresh.status, 200);
  assert.equal((await fresh.json()).translation, "你好");
});

test("a deployment that has set no floor serves every build", async () => {
  const { translate } = setup();
  const response = await translate({ text: "hello", to: "zh", appVersion: "0.9.0" });
  assert.equal(response.status, 200);
});
