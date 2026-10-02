[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where the two windows can be worked with the keyboard alone, and where a
contrast theme is something the app follows rather than something it paints over.

### Added

- **The keyboard reaches everything, in both windows.** The settings window walks its
  sidebar with the arrows, Home and End, and its sections with Tab in the order they are
  drawn; the confirm dialog that asks before something is thrown away now keeps the
  keyboard inside its two buttons while it is up, and gives it back to whatever had it
  before. In the card, the caret going into the grey original — or a Tab once the card
  holds the keyboard — is what hands the popup the front, and the engine menu at its foot
  opens with the arrows, moves with the arrows, Home and End, and closes with `Esc` or Tab
  back onto the button it belongs to.
- **The keyboard goes back when the card does.** A card that was given the front so a
  field in it could be typed into returns it to the program the selection came from when
  it closes, instead of leaving it on a window that is no longer there.
- **A high-contrast theme, taken from Windows.** With a contrast theme turned on, every
  colour in the app comes from the system palette: the surfaces turn solid, the strokes
  stop being subtle, the accent becomes the system highlight and the focus rings stay on
  top of it. It is not an entry in the theme menu, because the choice was already made in
  Windows — and a custom accent stops overriding the system highlight for as long as one
  is on.

### Changed

- **Nothing is read through something else while a contrast theme is on.** The card's
  opacity, the faint wordmark at its foot, the dimmed sections whose controls are off, and
  the pulsing placeholder all keep their own colour and their own weight in that mode.

### Fixed

- **The same measurement written twice is one row.** The annotation under the card
  compared the numbers as they were typed, so `12 ft` twice was one row but `12ft` next to
  `12 ft`, or `12 ft` next to `12 feet`, became two rows with the same answer. It compares
  the amount and the unit now, so a repeated measurement is one row however the
  translation spells it — and two different amounts are still two rows, which is what the
  reader asked about.
- **A shortcut field no longer holds the keyboard.** Recording starts when the box is
  focused, and it used to record every key it saw — including Tab, which is how the
  keyboard had reached the box. Tab and Shift+Tab now leave it unchanged, the hint under it
  says so, and the combination that was there is still there afterwards.

<a id="zh-cn"></a>

## 中文

这一版把两个窗口都交给了键盘，也让高对比主题变成应用跟随的东西，而不是被应用盖掉的东西。

### 新增

- **键盘能够到两个窗口里的每一个控件。** 设置窗口的侧栏可以用方向键、Home、End 走，页面里用 Tab 按绘制顺序走；在丢掉什么之前问一句的确认框，弹出期间把键盘留在自己的两个按钮里，关掉后交还给原先拿着它的控件。卡片这边，光标进入灰色的原文（或者卡片已经拿着键盘时按一下 Tab）就是把前台交给弹窗的动作；卡片底部的引擎菜单用方向键展开、用方向键和 Home/End 移动、用 `Esc` 或 Tab 关掉并回到它自己那个按钮上。
- **卡片关掉时键盘也回去。** 为了打字而拿到前台的那张卡片，关闭时会把前台交还给这段选区原本所在的程序，而不是留在一个已经不存在的窗口上。
- **一套取自 Windows 的高对比主题。** 高对比主题开着时，应用里的每一个颜色都来自系统调色板：表面变成实色，描边不再「若隐若现」，强调色变成系统高亮色，焦点环压在高亮色之上。它不是主题菜单里的一项，因为这个选择已经在 Windows 里做过了——而且只要它开着，自定义强调色就不再盖掉系统高亮色。

### 变更

- **高对比主题下，没有任何文字是靠别的东西读出来的。** 卡片的不透明度、卡片底部那枚淡淡的字标、控件关闭时变暗的区块，以及一闪一闪的占位文字，在那个模式里都保留自己的颜色和分量。

### 修复

- **同一个量写两遍只出一行。** 卡片下方那组换算过去比较的是「当时是怎么写的」，所以 `12 ft` 写两遍是一行，但 `12ft` 和 `12 ft` 挨在一起、或者 `12 ft` 和 `12 feet` 挨在一起，就变成了两行、答案却一模一样。现在比较的是数值和单位：同一个量无论译文怎么拼写都只出一行，而两个不同的量仍然各出一行——那正是读者问到的。
- **录制快捷键的输入框不再扣住键盘。** 录制是在输入框获得焦点时开始的，而它过去会记录看到的每一个按键——包括键盘正是靠它才走到这个输入框上的 Tab。现在 Tab 和 Shift+Tab 会原样离开，输入框下方的提示也这么写着，原来那条组合键也还在。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
