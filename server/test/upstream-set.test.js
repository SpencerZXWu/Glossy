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

/** Routes each call to the stub that matches its destination. */
function routingFetch({ llm, baidu } = {}) {
  return async (url) => {
    if (String(url).includes("bigmodel.cn")) return llm();
    return baidu();
  };
}

test("knows whether the deployment holds any key", () => {
  assert.equal(upstreamConfigured({}), false);
  assert.equal(upstreamConfigured({ BAIDU_APP_ID: "app" }), false);
  assert.equal(upstreamConfigured({ BAIDU_APP_ID: "app", BAIDU_KEY: "key" }), true);
  assert.equal(upstreamConfigured({ LLM_API_KEY: "sk-secret" }), true);
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
