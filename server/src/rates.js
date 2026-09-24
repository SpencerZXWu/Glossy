/**
 * Exchange rates, asked for by the App so the desktop client does not have to
 * reach the rate vendors from the user's own network.
 *
 * Two public sources, tried in turn, neither of them needing a key: the
 * exchangerate-api open endpoint is the fuller one, frankfurter (ECB) is the
 * fallback. Both are normalized into one shape before they leave here — the map
 * is always relative to the requested base and holds the base itself as 1 — so
 * the client never has to know which vendor answered.
 */

const PRIMARY_ENDPOINT = "https://open.er-api.com/v6/latest/";
const FALLBACK_ENDPOINT = "https://api.frankfurter.app/latest?from=";

const BASE_CODE = /^[A-Z]{3}$/;

/** True for something that could be a currency code, so junk never reaches a vendor. */
export function isBaseCode(value) {
  return BASE_CODE.test(String(value ?? "").trim().toUpperCase());
}

/** What the App shows under the conversion, one label per vendor. */
export const RATE_SOURCES = {
  primary: "exchangerate-api.com",
  fallback: "frankfurter.app",
};

const DEFAULT_TIMEOUT_MS = 6000;

/** Reads one answer, keeping only the codes that carry a usable number. */
function table(data) {
  const raw = data && typeof data === "object" ? data.rates : null;
  if (!raw || typeof raw !== "object") return null;

  const rates = {};
  for (const [code, value] of Object.entries(raw)) {
    if (typeof value === "number" && Number.isFinite(value) && value > 0) {
      rates[code.toUpperCase()] = value;
    }
  }
  return Object.keys(rates).length ? rates : null;
}

/** `Thu, 05 Feb 2026 00:02:31 +0000` -> `2026-02-05`. */
function isoDate(text) {
  const parts = String(text || "").split(/\s+/);
  if (parts.length < 4) return null;
  const month =
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].indexOf(
      parts[2],
    ) + 1;
  if (!month || !/^\d{4}$/.test(parts[3])) return null;
  return `${parts[3]}-${String(month).padStart(2, "0")}-${parts[1].padStart(2, "0")}`;
}

/** The primary answer, or `null` when it is not the shape we ask for. */
function fromPrimary(data) {
  if (data?.result !== "success") return null;
  const rates = table(data);
  return rates && { source: RATE_SOURCES.primary, date: isoDate(data.time_last_update_utc), rates };
}

/** The fallback answer, which leaves the base currency out of its own map. */
function fromFallback(data, base) {
  const rates = table(data);
  if (!rates) return null;
  return {
    source: RATE_SOURCES.fallback,
    date: /^\d{4}-\d{2}-\d{2}$/.test(data.date) ? data.date : null,
    rates: { ...rates, [base]: 1 },
  };
}

async function ask({ fetchImpl, url, timeoutMs }) {
  let response;
  try {
    response = await fetchImpl(url, { signal: AbortSignal.timeout(timeoutMs) });
  } catch (error) {
    return { ok: false, message: `连不上（${error}）` };
  }
  if (!response.ok) return { ok: false, message: `HTTP ${response.status}` };
  try {
    return { ok: true, data: await response.json() };
  } catch (error) {
    return { ok: false, message: `返回了无法解析的内容（${error}）` };
  }
}

/**
 * The rates for one base currency.
 *
 * `primary`/`fallback` only exist so a local mock can stand in for the vendors
 * while developing; production leaves them unset.
 */
export async function fetchRates({
  fetchImpl,
  base,
  primary,
  fallback,
  timeoutMs = DEFAULT_TIMEOUT_MS,
}) {
  const code = String(base || "").trim().toUpperCase();
  if (!isBaseCode(code)) {
    return { ok: false, code: "invalid_request", message: "需要三个字母的货币代码，例如 USD。" };
  }

  const attempts = [
    [primary || PRIMARY_ENDPOINT, (data) => fromPrimary(data)],
    [fallback || FALLBACK_ENDPOINT, (data) => fromFallback(data, code)],
  ];

  const problems = [];
  for (const [endpoint, parse] of attempts) {
    const answer = await ask({ fetchImpl, url: `${endpoint}${code}`, timeoutMs });
    if (!answer.ok) {
      problems.push(answer.message);
      continue;
    }
    const parsed = parse(answer.data);
    if (parsed) return { ok: true, base: code, ...parsed };
    problems.push("汇率内容不可用");
  }

  return { ok: false, code: "upstream_unreachable", message: `取不到汇率：${problems.join("；")}` };
}
