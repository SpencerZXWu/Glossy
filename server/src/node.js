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
import { createRequestListener } from "./node-server.js";
import { createFileStore } from "./store-file.js";
import { isLanguageTag, translateUpstream } from "./upstream.js";

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

const handler = createHandler({
  store: createFileStore({ file: stateFile }),
  upstream: {
    isLanguageTag,
    translate: (input) =>
      translateUpstream({
        fetchImpl: (...args) => fetch(...args),
        appId: process.env.BAIDU_APP_ID,
        key: process.env.BAIDU_KEY,
        endpoint: process.env.BAIDU_ENDPOINT,
        ...input,
      }),
  },
  config: process.env,
});

const server = createServer(
  createRequestListener({
    handler,
    clientIpHeaders: clientIpHeaders.length ? clientIpHeaders : DEFAULT_IP_HEADERS,
  }),
);

// 百度的往返可能比默认值慢，别让空闲连接把函数实例挂在半途。
server.keepAliveTimeout = 65_000;
server.headersTimeout = 70_000;

server.listen(port, host, () => {
  console.log(`glossy-cloud listening on http://${host}:${port}`);
  console.log(`glossy-cloud quota state: ${stateFile || "(memory only)"}`);
  console.log(`glossy-cloud baidu credentials: ${process.env.BAIDU_APP_ID && process.env.BAIDU_KEY ? "configured" : "missing"}`);
});
