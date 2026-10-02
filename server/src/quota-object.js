/**
 * Daily character counters and monthly reading counters, one Durable Object per
 * key: one UTC day for the characters, one month for the readings.
 *
 * The rules themselves live in `policy.js` and are shared with the Node store.
 * Everything inside a single object runs single threaded, so loading the state,
 * running the policy and writing it back is atomic — that is the whole reason
 * for using a Durable Object instead of KV counters. Only the handful of rows a
 * day actually touches is rewritten.
 */

import { DurableObject } from "cloudflare:workers";

import {
  createState,
  peek as policyPeek,
  peekOcr as policyPeekOcr,
  refund as policyRefund,
  refundOcr as policyRefundOcr,
  reserve as policyReserve,
  reserveOcr as policyReserveOcr,
  usageKey,
} from "./policy.js";

function splitKey(key) {
  const index = key.indexOf("|");
  return [key.slice(0, index), key.slice(index + 1)];
}

export class QuotaCounter extends DurableObject {
  constructor(ctx, env) {
    super(ctx, env);
    this.sql = ctx.storage.sql;
    this.sql.exec(`CREATE TABLE IF NOT EXISTS usage (
      scope TEXT NOT NULL,
      id TEXT NOT NULL,
      chars INTEGER NOT NULL DEFAULT 0,
      requests INTEGER NOT NULL DEFAULT 0,
      PRIMARY KEY (scope, id)
    )`);
    this.sql.exec(`CREATE TABLE IF NOT EXISTS minute (
      bucket TEXT PRIMARY KEY,
      requests INTEGER NOT NULL DEFAULT 0
    )`);

    this.state = createState();
    for (const row of this.sql.exec(`SELECT scope, id, chars, requests FROM usage`).toArray()) {
      this.state.usage.set(usageKey(row.scope, row.id), {
        chars: Number(row.chars ?? 0),
        requests: Number(row.requests ?? 0),
      });
    }
    for (const row of this.sql.exec(`SELECT bucket, requests FROM minute`).toArray()) {
      this.state.minutes.set(String(row.bucket), Number(row.requests ?? 0));
    }
  }

  /** Counters for the App's status line. */
  peek(input) {
    return policyPeek(this.state, input);
  }

  reserve(input) {
    const result = policyReserve(this.state, input);
    if (result.ok) this.#flush();
    return result;
  }

  refund(input) {
    policyRefund(this.state, input);
    this.#flush();
  }

  /** The monthly readings, in an object of their own, one per month. */
  reserveOcr(input) {
    const result = policyReserveOcr(this.state, input);
    if (result.ok) this.#flush();
    return result;
  }

  refundOcr(input) {
    policyRefundOcr(this.state, input);
    this.#flush();
  }

  peekOcr(input) {
    return policyPeekOcr(this.state, input);
  }

  #flush() {
    this.sql.exec(`DELETE FROM usage`);
    for (const [key, row] of this.state.usage) {
      const [scope, id] = splitKey(key);
      this.sql.exec(`INSERT INTO usage (scope, id, chars, requests) VALUES (?, ?, ?, ?)`, scope, id, row.chars, row.requests);
    }

    this.sql.exec(`DELETE FROM minute`);
    for (const [bucket, requests] of this.state.minutes) {
      this.sql.exec(`INSERT INTO minute (bucket, requests) VALUES (?, ?)`, bucket, requests);
    }
  }
}
