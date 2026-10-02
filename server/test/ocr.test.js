import assert from "node:assert/strict";
import test from "node:test";

import { forgetToken, ocrLanguage, recognizeWithBaidu } from "../src/ocr.js";

const NOW = 1_700_000_000_000;

/** A fetch that answers the token endpoint and the OCR endpoint separately. */
function fakeFetch({ token = { access_token: "tok-1", expires_in: 2592000 }, ocr, onCall } = {}) {
  return async (url, init) => {
    if (onCall) onCall(url, init);
    if (String(url).includes("oauth")) {
      const payload = typeof token === "function" ? token() : token;
      return new Response(JSON.stringify(payload), { status: 200 });
    }
    const payload = typeof ocr === "function" ? ocr() : ocr;
    return new Response(JSON.stringify(payload), { status: 200 });
  };
}

test("maps language tags onto the models baidu can read", () => {
  assert.equal(ocrLanguage(""), "CHN_ENG");
  assert.equal(ocrLanguage("auto"), "CHN_ENG");
  assert.equal(ocrLanguage("en"), "ENG");
  assert.equal(ocrLanguage("en-US"), "ENG");
  assert.equal(ocrLanguage("zh-Hans"), "CHN_ENG");
  assert.equal(ocrLanguage("ja"), "JAP");
  assert.equal(ocrLanguage("ko"), "KOR");
  assert.equal(ocrLanguage("ru"), "RUS");
  assert.equal(ocrLanguage("uk"), "CHN_ENG");
});

test("reads the words off the picture and joins them line by line", async () => {
  let seen;
  const result = await recognizeWithBaidu({
    fetchImpl: fakeFetch({
      ocr: { words_result: [{ words: "第一行" }, { words: " second " }, { words: "" }] },
      onCall: (url, init) => {
        if (String(url).includes("ocr")) seen = { url, init };
      },
    }),
    apiKey: "key-join",
    secretKey: "secret",
    image: "aW1n",
    language: "en",
    now: () => NOW,
  });

  assert.deepEqual(result, { ok: true, text: "第一行\nsecond", language: "ENG" });
  const url = new URL(seen.url);
  assert.equal(url.origin + url.pathname, "https://aip.baidubce.com/rest/2.0/ocr/v1/general_basic");
  assert.equal(url.searchParams.get("access_token"), "tok-1");
  const body = new URLSearchParams(seen.init.body);
  assert.equal(body.get("image"), "aW1n");
  assert.equal(body.get("language_type"), "ENG");
});

test("keeps the token until it is nearly stale", async () => {
  let tokens = 0;
  let clock = NOW;
  const fetchImpl = fakeFetch({
    ocr: { words_result: [{ words: "hi" }] },
    onCall: (url) => {
      if (String(url).includes("oauth")) tokens += 1;
    },
  });

  const ask = () =>
    recognizeWithBaidu({ fetchImpl, apiKey: "key-cache", secretKey: "secret", image: "aW1n", now: () => clock });

  await ask();
  clock += 24 * 3600 * 1000;
  await ask();
  assert.equal(tokens, 1, "第二个请求应该复用凭证");

  clock += 30 * 24 * 3600 * 1000;
  await ask();
  assert.equal(tokens, 2, "凭证过期后应该重新申请");
});

test("retries once with a fresh token when baidu says the old one is stale", async () => {
  let attempts = 0;
  const result = await recognizeWithBaidu({
    fetchImpl: fakeFetch({
      token: () => ({ access_token: `tok-${++attempts}`, expires_in: 60 }),
      ocr: () => (attempts === 1 ? { error_code: 110 } : { words_result: [{ words: "ok" }] }),
    }),
    apiKey: "key-retry",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });

  assert.deepEqual(result, { ok: true, text: "ok", language: "CHN_ENG" });
  assert.equal(attempts, 2);
});

test("gives up on a stale token the second time", async () => {
  let attempts = 0;
  const result = await recognizeWithBaidu({
    fetchImpl: fakeFetch({
      token: () => ({ access_token: `tok-${++attempts}`, expires_in: 60 }),
      ocr: { error_code: 110 },
    }),
    apiKey: "key-double-stale",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_credentials");
  assert.equal(attempts, 2);
});

test("translates baidu's own error codes", async () => {
  const quota = await recognizeWithBaidu({
    fetchImpl: fakeFetch({ ocr: { error_code: 18 } }),
    apiKey: "key-quota",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });
  assert.equal(quota.ok, false);
  assert.equal(quota.code, "upstream_limit");
  assert.equal(quota.upstream, "18");

  const unknown = await recognizeWithBaidu({
    fetchImpl: fakeFetch({ ocr: { error_code: 999999 } }),
    apiKey: "key-unknown",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });
  assert.equal(unknown.code, "upstream_error");
  assert.match(unknown.message, /999999/);
});

test("says so when the picture holds no text", async () => {
  const result = await recognizeWithBaidu({
    fetchImpl: fakeFetch({ ocr: { words_result: [{ words: "  " }] } }),
    apiKey: "key-empty",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "no_text");
});

test("reports a rejected key pair instead of retrying", async () => {
  const result = await recognizeWithBaidu({
    fetchImpl: fakeFetch({ token: { error: "invalid_client", error_description: "unknown client id" } }),
    apiKey: "key-bad",
    secretKey: "secret",
    image: "aW1n",
    now: () => NOW,
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_unreachable");
  assert.match(result.message, /API Key/);
});

test("says it is not configured without keys", async () => {
  const result = await recognizeWithBaidu({ fetchImpl: fakeFetch({}), image: "aW1n" });
  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
});

test("forgets a token on demand", async () => {
  let tokens = 0;
  const fetchImpl = fakeFetch({
    ocr: { words_result: [{ words: "hi" }] },
    onCall: (url) => {
      if (String(url).includes("oauth")) tokens += 1;
    },
  });
  const ask = () =>
    recognizeWithBaidu({ fetchImpl, apiKey: "key-forget", secretKey: "secret", image: "aW1n", now: () => NOW });

  await ask();
  forgetToken("key-forget");
  await ask();
  assert.equal(tokens, 2);
});
