/**
 * The store for a deployment with more than one instance.
 *
 * A cloud function is copied: several instances answer at once, each with its
 * own memory and its own `/tmp`, so counters kept in either of those belong to
 * one instance rather than to the deployment. That is what makes an allowance
 * look untouched (a fresh instance has counted nothing yet), and what lets the
 * same allowance be spent several times over — by an instance that was scaled
 * out during a burst, and again by the instance that replaced one which was
 * recycled for being idle.
 *
 * This store keeps the counters in one Redis instance, so every instance sees
 * the same numbers, and it holds a short lock while it reads, decides and
 * writes, so two requests arriving together cannot both spend the same
 * characters.
 *
 * The rules themselves are still `policy.js`, unchanged: the shared state is
 * read, handed to the same functions the file store uses, and written back. A
 * second copy of the rules in Lua would be a second thing to keep in step.
 *
 * Redis is an improvement, never a reason to refuse a translation: if it cannot
 * be reached, or the lock cannot be taken in time, the counters fall back to
 * this instance alone and the reason is logged. The numbers are then as wrong as
 * they were before this store existed, which is still better than an app that
 * cannot translate.
 */

import { setTimeout as sleep } from "node:timers/promises";

import {
  createState,
  peek as policyPeek,
  peekOcr as policyPeekOcr,
  refund as policyRefund,
  refundOcr as policyRefundOcr,
  reserve as policyReserve,
  reserveOcr as policyReserveOcr,
} from "./policy.js";
import { lockToken } from "./redis-link.js";
import { createFileStore } from "./store-file.js";
import { hydrateState, serializeState } from "./store-state.js";

/**
 * Releases a lock only when it is still the one this caller took.
 *
 * A plain `DEL` would be enough while nothing ever stalls. It is not enough for
 * a critical section whose lock has a deadline: a caller that lost its lock to
 * the deadline, and then finished, would delete the lock of whoever took over.
 */
const RELEASE = `
if redis.call('get', KEYS[1]) == ARGV[1] then
  return redis.call('del', KEYS[1])
end
return 0`;

/** How long a shared counter is kept after it stops being written to. */
const DEFAULT_TTL_SECONDS = 40 * 24 * 60 * 60;

export function createRedisStore({
  link,
  prefix = "glossy:quota:",
  ttlSeconds = DEFAULT_TTL_SECONDS,
  lockTtlMs = 5000,
  waitMs = 3000,
  /** Where the counters live when the shared ones cannot be reached. */
  fallback = createFileStore({ file: "" }),
} = {}) {
  let warned = false;

  /** Says once, not once per request, that the counters are local again. */
  function wentLocal(error) {
    if (warned) return;
    warned = true;
    console.error(
      `glossy-cloud: 共享计数不可用，暂时按本实例计数（额度可能不准）：${error?.message ?? error}`,
    );
  }

  async function withLock(key, token) {
    const deadline = Date.now() + waitMs;
    for (;;) {
      const taken = await link.send("SET", key, token, "NX", "PX", String(lockTtlMs));
      if (taken === "OK") return;
      if (Date.now() >= deadline) {
        throw new Error(`等待共享计数锁超时：${key}`);
      }
      await sleep(20);
    }
  }

  /** Reads the shared counters, lets `work` decide, and writes them back. */
  async function shared(day, work) {
    const lock = `${prefix}lock:${day}`;
    const key = `${prefix}${day}`;
    const token = lockToken();
    await withLock(lock, token);
    try {
      const raw = await link.send("GET", key);
      const state = raw ? hydrateState(JSON.parse(raw)) : createState();
      const { value, dirty } = work(state);
      if (dirty) {
        await link.send("SET", key, JSON.stringify(serializeState(day, state)), "EX", String(ttlSeconds));
      }
      return value;
    } finally {
      // A lock left behind is a lock nobody can take, so the release is
      // attempted whatever happened above; its own failure is not worth
      // reporting, since the deadline ends it either way.
      await link.send("EVAL", RELEASE, "1", lock, token).catch(() => {});
    }
  }

  /** Reads the shared counters without taking the lock: nothing changes. */
  async function peekShared(day, work) {
    const raw = await link.send("GET", `${prefix}${day}`);
    return work(raw ? hydrateState(JSON.parse(raw)) : createState());
  }

  /** Runs `shared`, and falls back on this instance alone when it cannot. */
  async function orLocal(run, local) {
    try {
      return await run();
    } catch (error) {
      wentLocal(error);
      return local();
    }
  }

  return {
    async reserve(day, input) {
      return orLocal(
        () =>
          shared(day, (state) => {
            const result = policyReserve(state, input);
            return { value: result, dirty: result.ok };
          }),
        () => fallback.reserve(day, input),
      );
    },

    async refund(day, input) {
      return orLocal(
        () =>
          shared(day, (state) => {
            policyRefund(state, input);
            return { value: undefined, dirty: true };
          }),
        () => fallback.refund(day, input),
      );
    },

    async peek(day, input) {
      return orLocal(
        () => peekShared(day, (state) => policyPeek(state, input)),
        () => fallback.peek(day, input),
      );
    },

    async reserveOcr(day, input) {
      return orLocal(
        () =>
          shared(day, (state) => {
            const result = policyReserveOcr(state, input);
            return { value: result, dirty: result.ok };
          }),
        () => fallback.reserveOcr(day, input),
      );
    },

    async refundOcr(day, input) {
      return orLocal(
        () =>
          shared(day, (state) => {
            policyRefundOcr(state, input);
            return { value: undefined, dirty: true };
          }),
        () => fallback.refundOcr(day, input),
      );
    },

    async peekOcr(day, input) {
      return orLocal(
        () => peekShared(day, (state) => policyPeekOcr(state, input)),
        () => fallback.peekOcr(day, input),
      );
    },
  };
}
