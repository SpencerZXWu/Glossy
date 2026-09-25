<p align="center">
  <img src="assets/icon.png" width="112" height="112" alt="Glossy">
</p>

[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

# Glossy

[![CI](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml/badge.svg)](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml)

A lightweight desktop translation popup for Windows. Select text in any
application with the mouse and Glossy shows a small floating card with the
translation under the cursor.

- **Word or short phrase** → phonetic symbols, part of speech, definitions, one
  example, the inflections, the synonyms, the sentence it came from and a
  pronunciation button.
- **Sentence or paragraph** → a smooth translation into your target language,
  sentence by sentence when the two do not divide the same way.
- **A measurement or an amount** that the translation still writes in a unit
  your target language does not use → the converted value and the conversion
  rate, right in the card (`12 ft ≈ 3.66 m`). Currency rates are looked up live;
  every other unit is built into the app. Can be switched off in the settings.
- The source language is detected automatically.
- A selection that ends with sentence punctuation (`.`, `?`, `!`, `。` …) is
  always translated as a sentence, however short it is.

## Install

Download the latest release from
[Releases](https://github.com/SpencerZXWu/Glossy/releases/latest) and pick the
one that suits you:

| Download | What it is |
| --- | --- |
| `Glossy_<version>_x64-setup.exe` | The installer, and the one to use if you want the automatic update to work. Registers Glossy in **Installed apps**, so it can be uninstalled the usual way |
| `Glossy_<version>_x64_en-US.msi` | The same application as an MSI package, for anyone deploying it with a policy or an installer script |
| `Glossy_<version>_x64_portable.zip` | Nothing to install: unpack it anywhere and run `glossy.exe` from that folder. Keep `WebView2Loader.dll` beside it, and note that start with Windows remembers the folder you unpacked to, so leave it where it is |

Windows 11 x64; WebView2 is required and ships with current Windows builds. The
`SHA256SUMS.txt` in the same release lists the hashes of all three.

None of them is code-signed yet, so SmartScreen warns about an unknown publisher.
To build from source instead, see [Development](#development).

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
4. Glossy shows a small Glossy icon below the selection. Nothing is translated —
   and nothing is charged against the day's allowance — until you **click that
   icon**, so a word you merely swiped over costs nothing. Clicking anywhere else
   makes the icon disappear.
5. The card appears below the selection. Drag its header to move it, use the
   buttons to copy the result or to open the settings, or click anywhere else to
   dismiss it.
   The row under the header shows the language pair: hover it and pick either side
   from the dropdowns to translate again with that language, or press the `⇄`
   button to translate the result back into the language it came from. The pair
   resets to *detect the source and use the configured target* for every new
   selection; a target you pick by hand is remembered and becomes that configured
   target, so the next selection already starts from it, while the source you
   pick is good for the card on screen only.
6. Selections shorter than the configured minimum (2 characters by default) are
   ignored, and a drag that starts or ends on the popup itself never triggers a
   translation.

A word card shows the phonetic symbols, the parts of speech with their definitions,
one simple example, and — in the same card — the inflections of the word, the words
that mean roughly the same, the sentence it was selected from with a translation of
that sentence, and a pronunciation button for each side of the card. A sentence or a
paragraph is shown as the translation itself, with the original above it (capped at
four lines) and, for a long paragraph, a sentence-by-sentence view underneath.

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
| Show the original text in the popup | Hides the source line in the card when off. A long selection is capped at four lines there and scrolls inside its block, so the translation keeps the room. |
| Convert units and currency | When the translation still measures something in a unit your target language does not use — feet, pounds, °F, a foreign amount — the card adds the converted value and the rate underneath (`12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`). The numbers are read from the translation, because the translator is what decides whether a symbol is a unit at all and always writes it in a language the tables know; if the translation holds none, the original is read instead. Length, mass, volume, speed, area and temperature convert inside their category, and the target unit is the one a person would write; the target currency follows the target language (`zh-CN` → CNY, `en` → USD). Currency rates come through Glossy's own server, which reads `open.er-api.com` and falls back to ECB (`api.frankfurter.app`); the app asks those two directly only when the server cannot answer, and either way a rate is cached for six hours. Everything else is built into the app. Off means no conversion and no rate request. |
| Show the sentence a word was selected from | The card adds the sentence the selection was taken from, next to a translation of that sentence. Off means the word is translated on its own. |
| Pair the original and the translation sentence by sentence | A paragraph whose translation divides differently is listed one aligned row at a time instead of as a block. A translation that comes back as a single row is left as the plain paragraph. |
| Draw a compact card, without the extras | Keeps the translation, the phonetic symbols and the meanings, and leaves out the example, the inflections, the synonyms, the context sentence, the sentence-by-sentence view and the conversions. |
| Speaking rate | How fast the pronunciation buttons read a text out: `Slow`, `Normal`, `Fast` or `Very fast` (Windows SAPI). The voices are the ones Windows already has, and the language of the text picks the voice. |
| Shortest selection to translate | Character count below which a selection is ignored (`1`–`40`, default `2`). |
| Global hotkey | Accelerator that translates the clipboard content, e.g. `Ctrl+Alt+C`. Clear the field to switch it off. The line under the field shows the registered combination or why Windows refused it. |
| Never translate in these programs | A list of process names (`idea64.exe`, `mstsc`) in which selection capture is skipped. Add one by typing it (the `.exe` suffix is optional — the `Add` button normalises it), by choosing it from the dropdown of currently running programs, or by pressing `Pick with the mouse` and clicking the window to ignore. Each entry has an `×` to remove it; duplicates are dropped case-insensitively. |
| Only translate these source languages | A list of languages, e.g. `English` and `日本語`. Empty means every language triggers a translation. A selection whose language cannot be pinned down — mixed scripts, digits, a word half a dozen languages share — is always let through, so a wrong guess never swallows a selection. The list is stored as a `sourceLangs` array. |
| Colours | `system` follows the Windows light/dark preference; `light` and `dark` force one scheme in both windows. |
| Text size | Multiplier for every text size in the popup (`90 %`–`150 %`). |
| Width | Popup card width (`300`–`520` CSS px). |
| Opacity | How see-through the popup card is (`100 %` solid down to `50 %`). The card fades while the text stays readable on top of whatever is behind it. |
| Close by itself | Seconds before the popup hides on its own; `Never` keeps it open until dismissed. |
| Close the popup right after the translation is copied | Hides the card once the copy button was used. |
| Target language | Language the result is translated into. |
| History | How many finished translations to remember (`Off` to `The last 500`, default 50). The list below the selector keeps the original, the translation, the provider and the time; `Search` filters both texts, clicking an entry shows it in the floating card again (no second provider call), and each entry has a copy and a remove button. `Forget everything` empties the list. The file lives in `%APPDATA%\com.glossy.translator\history.json`. |
| Settings file | `Export…` writes `Documents\glossy-settings.json`; `Import…` reads a file you pick back into the app. The file holds the choices and nothing secret — there is no key field left anywhere, so an export never asks about one. An import validates through `sanitized()`. |
| Updates | `Check for a new version when Glossy starts` asks GitHub Releases on every start (off by default). `Check now` looks immediately and says which version is waiting, and `Download and restart` installs it. A build without an update signing key — which is every build until the release key pair exists — hides the buttons and says so. |
| Translation service | Which service translates: `cloud-baidu` (Glossy's own server, translating with Baidu — nothing to fill in), `cloud-youdao` (the same server, translating with Youdao — nothing to fill in either), or `google` (the free public endpoint, no key). A new install starts on `cloud-baidu`. Stored as `channel` + `cloudProvider` + `cloudVendor` / `provider`, see below. |
| Ask another service when the chosen one fails | The service picked above is always tried first and cannot be moved; the entries below it are asked in order when it fails, rate-limits or answers with nothing, and can be reordered with the arrows. Turning the switch off means a failure is shown as a failure. The card names the service that answered when it was not the one asked for. |
| Allowance line (`cloud-baidu`, `cloud-youdao`) | Shown only for the two server-backed entries: what is left of today's allowance — or the reason the server could not be reached — with a `Check again` button next to it. The address itself is part of the build rather than a field, so nobody can break the one service that needs nothing set up; see [`server/`](./server/README.md) to run a deployment of your own. |

The hotkey accepts `Ctrl`/`Control`, `Alt`, `Shift`, `Win`/`Meta` plus one key:
a letter, a digit, `F1`–`F24`, `Space`, `Enter`, `Tab`, `Esc`, `Backspace`,
`Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown` or an arrow key. At least
one modifier is required. When Windows rejects the combination — usually because
another program already owns it — the reason is shown under the field and the
hotkey stays inactive until the setting is corrected.

If the clipboard holds no text at all, the hotkey falls back to copying the
current selection, so "select text, press the hotkey" works as well.

### Providers

Every entry in the dropdown works without an account of your own: `google` asks
a public endpoint, and the two server-backed
entries are answered by a Glossy deployment that holds the vendor account, so no
key ever reaches the app. They only name the engine the server should translate
with, and it falls back to another one when that engine cannot answer.

| Provider | Cost | Notes |
| --- | --- | --- |
| `cloud-baidu` | nothing to fill in | **Baidu Translate**: a server deployed from [`server/`](./server/README.md) does the translating with the project's own account, named `vendor: "baidu"` on the wire. The app only sends the text plus an install id, the credentials stay on the server, and it works from mainland China. The daily allowance is counted per device, per address and in total, and the settings window shows what is left of it; running your own deployment is a one-line change of the address inside the build. |
| `cloud-youdao` | nothing to fill in | **Youdao Translate** — the same server, asked to translate with 有道智云 (`vendor: "youdao"` on the wire). Nothing to fill in either, and the same fallback when Youdao cannot answer. |
| `google` | free, no key | Public `translate.googleapis.com` endpoint. Blocked on some networks, including much of mainland China. Always queried with the `dict-chrome-ex` client id; the throttled `gtx` id is only used as a fallback. |

Every engine translates a different set of languages: a standard Baidu account
refuses eight of the 31 Glossy offers (`uk`, `tr`, `hi`, `id`, `ms`, `he`, `no`,
`sk`), while Youdao and Google take all of them. Both language bars list only the
languages of the engine the card is using, so one it would refuse is never offered
and never sent.

Phonetics and definitions for single words come from `api.dictionaryapi.dev`, which is free,
needs no key and is reachable from mainland China. The Google endpoint is only asked for
details the selected provider did not return.

### Free quotas and commercial use

The free tiers differ in what they permit. What they forbid is handing the raw
quota — a key, an interface — to other people or reselling it; keeping the
credential on a server of our own and letting the app call that server is a use
of the quota by the account holder, which is why the two server-backed entries
relay it that way:

- **`cloud-baidu` / `cloud-youdao`** — they translate through an LLM account the
  project pays for, and
  fall back to Baidu credentials when no such account is configured, so what they
  make are paid calls and the terms above do not apply to them. No user ever sees
  or holds a vendor key: the app sends the text plus an install id, and the server
  meters a daily allowance per device and address.
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
The file opens with `formatVersion`, which says which shape it is written in; from
`1.3.0` onwards the keys are only ever added and the IPC command names do not move, so
a file written by an earlier build keeps loading. A file this build cannot read — one
that is not a JSON object, or one written by a newer format version — is copied to
`settings.backup-<unix seconds>.json` next to it and the app starts from the defaults,
rather than being overwritten.
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
Which service translates is stored in the older two-part shape: `channel`
(`cloud` or `api`) picks between the server-backed entries and the free Google
endpoint, and `provider` names the service inside it — `baidu` for the
server-backed entries (a value the channel ignores, kept because the field is
part of the file format) or `google`. `cloudProvider`
(`builtin` or `local`, the second of which is read as the built-in engine now that
the dropdown no longer offers it)
and `cloudVendor` (`baidu` or `youdao`) are what the server-backed entries
ask it for. A fresh install starts on `baidu`. The dropdown maps to that shape on
the way in and out, so a file written
by an older build keeps working: one without a `channel` is read as the API
channel it already was, a file that has none is written with `cloud`, a
`cloudVendor` that is not `youdao` reads as Baidu, and a `provider` this build no
longer offers — Baidu with one's own key, Zhipu, DeepL, OpenAI — reads as the
built-in Baidu entry and is replaced the next time the file is written.

### Translate inside the app

The top of the settings window holds the same translation feature as the popup: a
text box for typing or pasting text. Select a word, a phrase or a sentence in that
box — or press **Translate** (Ctrl+Enter) to use the whole text — and the result
appears right below, in a card identical to the popup one: the source/target
language bar with its swap button, the copy button and the same word/sentence
rendering. A pasted text is translated as soon as it lands, whether it arrives
through **Paste** or through Ctrl+V; **Clear** empties the box and takes the card
away. The switch on the heading turns unit conversion on and off for the whole
app: it is the same setting as the one in **Units** further down, and it is the
one place where the conversions of a card can be flipped on the spot. The
language bar follows the popup rules, so the source starts on *Detect language*,
the target follows the configured language, and a new selection resets the pair
(the source override is never saved; the target you pick is, and becomes the
configured language). Press **Show in floating popup** to open the
real popup with the current selection (or with the whole text when nothing is
selected) — this exercises the popup without the global hook.

### Troubleshooting

- **"Google Translate is rate limiting requests right now"** — Google answered `429`, which
  happens to desktop HTTP clients on the public endpoint even though a browser or `curl` still
  works. Glossy first retries with a second client id; if the message stays, wait a minute or
  switch the service to one of the other two entries in the dropdown.
- **"Could not reach Google Translate"** — the free `google` provider calls
  `translate.googleapis.com`, which is blocked on some networks (including much of mainland
  China). The card offers a retry button; if it keeps failing, switch to `cloud-baidu` or
  `cloud-youdao` (nothing to fill in).
- **"The free cloud translation quota for today is used up"** — the two server-backed
  providers count
  the characters they translate per device, per address and in total, and one of those counters
  hit its daily cap. It starts over at 00:00 UTC. Pick another entry in the dropdown, or deploy
  your own server from [`server/`](./server/README.md) and point the app at it.
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
captured text only ever goes to the translation service you selected — a vendor
endpoint for `google`, and the server the
project runs ([`server/`](./server/README.md)) for the two server-backed entries.

A gesture only opens the card when it really selected something. The release
point is checked first: a double click on a shell surface — a desktop icon, the
taskbar, the Start button — is ignored, so whatever the shell last put on the
clipboard is never translated. The copied text has to hold a letter and must not
look like the path of a file or folder, and a copy of files in Explorer counts as
no selection at all. When `Put the clipboard back after reading a selection` is
on, the previous content is restored once the application that was copied from has
stopped writing — it is checked again for up to 80 ms — so a late write cannot
leave the copied text behind. Glossy remembers the sequence number of every write
it makes itself and skips those while waiting for the `Ctrl+C` to land, so putting
the clipboard back can never be read back as a selection of its own.

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
npm.cmd run tauri build  # release build (installer + MSI + .exe)
```

`scripts\dev.cmd` is the dev command in a file that can be double clicked: it checks that
`cargo` is on `PATH`, installs the Tauri CLI when `node_modules` is missing, refuses to start
while another Glossy is running — a second one would install a second mouse hook — and keeps
the console open with the dev log. Running
`powershell -ExecutionPolicy Bypass -File scripts\make-shortcut.ps1` once puts a shortcut to
it on the desktop.

`npm.cmd run icon` regenerates `src-tauri/icons` from `assets/`.

`npm.cmd run tauri build` writes the NSIS installer to
`src-tauri\target\release\bundle\nsis\Glossy_<version>_x64-setup.exe`, the MSI to
`src-tauri\target\release\bundle\msi\Glossy_<version>_x64_en-US.msi` and the
standalone binary to `src-tauri\target\release\Glossy.exe`. Both bundlers need a
network connection the first time they run, to download their tooling — WiX for the
MSI, NSIS plug-ins for the installer — into `%LOCALAPPDATA%\tauri`.

The portable zip is not a Tauri target, since Tauri has none: `scripts\release.ps1`
packs it from the release binary and the `WebView2Loader.dll` that sits next to it
and that the GNU build loads at run time.

`scripts\release.ps1` wraps the release build: it builds and stages the installer,
the MSI and the portable zip in
`release\v<version>\` together with a checksum and the text for the release
description. Run it as `powershell -ExecutionPolicy Bypass -File
scripts\release.ps1` — scripts are blocked by the default execution policy, the same
reason `npm.cmd` is used above. See [release/README.md](./release/README.md).

### Checks

```powershell
powershell -ExecutionPolicy Bypass -File scripts\version.ps1 -Check   # all version numbers agree
node --test                                                          # 202 frontend tests
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                                                  # 192 Rust tests
cd ..\server
npm test                                                             # 74 server tests
```

`npm test` runs the same `node --test` command. The frontend tests load
`src/js/i18n.js` and `src/js/render.js` into a minimal DOM (see `tests/helpers/`)
and cover the dictionaries, the placeholder substitution and the card renderer.
Run the plain directory form on Node 24/Windows — `node --test tests` and
`node --test .` do not resolve the test files there.

The suite in `server/` needs nothing installed: the calls to the translation
provider are injected, so the quota rules, the request signature and the caller
address parsing are all covered without a network.

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
| 11 | Translate through each of the three translation services — Glossy's server with 百度, the same server with 有道, and the free public Google endpoint — then once with the network unplugged | A result for the three; a readable error with a retry button when nothing can answer |
| 12 | Add a running program to the ignore list, translate inside it, then remove it | Nothing pops up while it is listed, and the popup is back once it is removed |
| 13 | Press the global hotkey with text on the clipboard, then with an empty clipboard and a selection | The translation opens next to the cursor in the first case, the current selection is used in the second |
| 14 | Export the settings, edit a value in the file, then import it | The export holds no `credentials` block; the import applies the edited value and leaves everything else alone |
| 15 | Set the history to `The last 50`, translate 60 texts, then set it to `Off` | The list keeps 50, search filters them, clicking one reopens it in the card, and `Off` empties the file |
| 16 | Change the opacities, font size and width, restart | The popup keeps the chosen values |
| 17 | Open the Updates section | This build, without a signing key, hides the check buttons and says the build cannot update itself; after a key pair exists, `Check now` reports either the running version or the one that is waiting |
| 18 | Press a read-aloud button, press it again while it is lit, then start another reading and close the card | The button turns into a stop square while its text is read, and back into a speaker once the voice stops; the second press stops the reading, closing the card stops it too, and no reading talks over the next card |

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
- The GNU target links `WebView2Loader.dll` dynamically, where MSVC links it into
  the executable, so the installer has to carry the DLL as well. `src-tauri/build.rs`
  copies the one `webview2-com-sys` built into `src-tauri/resources/`,
  `bundle.resources` places it next to `glossy.exe` in the installed app, and
  `scripts/release.ps1` refuses to stage a release without it. The copy `tauri-build`
  leaves in `target/<profile>/` only covers running from the build directory.

```powershell
cd src-tauri
cargo test   # 204 tests; see "Checks" above for lint and format runs
```

### Content Security Policy

`app.security.csp` in `src-tauri/tauri.conf.json` is an explicit whitelist rather
than `null`:

```
default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;
font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; media-src 'none';
object-src 'none'; frame-src 'none'; frame-ancestors 'none'; base-uri 'none';
form-action 'none'
```

The windows load nothing remote — no CDN, no remote font, no remote image — so a
page that tries to reach the network is refused. Two entries exist for the
framework rather than for the app: `connect-src` names `ipc:` and
`http://ipc.localhost`, which is how a window talks to the Rust side on Windows,
and Tauri adds the `script-src` and `style-src` hashes it needs itself. `img-src`
allows `data:` so an icon can be inlined without editing the policy. Translation
requests are unaffected: they leave from Rust, not from the window.

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
  images/logo.png        the app icon, shown as the brand mark in both windows
tests/                   node --test suite for i18n.js and render.js
src-tauri/src/
  main.rs  lib.rs        window setup, Tauri commands
  selection.rs           what a gesture meant: click, drag, double click, capture
  classify.rs            word/phrase vs. sentence detection
  text.rs                cleanup of the text a selection comes back as
  morphology.rs          the forms of an English word (run, runs, running, ran)
  context.rs             the sentence a selected word stands in, via UI Automation
  units/                 unit and currency conversion for the card
  translate/             google and cloud providers, word dictionary,
                         and the languages each of them translates
  platform/
    mod.rs               what the backend may assume about an operating system
    windows/             the modules that talk to Win32: clipboard, console, desktop,
                         hotkey, input, input_hook, instance, secrets, speech, uia
  popup.rs               placement/clamping geometry
  surface.rs             Mica backdrop and title bar colour
  history.rs             the translation store behind the History panel
  autostart.rs           the login item and the --autostart marker
  updater.rs             the release feed behind the Updates section
  notice.rs              start card placement and lifetime
  tray.rs                notification area icon: open the window, quit
  settings.rs  state.rs
server/
  src/                   the proxy: quota rules, the providers, the two hosts
  test/                  node --test suite for the rules and the signatures
  README.md              how to deploy it (Cloudflare Worker or Tencent SCF)
scripts/
  version.ps1            the version number, in one place
  release.ps1            release build + staging for a GitHub release
.github/workflows/
  ci.yml                 format, lint, test and version check on every push
release/
  v<version>/            installer, RELEASE_NOTES.md and checksum of a release
```

## Roadmap

[ROADMAP.md](./ROADMAP.md) holds the releases (`v1.0.0` through `v2.0.0`) with
their acceptance criteria, the versioning policy, the known risks and the release
process. Each release maps to a GitHub milestone of the same name.

[CHANGELOG.md](./CHANGELOG.md) lists what shipped in every released version.

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by
[SignPath Foundation](https://signpath.org). That is what is meant to sit behind the Windows
installers of the [releases page](https://github.com/SpencerZXWu/Glossy/releases), so that
Windows shows a publisher instead of *unknown publisher*. Releases published before that
arrangement was in place are unsigned; with a certificate of one's own,
`scripts/release.ps1 -Sign` signs a local build, as [release/README.md](./release/README.md)
describes.

Roles: [@SpencerZXWu](https://github.com/SpencerZXWu) owns this repository and is its only
author, reviewer and approver — every commit and every release is reviewed and approved by
that maintainer, whose GitHub account requires multi-factor authentication. Only artifacts
built from a commit in this repository are ever signed.

What the app sends where is described in [PRIVACY.md](./PRIVACY.md).

## License

[MIT](./LICENSE) © 2026 Spencer Wu

<a id="zh-cn"></a>

# Glossy · 中文

[![CI](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml/badge.svg)](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml)

一款轻量的 Windows 桌面翻译弹窗。在任意应用里用鼠标选中文字，Glossy 就会在光标下方显示一张小小的浮动卡片，上面是译文。

- **单词或短语** → 音标、词性、释义、一个例句、词形变化、近义词、所在句子和一个朗读按钮。
- **句子或段落** → 顺应目标语言的流畅译文；原文与译文切分不一致时逐句对照。
- **译文仍用目标语言不使用的单位书写的计量或金额** → 换算后的数值和换算率，直接显示在卡片里（`12 ft ≈ 3.66 m`）。货币汇率实时查询，其余单位都内置于应用中。可在设置中关闭。
- 源语言会自动识别。
- 以句末标点（`.`, `?`, `!`, `。` 等）结尾的选区，无论多短，都按句子翻译。

## 安装

从 [Releases](https://github.com/SpencerZXWu/Glossy/releases/latest) 下载最新发布，按需要选一个：

| 下载 | 它是什么 |
| --- | --- |
| `Glossy_<version>_x64-setup.exe` | 安装包，想用自动更新就选它。会把 Glossy 注册进「已安装的应用」，因此可以按常规方式卸载 |
| `Glossy_<version>_x64_en-US.msi` | 同一个应用，以 MSI 包形式提供，适合用策略或安装脚本部署的人 |
| `Glossy_<version>_x64_portable.zip` | 无需安装：解压到任意位置，在该目录运行 `glossy.exe`。请让 `WebView2Loader.dll` 和它待在一起；另外「开机自启」记住的是你解压的那个目录，所以别随意搬动它 |

Windows 11 x64；需要 WebView2，当前的 Windows 版本已自带。同一发布里的
`SHA256SUMS.txt` 列出了三者的哈希。

三者都尚未代码签名，因此 SmartScreen 会警告未知发布者。若想改为从源码构建，请见 [开发](#开发)。

## 使用方法

1. 启动 Glossy。设置窗口只在首次启动时打开——之后的启动都会安静地在通知区域运行，Glossy 从启动那一刻起就开始监听选区。
2. 关闭该窗口并不会退出 Glossy——它会在后台继续监听选区。点击通知区域中的 Glossy 图标（或在其菜单中选择**打开 Glossy**）可以把窗口重新调出来，用同一个菜单中的**退出**来结束 Glossy。在已经运行时再次启动 Glossy，只会显示一条简短提示。静默启动会由右下角的一张小卡片告知；它几秒后淡出，点击它会打开设置窗口。Windows 11 会把新的通知区域图标收进溢出菜单（时钟旁的 `^`）——把图标拖到任务栏上，或在**设置 → 个性化 → 任务栏 → 其他系统托盘图标**中打开它，即可让它保持可见。
3. 在任何应用中，**拖动划过文字**（或**双击一个单词**）即可选中它。
4. Glossy 会在选区下方显示一个小图标。在**点击这个图标**之前不会翻译，也不会占用当天的翻译额度，因此只是随手划过的一个词不会有任何消耗。点击其他区域图标就会消失。
5. 卡片出现在选区下方。拖动它的标题栏可以移动它，用按钮复制结果或打开设置，也可以点击别处让它消失。标题栏下方的那一行显示语言对：悬停后用下拉框选择任意一侧，即可用该语言重新翻译；按下 `⇄` 按钮则把译文回译成它原本的语言。每次新的选区都会把这个语言对重置为*识别源语言并使用已配置的目标语言*；手动选定的目标语言会被记住并成为已配置的目标语言，所以下一次选区直接以它为译文语言，而手动选定的源语言只对当前这张卡片有效。
6. 短于所设最小长度的选区（默认 2 个字符）会被忽略，起点或终点落在弹窗本身的拖动永远不会触发翻译。

单词卡片会显示音标、各词性及其释义、一个简单例句，并在同一张卡片里补上这个词的词形变化、意思相近的词、它被选中时所在的句子及该句译文，以及卡片两侧各一个朗读按钮。句子或段落直接显示译文，原文在译文上方（最多四行），段落较长时下方还有逐句对照。

按下全局快捷键（默认 **Ctrl+Alt+C**）可以改为翻译剪贴板内容，而无需选中任何东西。

弹窗是一个不抢焦点、始终置顶的窗口：它绝不会从你正在阅读的应用那里抢走键盘焦点。它始终保持在光标所在显示器的工作区之内，下方没有空间时会翻到光标上方。

### 设置

| 选项 | 含义 |
| --- | --- |
| 界面语言 | `跟随系统`、`简体中文` 或 `English`。会立即切换设置窗口和弹窗。 |
| 开启划词翻译 | 总开关。关闭后立即暂停全局划词捕获。 |
| 随 Windows 启动 | 在 `HKCU\...\Run` 中写入一条 `--autostart` 项，这样登录后 Glossy 就已经在通知区域待命。这样启动时不会显示"Glossy 已在后台运行"卡片。 |
| 拖动鼠标划过文字时翻译 | 启用拖动划词手势。 |
| 双击单词时翻译 | 启用双击手势。 |
| 读取选区后恢复剪贴板 | 在 Glossy 复制了选区之后，恢复你原先的剪贴板内容。 |
| 在弹窗中显示原文 | 关闭时隐藏卡片中的原文行。选中的文字较长时原文最多显示四行，多出来的部分只在该区块内滚动，译文因此始终留得住位置。 |
| 把译文语言不常用的计量与货币换算过来 | 当译文仍用目标语言不使用的单位来计量某样东西时——英尺、磅、°F、外币金额——卡片会在下方补上换算后的数值和换算率（`12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`）。数值是从译文中读取的，因为只有翻译服务才能判断一个符号到底是不是单位，而且它总是用单位表所认识的语言书写；如果译文里没有，就改为读取原文。长度、质量、体积、速度、面积和温度都在各自类别内换算，目标单位取人们实际会写的那个；目标货币跟随目标语言（`zh-CN` → CNY，`en` → USD）。货币汇率由 Glossy 的服务端转发（服务端读 `open.er-api.com`，取不到就换欧洲央行的 `api.frankfurter.app`），服务端答不上来时才由客户端直连这两家；不管走哪条路，汇率都缓存六小时。其余一切都内置于应用中。关闭表示不换算，也不请求汇率。 |
| 显示所查单词所在的句子 | 卡片会补上这个单词被选中时所在的句子，以及该句的译文。关闭时只翻译单词本身。 |
| 原文与译文逐句对照 | 译文切分方式与原文不同的段落，会按对齐的一行一句列出，而不是整块显示。如果译文只回来一行，就仍然按普通段落显示。 |
| 精简卡片，不显示附加信息 | 保留译文、音标和释义，省略例句、词形变化、近义词、所在句子、逐句对照和单位换算。 |
| 朗读语速 | 朗读按钮的语速：`慢`、`正常`、`快` 或 `很快`（Windows SAPI）。音色用的是 Windows 已有的语音，音色由文本的语言决定。 |
| 触发翻译的最短长度 | 低于该字符数的选区会被忽略（`1`–`40`，默认 `2`）。 |
| 全局快捷键 | 用于翻译剪贴板内容的快捷键，例如 `Ctrl+Alt+C`。清空该字段即可关闭它。字段下方的一行显示已注册的组合，或 Windows 拒绝它的原因。 |
| 以下程序中不翻译 | 一份进程名列表（`idea64.exe`、`mstsc`），其中的程序会跳过划词捕获。输入名字即可添加（`.exe` 后缀可选——`添加` 按钮会把它规范化），也可以从当前运行程序的下拉框中选择，或按下 `用鼠标拾取` 后点选要忽略的窗口。每个条目都有一个 `×` 可以删除；重复项按大小写不敏感处理并被丢弃。 |
| 仅翻译以下原文语言 | 一份语言列表，例如 `英语` 和 `日语`。留空表示任何语言都会触发翻译。无法确定语言的选区——混排文字、数字、多种语言共有的词——一律放行，因此猜错也不会吞掉你的选区。该列表以 `sourceLangs` 数组保存。 |
| 配色 | `跟随系统` 跟随 Windows 的浅色/深色偏好；`始终浅色` 和 `始终深色` 会在两个窗口中强制使用一种方案。 |
| 文字大小 | 弹窗中所有文字大小的倍数（`90 %`–`150 %`）。 |
| 宽度 | 弹窗卡片宽度（`300`–`520` CSS 像素）。 |
| 不透明度 | 弹窗卡片的透明程度（从 `100 %` 不透明一直到 `50 %`）。卡片会变淡，而文字在它背后的任何内容之上都保持可读。 |
| 自动关闭 | 弹窗自行隐藏前的秒数；`不自动关闭` 会让它一直开着直到被关闭。 |
| 复制译文后立即关闭弹窗 | 使用过复制按钮后隐藏卡片。 |
| 翻译为 | 译文要翻译成的语言。 |
| 历史记录 | 记住多少条已完成的翻译（`关闭` 到 `最近 500 条`，默认 50）。选择器下方的列表保留原文、译文、翻译渠道和时间；`搜索` 会同时过滤两段文本，点击一条记录会在浮动卡片中再次显示它（不会再次请求翻译渠道），每条记录都有复制和删除按钮。`清空历史记录` 会清空列表。该文件位于 `%APPDATA%\com.glossy.translator\history.json`。 |
| 设置文件 | `导出…` 会写入 `Documents\glossy-settings.json`；`导入…` 会把你选择的文件读回应用中。文件里只有各项设置，没有任何机密——到处都没有密钥字段了，所以导出时不会再问你要不要带密钥。导入会通过 `sanitized()` 校验。 |
| 更新 | `启动 Glossy 时检查新版本` 会在每次启动时询问 GitHub Releases（默认关闭）。`立即检查` 会立刻查看并说明是哪个版本在等待，`下载并重启` 则会安装它。没有更新签名密钥的构建——在发布密钥对存在之前的所有构建都是如此——会隐藏这些按钮并说明原因。 |
| 翻译渠道 | 由哪个服务来翻译：`cloud-baidu`（Glossy 自己的服务器，用百度翻译——无需配置）、`cloud-youdao`（同一台服务器，用有道翻译——同样无需配置）、`google`（免费公开接口，无需密钥）。全新安装默认使用 `cloud-baidu`。底层仍按 `channel` + `cloudProvider` + `cloudVendor` / `provider` 保存，见下文。 |
| 所选服务失败时改用其他服务 | 上面选中的渠道永远第一个尝试，且不能移动；它下面的条目会在它失败、被限流或什么都没返回时按顺序接管，可以用箭头调整顺序。关闭开关后，失败就只是失败。当回答问题的不是首选渠道时，卡片会写明是谁回答的。 |
| 额度提示行（`cloud-baidu`、`cloud-youdao`） | 只在走服务器的那两个渠道下显示：今天还剩多少额度——或者服务器联系不上的原因——旁边是 `重新检查` 按钮。服务器地址写死在构建里而不是做成输入框，免得别人把「无需配置」的服务填坏；想用自己的部署见 [`server/`](./server/README.md)。 |

快捷键接受 `Ctrl`/`Control`、`Alt`、`Shift`、`Win`/`Meta` 外加一个按键：
字母、数字、`F1`–`F24`、`Space`、`Enter`、`Tab`、`Esc`、`Backspace`、
`Delete`、`Insert`、`Home`、`End`、`PageUp`、`PageDown` 或方向键。至少
需要一个修饰键。当 Windows 拒绝该组合时——通常是因为别的程序已经占用了它——
原因会显示在字段下方，并且在该设置被改正之前快捷键一直不会生效。

如果剪贴板里完全没有文本，快捷键会退回到复制当前选区，所以"选中文字，按下快捷键"也一样管用。

### 翻译渠道

下拉框里的三个渠道都不需要你自己的账号：`google` 走公开接口，走服务器的那两个由 Glossy 的部署拿着厂商账号来回答，因此任何密钥都不会进入客户端。它们只是点名让服务器用哪家引擎翻译，答不上来时服务器会自动改用别的上游。

| 翻译渠道 | 费用 | 说明 |
| --- | --- | --- |
| `cloud-baidu` | 无需填写 | **百度翻译**：由后端服务器（[`server/`](./server/README.md)）用本项目自己的账号完成翻译，在请求里名为 `vendor: "baidu"`。应用只发送文本和一个安装 id，凭据留在服务器上，中国大陆可直接访问。每日额度按设备、按地址以及总量分别统计，设置窗口里会显示当天还剩多少；想换成自己的部署，只要改构建里的那一行地址即可。 |
| `cloud-youdao` | 无需填写 | **有道翻译**：同一台服务器，只是指定用有道智云来译（`vendor: "youdao"`）。同样无需填写，有道答不上来时同样会自动改用别的上游。 |
| `google` | 免费，无需密钥 | 公开的 `translate.googleapis.com` 接口。在部分网络中被屏蔽，包括中国大陆的大部分地区。始终以 `dict-chrome-ex` 客户端 id 查询；被限流的 `gtx` id 只作为后备。 |

每个渠道能翻的语言并不相同：百度普通账号会拒绝 Glossy 提供的 31 种语言中的 8 种（`uk`、`tr`、`hi`、`id`、`ms`、`he`、`no`、`sk`），而有道和 Google 全部支持。两个语言栏只列出卡片当前所用渠道支持的语言，因此它翻不了的语言既不会出现在列表里，也不会被发出去。

单个单词的音标和释义来自 `api.dictionaryapi.dev`，它免费、无需密钥，且中国大陆可直接访问。只有在所选服务没有返回某些细节时，才会去请求 Google 接口。

### 免费额度与商业使用

各免费额度在许可范围上并不相同。它们禁止的是把额度本身转手给他人或转售，也就是把密钥、接口交给别人；把凭据留在我们自己的服务器上、让应用去调用这台服务器，属于账号持有者自己使用额度，因此走服务器的那两个渠道正是以这种方式中转：

- **`cloud-baidu` / `cloud-youdao`** —— 它们走的是本项目付费的 LLM 账号；没有配置该账号时则退回百度凭据，因此发出的是付费调用，上面的条款对它不适用。任何用户都看不到也拿不到厂商密钥：应用只发送文本和一个安装 id，服务器按设备和地址计算每日额度。
- **智谱** —— 用户协议 §非付费功能 只把免费模型授权给*非商业的、个人研究学习*用途。个人使用没问题；用于已发布或收费的产品则不行。
- **ModelScope API-Inference** —— 明确为非商业化、非盈利。
- **阿里云机器翻译** —— 每月免费额度明确仅适用客户试用场景。
- **腾讯云 TMT** —— 每月 5M 字符的免费额度仍在宣传，但该服务已不再提供文本翻译：自 2026-03 和 2026-07 的版本起，`TextTranslate`、`TextTranslateBatch`、`ImageTranslate`、`LanguageDetect` 和 `SpeechTranslate` 动作已被移除，API 概览中只列出 `ImageTranslateLLM`。计费概述页面已经过时，所以不要指望它。
- **百度翻译开放平台** —— 未认证 50k 字符/月，个人认证 1M，企业认证 2M。其条款对商业使用只字未提，但服务协议禁止*客户端程序*缓存百度翻译数据，也禁止转售。Glossy 不缓存任何内容，所以这不会造成影响；某个加了缓存的分支则需要重新审视这一点。额度受 QPS 限制为 1 / 10 / 100。
- **火山引擎** —— 每月 2M 字符，但开通需要签订销售合同。
- **小牛翻译** —— 注册后每天 200k 字符；商业条款未公开。
- **SiliconFlow** —— `tencent/Hunyuan-MT-7B` 免费，但平台条款对免费模型只字未提，且只限于企业内部业务用途。
- **完全离线** —— `Opus-MT` / `Argos Translate` 权重是 CC-BY-4.0/Apache-2.0（可商用，int8 约 83 MB，内存约 300 MB）。`NLLB-200` 是 CC-BY-NC-4.0，**不能**发布。

设置以 JSON 形式保存在 `%APPDATA%\com.glossy.translator\settings.json`。
文件开头是 `formatVersion`，标明它用的是哪一版格式；自 `1.3.0` 起设置项只增不改，IPC 命令名也不再变动，因此较早版本写出的文件始终能被读入。本构建读不了的文件——不是 JSON 对象，或格式版本比本构建更新的——会被复制为旁边的 `settings.backup-<Unix 秒数>.json`，应用则从默认值开始，而不是把它覆盖掉。
该文件中的 `firstRun` 标记记录着欢迎窗口已经显示过；删除它（或整个文件）会在下次启动时把该窗口带回来。
翻译凭证存放在一个以渠道为键的 `credentials` 映射中，已有的单渠道 `apiKey`/`appId` 值会在首次启动时迁移进该映射。
该映射中的每个值在写入文件之前都会用 Windows DPAPI（`CryptProtectData`，当前用户范围）加密，因此文件里保存的是
`"apiKey": "dpapi:AQAAANCM…"` 而不是密钥本身，只有录入它的那个 Windows 登录账户才能读回它。由较早版本写出的文件会在该构建首次启动时得到保护；属于其他登录账户或计算机的值无法解锁，会被丢弃，因此必须重新输入密钥。
被忽略的程序列表以 `ignoredApps` 数组保存；旧版用逗号分隔的字符串也能接受，并在加载时拆分。
由哪个服务翻译仍按旧的两段式保存：`channel`（`cloud` 或 `api`）区分是走服务器的那两个渠道还是你的账号——`provider` 指出其中的具体服务——走服务器时是 `baidu`（渠道自己忽略这个值，保留它只是因为该字段属于文件格式的一部分），免费 Google 接口则是 `google`；`cloudProvider`（`builtin` 或 `local`，后者现在读作内置引擎，因为下拉框已不再提供它）以及 `cloudVendor`（`baidu` 或 `youdao`）是走服务器的那两个渠道点名要用的上游。全新安装从 `baidu` 开始。下拉框在读取和写入时都会做这个映射，因此旧版本写下的文件依然可用：没有 `channel` 的文件会被当作它原本就是的 API 渠道读取，新文件会写成 `cloud`，不是 `youdao` 的 `cloudVendor` 读作百度，而这个构建不再提供的 `provider`——自带密钥的百度、智谱、DeepL、OpenAI——会读作内置的百度渠道，并在下次写文件时被替换掉。

### 在应用内翻译

设置窗口顶部是与弹窗相同的翻译功能：一个用来输入或粘贴文本的文本框。在该框中选中一个单词、一个短语或一个句子——或者按下**翻译**（Ctrl+Enter）来使用整段文本——结果会立刻显示在下方一张与弹窗完全相同的卡片里：带交换按钮的源语言/目标语言栏、复制按钮，以及相同的单词/句子渲染。粘贴进来的文本会立即翻译，无论用的是**粘贴**按钮还是 Ctrl+V；**清空**会清空文本框并收起卡片。标题右侧的开关为整个应用开关单位换算：它与下方**单位换算**面板里的是同一个设置，也是唯一能就地切换卡片换算结果的地方。语言栏遵循弹窗的规则，因此源语言从*自动检测*开始，目标语言跟随已配置的语言，新的选区会重置语言对（源语言的临时选择不会被保存，目标语言则会被保存下来并成为已配置的语言）。按下**在悬浮窗中显示**会用当前选区（没有选中内容时则用整段文本）打开真正的弹窗——这样无需全局钩子就能检验弹窗。

### 常见问题

- **"Google Translate is rate limiting requests right now"**（Google 正在限流）—— Google 返回了 `429`，公开接口对桌面 HTTP 客户端常常如此，即便浏览器或 `curl` 仍然能正常工作。Glossy 会先用第二个客户端 id 重试；如果提示还在，等一分钟，或改用下拉框里另外两个渠道中的任意一个。
- **"Could not reach Google Translate"**（无法连接 Google Translate）—— 免费的 `google` 渠道会访问 `translate.googleapis.com`，它在部分网络中被屏蔽（包括中国大陆的大部分地区）。卡片会提供重试按钮；如果一直失败，请切换到 `cloud-baidu` 或 `cloud-youdao`（什么都不用填）。
- **"The free cloud translation quota for today is used up."**（今天的免费云端翻译额度已用完）—— 走服务器的那两个渠道会按设备、按地址和总量分别统计它翻译过的字符数，其中某一个计数器碰到了当天的上限，它会在 UTC 00:00 重新开始。可以换用下拉框里的别的渠道，或者从 [`server/`](./server/README.md) 部署一个自己的服务器并把应用指向它。
- **"Baidu rejected the APP ID" / "Baidu rejected the signature"**（百度拒绝了 APP ID / 百度拒绝了签名）—— 凭证对的两半被对调或输错了。`baidu` 需要在第一个框中填 APP ID，第二个框填密钥；密钥从不会被发送给百度，它只用来给请求签名。
- **"Baidu rejected this computer's IP address"**（百度拒绝了本机的 IP 地址）—— 百度控制台中该应用的 IP 白名单被填上了内容。要么清空它，要么把 Glossy 拨号所用的地址加进去。
- **弹窗显示"已暂停"**，尽管翻译是开启的 —— 设置窗口中的总开关被关掉了，或者启动时设置文件无法读取。

## 划词是如何捕获的

Glossy 会安装一个 `WH_MOUSE_LL` 钩子，并监听鼠标左键的按下/松开配对。钩子回调只记录坐标；工作线程判断该手势是拖动还是双击，用 `Ctrl+C` 复制选区（当前台窗口以管理员权限运行时改发 `Ctrl+Insert`），按需恢复剪贴板，最后告诉弹窗窗口该显示什么。捕获到的文本只会发送给你所选择的翻译渠道——`google` 发给厂商接口，走服务器的那两个渠道则发往本项目自己运行的服务器（[`server/`](./server/README.md)）。

只有当手势真的选中了东西时，卡片才会打开。首先检查松开鼠标的位置：在 shell 表面（桌面图标、任务栏、开始按钮）上的双击会被忽略，因此 shell 最后放进剪贴板的东西永远不会被拿去翻译。复制到的文本必须含有字母，且不能看起来像文件或文件夹的路径；在资源管理器里复制文件则一律视为没有选中内容。勾选 `读取选区后恢复剪贴板` 时，会在被复制的那个程序停止写入之后再恢复原内容——最多再检查 80 ms——因此迟到的写入不会把复制到的文本留在剪贴板里。Glossy 会记下自己每一次写入的序列号，在等待 `Ctrl+C` 落地时跳过这些写入，因此"恢复剪贴板"这一步不会被它自己当成一次选区读取。

## 开发

### 环境要求

| 工具 | 版本 | 说明 |
| --- | --- | --- |
| Node.js | 18 或更新 | 仅用于 Tauri CLI；前端没有打包器 |
| Rust | stable，`1.77` 或更新 | GNU 目标 `x86_64-pc-windows-gnu` —— 见下文 |
| `rustfmt` + `clippy` | 同一工具链 | `rustup component add rustfmt clippy` |
| MinGW-w64 | 8.1 或更新 | GNU 目标的链接器；需在 `PATH` 中作为 `gcc.exe` |

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu --component rustfmt --component clippy
rustup default stable-x86_64-pc-windows-gnu
npm.cmd install          # npm.ps1 is blocked by the default execution policy
npm.cmd run tauri dev    # dev build with hot reload of src/
npm.cmd run tauri build  # release build (installer + MSI + .exe)
```

`scripts\dev.cmd` 把上面那条开发命令包成一个可以双击的文件：它会检查 `cargo` 是否在
`PATH` 中、在缺少 `node_modules` 时安装 Tauri CLI、发现已有 Glossy 在运行时拒绝启动
（第二个实例会装上第二个鼠标钩子），并让控制台留着显示开发日志。运行一次
`powershell -ExecutionPolicy Bypass -File scripts\make-shortcut.ps1` 可以在桌面上生成
指向它的快捷方式。

`npm.cmd run icon` 会从 `assets/` 重新生成 `src-tauri/icons`。

`npm.cmd run tauri build` 会把 NSIS 安装包写到
`src-tauri\target\release\bundle\nsis\Glossy_<version>_x64-setup.exe`，把 MSI 写到
`src-tauri\target\release\bundle\msi\Glossy_<version>_x64_en-US.msi`，把独立可执行文件写到
`src-tauri\target\release\Glossy.exe`。两个打包器首次运行都需要网络连接，以下载各自的
工具（MSI 用 WiX，安装包用 NSIS 插件）到 `%LOCALAPPDATA%\tauri`。

便携 zip 不是 Tauri 的打包目标，因为 Tauri 没有这种目标：它由 `scripts\release.ps1`
从发布二进制和紧挨着它的 `WebView2Loader.dll`（GNU 构建在运行时加载它）打包而成。

`scripts\release.ps1` 封装了发布构建：它会构建安装包、MSI 和便携 zip，并把三者暂存到
`release\v<version>\`，连同校验和与发布说明的文本。用
`powershell -ExecutionPolicy Bypass -File
scripts\release.ps1` 运行它——脚本会被默认执行策略拦截，这也是上面使用 `npm.cmd` 的同一个原因。参见 [release/README.md](./release/README.md)。

### 检查

```powershell
powershell -ExecutionPolicy Bypass -File scripts\version.ps1 -Check   # all version numbers agree
node --test                                                          # 202 frontend tests
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                                                  # 192 Rust tests
cd ..\server
npm test                                                             # 74 server tests
```

`npm test` 运行的是同一条 `node --test` 命令。前端测试把
`src/js/i18n.js` 和 `src/js/render.js` 加载进一个最小 DOM（见 `tests/helpers/`），
覆盖词典、占位符替换和卡片渲染器。在 Node 24/Windows 上请使用纯目录形式运行——`node --test tests` 和
`node --test .` 在那里无法解析测试文件。

`server/` 里的测试不需要安装任何依赖：对上游的调用是注入进去的，所以
额度规则、请求签名和调用方地址解析都能在离线环境下被覆盖。

版本号存在于六个地方（见 `scripts\version.ps1`），所以绝不要手动修改它们：
`tauri.conf.json` 是权威来源，`scripts\version.ps1 -Set 0.1.2` 会把它写到其他所有地方，
`scripts\version.ps1 -Get` 会为 `release.ps1` 之类的脚本打印它，后者在版本号不一致时拒绝构建。同一个检查也在 CI 中运行，所以忘记升版本会让构建失败。

[`.github/workflows/ci.yml`](./.github/workflows/ci.yml) 会在每次推送和拉取请求时，于 `windows-latest` 上运行全部四项检查以及前端测试套件。

### 更新

Glossy 会向
`https://github.com/SpencerZXWu/Glossy/releases/latest/download/latest.json` 查询是否有更新的版本。只有携带用发布密钥生成的签名时，更新才会被接受：

```powershell
cargo tauri signer generate -w $env:USERPROFILE\.tauri\glossy.key   # once
```

把打印出来的公钥填入
`src-tauri\tauri.conf.json` 中的 `plugins.updater.pubkey`（里面的占位符正是设置窗口声称该构建无法自我更新的原因），并在构建发布版之前导出私钥：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $env:USERPROFILE\.tauri\glossy.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<the password you chose>'
```

`tauri.conf.json` 中的 `bundle.createUpdaterArtifacts` 在该私钥存在之前保持关闭：
没有它打包器会失败，有它构建还会写出已签名的安装包和读取更新所用的 `latest.json`。请把私钥留在仓库之外——它是唯一能让发布版被替换的东西。

### 手动回归检查清单

在给发布版打标签之前运行这一清单。其中每一项都曾在某个时刻是真实的 bug，而且没有任何一项被自动化测试覆盖。

| # | 要做的事 | 应有的结果 |
| --- | --- | --- |
| 1 | 在干净的配置下启动 Glossy | 设置窗口打开，没有任何东西被意外预选，状态行显示捕获正在运行 |
| 2 | 关闭设置窗口，然后再次启动 Glossy | 没有第二个窗口，也没有第二个鼠标钩子：通知显示 Glossy 已在运行，托盘图标可以再次打开窗口 |
| 3 | 勾选"随 Windows 启动"，重启，然后登录 | 没有控制台窗口闪现，没有"运行中"卡片出现，托盘图标就在那里 |
| 4 | 取消勾选，重启 | Glossy 没有启动（`reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Glossy` 查不到任何内容） |
| 5 | 在记事本、浏览器、Word 和终端中选中一个单词 | 每次弹窗都出现在光标下方，并带有该单词的音标和释义 |
| 6 | 在同样的每个程序中选中一个句子 | 弹窗只显示译文，没有音标或释义 |
| 7 | 把弹窗拖到每台显示器和每个屏幕边缘 | 卡片在每台显示器上、在 100 %、125 % 和 150 % 缩放下都保持在工作区内 |
| 8 | 固定弹窗，点击桌面别处，等待超过自动关闭超时 | 卡片保持打开，直到取消固定或使用 × |
| 9 | 点击未固定卡片的外部 | 点击一落到外面，它就关闭 |
| 10 | 在 Windows 中切换浅色/深色，然后在设置中强制使用每种方案 | 两个窗口都跟随该选择，两者的文字和边框都清晰可读 |
| 11 | 对三个翻译渠道各翻译一次——Glossy 自己的服务器配百度、同一台服务器配有道，以及免费的公开 Google 接口——然后断开网络再翻一次 | 前三个能出结果；什么都连不上时显示可读的错误和重试按钮 |
| 12 | 把正在运行的程序加入忽略列表，在其中翻译，然后移除它 | 它在列表中时不会弹出任何东西，移除后弹窗恢复 |
| 13 | 剪贴板中有文本时按全局快捷键，然后在剪贴板为空且有选区时再按一次 | 第一种情况下译文在光标旁打开，第二种情况下使用当前选区 |
| 14 | 导出设置，在文件中改一个值，然后导入它 | 导出文件里没有 `credentials` 块；导入会应用改过的值，其余设置保持不变 |
| 15 | 把历史记录设为 `最近 50 条`，翻译 60 段文本，然后把它设为 `关闭` | 列表保留 50 条，搜索能过滤它们，点击一条会在卡片中重新打开它，`关闭` 会清空文件 |
| 16 | 修改不透明度、字号和宽度，然后重启 | 弹窗保持所选择的值 |
| 17 | 打开更新部分 | 没有签名密钥的这个构建会隐藏检查按钮并说明该构建无法自我更新；密钥对存在之后，`立即检查` 要么报告当前运行的版本，要么报告正在等待的那个版本 |
| 18 | 按一次朗读按钮，再按一次那个正在朗读的按钮，然后开始一次朗读并关闭卡片 | 朗读时按钮变成停止方块，语音停下后变回喇叭；第二次按下会停止朗读，关闭卡片也会停止朗读，下一次卡片不会被上一次的朗读盖过 |

### 工具链说明（Windows，GNU 工具链）

本项目使用 GNU Rust 目标（`x86_64-pc-windows-gnu`）构建，它不需要安装
Visual Studio，但有三个小怪癖：

- `windows` crate 会让导入表增长到旧版 MinGW binutils 无法接受的程度。
  因此 `src-tauri/Cargo.toml` 声明了 `[lib] crate-type = ["rlib"]`，应用 crate 只作为二进制文件链接。
- `cargo test` 在测试执行器中也同样需要生成的资源归档（图标、版本信息和
  common-controls v6 清单），否则执行器会在 `main` 之前以 `0xC0000139` 启动失败。`src-tauri/build.rs` 为 `libresource.a` 添加了一条普通的
  `cargo:rustc-link-arg`，以覆盖每个测试目标。构建脚本无法区分二进制目标和测试目标，
  因此二进制目标会收到两次该归档，GNU ld 会打印 `.rsrc merge failure: multiple non-default
  manifests` 警告；生成的 `.exe` 仍然携带清单并能运行。
- GNU 目标以动态方式链接 `WebView2Loader.dll`（MSVC 会把它直接链进可执行文件），
  因此安装包必须一并携带这个 DLL。`src-tauri/build.rs` 会把 `webview2-com-sys`
  构建出来的那个复制到 `src-tauri/resources/`，`bundle.resources` 让它在安装好的
  程序目录里落在 `glossy.exe` 旁边，`scripts/release.ps1` 在缺少它时会拒绝发版。
  `tauri-build` 自己复制到 `target/<profile>/` 的那一份只够从构建目录直接运行。

```powershell
cd src-tauri
cargo test   # 204 tests; see "Checks" above for lint and format runs
```

### 内容安全策略

`src-tauri/tauri.conf.json` 里的 `app.security.csp` 不再是 `null`，而是一份显式白名单：

```
default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;
font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; media-src 'none';
object-src 'none'; frame-src 'none'; frame-ancestors 'none'; base-uri 'none';
form-action 'none'
```

窗口不加载任何远程内容 —— 没有 CDN、没有远程字体、没有远程图片 —— 所以试图联网的
页面会被拒绝。其中两项是为框架而不是为应用准备的：`connect-src` 里的 `ipc:` 和
`http://ipc.localhost` 是窗口在 Windows 上与 Rust 侧通信的方式，`script-src` 与
`style-src` 需要的哈希由 Tauri 自己补上。`img-src` 允许 `data:`，这样要内联一个图标
时不必改动策略。翻译请求不受影响：它们从 Rust 发出，而不是从窗口发出。

## 目录结构

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
  images/logo.png        the app icon, shown as the brand mark in both windows
tests/                   node --test suite for i18n.js and render.js
src-tauri/src/
  main.rs  lib.rs        window setup, Tauri commands
  selection.rs           what a gesture meant: click, drag, double click, capture
  classify.rs            word/phrase vs. sentence detection
  text.rs                cleanup of the text a selection comes back as
  morphology.rs          the forms of an English word (run, runs, running, ran)
  context.rs             the sentence a selected word stands in, via UI Automation
  units/                 unit and currency conversion for the card
  translate/             google and cloud providers, word dictionary,
                         and the languages each of them translates
  platform/
    mod.rs               what the backend may assume about an operating system
    windows/             the modules that talk to Win32: clipboard, console, desktop,
                         hotkey, input, input_hook, instance, secrets, speech, uia
  popup.rs               placement/clamping geometry
  surface.rs             Mica backdrop and title bar colour
  history.rs             the translation store behind the History panel
  autostart.rs           the login item and the --autostart marker
  updater.rs             the release feed behind the Updates section
  notice.rs              start card placement and lifetime
  tray.rs                notification area icon: open the window, quit
  settings.rs  state.rs
server/
  src/                   the proxy: quota rules, the providers, the two hosts
  test/                  node --test suite for the rules and the signatures
  README.md              how to deploy it (Cloudflare Worker or Tencent SCF)
scripts/
  version.ps1            the version number, in one place
  release.ps1            release build + staging for a GitHub release
.github/workflows/
  ci.yml                 format, lint, test and version check on every push
release/
  v<version>/            installer, RELEASE_NOTES.md and checksum of a release
```

## 路线图

[ROADMAP.md](./ROADMAP.md) 记录了各版本（`v1.0.0` 到 `v2.0.0`）及其验收标准、版本号策略、已知风险和发布流程。每个版本都对应一个同名的 GitHub 里程碑。

[CHANGELOG.md](./CHANGELOG.md) 列出了每个已发布版本中交付的内容。

## Code signing policy · 代码签名策略

[发布页](https://github.com/SpencerZXWu/Glossy/releases)的 Windows 安装包由
[SignPath Foundation](https://signpath.org) 提供证书、[SignPath.io](https://about.signpath.io)
提供免费签名（Free code signing provided by SignPath.io, certificate by SignPath Foundation），
让 Windows 显示发布者而不是"未知发布者"。在这套办法落地之前发布的版本没有签名；若自己持有证书，
可用 `scripts/release.ps1 -Sign` 为本地构建签名，见 [release/README.md](./release/README.md)。

角色：[@SpencerZXWu](https://github.com/SpencerZXWu) 拥有本仓库，是唯一的作者、审查者和批准者
——每个提交、每个版本都由该维护者审查批准，其 GitHub 账号启用了双因素认证。只会签名由本仓库的提交
构建出的产物。

应用会发送什么、发给谁，见 [PRIVACY.md](./PRIVACY.md)。

## 许可证

[MIT](./LICENSE) © 2026 Spencer Wu

<a id="es"></a>

# Glossy · Español

[![CI](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml/badge.svg)](https://github.com/SpencerZXWu/Glossy/actions/workflows/ci.yml)

Un traductor emergente de escritorio y ligero para Windows. Selecciona texto con
el ratón en cualquier aplicación y Glossy muestra una pequeña tarjeta flotante
con la traducción bajo el cursor.

- **Palabra o frase corta** → símbolos fonéticos, categoría gramatical,
  definiciones, un ejemplo, las formas de la palabra, los sinónimos, la frase de
  la que procede y un botón de pronunciación.
- **Oración o párrafo** → una traducción fluida a tu idioma de destino, frase por
  frase cuando los dos no se dividen igual.
- **Una medida o una cantidad** que la traducción sigue expresando en una unidad
  que tu idioma de destino no utiliza → el valor convertido y el factor de
  conversión, directamente en la tarjeta (`12 ft ≈ 3.66 m`). Los tipos de cambio
  de las divisas se consultan en vivo; todas las demás unidades están integradas
  en la aplicación. Se puede desactivar en los ajustes.
- El idioma de origen se detecta automáticamente.
- Una selección que termina con puntuación de oración (`.`, `?`, `!`, `。` …) se
  traduce siempre como oración, por corta que sea.

## Instalación

Descarga la última versión desde
[Releases](https://github.com/SpencerZXWu/Glossy/releases/latest) y elige lo que
te convenga:

| Descarga | Qué es |
| --- | --- |
| `Glossy_<version>_x64-setup.exe` | El instalador, y la opción que hay que elegir si quieres que funcione la actualización automática. Registra Glossy en **Aplicaciones instaladas**, así que se desinstala de la forma habitual |
| `Glossy_<version>_x64_en-US.msi` | La misma aplicación como paquete MSI, para quien la despliegue con una directiva o un script de instalación |
| `Glossy_<version>_x64_portable.zip` | Nada que instalar: descomprímelo donde quieras y ejecuta `glossy.exe` desde esa carpeta. Deja `WebView2Loader.dll` a su lado y ten en cuenta que el inicio con Windows recuerda la carpeta donde lo descomprimiste, así que no la muevas |

Windows 11 x64; se necesita WebView2, que viene incluido en las versiones actuales
de Windows. El `SHA256SUMS.txt` de la misma publicación lista los hashes de los tres.

Ninguno está firmado con un certificado de código todavía, así que
SmartScreen advierte de un editor desconocido. Para compilarlo desde el código
fuente, consulta [Desarrollo](#desarrollo).

## Uso

1. Inicia Glossy. La ventana de ajustes se abre solo en el primer arranque; los
   arranques posteriores se inician en silencio en el área de notificación, y
   Glossy escucha las selecciones desde el momento en que arranca.
2. Cerrar esa ventana no cierra Glossy: sigue vigilando las selecciones en
   segundo plano. Haz clic en el icono de Glossy del área de notificación (o
   elige **Open Glossy** en su menú) para volver a mostrar la ventana, y usa
   **Quit** en ese mismo menú para detener Glossy. Iniciar Glossy cuando ya se
   está ejecutando solo muestra un breve aviso. Un arranque silencioso se anuncia
   con una pequeña tarjeta en la esquina inferior derecha; se desvanece a los
   pocos segundos y, al hacer clic en ella, se abre la ventana de ajustes. Windows
   11 mantiene los iconos nuevos del área de notificación en el menú de
   desbordamiento (el `^` que hay junto al reloj): arrastra el icono a la barra de
   tareas, o actívalo en **Settings → Personalization → Taskbar → Other system
   tray icons**, para que siga visible.
3. En cualquier aplicación, **arrastra el ratón sobre el texto** (o **haz doble
   clic en una palabra**) para seleccionarlo.
4. Glossy muestra un pequeño icono de Glossy bajo la selección. No se traduce
   nada —y no se consume nada de la cuota del día— hasta que **hagas clic en ese
   icono**, así que una palabra por la que solo has pasado por encima no cuesta
   nada. Al hacer clic en cualquier otro sitio el icono desaparece.
5. La tarjeta aparece debajo de la selección. Arrastra su encabezado para moverla,
   usa los botones para copiar el resultado o abrir los ajustes, o haz clic en
   cualquier otro sitio para descartarla. La fila que hay bajo el encabezado
   muestra el par de idiomas: pasa el cursor por encima y elige cualquiera de los
   dos lados en los desplegables para volver a traducir con ese idioma, o pulsa el
   botón `⇄` para traducir el resultado de vuelta al idioma del que procede. El
   par se restablece a *detectar el idioma de origen y usar el destino
   configurado* en cada nueva selección; el destino que elijas a mano se recuerda
   y pasa a ser ese destino configurado, de modo que la siguiente selección ya
   empieza en él, mientras que el origen elegido solo vale para la tarjeta que
   tienes delante.
6. Las selecciones más cortas que el mínimo configurado (2 caracteres por
   defecto) se ignoran, y un arrastre que empieza o termina sobre el propio
   emergente nunca activa una traducción.

La tarjeta de una palabra muestra los símbolos fonéticos, las categorías
gramaticales con sus definiciones, un ejemplo sencillo y, en la misma tarjeta,
las formas de la palabra, las palabras que significan más o menos lo mismo, la
frase de la que se tomó la selección con una traducción de esa frase, y un botón
de pronunciación para cada lado. Una frase o un párrafo se muestra como la
traducción misma, con el original encima (hasta cuatro líneas) y, si el párrafo
es largo, una vista frase por frase debajo.

Pulsa el atajo de teclado global (**Ctrl+Alt+C** de forma predeterminada) para
traducir el contenido del portapapeles en lugar de seleccionar algo.

El emergente es una ventana que no se activa y que está siempre encima: nunca
roba el foco del teclado a la aplicación en la que estás leyendo. Se mantiene
dentro del área de trabajo del monitor en el que está el cursor y salta por
encima de él cuando no hay espacio debajo.

### Ajustes

| Opción | Significado |
| --- | --- |
| Interface language | `Follow Windows`, `简体中文` o `English`. Cambia al instante la ventana de ajustes y el emergente. |
| Enable selection translation | Interruptor principal. Al desactivarlo se pausa de inmediato la captura global de selecciones. |
| Start Glossy with Windows | Añade una entrada `--autostart` a `HKCU\...\Run`, de modo que Glossy ya está esperando en el área de notificación tras iniciar sesión. Iniciado así, no muestra la tarjeta "Glossy is running". |
| Translate when the mouse drags across text | Activa el gesto de arrastre. |
| Translate a word on double click | Activa el gesto de doble clic. |
| Put the clipboard back after reading a selection | Restaura el contenido anterior del portapapeles después de que Glossy haya copiado la selección. |
| Show the original text in the popup | Oculta la línea de origen de la tarjeta cuando está desactivado. Una selección larga se recorta a cuatro líneas y se desplaza dentro de su bloque, así la traducción conserva su espacio. |
| Convert units and currency | Cuando la traducción sigue midiendo algo en una unidad que tu idioma de destino no usa —pies, libras, °F, una cantidad extranjera—, la tarjeta añade debajo el valor convertido y el factor de conversión (`12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`). Los números se leen de la traducción, porque es el traductor quien decide si un símbolo es una unidad y siempre la escribe en un idioma que las tablas conocen; si la traducción no contiene ninguna, se lee el original. Longitud, masa, volumen, velocidad, superficie y temperatura se convierten dentro de su categoría, y la unidad de destino es la que escribiría una persona; la moneda de destino sigue al idioma de destino (`zh-CN` → CNY, `en` → USD). Los tipos de cambio los sirve el servidor de Glossy —que lee `open.er-api.com` y, si no responde, recurre al BCE (`api.frankfurter.app`)—; la aplicación pregunta directamente a esos proveedores solo cuando el servidor no puede contestar, y en cualquier caso el tipo de cambio se guarda en caché durante seis horas; todo lo demás está integrado en la aplicación. Desactivado significa que no hay conversión ni petición de tipos de cambio. |
| Show the sentence a word was selected from | La tarjeta añade la frase de la que se tomó la selección, junto a una traducción de esa frase. Desactivado, la palabra se traduce por sí sola. |
| Pair the original and the translation sentence by sentence | Un párrafo cuya traducción se divide de otra manera se lista como filas alineadas en lugar de como un bloque. Una traducción que vuelve en una sola fila se deja como párrafo normal. |
| Draw a compact card, without the extras | Conserva la traducción, los símbolos fonéticos y los significados, y deja fuera el ejemplo, las formas, los sinónimos, la frase de contexto, la vista frase por frase y las conversiones. |
| Speaking rate | Velocidad con la que los botones de pronunciación leen un texto: `Slow`, `Normal`, `Fast` o `Very fast` (SAPI de Windows). Las voces son las que ya tiene Windows, y el idioma del texto elige la voz. |
| Shortest selection to translate | Número de caracteres por debajo del cual se ignora una selección (`1`–`40`, por defecto `2`). |
| Global hotkey | Combinación que traduce el contenido del portapapeles, p. ej. `Ctrl+Alt+C`. Vacía el campo para desactivarla. La línea que hay bajo el campo muestra la combinación registrada o por qué Windows la rechazó. |
| Never translate in these programs | Una lista de nombres de proceso (`idea64.exe`, `mstsc`) en los que se omite la captura de selecciones. Añade uno escribiéndolo (el sufijo `.exe` es opcional: el botón `Add` lo normaliza), eligiéndolo en el desplegable de programas en ejecución o pulsando `Pick with the mouse` y haciendo clic en la ventana que quieras ignorar. Cada entrada tiene una `×` para eliminarla; los duplicados se descartan sin distinguir mayúsculas y minúsculas. |
| Only translate these source languages | Una lista de idiomas, por ejemplo `English` y `日本語`. Vacía significa que cualquier idioma abre una traducción. Una selección cuyo idioma no se puede determinar —texto mezclado, dígitos, una palabra que comparten media docena de idiomas— pasa siempre, así que una suposición equivocada nunca se traga tu selección. La lista se guarda como un array `sourceLangs`. |
| Colours | `system` sigue la preferencia de Windows para modo claro u oscuro; `light` y `dark` fuerzan un esquema en ambas ventanas. |
| Text size | Multiplicador de todos los tamaños de texto del emergente (`90 %`–`150 %`). |
| Width | Ancho de la tarjeta emergente (`300`–`520` px CSS). |
| Opacity | Cuán transparente es la tarjeta emergente (de `100 %` opaca hasta `50 %`). La tarjeta se atenúa mientras el texto sigue siendo legible sobre lo que haya detrás. |
| Close by itself | Segundos antes de que el emergente se oculte por sí solo; `Never` lo mantiene abierto hasta que lo descartes. |
| Close the popup right after the translation is copied | Oculta la tarjeta en cuanto se ha usado el botón de copiar. |
| Target language | Idioma al que se traduce el resultado. |
| History | Cuántas traducciones terminadas recordar (`Off` hasta `The last 500`, por defecto 50). La lista que hay bajo el selector conserva el original, la traducción, el proveedor y la hora; `Search` filtra ambos textos, al hacer clic en una entrada se muestra de nuevo en la tarjeta flotante (sin una segunda llamada al proveedor), y cada entrada tiene un botón de copiar y otro de eliminar. `Forget everything` vacía la lista. El archivo está en `%APPDATA%\com.glossy.translator\history.json`. |
| Settings file | `Export…` escribe `Documents\glossy-settings.json`; `Import…` vuelve a leer en la aplicación un archivo que elijas. El archivo contiene las opciones y nada secreto —ya no queda ningún campo de claves, así que una exportación nunca pregunta por una—. Una importación se valida mediante `sanitized()`. |
| Updates | `Check for a new version when Glossy starts` consulta GitHub Releases en cada arranque (desactivado por defecto). `Check now` busca de inmediato y dice qué versión está esperando, y `Download and restart` la instala. Una compilación sin clave de firma de actualizaciones —que es toda compilación hasta que exista el par de claves de publicación— oculta los botones y lo indica. |
| Servicio de traducción | Qué servicio traduce: `cloud-baidu` (el servidor del propio Glossy, traduciendo con Baidu —no hay nada que configurar), `cloud-youdao` (ese mismo servidor, traduciendo con Youdao —tampoco hay nada que rellenar) o `google` (el punto de conexión público gratuito, sin clave). Una instalación nueva empieza en `cloud-baidu`. Se guarda como `channel` + `cloudProvider` + `cloudVendor` / `provider`, ver más abajo. |
| Ask another service when the chosen one fails | El servicio elegido arriba se intenta siempre primero y no se puede mover; las entradas que hay debajo se consultan en orden cuando falla, se limita o no devuelve nada, y pueden reordenarse con las flechas. Apagar el interruptor significa que un fallo se muestra como un fallo. La tarjeta indica qué servicio respondió cuando no fue el solicitado. |
| Línea de cuota (`cloud-baidu`, `cloud-youdao`) | Se muestra solo para las dos entradas que responden desde el servidor: cuánto queda de la cuota de hoy —o el motivo por el que no se pudo contactar con el servidor— junto a un botón `Check again`. La dirección forma parte de la compilación en lugar de ser un campo, para que nadie pueda romper el servicio que no necesita configurar nada; consulta [`server/`](./server/README.md) para mantener un despliegue propio. |

El atajo de teclado acepta `Ctrl`/`Control`, `Alt`, `Shift`, `Win`/`Meta` más una
tecla: una letra, un dígito, `F1`–`F24`, `Space`, `Enter`, `Tab`, `Esc`,
`Backspace`, `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown` o una tecla
de flecha. Se necesita al menos un modificador. Cuando Windows rechaza la
combinación —normalmente porque otro programa ya la tiene—, el motivo se muestra
bajo el campo y el atajo permanece inactivo hasta que se corrija el ajuste.

Si el portapapeles no contiene ningún texto, el atajo recurre a copiar la
selección actual, de modo que "seleccionar el texto y pulsar el atajo" también
funciona.

### Proveedores

Ninguna de las tres entradas del desplegable necesita tu propia cuenta: `google`
usa el punto de conexión público, y las dos
entradas que responden desde el servidor lo hacen con las cuentas que tiene el
proyecto, así que ninguna clave llega al cliente. Esas dos solo indican con qué
proveedor debe traducir el servidor, y si ese no responde se usa otro motor.

| Proveedor | Coste | Notas |
| --- | --- | --- |
| `cloud-baidu` | nada que rellenar | **Baidu Translate**: un servidor desplegado desde [`server/`](./server/README.md) traduce con la cuenta del propio proyecto, y en la red se llama `vendor: "baidu"`. La aplicación solo envía el texto más un id de instalación, las credenciales se quedan en el servidor y funciona desde China continental. La cuota diaria se cuenta por dispositivo, por dirección y en total, y la ventana de ajustes muestra lo que queda; usar tu propio despliegue es cambiar una línea de la dirección dentro de la compilación. |
| `cloud-youdao` | nada que rellenar | **Youdao Translate**: otra vez el servidor de Glossy, esta vez con 有道智云 (`vendor: "youdao"`). Igual: sin clave, nada que rellenar, y con motor de reserva cuando Youdao no responde. |
| `google` | gratis, sin clave | Punto de conexión público `translate.googleapis.com`. Bloqueado en algunas redes, incluida buena parte de China continental. Siempre se consulta con el id de cliente `dict-chrome-ex`; el id `gtx`, limitado, solo se usa como alternativa. |

Cada motor traduce un conjunto distinto de idiomas: una cuenta estándar de Baidu
rechaza ocho de los 31 que ofrece Glossy (`uk`, `tr`, `hi`, `id`, `ms`, `he`, `no`,
`sk`), mientras que Youdao y Google los admiten todos. Las dos barras de idioma
solo listan los idiomas del motor que está usando la tarjeta, así que uno que
rechazaría no se ofrece nunca y nunca se envía.

Los símbolos fonéticos y las definiciones de palabras sueltas provienen de
`api.dictionaryapi.dev`, que es gratuito, no necesita clave y es accesible desde
China continental. Solo se pregunta al punto de conexión de Google por los
detalles que el proveedor seleccionado no haya devuelto.

### Cuotas gratuitas y uso comercial

Los niveles gratuitos se diferencian en lo que permiten. Lo que prohíben es
entregar la cuota en bruto —una clave, una interfaz— a otras personas o
revenderla; quedarse con la credencial en un servidor propio y dejar que la
aplicación llame a ese servidor es un uso de la cuota por parte del titular, y por
eso las dos entradas que responden desde el servidor las intermedian así:

- **`cloud-baidu` / `cloud-youdao`** — traducen con una cuenta de LLM que paga el proyecto y, si
  no hay tal
  cuenta configurada, recurren a las credenciales de Baidu, así que sus llamadas son
  de pago y las condiciones de arriba no les afectan. Ningún usuario ve ni tiene
  nunca una clave de proveedor: la aplicación envía el texto más un id de
  instalación, y el servidor mide una cuota diaria por dispositivo y dirección.
- **Zhipu** — 用户协议 §非付费功能 licencia los modelos gratuitos solo para uso
  *非商业的、个人研究学习*. Está bien para uso personal; no lo está para un producto
  publicado o de pago.
- **ModelScope API-Inference** — explícitamente 非商业化, 非盈利.
- **Aliyun 机器翻译** — la cuota mensual gratuita es explícitamente 仅适用客户试用场景.
- **Tencent 腾讯云 TMT** — la cuota gratuita de 5 M de caracteres al mes todavía se
  anuncia, pero el servicio ya no ofrece traducción de texto: en las versiones de
  2026-03 y 2026-07 se eliminaron las acciones `TextTranslate`, `TextTranslateBatch`,
  `ImageTranslate`, `LanguageDetect` y `SpeechTranslate`, y la 概览 de la API solo
  incluye `ImageTranslateLLM`. La página 计费概述 está desactualizada, así que no
  cuentes con ella.
- **Baidu 翻译开放平台** — 50 000 caracteres al mes sin verificar, 1 M con
  verificación personal y 2 M con verificación de empresa. Sus condiciones no dicen
  nada sobre el uso comercial, pero el 服务协议 prohíbe que un *programa cliente*
  guarde en caché los datos de traducción de Baidu y prohíbe su reventa. Glossy no
  guarda nada en caché, así que esto no le afecta; un fork que añada una caché
  tendría que volver a revisarlo. Las cuotas están limitadas por QPS a 1 / 10 / 100.
- **Volcengine 火山引擎** — 2 M de caracteres al mes, pero la incorporación requiere
  un contrato de ventas.
- **NiuTrans 小牛翻译** — 200 000 caracteres al día tras registrarte; las condiciones
  comerciales no están publicadas.
- **SiliconFlow** — `tencent/Hunyuan-MT-7B` es gratuito, pero las condiciones de la
  plataforma no dicen nada sobre los modelos gratuitos y están restringidas a fines
  comerciales internos.
- **Totalmente sin conexión** — los pesos de `Opus-MT` / `Argos Translate` son
  CC-BY-4.0/Apache-2.0 (uso comercial permitido, ~83 MB int8, ~300 MB de RAM).
  `NLLB-200` es CC-BY-NC-4.0 y **no** debe distribuirse.

Los ajustes se guardan como JSON en `%APPDATA%\com.glossy.translator\settings.json`.
El archivo empieza por `formatVersion`, que indica en qué forma está escrito; desde
`1.3.0` las claves solo se añaden y los nombres de los comandos IPC no cambian, así
que un archivo escrito por una compilación anterior se sigue cargando. Un archivo que
esta compilación no puede leer —uno que no sea un objeto JSON, o uno escrito por una
versión de formato más nueva— se copia a `settings.backup-<segundos unix>.json` a su
lado y la aplicación arranca con los valores por defecto, en lugar de sobrescribirlo.
El indicador `firstRun` de ese archivo registra que la ventana de bienvenida ya se
mostró; si lo eliminas (o eliminas el archivo entero), la ventana vuelve a aparecer
en el siguiente arranque.
Las credenciales de traducción viven en un mapa `credentials` indexado por
proveedor, y los valores `apiKey`/`appId` de un único proveedor se migran a ese
mapa en el primer arranque. Todos los valores de ese mapa se cifran con DPAPI de
Windows (`CryptProtectData`, ámbito del usuario actual) antes de escribir el
archivo, así que este contiene `"apiKey": "dpapi:AQAAANCM…"` en lugar de la clave
en sí, y solo el inicio de sesión de Windows que la introdujo puede leerla. Un
archivo escrito por una versión anterior se protege la primera vez que arranca
esta compilación; un valor que pertenece a otro inicio de sesión o a otro equipo
no se puede desbloquear y se descarta, así que hay que volver a introducir la clave.
La lista de programas ignorados se guarda como un array `ignoredApps`; una cadena
heredada separada por comas se acepta y se divide al cargar.
El servicio que traduce se guarda en la antigua forma de dos partes: `channel`
(`cloud` o `api`) elige entre las dos entradas que responden desde el servidor y
el punto de conexión gratuito de Google,
y `provider` nombra el servicio dentro de ella — `baidu` para las entradas servidas
por el servidor (un valor que el canal ignora, conservado porque el campo forma
parte del formato del archivo) o `google`. `cloudProvider` (`builtin` o `local`, y el
segundo se lee ahora como el motor integrado, ya que el desplegable no lo ofrece) y `cloudVendor`
(`baidu` o `youdao`) es el proveedor que piden
las entradas servidas por el servidor. Una instalación nueva empieza en `baidu`. El
desplegable hace ese mapeo al leer y al
escribir, así que un archivo escrito por una versión anterior sigue funcionando:
uno sin `channel` se lee como el canal API que ya era, uno que no lo tenga se
escribe con `cloud`, un `cloudVendor` que no sea `youdao` se lee como Baidu, y un
`provider` que esta compilación ya no ofrece —Baidu con clave propia, Zhipu, DeepL,
OpenAI— se lee como la entrada Baidu integrada y se sustituye la próxima vez que se
escriba el archivo.

### Traducir dentro de la aplicación

La parte superior de la ventana de ajustes contiene la misma función de traducción
que el emergente: un cuadro de texto para escribir o pegar texto. Selecciona una
palabra, una frase o una oración en ese cuadro —o pulsa **Translate** (Ctrl+Enter)
para usar todo el texto— y el resultado aparece justo debajo, en una tarjeta
idéntica a la del emergente: la barra de idioma de origen y destino con su botón de
intercambio, el botón de copiar y la misma representación de palabra u oración. El
texto pegado se traduce en cuanto llega, ya sea mediante **Paste** o con Ctrl+V;
**Clear** vacía el cuadro y retira la tarjeta. El interruptor que hay en el
encabezado activa o desactiva la conversión de unidades para toda la aplicación: es
el mismo ajuste que el del panel **Units** de más abajo, y es el único sitio donde
las conversiones de una tarjeta se pueden cambiar al momento. La barra de idioma
sigue las reglas del emergente, así que el origen empieza en *Detect language*, el
destino sigue al idioma configurado y una nueva selección restablece el par (el
origen elegido no se guarda nunca; el destino sí, y pasa a ser el idioma
configurado). Pulsa **Show in floating popup** para abrir el
emergente real con la selección actual (o con todo el texto cuando no hay nada
seleccionado): así se prueba el emergente sin el enganche global.

### Solución de problemas

- **"Google Translate is rate limiting requests right now"** — Google respondió `429`, algo
  que les ocurre a los clientes HTTP de escritorio en el punto de conexión público aunque un
  navegador o `curl` sigan funcionando. Glossy reintenta primero con un segundo id de cliente;
  si el mensaje persiste, espera un minuto o cambia a cualquiera de las otras dos entradas
  del desplegable.
- **"Could not reach Google Translate"** — el proveedor gratuito `google` llama a
  `translate.googleapis.com`, que está bloqueado en algunas redes (incluida buena parte de
  China continental). La tarjeta ofrece un botón de reintento; si sigue fallando, cambia a
  `cloud-baidu` o `cloud-youdao` (nada que rellenar).
- **"The free cloud translation quota for today is used up."** — las dos entradas que
  responden desde el servidor cuentan
  los caracteres que traducen por dispositivo, por dirección y en total, y alguno de esos
  contadores ha llegado a su tope diario. Vuelve a empezar a las 00:00 UTC. Usa otra entrada
  del desplegable,
  o despliega tu propio servidor desde [`server/`](./server/README.md) y apunta la aplicación
  a él.
- **"Baidu rejected the APP ID" / "Baidu rejected the signature"** — las dos mitades del par
  de credenciales se han intercambiado o están mal escritas. `baidu` necesita el APP ID en el
  primer cuadro y la 密钥 en el segundo; la clave nunca se envía a Baidu, solo firma la petición.
- **"Baidu rejected this computer's IP address"** — la lista de IP permitidas de la aplicación
  en la consola de Baidu está rellenada. Bórrala o añade la dirección desde la que se conecta
  Glossy.
- **El emergente muestra "Paused"** aunque las traducciones están activadas: el interruptor
  principal de la ventana de ajustes está desactivado, o no se pudo leer el archivo de ajustes
  al arrancar.

## Cómo se captura la selección

Glossy instala un enganche `WH_MOUSE_LL` y vigila los pares de pulsación y
liberación del botón izquierdo. La función de retorno del enganche solo registra
coordenadas; un hilo de trabajo decide si el gesto fue un arrastre o un doble clic,
copia la selección con `Ctrl+C` (enviando `Ctrl+Insert` cuando la ventana en primer
plano se ejecuta con privilegios elevados), restaura el portapapeles si se ha
pedido y, por último, indica a la ventana emergente qué debe mostrar. El texto
capturado solo se envía al servicio de traducción que hayas elegido — para `google`
es un tercero, y para las dos entradas
que responden desde el servidor es el servidor que mantiene
el proyecto ([`server/`](./server/README.md)).

Un gesto solo abre la tarjeta cuando de verdad ha seleccionado algo. Primero se
comprueba el punto donde se suelta el botón: un doble clic sobre una superficie del
shell —un icono del escritorio, la barra de tareas, el botón Inicio— se ignora, así
que lo último que el shell dejó en el portapapeles nunca se traduce. El texto
copiado tiene que contener una letra y no puede parecer la ruta de un archivo o
carpeta, y copiar archivos en el Explorador cuenta como que no hay selección.
Con `Put the clipboard back after reading a selection` activado, el contenido
anterior se restaura una vez que la aplicación de la que se copió ha dejado de
escribir —se vuelve a comprobar hasta 80 ms—, así que una escritura tardía no
puede dejar ahí el texto copiado. Glossy recuerda el número de secuencia de cada
escritura que hace él mismo y las omite mientras espera a que llegue el `Ctrl+C`,
así que devolver el portapapeles a su sitio nunca se puede leer como una selección
propia.

## Desarrollo

### Qué necesitas

| Herramienta | Versión | Notas |
| --- | --- | --- |
| Node.js | 18 o posterior | solo para la CLI de Tauri; el frontend no tiene empaquetador |
| Rust | estable, `1.77` o posterior | el destino GNU `x86_64-pc-windows-gnu`; consulta más abajo |
| `rustfmt` + `clippy` | la misma cadena de herramientas | `rustup component add rustfmt clippy` |
| MinGW-w64 | 8.1 o posterior | el enlazador del destino GNU; en el `PATH` como `gcc.exe` |

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu --component rustfmt --component clippy
rustup default stable-x86_64-pc-windows-gnu
npm.cmd install          # npm.ps1 is blocked by the default execution policy
npm.cmd run tauri dev    # dev build with hot reload of src/
npm.cmd run tauri build  # release build (installer + MSI + .exe)
```

`scripts\dev.cmd` es ese comando de desarrollo dentro de un archivo que se abre con doble
clic: comprueba que `cargo` esté en el `PATH`, instala la CLI de Tauri si falta
`node_modules`, se niega a arrancar mientras haya otro Glossy en ejecución —un segundo
instalaría un segundo gancho del ratón— y deja la consola abierta con el registro del
desarrollo. Ejecutar una vez
`powershell -ExecutionPolicy Bypass -File scripts\make-shortcut.ps1` deja un acceso directo
en el escritorio.

`npm.cmd run icon` regenera `src-tauri/icons` a partir de `assets/`.

`npm.cmd run tauri build` escribe el instalador NSIS en
`src-tauri\target\release\bundle\nsis\Glossy_<version>_x64-setup.exe`, el MSI en
`src-tauri\target\release\bundle\msi\Glossy_<version>_x64_en-US.msi` y el binario
independiente en `src-tauri\target\release\Glossy.exe`. Los dos empaquetadores
necesitan conexión de red la primera vez que se ejecutan, para descargar sus
herramientas — WiX para el MSI, los complementos de NSIS para el instalador — en
`%LOCALAPPDATA%\tauri`.

El zip portátil no es un destino de Tauri, porque Tauri no tiene ninguno:
`scripts\release.ps1` lo empaqueta a partir del binario de publicación y del
`WebView2Loader.dll` que está a su lado y que la compilación GNU carga en tiempo de
ejecución.

`scripts\release.ps1` envuelve la compilación de publicación: compila y prepara el
instalador, el MSI y el zip portátil
en `release\v<version>\` junto con una suma de comprobación y el texto para la
descripción de la publicación. Ejecútalo como `powershell -ExecutionPolicy Bypass
-File scripts\release.ps1` — los scripts están bloqueados por la directiva de
ejecución predeterminada, la misma razón por la que arriba se usa `npm.cmd`.
Consulta [release/README.md](./release/README.md).

### Comprobaciones

```powershell
powershell -ExecutionPolicy Bypass -File scripts\version.ps1 -Check   # all version numbers agree
node --test                                                          # 202 frontend tests
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked                                                  # 192 Rust tests
cd ..\server
npm test                                                             # 74 server tests
```

`npm test` ejecuta el mismo comando `node --test`. Las pruebas del frontend cargan
`src/js/i18n.js` y `src/js/render.js` en un DOM mínimo (consulta `tests/helpers/`)
y cubren los diccionarios, la sustitución de marcadores y el renderizador de
tarjetas. Ejecuta la forma de directorio simple en Node 24/Windows: `node --test
tests` y `node --test .` no resuelven allí los archivos de prueba.

La suite de `server/` no necesita nada instalado: las llamadas al proveedor se
inyectan, así que las reglas de cuota, la firma de la petición y el análisis de la
dirección del llamante están cubiertos sin red.

La versión vive en seis sitios (consulta `scripts\version.ps1`), así que no los
edites nunca a mano: `tauri.conf.json` es la fuente autorizada,
`scripts\version.ps1 -Set 0.1.2` la escribe en todos los demás y
`scripts\version.ps1 -Get` la imprime para scripts como `release.ps1`, que se niega
a compilar cuando los números no coinciden. La misma comprobación se ejecuta en CI,
así que un incremento olvidado hace fallar la compilación.

[`.github/workflows/ci.yml`](./.github/workflows/ci.yml) ejecuta las cuatro
comprobaciones más la suite de pruebas del frontend en `windows-latest` en cada
push y cada pull request.

### Actualizaciones

Glossy pide a
`https://github.com/SpencerZXWu/Glossy/releases/latest/download/latest.json` una
versión más reciente. Una actualización solo se acepta si lleva una firma hecha con
la clave de publicación:

```powershell
cargo tauri signer generate -w $env:USERPROFILE\.tauri\glossy.key   # once
```

Pon la clave pública impresa en `plugins.updater.pubkey` de
`src-tauri\tauri.conf.json` (el marcador de posición que hay allí es lo que hace
que la ventana de ajustes diga que la compilación no puede actualizarse) y exporta
la privada antes de compilar una publicación:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $env:USERPROFILE\.tauri\glossy.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<the password you chose>'
```

`bundle.createUpdaterArtifacts` en `tauri.conf.json` permanece desactivado hasta que
exista ese secreto: sin él el empaquetador falla, y con él la compilación también
escribe el instalador firmado y el `latest.json` del que se lee una actualización.
Mantén la clave privada fuera del repositorio: es lo único que permite reemplazar
una publicación.

### Lista de comprobación de regresión manual

Ejecuta esto antes de etiquetar una publicación. Todos los puntos fueron un error
real en algún momento y ninguno está cubierto por las pruebas automatizadas.

| # | Qué hacer | Qué debe ocurrir |
| --- | --- | --- |
| 1 | Iniciar Glossy con un perfil limpio | Se abre la ventana de ajustes, no hay nada preseleccionado por accidente y la línea de estado indica que la captura está en marcha |
| 2 | Cerrar la ventana de ajustes y volver a iniciar Glossy | No aparece una segunda ventana ni un segundo enganche del ratón: la notificación dice que Glossy ya se está ejecutando y el icono de la bandeja vuelve a abrir la ventana |
| 3 | Marcar "Start Glossy with Windows", reiniciar e iniciar sesión | No parpadea ninguna ventana de consola, no aparece la tarjeta "running" y el icono de la bandeja está ahí |
| 4 | Desmarcarlo y reiniciar | Glossy no arranca (`reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Glossy` no encuentra nada) |
| 5 | Seleccionar una palabra en Notepad, un navegador, Word y una terminal | El emergente aparece bajo el cursor cada vez, con los símbolos fonéticos y las definiciones de la palabra |
| 6 | Seleccionar una oración en cada uno de los mismos programas | El emergente muestra solo la traducción, sin símbolos fonéticos ni acepciones |
| 7 | Arrastrar el emergente a cada monitor y a cada borde de la pantalla | La tarjeta se mantiene dentro del área de trabajo en todos los monitores y con escalas del 100 %, 125 % y 150 % |
| 8 | Fijar el emergente, hacer clic en otro punto del escritorio y esperar más allá del tiempo de cierre automático | La tarjeta permanece abierta hasta que se suelte la fijación o se use la × |
| 9 | Hacer clic fuera de una tarjeta sin fijar | Se cierra en cuanto el clic cae fuera |
| 10 | Cambiar Windows entre modo claro y oscuro y luego forzar cada esquema en los ajustes | Ambas ventanas siguen la elección, con texto y bordes legibles en las dos |
| 11 | Traducir con cada uno de los tres servicios —el servidor de Glossy con Baidu, ese mismo servidor con Youdao y el punto de conexión público de Google— y luego una vez sin red | Un resultado para los tres; un error legible con un botón de reintento cuando no se puede contactar con ninguno |
| 12 | Añadir un programa en ejecución a la lista de ignorados, traducir dentro de él y luego quitarlo | No aparece nada mientras está en la lista, y el emergente vuelve en cuanto se quita |
| 13 | Pulsar el atajo de teclado global con texto en el portapapeles y luego con el portapapeles vacío y una selección | La traducción se abre junto al cursor en el primer caso y se usa la selección actual en el segundo |
| 14 | Exportar los ajustes, cambiar un valor en el archivo y luego importarlo | La exportación no tiene bloque `credentials`; la importación aplica el valor editado y deja lo demás intacto |
| 15 | Poner el historial en `The last 50`, traducir 60 textos y luego ponerlo en `Off` | La lista guarda 50, la búsqueda los filtra, al hacer clic en uno se reabre en la tarjeta y `Off` vacía el archivo |
| 16 | Cambiar las transparencias, el tamaño de fuente y el ancho, y reiniciar | El emergente conserva los valores elegidos |
| 17 | Abrir la sección de actualizaciones | Esta compilación, sin clave de firma, oculta los botones de comprobación y dice que la compilación no puede actualizarse; una vez que existe un par de claves, `Check now` informa de la versión en ejecución o de la que está esperando |
| 18 | Pulsar un botón de lectura, volver a pulsarlo mientras está encendido y luego empezar otra lectura y cerrar la tarjeta | El botón se convierte en un cuadrado de detener mientras se lee su texto y vuelve a ser un altavoz cuando la voz se detiene; la segunda pulsación detiene la lectura, cerrar la tarjeta también la detiene y ninguna lectura se solapa con la tarjeta siguiente |

### Notas sobre la cadena de herramientas (Windows, cadena GNU)

El proyecto se compila con el destino GNU de Rust (`x86_64-pc-windows-gnu`), que no
necesita una instalación de Visual Studio, pero hay tres peculiaridades:

- El crate `windows` hace crecer la tabla de importaciones más allá de lo que
  aceptan los binutils antiguos de MinGW. Por eso `src-tauri/Cargo.toml` declara
  `[lib] crate-type = ["rlib"]` y el crate de la aplicación se enlaza solo como
  binario.
- `cargo test` necesita también el archivo de recursos generado (icono, información
  de versión y el manifiesto de common-controls v6) en el arnés de pruebas; de lo
  contrario, el arnés no arranca con `0xC0000139` antes de `main`.
  `src-tauri/build.rs` añade un `cargo:rustc-link-arg` simple para `libresource.a`
  para llegar a todos los objetivos de prueba. Los scripts de compilación no
  distinguen entre objetivos binarios y de prueba, así que los binarios reciben el
  archivo dos veces y GNU ld imprime avisos `.rsrc merge failure: multiple
  non-default manifests`; el `.exe` resultante sigue llevando el manifiesto y
  funciona.
- El destino GNU enlaza `WebView2Loader.dll` de forma dinámica (MSVC lo integra en
  el ejecutable), así que el instalador también tiene que llevarse el DLL.
  `src-tauri/build.rs` copia el que compila `webview2-com-sys` en
  `src-tauri/resources/`, `bundle.resources` lo coloca junto a `glossy.exe` en la
  aplicación instalada y `scripts/release.ps1` se niega a preparar una versión sin
  él. La copia que deja `tauri-build` en `target/<profile>/` solo sirve para
  ejecutar desde el directorio de compilación.

```powershell
cd src-tauri
cargo test   # 204 tests; see "Checks" above for lint and format runs
```

### Política de seguridad de contenido

`app.security.csp` en `src-tauri/tauri.conf.json` ya no es `null`, sino una lista
blanca explícita:

```
default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;
font-src 'self'; connect-src 'self' ipc: http://ipc.localhost; media-src 'none';
object-src 'none'; frame-src 'none'; frame-ancestors 'none'; base-uri 'none';
form-action 'none'
```

Las ventanas no cargan nada remoto — ni CDN, ni fuentes remotas, ni imágenes
remotas — así que una página que intente salir a la red es rechazada. Dos entradas
existen para el framework y no para la aplicación: `connect-src` nombra `ipc:` y
`http://ipc.localhost`, que es como una ventana habla con el lado de Rust en
Windows, y los hashes que necesitan `script-src` y `style-src` los añade Tauri por
su cuenta. `img-src` permite `data:` para poder incrustar un icono sin tocar la
política. Las peticiones de traducción no se ven afectadas: salen de Rust, no de la
ventana.

## Estructura

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
  images/logo.png        the app icon, shown as the brand mark in both windows
tests/                   node --test suite for i18n.js and render.js
src-tauri/src/
  main.rs  lib.rs        window setup, Tauri commands
  selection.rs           what a gesture meant: click, drag, double click, capture
  classify.rs            word/phrase vs. sentence detection
  text.rs                cleanup of the text a selection comes back as
  morphology.rs          the forms of an English word (run, runs, running, ran)
  context.rs             the sentence a selected word stands in, via UI Automation
  units/                 unit and currency conversion for the card
  translate/             google and cloud providers, word dictionary,
                         and the languages each of them translates
  platform/
    mod.rs               what the backend may assume about an operating system
    windows/             the modules that talk to Win32: clipboard, console, desktop,
                         hotkey, input, input_hook, instance, secrets, speech, uia
  popup.rs               placement/clamping geometry
  surface.rs             Mica backdrop and title bar colour
  history.rs             the translation store behind the History panel
  autostart.rs           the login item and the --autostart marker
  updater.rs             the release feed behind the Updates section
  notice.rs              start card placement and lifetime
  tray.rs                notification area icon: open the window, quit
  settings.rs  state.rs
server/
  src/                   the proxy: quota rules, the providers, the two hosts
  test/                  node --test suite for the rules and the signatures
  README.md              how to deploy it (Cloudflare Worker or Tencent SCF)
scripts/
  version.ps1            the version number, in one place
  release.ps1            release build + staging for a GitHub release
.github/workflows/
  ci.yml                 format, lint, test and version check on every push
release/
  v<version>/            installer, RELEASE_NOTES.md and checksum of a release
```

## Hoja de ruta

[ROADMAP.md](./ROADMAP.md) contiene las publicaciones (`v1.0.0` a `v2.0.0`) con
sus criterios de aceptación, la política de versiones, los riesgos conocidos y el
proceso de publicación. Cada publicación se corresponde con un hito de GitHub del
mismo nombre.

[CHANGELOG.md](./CHANGELOG.md) enumera lo que se incluyó en cada versión publicada.

## Code signing policy · Política de firma de código

Los instaladores de Windows de la [página de publicaciones](https://github.com/SpencerZXWu/Glossy/releases)
llevan la firma gratuita de [SignPath.io](https://about.signpath.io) y el certificado de la
[SignPath Foundation](https://signpath.org) (free code signing provided by SignPath.io,
certificate by SignPath Foundation), de modo que Windows muestra un editor en lugar de
*editor desconocido*. Las versiones publicadas antes de ese acuerdo no están firmadas; con
un certificado propio, `scripts/release.ps1 -Sign` firma una compilación local, como describe
[release/README.md](./release/README.md).

Roles: [@SpencerZXWu](https://github.com/SpencerZXWu) es el propietario de este repositorio y
su único autor, revisor y aprobador: cada confirmación y cada publicación las revisa y las
aprueba ese responsable, cuya cuenta de GitHub exige autenticación de dos factores. Solo se
firman artefactos compilados desde una confirmación de este repositorio.

Lo que la aplicación envía y a dónde se describe en [PRIVACY.md](./PRIVACY.md).

## Licencia

[MIT](./LICENSE) © 2026 Spencer Wu
