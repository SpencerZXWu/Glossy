import assert from "node:assert/strict";
import test from "node:test";

import { createUpstream, upstreamConfigured } from "../src/upstream.js";

function baiduReply(translation = "百度的译文") {
  return new Response(JSON.stringify({ from: "en", to: "zh", trans_result: [{ src: "hello", dst: translation }] }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

function llmReply(translation = "大模型的译文") {
  return new Response(
    JSON.stringify({ choices: [{ message: { content: JSON.stringify({ sourceLang: "en", translation }) } }] }),
    { status: 200, headers: { "Content-Type": "application/json" } },
  );
}

function youdaoReply(translation = "有道的译文") {
  return new Response(JSON.stringify({ errorCode: "0", translation: [translation], l: "en" }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

/** Routes each call to the stub that matches its destination. */
function routingFetch({ llm, baidu, youdao } = {}) {
  return async (url) => {
    if (String(url).includes("bigmodel.cn")) return llm();
    if (String(url).includes("youdao.com")) return youdao();
    return baidu();
  };
}

test("knows whether the deployment holds any key", () => {
  assert.equal(upstreamConfigured({}), false);
  assert.equal(upstreamConfigured({ BAIDU_APP_ID: "app" }), false);
  assert.equal(upstreamConfigured({ BAIDU_APP_ID: "app", BAIDU_KEY: "key" }), true);
  assert.equal(upstreamConfigured({ LLM_API_KEY: "sk-secret" }), true);
  assert.equal(upstreamConfigured({ YOUDAO_APP_KEY: "app" }), false);
  assert.equal(upstreamConfigured({ YOUDAO_APP_KEY: "app", YOUDAO_APP_SECRET: "secret" }), true);
});

test("lists the vendors it can actually serve", () => {
  assert.deepEqual(createUpstream({}).vendors, []);
  assert.deepEqual(createUpstream({ YOUDAO_APP_KEY: "app", YOUDAO_APP_SECRET: "secret" }).vendors, ["youdao"]);
  assert.deepEqual(
    createUpstream({ LLM_API_KEY: "sk-secret", BAIDU_APP_ID: "app", BAIDU_KEY: "key" }).vendors,
    ["llm", "baidu"],
  );
});

test("starts at the requested vendor instead of the top of the list", async () => {
  const upstream = createUpstream({
    LLM_API_KEY: "sk-secret",
    BAIDU_APP_ID: "app",
    BAIDU_KEY: "key",
    YOUDAO_APP_KEY: "y-app",
    YOUDAO_APP_SECRET: "y-secret",
  });

  const hit = [];
  const result = await withFetch(
    async (url) => {
      if (String(url).includes("bigmodel.cn")) {
        hit.push("llm");
        return llmReply();
      }
      if (String(url).includes("youdao.com")) {
        hit.push("youdao");
        return youdaoReply();
      }
      hit.push("baidu");
      return baiduReply();
    },
    () => upstream.translate({ text: "hello", from: "en", to: "zh", vendor: "Youdao" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.translation, "有道的译文");
  assert.equal(result.vendor, "youdao");
  assert.deepEqual(hit, ["youdao"]);
});

test("falls back to the remaining vendors when the requested one fails", async () => {
  const upstream = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key", YOUDAO_APP_KEY: "y-app", YOUDAO_APP_SECRET: "y-secret" });

  const hit = [];
  const result = await withFetch(
    async (url) => {
      if (String(url).includes("youdao.com")) {
        hit.push("youdao");
        return new Response(JSON.stringify({ errorCode: "108" }), { status: 200 });
      }
      hit.push("baidu");
      return baiduReply("百度的兜底译文");
    },
    () => upstream.translate({ text: "hello", from: "en", to: "zh", vendor: "youdao" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.translation, "百度的兜底译文");
  assert.equal(result.vendor, "baidu");
  assert.deepEqual(hit, ["youdao", "baidu"]);
  // The vendor that stepped aside is named, with the code it gave, so the App
  // can say which engine refused to answer instead of showing another one's
  // work as if it were the chosen one's.
  assert.deepEqual(result.attempts, [{ vendor: "youdao", code: "upstream_credentials" }]);
});

test("names no vendor when the one that was asked for answered", async () => {
  const upstream = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key", YOUDAO_APP_KEY: "y-app", YOUDAO_APP_SECRET: "y-secret" });

  const result = await withFetch(routingFetch({ baidu: () => baiduReply() }), () =>
    upstream.translate({ text: "hello", from: "en", to: "zh", vendor: "baidu" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.vendor, "baidu");
  assert.equal(result.attempts, undefined);
});

test("ignores a vendor it does not know", async () => {
  const upstream = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key" });

  const result = await withFetch(routingFetch({ baidu: () => baiduReply() }), () =>
    upstream.translate({ text: "hello", from: "en", to: "zh", vendor: "does-not-exist" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.vendor, "baidu");
});

test("refuses to translate when nothing is configured", async () => {
  const upstream = createUpstream({});
  assert.equal(upstream.configured, false);

  const result = await upstream.translate({ text: "hello", from: "en", to: "zh" });
  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
});

/** Runs `body` with `fetch` replaced, so no test ever reaches the network. */
async function withFetch(impl, body) {
  const original = globalThis.fetch;
  globalThis.fetch = impl;
  try {
    return await body();
  } finally {
    globalThis.fetch = original;
  }
}

test("translates with baidu when that is all there is", async () => {
  const upstream = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key" });
  assert.equal(upstream.configured, true);

  const result = await withFetch(routingFetch({ baidu: () => baiduReply() }), () =>
    upstream.translate({ text: "hello", from: "en", to: "zh" }),
  );
  assert.equal(result.ok, true);
  assert.equal(result.translation, "百度的译文");
});

test("prefers the model when a key for one is present", async () => {
  const upstream = createUpstream({
    LLM_API_KEY: "sk-secret",
    BAIDU_APP_ID: "app",
    BAIDU_KEY: "key",
  });
  assert.equal(upstream.configured, true);

  const hit = [];
  const result = await withFetch(
    async (url) => {
      hit.push(String(url).includes("bigmodel.cn") ? "llm" : "baidu");
      return String(url).includes("bigmodel.cn") ? llmReply() : baiduReply();
    },
    () => upstream.translate({ text: "hello", from: "en", to: "zh" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.translation, "大模型的译文");
  assert.deepEqual(hit, ["llm"]);
});

test("hands the request to baidu when the model fails", async () => {
  const upstream = createUpstream({
    LLM_API_KEY: "sk-secret",
    BAIDU_APP_ID: "app",
    BAIDU_KEY: "key",
  });

  // The model is down, so the request has to reach Baidu instead of failing.
  const result = await withFetch(
    routingFetch({
      llm: () => {
        throw new Error("connect ECONNREFUSED");
      },
      baidu: () => baiduReply("百度的兜底译文"),
    }),
    () => upstream.translate({ text: "hello", from: "en", to: "zh" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.translation, "百度的兜底译文");
});

test("reports the model's failure when baidu cannot help either", async () => {
  const upstream = createUpstream({ LLM_API_KEY: "sk-secret", BAIDU_APP_ID: "app", BAIDU_KEY: "key" });

  const result = await withFetch(
    routingFetch({
      llm: () => {
        throw new Error("connect ECONNREFUSED");
      },
      baidu: () => new Response(JSON.stringify({ error_code: 54004 }), { status: 200 }),
    }),
    () => upstream.translate({ text: "hello", from: "en", to: "zh" }),
  );

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_limit");
});

const TENCENT_KEYS = { TENCENT_SECRET_ID: "id-1", TENCENT_SECRET_KEY: "secret-1" };
const BAIDU_OCR_KEYS = { BAIDU_OCR_API_KEY: "ocr-key", BAIDU_OCR_SECRET_KEY: "ocr-secret" };

function tencentReply(text = "腾讯认出来的字") {
  return new Response(JSON.stringify({ Response: { TextDetections: [{ DetectedText: text }] } }), { status: 200 });
}

function baiduOcrReply(text = "百度认出来的字") {
  return new Response(JSON.stringify({ words_result: [{ words: text }] }), { status: 200 });
}

/** Routes each call to the stub that matches its destination. */
function ocrFetch({ tencent, baidu } = {}) {
  return async (url, init) => {
    if (String(url).includes("tencentcloudapi.com")) return tencent(init);
    if (String(url).includes("oauth")) return new Response(JSON.stringify({ access_token: "tok-1" }), { status: 200 });
    return baidu(init);
  };
}

test("knows whether the deployment can read a picture at all", () => {
  assert.equal(createUpstream({}).ocrConfigured, false);
  assert.equal(createUpstream({ TENCENT_SECRET_ID: "id-1" }).ocrConfigured, false);
  assert.equal(createUpstream(TENCENT_KEYS).ocrConfigured, true);
  assert.equal(createUpstream(BAIDU_OCR_KEYS).ocrConfigured, true);
  assert.equal(createUpstream({ ...TENCENT_KEYS, ...BAIDU_OCR_KEYS }).ocrConfigured, true);
});

test("names the vendor it reads pictures with", () => {
  assert.equal(createUpstream({}).ocrVendor, null);
  assert.equal(createUpstream(BAIDU_OCR_KEYS).ocrVendor, "baidu");
  assert.equal(createUpstream(TENCENT_KEYS).ocrVendor, "tencent");
  // Tencent is asked first when both are held: it reads a screen a little more
  // steadily, and an operator who left the Baidu pair behind meant it as backup.
  assert.equal(createUpstream({ ...BAIDU_OCR_KEYS, ...TENCENT_KEYS }).ocrVendor, "tencent");
});

test("reads the picture with tencent when that is what is configured", async () => {
  const upstream = createUpstream(TENCENT_KEYS);
  const hit = [];
  const result = await withFetch(
    ocrFetch({
      tencent: () => {
        hit.push("tencent");
        return tencentReply();
      },
    }),
    () => upstream.ocr({ image: "aW1n", language: "zh" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.text, "腾讯认出来的字");
  assert.deepEqual(hit, ["tencent"]);
});

test("reads the picture with baidu for a deployment that only holds those keys", async () => {
  const upstream = createUpstream(BAIDU_OCR_KEYS);
  const result = await withFetch(ocrFetch({ baidu: () => baiduOcrReply() }), () =>
    upstream.ocr({ image: "aW1n", language: "zh" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.text, "百度认出来的字");
});

test("hands the picture to baidu when tencent is out of quota", async () => {
  const upstream = createUpstream({ ...TENCENT_KEYS, ...BAIDU_OCR_KEYS });
  const hit = [];
  const result = await withFetch(
    ocrFetch({
      tencent: () => {
        hit.push("tencent");
        return new Response(JSON.stringify({ Response: { Error: { Code: "ResourceUnavailable.NotExist" } } }), {
          status: 200,
        });
      },
      baidu: () => {
        hit.push("baidu");
        return baiduOcrReply("百度兜底认出来的字");
      },
    }),
    () => upstream.ocr({ image: "aW1n", language: "zh" }),
  );

  assert.equal(result.ok, true);
  assert.equal(result.text, "百度兜底认出来的字");
  assert.deepEqual(hit, ["tencent", "baidu"]);
});

test("does not ask the second vendor about a picture neither can read", async () => {
  const upstream = createUpstream({ ...TENCENT_KEYS, ...BAIDU_OCR_KEYS });
  const hit = [];
  const result = await withFetch(
    ocrFetch({
      tencent: () => {
        hit.push("tencent");
        return new Response(JSON.stringify({ Response: { TextDetections: [] } }), { status: 200 });
      },
      baidu: () => {
        hit.push("baidu");
        return baiduOcrReply();
      },
    }),
    () => upstream.ocr({ image: "aW1n" }),
  );

  assert.equal(result.ok, false);
  assert.equal(result.code, "no_text");
  assert.deepEqual(hit, ["tencent"]);
});

test("reports the first vendor's failure when neither can read the picture", async () => {
  const upstream = createUpstream({ ...TENCENT_KEYS, ...BAIDU_OCR_KEYS });
  const result = await withFetch(
    ocrFetch({
      tencent: () => {
        throw new Error("connect ECONNREFUSED");
      },
      baidu: () => new Response(JSON.stringify({ error_code: 6 }), { status: 200 }),
    }),
    () => upstream.ocr({ image: "aW1n" }),
  );

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_credentials");
});

test("refuses to read a picture when nothing is configured", async () => {
  const upstream = createUpstream({ BAIDU_APP_ID: "app", BAIDU_KEY: "key" });
  const result = await upstream.ocr({ image: "aW1n" });

  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
});
