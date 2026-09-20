/**
 * The quota rules, kept pure so every host can share them: the Cloudflare
 * Durable Object and the plain Node store used by the domestic cloud all run
 * this same code over their own copy of the state.
 *
 * State is a plain object with two maps:
 *   usage   `${scope}|${id}` -> { chars, requests }
 *   minutes `${ipHash}|${minute}` -> requests
 */

const TOTAL_SCOPE = "total";
const TOTAL_ID = "all";
const OVERFLOW_ID = "*";

export function createState() {
  return { usage: new Map(), minutes: new Map() };
}

/** Keys are shared with the Durable Object's SQL rows, so keep the separator. */
export function usageKey(scope, id) {
  return `${scope}|${id}`;
}

/** Returns a copy: the caller must not alias the object the state keeps. */
export function readUsage(state, scope, id) {
  const row = state.usage.get(usageKey(scope, id));
  return { chars: row?.chars ?? 0, requests: row?.requests ?? 0 };
}

/**
 * New buckets collapse into one shared row once the map is full, so a flood of
 * invented install ids cannot grow the state without bound.
 */
function keyFor(state, scope, id, maxBuckets) {
  const key = usageKey(scope, id);
  if (state.usage.has(key) || state.usage.size < maxBuckets) return key;
  return usageKey(scope, OVERFLOW_ID);
}

function add(state, scope, id, chars, maxBuckets) {
  const key = keyFor(state, scope, id, maxBuckets);
  const current = state.usage.get(key) ?? { chars: 0, requests: 0 };
  current.chars += chars;
  current.requests += 1;
  state.usage.set(key, current);
}

/** Counters for the App's status line. */
export function peek(state, { clientId, ipHash }) {
  return {
    client: readUsage(state, "client", clientId).chars,
    ip: readUsage(state, "ip", ipHash).chars,
    total: readUsage(state, TOTAL_SCOPE, TOTAL_ID).chars,
  };
}

/**
 * Books `chars` characters. Every check happens before anything is written, so
 * a rejected request leaves the counters untouched.
 */
export function reserve(state, { clientId, ipHash, chars, limits, now, minute }) {
  const maxBuckets = limits.maxTrackedBuckets ?? 5000;

  if (limits.requestsPerMinute > 0) {
    pruneMinutes(state, minute);
    const bucket = `${ipHash}|${minute}`;
    const used = state.minutes.get(bucket) ?? 0;
    if (used >= limits.requestsPerMinute) {
      return {
        ok: false,
        code: "rate_limited",
        message: "请求太频繁了，休息一下再试。",
        retryAfter: Math.max(1, Math.ceil((minute * 60000 + 60000 - now) / 1000)),
        limit: limits.requestsPerMinute,
      };
    }
  }

  const client = readUsage(state, "client", clientId);
  const ip = readUsage(state, "ip", ipHash);
  const total = readUsage(state, TOTAL_SCOPE, TOTAL_ID);

  const checks = [
    [client.chars, limits.charsPerClient, "client_quota_exceeded", "今天的免费翻译额度用完了，明天再来吧。"],
    [ip.chars, limits.charsPerIp, "ip_quota_exceeded", "这个网络今天的免费翻译额度用完了，明天再来吧。"],
    [total.chars, limits.charsTotal, "global_quota_exceeded", "今天的公共翻译额度已经用完了，请稍后再来。"],
  ];
  for (const [used, limit, code, message] of checks) {
    if (limit > 0 && used + chars > limit) {
      return { ok: false, code, message, limit, used, remaining: Math.max(0, limit - used) };
    }
  }

  add(state, "client", clientId, chars, maxBuckets);
  add(state, "ip", ipHash, chars, maxBuckets);
  add(state, TOTAL_SCOPE, TOTAL_ID, chars, maxBuckets);
  if (limits.requestsPerMinute > 0) {
    const bucket = `${ipHash}|${minute}`;
    state.minutes.set(bucket, (state.minutes.get(bucket) ?? 0) + 1);
  }

  return {
    ok: true,
    used: client.chars + chars,
    remaining: limits.charsPerClient > 0 ? Math.max(0, limits.charsPerClient - client.chars - chars) : null,
  };
}

/** Gives the characters back when the upstream call did not produce a translation. */
export function refund(state, { clientId, ipHash, chars }) {
  for (const [scope, id] of [
    ["client", clientId],
    ["ip", ipHash],
    [TOTAL_SCOPE, TOTAL_ID],
  ]) {
    const row = state.usage.get(usageKey(scope, id));
    if (row) row.chars = Math.max(0, row.chars - chars);
  }
}

/** Minute buckets are only needed for the current and previous minute. */
export function pruneMinutes(state, before) {
  for (const key of state.minutes.keys()) {
    const minute = Number(key.split("|").pop());
    if (Number.isFinite(minute) && minute < before) state.minutes.delete(key);
  }
}
