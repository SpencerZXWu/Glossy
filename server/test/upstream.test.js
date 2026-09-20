import assert from "node:assert/strict";
import test from "node:test";

import { md5 } from "../src/md5.js";
import { baiduCode, isLanguageTag, translateUpstream } from "../src/upstream.js";

test("maps app language tags onto baidu codes", () => {
  assert.equal(baiduCode("auto"), "auto");
  assert.equal(baiduCode(""), "auto");
  assert.equal(baiduCode("en"), "en");
  assert.equal(baiduCode("EN"), "en");
  assert.equal(baiduCode("zh"), "zh");
  assert.equal(baiduCode("zh-CN"), "zh");
  assert.equal(baiduCode("zh-Hant"), "cht");
  assert.equal(baiduCode("zh-TW"), "cht");
  assert.equal(baiduCode("ja"), "jp");
  assert.equal(baiduCode("ko"), "kor");
  assert.equal(baiduCode("fr"), "fra");
  assert.equal(baiduCode("es"), "spa");
  assert.equal(baiduCode("uk"), "ukr");
  assert.equal(baiduCode("vi"), "vie");
  assert.equal(baiduCode("no"), "nor");
  assert.equal(baiduCode("pt-BR"), "pt");
});

test("rejects junk that is not a language tag", () => {
  assert.ok(isLanguageTag("auto"));
  assert.ok(isLanguageTag("en"));
  assert.ok(isLanguageTag("zh-Hant"));
  assert.ok(!isLanguageTag("english please"));
  assert.equal(isLanguageTag("x".repeat(20)), false);
  assert.ok(!isLanguageTag("<script>"));
});

function jsonFetch(payload, { status = 200, capture } = {}) {
  return async (url, init) => {
    if (capture) capture(url, init);
    return new Response(typeof payload === "string" ? payload : JSON.stringify(payload), {
      status,
      headers: { "Content-Type": "application/json" },
    });
  };
}

test("signs the request the way baidu expects", async () => {
  let seen;
  await translateUpstream({
    fetchImpl: jsonFetch({ from: "en", to: "zh", trans_result: [{ src: "hello", dst: "你好" }] }, { capture: (url, init) => { seen = { url, init }; } }),
    appId: "2024000000",
    key: "secret-key",
    text: "hello",
    from: "en",
    to: "zh-CN",
  });

  const body = new URLSearchParams(seen.init.body);
  assert.equal(seen.url, "https://fanyi-api.baidu.com/api/trans/vip/translate");
  assert.equal(seen.init.method, "POST");
  assert.equal(body.get("q"), "hello");
  assert.equal(body.get("from"), "en");
  assert.equal(body.get("to"), "zh");
  assert.equal(body.get("appid"), "2024000000");
  assert.equal(body.get("sign"), md5(`2024000000${"hello"}${body.get("salt")}secret-key`));
  assert.match(body.get("salt"), /^[0-9]+$/);
});

test("joins multi line results and keeps the detected language", async () => {
  const result = await translateUpstream({
    fetchImpl: jsonFetch({ from: "en", to: "fra", trans_result: [{ dst: "Bonjour" }, { dst: "Monde" }] }),
    appId: "id",
    key: "key",
    text: "hello\nworld",
    from: "auto",
    to: "fr",
  });

  assert.deepEqual(result, { ok: true, from: "en", to: "fra", translation: "Bonjour\nMonde" });
});

test("maps baidu error codes onto app codes", async () => {
  const cases = [
    ["52003", "upstream_credentials"],
    ["54001", "upstream_credentials"],
    ["90107", "upstream_credentials"],
    ["54003", "upstream_limit"],
    ["54004", "upstream_limit"],
    ["52001", "upstream_timeout"],
    ["58001", "unsupported_language"],
    ["99999", "upstream_error"],
  ];
  for (const [upstreamCode, expected] of cases) {
    const result = await translateUpstream({
      fetchImpl: jsonFetch({ error_code: upstreamCode, error_msg: "boom" }),
      appId: "id",
      key: "key",
      text: "hi",
      from: "en",
      to: "zh",
      sleep: async () => {},
    });
    assert.equal(result.ok, false, upstreamCode);
    assert.equal(result.code, expected, upstreamCode);
    assert.ok(result.message.length > 0);
  }
});

test("retries the throttling answer instead of failing the user", async () => {
  const waits = [];
  let calls = 0;
  const fetchImpl = async () => {
    calls += 1;
    const payload =
      calls === 1
        ? { error_code: "54003", error_msg: "qps" }
        : { from: "en", to: "zh", trans_result: [{ dst: "你好" }] };
    return new Response(JSON.stringify(payload), { status: 200 });
  };

  const result = await translateUpstream({
    fetchImpl,
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
    sleep: async (ms) => waits.push(ms),
  });

  assert.equal(result.ok, true);
  assert.equal(result.translation, "你好");
  assert.equal(calls, 2);
  assert.deepEqual(waits, [600]);
});

test("gives up after the retries instead of hammering baidu", async () => {
  let calls = 0;
  const result = await translateUpstream({
    fetchImpl: jsonFetch({ error_code: "54003", error_msg: "qps" }, { capture: () => { calls += 1; } }),
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
    sleep: async () => {},
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_limit");
  assert.equal(calls, 3);
});

test("does not retry an answer that will not change", async () => {
  let calls = 0;
  const result = await translateUpstream({
    fetchImpl: jsonFetch({ error_code: "54001", error_msg: "sign" }, { capture: () => { calls += 1; } }),
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
    sleep: async () => {},
  });

  assert.equal(result.code, "upstream_credentials");
  assert.equal(calls, 1);
});

test("reports missing credentials and unparsable answers", async () => {
  const notConfigured = await translateUpstream({
    fetchImpl: jsonFetch({}),
    appId: "",
    key: "",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(notConfigured.code, "not_configured");

  const broken = await translateUpstream({
    fetchImpl: jsonFetch("<html>nope</html>", { status: 502 }),
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(broken.code, "upstream_error");

  const empty = await translateUpstream({
    fetchImpl: jsonFetch({ from: "en", to: "zh", trans_result: [] }),
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(empty.code, "upstream_error");

  const offline = await translateUpstream({
    fetchImpl: async () => {
      throw new Error("dns");
    },
    appId: "id",
    key: "key",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(offline.code, "upstream_unreachable");
});
