"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const TOKENS = fs.readFileSync(
  path.join(__dirname, "..", "src", "styles", "tokens.css"),
  "utf8",
);

/** The same file with its comments removed: a block's selector is what follows
    the previous one, and a comment in between would otherwise be read as part of
    the selector. */
const RULES = TOKENS.replace(/\/\*[\s\S]*?\*\//g, "");

/** Every `selector { ... }` block that states a `--g-` token, in file order. */
function blocks(css) {
  const found = [];
  const pattern = /([^{}]+)\{([^{}]*)\}/g;
  let match;
  while ((match = pattern.exec(css)) !== null) {
    const body = match[2];
    if (!/--g-[a-z-]+\s*:/.test(body)) continue;
    const declarations = {};
    for (const line of body.split(";")) {
      const declaration = /^\s*(--g-[a-z-]+)\s*:\s*(.+?)\s*$/.exec(line);
      if (declaration) declarations[declaration[1]] = declaration[2];
    }
    found.push({
      selector: match[1].trim().replace(/\s+/g, " "),
      declarations,
    });
  }
  return found;
}

const BLOCKS = blocks(RULES);

function rgb(value) {
  const hex = /^#([0-9a-f]{6})$/i.exec(value);
  if (hex) {
    const n = parseInt(hex[1], 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255, 1];
  }
  const rgba = /^rgba?\(([^)]+)\)$/.exec(value);
  if (!rgba) return null;
  const parts = rgba[1].split(",").map((part) => Number(part.trim()));
  return [parts[0], parts[1], parts[2], parts.length > 3 ? parts[3] : 1];
}

/** What a reader sees where `value` is painted on `base`. */
function paint(value, base) {
  const color = rgb(value);
  if (!color) return null;
  const under = rgb(base) || [255, 255, 255, 1];
  const alpha = color[3];
  return [
    color[0] * alpha + under[0] * (1 - alpha),
    color[1] * alpha + under[1] * (1 - alpha),
    color[2] * alpha + under[2] * (1 - alpha),
    1,
  ];
}

function luminance(color) {
  const channel = (value) => {
    const c = value / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(color[0]) + 0.7152 * channel(color[1]) + 0.0722 * channel(color[2]);
}

function contrast(foreground, background) {
  const painted = paint(foreground, background);
  assert.ok(painted, `"${foreground}" is not a colour this check can read`);
  const a = luminance(painted);
  const b = luminance(rgb(background));
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

/** The palettes the picker offers, taken from the stylesheet itself. */
const PALETTE_IDS = [
  ...new Set(
    [...RULES.matchAll(/\[data-palette="([a-z]+)"\]/g)].map((match) => match[1]),
  ),
];

/** The tokens in force for a palette and a mode, layered the way the browser
    layers them: the bare `:root` defaults, then the palette's light half, then
    the dark defaults over both, then the palette's own dark half — which is the
    order their specificity puts them in. */
function theme(palette, dark) {
  const layers = [":root"];
  if (palette) layers.push(`[data-palette="${palette}"]`);
  if (dark) layers.push(':root[data-theme="dark"]');
  if (palette && dark) layers.push(`[data-palette="${palette}"][data-theme="dark"]`);
  const merged = {};
  for (const wanted of layers) {
    for (const block of BLOCKS) {
      if (block.selector === wanted) Object.assign(merged, block.declarations);
    }
  }
  return merged;
}

test("the token layer holds a palette for every colour it states", () => {
  assert.ok(PALETTE_IDS.length >= 5, `found ${PALETTE_IDS.length} palettes`);
  for (const id of PALETTE_IDS) {
    for (const dark of [false, true]) {
      const tokens = theme(id, dark);
      if (dark) {
        assert.ok(
          RULES.includes(`[data-palette="${id}"][data-theme="dark"]`),
          `${id} has no dark half`,
        );
      }
      assert.ok(tokens["--g-surface-window"], `${id} states no window colour`);
      assert.ok(tokens["--g-accent"], `${id} states no accent`);
    }
  }
});

test("every palette keeps the text a reader has to read above 4.5:1", () => {
  const failures = [];
  for (const id of PALETTE_IDS) {
    for (const dark of [false, true]) {
      const tokens = theme(id, dark);
      const where = `${id}/${dark ? "dark" : "light"}`;
      // The card is translucent in every palette, so it is measured over the
      // window every palette states underneath it.
      const window = tokens["--g-surface-window"];
      const card = paint(tokens["--g-surface-card"], window);
      const cardText = `rgb(${card.slice(0, 3).map(Math.round).join(", ")})`;
      for (const tier of ["--g-text-primary", "--g-text-secondary", "--g-text-tertiary"]) {
        const ratio = contrast(tokens[tier], cardText);
        if (ratio < 4.5) failures.push(`${where} ${tier} = ${ratio.toFixed(2)}`);
      }
      for (const state of ["--g-ok", "--g-warn", "--g-bad"]) {
        const ratio = contrast(tokens[state], cardText);
        if (ratio < 4.5) failures.push(`${where} ${state} = ${ratio.toFixed(2)}`);
      }
    }
  }
  assert.deepEqual(failures, [], `below 4.5:1:\n  ${failures.join("\n  ")}`);
});

test("a label on an accent fill stays readable in every palette", () => {
  const failures = [];
  for (const id of PALETTE_IDS) {
    for (const dark of [false, true]) {
      const tokens = theme(id, dark);
      const ratio = contrast(tokens["--g-on-accent"], tokens["--g-accent"]);
      if (ratio < 4.5) {
        failures.push(`${id}/${dark ? "dark" : "light"} = ${ratio.toFixed(2)}`);
      }
    }
  }
  assert.deepEqual(failures, [], `below 4.5:1:\n  ${failures.join("\n  ")}`);
});

test("the default palette is the root default, stated as a palette", () => {
  // The settings window previews every entry the same way, so the WinUI colours
  // are stated as a palette too. This is what keeps the two copies from drifting.
  for (const dark of [false, true]) {
    const root = theme(null, dark);
    const palette = theme("default", dark);
    for (const name of Object.keys(palette)) {
      assert.equal(palette[name], root[name], `${name} drifted (${dark ? "dark" : "light"})`);
    }
  }
});

test("the picker offers exactly the palettes the stylesheet states", () => {
  // Two lists that have to agree: the stylesheet's blocks and the order the
  // settings window shows them in. A palette in one and not the other is either
  // a card that paints nothing or a block nobody can choose.
  const app = fs.readFileSync(path.join(__dirname, "..", "src", "js", "app.js"), "utf8");
  const list = /const PALETTES = \[([^\]]+)\]/.exec(app);
  assert.ok(list, "app.js no longer names the palettes");

  const offered = list[1]
    .split(",")
    .map((entry) => entry.trim().replace(/^"|"$/g, ""))
    .filter(Boolean);

  assert.deepEqual(offered, PALETTE_IDS);
  // And the first one is the WinUI palette the root defaults already state.
  assert.equal(offered[0], "default");
});

test("a palette can be previewed without being applied to the window", () => {
  // A palette is stated as `[data-palette]`, not on `:root` alone, because the
  // picker draws each one inside its own card by putting the attribute there.
  const selectors = BLOCKS.map((block) => block.selector);
  for (const id of PALETTE_IDS) {
    assert.ok(
      selectors.includes(`[data-palette="${id}"]`),
      `${id} is only stated for the root, so it cannot be previewed`,
    );
  }
});
