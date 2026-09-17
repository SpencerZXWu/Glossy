# Changelog

All notable changes to Glossy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
See [ROADMAP.md](./ROADMAP.md) for what is planned next.

## [Unreleased]

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

[Unreleased]: https://github.com/SpencerZXWu/Glossy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.1.0
