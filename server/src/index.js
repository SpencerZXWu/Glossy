/**
 * Worker entry point: binds the Durable Object that holds today's counters to
 * the request handler.
 */

import { createHandler } from "./handler.js";
import { QuotaCounter } from "./quota-object.js";
import { createUpstream } from "./upstream.js";

export { QuotaCounter };

function makeStore(env) {
  const stub = (day) => env.QUOTA.get(env.QUOTA.idFromName(day));

  return {
    reserve: (day, input) => stub(day).reserve(input),
    refund: (day, input) => stub(day).refund(input),
    peek: (day, input) => stub(day).peek(input),
  };
}

export default {
  fetch(request, env) {
    return createHandler({
      store: makeStore(env),
      upstream: createUpstream(env),
      config: env,
    })(request);
  },
};
