/**
 * Worker entry point: binds the Durable Object that holds today's counters to
 * the request handler.
 */

import { createHandler } from "./handler.js";
import { QuotaCounter } from "./quota-object.js";
import { isLanguageTag, translateUpstream } from "./upstream.js";

export { QuotaCounter };

function makeStore(env) {
  const stub = (day) => env.QUOTA.get(env.QUOTA.idFromName(day));

  return {
    reserve: (day, input) => stub(day).reserve(input),
    refund: (day, input) => stub(day).refund(input),
    peek: (day, input) => stub(day).peek(input),
  };
}

function makeUpstream(env) {
  return {
    isLanguageTag,
    translate: (input) =>
      translateUpstream({
        fetchImpl: (...args) => fetch(...args),
        appId: env.BAIDU_APP_ID,
        key: env.BAIDU_KEY,
        endpoint: env.BAIDU_ENDPOINT,
        ...input,
      }),
  };
}

export default {
  fetch(request, env) {
    return createHandler({
      store: makeStore(env),
      upstream: makeUpstream(env),
      config: env,
    })(request);
  },
};
