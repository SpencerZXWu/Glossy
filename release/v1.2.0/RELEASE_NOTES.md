[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

Windows is no longer the operating system of the whole backend: every call into Win32 now
lives behind one dispatch layer, so a port has a single place to stand. The window content
is governed by an explicit Content Security Policy instead of none, and a release ships
three ways — the installer, an MSI package and a portable zip. macOS and Linux are still
ahead (`src/platform/mod.rs` refuses to compile anywhere but Windows), and this release
deliberately stops at the interface those ports will be built against.

### Added

- **The platform layer.** `src/platform/mod.rs` publishes what the rest of the backend is
  allowed to assume about an operating system — without a screen, `ScreenRect` and its
  padded hit test — and dispatches to `src/platform/windows/`, which holds the ten modules
  that talk to Win32: `clipboard`, `console`, `desktop`, `hotkey`, `input`, `input_hook`,
  `instance`, `secrets`, `speech` and `uia`. A target that is not Windows fails to compile
  with a message pointing at the layer instead of failing to link. No module in it leaks a
  `windows` type through its public surface: the low-level mouse hook reports a
  `Click { pressed, x, y }`, the window handle travels as an `isize`, and the accessibility
  text read for a word's sentence is asked for as a `Unit::Line` or `Unit::Paragraph`.
- **An MSI package and a portable zip, next to the installer.** `bundle.targets` builds
  NSIS and MSI, and `scripts/release.ps1` stages both, then packs
  `Glossy_<version>_x64_portable.zip` — the release binary and the `WebView2Loader.dll` the
  GNU toolchain links against, with nothing to install — and checksums all three. A build
  that names MSI in its targets but produces no MSI fails the release instead of staging
  two files out of three.

### Security

- **The content is behind a policy now.** `app.security.csp` was `null`, which let the
  window load anything a script asked for. It is an explicit whitelist: `default-src`,
  `script-src`, `style-src` and `font-src` are `'self'`, images may add `data:`, and
  `object-src`, `frame-src` and `form-action` are closed. The frontend loads no remote
  script, style, font or image, so nothing had to be widened for it — verified in the
  running app by reading each window's console, where a deliberate remote `fetch` is
  refused and nothing this app loads is.

### Changed

- **The backend's OS-bound modules moved under `src/platform/windows/`.** `clipboard.rs`,
  `console.rs`, `hotkey.rs`, `input.rs`, `instance.rs`, `secrets.rs`, `speech.rs` and
  `platform.rs` (now `desktop.rs`) were moved, not rewritten, so the behaviour of a
  selection is the same one as before. What they exported is reached through the layer
  now: `platform::clipboard`, `platform::hotkey`, `platform::speech`, `platform::instance`
  and `platform::desktop` from the window setup, the popup, the hint, the settings and the
  console borrow. The mouse hook itself was split: the thread, the message loop and the
  callback plumbing live in `platform::windows::input_hook`, and the click chain that
  decides what a gesture meant stays in `selection.rs`, where it can be tested without a
  screen.
- **`windows` and `window-vibrancy` are Windows-only dependencies.** `Cargo.toml` declares
  them under `[target.'cfg(windows)'.dependencies]`, so the tree a port starts from does
  not contain them at all.
- **The screenshot-shaped parts still inside the Windows modules are named as debt.** The
  hotkey's key parsing, the speech queue, `secrets::is_protected`, `context::sentence_in`
  and the click chain in `selection.rs` are portable but still sit in OS-bound files; the
  capability table in `src/platform/mod.rs` says so, and the roadmap records it.

### Fixed

- **The start hint goes away every time it appears.** A 6.5 second timer in the page took it
  off screen, and the page loads once: the timer ran for the first hint and never again, so
  the card a later start of Glossy put back stayed there for as long as the process lived.
  Starting Glossy a second time — double clicking the shortcut again, which is how most
  people reopen the settings window — is exactly the path that showed it. The countdown runs
  in `notice.rs` now, started by `notice::show` itself and guarded by a show counter, so
  every show gets a full one and a countdown left over from an earlier hint cannot take down
  the one that replaced it. `notice.js` no longer arms a timer.
- **The names that live only in an attribute are translated.** `i18n.apply` fills
  `data-i18n-aria-label` beside `data-i18n-title`, which is what the close button of the hint
  and the interface language selector were missing: both had an English tooltip and an
  English accessible name in every language, and the selector's `ui.lang` key — written for
  exactly this control — had no reader left. It has one again.
- **A counted sentence reads correctly for one item.** `"{0} program(s) ignored."` printed
  its own brackets at one program. `ignored.count` and `source.count` are stored as
  `key.one` and `key.other` and read through the new `i18n.plural`, so a single item says
  `1 program ignored.` The Chinese text is unchanged.
- **Choosing a language no longer takes the card away.** The list of a language selector is
  a window of its own, drawn by the browser process of the webview and taller than the card,
  so pressing an entry fell outside the card's edges: the mouse hook read it as a click on
  the program behind Glossy and dismissed the card mid-choice. A click now also belongs to
  the card when the window under it hangs off one of the card's windows or is drawn by the
  process that draws the card, so entries are pressed normally and only a click that really
  lands elsewhere closes the card.

<a id="zh-cn"></a>

## 中文

Windows 不再是整个后端所依附的操作系统：每一处对 Win32 的调用现在都位于同一个分发层之后，移植因此有了唯一的立足点。窗口内容由一条明确的内容安全策略（CSP）约束，而不再是不设防，并且一次发布同时提供三种产物——安装程序、MSI 安装包和便携版压缩包。macOS 和 Linux 仍在前方（`src/platform/mod.rs` 拒绝在 Windows 之外的平台编译），本次发布有意只做到这些移植将要依托的接口为止。

### 新增

- **平台层。** `src/platform/mod.rs` 说明后端其余部分可以假定操作系统提供什么——没有屏幕时的 `ScreenRect` 及其带留白的命中测试——并把调用分发到 `src/platform/windows/`，那里存放与 Win32 打交道的十个模块：`clipboard`、`console`、`desktop`、`hotkey`、`input`、`input_hook`、`instance`、`secrets`、`speech` 和 `uia`。非 Windows 目标会在编译期失败，错误信息指向这一层，而不是在链接期才失败。该层中没有任何模块通过公开接口泄漏 `windows` 类型：底层鼠标钩子上报 `Click { pressed, x, y }`，窗口句柄以 `isize` 传递，为取某个词的句子而读取的无障碍文本则以 `Unit::Line` 或 `Unit::Paragraph` 的形式请求。
- **在安装程序之外，还有 MSI 安装包和便携版压缩包。** `bundle.targets` 同时构建 NSIS 与 MSI，`scripts/release.ps1` 会暂存两者，然后打包 `Glossy_<version>_x64_portable.zip`——其中是发布版可执行文件以及 GNU 工具链所链接的 `WebView2Loader.dll`，无需安装任何东西——并为三者生成校验和。若构建目标中声明了 MSI 却没有产出 MSI，发布将直接失败，而不是只暂存三份中的两份。

### 安全

- **窗口内容现在受策略约束。** 原先 `app.security.csp` 为 `null`，脚本请求什么，窗口就能加载什么。现在是一份显式白名单：`default-src`、`script-src`、`style-src`、`font-src` 均为 `'self'`，图片额外允许 `data:`，而 `object-src`、`frame-src`、`form-action` 一律关闭。前端不加载任何远程脚本、样式、字体或图片，因此无需为它放宽任何一条——这一点在运行中的应用里逐个窗口读取控制台得到验证：故意发起的远程 `fetch` 被拒绝，而本应用所加载的资源都没有被拒绝。

### 变更

- **后端与操作系统绑定的模块已移动到 `src/platform/windows/` 下。** `clipboard.rs`、`console.rs`、`hotkey.rs`、`input.rs`、`instance.rs`、`secrets.rs`、`speech.rs` 和 `platform.rs`（现为 `desktop.rs`）是移动而非重写，因此选取文本的行为与之前完全一致。它们导出的内容现在通过该层访问：窗口初始化、弹窗、提示窗、设置页和控制台借用分别使用 `platform::clipboard`、`platform::hotkey`、`platform::speech`、`platform::instance` 和 `platform::desktop`。鼠标钩子本身被拆分：线程、消息循环与回调串联位于 `platform::windows::input_hook`，判定手势含义的点击链留在 `selection.rs`，因此无需屏幕即可测试。
- **`windows` 与 `window-vibrancy` 成为仅 Windows 的依赖。** `Cargo.toml` 将它们声明在 `[target.'cfg(windows)'.dependencies]` 下，移植工作的起点依赖树中不再包含它们。
- **仍留在 Windows 模块中的、与平台相关的部分被明确记为技术债。** 热键的按键解析、朗读队列、`secrets::is_protected`、`context::sentence_in` 以及 `selection.rs` 中的点击链本身是可移植的，但仍位于与操作系统绑定的文件里；`src/platform/mod.rs` 的能力表写明了这一点，路线图也记录了它。

### 修复

- **启动提示窗现在每次出现都会消失。** 页面中一个 6.5 秒的定时器负责把它移出屏幕，而页面只加载一次：定时器只在第一次提示时运行过，此后再也没有运行，于是 Glossy 之后某次启动重新放回的卡片会一直留在屏幕上，直到进程结束。再次启动 Glossy——再次双击快捷方式，这也是多数人重新打开设置窗口的方式——正是会复现它的那条路径。倒计时现在位于 `notice.rs`，由 `notice::show` 自己启动，并以显示计数器加以保护，因此每次显示都会得到完整的一轮倒计时，早先提示残留的倒计时也不会把后来者关掉。`notice.js` 不再设置定时器。
- **只存在于属性中的名称也会被翻译。** `i18n.apply` 现在会在 `data-i18n-title` 之外填充 `data-i18n-aria-label`，这正是提示窗的关闭按钮与界面语言选择器所缺失的：两者在任何语言下都显示英文提示和英文无障碍名称，而选择器的 `ui.lang` 键——本就为这个控件而写——却已无人读取。现在又有读者了。
- **计数句子在只有一项时读法正确。** `"{0} program(s) ignored."` 在只有一个程序时会把它自己的括号一起打印出来。`ignored.count` 与 `source.count` 现在分别以 `key.one` 和 `key.other` 存储，并通过新增的 `i18n.plural` 读取，因此单项时会显示 `1 program ignored.`。中文文案保持不变。
- **选择语言不再让卡片消失。** 语言选择器的下拉列表是一个独立窗口，由 webview 的浏览器进程绘制，且比卡片更高，因此按下其中一项时落点已在卡片边界之外：鼠标钩子把它当成点击 Glossy 身后的程序，于是在选择过程中收起了卡片。现在，只要落点下方的窗口挂在卡片某个窗口之下、或由绘制卡片的那个进程绘制，这次点击同样算作卡片上的点击，因此列表项可以正常按下，只有真正落在别处的点击才会关闭卡片。

<a id="es"></a>

## Español

Windows ya no es el sistema operativo de todo el backend: cada llamada a Win32 vive ahora
detrás de una única capa de despacho, de modo que un port tiene un solo lugar donde
apoyarse. El contenido de las ventanas se rige por una política de seguridad de contenido
explícita en lugar de ninguna, y una versión se distribuye de tres maneras: el instalador,
un paquete MSI y un zip portátil. macOS y Linux siguen por delante (`src/platform/mod.rs`
se niega a compilar en cualquier plataforma que no sea Windows), y esta versión se detiene
deliberadamente en la interfaz sobre la que se construirán esos ports.

### Añadido

- **La capa de plataforma.** `src/platform/mod.rs` publica lo que el resto del backend
  puede dar por supuesto sobre un sistema operativo —sin pantalla, `ScreenRect` y su test
  de impacto con margen— y despacha a `src/platform/windows/`, que contiene los diez
  módulos que hablan con Win32: `clipboard`, `console`, `desktop`, `hotkey`, `input`,
  `input_hook`, `instance`, `secrets`, `speech` y `uia`. Un destino que no sea Windows no
  compila y el mensaje apunta a la capa, en lugar de fallar al enlazar. Ningún módulo de
  ella filtra un tipo de `windows` por su interfaz pública: el hook de ratón de bajo nivel
  informa un `Click { pressed, x, y }`, el manejador de ventana viaja como `isize` y el
  texto de accesibilidad leído para la frase de una palabra se pide como `Unit::Line` o
  `Unit::Paragraph`.
- **Un paquete MSI y un zip portátil, junto al instalador.** `bundle.targets` construye
  NSIS y MSI, y `scripts/release.ps1` prepara ambos, empaqueta
  `Glossy_<version>_x64_portable.zip` —el binario de la versión y el `WebView2Loader.dll`
  que enlaza la cadena de herramientas GNU, sin nada que instalar— y calcula las sumas de
  comprobación de los tres. Una compilación que declara MSI entre sus objetivos pero no
  produce ningún MSI falla la versión en lugar de preparar dos archivos de tres.

### Seguridad

- **El contenido está bajo una política.** `app.security.csp` era `null`, lo que permitía a
  la ventana cargar cualquier cosa que pidiera un script. Ahora es una lista blanca
  explícita: `default-src`, `script-src`, `style-src` y `font-src` son `'self'`, las
  imágenes pueden añadir `data:`, y `object-src`, `frame-src` y `form-action` están
  cerrados. El frontend no carga ningún script, estilo, fuente ni imagen remotos, así que
  no hubo que ampliar nada para él: verificado en la aplicación en marcha leyendo la consola
  de cada ventana, donde un `fetch` remoto deliberado es rechazado y nada de lo que carga
  esta aplicación lo es.

### Cambios

- **Los módulos del backend ligados al sistema operativo se movieron a
  `src/platform/windows/`.** `clipboard.rs`, `console.rs`, `hotkey.rs`, `input.rs`,
  `instance.rs`, `secrets.rs`, `speech.rs` y `platform.rs` (ahora `desktop.rs`) se movieron,
  no se reescribieron, así que el comportamiento de una selección es el mismo de antes. Lo
  que exportaban se alcanza ahora a través de la capa: `platform::clipboard`,
  `platform::hotkey`, `platform::speech`, `platform::instance` y `platform::desktop` desde
  el montaje de las ventanas, el popup, el aviso, los ajustes y el préstamo de la consola.
  El propio hook de ratón se dividió: el hilo, el bucle de mensajes y la conexión de
  devoluciones de llamada viven en `platform::windows::input_hook`, y la cadena de clics que
  decide qué significa un gesto se queda en `selection.rs`, donde se puede probar sin
  pantalla.
- **`windows` y `window-vibrancy` son dependencias solo de Windows.** `Cargo.toml` las
  declara bajo `[target.'cfg(windows)'.dependencies]`, así que el árbol desde el que parte
  un port no las contiene en absoluto.
- **Las partes con forma de captura de pantalla que siguen dentro de los módulos de Windows
  quedan nombradas como deuda.** El análisis de teclas del atajo, la cola de voz,
  `secrets::is_protected`, `context::sentence_in` y la cadena de clics de `selection.rs` son
  portables pero siguen en archivos ligados al sistema operativo; la tabla de capacidades de
  `src/platform/mod.rs` lo dice y la hoja de ruta lo registra.

### Corregido

- **El aviso de inicio desaparece cada vez que aparece.** Un temporizador de 6,5 segundos en
  la página lo quitaba de la pantalla, y la página se carga una vez: el temporizador corrió
  para el primer aviso y nunca más, de modo que la tarjeta que un arranque posterior de
  Glossy volvía a poner se quedaba ahí mientras viviera el proceso. Iniciar Glossy por
  segunda vez —volver a hacer doble clic en el acceso directo, que es como la mayoría
  reabre la ventana de ajustes— es exactamente el camino que lo mostraba. La cuenta atrás
  vive ahora en `notice.rs`, la inicia `notice::show` y la protege un contador de
  apariciones, así que cada aparición recibe una cuenta atrás completa y una sobrante de un
  aviso anterior no puede retirar el que la reemplazó. `notice.js` ya no arma ningún
  temporizador.
- **Los nombres que solo viven en un atributo se traducen.** `i18n.apply` rellena
  `data-i18n-aria-label` junto a `data-i18n-title`, que es lo que les faltaba al botón de
  cerrar del aviso y al selector de idioma de la interfaz: ambos tenían una descripción
  emergente y un nombre accesible en inglés en todos los idiomas, y la clave `ui.lang` del
  selector —escrita precisamente para este control— se había quedado sin lector. Vuelve a
  tenerlo.
- **Una frase contada se lee bien cuando hay un solo elemento.** `"{0} program(s)
  ignored."` imprimía sus propios corchetes con un solo programa. `ignored.count` y
  `source.count` se guardan como `key.one` y `key.other` y se leen mediante el nuevo
  `i18n.plural`, así que un solo elemento dice `1 program ignored.` El texto en chino no
  cambia.
- **Elegir un idioma ya no se lleva la tarjeta.** La lista de un selector de idioma es una
  ventana propia, dibujada por el proceso de navegador del webview y más alta que la
  tarjeta, así que pulsar una entrada caía fuera de los bordes de la tarjeta: el hook de
  ratón lo leía como un clic en el programa que hay detrás de Glossy y retiraba la tarjeta
  en mitad de la elección. Un clic también pertenece ahora a la tarjeta cuando la ventana
  que hay debajo cuelga de alguna ventana de la tarjeta o la dibuja el mismo proceso que la
  dibuja a ella, así que las entradas se pulsan con normalidad y solo un clic que cae de
  verdad en otro sitio cierra la tarjeta.
