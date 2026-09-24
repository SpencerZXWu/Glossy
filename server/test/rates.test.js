import assert from "node:assert/strict";
import test from "node:test";

import { createUpstream } from "../src/upstream.js";
import { fetchRates, isBaseCode, RATE_SOURCES } from "../src/rates.js";

const PRIMARY_BODY = JSON.stringify({
  result: "success",
  base_code: "USD",
  time_last_update_utc: "Thu, 05 Feb 2026 00:02:31 +0000",
  rates: { USD: 1, CNY: 7.1234, JPY: 150.2 },
});

const FALLBACK_BODY = JSON.stringify({ amount: 1.0, base: "USD", date: "2026-02-05", rates: { CNY: 7.11 } });

/** Answers `bodies` in the order they are asked, and remembers the urls. */
function stub(bodies) {
  const urls = [];
  const fetchImpl = async (url) => {
    urls.push(url);
    const body = bodies.shift();
    if (body instanceof Error) throw body;
    return body === undefined
      ? { ok: false, status: 500, json: async () => ({}) }
      : { ok: true, status: 200, json: async () => JSON.parse(body) };
  };
  return { fetchImpl, urls };
}

test("accepts three letter codes only", () => {
  assert.equal(isBaseCode("usd"), true);
  assert.equal(isBaseCode(" USD "), true);
  assert.equal(isBaseCode("US"), false);
  assert.equal(isBaseCode("USDD"), false);
  assert.equal(isBaseCode(""), false);
  assert.equal(isBaseCode(undefined), false);
});

test("reads the primary vendor's answer and formats its date", async () => {
  const { fetchImpl, urls } = stub([PRIMARY_BODY]);
  const result = await fetchRates({ fetchImpl, base: "usd" });
  assert.equal(result.ok, true);
  assert.equal(result.base, "USD");
  assert.equal(result.source, RATE_SOURCES.primary);
  assert.equal(result.date, "2026-02-05");
  assert.equal(result.rates.CNY, 7.1234);
  assert.deepEqual(urls, ["https://open.er-api.com/v6/latest/USD"]);
});

test("falls back to the ECB vendor and fills in the base it left out", async () => {
  const { fetchImpl, urls } = stub([new Error("blocked"), FALLBACK_BODY]);
  const result = await fetchRates({ fetchImpl, base: "USD" });
  assert.equal(result.ok, true);
  assert.equal(result.source, RATE_SOURCES.fallback);
  assert.equal(result.date, "2026-02-05");
  assert.equal(result.rates.CNY, 7.11);
  assert.equal(result.rates.USD, 1);
  assert.deepEqual(urls, [
    "https://open.er-api.com/v6/latest/USD",
    "https://api.frankfurter.app/latest?from=USD",
  ]);
});

test("skips a vendor that answered something unusable", async () => {
  const wrong = JSON.stringify({ result: "error", rates: {} });
  const { fetchImpl } = stub([wrong, FALLBACK_BODY]);
  const result = await fetchRates({ fetchImpl, base: "USD" });
  assert.equal(result.source, RATE_SOURCES.fallback);
});

test("drops codes that carry no usable number", async () => {
  const body = JSON.stringify({
    result: "success",
    rates: { USD: 1, BAD: "7.1", ZERO: 0, NEGATIVE: -1, NULL: null },
  });
  const { fetchImpl } = stub([body]);
  const result = await fetchRates({ fetchImpl, base: "USD" });
  assert.deepEqual(Object.keys(result.rates), ["USD"]);
});

test("gives up when neither vendor answers", async () => {
  const { fetchImpl } = stub([new Error("blocked"), new Error("blocked")]);
  const result = await fetchRates({ fetchImpl, base: "USD" });
  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_unreachable");
  assert.match(result.message, /取不到汇率/);
});

test("rejects a base that is not a currency code without asking anybody", async () => {
  const { fetchImpl, urls } = stub([]);
  const result = await fetchRates({ fetchImpl, base: "dollars" });
  assert.equal(result.ok, false);
  assert.equal(result.code, "invalid_request");
  assert.deepEqual(urls, []);
});

test("the deployment is offered rates even without translation keys", async () => {
  const upstream = createUpstream({});
  assert.equal(upstream.configured, false);
  assert.equal(typeof upstream.rates, "function");
});
