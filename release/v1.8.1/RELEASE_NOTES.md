[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

The release that makes the two rectangles of a subtitle reading visible, and lets them be moved
while the reading is running.

### Added

- **Both boxes are drawn, and both can be moved.** A region of the screen that is read every
  second is invisible by nature, so each box wears a faint dashed frame: one around the part
  the subtitles are read from, one around the window the translation goes in. Neither frame
  takes a click — they are guides, not targets. **Move the boxes** on the page — or
  *Adjust the subtitle boxes* on the tray menu, which is the way to them while a reading runs
  and the settings window is out of the way — opens both over the video, where they are
  dragged by their middles and resized by their eight handles. A box is never dragged off the
  screen or shrunk below what a reading needs, and the reading follows the boxes it is left
  with: the next line is read from the new rectangle, and the translation is drawn in the new
  place, without stopping and starting again. `Esc` leaves the boxes where they were.

<a id="zh-cn"></a>

## 中文

这一版让字幕识别的两个矩形看得见，并且可以在识别进行中随时移动它们。

### 新增

- **两个框都会画出来，也都能移动。**一块每秒都被读取的屏幕区域本来是完全看不见的，所以两个框各带一圈
  很淡的虚线：一个框住读取字幕的区域，一个框住画译文的窗口。这两圈虚线都不吃点击——它们只是参考线，
  不是可点的目标。页面上的**调整框**，或者托盘菜单里的「调整字幕框」（识别正在跑、设置窗口已经让开的
  时候，就从这里进），会把两个框显示在画面上：拖框中间移动位置，拖八个手柄改大小。框不会被拖出屏幕，
  也不会被缩到比一次识别所需更小；改完之后识别就接着用新的框——下一行从新的区域读取，译文画在新的
  位置，不需要停下来再重开。按 `Esc` 则保持原样退出。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
