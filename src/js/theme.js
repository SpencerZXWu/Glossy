/**
 * Resolves the colour scheme once for every window.
 *
 * The stylesheets only know `:root[data-theme="dark"]`, so a page that wants
 * dark has to say so explicitly - including when the setting is "system". That
 * resolution happens here, before first paint, which also removes the flash of
 * the wrong palette while the settings round-trip to the backend.
 *
 * The last known choice is cached in localStorage so the first paint of a new
 * window is already correct.
 */
(() => {
  const CACHE_KEY = "glossy.theme";
  const ACCENT_KEY = "glossy.accent";
  const DENSITY_KEY = "glossy.density";

  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const root = document.documentElement;
  /** Backdrop of the current window; kept so an OS theme change cannot drop it. */
  let lastBackdrop = "none";

  function read(key) {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  }

  function write(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch {
      /* private mode: the cache is a nicety, not a requirement */
    }
  }

  function resolve(theme) {
    if (theme === "light" || theme === "dark") return theme;
    return media.matches ? "dark" : "light";
  }

  function channel(hex, index) {
    return parseInt(hex.slice(1 + index * 2, 3 + index * 2), 16);
  }

  function mix(hex, other, ratio) {
    const parts = [0, 1, 2].map((index) =>
      Math.round(channel(hex, index) * (1 - ratio) + channel(other, index) * ratio),
    );
    return `rgb(${parts.join(", ")})`;
  }

  function isHex(value) {
    return typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value);
  }

  /** Paints a custom accent, or clears the overrides so the token layer wins. */
  function applyAccent(accent, theme) {
    const style = root.style;
    if (!isHex(accent)) {
      for (const name of ["accent", "accent-hover", "accent-pressed", "accent-soft", "on-accent"]) {
        style.removeProperty(`--g-${name}`);
      }
      return;
    }

    const dark = theme === "dark";
    const luminance =
      (0.2126 * channel(accent, 0) + 0.7152 * channel(accent, 1) + 0.0722 * channel(accent, 2)) / 255;
    style.setProperty("--g-accent", accent);
    style.setProperty(
      "--g-accent-hover",
      dark ? mix(accent, "#ffffff", 0.12) : mix(accent, "#000000", 0.14),
    );
    style.setProperty("--g-accent-pressed", mix(accent, "#000000", dark ? 0.16 : 0.32));
    style.setProperty(
      "--g-accent-soft",
      `rgba(${channel(accent, 0)}, ${channel(accent, 1)}, ${channel(accent, 2)}, ${dark ? 0.16 : 0.1})`,
    );
    style.setProperty("--g-on-accent", luminance > 0.5 ? "#000000" : "#ffffff");
  }

  function apply(options = {}) {
    const theme = resolve(options.theme);
    lastBackdrop = options.backdrop || "none";
    root.dataset.theme = theme;
    root.dataset.backdrop = lastBackdrop;
    if (options.density) root.dataset.density = options.density;
    else delete root.dataset.density;
    applyAccent(options.accent, theme);
    write(CACHE_KEY, options.theme || "system");
    write(ACCENT_KEY, options.accent || "default");
    write(DENSITY_KEY, options.density || "comfortable");
  }

  media.addEventListener("change", () => {
    const cached = read(CACHE_KEY);
    if (cached === "light" || cached === "dark") return;
    apply({
      theme: "system",
      accent: read(ACCENT_KEY),
      density: read(DENSITY_KEY),
      backdrop: lastBackdrop,
    });
  });

  const boot = {
    theme: read(CACHE_KEY),
    accent: read(ACCENT_KEY),
    density: read(DENSITY_KEY),
  };
  if (boot.theme === "light" || boot.theme === "dark" || boot.theme === "system") {
    root.dataset.theme = resolve(boot.theme);
    root.dataset.backdrop = "none";
  } else {
    root.dataset.theme = resolve("system");
  }
  if (boot.density) root.dataset.density = boot.density;
  applyAccent(boot.accent, root.dataset.theme);

  window.GlossyTheme = { apply, resolve };
})();
