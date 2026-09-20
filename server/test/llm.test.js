import assert from "node:assert/strict";
import test from "node:test";

import { DEFAULT_LLM_MODEL, extractJson, translateWithLlm } from "../src/llm.js";

function jsonFetch(payload, { status = 200, capture } = {}) {
  return async (url, init) => {
    if (capture) capture(url, init);
    return new Response(typeof payload === "string" ? payload : JSON.stringify(payload), {
      status,
      headers: { "Content-Type": "application/json" },
    });
  };
}

function completion(content) {
  return { choices: [{ message: { role: "assistant", content } }] };
}

test("asks for json and returns the translation", async () => {
  let seen;
  const result = await translateWithLlm({
    fetchImpl: jsonFetch(
      completion('{"sourceLang":"en","translation":"你好"}'),
      { capture: (url, init) => { seen = { url, init }; } },
    ),
    key: "sk-secret",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(seen.init.method, "POST");
  assert.equal(seen.init.headers.Authorization, "Bearer sk-secret");
  const body = JSON.parse(seen.init.body);
  assert.equal(body.model, DEFAULT_LLM_MODEL);
  assert.match(body.messages.at(-1).content, /Translate the text below from English into Chinese/);
  assert.match(body.messages.at(-1).content, /hello/);
  assert.deepEqual(result, { ok: true, from: "en", to: "zh", translation: "你好" });
});

test("reads the language the model detected back", async () => {
  const result = await translateWithLlm({
    fetchImpl: jsonFetch(completion('{"sourceLang":"ja","translation":"Good morning."}')),
    key: "sk-secret",
    text: "おはよう",
    from: "auto",
    to: "en",
  });

  assert.equal(result.from, "ja");
  assert.equal(result.translation, "Good morning.");
});

test("accepts an answer wrapped in a code fence", async () => {
  const fenced = '```json\n{"sourceLang":"en","translation":"你好"}\n```';
  const result = await translateWithLlm({
    fetchImpl: jsonFetch(completion(fenced)),
    key: "sk-secret",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(result.ok, true);
  assert.equal(result.translation, "你好");
  assert.equal(extractJson(fenced), '{"sourceLang":"en","translation":"你好"}');
});

test("explains a missing key without calling anyone", async () => {
  const result = await translateWithLlm({
    fetchImpl: () => {
      throw new Error("should not be called");
    },
    key: "",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "not_configured");
});

test("turns an http failure into a code the app can use", async () => {
  for (const [status, code] of [
    [401, "upstream_credentials"],
    [403, "upstream_credentials"],
    [500, "upstream_error"],
  ]) {
    const result = await translateWithLlm({
      fetchImpl: jsonFetch({ error: "nope" }, { status }),
      key: "sk-secret",
      text: "hello",
      from: "en",
      to: "zh",
    });
    assert.equal(result.ok, false, `HTTP ${status}`);
    assert.equal(result.code, code, `HTTP ${status}`);
  }
});

test("rejects an answer that is not the agreed json", async () => {
  const result = await translateWithLlm({
    fetchImpl: jsonFetch(completion("Sure! Here is the translation: 你好")),
    key: "sk-secret",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_error");
});

test("rejects a json answer with no translation in it", async () => {
  const result = await translateWithLlm({
    fetchImpl: jsonFetch(completion('{"sourceLang":"en","translation":"   "}')),
    key: "sk-secret",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_error");
});

test("reports a service that cannot be reached", async () => {
  const result = await translateWithLlm({
    fetchImpl: async () => {
      throw new Error("connect ECONNREFUSED");
    },
    key: "sk-secret",
    text: "hello",
    from: "en",
    to: "zh",
  });

  assert.equal(result.ok, false);
  assert.equal(result.code, "upstream_unreachable");
});
