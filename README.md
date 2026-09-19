# Glossy

[![CI](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml/badge.svg)](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml)

A lightweight desktop translation popup for Windows. Select text in any
application with the mouse and Glossy shows a small floating card with the
translation under the cursor.

- **Word or short phrase** → phonetic symbols, part of speech, definitions and one example.
- **Sentence or paragraph** → a smooth translation into your target language.
- **A measurement or an amount** written in a unit your target language does not
  use → the converted value and the conversion rate, right in the card
  (`12 ft ≈ 3.66 m`). Currency rates are looked up live; every other unit is
  built into the app. Can be switched off in the settings.
- The source language is detected automatically.
- A selection that ends with sentence punctuation (`.`, `?`, `!`, `。` …) is
  always translated as a sentence, however short it is.

## Install

Download the latest installer from
[Releases](https://github.com/SpencerZXWu/Glossy/releases/latest)
(`Glossy_<version>_x64-setup.exe`) and run it. Windows 11 x64; WebView2 is
required and ships with current Windows builds.

The installer is not code-signed yet, so SmartScreen warns about an unknown
publisher. To build from source instead, see [Development](#development).

## Using it

1. Start Glossy. The settings window opens on the very first launch only —
   later launches start quietly in the notification area, and Glossy listens for
   selections from the moment it starts.
2. Closing that window does not quit Glossy — it keeps watching for selections
   in the background. Click the Glossy icon in the notification area (or pick
   **Open Glossy** from its menu) to bring the window back, and use **Quit** in
   the same menu to stop Glossy. Starting Glossy while it is already running only
   shows a short notice. A silent start is announced by a small card in the
   bottom right corner; it fades away after a few seconds, and clicking it opens
   the settings window. Windows 11 keeps new notification area icons in the
   overflow menu (the `^` next to the clock) — drag the icon onto the taskbar, or
   turn it on under **Settings → Personalization → Taskbar → Other system tray
   icons**, to keep it visible.
3. In any application, **drag across text** (or **double click a word**) to select it.
4. The popup appears below the cursor. Drag its header to move it, use the buttons
   to copy the result or to close it, or click anywhere else to dismiss it.
   The row under the header shows the language pair: hover it and pick either side
   from the dropdowns to translate again with that language, or press the `⇄`
   button to translate the result back into the language it came from. The pair
   resets to *detect the source and use the configured target* for every new
   selection.
5. Selections shorter than the configured minimum (2 characters by default) are
   ignored, and a drag that starts or ends on the popup itself never triggers a
   translation.

Press the global hotkey (**Ctrl+Alt+C** by default) to translate the clipboard
content instead of selecting anything.

The popup is a non-activating always-on-top window: it never steals the keyboard
focus from the application you are reading in. It is kept inside the work area of
the monitor the cursor is on, and flips above the cursor when there is no room below.

### Settings

| Option | Meaning |
| --- | --- |
| Interface language | `Follow Windows`, `简体中文` or `English`. Switches the settings window and the popup immediately. |
| Enable selection translation | Master switch. Turning it off pauses the global selection capture immediately. |
| Start Glossy with Windows | Adds a `--autostart` entry to `HKCU\...\Run`, so Glossy is already waiting in the notification area after a login. Started that way it does not show the "Glossy is running" card. |
| Translate when the mouse drags across text | Enables the drag gesture. |
| Translate a word on double click | Enables the double-click gesture. |
| Put the clipboard back after reading a selection | Restores your previous clipboard content after Glossy copied the selection. |
| Show the original text in the popup | Hides the source line in the card when off. |
| Convert units and currency | When the original measures something in a unit your target language does not use — feet, pounds, °F, a foreign amount — the card adds the converted value and the rate underneath (`12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`). Length, mass, volume, speed, area and temperature convert inside their category, and the target unit is the one a person would write; the target currency follows the target language (`zh-CN` → CNY, `en` → USD). Currency rates come from `open.er-api.com` (ECB as a fallback) and are cached for six hours; everything else is built into the app. Off means no conversion and no rate request. |
| Shortest selection to translate | Character count below which a selection is ignored (`1`–`40`, default `2`). |
| Global hotkey | Accelerator that translates the clipboard content, e.g. `Ctrl+Alt+C`. Clear the field to switch it off. The line under the field shows the registered combination or why Windows refused it. |
| Never translate in these programs | A list of process names (`idea64.exe`, `mstsc`) in which selection capture is skipped. Add one by typing it (the `.exe` suffix is optional — the `Add` button normalises it), by choosing it from the dropdown of currently running programs, or by pressing `Pick with the mouse` and clicking the window to ignore. Each entry has an `×` to remove it; duplicates are dropped case-insensitively. |
| Colours | `system` follows the Windows light/dark preference; `light` and `dark` force one scheme in both windows. |
| Text size | Multiplier for every text size in the popup (`90 %`–`150 %`). |
| Width | Popup card width (`300`–`520` CSS px). |
| Opacity | How see-through the popup card is (`100 %` solid down to `50 %`). The card fades while the text stays readable on top of whatever is behind it. |
| Close by itself | Seconds before the popup hides on its own; `Never` keeps it open until dismissed. |
| Close the popup right after the translation is copied | Hides the card once the copy button was used. |
| Target language | Language the result is translated into. |
| History | How many finished translations to remember (`Off` to `The last 500`, default 50). The list below the selector keeps the original, the translation, the provider and the time; `Search` filters both texts, clicking an entry shows it in the floating card again (no second provider call), and each entry has a copy and a remove button. `Forget everything` empties the list. The file lives in `%APPDATA%\com.glossy.translator\history.json`. |
| Settings file | `Export…` writes `Documents\glossy-settings.json`; `Import…` reads a file you pick back into the app. Tick `Include my API keys in the exported file` to carry the keys as well — Glossy asks once more before it writes them in plain text. An import validates through `sanitized()` and protects the keys it brings with DPAPI on the way to disk. |
| Updates | `Check for a new version when Glossy starts` asks GitHub Releases on every start (off by default). `Check now` looks immediately and says which version is waiting, and `Download and restart` installs it. A build without an update signing key — which is every build until the release key pair exists — hides the buttons and says so. |
| Translation provider | `google` (free, no key), `baidu` (free monthly quota, APP ID + key), `zhipu` (free tier, API key), `deepl` or `openai` (API key). |
| APP ID / API key | Shown only for the providers that need them: `baidu` asks for both fields, `zhipu`, `deepl` and `openai` for the key alone, and the free `google` provider hides both. The values are remembered per provider, so switching to a provider you configured earlier fills its fields back in. |

The hotkey accepts `Ctrl`/`Control`, `Alt`, `Shift`, `Win`/`Meta` plus one key:
a letter, a digit, `F1`–`F24`, `Space`, `Enter`, `Tab`, `Esc`, `Backspace`,
`Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown` or an arrow key. At least
one modifier is required. When Windows rejects the combination — usually because
another program already owns it — the reason is shown under the field and the
hotkey stays inactive until the setting is corrected.

If the clipboard holds no text at all, the hotkey falls back to copying the
current selection, so "select text, press the hotkey" works as well.

### Providers

| Provider | Cost | Notes |
| --- | --- | --- |
| `google` | free, no key | Public `translate.googleapis.com` endpoint. Blocked on some networks, including much of mainland China. Always queried with the `dict-chrome-ex` client id; the throttled `gtx` id is only used as a fallback. |
| `baidu` | free monthly quota, APP ID + key | Baidu 翻译开放平台 (`fanyi-api.baidu.com/api/trans/vip/translate`). Reachable from mainland China with a monthly free quota of 50,000 characters, raising to 1,000,000 after the free personal 个人认证. Needs both the **APP ID** and the **密钥** from <https://fanyi-api.baidu.com>. Passes `from=auto`, so the source language is detected. |
| `zhipu` | free tier, API key | Zhipu `glm-4.7-flash` chat model. Reachable from mainland China and returns translation, phonetics, definitions and an example in a single call. Key from <https://open.bigmodel.cn>. ⚠️ Zhipu's user agreement licenses the non-paid models for **non-commercial personal study only** — see below before shipping Glossy. |
| `deepl` | API key | Keys ending in `:fx` use the free endpoint. |
| `openai` | API key | `gpt-4o-mini`. |

Phonetics and definitions for single words come from `api.dictionaryapi.dev`, which is free,
needs no key and is reachable from mainland China. The Google endpoint is only asked for
details the selected provider did not return.

### Free quotas and commercial use

The free tiers differ in what they permit:

- **Zhipu** — 用户协议 §非付费功能 licenses the free models for *非商业的、个人研究学习* use
  only. Fine for personal use; not fine for a published or paid product.
- **ModelScope API-Inference** — explicitly 非商业化, 非盈利.
- **Aliyun 机器翻译** — the monthly free quota is explicitly 仅适用客户试用场景.
- **Tencent 腾讯云 TMT** — the free 5M characters/month quota is still advertised, but the
  service no longer offers text translation: as of the 2026-03 and 2026-07 releases the
  `TextTranslate`, `TextTranslateBatch`, `ImageTranslate`, `LanguageDetect` and
  `SpeechTranslate` actions were removed and the API 概览 lists only `ImageTranslateLLM`.
  The 计费概述 page is stale, so do not plan on it.
- **Baidu 翻译开放平台** — 50k characters/month unverified, 1M personal-verified, 2M
  business-verified. Its terms are silent on commercial use, but 服务协议 forbids a *client
  program* from caching Baidu translation data and forbids resale. Glossy does not cache
  anything, so this does not bite; a fork that adds a cache would need to revisit it.
  Quotas are QPS-limited to 1 / 10 / 100.
- **Volcengine 火山引擎** — 2M characters/month, but onboarding requires a sales contract.
- **NiuTrans 小牛翻译** — 200k characters/day after registering; commercial terms unpublished.
- **SiliconFlow** — `tencent/Hunyuan-MT-7B` is free, but the platform terms are silent about
  free models and are restricted to internal business purposes.
- **Fully offline** — `Opus-MT` / `Argos Translate` weights are CC-BY-4.0/Apache-2.0 (commercial
  OK, ~83 MB int8, ~300 MB RAM). `NLLB-200` is CC-BY-NC-4.0 and must **not** be shipped.

Settings are stored as JSON in `%APPDATA%\com.glossy.translator\settings.json`.
The `firstRun` flag in that file records that the welcome window was already
shown; removing it (or the whole file) brings the window back on the next start.
Translation credentials live in a `credentials` map keyed by provider, and existing
single-provider `apiKey`/`appId` values are migrated into that map on first launch.
Every value in that map is encrypted with Windows DPAPI (`CryptProtectData`,
current-user scope) before the file is written, so it holds
`"apiKey": "dpapi:AQAAANCM…"` rather than the key itself and only the Windows login
that entered it can read it back. A file written by an earlier version is protected
the first time this build starts; a value that belongs to another login or computer
cannot be unlocked and is dropped, so the key has to be entered again.
The ignored-program list is stored as an `ignoredApps` array; a legacy
comma-separated string is accepted and split on load.

### Translate inside the app

The settings window contains the same translation feature as the popup: a text
box for typing or pasting text. Select a word, a phrase or a sentence in that box
— or press **Translate** to use the whole text — and the result appears right
below, in a card identical to the popup one: the source/target language bar with
its swap button, the copy button and the same word/sentence rendering. The
language bar follows the popup rules, so the source starts on *Detect language*,
the target follows the configured language, and a new selection resets the pair
(the overrides are never saved). Press **Show in floating popup** to open the
real popup with the current selection (or with the whole text when nothing is
selected) — this exercises the popup without the global hook.

### Troubleshooting

- **"Google Translate is rate limiting requests right now"** — Google answered `429`, which
  happens to desktop HTTP clients on the public endpoint even though a browser or `curl` still
  works. Glossy first retries with a second client id; if the message stays, wait a minute or
  switch the provider to `baidu`, `zhipu`, `deepl` or `openai`.
- **"Could not reach Google Translate"** — the free `google` provider calls
  `translate.googleapis.com`, which is blocked on some networks (including much of mainland
  China). The card offers a retry button; if it keeps failing, switch the provider to `baidu`
  (free monthly quota, APP ID and 密钥 from fanyi-api.baidu.com), `zhipu` (free tier, key from
  open.bigmodel.cn), `deepl` or `openai`.
- **"Baidu rejected the APP ID" / "Baidu rejected the signature"** — the two halves of the
  credential pair were swapped or mistyped. `baidu` needs the APP ID in the first box and the
  密钥 in the second; the key is never sent to Baidu, it only signs the request.
- **"Baidu rejected this computer's IP address"** — the app's IP whitelist in the Baidu
  console is filled in. Either clear it or add the address Glossy dials from.
- **The popup shows "Paused"** although translations are enabled — the master switch
  in the settings window is off, or the settings file could not be read at start-up.

## How the selection is captured

Glossy installs a `WH_MOUSE_LL` hook and watches for left button press/release
pairs. The hook callback only records coordinates; a worker thread decides whether
the gesture was a drag or a double click, copies the selection with `Ctrl+C`
(sending `Ctrl+Insert` when the foreground window runs elevated), restores the
clipboard if requested, and finally tells the popup window what to show. The
captured text only ever goes to the translation provider you selected.

## Development

### What you need

| Tool | Version | Notes |
| --- | --- | --- |
| Node.js | 18 or newer | only for the Tauri CLI; the frontend has no bundler |
| Rust | stable, `1.77` or newer | the GNU target `x86_64-pc-windows-gnu` — see below |
| `rustfmt` + `clippy` | same toolchain | `rustup component add rustfmt clippy` |
| MinGW-w64 | 8.1 or newer | the linker for the GNU target; on `PATH` as `gcc.exe` |

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu --component rustfmt --component clippy
rustup default stable-x86_64-pc-windows-gnu
npm.cmd install          # npm.ps1 is blocked by the default execution policy
npm.cmd run tauri dev    # dev build with hot reload of src/
npm.cmd run tauri build  # release build (installer + .exe)
```

`npm.cmd run icon` regenerates `src-tauri/icons` from `assets/`.

`npm.cmd run tauri build` writes the installer to
`src-tauri\target\release\bundle\nsis\Glossy_<version>_x64-setup.exe` and the
standalone binary to `src-tauri\target\release\Glossy.exe`. The NSIS bundler needs
a network connection the first time it runs, to download its plug-ins.

`scripts\release.ps1` wraps the release build: it stages the installer in
`release\v<version>\` together with a checksum and the text for the release
description. Run it as `powershell -ExecutionPolicy Bypass -File
scripts\release.ps1` — scripts are blocked by the default execution policy, the same
reason `npm.cmd` is used above. See [release/README.md](./release/README.md).

### Checks

```powershell
powershell -ExecutionPolicy Bypass -File scripts\version.ps1 -Check   # all version numbers agree
node --test                                                          # 81 frontend tests
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

`npm test` runs the same `node --test` command. The frontend tests load
`src/js/i18n.js` and `src/js/render.js` into a minimal DOM (see `tests/helpers/`)
and cover the dictionaries, the placeholder substitution and the card renderer.
Run the plain directory form on Node 24/Windows — `node --test tests` and
`node --test .` do not resolve the test files there.

The version lives in six places (see `scripts\version.ps1`), so never edit them by
hand: `tauri.conf.json` is authoritative, `scripts\version.ps1 -Set 0.1.2` writes it
everywhere else, and `scripts\version.ps1 -Get` prints it for scripts such as
`release.ps1`, which refuses to build when the numbers disagree. The same check runs
in CI, so a forgotten bump fails the build.

[`.github/workflows/ci.yml`](./.github/workflows/ci.yml) runs all four checks plus
the frontend test suite on `windows-latest` for every push and pull request.

### Updates

Glossy asks
`https://github.com/SpencerZXWu/Glossy/releases/latest/download/latest.json` for a
newer release. An update is only accepted if it carries a signature made with the
release key:

```powershell
cargo tauri signer generate -w $env:USERPROFILE\.tauri\glossy.key   # once
```

Put the printed public key into `plugins.updater.pubkey` in
`src-tauri\tauri.conf.json` (the placeholder there is what makes the settings
window say the build cannot update itself) and export the private one before
building a release:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $env:USERPROFILE\.tauri\glossy.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<the password you chose>'
```

`bundle.createUpdaterArtifacts` in `tauri.conf.json` stays off until that secret
exists: without it the bundler fails, with it the build also writes the signed
installer and the `latest.json` an update is read from. Keep the private key out of
the repository — it is the only thing that lets a release be replaced.

### Manual regression checklist

Run this before tagging a release. Every item was a real bug at some point, and
none of them is covered by the automated tests.

| # | What to do | What has to happen |
| --- | --- | --- |
| 1 | Start Glossy on a clean profile | The settings window opens, nothing is preselected by accident, and the status line says the capture is running |
| 2 | Close the settings window, then start Glossy again | No second window and no second mouse hook: the notification says Glossy is already running, and the tray icon opens the window again |
| 3 | Tick "Start Glossy with Windows", reboot, then sign in | No console window flashes, no "running" card appears, the tray icon is there |
| 4 | Untick it, reboot | Glossy does not start (`reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Glossy` finds nothing) |
| 5 | Select a word in Notepad, a browser, Word and a terminal | The popup appears under the cursor each time, with phonetics and definitions for the word |
| 6 | Select a sentence in each of the same programs | The popup shows the translation only, without phonetics or meanings |
| 7 | Drag the popup to each monitor, and to each screen edge | The card stays inside the work area on every monitor and at 100 %, 125 % and 150 % scaling |
| 8 | Pin the popup, click elsewhere on the desktop, wait past the auto-close timeout | The card stays open until the pin is released or the × is used |
| 9 | Click outside an unpinned card | It closes as soon as the click lands outside |
| 10 | Switch Windows between light and dark, then force each scheme in the settings | Both windows follow the choice, with readable text and borders in both |
| 11 | Translate with each of the five providers, including one with a bad key | A result for the good ones; a readable error with a retry button for the bad one |
| 12 | Add a running program to the ignore list, translate inside it, then remove it | Nothing pops up while it is listed, and the popup is back once it is removed |
| 13 | Press the global hotkey with text on the clipboard, then with an empty clipboard and a selection | The translation opens next to the cursor in the first case, the current selection is used in the second |
| 14 | Export without keys, export with keys, then import each file | Plain export has no `credentials` block; the keyed export asks for confirmation first; an import restores every setting and the keys work |
| 15 | Set the history to `The last 50`, translate 60 texts, then set it to `Off` | The list keeps 50, search filters them, clicking one reopens it in the card, and `Off` empties the file |
| 16 | Change the opacities, font size and width, restart | The popup keeps the chosen values |
| 17 | Open the Updates section | This build, without a signing key, hides the check buttons and says the build cannot update itself; after a key pair exists, `Check now` reports either the running version or the one that is waiting |

### Toolchain notes (Windows, GNU toolchain)

The project builds with the GNU Rust target (`x86_64-pc-windows-gnu`), which needs
no Visual Studio installation, but there are two quirks:

- The `windows` crate grows the import table past what old MinGW binutils accept.
  `src-tauri/Cargo.toml` therefore declares `[lib] crate-type = ["rlib"]` and the
  application crate is linked as a binary only.
- `cargo test` needs the generated resource archive (icon, version info and the
  common-controls v6 manifest) in the test harness as well, otherwise the harness
  fails to start with `0xC0000139` before `main`. `src-tauri/build.rs` adds a plain
  `cargo:rustc-link-arg` for `libresource.a` to reach every test target. Build
  scripts cannot tell binary and test targets apart, so binary targets receive the
  archive twice and GNU ld prints `.rsrc merge failure: multiple non-default
  manifests` warnings; the resulting `.exe` still carries the manifest and runs.

```powershell
cd src-tauri
cargo test   # 128 tests; see "Checks" above for lint and format runs
```

## Layout

```
src/                     frontend (plain HTML/CSS/JS, no bundler)
  index.html             settings window + in-app translate card
  popup.html             floating card
  notice.html            "already running in the background" card
  js/bridge.js           Tauri IPC helpers used by both windows
  js/i18n.js             English/Chinese dictionaries and DOM translation
  js/render.js           shared card renderer (word, sentence, loading, error)
  js/theme.js            resolves system/light/dark for every window
  js/app.js              settings window logic
  js/popup.js            popup window logic
  js/notice.js           start card logic
  styles/tokens.css      the only place a colour, radius, shadow or duration is written
tests/                   node --test suite for i18n.js and render.js
src-tauri/src/
  main.rs  lib.rs        window setup, Tauri commands
  selection.rs           global mouse hook, gesture tracking, capture
  hotkey.rs              global hotkey registration and parsing
  classify.rs            word/phrase vs. sentence detection
  units/                 unit and currency conversion for the card
  translate/             google, baidu, zhipu, deepl and openai providers, word dictionary
  popup.rs               placement/clamping geometry
  surface.rs             Mica backdrop and title bar colour
  history.rs             the translation store behind the History panel
  autostart.rs           the login item and the --autostart marker
  updater.rs             the release feed behind the Updates section
  secrets.rs             DPAPI protection for the stored API keys
  notice.rs              start card placement and lifetime
  tray.rs                notification area icon: open the window, quit
  instance.rs            named-mutex guard against a second Glossy
  console.rs             borrows the console of the terminal that started Glossy
  platform.rs            DPI aware cursor, work area, visible windows, click-through helpers
  clipboard.rs  settings.rs  state.rs  input.rs
scripts/
  version.ps1            the version number, in one place
  release.ps1            release build + staging for a GitHub release
.github/workflows/
  ci.yml                 format, lint, test and version check on every push
release/
  v<version>/            installer, RELEASE_NOTES.md and checksum of a release
```

## Roadmap

[ROADMAP.md](./ROADMAP.md) holds the planned releases (`v0.1.1` through `v1.0.0`) with
their acceptance criteria, the versioning policy, the known risks and the release
process. Each release maps to a GitHub milestone of the same name.

[CHANGELOG.md](./CHANGELOG.md) lists what shipped in every released version.

## License

[MIT](./LICENSE) © 2026 Spencer Wu
