import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { createHandler } from "../src/handler.js";
import { clientIpFrom, createRequestListener } from "../src/node-server.js";
import { createFileStore } from "../src/store-file.js";
import { isLanguageTag } from "../src/upstream.js";

const NOW = Date.parse("2025-09-01T10:00:00Z");
const silent = { log() {}, error() {} };

async function serve(t, { handler, maxBody } = {}) {
  const server = createServer(createRequestListener({ handler, log: silent, maxBody }));
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));
  return `http://127.0.0.1:${server.address().port}`;
}

async function startGlossy(t) {
  const dir = mkdtempSync(join(tmpdir(), "glossy-cloud-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const calls = [];
  const handler = createHandler({
    store: createFileStore({ file: join(dir, "quota.json") }),
    upstream: {
      isLanguageTag,
      translate: async (input) => {
        calls.push(input);
        return { ok: true, from: "en", to: "zh", translation: "你好" };
      },
    },
    config: {
      BAIDU_APP_ID: "app",
      BAIDU_KEY: "key",
      IP_SALT: "salt",
      DAILY_CHARS_PER_CLIENT: "100",
      DAILY_CHARS_PER_IP: "150",
      DAILY_CHARS_TOTAL: "1000",
      MAX_CHARS_PER_REQUEST: "50",
      MAX_REQUESTS_PER_MINUTE: "30",
    },
    now: () => NOW,
  });

  return { base: await serve(t, { handler }), calls };
}

test("answers /v1/health over real HTTP", async (t) => {
  const { base } = await startGlossy(t);
  const response = await fetch(`${base}/v1/health`);

  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), {
    ok: true,
    service: "glossy-cloud",
    day: "2025-09-01",
    configured: true,
  });
});

test("translates through the whole stack and reports the usage", async (t) => {
  const { base, calls } = await startGlossy(t);

  const response = await fetch(`${base}/v1/translate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text: "Good morning.", clientId: "client-0000000000000001", to: "zh" }),
  });

  assert.equal(response.status, 200);
  const body = await response.json();
  assert.equal(body.translation, "你好");
  assert.equal(body.chars, 13);
  assert.deepEqual(body.usage, { client: 13, remaining: 87 });
  assert.deepEqual(calls, [{ text: "Good morning.", from: "auto", to: "zh" }]);

  const quota = await (await fetch(`${base}/v1/quota?client=client-0000000000000001`)).json();
  assert.equal(quota.usage.client, 13);
  assert.equal(quota.remaining, 87);
});

test("keeps working when the gateway adds a path prefix", async (t) => {
  const { base } = await startGlossy(t);

  const health = await fetch(`${base}/release/v1/health`);
  assert.equal(health.status, 200);

  const missing = await fetch(`${base}/release/nope`);
  assert.equal(missing.status, 404);
});

test("refuses a body over the cap without calling the handler", async (t) => {
  const { base, calls } = await startGlossy(t);

  const response = await fetch(`${base}/v1/translate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text: "x".repeat(70 * 1024), clientId: "client-0000000000000001", to: "zh" }),
  });

  assert.equal(response.status, 413);
  assert.equal((await response.json()).code, "too_long");
  assert.deepEqual(calls, []);
});

test("hands the address the gateway appended to the handler", async (t) => {
  const seen = [];
  const base = await serve(t, {
    handler: async (request) => {
      seen.push(request.headers.get("cf-connecting-ip"));
      return Response.json({ ok: true });
    },
  });

  await fetch(`${base}/v1/health`, { headers: { "x-forwarded-for": "203.0.113.7, 11.163.17.77, 10.132.167.34" } });
  await fetch(`${base}/v1/health`, { headers: { "x-real-ip": "8.8.8.8" } });
  await fetch(`${base}/v1/health`);

  assert.deepEqual(seen, ["203.0.113.7", "8.8.8.8", null]);
});

test("reads a client address out of whichever header is present", () => {
  assert.equal(clientIpFrom({ "x-forwarded-for": "203.0.113.7" }), "203.0.113.7");
  // 网关追加的边缘节点和内网跳转每个请求都不一样，必须跳过它们。
  assert.equal(clientIpFrom({ "x-forwarded-for": "203.0.113.7, 11.163.17.77, 10.132.167.34" }), "203.0.113.7");
  // 客户端自己伪造的前缀同样落在最后两段之外，不会被当成真实地址。
  assert.equal(clientIpFrom({ "x-forwarded-for": "7.7.7.7, 203.0.113.7, 11.163.17.77, 10.132.167.34" }), "203.0.113.7");
  assert.equal(clientIpFrom({ "x-forwarded-for": ["203.0.113.7", "11.163.17.77, 10.132.167.34"] }), "203.0.113.7");
  assert.equal(clientIpFrom({ "x-real-ip": "203.0.113.7" }), "203.0.113.7");
  assert.equal(clientIpFrom({}, ["x-forwarded-for"]), "");
});
