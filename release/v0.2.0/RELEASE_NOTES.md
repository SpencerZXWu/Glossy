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
  of being sent to the dictionary. "Nice to meet you." and "ä»Šå¤©å¤©æ°”å¾ˆå¥½ã€‚" used to
  be judged as words because the search for punctuation skipped the last
  character.
- Google definitions keep the text that follows a less-than sign. Stripping the
  markup of a definition ended at the first `<` it could not match, so `if x < 5`
  was shown as `if x`.
- Translations show the language in words instead of the provider's own code.
  Baidu and DeepL report their own codes â€” `jp`, `cht`, `auto` â€” and the popup
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
