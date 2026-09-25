[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The global hotkey stops being a text field. A combination has to be spelled the way the
backend reads it back, and typing it by hand was the one place a working setup could be
broken by a typo, so the field now records what is pressed instead of accepting what is
written.

### Added

- **The hotkey is recorded, not typed.** `src/js/keycombo.js` reads a keyboard event into
  the spelling `hotkey::parse` accepts: modifiers in a fixed order, letters and digits
  upper-cased, function keys and the named keys the backend knows. The field opens for
  recording when it is clicked, `Record` reopens it, `Esc` or clicking away puts the stored
  combination back, and `Clear` switches the hotkey off. A key Glossy cannot name is
  reported by name instead of being silently dropped; a stray key without a modifier says
  which modifier is missing.
- **Combinations Windows will not hand over are refused before they are saved.**
  `Ctrl+Alt+Delete`, `Win+L`, `Win+Tab`, `Alt+Tab`, `Alt+Esc`, `Ctrl+Esc` and
  `Ctrl+Shift+Esc` never reach `RegisterHotKey`, so recording one now says so and keeps the
  combination that was working. Ones Windows or Explorer usually owns — `Win+D`, `Win+E`,
  `Win+R`, `Alt+F4` and the like — are recorded with a warning, because only the
  registration can tell. A test reads the named keys out of `hotkey.rs` and fails if the two
  tables ever drift apart.

<a id="zh-cn"></a>

## 中文

全局快捷键不再是一个输入框。组合键的写法必须和后端读回来的写法完全一致，而手打正是这套本来能用的设置唯一可能被一个错字毁掉的地方，所以这个框现在录下按了什么，而不是收下写了什么。

### 新增

- **快捷键是录下来的，不是打出来的。** `src/js/keycombo.js` 把一次键盘事件读成 `hotkey::parse` 认得的那种写法：修饰键按固定顺序排列，字母和数字转成大写，功能键和后端认识的具名键照原名。点一下输入框就开始录制，`录制` 可以重新开始，`Esc` 或点到别处会把存着的组合键放回去，`清除` 则关掉快捷键。Glossy 叫不出名字的键会报出自己的名字，而不是被悄悄丢掉；只按了一个键没按修饰键时，会告诉你缺哪个修饰键。
- **Windows 不会让出来的组合键，在保存前就被拒绝。** `Ctrl+Alt+Delete`、`Win+L`、`Win+Tab`、`Alt+Tab`、`Alt+Esc`、`Ctrl+Esc` 和 `Ctrl+Shift+Esc` 永远到不了 `RegisterHotKey`，所以现在录到它们会直接说明，并保留原本能用的组合键。那些通常被 Windows 或资源管理器占着的——`Win+D`、`Win+E`、`Win+R`、`Alt+F4` 之类——会带着提醒录下来，因为只有注册结果说了才算。有一个测试直接从 `hotkey.rs` 里读出具名键，两张表一旦对不上就会失败。

<a id="es"></a>

## Español

El atajo global deja de ser un campo de texto. Una combinación tiene que escribirse igual que
como la lee el backend, y teclearla a mano era el único punto donde una configuración que
funcionaba podía romperse por una errata, así que ahora el campo graba lo que se pulsa en
lugar de aceptar lo que se escribe.

### Añadido

- **El atajo se graba, no se teclea.** `src/js/keycombo.js` lee un evento de teclado y lo
  convierte en la grafía que acepta `hotkey::parse`: modificadores en un orden fijo, letras y
  dígitos en mayúscula, teclas de función y las teclas con nombre que conoce el backend. El
  campo se abre para grabar al hacer clic, `Grabar` lo reabre, `Esc` o hacer clic fuera
  devuelven la combinación guardada y `Borrar` apaga el atajo. Una tecla que Glossy no sabe
  nombrar se indica por su nombre en vez de desaparecer en silencio; una tecla suelta sin
  modificador dice qué modificador falta.
- **Las combinaciones que Windows no cede se rechazan antes de guardarse.** `Ctrl+Alt+Delete`,
  `Win+L`, `Win+Tab`, `Alt+Tab`, `Alt+Esc`, `Ctrl+Esc` y `Ctrl+Shift+Esc` nunca llegan a
  `RegisterHotKey`, así que grabarlas ahora lo dice y conserva la combinación que funcionaba.
  Las que suelen pertenecer a Windows o al Explorador —`Win+D`, `Win+E`, `Win+R`, `Alt+F4` y
  similares— se graban con un aviso, porque solo el registro puede decidir. Una prueba lee
  las teclas con nombre directamente de `hotkey.rs` y falla si las dos tablas se separan.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
