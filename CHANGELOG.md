# Changelog

All notable changes to Glossy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
See [ROADMAP.md](./ROADMAP.md) for what is planned next.

## [1.2.1] - 2026-09-24

A language bar now tells the truth about the engine behind it. The eight languages a
standard Baidu account refuses are no longer offered, and the language a reader picks in
the card is the language the next selection starts from.

### Changed

- **A language bar offers only the languages the chosen engine translates.** The bars were
  built from one list for all three engines, and a standard Baidu account refuses eight of
  its 31 entries — `uk`, `tr`, `hi`, `id`, `ms`, `he`, `no` and `sk` — so 百度 could be
  asked for a language it answers with `58001`. `src-tauri/src/translate/languages.rs` is
  now the only table (`ALL`, plus `BAIDU`, `YOUDAO` and Google's, which is every language),
  published to both windows through the new `service_languages` command, and the two bars —
  the card's and the settings window's — rebuild themselves from it whenever the engine
  changes, in either direction. A code the provider detected that the shared menu never had
  stays selectable, so a bar is never blank, and a target the backend is handed anyway is
  clamped to a language the engine does take rather than sent and refused.
- **The language a reader picks in the card is the configured language from then on.** The
  target chosen in either window's bar was good for the card on screen and forgotten at the
  next selection, so translating a page into Japanese and then selecting the next word put
  it back into the configured language. The new `set_target_lang` stores the pick, and the
  source language keeps its old behaviour — it is reset to *detect it* for every selection.
  The swap button does not save either, so a reversal does not change the setting.

## [1.2.0] - 2026-09-22

Windows is no longer the operating system of the whole backend: every call into Win32 now
lives behind one dispatch layer, so a port has a single place to stand. The window content
is governed by an explicit Content Security Policy instead of none, and a release ships
three ways — the installer, an MSI package and a portable zip. macOS and Linux are still
ahead (`src/platform/mod.rs` refuses to compile anywhere but Windows), and this release
deliberately stops at the interface those ports will be built against.

### Added

- **The platform layer.** `src/platform/mod.rs` publishes what the rest of the backend is
  allowed to assume about an operating system — without a screen, `ScreenRect` and its
  padded hit test — and dispatches to `src/platform/windows/`, which holds the ten modules
  that talk to Win32: `clipboard`, `console`, `desktop`, `hotkey`, `input`, `input_hook`,
  `instance`, `secrets`, `speech` and `uia`. A target that is not Windows fails to compile
  with a message pointing at the layer instead of failing to link. No module in it leaks a
  `windows` type through its public surface: the low-level mouse hook reports a
  `Click { pressed, x, y }`, the window handle travels as an `isize`, and the accessibility
  text read for a word's sentence is asked for as a `Unit::Line` or `Unit::Paragraph`.
- **An MSI package and a portable zip, next to the installer.** `bundle.targets` builds
  NSIS and MSI, and `scripts/release.ps1` stages both, then packs
  `Glossy_<version>_x64_portable.zip` — the release binary and the `WebView2Loader.dll` the
  GNU toolchain links against, with nothing to install — and checksums all three. A build
  that names MSI in its targets but produces no MSI fails the release instead of staging
  two files out of three.

### Security

- **The content is behind a policy now.** `app.security.csp` was `null`, which let the
  window load anything a script asked for. It is an explicit whitelist: `default-src`,
  `script-src`, `style-src` and `font-src` are `'self'`, images may add `data:`, and
  `object-src`, `frame-src` and `form-action` are closed. The frontend loads no remote
  script, style, font or image, so nothing had to be widened for it — verified in the
  running app by reading each window's console, where a deliberate remote `fetch` is
  refused and nothing this app loads is.

### Changed

- **The backend's OS-bound modules moved under `src/platform/windows/`.** `clipboard.rs`,
  `console.rs`, `hotkey.rs`, `input.rs`, `instance.rs`, `secrets.rs`, `speech.rs` and
  `platform.rs` (now `desktop.rs`) were moved, not rewritten, so the behaviour of a
  selection is the same one as before. What they exported is reached through the layer
  now: `platform::clipboard`, `platform::hotkey`, `platform::speech`, `platform::instance`
  and `platform::desktop` from the window setup, the popup, the hint, the settings and the
  console borrow. The mouse hook itself was split: the thread, the message loop and the
  callback plumbing live in `platform::windows::input_hook`, and the click chain that
  decides what a gesture meant stays in `selection.rs`, where it can be tested without a
  screen.
- **`windows` and `window-vibrancy` are Windows-only dependencies.** `Cargo.toml` declares
  them under `[target.'cfg(windows)'.dependencies]`, so the tree a port starts from does
  not contain them at all.
- **The screenshot-shaped parts still inside the Windows modules are named as debt.** The
  hotkey's key parsing, the speech queue, `secrets::is_protected`, `context::sentence_in`
  and the click chain in `selection.rs` are portable but still sit in OS-bound files; the
  capability table in `src/platform/mod.rs` says so, and the roadmap records it.

### Fixed

- **The start hint goes away every time it appears.** A 6.5 second timer in the page took it
  off screen, and the page loads once: the timer ran for the first hint and never again, so
  the card a later start of Glossy put back stayed there for as long as the process lived.
  Starting Glossy a second time — double clicking the shortcut again, which is how most
  people reopen the settings window — is exactly the path that showed it. The countdown runs
  in `notice.rs` now, started by `notice::show` itself and guarded by a show counter, so
  every show gets a full one and a countdown left over from an earlier hint cannot take down
  the one that replaced it. `notice.js` no longer arms a timer.
- **The names that live only in an attribute are translated.** `i18n.apply` fills
  `data-i18n-aria-label` beside `data-i18n-title`, which is what the close button of the hint
  and the interface language selector were missing: both had an English tooltip and an
  English accessible name in every language, and the selector's `ui.lang` key — written for
  exactly this control — had no reader left. It has one again.
- **A counted sentence reads correctly for one item.** `"{0} program(s) ignored."` printed
  its own brackets at one program. `ignored.count` and `source.count` are stored as
  `key.one` and `key.other` and read through the new `i18n.plural`, so a single item says
  `1 program ignored.` The Chinese text is unchanged.
- **Choosing a language no longer takes the card away.** The list of a language selector is
  a window of its own, drawn by the browser process of the webview and taller than the card,
  so pressing an entry fell outside the card's edges: the mouse hook read it as a click on
  the program behind Glossy and dismissed the card mid-choice. A click now also belongs to
  the card when the window under it hangs off one of the card's windows or is drawn by the
  process that draws the card, so the entries are pressed normally and only a click that
  really lands elsewhere closes the card. `platform::desktop::owns_point` walks the owner
  chain from the top level window down, `child_process_id` finds the browser process behind
  the card, and `popup::owns_point` remembers both when the card is placed.

## [1.1.2] - 2026-09-22

The bottom line of the card and the order of the settings window: the card says which
engine translated it and can change engines in place, and the settings finally group each
switch with the thing it changes. The local model is gone, so nothing has to be installed
to translate.

### Added

- **The card names the engine that translated it, and can change it.** The line under
  the translation printed the literal word `cloud` for both 百度 and 有道, because it
  was reading the storage shape instead of the service. It names the service that
  answered now — Baidu, Youdao or Google — and the name is a button: pressing
  it opens the services in place inside the card with the current one ticked,
  and picking one saves the choice and translates the selection again. `cloud::answered_by`
  keeps the literal `cloud` out of what the card is told, `current_service` and
  `set_service` carry the choice back and forth, and Escape closes the list before it
  closes the card.

### Changed

- **The settings window shows the services as plain names.** The line under the
  dropdown that promised a service needed no setup and cost nothing is gone. The card's
  own line carries the same names, so the two places a service is chosen finally agree.
- **The name of the engine shares one line with the two read-aloud buttons.** It sat on
  the line above them and could wrap away from them on a narrow card. The footer is a
  single row now, and the name is given the room it needs and kept to one line
  rather than wrapped, so the buttons keep their place at the end of the line.
- **The settings window groups each switch with the thing it changes.** The panels now
  run from the master switch through triggering, languages, the fallback service and the
  card's options to reading, units, history, the settings file and updates, and the two
  switches that describe the card — showing the original sentence and the compact layout
  — sit with the rest of the card's options instead of in the master and reading panels.
  Starting Glossy with the system is its own group: it sat among the switches the master
  switch dims while translation is off, which left it unreachable from a paused app.

### Removed

- **The local model is no longer an option.** The entry that translated through Ollama,
  or any other OpenAI compatible server on the same machine, is gone from the dropdown,
  from the fallback list and from the card's own list, along with the address and model
  fields it needed and the two settings keys behind them. Nothing has to be installed to
  translate: the two engines that go through Glossy's server and the free Google
  endpoint are the whole list. A settings file that still names the local model reads as
  the built-in engine and is rewritten on the next save.

## [1.1.1] - 2026-09-21

The appearance pass v1.1.0 left open: the popup was measured in a browser, and what
came back was contrast, type size, motion and two states that said the same thing
twice. Almost nothing here changes what Glossy does; the read-aloud buttons are the
one exception, and that is because their logic was wrong as well as their look.

### Changed

- **The muted text is readable now.** The phonetic symbols, the example, the
  footnotes, the block titles and the line that stands in while a word card is still
  being completed all drew from `--g-text-tertiary`, which measured about 3.3:1
  against the card. The token is `rgba(0, 0, 0, 0.558)` in light and
  `rgba(255, 255, 255, 0.5646)` in dark, which puts every one of them at 4.86:1 or
  better in either theme.

- **The opacity setting dims the plate, not the words.** The card used to fade as a
  whole, so a setting below about 70 % took the text down with it. The surface moved
  into `.card::before` — border, background and shadow, with `isolation: isolate` on
  the card to keep the layer under the text — and that layer is the only thing
  `--popup-opacity` touches. The text stays fully opaque at the 50 % minimum.

- **The smallest text and the buttons both grew.** The close, copy and pin buttons go
  from 24px to `calc(26px * var(--popup-font))` with a 15px icon, the language swap
  button is 24px, the footnotes from 10.5px to 11.5px and the block titles from 10px
  to 10.5px.

- **The read-aloud buttons are a pair of icons in the corner.** They were two wide
  pills under the translation; they are now 26px buttons built like the pin, copy and
  close buttons, side by side and right-aligned at the bottom corner of the card,
  each one labelled for its own side. A speaker at rest becomes a stop square while
  that side is being read, so which of the two is playing is never something the
  reader has to remember.

- **Nothing flashes while a word card is completed.** `refine()` used to draw a card
  with the extras stripped out and fill them in afterwards; it draws the one card with
  a pending marker instead, so the provider's content is never wiped and redrawn.

- **Motion and colour come from the tokens.** Every literal `120ms ease` became
  `var(--g-dur-fast) var(--g-ease-standard)`, the entrance animation translates the
  card without animating its opacity, `.card::before` fades in on its own keyframes,
  the focus ring is one `:focus-visible` rule on `--accent`, and the scrollbar thumb is
  `--g-stroke-strong`, lifting to `--muted` under the pointer.

### Fixed

- **Pressing a pronunciation button a second time did nothing.** `say` answers as
  soon as the voice has accepted the text — the reading itself runs on a SAPI thread
  of its own — so a second press on a button that was still lit was treated as a
  first press: the highlight lasted one frame and a reading could only be stopped by
  closing the card. `speech.rs` now keeps a speaking flag under a generation counter,
  the `speaking` command reports it, and the card follows it on a 250ms timer so the
  button goes back to rest when the voice falls silent.

- **A reading outlived the card that started it.** Closing the card, replacing it
  with a new selection, or switching to the other side all used to leave the old
  reading talking over the new one. Each of them stops the voice first: `popup::hide`
  on the way out, `dismiss()` before the window goes, and the card itself before
  arming the next button.

- **A reading that could not start said nothing.** A missing voice, a busy one or an
  outright refusal now says so on the button that asked — tinted, relabelled, and
  back to rest a couple of seconds later.

- **Reduced motion was not honoured.** The `prefers-reduced-motion` block sat above
  the rules it was meant to switch off, and at equal specificity the later declaration
  wins, so the loading shimmer and the plate fade kept playing. The block is last in
  `popup.css` now and covers `.card::before`, not just `.card`.

- **An error message repeated itself.** The card printed the localized headline and
  then the raw message, and the two were often the same sentence. The headline is
  always the localized one, and the provider's own text follows as a muted note only
  when it is different and not empty.

- **An empty translation looked like an answer.** The fallback message draws as
  `translation empty` — muted and italic — so it reads as a notice. The line that
  stands in while a lookup is running stays a plain muted line: a pulsing dot would
  claim more liveness than a single lookup deserves.

## [1.1.0] - 2026-09-21

### Added

- **The word card shows how the word behaves.** A word entry now carries its
  inflections (plural, third person, participles, past, comparative, superlative),
  the words that mean roughly the same, and — when **Reading** is on — the sentence
  the selection was taken from next to a translation of that sentence. The Rust side
  does the work: `morphology.rs` derives the forms, `translate/dictionary.rs` and
  `translate/mod.rs` collect and de-duplicate the alternatives, and `context.rs`
  looks the sentence up.

- **Sentence by sentence.** A paragraph whose translation divides differently than
  the original is no longer a wall of text: with **Sentence by sentence** on, the
  card lists the original and the translation one aligned row at a time. A
  translation that comes back as a single row is left as the plain paragraph, since
  a one-row table would only repeat the card.

- **Read it out loud.** Every card carries a pronunciation button for the original
  and one for the translation. The text is spoken through Windows SAPI at the rate
  chosen in the settings window (**Speaking rate**: Slow / Normal / Fast / Very
  fast), and pressing the button again stops it, as does closing the card.

- **A fallback order you can set.** When the chosen service fails, Glossy now walks
  a list of the other services instead of giving up. The settings window shows that
  list under **Fallback**: the service chosen above is always tried first and cannot
  be moved, the rest can be reordered with the arrows, and the whole thing can be
  turned off. The card names the service that answered when it was not the one
  asked for.

- **A compact card.** **Compact popup** keeps the translation, the phonetic symbols
  and the meanings, and leaves out the example, the inflections, the synonyms, the
  context sentence, the sentence-by-sentence view and the unit conversions — for a
  popup that is meant to be read at a glance.

- **Reading, and how much of it.** The settings window has a **Reading** panel that
  decides what a word card is allowed to show: **Word in its sentence** (the context
  lookup), **Sentence by sentence** (the aligned view) and **Compact popup**, plus
  the **Speaking rate** of the pronunciation buttons.

- **The selection is cleaned up before it is sent.** `text.rs` reduces what
  Ctrl+C handed over to the sentences the author wrote: soft hyphens and zero-width
  spaces, terminal escape sequences, line breaks the window width inserted, runs of
  non-breaking spaces and stray control characters are removed, so the provider is
  not asked to translate the layout of the page.

### Changed

- **Exporting the settings file no longer asks about keys.** With no field left
  anywhere that takes an API key, there is nothing an export could carry: the
  checkbox, its warning and the confirmation dialog are gone, and
  `export_settings` has no `include_credentials` argument any more. The file holds
  the choices and nothing secret.

- **Four translation services, and no key field anywhere.** The dropdown in the
  settings window now holds exactly four entries — **Baidu Translate** and **Youdao
  Translate** (the project's server, each naming the upstream it should use, so
  there is nothing to fill in), **Local translation · Ollama** (any
  OpenAI-compatible endpoint on this machine) and **Google** (the free public
  endpoint) — and the **Extensions · my own API** panel is gone, along with the
  Zhipu, DeepL and OpenAI entries it served. A new install starts on Baidu
  Translate. Nothing about the stored shape changed: `channel` + `cloudProvider` +
  `cloudVendor` / `provider` are written exactly as before, so a settings file from
  an older build keeps working — a `provider` this build no longer offers reads as
  the built-in Baidu entry and is replaced the next time the file is written, and a
  `cloudVendor` that is not `youdao` reads as Baidu. README and ROADMAP were updated
  in all three languages to match.

- **The original line in the card is capped at four lines.** A long selection used
  to push the translation down and out of view; the source block now scrolls inside
  itself once it would grow past four lines, in the floating popup and in the card
  of the settings window alike. The translation is what the card is read for, so it
  keeps its place.

### Fixed

- The confirmation that appears at the bottom of the settings window was invisible
  in dark mode while the window sat on the Mica backdrop: the toast drew its
  background from `--fg` and its text from `--bg`, and Mica remaps `--bg` to a 6 %
  white, so dark-mode text landed on a dark surface. The toast has its own
  `--g-toast-surface` / `--g-toast-text` / `--g-toast-outline` tokens now, and
  dialog boxes switched to a solid surface with the dialog shadow, so both read the
  same in either theme and under any backdrop.

## [1.0.2] - 2026-09-20

### Added

- **Baidu and Youdao as channels of their own, still with nothing to set up.**
  The dropdown now lists **Baidu Translate** and **Youdao Translate** next to
  Glossy's own server. Both are that same server — no key, no field to fill in —
  and they only say which upstream it should translate with, so the vendor that
  answers is visible in the list without anyone handling credentials. The server
  answers with the named upstream first (the request gained a `vendor` field) and
  still falls back to the others when it cannot, and the result card names whichever
  one answered. The settings file stores the choice as `cloudVendor` (`baidu`,
  `youdao`, or empty for "let the server decide"), the server keeps its
  `/v1/health` list of the upstreams a deployment actually has, and
  `server/src/youdao.js` implements the Youdao Zhiyun upstream with its v3
  signature.

- **Translation service, one list.** The settings window no longer asks whether
  results come from a cloud or from your own account; it asks which service
  translates, and the same dropdown holds all of them: Glossy's own server, which
  holds the provider credentials so no key of yours is involved; a model running on
  this machine over any OpenAI-compatible endpoint (Ollama's
  `http://127.0.0.1:11434/v1` by default, model `qwen2.5:7b`), which keeps the text
  on the computer and costs nothing; the free public Google endpoint, which needs no
  key; and then the vendors that take the user's own key — Baidu, Zhipu, DeepL and
  OpenAI. A new settings file starts on Glossy's server, and a file written before
  this release keeps the provider it already had, so nothing has to be re-entered.
  The credentials of the last group moved out of that panel into **Extensions · my
  own API**, a panel of its own at the bottom of the settings window, where they sit
  dimmed until one of those services is chosen. Under the stored shape nothing
  changed — Glossy's server and the local model are still the cloud channel, the
  rest still use the API channel — so an older build reads the file as it always
  did. The server gained a second upstream to go with it: it now translates through
  an LLM account when one is configured and falls back to the Baidu credentials it
  had before, so a deployment works either way.

- **The translation server's address is no longer a field.** Glossy's own service
  is the one that needs nothing set up, and a text box for the address only gave
  people a way to break it — a wrong or stale address left the app unable to reach
  any server at all, which is what the settings file of an upgraded install could
  hold. The address is now part of the build: whichever one the app was compiled
  with wins, a leftover value in an older settings file is ignored, and the window
  keeps showing what is left of today's allowance with its `Check again` button.
  Running your own deployment is a one-line change of `DEFAULT_ENDPOINT` in
  `src-tauri/src/translate/cloud.rs`; a build made without one still honors the
  setting.

### Fixed

- A second launch of Glossy now shows the same starting hint in the corner of the
  screen that a start of its own would, instead of an unstyled Windows dialog. The
  running instance is told through a named event and answers by opening the settings
  window when it is already on screen, or by showing the hint when it is not; the
  dialog is left only for the case where nothing answers at all.

## [1.0.1] - 2026-09-20

### Added

- **Only translate these source languages**, a new list in the Trigger panel of
  the settings window. Add a language and Glossy stops translating selections
  written in the others, which is how a reader who works in one language keeps a
  second one out of the way. The language of a selection is guessed from its
  script and its most frequent short words, and a selection that cannot be told
  apart (a name, a number, a line of code) is translated anyway, so a wrong guess
  never swallows text the user wanted. The list is stored as `sourceLangs`, and an
  empty list means every language, which is what settings files written before
  this release get.

### Fixed

- A double click that selects nothing no longer opens a popup. Clicks on the
  desktop, the taskbar, the start menu and the other surfaces of the shell are
  ignored, because the Ctrl+C Glossy sends afterwards still reaches the program in
  front, and a program that answers it by copying a stale clipboard entry used to
  bring the popup up out of nowhere.
- Selections that carry no language are skipped: a run of digits or punctuation,
  and - when the click happens over a program that copies it - the path of a file.
- A copy of a file in the file explorer no longer counts as a selection. Explorer
  publishes the files it copies as `CF_HDROP` while offering the path as text as
  well; the text is what used to be translated.
- The clipboard is really put back. Some programs fill it from a worker thread,
  so their write landed just after the restore and undid it; the restore now
  watches the clipboard for a moment longer and repairs it.
- Glossy no longer reads its own clipboard writes as a copy. Putting the previous
  content back happens after a selection has been read, and the write that does it
  moves the clipboard along just like a real copy; a second trigger arriving in
  that window used to see the restored text as a fresh selection and translate it.
  Every write Glossy makes is now remembered by its sequence number and ignored
  while waiting for an answer.

## [1.0.0] - 2026-09-20

The first stable release: Glossy now ships with a server of its own, so it can be
installed and used without anyone having to obtain an API key first.

### Added

- A `cloud` translation provider, **Glossy Cloud**. It talks to a server
  deployed from the new `server/` directory instead of to a vendor, so there is
  nothing to fill in: the account behind the server pays for the translation and
  the keys never reach the app. The settings window asks that server how much of
  today's allowance is left and shows the answer under the provider row, with
  **Check again** next to it; a server that cannot be reached says so in the same
  line instead of blocking the rest of the window. The address of the server
  lives in the new `cloudEndpoint` setting, is validated for an `https://` URL,
  and is written to the settings file as plain text because it is not a secret; a
  build made from this source already carries the address of the deployment the
  author runs, so the provider works as soon as it is selected.
- `server/`, a translation proxy that keeps the provider credentials on the
  machine that already holds them and hands each client a daily character
  allowance instead. It runs unchanged on two hosts — a Cloudflare Worker with a
  Durable Object for exact counting, and a Node 18 runtime for Tencent Cloud SCF
  Web 函数 — and carries its own README with the deployment steps for both,
  `npm test` coverage for the quota rules, the signature the providers expect and
  the IP parsing, plus a script that builds the SCF zip. The allowance is checked
  per installation, per caller address and globally, requests are capped per
  minute and per request, and the caller address is read from the end of
  `X-Forwarded-For` minus the two hops a function URL gateway appends, so a
  client cannot spoof its way into a fresh bucket. Baidu answers a burst with its
  per-second throttling code; that answer, and Baidu's own hiccups, are retried
  twice before the app is told anything.
- An install id (`cloudId` in the settings file), generated once so the server
  counts one device's characters rather than one launch's, and rewritten when a
  settings file written before the cloud provider existed has none.

### Changed

- A fresh install starts on the `cloud` provider instead of `google`, so the first
  selection translates without anything being configured or obtained. Existing
  settings keep the provider they were saved with.
- The new Glossy mark is the icon of the application, of the installer and of the
  tray, and it replaces the drawn `G` at the top of the settings window and on the
  start card. `assets/icon.png` is the source; `npm.cmd run icon` rebuilds the
  whole icon set, and the windows show the copy in `src/images/logo.png`.

### Fixed

- The language selectors in the popup card are readable in dark mode. Windows
  paints a `select` **and its native option list** with the background of the
  control, so the translucent fill the card has used left the language names
  sitting on the desktop behind the popup; both the popup selectors and the
  settings window's now use an opaque surface, carry a visible border and grow to
  a size that reads as a combo box.

## [0.3.3] - 2026-09-20

### Added

- The translate box that the settings window already carried now sits at the top
  of the window, above the enable switch, so the app can be used without
  selecting text anywhere: it gained **Paste** (reads the Windows clipboard
  through a new `read_clipboard` command), **Clear**, Ctrl+Enter to translate the
  whole text, and a pasted text is translated as soon as it lands. A new
  **Units** switch on the same heading turns unit conversion on and off from
  there — it is bound to the existing setting below, so either switch moves both
  and the visible card is redrawn with or without its conversions.

## [0.3.2] - 2026-09-20

### Fixed

- The installer now ships `WebView2Loader.dll`, so a freshly installed Glossy
  starts instead of failing at launch with the Windows error "WebView2Loader.dll
  was not found" (`由于找不到 WebView2Loader.dll，无法继续执行代码`). The GNU
  toolchain Glossy is built with links that DLL dynamically, and `tauri-build`
  copies it next to `glossy.exe` for local runs only, so the previous installers
  carried nothing but the executable. `build.rs` now stages the DLL from the
  `webview2-com-sys` build output into `src-tauri/resources/` and
  `bundle.resources` ships it, which installs it flat into the app folder beside
  the executable. `scripts/release.ps1` refuses to stage a release when the
  staged DLL is missing or the generated installer does not include it, so this
  cannot ship silently again.

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

[Unreleased]: https://github.com/SpencerZXWu/Glossy/compare/v1.2.1...HEAD
[1.2.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.2...v1.1.0
[1.0.2]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.3...v1.0.0
[0.3.3]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.3.0
[0.2.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.2.0
[0.1.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.1.0
