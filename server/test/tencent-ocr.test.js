import assert from "node:assert/strict";
import test from "node:test";

import { recognizeWithTencent, tencentLanguage, tc3Headers } from "../src/tencent-ocr.js";

const NOW = 1_700_000_000_000;

/** A fetch that answers with whatever Tencent would answer, and records the call. */
function fakeFetch({ ocr, status = 200, body, onCall } = {}) {
  return async (url, init) => {
    if (onCall) onCall(url, init);
    if (body !== undefined) return new Response(body, { status });
    const payload = typeof ocr === "function" ? ocr() : ocr;
    return new Response(JSON.stringify(payload), { status });
  };
}

function read({ fetchImpl, ...rest } = {}) {
  return recognizeWithTencent({
    fetchImpl: fetchImpl || fakeFetch({ ocr: { Response: { TextDetections: [{ DetectedText: "hi" }] } } }),
    secretId: "id-1",
    secretKey: "secret-1",
    image: "aW1n",
    now: () => NOW,
    ...rest,
  });
}

test("maps language tags onto the models tencent can read", () => {
  assert.equal(tencentLanguage(""), "auto");
  assert.equal(tencentLanguage("auto"), "auto");
  assert.equal(tencentLanguage("zh-Hans"), "zh");
  assert.equal(tencentLanguage("zh"), "zh");
  assert.equal(tencentLanguage("en-US"), "en");
  assert.equal(tencentLanguage("ja"), "jap");
  assert.equal(tencentLanguage("ko"), "kor");
  // Nothing else is claimed: a mixed screen is what `auto` is for.
  assert.equal(tencentLanguage("ru"), "auto");
  assert.equal(tencentLanguage("fr-FR"), "auto");
});

test("reads the words off the picture and joins them line by line", async () => {
  let seen;
  const result = await read({
    language: "en",
    fetchImpl: fakeFetch({
      ocr: {
        Response: {
          TextDetections: [{ DetectedText: "第一行" }, { DetectedText: " second " }, { DetectedText: "  " }],
        },
      },
      onCall: (url, init) => {
        seen = { url, init };
      },
    }),
  });

  assert.deepEqual(result, { ok: true, text: "第一行\nsecond", language: "en" });

  const url = new URL(seen.url);
  assert.equal(url.origin, "https://ocr.tencentcloudapi.com");
  assert.equal(url.pathname, "/");
  assert.equal(seen.init.method, "POST");
  assert.equal(seen.init.headers["X-TC-Action"], "GeneralBasicOCR");
  assert.equal(seen.init.headers["X-TC-Version"], "2018-11-19");
  assert.equal(seen.init.headers["X-TC-Region"], "ap-guangzhou");
  assert.equal(seen.init.headers["X-TC-Timestamp"], String(NOW / 1000));
  assert.match(seen.init.headers.Authorization, /^TC3-HMAC-SHA256 Credential=id-1\/\d{4}-\d{2}-\d{2}\/ocr\/tc3_request, SignedHeaders=content-type;host;x-tc-action, Signature=[0-9a-f]{64}$/);
  assert.deepEqual(JSON.parse(seen.init.body), { ImageBase64: "aW1n", LanguageType: "en" });
});

test("signs the same inputs to the same signature every time", async () => {
  // Pinned against an independent HMAC-SHA256 implementation: if this changes,
  // every live call is being refused for a reason that has nothing to do with
  // the user's keys.
  //
  // The two credentials are made up rather than taken from Tencent's own
  // documentation: the documented example carries the `AKID` shape a real Secret
  // ID has, which is enough for a secret scanner to refuse the push — and a
  // fixture that trips one is a fixture nobody can clone without working around
  // it. Nothing about the signature depends on their being plausible.
  const headers = await tc3Headers({
    service: "ocr",
    host: "ocr.tencentcloudapi.com",
    action: "GeneralBasicOCR",
    version: "2018-11-19",
    region: "ap-guangzhou",
    payload: JSON.stringify({ ImageBase64: "aW1n", LanguageType: "auto" }),
    secretId: "glossy-test-secret-id",
    secretKey: "glossy-test-secret-key",
    timestamp: 1_700_000_000_000,
  });

  assert.equal(
    headers.Authorization,
    "TC3-HMAC-SHA256 Credential=glossy-test-secret-id/2023-11-14/ocr/tc3_request, " +
      "SignedHeaders=content-type;host;x-tc-action, " +
      "Signature=5d373c721a1b7d8ae113d4d1583e72fcd3c77c819099c9240d03c037b341b94b",
  );
});

test("leaves the region alone when the deployment pins another one", async () => {
  let seen;
  const result = await read({
    region: "ap-shanghai",
    endpoint: "https://ocr.example.test",
    fetchImpl: fakeFetch({
      ocr: { Response: { TextDetections: [{ DetectedText: "hi" }] } },
      onCall: (url, init) => {
        seen = { url, init };
      },
    }),
  });

  assert.equal(result.ok, true);
  assert.equal(seen.url, "https://ocr.example.test");
  assert.equal(seen.init.headers["X-TC-Region"], "ap-shanghai");
  assert.equal(seen.init.headers.Host, "ocr.example.test");
});

test("says there is no text when the picture holds none", async () => {
  const result = await read({ fetchImpl: fakeFetch({ ocr: { Response: { TextDetections: [] } } }) });
  assert.equal(result.ok, false);
  assert.equal(result.code, "no_text");
});

test("asks for nothing at all without a key pair", async () => {
  let called = false;
  const result = await read({
    secretId: "",
    fetchImpl: fakeFetch({ onCall: () => (called = true) }),
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
  assert.equal(called, false);
});

test("maps tencent refusals onto the relay's own codes", async () => {
  const cases = [
    ["FailedOperation.UnOpenError", "upstream_limit"],
    ["ResourceUnavailable.InArrears", "upstream_limit"],
    ["ResourceUnavailable.NotExist", "upstream_limit"],
    ["RequestLimitExceeded", "upstream_limit"],
    ["AuthFailure.SignatureFailure", "upstream_credentials"],
    ["AuthFailure.SecretIdNotFound", "upstream_credentials"],
    ["FailedOperation.OcrFailed", "upstream_error"],
    ["InternalError", "upstream_error"],
    ["SomethingBrandNew", "upstream_error"],
  ];

  for (const [code, expected] of cases) {
    const result = await read({ fetchImpl: fakeFetch({ ocr: { Response: { Error: { Code: code, Message: "x" } } } }) });
    assert.equal(result.ok, false, code);
    assert.equal(result.code, expected, code);
    assert.equal(result.upstream, code);
  }
});

test("reports an unreachable upstream instead of throwing", async () => {
  const result = await read({
    fetchImpl: async () => {
      throw new Error("boom");
    },
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_unreachable");
});

test("reports a body it cannot parse", async () => {
  const result = await read({ fetchImpl: fakeFetch({ body: "<html>502</html>", status: 502 }) });
  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_error");
  assert.match(result.message, /502/);
});
