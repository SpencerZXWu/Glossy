/**
 * What the deployment can say about its own counters.
 *
 * The counters exist to keep the free allowance honest. This is the same numbers
 * read the other way round, for the person who runs the deployment: how much of
 * today's allowance is gone, whose it went to, and how many readings the month
 * has used. It is answered from the counters themselves rather than from a second
 * book kept for the purpose, so it can never disagree with what an App is told.
 *
 * None of it is for a client to see: the report names installation ids and what
 * each of them has spent. `/v1/stats` is therefore off unless the deployment sets
 * `ADMIN_TOKEN`, and answers only to that.
 */

/** How many rows of each kind a report carries before the rest is rolled up. */
const TOP_ROWS = 15;

/** Splits `client|abc` back into its two halves. */
function splitKey(key) {
  const cut = key.indexOf("|");
  return cut === -1 ? ["", key] : [key.slice(0, cut), key.slice(cut + 1)];
}

/** Every row of one scope, biggest first. */
function rowsOf(usage, scope) {
  const rows = [];
  for (const [key, row] of Object.entries(usage ?? {})) {
    const [rowScope, id] = splitKey(key);
    if (rowScope !== scope) continue;
    rows.push({ id, chars: Number(row?.chars) || 0, requests: Number(row?.requests) || 0 });
  }
  return rows.sort((left, right) => right.chars - left.chars || right.requests - left.requests);
}

/**
 * The first `limit` rows, with everything after them added up into one line of
 * its own: a report that quietly dropped them would not add up to the total
 * printed next to it, which is the one thing a report must never do.
 */
function top(rows, limit = TOP_ROWS) {
  if (rows.length <= limit) return rows;
  const rest = rows.slice(limit);
  return [
    ...rows.slice(0, limit),
    {
      // No id and a count instead: what this line is called is the display's
      // business, and the two languages that read it do not agree.
      id: "",
      rolled: rest.length,
      chars: rest.reduce((sum, row) => sum + row.chars, 0),
      requests: rest.reduce((sum, row) => sum + row.requests, 0),
    },
  ];
}

/** An address is already a hash; eight characters are enough to tell two apart. */
function shortAddress(id) {
  return String(id).slice(0, 8);
}

/**
 * Wheher the caller holds the token the deployment named.
 *
 * Compared without stopping at the first difference, so how long the answer takes
 * says nothing about how much of the token was right. The length is not hidden,
 * which for a long random token is not worth hiding.
 */
export function tokenMatches(given, expected) {
  const left = String(given ?? "");
  const right = String(expected ?? "");
  if (!left || !right || left.length !== right.length) return false;
  let difference = 0;
  for (let index = 0; index < left.length; index += 1) {
    difference |= left.charCodeAt(index) ^ right.charCodeAt(index);
  }
  return difference === 0;
}

/** The token out of an `Authorization: Bearer …` header, or an empty string. */
export function bearerToken(header) {
  const text = String(header ?? "").trim();
  const match = /^Bearer\s+(.+)$/i.exec(text);
  return match ? match[1].trim() : "";
}

/**
 * One report of what the counters hold.
 *
 * `dayCounters` and `monthCounters` are the stored shapes — `{ day, usage,
 * minutes }` — so this works the same whether they came out of a file, a Redis
 * key or a Durable Object.
 */
export function buildStats({
  day,
  month,
  dayCounters,
  monthCounters,
  limits,
  now,
  shared = true,
}) {
  const clients = rowsOf(dayCounters?.usage, "client");
  const addresses = rowsOf(dayCounters?.usage, "ip");
  const total = rowsOf(dayCounters?.usage, "total").find((row) => row.id === "all") ?? {
    chars: 0,
    requests: 0,
  };
  const minutes = Object.values(dayCounters?.minutes ?? {});
  const readings = rowsOf(monthCounters?.usage, "client").sort(
    (left, right) => right.requests - left.requests,
  );

  return {
    ok: true,
    service: "glossy-cloud",
    day,
    month,
    generatedAt: new Date(now).toISOString(),
    // Whether this reading came from the deployment's own counters or from one
    // instance's copy of them: a report that does not say which is a report that
    // can be read as an exact answer when it is one instance's guess.
    shared: Boolean(shared),
    limits: {
      charsPerClient: limits.charsPerClient,
      charsPerIp: limits.charsPerIp,
      charsTotal: limits.charsTotal,
      maxCharsPerRequest: limits.maxCharsPerRequest,
      requestsPerMinute: limits.requestsPerMinute,
      ocrPerClientMonth: limits.ocrPerClientMonth,
    },
    today: {
      chars: total.chars,
      requests: total.requests,
      installations: clients.length,
      addresses: addresses.length,
      remaining: limits.charsTotal > 0 ? Math.max(0, limits.charsTotal - total.chars) : null,
      busiestInstallations: top(clients),
      busiestAddresses: top(
        addresses.map((row) => ({ ...row, id: shortAddress(row.id) })),
      ),
      // The rate limiter only ever holds the last minute or two, so this is a
      // picture of right now rather than of the day.
      rate: {
        trackedMinutes: minutes.length,
        requests: minutes.reduce((sum, count) => sum + (Number(count) || 0), 0),
      },
    },
    // The month's readings, which are counted apart from the day's characters:
    // one screenshot costs the vendor one call whatever is on it.
    readings: {
      total: readings.reduce((sum, row) => sum + row.requests, 0),
      installations: readings.length,
      busiestInstallations: top(readings.map((row) => ({ ...row, chars: 0 }))),
      perInstallation: limits.ocrPerClientMonth > 0 ? limits.ocrPerClientMonth : null,
    },
  };
}
