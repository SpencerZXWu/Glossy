/**
 * How a day of counters looks on the way in and out of storage.
 *
 * The file store and the Redis store keep the same shape, so a deployment can
 * move from one to the other without losing a day's counters: both write the
 * same JSON, and both read whatever the other one left behind.
 */

import { createState } from "./policy.js";

/** The `{day, usage, minutes}` a store writes, as in-memory state again. */
export function hydrateState(raw) {
  const state = createState();
  for (const [key, row] of Object.entries(raw?.usage ?? {})) {
    state.usage.set(key, { chars: Number(row?.chars) || 0, requests: Number(row?.requests) || 0 });
  }
  for (const [key, count] of Object.entries(raw?.minutes ?? {})) {
    state.minutes.set(key, Number(count) || 0);
  }
  return state;
}

/** The printable form of one key's counters. */
export function serializeState(day, state) {
  return {
    day,
    usage: Object.fromEntries(state.usage),
    minutes: Object.fromEntries(state.minutes),
  };
}
