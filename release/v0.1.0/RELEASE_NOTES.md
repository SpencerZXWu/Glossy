First public release - a lightweight select-to-translate app for Windows.

**Select text anywhere, release the mouse, get a translation in a popup below the cursor.**

### Added

- Selection capture via a low-level mouse hook: triggers on mouse-up after a drag across text, or on double click. Selections shorter than the configured minimum length are ignored, and a drag that starts or ends on the popup never triggers a translation.
- Floating popup below the cursor: non-activating, always on top, draggable by its header, clamped to the work area of the monitor under the cursor, flipped above the cursor when there is no room below.
- Word card with phonetic symbols, part of speech, definitions and one example; sentence mode renders a fluent translation instead.
- Language bar in the popup: pick either side of the pair to translate again, or press `?` to translate the result back. The pair resets for every new selection and overrides are never persisted.
- Five translation providers: Google (free, no key), Baidu, Zhipu GLM, DeepL and OpenAI.
- Per-provider credentials, so switching back to an already configured provider just works.
- Settings window with a master toggle, trigger options, minimum selection length, global hotkey, per-program ignore list, theme, popup width, text size, opacity and auto-close, plus an interface language switch (`Follow Windows`, `????`, `English`).
- Translate-in-place card in the settings window: type or paste text, select part of it (or press *Translate*) and the result renders below in the same card the popup uses.
- Global hotkey (default `Ctrl+Alt+C`) that translates the clipboard content, and a setting to restore the previous clipboard content after reading a selection.

### Install

Download `Glossy_0.1.0_x64-setup.exe` and run it. Windows 11 x64; WebView2 is required and is already present on current Windows builds.

### Known limitations

- The installer is **not code-signed**, so SmartScreen will warn about an unknown publisher.
- API keys are stored in plaintext in `%APPDATA%\com.glossy.translator\settings.json`. Encrypting them with DPAPI is the first item of [v0.2.0](https://github.com/SpencerZXWu/Glossy/milestone/2).
- Windows only. The `windows` crate is used unconditionally today, so other platforms are a port rather than a flag.

The full changelog is in [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) and what comes next is in [ROADMAP.md](https://github.com/SpencerZXWu/Glossy/blob/main/ROADMAP.md).