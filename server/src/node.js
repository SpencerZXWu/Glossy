/**
 * Node entry point: run this for `npm run dev:node`, in a container, and on
 * Tencent Cloud SCF (as the bundled `dist/scf/index.mjs`).
 *
 * SCF 的 Web 函数要求监听 0.0.0.0:9000，正好是这里的默认值。
 */

import "./node-crypto.js";

import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { createHandler } from "./handler.js";
import { createRequestListener, DEFAULT_MAX_BODY } from "./node-server.js";
import { createRedisLink } from "./redis-link.js";
import { createFileStore } from "./store-file.js";
import { createRedisStore } from "./store-redis.js";
import { createUpstream } from "./upstream.js";

const DEFAULT_IP_HEADERS = ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"];

const port = Number.parseInt(process.env.PORT ?? "", 10) || 9000;
const host = process.env.HOST || "0.0.0.0";

// 函数实例的磁盘只有 /tmp 可写。显式把 STATE_FILE 设成空串可以关掉落盘。
const stateFile =
  process.env.STATE_FILE === undefined ? join(tmpdir(), "glossy-cloud-quota.json") : process.env.STATE_FILE;

const clientIpHeaders = (process.env.CLIENT_IP_HEADERS || "")
  .split(",")
  .map((name) => name.trim().toLowerCase())
  .filter(Boolean);

const upstream = createUpstream(process.env);

/**
 * Where the counters live.
 *
 * With `REDIS_HOST` set they live in one Redis instance that every instance of
 * this function reads and writes, so the allowance is the deployment's rather
 * than the instance's; without it they live in this instance's memory and
 * `/tmp`, as they always did. The file store is kept either way: it is what the
 * counters fall back to when Redis cannot be reached, so a Redis that is down
 * costs accuracy rather than translations.
 */
function createStores() {
  const fileStore = createFileStore({ file: stateFile });
  const fileOcrStore = createFileStore({ file: stateFile ? `${stateFile}.month` : "" });
  const host = (process.env.REDIS_HOST || "").trim();
  if (!host) {
    return { day: fileStore, month: fileOcrStore, where: stateFile || "(memory only)" };
  }

  const dbPort = Number.parseInt(process.env.REDIS_PORT ?? "", 10) || 6379;
  const link = createRedisLink({
    host,
    port: dbPort,
    user: process.env.REDIS_USER || "",
    password: process.env.REDIS_PASSWORD || "",
  });
  return {
    day: createRedisStore({ link, fallback: fileStore }),
    month: createRedisStore({ link, fallback: fileOcrStore }),
    where: `redis://${host}:${dbPort}`,
  };
}

const stores = createStores();

const handler = createHandler({
  store: stores.day,
  // 截图次数按自然月统计：日度计数每天清零，所以月度计数要另存一个键。
  ocrStore: stores.month,
  upstream,
  config: process.env,
});

// 截图走 /v1/ocr，正文比其他接口大得多，所以只给这一条路由放宽限制。
const ocrMaxBody = Number.parseInt(process.env.OCR_MAX_BODY ?? "", 10) || 4 * 1024 * 1024;

const server = createServer(
  createRequestListener({
    handler,
    clientIpHeaders: clientIpHeaders.length ? clientIpHeaders : DEFAULT_IP_HEADERS,
    maxBody: (path) => (path === "/v1/ocr" ? ocrMaxBody : DEFAULT_MAX_BODY),
  }),
);

// 大模型和百度的往返都可能比默认值慢，别让空闲连接把函数实例挂在半途。
server.keepAliveTimeout = 65_000;
server.headersTimeout = 70_000;

server.listen(port, host, () => {
  console.log(`glossy-cloud listening on http://${host}:${port}`);
  console.log(`glossy-cloud quota state: ${stores.where || "(memory only)"}`);
  console.log(`glossy-cloud upstream: ${upstream.configured ? "configured" : "missing"}`);
});
