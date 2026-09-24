# Glossy · Privacy

<a id="en"></a>

## English

Glossy has no account, no analytics and no telemetry, and it does not report what you
select. Text leaves the machine only when you ask for a translation, and then only the
text you asked about.

**Where the text goes** — the target you picked in the settings:

- **Google Translate** (`translate.googleapis.com`) — a public endpoint run by Google.
- **The built-in engines** — the project's own relay, a Cloudflare Worker whose source is
  in [`server/`](./server/README.md), which passes the text on to Baidu, Youdao or Zhipu
  depending on how it is deployed. It counts the characters a day per installation and per
  address and stores nothing else: the text is not logged and not kept. The installation is
  identified by a random id generated on the first run.
- **A provider with your own key** — DeepL and the like, contacted directly with the
  credential you entered.

**Definitions** for single words come from `api.dictionaryapi.dev`, a public service that
just receives the word.

**Exchange rates** are fetched when a currency amount is recognised, from the relay or from
a public rates endpoint, and carry a base currency code and nothing about you.

**Update checks** are off by default. Turned on in the settings, the app asks GitHub for
`releases/latest` of this repository.

**What is stored locally**, in `%APPDATA%\com.glossy.translator\`: `settings.json` (with
every credential encrypted for your Windows login through DPAPI), `history.json` and a
small `rates.json` cache. Nothing in that folder is uploaded; deleting it removes the
settings, the history and the stored keys.

The services above receive what they are sent under their own privacy policies.

<a id="zh-cn"></a>

## 中文

Glossy 没有账号、没有统计分析、没有遥测，也不会记录你选了什么。只有当你要求翻译时，文本才会离开
本机，而且只发送你要求翻译的那段文字。

**文本发给谁** —— 取决于你在设置里选的渠道：

- **Google 翻译**（`translate.googleapis.com`）—— Google 提供的公共端点。
- **内置引擎** —— 项目自己的中转服务，源码在 [`server/`](./server/README.md)，是一个
  Cloudflare Worker，按部署配置把文本转给百度、有道或智谱。它只按"每台设备 / 每个地址 / 每天"
  统计字符数，其余什么都不存：文本不写日志、不落盘。设备用一个首次运行时随机生成的安装 ID 标识。
- **自填密钥的渠道** —— DeepL 等，用你填的凭据直接访问。

单词释义来自公共的 `api.dictionaryapi.dev`，它只会收到那个单词。

**汇率**在识别到货币金额时查询，走中转服务或公开的汇率端点，只带一个基准货币代码，不含任何与你
相关的信息。

**更新检查**默认关闭。在设置里打开后，应用会向 GitHub 查询本仓库的 `releases/latest`。

**本地存储**在 `%APPDATA%\com.glossy.translator\`：`settings.json`（其中所有凭据都用 DPAPI 以
当前 Windows 登录加密）、`history.json`，以及一个小的 `rates.json` 缓存。该目录的内容不会上传；
删掉它就等于删掉了设置、历史和已保存的密钥。

上述服务按照它们各自的隐私政策处理收到的内容。

<a id="es"></a>

## Español

Glossy no tiene cuentas, ni analítica, ni telemetría, y no registra lo que seleccionas. El
texto sale del equipo solo cuando pides una traducción, y solo el texto que pediste.

**A dónde va el texto** — según el destino elegido en los ajustes:

- **Google Translate** (`translate.googleapis.com`) — un punto público de Google.
- **Los motores integrados** — el relé del propio proyecto, un Worker de Cloudflare cuyo
  código está en [`server/`](./server/README.md), que reenvía el texto a Baidu, Youdao o
  Zhipu según cómo esté desplegado. Solo cuenta los caracteres por día, por instalación y
  por dirección: el texto no se registra ni se conserva. La instalación se identifica con un
  id aleatorio creado en el primer arranque.
- **Un proveedor con tu propia clave** — DeepL y similares, contactados directamente con la
  credencial que introdujiste.

Las **definiciones** de palabras sueltas vienen de `api.dictionaryapi.dev`, un servicio
público que solo recibe la palabra.

Los **tipos de cambio** se consultan cuando se reconoce un importe en divisas, desde el relé
o desde un punto público de cotizaciones, y llevan un código de divisa base y nada sobre ti.

La **comprobación de actualizaciones** está desactivada por defecto. Si la activas en los
ajustes, la aplicación pide a GitHub `releases/latest` de este repositorio.

**Lo que se guarda en el equipo**, en `%APPDATA%\com.glossy.translator\`: `settings.json`
(con cada credencial cifrada para tu sesión de Windows mediante DPAPI), `history.json` y una
pequeña caché `rates.json`. Nada de esa carpeta se sube; borrarla elimina los ajustes, el
historial y las claves guardadas.

Los servicios anteriores tratan lo que reciben según sus propias políticas de privacidad.
