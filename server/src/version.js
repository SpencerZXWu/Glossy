/**
 * What the deployment may tell the App about its own version.
 *
 * From 2.1.1 onwards the App sends the version it was built from with every
 * cloud translation; older builds send nothing at all. Nothing is what a
 * missing version is treated as: a request without one is never refused and
 * never nagged, because the builds that would understand a nudge are exactly
 * the ones that already send the number. That is what makes this safe to turn
 * on for a deployment that already has users on older versions.
 *
 * Both knobs are off unless the environment sets them:
 *   MIN_VERSION   the oldest App allowed to go unnudged, e.g. `2.2.0`
 *   ANNOUNCEMENT  one line for every App; a build too old to know the field
 *                 ignores it, and one old enough to need it is nudged first
 */

/**
 * `[2, 1, 1]` for `2.1.1`, or `null` for anything that is not a version.
 *
 * A leading `v` is accepted; a pre-release or build suffix is ignored, since
 * nothing here ships one. An unparseable string gives `null`, which every
 * comparison treats as "cannot tell" rather than as an old version.
 */
function numericParts(version) {
  const match = /^\s*v?(\d+(?:\.\d+)*)\s*$/.exec(String(version ?? ""));
  if (!match) return null;
  return match[1].split(".").map(Number);
}

/**
 * `-1` when `a` is older than `b`, `1` when it is newer, `0` when they are the
 * same or either cannot be read.
 *
 * Compared field by field — never as text, which would put `2.10.0` below
 * `2.9.0`.
 */
export function compareVersions(a, b) {
  const left = numericParts(a);
  const right = numericParts(b);
  if (!left || !right) return 0;
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    const x = left[index] ?? 0;
    const y = right[index] ?? 0;
    if (x !== y) return x < y ? -1 : 1;
  }
  return 0;
}

/** Longest announcement a response carries; the card shows one line. */
const MAX_ANNOUNCEMENT = 400;

/**
 * The `notice` a translation response carries, or `null` when there is nothing
 * to say.
 *
 * An outdated App is told to update and nothing else: it has one thing to do,
 * and a second line next to it would only be noise. A version that cannot be
 * read is not outdated — the App is then left alone, which is what keeps a
 * build this deployment has never heard of working.
 *
 * An announcement is the one thing that goes out without knowing the version:
 * the field is additive, so an App that does not understand it simply ignores
 * it, and an App that does would be wrong to miss a line meant for everyone.
 */
export function noticeFor({ appVersion = "", minVersion = "", announcement = "" } = {}) {
  if (minVersion && appVersion && compareVersions(appVersion, minVersion) < 0) {
    return { code: "update_available", minVersion: String(minVersion).trim() };
  }
  const text = String(announcement || "").trim();
  if (text) return { code: "announce", message: text.slice(0, MAX_ANNOUNCEMENT) };
  return null;
}
