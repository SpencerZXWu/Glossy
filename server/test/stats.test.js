/**
 * The report the deployment makes about its own counters.
 *
 * The one thing it must never do is disagree with itself: the rows it lists have
 * to add up to the total printed beside them, or an operator reading it learns
 * the wrong thing about a day.
 */

import assert from "node:assert/strict";
import test from "node:test";

import { bearerToken, buildStats, tokenMatches } from "../src/stats.js";

const LIMITS = {
  charsPerClient: 100,
  charsPerIp: 150,
  charsTotal: 1000,
  maxCharsPerRequest: 50,
  requestsPerMinute: 30,
  ocrPerClientMonth: 5,
};
const NOW = Date.parse("2025-09-01T10:00:00Z");

function counters({ usage = {}, minutes = {} } = {}) {
  return { day: "2025-09-01", usage, minutes };
}

function report(over = {}) {
  return buildStats({
    day: "2025-09-01",
    month: "2025-09",
    dayCounters: counters(),
    monthCounters: counters(),
    limits: LIMITS,
    now: NOW,
    ...over,
  });
}

test("a report says whether it read one set of numbers or one instance's", () => {
  // The default is the shared reading, and a deployment that counted in one
  // place passes nothing; an instance that fell back on its own file says so.
  assert.equal(report().shared, true);
  assert.equal(report({ shared: false }).shared, false);
  // Anything that is not a real yes reads as its own, never as shared.
  assert.equal(report({ shared: 0 }).shared, false);
});

test("a report says what the counters hold, and adds up", () => {
  const stats = report({
    dayCounters: counters({
      usage: {
        "client|install-a": { chars: 30, requests: 2 },
        "client|install-b": { chars: 10, requests: 1 },
        "ip|0f1e2d3c4b5a": { chars: 40, requests: 3 },
        "total|all": { chars: 40, requests: 3 },
      },
      minutes: { "0f1e2d3c4b5a|29500000": 3 },
    }),
    monthCounters: counters({ usage: { "client|install-a": { chars: 0, requests: 2 } } }),
  });

  assert.equal(stats.day, "2025-09-01");
  assert.equal(stats.month, "2025-09");
  assert.equal(stats.today.chars, 40);
  assert.equal(stats.today.requests, 3);
  assert.equal(stats.today.installations, 2);
  assert.equal(stats.today.addresses, 1);
  assert.equal(stats.today.remaining, 960);
  assert.deepEqual(stats.today.busiestInstallations[0], {
    id: "install-a",
    chars: 30,
    requests: 2,
  });
  assert.equal(stats.today.rate.trackedMinutes, 1);
  assert.equal(stats.today.rate.requests, 3);
  assert.equal(stats.readings.total, 2);
  assert.equal(stats.readings.installations, 1);
  assert.equal(stats.generatedAt, new Date(NOW).toISOString());
  assert.equal(stats.limits.charsTotal, 1000);
});

test("an address is reported the way it is stored, and shortened", () => {
  const stats = report({
    dayCounters: counters({ usage: { "ip|0f1e2d3c4b5a6e7f": { chars: 5, requests: 1 } } }),
  });
  // The store only ever holds a hash, and the report does not put more of it on
  // screen than it takes to tell two callers apart.
  assert.equal(stats.today.busiestAddresses[0].id, "0f1e2d3c");
});

test("a report of a quiet day is still a report", () => {
  const stats = report({ dayCounters: undefined, monthCounters: undefined });
  assert.equal(stats.ok, true);
  assert.equal(stats.today.chars, 0);
  assert.equal(stats.today.installations, 0);
  assert.deepEqual(stats.today.busiestInstallations, []);
  assert.equal(stats.readings.total, 0);
  assert.equal(stats.today.remaining, 1000);
});

test("a long tail is rolled into a line of its own, so the lines still add up", () => {
  const usage = {};
  let spent = 0;
  for (let index = 0; index < 20; index += 1) {
    const chars = 100 - index;
    spent += chars;
    usage[`client|install-${index}`] = { chars, requests: 1 };
  }
  usage["total|all"] = { chars: spent, requests: 20 };

  const stats = report({ dayCounters: counters({ usage }) });
  assert.equal(stats.today.installations, 20);
  assert.equal(stats.today.busiestInstallations.length, 16);
  const tail = stats.today.busiestInstallations.at(-1);
  assert.equal(tail.rolled, 5);
  assert.equal(tail.id, "");
  // The fifteen named lines plus the roll-up are the whole day.
  assert.equal(
    stats.today.busiestInstallations.reduce((sum, row) => sum + row.chars, 0),
    stats.today.chars,
  );
});

test("a deployment with no ceiling on the day has no remainder to report", () => {
  const stats = report({
    limits: { ...LIMITS, charsTotal: 0, ocrPerClientMonth: 0 },
  });
  assert.equal(stats.today.remaining, null);
  assert.equal(stats.readings.perInstallation, null);
});

test("the token is compared whole, never as a prefix", () => {
  assert.equal(tokenMatches("s3cret", "s3cret"), true);
  assert.equal(tokenMatches("s3cre", "s3cret"), false);
  assert.equal(tokenMatches("s3cret ", "s3cret"), false);
  assert.equal(tokenMatches("S3CRET", "s3cret"), false);
  // Nothing configured, or nothing given, opens nothing.
  assert.equal(tokenMatches("", "s3cret"), false);
  assert.equal(tokenMatches("s3cret", ""), false);
  assert.equal(tokenMatches(undefined, "s3cret"), false);
  assert.equal(tokenMatches(null, null), false);
});

test("the token is read out of a bearer header however it is spelled", () => {
  assert.equal(bearerToken("Bearer s3cret"), "s3cret");
  assert.equal(bearerToken("bearer   s3cret  "), "s3cret");
  assert.equal(bearerToken("BEARER s3cret"), "s3cret");
  // Anything else is not a token, including the bare token a caller might try.
  assert.equal(bearerToken("s3cret"), "");
  assert.equal(bearerToken("Basic s3cret"), "");
  assert.equal(bearerToken(""), "");
  assert.equal(bearerToken(undefined), "");
});
