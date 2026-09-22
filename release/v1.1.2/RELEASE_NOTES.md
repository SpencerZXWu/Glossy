[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The bottom line of the card and the order of the settings window: the card says which
engine translated it and can change engines in place, and the settings finally group each
switch with the thing it changes. The local model is gone, so nothing has to be installed
to translate.

### Added

- **The card names the engine that translated it, and can change it.** The line under
  the translation printed the literal word `cloud` for both 百度 and 有道, because it
  was reading the storage shape instead of the service. It names the service that
  answered now — Baidu, Youdao or Google — and the name is a button: pressing
  it opens the services in place inside the card with the current one ticked,
  and picking one saves the choice and translates the selection again. `cloud::answered_by`
  keeps the literal `cloud` out of what the card is told, `current_service` and
  `set_service` carry the choice back and forth, and Escape closes the list before it
  closes the card.

### Changed

- **The settings window shows the services as plain names.** The line under the
  dropdown that promised a service needed no setup and cost nothing is gone. The card's
  own line carries the same names, so the two places a service is chosen finally agree.
- **The name of the engine shares one line with the two read-aloud buttons.** It sat on
  the line above them and could wrap away from them on a narrow card. The footer is a
  single row now, and the name is given the room it needs and kept to one line
  rather than wrapped, so the buttons keep their place at the end of the line.
- **The settings window groups each switch with the thing it changes.** The panels now
  run from the master switch through triggering, languages, the fallback service and the
  card's options to reading, units, history, the settings file and updates, and the two
  switches that describe the card — showing the original sentence and the compact layout
  — sit with the rest of the card's options instead of in the master and reading panels.
  Starting Glossy with the system is its own group: it sat among the switches the master
  switch dims while translation is off, which left it unreachable from a paused app.

### Removed

- **The local model is no longer an option.** The entry that translated through Ollama,
  or any other OpenAI compatible server on the same machine, is gone from the dropdown,
  from the fallback list and from the card's own list, along with the address and model
  fields it needed and the two settings keys behind them. Nothing has to be installed to
  translate: the two engines that go through Glossy's server and the free Google
  endpoint are the whole list. A settings file that still names the local model reads as
  the built-in engine and is rewritten on the next save.

<a id="zh-cn"></a>

## 中文

这次动的是卡片的底部和设置页的顺序：卡片底部会写明是哪个引擎翻的，并能在卡片里直接换引擎；设置页也终于把每个开关和它真正影响的东西放在了一起。本机模型已经移除，翻译不需要安装任何东西。

### 新增

- **卡片会写明是哪个引擎翻的，并且可以当场更换。** 译文下面那行以前对百度和有道都印着 `cloud` 这个词，因为它读的是存储结构而不是渠道。现在它写出真正作答的渠道名——百度、有道或 Google——而且这个名字本身是个按钮：点一下就在卡片里展开渠道列表、当前渠道带勾，选中即保存选择并重新翻译这段选区。`cloud::answered_by` 让字面的 `cloud` 不再出现在传给卡片的数据里，`current_service` 和 `set_service` 负责把选择传回去，Esc 会先关掉列表、再关卡片。

### 变更

- **设置窗口里只显示渠道名字。** 下拉框下面那句「无需配置、免费」的说明已经删掉。卡片上那一行用的是同一批名字，两个选渠道的地方终于一致了。
- **渠道名和两个朗读按钮在同一行。** 以前渠道名在上面一行，卡片窄的时候会被挤到按钮之外。现在底部是一整行：渠道名占它需要的位置、始终不换行，两个按钮固定在该行末尾。
- **设置页把每个开关和它影响的东西放在一起。** 面板顺序现在是：总开关 → 触发 → 语言 → 备用服务 → 弹窗 → 阅读 → 单位换算 → 历史记录 → 设置文件 → 更新；描述弹窗的两个开关（显示原文、紧凑布局）和弹窗的其他选项放在一起，不再留在总开关和阅读面板里。「开机自启」也单独成组：它原来和那些在总开关关闭时会被置灰的开关放在一起，导致关掉划词翻译后就点不动了。

### 移除

- **本机模型不再是选项。** 通过 Ollama 或同机器上任何 OpenAI 兼容服务翻译的选项，已从下拉框、备用服务列表和卡片内的列表里移除，连同它需要的地址、模型输入框和背后的两个设置项。翻译不需要安装任何东西：走 Glossy 服务器的两个引擎加上免费的 Google 端点就是全部。仍写着本机模型的设置文件会按内置引擎读取，并在下次保存时被改写。

<a id="es"></a>

## Español

Esta vez le tocó a la última línea de la tarjeta y al orden de la ventana de ajustes: la tarjeta dice qué motor ha traducido y permite cambiar de motor sin salir de ella, y los ajustes por fin agrupan cada interruptor con lo que cambia. El modelo local ya no está, así que no hay que instalar nada para traducir.

### Añadido

- **La tarjeta dice qué motor ha traducido y permite cambiarlo.** La línea bajo la traducción imprimía la palabra literal `cloud` tanto para 百度 como para 有道, porque leía la forma del almacenamiento en lugar del servicio. Ahora nombra el servicio que respondió — Baidu, Youdao o Google — y ese nombre es un botón: al pulsarlo se abre la lista de servicios dentro de la tarjeta con el actual marcado, y al elegir uno se guarda la elección y se traduce otra vez la selección. `cloud::answered_by` mantiene el `cloud` literal fuera de lo que se le cuenta a la tarjeta, `current_service` y `set_service` llevan la elección de ida y vuelta, y Escape cierra la lista antes de cerrar la tarjeta.

### Cambios

- **La ventana de ajustes muestra los servicios como nombres a secas.** La línea bajo el desplegable que prometía que un servicio no necesitaba configuración y no costaba nada ya no está. La línea de la propia tarjeta lleva los mismos nombres, así que los dos sitios donde se elige un servicio por fin coinciden.
- **El nombre del motor comparte una sola línea con los dos botones de lectura.** Antes estaba en la línea de encima y podía separarse de ellos en una tarjeta estrecha. Ahora el pie es una sola fila: el nombre recibe el espacio que necesita y se mantiene en una línea en lugar de partirse, así que los botones conservan su sitio al final de la fila.
- **La ventana de ajustes agrupa cada interruptor con lo que cambia.** Los paneles van ahora del interruptor maestro a los disparadores, los idiomas, el servicio de reserva y las opciones de la tarjeta, y después lectura, unidades, historial, el archivo de ajustes y las actualizaciones; y los dos interruptores que describen la tarjeta — mostrar la frase original y el diseño compacto — están con el resto de opciones de la tarjeta en lugar de en los paneles del maestro y de lectura. Arrancar Glossy con el sistema es un grupo propio: estaba entre los interruptores que el maestro atenúa cuando la traducción está apagada, lo que lo dejaba fuera de alcance con la aplicación en pausa.

### Eliminado

- **El modelo local ya no es una opción.** La entrada que traducía mediante Ollama, o cualquier otro servidor compatible con OpenAI en la misma máquina, ha desaparecido del desplegable, de la lista de reserva y de la lista de la propia tarjeta, junto con los campos de dirección y modelo que necesitaba y las dos claves de ajustes que los respaldaban. No hay que instalar nada para traducir: los dos motores que pasan por el servidor de Glossy y el punto de acceso gratuito de Google son toda la lista. Un archivo de ajustes que aún nombre el modelo local se lee como el motor integrado y se reescribe en el siguiente guardado.
