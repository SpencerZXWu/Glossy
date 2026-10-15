[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

### Changed

- **A screenshot is read, and then it waits.** Dragging a rectangle used to read it
  and translate it in one go: the card arrived with the translation already in it, and
  the day's characters were spent on whatever the rectangle happened to hold — a
  heading, a file name, a line the reader only wanted to copy. The card now stops at
  what was read: the text sits in the same grey block the original is drawn in, with
  the copy button beside it, and one button translates it. Nothing else on that card
  reaches a provider — the language bar is not drawn on it and the text cannot be
  edited, because both of those are what translate on every other card — and no
  auto-close is armed while the reader is deciding.
- **The language bar is hidden when a card says it is.** `.langbar` is `display: flex`,
  which wins over the `hidden` attribute that the badge, the loading card and the
  engine wait all set on it, so the bar and its two live combo boxes were on screen in
  three states that have nothing to choose yet. Changing a language there starts a
  translation, which is why this had to be right rather than merely tidy.

<a id="zh-cn"></a>

## 中文

<!-- TODO: translate the English section above, then delete this comment. -->

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
