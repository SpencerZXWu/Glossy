[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release where replacing a selection actually lands on it, in every program it was
asked to.

### Fixed

- **The replacement is written over the selection instead of next to it.** A paste goes to
  whichever window is in front, and the card was in front because it had just been clicked,
  so `Ctrl+Enter` could land in the popup or at the end of the line. The window the
  selection was read from — remembered while the selection is still live — is put back in
  front and given the keyboard before the paste, and keeps it until the paste has been
  made.
- **Clicking the badge no longer throws away the selection in WeChat.** WeChat clears the
  text it has selected the moment it stops being the window in front, so the badge — whose
  whole job is to be clicked — cost the very selection the translation was meant to
  replace. The popup is shown without taking the foreground, and the style that does that
  survives being shown, so the click lands on Glossy while the program behind it stays in
  front and keeps the selection selected. The card's own original can still be corrected in
  place: opening that field is what asks for the keyboard.

<a id="zh-cn"></a>

## 中文

这一版修的是「替换原文」本身：在每一个被要求替换的程序里，译文现在真的会落在选区上面。

### 修复

- **译文写在选区上，而不是写在它后面。** 粘贴会落进最前面的那个窗口，而卡片正是最前面
  的那个——它刚被点过——于是 `Ctrl+Enter` 可能落进弹窗里或落进行尾。现在，在粘贴之前，
  译文由之而来的那个窗口（趁着选区还在时就记下来的）会先被放回最前面并拿到键盘，并且一直
  拿到粘贴完成为止。
- **在微信里点图标不再丢掉选区。** 微信一旦不再是前面的那个窗口，就会把自己选中的文字
  取消掉，于是「点一下图标」这个动作本身——它是小图标存在的全部意义——恰好毁掉了那段本来
  要被替换掉的选区。现在弹窗显示时不抢前台，并且这个「不抢前台」的属性在显示之后依然
  还在，所以点击落在 Glossy 上，而后面的程序仍在前面、选区仍是选中的。卡片里的原文依旧
  可以就地修改：点进那个输入区域，才是要键盘的动作。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
