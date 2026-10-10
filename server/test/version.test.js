import assert from "node:assert/strict";
import test from "node:test";

import { blockedBy, compareVersions, noticeFor } from "../src/version.js";

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

test("a floor nobody has set refuses nothing", () => {
  assert.equal(blockedBy({ appVersion: "1.0.0" }), false);
  assert.equal(blockedBy({ appVersion: "1.0.0", blockBelow: "" }), false);
  assert.equal(blockedBy({ appVersion: "1.0.0", blockBelow: "   " }), false);
  // A floor that is not a version is a typo, and refusing everybody over it
  // would be the worst possible reading of one.
  assert.equal(blockedBy({ appVersion: "1.0.0", blockBelow: "latest" }), false);
  assert.equal(blockedBy({ appVersion: "", blockBelow: "nonsense" }), false);
});

test("a floor refuses every build below it and nothing else", () => {
  assert.equal(blockedBy({ appVersion: "2.0.0", blockBelow: "2.1.1" }), true);
  assert.equal(blockedBy({ appVersion: "2.1.0", blockBelow: "2.1.1" }), true);
  assert.equal(blockedBy({ appVersion: "1.9.9", blockBelow: "2.1.1" }), true);
  // At the floor and above it, the App is served as usual.
  assert.equal(blockedBy({ appVersion: "2.1.1", blockBelow: "2.1.1" }), false);
  assert.equal(blockedBy({ appVersion: "2.10.0", blockBelow: "2.9.0" }), false);
});

test("a build that sends no version looks like an old one, and is refused", () => {
  // Nothing before 2.1.1 sends a version, so this is the only way a floor can
  // reach the builds it exists for — and the reason a floor cannot be drawn
  // between, say, 1.5.0 and 2.0.0: from here they are the same build.
  assert.equal(blockedBy({ appVersion: "", blockBelow: "2.1.1" }), true);
  assert.equal(blockedBy({ appVersion: "   ", blockBelow: "2.1.1" }), true);
  assert.equal(blockedBy({ blockBelow: "2.1.1" }), true);
});

test("a version that cannot be read is not judged", () => {
  // Locking out a whole release over a suffix this file never learned would be a
  // far worse failure than serving one build too many.
  assert.equal(blockedBy({ appVersion: "2.2.0-beta.1", blockBelow: "2.1.1" }), false);
  assert.equal(blockedBy({ appVersion: "preview", blockBelow: "2.1.1" }), false);
  assert.equal(blockedBy({ appVersion: "v2.1.1", blockBelow: "2.1.1" }), false);
});
