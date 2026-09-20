[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Added

- **Baidu and Youdao as channels of their own, still with nothing to set up.**
  The dropdown now lists **Baidu Translate** and **Youdao Translate** next to
  Glossy's own server. Both are that same server — no key, no field to fill in —
  and they only say which upstream it should translate with, so the vendor that
  answers is visible in the list without anyone handling credentials. The server
  answers with the named upstream first (the request gained a `vendor` field) and
  still falls back to the others when it cannot, and the result card names whichever
  one answered. The settings file stores the choice as `cloudVendor` (`baidu`,
  `youdao`, or empty for "let the server decide"), the server keeps its
  `/v1/health` list of the upstreams a deployment actually has, and
  `server/src/youdao.js` implements the Youdao Zhiyun upstream with its v3
  signature.

- **Translation service, one list.** The settings window no longer asks whether
  results come from a cloud or from your own account; it asks which service
  translates, and the same dropdown holds all of them: Glossy's own server, which
  holds the provider credentials so no key of yours is involved; a model running on
  this machine over any OpenAI-compatible endpoint (Ollama's
  `http://127.0.0.1:11434/v1` by default, model `qwen2.5:7b`), which keeps the text
  on the computer and costs nothing; the free public Google endpoint, which needs no
  key; and then the vendors that take the user's own key — Baidu, Zhipu, DeepL and
  OpenAI. A new settings file starts on Glossy's server, and a file written before
  this release keeps the provider it already had, so nothing has to be re-entered.
  The credentials of the last group moved out of that panel into **Extensions · my
  own API**, a panel of its own at the bottom of the settings window, where they sit
  dimmed until one of those services is chosen. Under the stored shape nothing
  changed — Glossy's server and the local model are still the cloud channel, the
  rest still use the API channel — so an older build reads the file as it always
  did. The server gained a second upstream to go with it: it now translates through
  an LLM account when one is configured and falls back to the Baidu credentials it
  had before, so a deployment works either way.

- **The translation server's address is no longer a field.** Glossy's own service
  is the one that needs nothing set up, and a text box for the address only gave
  people a way to break it — a wrong or stale address left the app unable to reach
  any server at all, which is what the settings file of an upgraded install could
  hold. The address is now part of the build: whichever one the app was compiled
  with wins, a leftover value in an older settings file is ignored, and the window
  keeps showing what is left of today's allowance with its `Check again` button.
  Running your own deployment is a one-line change of `DEFAULT_ENDPOINT` in
  `src-tauri/src/translate/cloud.rs`; a build made without one still honors the
  setting.

### Fixed

- A second launch of Glossy now shows the same starting hint in the corner of the
  screen that a start of its own would, instead of an unstyled Windows dialog. The
  running instance is told through a named event and answers by opening the settings
  window when it is already on screen, or by showing the hint when it is not; the
  dialog is left only for the case where nothing answers at all.

<a id="zh-cn"></a>

## 中文

### 新增

- **百度翻译和有道翻译成为独立渠道，依旧无需任何配置。**
  翻译渠道下拉框里，现在在 Glossy 自己的服务器旁边多了 **百度翻译** 和 **有道翻译**。
  它们指向的还是同一个服务器——不用密钥、没有要填的输入框——只是指定服务器用哪个上游来翻译，
  于是谁给出了译文在列表里一目了然，而没人需要经手凭证。服务器优先用请求点名的上游
  （请求里新增了 `vendor` 字段），用不了时仍然回退到其他上游，译文卡片会写明是谁给出的结果。
  设置文件把选择存成 `cloudVendor`（`baidu`、`youdao`，留空表示"由服务器决定"），
  服务器的 `/v1/health` 依旧列出该部署实际拥有的上游，
  `server/src/youdao.js` 实现了有道智云的 v3 签名接口。

- **翻译服务合并成一个列表。** 设置窗口不再问译文来自云端还是自己的账号，
  而是问用哪个服务翻译，同一个下拉框里放着全部选项：Glossy 自己的服务器（凭证在服务器上，
  不涉及你的密钥）；跑在这台机器上、任何兼容 OpenAI 接口的模型
  （默认指向 Ollama 的 `http://127.0.0.1:11434/v1`，模型 `qwen2.5:7b`），
  文字不出本机、也不花钱；免费的公共谷歌翻译接口，不需要密钥；
  以及需要你自己密钥的服务商——百度、智谱、DeepL 和 OpenAI。
  新的设置文件默认用 Glossy 的服务器，本次更新之前写下的文件会保留原有的服务，
  不需要重新填写。最后那一组的凭证从该面板移到了设置窗口最下方独立的
  **扩展 · 我的 API** 面板，未选中对应服务时它们会以置灰状态显示。
  存储结构没有变化——Glossy 的服务器和本地模型仍属于云端通道，其余仍走 API 通道——
  因此旧版本依然能像以前一样读取该文件。服务器也相应增加了第二个上游：
  配置了 LLM 账号时用它翻译，不可用时回退到原有的百度凭证，两种部署都能正常工作。

- **翻译服务器的地址不再是输入框。** Glossy 自己的服务是那个开箱即用的服务，
  而一个地址输入框只会给人留出把它改坏的机会——地址写错或过期会让应用彻底连不上任何服务器，
  升级安装后的设置文件里就可能残留这样的值。现在地址是构建的一部分：
  应用编译时用的地址生效，旧设置文件里遗留的值会被忽略，
  窗口照旧显示当天剩余的额度以及 `重新查询` 按钮。
  想跑自己的部署，只需改 `src-tauri/src/translate/cloud.rs` 里的 `DEFAULT_ENDPOINT` 一行；
  没有配置地址的构建仍然沿用该设置项。

### 修复

- 重复启动 Glossy 时，现在会和正常启动一样在屏幕角落弹出那个提示卡片，
  而不再是一个没有样式的 Windows 对话框。运行中的实例通过一个有名字的事件收到通知，
  它会在设置窗口已经打开时把窗口调到前面，没打开时则显示提示卡片；
  只有完全无人应答时才会退回使用对话框。

<a id="es"></a>

## Español

### Añadido

- **Baidu y Youdao como canales propios, y sin configurar nada.**
  La lista de servicios incluye ahora **Baidu Translate** y **Youdao Translate**
  junto al servidor de Glossy. Los dos son ese mismo servidor —sin clave y sin
  ningún campo que rellenar— y solo indican con qué proveedor debe traducir, de modo
  que se ve quién respondió sin que nadie tenga que manejar credenciales. El servidor
  usa primero el proveedor indicado (la petición ganó un campo `vendor`) y sigue
  recurriendo a los demás cuando no puede, y la tarjeta del resultado nombra al que
  respondió. El archivo de ajustes guarda la elección como `cloudVendor` (`baidu`,
  `youdao`, o vacío para «que decida el servidor»), el servidor mantiene su lista de
  proveedores en `/v1/health` y `server/src/youdao.js` implementa el proveedor de
  Youdao Zhiyun con su firma v3.

- **Un solo servicio de traducción, una sola lista.** La ventana de ajustes ya no
  pregunta si los resultados vienen de la nube o de tu propia cuenta: pregunta qué
  servicio traduce, y la misma lista los contiene todos: el servidor de Glossy, que
  guarda las credenciales y no pide ninguna clave; un modelo que se ejecuta en este
  equipo por cualquier extremo compatible con OpenAI (por defecto el
  `http://127.0.0.1:11434/v1` de Ollama, modelo `qwen2.5:7b`), que deja el texto en
  el ordenador y no cuesta nada; el extremo público y gratuito de Google, que no
  necesita clave; y los proveedores que usan tu propia clave: Baidu, Zhipu, DeepL y
  OpenAI. Un archivo nuevo empieza con el servidor de Glossy, y un archivo escrito
  antes de esta versión conserva el proveedor que ya tenía, así que no hay que
  volver a introducir nada. Las credenciales del último grupo salieron de ese panel
  a **Extensiones · mi propia API**, un panel propio al final de la ventana, donde
  quedan atenuadas hasta que se elige uno de esos servicios. En lo guardado nada
  cambió —el servidor de Glossy y el modelo local siguen siendo el canal de nube, el
  resto sigue usando el canal de API— así que una versión anterior lee el archivo
  como siempre. El servidor ganó un segundo proveedor: traduce con la cuenta de LLM
  cuando hay una configurada y recurre a las credenciales de Baidu que ya tenía, de
  modo que una instalación funciona en cualquiera de los dos casos.

- **La dirección del servidor de traducción ya no es un campo.** El servicio propio
  de Glossy es el que no necesita configurar nada, y una caja de texto para la
  dirección solo daba una forma de romperlo: una dirección equivocada o antigua
  dejaba la aplicación sin poder alcanzar ningún servidor, que es justo lo que podía
  contener el archivo de ajustes de una instalación actualizada. Ahora la dirección
  forma parte de la compilación: gana la que se usó al compilar, un valor sobrante en
  un archivo antiguo se ignora, y la ventana sigue mostrando lo que queda del cupo
  del día con su botón `Comprobar de nuevo`. Ejecutar tu propio despliegue es cambiar
  una línea (`DEFAULT_ENDPOINT` en `src-tauri/src/translate/cloud.rs`); una
  compilación sin dirección sigue respetando el ajuste.

### Corregido

- Un segundo inicio de Glossy muestra ahora el mismo aviso en la esquina de la
  pantalla que mostraría un inicio propio, en lugar de un diálogo de Windows sin
  estilo. La instancia en ejecución se avisa mediante un evento con nombre y
  responde abriendo la ventana de ajustes si ya está en pantalla, o mostrando el
  aviso si no lo está; el diálogo queda solo para el caso en que nadie responda.
