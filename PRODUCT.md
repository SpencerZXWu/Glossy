# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

The interface is HTML/CSS/JS. Glossy ships as a Windows desktop application in a
Tauri v2 shell with WebView2, so `web` is the closest true value the schema
offers; there is no iOS, Android or adaptive surface.

## Users

Students in China, the UK, the US and other countries who study either a foreign
language or another subject taught in a language that is not their own. They read
inside a web browser, a PDF reader, a word processor, a chat client or an e-book
app.

Their job: understand the word, sentence or paragraph in front of them without
leaving the document and without losing the place they were reading.

## Product Purpose

Glossy is a lightweight Windows desktop translation popup. The user drags across
text — or double-clicks a word — in any application, and a small floating card
appears under the cursor with the translation. The source language is detected
automatically.

Success: the reader gets the meaning they were missing in a moment and keeps
reading. Nothing has to be pasted anywhere, no page has to be opened, and the
application they are reading in keeps the keyboard focus.

## Positioning

The gesture is available in every Windows application, not only in a browser, and
it never interrupts the reading. One selection returns either a whole word card
(phonetics, parts of speech, definitions, one example, inflections, synonyms, the
sentence the word came from, and a pronunciation button) or a smooth sentence or
paragraph translation, with the original and the translation paired sentence by
sentence when the two do not divide the same way. The popup is a non-activating,
always-on-top window: it never takes the keyboard focus from the application being
read, it is kept inside the work area of the monitor the cursor is on, and it
flips above the cursor when there is no room below. Select-and-read in any
application is the part a browser extension or a dictionary website cannot
truthfully copy.

## Operating Context

- Windows 11 x64. The installer is `Glossy_<version>_x64-setup.exe`; it is not
  code-signed yet, so SmartScreen warns about an unknown publisher. WebView2 is
  required and ships with current Windows builds.
- Tauri v2 shell, WebView2, HTML/CSS/JS renderer in `src/`, Rust in `src-tauri/`,
  and a Cloudflare Worker deployment in `server/`.
- Glossy lives in the notification area. The first launch opens the settings
  window; later launches start quietly and are announced by a card in the bottom
  right that fades away. Closing the window does not quit Glossy — **Quit** in
  the tray menu does.
- The global hotkey (`Ctrl+Alt+C` by default) translates the clipboard instead of
  a selection.
- A selection shorter than the configured minimum (2 characters by default) is
  ignored; a drag that starts or ends on the popup never triggers a translation;
  a selection ending in sentence punctuation is translated as a sentence however
  short it is.
- The interface of the settings window and the popup is maintained in English and
  简体中文 (`src/js/i18n.js` holds exactly those two tables). The documentation is
  trilingual (English, 简体中文, Español), so Español is a documentation language
  today, not an interface language. Adding a third interface language also means
  extending `LANGUAGE_NAMES`, which currently only covers English and Chinese.

## Capabilities and Constraints

- Three translation channels, every one of them usable without an account of the
  user's own: `google` (a public endpoint) and two server-backed entries named
  after the engine they translate with — **Baidu Translate** and **Youdao
  Translate**.
- The two server-backed entries are presented beside Google as ordinary
  translation channels. The deployment that answers them is never named or shown
  in the interface, and no deployment address appears anywhere in the product. A
  user never holds a vendor key: the app sends the text plus an install id, the
  credentials stay on the deployment, and it meters a daily allowance per device,
  per address and in total.
- The provider falls back to another engine when the chosen one cannot answer;
  the order is configurable.
- Word cards are enriched for single words: phonetics and definitions come from
  `api.dictionaryapi.dev`, and the card adds inflections, synonyms, the sentence
  the word was selected from with a translation of that sentence, and pronunciation
  buttons for each side.
- Unit and currency conversion is added when a translation still measures something
  in a unit the target language does not use (`12 ft ≈ 3.66 m`), including the rate.
  Currency rates are fetched live and cached for six hours; every other unit is
  built into the app. It can be switched off.
- Settings include: interface language, the master switch for selection
  translation, start with Windows, the drag and double-click gestures, clipboard
  restore, showing the original, unit conversion, the reading options (source
  sentence, sentence pairing, compact card), the fallback order, and the hotkey.
- Open decision: macOS and Linux support is an intention, not a commitment.
  Nothing has been designed or built for either.

## Brand Commitments

- The name **Glossy**; the icon at `assets/icon.png` and the in-app logo at
  `src/images/logo.png`.
- MIT licensed, © 2026 Spencer Wu.
- Documentation is trilingual (English, 简体中文, Español) and that coverage is
  expected to continue.
- There is no account and no login, and no key for the user to fill in.
- The Windows installer is published unsigned, together with a checksum.

## Evidence on Hand

- `README.md` — the trilingual user guide: install, usage, settings tables,
  providers and quotas, troubleshooting, how the selection is captured, layout,
  development and the manual regression checklist.
- `ROADMAP.md` and `CHANGELOG.md` — delivered work and planned work, per version.
- `release/v1.0.2/`, `release/v1.1.0/`, `release/v1.1.1/` and `release/v1.1.2/` — the
  shipped release notes and checksums.
- The automated suites: 177 frontend tests and 189 Rust tests at v1.1.2.
- Absent, and not to be invented: there are no screenshots or recordings of the
  product, no testimonials, no user research and no benchmarks.

## Product Principles

1. The gesture lives where the reading happens — any Windows application, never
   only the browser.
2. Reading is never interrupted: the popup does not take the keyboard focus, it
   appears under the cursor, and a click outside dismisses it.
3. Answer the whole question the reader has about the selection — a word deserves
   a card, a paragraph deserves a smooth translation.
4. Nothing to set up: no account, no login, no key.
5. A spare, quick card beats a rich but slow one, and the extras stay optional.

## Accessibility & Inclusion

No product-specific accessibility standard has been agreed. The roadmap still
carries an unfinished "Icons, motion and accessibility" item, so this is an open
area rather than a settled requirement. What the product already owes its
audience: trilingual interface and documentation, correct CJK and Latin
typography side by side in the same card, and a reading surface that does not
depend on a second, focused window being available.
