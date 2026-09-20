# Changelog

All notable changes to Glossy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
See [ROADMAP.md](./ROADMAP.md) for what is planned next.

## [Unreleased]

### Changed

- The README is written in the same three languages as the release notes, in the
  same single-file layout: an English block, then 中文, then Español, each behind
  an anchor the link line at the top jumps to. The English block stays first, so
  existing links to `README.md` and its headings keep working.
- Release notes are written in three languages — English, Chinese and Spanish — in a
  single `RELEASE_NOTES.md`, with a link line at the top that jumps to an anchor placed
  directly above each language's block. `scripts/release.ps1` fills the English block
  from this file and leaves the other two as placeholders it warns about until they are
  translated, so a release cannot go out half-translated by accident. `CHANGELOG.md`
  itself stays English, as Keep a Changelog expects.

## [0.3.1] - 2026-09-20

### Changed

- Unit and currency conversion reads the numbers and units out of the
  **translation** instead of the selection. The translator is what decides
  whether a symbol is a unit at all — it settles `5 in the morning` against
  `5 in`, and it writes `12 ft` as `12英尺` for a Chinese reader — so the card
  converts what the reader is actually being shown. A source language whose unit
  words the tables never knew now works as soon as the translation writes the
  measurement in one they do, for example Spanish `mide 12 pies de ancho` →
  `房间宽12英尺。` → `12英尺 ≈ 3.66 m`. When the translation holds nothing
  convertible — the translator dropped the measurement, or spelled the number
  out in words — the original is read instead, so nothing that used to be
  annotated lost its annotation. Where both have something, the translation is
  the one that is shown.

## [0.3.0] - 2026-09-19

### Added

- Unit and currency conversion in the card. A measurement or an amount written
  the way the original's language writes it, but not the way the target language
  does, is expressed a second time in the unit a reader of the target language
  expects, with the rate underneath: `12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`,
  `212 °F ≈ 100 °C` / `°C = (°F − 32) × 5/9`, `200 US dollars ≈ ¥1,342.82` /
  `1 USD = 6.71409 CNY`. Length, mass, volume, speed, area and temperature
  convert inside their own category; the target unit is picked so a person would
  read it (5 mi → `8.05 km`, 100 lb → `45.4 kg`, never `45,359,237 mg`). Metric
  is used for every target language except English, and the target currency
  follows the target language (`zh-CN` → CNY, `en` → USD, `ja` → JPY, and so on).
- Live currency rates. The rate comes from `open.er-api.com` with the ECB
  (`frankfurter.app`) as a fallback, is cached in `%APPDATA%\com.glossy.translator\rates.json`
  for six hours, and is usable for seven days: an old table is still shown, marked
  `Stale rate`, rather than nothing. Every other unit is built into the app, so
  the feature works offline. A card whose original holds money waits at most
  2.5 s for the rate (2164 ms cold, 390 ms once the table is cached).
- A `Convert units and currency` switch in the settings. Off means the card shows
  nothing but the translation, and no rate is ever requested. On by default.
- One visual language for the three windows. A single token sheet
  (`src/styles/tokens.css`) holds every colour, radius, shadow, font size and
  duration — the palette is the Windows 11 / WinUI one, and the surface sheets
  only alias the tokens onto the names their rules already used — so the popup,
  the settings window and the "Glossy is running" card can no longer drift apart.
  `theme.js` resolves `system`/`light`/`dark` in one place instead of three
  copies of the same branch.
- The Mica material behind the settings window. On Windows 11 the window is
  transparent and Windows draws the desktop-tinted backdrop; a title bar colour
  that follows the theme is set at the same time. A build too old for Mica keeps
  an opaque background of its own — the stylesheet only turns transparent once
  the backend confirms the backdrop (`SurfaceInfo.ready`), which also removes the
  grey flash the window used to open with.

### Changed

- The card says `Looking up the dictionary…` while the phonetic symbols, parts of
  speech and definitions are still on their way, instead of staying a bare
  translation and changing later.
- Word lookups share one 1.5 s budget between their two sources, and a source that
  just timed out is not asked again for two minutes. The first lookup of a word
  went from 6025 ms to 1269 ms, and every lookup after it to 289 ms. The
  placeholder is on screen after 300–600 ms.

## [0.2.1] - 2026-09-19

### Fixed

- Word lookups no longer leave the card empty for half a minute. The translation
  is rendered as soon as the provider answers - 266-679 ms in place of the
  24-28 s a lookup used to take - and the dictionary details follow on their own
  through the new `word_details` command, merging into the card that is already
  on screen instead of holding it back.
- Phonetic symbols, parts of speech, definitions and the example sentence are
  looked up by two sources at once, and the free endpoint that carries the
  example gets a short budget of its own: one source timing out no longer
  discards what the other one found. The dictionary timeout went from 4 s to 8 s,
  which is what the endpoint's variable response time (0.75-20 s) needs.
- The history panel refreshes itself. A translation emits `glossy://history`, so
  the list in the settings window is current without restarting the app, and the
  details that arrive late are written back to the stored entry.
- The history search box follows the shared input rule - same height, radius,
  background and padding as every other field - instead of keeping the browser
  default look.

## [0.2.0] - 2026-09-18

### Added

- The popup can be pinned. The thumbtack in its header keeps the card open
  through clicks elsewhere on the desktop and switches the auto-close timer off
  until the pin is released or the card is closed.
- Historical translations. The settings window keeps the last translations (the
  cap is configurable; `Off` clears and disables the list) in `history.json`,
  with a search box that filters both the original and the translation, a copy
  button, a remove button, and a click that shows the stored card again without
  asking the provider a second time.
- Glossy can start with Windows. The new toggle writes a `--autostart` entry to
  the login items; a start that came from Windows is silent - no console window,
  no "Glossy is running" card - and leaves the tray icon in place.
- Settings can be exported to and imported from a JSON file. The export writes
  `Documents\glossy-settings.json` and only includes the API keys when the
  corresponding box is ticked and the confirmation is accepted; an import
  validates every field through `sanitized()` and protects the keys it brings
  with DPAPI before they reach the disk.
- The frontend has a test suite. `node --test` now covers `i18n.js` and
  `render.js` (71 cases, no browser needed) and runs in CI next to the Rust
  checks. The README also gained a 17-item manual regression checklist for the
  behaviour the tests cannot reach.
- An update check, through `tauri-plugin-updater`. The settings window gained an
  **Updates** section: a toggle that asks GitHub for a new release on every start
  (off by default), a **Check now** button, and a **Download and restart** button
  that installs what it finds. This build has no update signing key yet, so it
  says so instead of pretending, and the section stays inert until
  `tauri.conf.json` carries a real public key.
- The version number has a single source. `tauri.conf.json` carries the number,
  `scripts/version.ps1` writes it to the five other places that need it
  (`package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`) and
  `scripts/release.ps1` refuses to build when they disagree. `version.ps1 -Check`
  runs in CI, so a forgotten bump fails the build instead of shipping a mislabelled
  installer.
- Continuous integration: `.github/workflows/ci.yml` runs the version check,
  `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` on a Windows
  runner for every push and pull request.
- Glossy now lives in the notification area. It keeps watching for selections
  after its window is closed, and the tray icon brings the window back. Its menu
  has **Open Glossy** and **Quit**; use **Quit** to stop Glossy completely.
- A silent start now announces itself with a small card in the bottom right
  corner: *Glossy is running in the background*. It goes away after a few
  seconds, and clicking it opens the settings window.

### Changed

- The README documents building from source: the required Node and Rust (GNU
  toolchain) versions, the `rustup` command that installs them, where the installer
  and the standalone binary land, and the checks to run before pushing.
- The settings window opens on the very first launch only. Every later launch
  starts silently in the notification area, so the only way back to the settings
  is the tray icon. Delete `settings.json` to get the window back on the next
  start.
- Starting Glossy no longer opens a console window. The executable is linked as
  a Windows GUI application in every build, and diagnostics are written to the
  console of the terminal that started it, if there is one.

### Fixed

- The phonetic row of a word card is no longer drawn for a whitespace-only
  phonetic, which used to leave an empty line above the definition.
- Language codes are matched regardless of case and region spelling, so `zh-cn`,
  `ZH` and `zh` are named instead of being printed as raw codes.
- Only one Glossy can run at a time. A second launch used to install a second
  mouse hook and a second popup; it now reports that Glossy is already running
  in the notification area and exits.
- Reading a selection no longer destroys what the clipboard held: the copy that
  lifts the selection out of the foreground application replaces the whole
  clipboard, and an image, a file list or rich text used to be lost for good.
  Every published format is now captured first and written back afterwards.
- Dragging the popup by its header keeps its click area in step with the window.
  The stale rectangle made clicks on the moved popup close it, and the release
  that followed triggered a second translation.
- Triple and quadruple clicks translate once. Every click of a chain used to
  report a double click, so selecting a paragraph sent two requests; the chain is
  now collapsed into a single request for the text the last click selected.
- Short selections that end like a sentence are now translated as prose instead
  of being sent to the dictionary. "Nice to meet you." and "今天天气很好。" used to
  be judged as words because the search for punctuation skipped the last
  character.
- Google definitions keep the text that follows a less-than sign. Stripping the
  markup of a definition ended at the first `<` it could not match, so `if x < 5`
  was shown as `if x`.
- Translations show the language in words instead of the provider's own code.
  Baidu and DeepL report their own codes — `jp`, `cht`, `auto` — and the popup
  printed those raw; every provider code is now mapped onto the shared set before
  the result is displayed, and a target language that arrives in a provider's
  spelling is normalised when the settings are saved.
- A settings change is no longer lost when the window is closed right after it.
  Closing the window used to cancel the save that was still waiting for its
  moment, so the last edit was gone; the pending change is now written out before
  the window goes away.
- The shortcut row shows the state of the shortcut that was just saved. The
  window used to read the status before the backend had re-registered the key, so
  a shortcut that worked was reported as unavailable until the next refresh.
- The popup card is measured against the monitor it lands on. The height limit
  came from the monitor the window was still on, so a selection made on a screen
  with a different scale factor was cut short; the backend now reports the usable
  height of the monitor under the cursor and the card is re-measured after the
  window moves there.
- Pressing the global shortcut translates the text that is selected right now.
  It used to read the clipboard after copying, which restored the previous
  content, so an older copy was translated instead of the live selection.
- The global shortcut ignores selections made inside Glossy's own windows and in
  the applications on the ignore list, exactly like the mouse triggers do.
- Writing to the clipboard no longer leaks the memory block it allocates when a
  step of the copy fails.
- One unreadable setting no longer resets all the others. A single field that
  could not be read made the whole file fall back to the defaults; the settings
  that can be read are now kept, and the field that could not be is reported in
  the console.
- The floating popup keeps its distance on a scaled display. The gap below the
  cursor, the margin to the screen edge and the position of a dragged popup were
  measured in physical pixels, so on a display set to 125 % or 150 % the popup
  drifted away from the cursor and could leave the screen. The start hint now
  keeps its distance from the screen corner there as well.

### Security

- Saved API keys are encrypted with Windows DPAPI (`CryptProtectData`) and tied
  to the Windows login that entered them, so `settings.json` no longer carries
  them in clear text. A file written by an older version is protected the first
  time Glossy starts, and a key that belongs to another login is dropped instead
  of being sent to the provider.

## [0.1.0] - 2026-09-17

First public release.

### Added

- Selection capture based on a low-level mouse hook: the translation is triggered
  when the mouse button is released after dragging across text, or on double click.
  Selections shorter than the configured minimum length are ignored, and a drag that
  starts or ends on the popup itself never triggers a translation.
- Floating popup window below the cursor: non-activating, always on top, draggable
  by its header, clamped to the work area of the monitor under the cursor and flipped
  above the cursor when there is no room below.
- Word card with phonetic symbols, part of speech, definitions and one example;
  sentence mode renders a fluent translation instead.
- Language bar in the popup: pick either side of the pair to translate again, or press
  `⇄` to translate the result back. The pair resets for every new selection and the
  overrides are never persisted.
- Five translation providers: Google (free, no key), Baidu, Zhipu GLM, DeepL and OpenAI.
- Per-provider credentials, so switching back to an already configured provider just works.
- Settings window with a master toggle, trigger options, minimum selection length,
  global hotkey, per-program ignore list, theme, popup width, text size, opacity and
  auto-close, plus an interface language switch (`Follow Windows`, `简体中文`, `English`).
- Translate-in-place card in the settings window: type or paste text, select part of it
  (or press *Translate*) and the result renders below in the same card the popup uses.
- Global hotkey (default `Ctrl+Alt+C`) that translates the clipboard content, and a
  setting to restore the previous clipboard content after reading a selection.

[Unreleased]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.3.0
[0.2.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.2.0
[0.1.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.1.0
