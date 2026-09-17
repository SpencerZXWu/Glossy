# Glossy

A lightweight desktop translation popup for Windows. Select text in any
application with the mouse and Glossy shows a small floating card with the
translation under the cursor.

- **Word or short phrase** → phonetic symbols, part of speech, definitions and one example.
- **Sentence or paragraph** → a smooth translation into your target language.
- The source language is detected automatically.

## Using it

1. Start Glossy: the settings window opens and Glossy starts listening.
2. In any application, **drag across text** (or **double click a word**) to select it.
3. The popup appears below the cursor. Drag its header to move it, use the buttons
   to copy the result or to close it, or click anywhere else to dismiss it.
   The row under the header shows the language pair: hover it and pick either side
   from the dropdowns to translate again with that language, or press the `⇄`
   button to translate the result back into the language it came from. The pair
   resets to *detect the source and use the configured target* for every new
   selection.
4. Selections shorter than the configured minimum (2 characters by default) are
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
| Translate when the mouse drags across text | Enables the drag gesture. |
| Translate a word on double click | Enables the double-click gesture. |
| Put the clipboard back after reading a selection | Restores your previous clipboard content after Glossy copied the selection. |
| Show the original text in the popup | Hides the source line in the card when off. |
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
Translation credentials live in a `credentials` map keyed by provider
(`{"baidu":{"appId":"…","apiKey":"…"},"zhipu":{"apiKey":"…"}}`), and existing
single-provider `apiKey`/`appId` values are migrated into that map on first launch.
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

```powershell
npm.cmd install          # npm.ps1 is blocked by the default execution policy
npm.cmd run tauri dev    # dev build with hot reload of src/
npm.cmd run tauri build  # release build (installer + .exe)
```

`npm.cmd run icon` regenerates `src-tauri/icons` from `assets/`.

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
cargo test   # 55 tests
```

## Layout

```
src/                     frontend (plain HTML/CSS/JS, no bundler)
  index.html             settings window + playground
  popup.html             floating card
  js/bridge.js           Tauri IPC helpers used by both windows
  js/i18n.js             English/Chinese dictionaries and DOM translation
  js/render.js           shared card renderer (word, sentence, loading, error)
  js/app.js              settings window logic
  js/popup.js            popup window logic
src-tauri/src/
  main.rs  lib.rs        window setup, Tauri commands
  selection.rs           global mouse hook, gesture tracking, capture
  hotkey.rs              global hotkey registration and parsing
  classify.rs            word/phrase vs. sentence detection
  translate/             google, baidu, zhipu, deepl and openai providers, word dictionary
  popup.rs               placement/clamping geometry
  platform.rs            DPI aware cursor, work area, visible windows, click-through helpers
  clipboard.rs  settings.rs  state.rs  input.rs
```
