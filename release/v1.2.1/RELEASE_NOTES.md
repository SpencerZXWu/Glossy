[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

A language bar now tells the truth about the engine behind it. The eight languages a
standard Baidu account refuses are no longer offered, and the language a reader picks in
the card is the language the next selection starts from.

### Changed

- **A language bar offers only the languages the chosen engine translates.** The bars were
  built from one list for all three engines, and a standard Baidu account refuses eight of
  its 31 entries — `uk`, `tr`, `hi`, `id`, `ms`, `he`, `no` and `sk` — so 百度 could be
  asked for a language it answers with `58001`. `src-tauri/src/translate/languages.rs` is
  now the only table (`ALL`, plus `BAIDU`, `YOUDAO` and Google's, which is every language),
  published to both windows through the new `service_languages` command, and the two bars —
  the card's and the settings window's — rebuild themselves from it whenever the engine
  changes, in either direction. A code the provider detected that the shared menu never had
  stays selectable, so a bar is never blank, and a target the backend is handed anyway is
  clamped to a language the engine does take rather than sent and refused.
- **The language a reader picks in the card is the configured language from then on.** The
  target chosen in either window's bar was good for the card on screen and forgotten at the
  next selection, so translating a page into Japanese and then selecting the next word put
  it back into the configured language. The new `set_target_lang` stores the pick, and the
  source language keeps its old behaviour — it is reset to *detect it* for every selection.
  The swap button does not save either, so a reversal does not change the setting.

<a id="zh-cn"></a>

## 中文

语言栏现在会如实反映它背后的渠道：百度普通账号翻不了的那八种语言不再出现在列表里，而读者在卡片里选定的目标语言，会成为下一次选区直接使用的语言。

### 变更

- **语言栏只提供所选渠道真正翻译的语言。** 过去三个渠道共用一份列表，而百度普通账号会拒绝其中 31 种里的 8 种——`uk`、`tr`、`hi`、`id`、`ms`、`he`、`no` 和 `sk`——于是百度可能被要求翻译一个它只会以 `58001` 回应的语言。现在 `src-tauri/src/translate/languages.rs` 是唯一的表（`ALL`，以及 `BAIDU`、`YOUDAO` 和 Google 的表，后者包含全部语言），通过新增的 `service_languages` 命令下发给两个窗口，卡片和设置页的两个语言栏都会在渠道变化时按它重建，两个方向都如此。渠道自己识别出、而共用菜单里从未有过的语言代码仍然可选，因此语言栏不会出现空白；即便后端仍然收到了渠道不支持的目标语言，也会被改成该渠道确实接受的语言，而不是发出去被拒绝。
- **读者在卡片里选定的语言，从此就是已配置的语言。** 过去在任一窗口的语言栏里选定的目标语言只对眼前这张卡片有效，下一次选区就被遗忘，于是把一页内容翻成日语后再选下一个词，又回到了已配置的语言。新增的 `set_target_lang` 会记住这次选择，而源语言保持原有行为——每次选区都重置为「自动检测」。互换按钮同样不会保存，因此一次调换不会改掉设置。

<a id="es"></a>

## Español

Una barra de idiomas ahora dice la verdad sobre el motor que tiene detrás. Los ocho idiomas que rechaza una cuenta estándar de Baidu ya no se ofrecen, y el idioma que el lector elige en la tarjeta es el idioma con el que empieza la siguiente selección.

### Cambiado

- **Una barra de idiomas solo ofrece los idiomas que traduce el motor elegido.** Las barras se construían a partir de una única lista para los tres motores, y una cuenta estándar de Baidu rechaza ocho de sus 31 entradas —`uk`, `tr`, `hi`, `id`, `ms`, `he`, `no` y `sk`—, así que a 百度 se le podía pedir un idioma al que responde con `58001`. Ahora `src-tauri/src/translate/languages.rs` es la única tabla (`ALL`, más `BAIDU`, `YOUDAO` y la de Google, que es todos los idiomas), publicada a las dos ventanas mediante el nuevo comando `service_languages`, y las dos barras —la de la tarjeta y la de la ventana de ajustes— se reconstruyen a partir de ella cada vez que cambia el motor, en cualquier dirección. Un código que el proveedor detectó y que el menú compartido nunca tuvo sigue siendo seleccionable, de modo que una barra nunca queda vacía, y un destino que aun así llegue al backend se ajusta a un idioma que el motor sí acepta, en lugar de enviarse y ser rechazado.
- **El idioma que el lector elige en la tarjeta es, desde entonces, el idioma configurado.** El destino elegido en la barra de cualquiera de las dos ventanas servía para la tarjeta en pantalla y se olvidaba en la siguiente selección, así que traducir una página al japonés y luego seleccionar la palabra siguiente volvía al idioma configurado. El nuevo `set_target_lang` guarda la elección, y el idioma de origen conserva su comportamiento anterior: se restablece a *detectarlo* en cada selección. El botón de intercambio tampoco guarda, de modo que una inversión no cambia el ajuste.
