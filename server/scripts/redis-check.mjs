/**
 * Checks a real Redis instance the way the deployment uses it.
 *
 * The store is covered by tests against a Redis that is only as real as it needs
 * to be, which says nothing about a managed instance on the other side of a
 * network: whether the address is reachable from where the function runs, whether
 * the password is the one it wants, and whether `EVAL` is allowed at all. This
 * script answers those three in one command.
 *
 *   set REDIS_HOST=10.0.0.5
 *   set REDIS_PASSWORD=…
 *   npm run check:redis
 *
 * It writes one scratch key, reads it back, runs one reservation through the
 * store and deletes what it wrote. Nothing of the real counters is touched.
 */

import { createRedisLink } from "../src/redis-link.js";
import { createRedisStore } from "../src/store-redis.js";

const host = (process.env.REDIS_HOST || "").trim();
const port = Number.parseInt(process.env.REDIS_PORT ?? "", 10) || 6379;

if (!host) {
  console.error("没有设置 REDIS_HOST，先把它指向 Redis 实例的内网地址。");
  process.exit(2);
}

const link = createRedisLink({
  host,
  port,
  user: process.env.REDIS_USER || "",
  password: process.env.REDIS_PASSWORD || "",
  connectTimeout: 8000,
});

const prefix = "glossy:check:";
const day = `run-${Date.now()}`;
const limits = {
  charsPerClient: 100,
  charsPerIp: 150,
  charsTotal: 1000,
  maxCharsPerRequest: 50,
  requestsPerMinute: 0,
};

async function main() {
  console.log(`连接 ${host}:${port} …`);
  await link.send("SET", `${prefix}ping`, "1", "EX", "60");
  const back = await link.send("GET", `${prefix}ping`);
  if (back !== "1") throw new Error(`GET 读回来的不是刚写下的值：${back}`);
  console.log("✓ 读写正常");

  const store = createRedisStore({ link, prefix, waitMs: 4000 });
  const booked = await store.reserve(day, {
    clientId: "check",
    ipHash: "check",
    chars: 10,
    limits,
    now: Date.now(),
    minute: Math.floor(Date.now() / 60000),
  });
  if (!booked.ok) throw new Error(`预扣被拒绝了：${booked.code ?? ""} ${booked.message ?? ""}`);
  const seen = await store.peek(day, { clientId: "check", ipHash: "check" });
  if (seen.client !== 10) throw new Error(`锁与读回不一致：记账 ${seen.client}，应为 10`);
  console.log("✓ 加锁、记账、读回都正常（说明 EVAL 也能用）");

  await link.send("DEL", `${prefix}${day}`, `${prefix}ping`);
  console.log("✓ 清理完成，这个实例可以直接用了：把 REDIS_HOST 配到函数的环境变量里即可。");
}

main()
  .then(() => link.close())
  .catch((error) => {
    console.error(`✗ ${error.message}`);
    console.error("  · 连不上：先确认函数和 Redis 在同一个地域、同一个 VPC，且用的是内网地址。");
    console.error("  · 认证失败：检查 REDIS_PASSWORD / REDIS_USER（Redis 6 之后可能两样都要）。");
    link.close();
    process.exit(1);
  });
