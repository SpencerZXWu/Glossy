[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where a translation can take the place of the text it came from, and where the
Document page asks for everything it needs to know before it starts instead of after.

### Added

- **A translation can replace the text it was made from.** With the card up for a
  selection, `Ctrl+Enter` — or the third button under the translation — writes the
  translation over that selection in the program in front: select, read, replace, which is
  what writing a reply or a paragraph in another language is actually made of. The key is
  registered for exactly as long as a card holds the translation of a selection that is
  still in place, so no other program loses `Ctrl+Enter` to Glossy, and a card brought back
  from the history — which has no selection behind it — never takes it at all. The text
  travels through the clipboard with a pasted `Ctrl+V`, so the clipboard is put back the way
  the *restore clipboard* setting asks.
- **The Document page asks for its languages and its options up front.** *Document* opens
  on a box a file can be dropped onto or clicked to browse through, a source/target pair of
  pickers above it, and an *Advanced settings* block: **Leave the pieces that hold no
  letters as they are** keeps numbers, symbols and page furniture out of the bill,
  **Save the translation as soon as it is finished** writes the file without a second
  click, and **Longest piece sent in one request** caps how much goes into a single call.
- **A translated document is saved where you say.** The page remembers a folder and shows
  it next to the buttons, `Change…` opens the folder picker, and until one is chosen the
  translation lands on the **Desktop** rather than in `Documents`.

### Fixed

- **Clicking the gear on a pinned card no longer throws the card away.** The settings
  button was the one header button that still dismissed the card it was pressed on, so a
  card the user had explicitly asked to stay disappeared exactly when it was needed.

<a id="zh-cn"></a>

## 中文

这一版里，译文可以直接顶掉它的原文；文档页也会在开始之前就把需要知道的事情都问清楚，
而不是翻译完再问。

### 新增

- **译文可以替换掉它由之而来的那段原文。** 选区对应的卡片打开时，按 `Ctrl+Enter`，
  或点译文下方的第三个按钮，就会把译文写回前台程序里的那段选区：选中、看一眼、替换——
  用另一种语言回一条消息或写一段话，本来就是这几步。这个按键只在卡片确实拿着「仍在原处
  的选区」的译文时注册，所以别的程序不会从 Glossy 这里丢掉 `Ctrl+Enter`；而从历史记录里
  翻出来的卡片背后没有选区，压根不会占用它。文本经由剪贴板以粘贴的方式写入，所以剪贴板会
  按「还原剪贴板」设置的要求恢复原样。
- **文档页会在开始之前问清语言和选项。** 「文档」打开时是一个可以拖入文件、也可以点击
  浏览的方框，上方是原文/译文两个语言选择器，下面是一个「高级设置」区：
  **跳过不含字母的片段** 让数字、符号和版式杂项不计入用量，**翻译完成后立即保存译文**
  不用再点一次就把文件写出来，**单次请求最长片段** 限制一次调用送出的字数。
- **译好的文档保存在你指定的位置。** 页面会记住一个文件夹并把它显示在按钮旁边，
  `更改…` 打开文件夹选择器；在选定之前，译文会落在**桌面**，而不是「文档」。

### 修复

- **在固定住的卡片上点齿轮不会再让卡片消失。** 这是标题栏里唯一一个仍会把所在卡片关掉的
  按钮，于是用户明确要求留下来的那张卡片，恰恰在最需要的时候不见了。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
