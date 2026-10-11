/**
 * Tencent Cloud's machine translation, the last engine in the relay's chain.
 *
 * What matters here is the two things a chain depends on: that a pair this
 * vendor does not have is refused without a request being made, so the next
 * engine is reached instead of a round trip being wasted, and that everything
 * the vendor says back is turned into one of the codes the relay already knows.
 */

import assert from "node:assert/strict";
import test from "node:test";

import { mtLanguage, mtSupports, translateWithTencentMt } from "../src/tencent-mt.js";

const NOW = Date.parse("2025-09-01T10:00:00Z");

/** Answers the shape the vendor answers with, and remembers what was asked. */
function upstream(answerFor) {
  const calls = [];
  const fetchImpl = async (url, init) => {
    calls.push({ url, init, body: JSON.parse(init.body) });
    const answer = answerFor ? answerFor(calls.length, calls[calls.length - 1]) : { TargetText: "你好" };
    return {
      status: answer.status ?? 200,
      text: async () => (answer.raw === undefined ? JSON.stringify({ Response: answer }) : answer.raw),
    };
  };
  fetchImpl.calls = calls;
  return fetchImpl;
}

function ask(over = {}) {
  const fetchImpl = over.fetchImpl ?? upstream(over.answerFor);
  return translateWithTencentMt({
    fetchImpl,
    secretId: "id-1",
    secretKey: "key-1",
    text: "hello",
    from: "en",
    to: "zh-CN",
    now: () => NOW,
    ...over.input,
  }).then((result) => ({ result, fetchImpl }));
}

test("maps the tags an App sends onto the codes this vendor takes", () => {
  assert.equal(mtLanguage("zh-CN"), "zh");
  assert.equal(mtLanguage("zh-Hans"), "zh");
  assert.equal(mtLanguage("zh-TW"), "zh-TW");
  // Hong Kong's Traditional Chinese reads as the one Traditional entry the text
  // endpoint has.
  assert.equal(mtLanguage("zh-HK"), "zh-TW");
  assert.equal(mtLanguage("EN-us"), "en");
  assert.equal(mtLanguage("ja"), "ja");
  assert.equal(mtLanguage("ko"), "ko");
  assert.equal(mtLanguage("auto"), "auto");
  assert.equal(mtLanguage(""), "auto");
  // A language this vendor does not name is not guessed at.
  assert.equal(mtLanguage("fil"), "");
  assert.equal(mtLanguage("yue"), "");
});

test("says which pairs it can take at all", () => {
  assert.equal(mtSupports("en", "zh-CN"), true);
  assert.equal(mtSupports("auto", "zh-CN"), true);
  assert.equal(mtSupports("zh-CN", "ja"), true);
  assert.equal(mtSupports("en", "fil"), false);
});

test("refuses a pair it cannot take without asking anybody", async () => {
  const { result, fetchImpl } = await ask({ input: { to: "fil" } });
  assert.equal(result.ok, false);
  assert.equal(result.code, "unsupported_language");
  assert.equal(fetchImpl.calls.length, 0, "a request was made for a pair it cannot translate");
});

test("signs a machine translation request the way the vendor expects", async () => {
  const { result, fetchImpl } = await ask();
  assert.equal(result.ok, true);
  assert.equal(result.translation, "你好");
  assert.equal(result.from, "en");
  assert.equal(result.to, "zh-CN");

  const [call] = fetchImpl.calls;
  assert.equal(call.url, "https://tmt.tencentcloudapi.com");
  assert.deepEqual(call.body, { SourceText: "hello", Source: "en", Target: "zh", ProjectId: 0 });
  assert.equal(call.init.headers["X-TC-Action"], "TextTranslate");
  assert.equal(call.init.headers["X-TC-Version"], "2018-03-21");
  assert.equal(call.init.headers["X-TC-Region"], "ap-guangzhou");
  assert.equal(call.init.headers["X-TC-Timestamp"], String(NOW / 1000));
  assert.equal(call.init.headers.Host, "tmt.tencentcloudapi.com");
  assert.match(
    call.init.headers.Authorization,
    /^TC3-HMAC-SHA256 Credential=id-1\/\d{4}-\d{2}-\d{2}\/tmt\/tc3_request, SignedHeaders=content-type;host;x-tc-action, Signature=[0-9a-f]{64}$/,
  );
});

test("leaves the endpoint and the region to a deployment that pins them", async () => {
  const { fetchImpl } = await ask({
    input: { endpoint: "http://127.0.0.1:9014", region: "ap-shanghai" },
  });
  const [call] = fetchImpl.calls;
  assert.equal(call.url, "http://127.0.0.1:9014");
  assert.equal(call.init.headers.Host, "127.0.0.1:9014");
  assert.equal(call.init.headers["X-TC-Region"], "ap-shanghai");
});

test("asks for nothing at all without a key pair", async () => {
  const fetchImpl = upstream();
  const result = await translateWithTencentMt({
    fetchImpl,
    secretId: "",
    secretKey: "",
    text: "hello",
    from: "en",
    to: "zh-CN",
  });
  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
  assert.equal(fetchImpl.calls.length, 0);
});

test("says the answer was empty when it holds no translation", async () => {
  const { result } = await ask({ answerFor: () => ({ TargetText: "  " }) });
  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_error");
});

test("maps what the vendor refuses with onto the relay's own codes", async () => {
  const cases = [
    ["FailedOperation.NoFreeAmount", "upstream_limit"],
    ["FailedOperation.ServiceIsolate", "upstream_limit"],
    ["FailedOperation.UserNotRegistered", "upstream_credentials"],
    ["AuthFailure.SignatureFailure", "upstream_credentials"],
    ["AuthFailure.SignatureExpire", "upstream_credentials"],
    ["LimitExceeded", "upstream_limit"],
    ["RequestLimitExceeded", "upstream_limit"],
    ["InternalError", "upstream_error"],
    ["SomethingNew.Whatever", "upstream_error"],
  ];
  for (const [code, expected] of cases) {
    const { result } = await ask({
      answerFor: () => ({ Error: { Code: code, Message: "no" } }),
    });
    assert.equal(result.ok, false, `${code} was answered as a translation`);
    assert.equal(result.code, expected, `${code} mapped to ${result.code}`);
    assert.equal(result.upstream, code);
  }
});

test("reports an unreachable upstream instead of throwing", async () => {
  const result = await translateWithTencentMt({
    fetchImpl: async () => {
      throw new Error("connect ECONNREFUSED");
    },
    secretId: "id-1",
    secretKey: "key-1",
    text: "hello",
    from: "en",
    to: "zh-CN",
  });
  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_unreachable");
});

test("reports a body it cannot parse", async () => {
  const { result } = await ask({ answerFor: () => ({ raw: "<html>nope</html>" }) });
  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_error");
});
