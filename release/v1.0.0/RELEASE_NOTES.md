[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The first stable release. Glossy now ships with a server of its own, so it can be
installed and used without obtaining an API key first.

### Added

- **Glossy Cloud**, a translation provider that talks to a server deployed from
  the new `server/` directory instead of to a vendor, so there is nothing to fill
  in: the account behind the server pays for the translation and the keys never
  reach the app. The settings window asks that server how much of today's
  allowance is left and shows the answer under the provider row, with **Check
  again** next to it.
- `server/`, a translation proxy that keeps the provider credentials on the
  machine that already holds them and hands each client a daily character
  allowance instead. It runs unchanged on a Cloudflare Worker (with a Durable
  Object for exact counting) and on a Node 18 runtime as a Tencent Cloud SCF Web
  function. The allowance is checked per installation, per caller address and
  globally, requests are capped per minute and per request, and the caller address
  is read in a way a client cannot spoof into a fresh bucket.
- An install id, generated once so the server counts one device's characters
  rather than one launch's.

### Changed

- A fresh install starts on the **cloud** provider instead of **Google**, so the
  first selection translates without anything being configured or obtained.
  Existing settings keep the provider they were saved with.
- The new Glossy mark is the icon of the application, of the installer and of the
  tray, and it replaces the drawn `G` at the top of the settings window and on the
  start card.

### Fixed

- The language selectors in the popup card are readable in dark mode. Windows
  paints a `select` and its native option list with the background of the control,
  so the translucent fill left the language names sitting on the desktop behind
  the popup; both the popup selectors and the settings window's now use an opaque
  surface with a visible border.

<a id="zh-cn"></a>

## 中文

第一个稳定版。Glossy 现在自带服务器，装好就能用，不需要先去申请 API key。

### 新增

- **Glossy Cloud**：新增的翻译服务商，直连由新的 `server/` 目录部署出来的服务器，而不是直连翻译厂商，所以应用里什么都不用填——翻译费用由服务器背后的账号承担，密钥也不会下发到应用。设置窗口会向服务器查询今天还剩多少额度，并把结果显示在服务商那一行下面，旁边有**重新查询**。
- `server/`：翻译代理服务，把厂商密钥留在本来就持有它的机器上，只给每个客户端发放每日字符额度。同一份代码可以原样运行在两种环境上——Cloudflare Worker（用 Durable Object 做精确计数）和 Node 18 运行时的腾讯云 SCF Web 函数。额度按安装、按调用方地址、以及全局三个维度检查，请求有限速和单次长度上限，调用方地址的解析方式让客户端无法伪造成一个新的额度桶。
- 安装标识（install id）：只生成一次，这样服务器统计的是「一台设备」的字数，而不是「一次启动」的字数。

### 变更

- 全新安装默认使用 **cloud** 服务商，而不再是 **Google**，因此第一次划词翻译不需要任何配置或申请。已有设置里保存的服务商保持不变。
- 新的 Glossy 图标成为应用、安装包和托盘栏的图标，并替换了设置窗口顶部和开始卡片上原来用 CSS 画的 `G`。

### 修复

- 弹窗卡片里的语言选择框在深色模式下可以看清了。Windows 会用控件自身的背景色去画 `select` 以及它弹出的原生选项列表，之前卡片的半透明填充导致语言名直接叠在弹窗后面的桌面上；现在弹窗里的选择框和设置窗口里的都改用不透明背景，并带上了可见的边框。

<a id="es"></a>

## Español

La primera versión estable. Glossy ahora trae su propio servidor, así que se
instala y se usa sin tener que obtener antes una clave de API.

### Añadido

- **Glossy Cloud**, un proveedor de traducción que habla con un servidor
  desplegado desde el nuevo directorio `server/` en lugar de con un proveedor
  externo, así que no hay nada que rellenar: la cuenta que hay detrás del servidor
  paga la traducción y las claves nunca llegan a la aplicación. La ventana de
  ajustes pregunta a ese servidor cuánto queda de la cuota de hoy y muestra la
  respuesta bajo la fila del proveedor, con **Volver a comprobar** al lado.
- `server/`, un proxy de traducción que se queda con las credenciales del
  proveedor en la máquina que ya las tiene y, en su lugar, entrega a cada cliente
  una cuota diaria de caracteres. Funciona sin cambios en un Cloudflare Worker (con
  un Durable Object para un recuento exacto) y en un entorno de Node 18 como
  función web de Tencent Cloud SCF. La cuota se comprueba por instalación, por
  dirección de origen y de forma global; las peticiones tienen límite por minuto y
  por petición, y la dirección de origen se lee de una forma que un cliente no
  puede falsificar para conseguir una cuota nueva.
- Un identificador de instalación, generado una sola vez para que el servidor
  cuente los caracteres de un dispositivo y no los de un arranque.

### Cambios

- Una instalación nueva empieza en el proveedor **cloud** en lugar de **Google**,
  así que la primera selección se traduce sin configurar ni obtener nada. Los
  ajustes existentes conservan el proveedor con el que se guardaron.
- La nueva marca de Glossy es el icono de la aplicación, del instalador y de la
  bandeja, y sustituye la `G` dibujada en la parte superior de la ventana de
  ajustes y en la tarjeta de inicio.

### Correcciones

- Los selectores de idioma de la tarjeta emergente se leen bien en modo oscuro.
  Windows pinta un `select` y su lista nativa de opciones con el fondo del propio
  control, así que el relleno translúcido dejaba los nombres de los idiomas sobre
  el escritorio que quedaba detrás de la tarjeta; ahora tanto los selectores de la
  tarjeta como los de la ventana de ajustes usan una superficie opaca con un borde
  visible.
