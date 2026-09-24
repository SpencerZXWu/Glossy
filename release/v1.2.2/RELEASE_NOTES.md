[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

A selection no longer costs anything the moment it is made. Glossy answers it with a small
icon under the text, and the translation — and the day's allowance with it — waits for a
click on that icon. The card's close button is gone, and a settings button has taken its
place.

### Changed

- **A selection shows an icon instead of the card.** Releasing the mouse used to start the
  translation straight away, which spent one of the day's translations on every word the
  reader happened to swipe over. The selection now puts a 44px Glossy icon under the text
  and stops there: the translation is asked for when the icon is clicked, so a selection
  that was never meant to be translated costs nothing. Clicking anywhere else dismisses
  the icon, and the shortcut (`Ctrl+Alt+C`) behaves the same way.
- **The card's close button is a settings button.** Closing the card is what a click
  anywhere else already does, so the `×` was one button for something the window never
  needed help with; it now opens the settings window through the new `open_settings`
  command. `Esc` still closes the card.
- **Exchange rates come through the server.** A currency conversion used to call the rate
  feed straight from the app; it now asks Glossy's own server first (`/v1/rates`, which
  costs no translation allowance) and only reaches out to the feed when the server cannot
  answer.

<a id="zh-cn"></a>

## 中文

选区本身不再立刻产生任何消耗：Glossy 会在文字下方显示一个小图标，翻译——以及当天额度——都等到你点击这个图标时才开始。卡片右上角的关闭按钮被移除，改成了设置按钮。

### 变更

- **选区只显示图标，不再直接显示卡片。** 过去一松开鼠标就会开始翻译，读者只是随手划过的每一个词都要花掉当天的一次翻译。现在选区会在文字下方放一个 44px 的 Glossy 图标，然后就停下：只有点击该图标才会请求翻译，因此从未打算翻译的选区不会有任何消耗。点击其他区域图标即消失，快捷键（`Ctrl+Alt+C`）的行为同样如此。
- **卡片的关闭按钮改成了设置按钮。** 关闭卡片本来就能通过点击别处完成，所以这个 `×` 只是为一个窗口本来就不需要帮忙的动作多设了一个按钮；它现在通过新增的 `open_settings` 命令打开设置窗口。`Esc` 仍然可以关闭卡片。
- **汇率改由服务器转发。** 货币换算过去由客户端直接请求汇率接口，现在先问 Glossy 自己的服务器（`/v1/rates`，不占翻译额度），只有服务器答不上来时客户端才直连接口。

<a id="es"></a>

## Español

Una selección ya no cuesta nada en el momento de hacerla. Glossy responde con un pequeño icono bajo el texto, y la traducción —y con ella la cuota del día— espera a un clic sobre ese icono. El botón de cerrar de la tarjeta ha desaparecido y en su lugar hay un botón de ajustes.

### Cambiado

- **Una selección muestra un icono en lugar de la tarjeta.** Soltar el ratón empezaba la traducción de inmediato, lo que gastaba una de las traducciones del día en cada palabra que el lector rozaba sin querer. Ahora la selección coloca un icono de Glossy de 44px bajo el texto y se detiene ahí: la traducción se pide al hacer clic en el icono, así que una selección que nunca pretendía traducirse no cuesta nada. Al hacer clic en cualquier otro sitio el icono desaparece, y el atajo (`Ctrl+Alt+C`) se comporta igual.
- **El botón de cerrar de la tarjeta es ahora un botón de ajustes.** Cerrar la tarjeta es lo que ya hace un clic en cualquier otro sitio, así que la `×` era un botón para algo con lo que la ventana nunca necesitó ayuda; ahora abre la ventana de ajustes mediante el nuevo comando `open_settings`. `Esc` sigue cerrando la tarjeta.
- **Los tipos de cambio pasan por el servidor.** Una conversión de moneda antes llamaba directamente al proveedor de tipos de cambio; ahora pregunta primero al servidor propio de Glossy (`/v1/rates`, que no consume cuota de traducción) y solo acude al proveedor cuando el servidor no puede responder.
