[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The engine can be changed while an answer is still on its way.

### Changed

- **The engine switcher is on the card before the answer is.** The name in the card's bottom
  row was drawn from the engine that answered, so a card that had not been filled yet had no
  bottom row at all: a translation that was slow — or one that went to the wrong engine — had
  to be waited out before the list of engines could be opened, and the word card behaved the
  same way while its dictionary lookups were still running. The row is now drawn on the
  loading card too, and the name in it is the engine the settings picked, which is the one
  that was asked rather than the one that answered (`src/js/render.js`, `src/js/popup.js`).
  Choosing from it during a translation restarts that translation on the engine that was
  picked: the answer on its way is dropped, and the characters are billed to the new engine.
- **The Worker's default daily allowance matches the code.** `DAILY_CHARS_TOTAL` in
  `server/wrangler.toml` said `300000` while the code default and the function that is
  deployed both use `30000`; the file now says `30000`, with the reasoning written next to
  it, so a Worker deployed from it allows the same day as the function already running.

<a id="zh-cn"></a>

## 中文

<!-- TODO: translate the English section above, then delete this comment. -->

<a id="es"></a>

## Español

<!-- TODO: translate the English section above, then delete this comment. -->

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
