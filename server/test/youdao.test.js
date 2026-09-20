import assert from "node:assert/strict";
import test from "node:test";

import "./../src/node-crypto.js";
import { translateWithYoudao, youdaoCode, youdaoInput } from "../src/youdao.js";

test("maps app language tags onto youdao codes", () => {
  assert.equal(youdaoCode("auto"), "auto");
  assert.equal(youdaoCode(""), "auto");
  assert.equal(youdaoCode("en"), "en");
  assert.equal(youdaoCode("EN"), "en");
  assert.equal(youdaoCode("zh"), "zh-CHS");
  assert.equal(youdaoCode("zh-CN"), "zh-CHS");
  assert.equal(youdaoCode("zh-Hant"), "zh-CHT");
  assert.equal(youdaoCode("zh-TW"), "zh-CHT");
  assert.equal(youdaoCode("pt-BR"), "pt");
  assert.equal(youdaoCode("nb-NO"), "no");
});

test("truncates the text the way the signature asks for", () => {
  assert.equal(youdaoInput("hello"), "hello");
  assert.equal(youdaoInput("x".repeat(20)), "x".repeat(20));
  assert.equal(youdaoInput("x".repeat(25)), `${"x".repeat(10)}25${"x".repeat(10)}`);
  // 长度按字符算，不是字节。
  assert.equal(youdaoInput("你".repeat(21)), `${"你".repeat(10)}21${"你".repeat(10)}`);
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

test("signs the request the way youdao expects", async () => {
  let seen;
  const result = await translateWithYoudao({
    fetchImpl: jsonFetch({ errorCode: "0", translation: ["你好"], l: "en" }, { capture: (url, init) => { seen = { url, init }; } }),
    appKey: "app-key",
    secret: "app-secret",
    text: "hello",
    from: "en",
    to: "zh-CN",
    now: () => 1_700_000_000_000,
  });

  const body = new URLSearchParams(seen.init.body);
  assert.equal(seen.url, "https://openapi.youdao.com/api");
  assert.equal(seen.init.method, "POST");
  assert.equal(body.get("q"), "hello");
  assert.equal(body.get("from"), "en");
  assert.equal(body.get("to"), "zh-CHS");
  assert.equal(body.get("appKey"), "app-key");
  assert.equal(body.get("signType"), "v3");
  assert.equal(body.get("curtime"), "1700000000");

  // sign = sha256(appKey + truncate(q) + salt + curtime + secret)
  const expected = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(`app-keyhello${body.get("salt")}1700000000app-secret`),
  );
  const hex = [...new Uint8Array(expected)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  assert.equal(body.get("sign"), hex);

  assert.equal(result.ok, true);
  assert.equal(result.translation, "你好");
  assert.equal(result.from, "en");
});

test("reports the detected source language as an app tag", async () => {
  const result = await translateWithYoudao({
    fetchImpl: jsonFetch({ errorCode: "0", translation: ["Hello"], l: "zh-CHS" }),
    appKey: "app-key",
    secret: "app-secret",
    text: "你好",
    from: "auto",
    to: "en",
  });

  assert.equal(result.from, "zh-CN");
  assert.equal(result.to, "en");
});

test("maps youdao error codes onto app codes", async () => {
  const call = (errorCode) =>
    translateWithYoudao({
      fetchImpl: jsonFetch({ errorCode }),
      appKey: "app-key",
      secret: "app-secret",
      text: "hello",
      from: "en",
      to: "zh",
    });

  assert.equal((await call("102")).code, "unsupported_language");
  assert.equal((await call("108")).code, "upstream_credentials");
  assert.equal((await call("202")).code, "upstream_credentials");
  assert.equal((await call("401")).code, "upstream_limit");
  assert.equal((await call("501")).code, "upstream_error");
  // An unknown code keeps its number so the log still says what happened.
  const unknown = await call("999");
  assert.equal(unknown.code, "upstream_error");
  assert.equal(unknown.upstream, "999");
});

test("retries the throttling answer instead of failing the user", async () => {
  const answers = ["411", "0"];
  let waited = 0;
  const result = await translateWithYoudao({
    fetchImpl: async () =>
      new Response(JSON.stringify({ errorCode: answers.shift(), translation: ["你好"] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    appKey: "app-key",
    secret: "app-secret",
    text: "hello",
    from: "en",
    to: "zh",
    sleep: async (ms) => {
      waited += ms;
    },
  });

  assert.equal(result.ok, true);
  assert.equal(result.translation, "你好");
  assert.equal(waited, 600);
});

test("reports missing credentials and unparsable answers", async () => {
  const missing = await translateWithYoudao({ fetchImpl: async () => new Response("{}"), text: "hi", from: "en", to: "zh" });
  assert.equal(missing.code, "not_configured");

  const broken = await translateWithYoudao({
    fetchImpl: jsonFetch("<html>", { status: 502 }),
    appKey: "app-key",
    secret: "app-secret",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(broken.code, "upstream_error");

  const empty = await translateWithYoudao({
    fetchImpl: jsonFetch({ errorCode: "0", translation: [] }),
    appKey: "app-key",
    secret: "app-secret",
    text: "hi",
    from: "en",
    to: "zh",
  });
  assert.equal(empty.code, "upstream_error");
});
