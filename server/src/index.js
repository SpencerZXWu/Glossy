/**
 * Worker entry point: binds the Durable Objects that hold today's counters and
 * this month's readings to the request handler.
 */

import { createHandler } from "./handler.js";
import { QuotaCounter } from "./quota-object.js";
import { createUpstream } from "./upstream.js";

export { QuotaCounter };

/** `prefix` keeps the month's objects apart from the day's in the same namespace. */
function makeStore(env, prefix = "") {
  const stub = (key) => env.QUOTA.get(env.QUOTA.idFromName(`${prefix}${key}`));

  return {
    reserve: (key, input) => stub(key).reserve(input),
    refund: (key, input) => stub(key).refund(input),
    peek: (key, input) => stub(key).peek(input),
    reserveOcr: (key, input) => stub(key).reserveOcr(input),
    refundOcr: (key, input) => stub(key).refundOcr(input),
    peekOcr: (key, input) => stub(key).peekOcr(input),
  };
}

export default {
  fetch(request, env) {
    return createHandler({
      store: makeStore(env),
      ocrStore: makeStore(env, "month-"),
      upstream: createUpstream(env),
      config: env,
    })(request);
  },
};
