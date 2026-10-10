import assert from "node:assert/strict";
import test from "node:test";

import { compareVersions, noticeFor } from "../src/version.js";

test("versions compare field by field, not as text", () => {
  assert.equal(compareVersions("2.1.1", "2.1.1"), 0);
  assert.equal(compareVersions("2.1.0", "2.1.1"), -1);
  assert.equal(compareVersions("2.1.1", "2.1.0"), 1);
  // The one a string comparison gets wrong.
  assert.equal(compareVersions("2.10.0", "2.9.0"), 1);
  // A shorter number is padded with zeroes.
  assert.equal(compareVersions("2.2", "2.2.0"), 0);
  assert.equal(compareVersions("2.2", "2.2.1"), -1);
});

test("a version that cannot be read is never called old", () => {
  assert.equal(compareVersions("", "2.0.0"), 0);
  assert.equal(compareVersions("nightly", "2.0.0"), 0);
  assert.equal(compareVersions("2.1.1", ""), 0);
});

test("a leading v is accepted", () => {
  assert.equal(compareVersions("v2.1.1", "2.1.1"), 0);
  assert.equal(compareVersions(" v2.0.0 ", "2.1.1"), -1);
});

test("nothing is said when the deployment asks for nothing", () => {
  assert.equal(noticeFor({}), null);
  assert.equal(noticeFor({ appVersion: "2.1.1" }), null);
  assert.equal(noticeFor({ appVersion: "1.0.0", minVersion: "" }), null);
});

test("an old App is told to update, and an unknown one is left alone", () => {
  const update = noticeFor({ appVersion: "2.1.0", minVersion: "2.2.0" });
  assert.deepEqual(update, { code: "update_available", minVersion: "2.2.0" });
  // The build on the minimum, and anything newer, hears nothing.
  assert.equal(noticeFor({ appVersion: "2.2.0", minVersion: "2.2.0" }), null);
  assert.equal(noticeFor({ appVersion: "2.3.0", minVersion: "2.2.0" }), null);
  // A build that sends no version at all — every one before 2.1.1 — is not
  // nagged, and neither is one whose version cannot be read.
  assert.equal(noticeFor({ minVersion: "2.2.0" }), null);
  assert.equal(noticeFor({ appVersion: "nightly", minVersion: "2.2.0" }), null);
});

test("an announcement reaches every App, version or no version", () => {
  assert.deepEqual(noticeFor({ appVersion: "2.1.1", announcement: "  明天维护  " }), {
    code: "announce",
    message: "明天维护",
  });
  // A build too old to send a version still gets the field; it ignores it.
  assert.deepEqual(noticeFor({ announcement: "明天维护" }), {
    code: "announce",
    message: "明天维护",
  });
});

test("an update outranks an announcement", () => {
  assert.deepEqual(
    noticeFor({ appVersion: "2.0.0", minVersion: "2.2.0", announcement: "明天维护" }),
    { code: "update_available", minVersion: "2.2.0" },
  );
});

test("an announcement is clipped to one line's worth", () => {
  const notice = noticeFor({ announcement: "x".repeat(1000) });
  assert.equal(notice.code, "announce");
  assert.equal(notice.message.length, 400);
});
