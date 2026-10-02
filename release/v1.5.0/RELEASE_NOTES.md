[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The release where Glossy stops translating only what the mouse can reach. Three global
shortcuts instead of one, a card that says what it was an answer to, and two ways of
handing it text a selection cannot carry: a rectangle drawn over the screen, and a whole
document. The mouse is still the default — a selection still waits for the icon to be
clicked — but nothing about a translation has to start with the mouse any more.

### Added

- **Three global shortcuts, each one recorded and each one checked.** `hotkey` stays the
  translate shortcut (`Ctrl+Alt+C`), `hotkey_settings` brings the window to the front
  (`Ctrl+Alt+G`) and `hotkey_ocr` starts a screen reading (`Ctrl+Alt+Q`). `hotkey::Slot`
  is what the hook reports, so the callback decides what a press means instead of
  assuming every combination is a translation, and the settings page lists all three with
  their own field, their own clash line and their own `Clear`. A slot left empty is not
  registered at all, and one Windows refuses is reported without taking the others down.
- **Reading text off the screen.** A screen shot of the whole desktop is taken behind a
  frozen-looking overlay, a rectangle is dragged over the text, and what is inside it goes
  to Glossy's server, which asks Baidu OCR and answers with the recognised text. That text
  is then translated by the ordinary popup path — the same card, the same allowances, the
  same history. `Esc`, a right click or a rectangle that selects nothing cancels without a
  request; the overlay is a real window above everything, so the screenshot cannot capture
  the overlay itself.
- **Translating a document.** The **Document** page under *Features* takes a `.txt`, `.md`
  or `.srt` file and translates the parts of it that are prose: Markdown keeps its code
  fences, its headings and its lists, and a subtitle file keeps its cue numbers and time
  codes, so the result lines up with the original. Text is sent paragraph by paragraph,
  capped at 1500 characters per request and 60 000 characters per document, and the page
  reports how far along it is and can be cancelled between requests. **Save the
  translation** writes `<name>.<target language>.<ext>` into `Documents` and never touches
  the file that was read. A file that is not UTF-8 — a BOM, or the system code page — is
  decoded rather than reported as unreadable, which is what `platform::windows::encoding`
  is for.
- **The card shows what was translated.** Both shapes of card — the word card and the
  sentence card — now open with the grey original text under the language row, not just the
  sentence card as before, and that grey text can be edited in place: click it, correct the
  selection Glossy was handed, and `Ctrl+Enter` or clicking away translates the corrected
  text instead. Nothing is sent while the caret is still in the field, an empty or
  unchanged text costs nothing, and `Esc` puts the translated text back without closing the
  card.
- **The Glossy mark closes the line that names the engine.** The row that says which
  service answered ends with the translucent logo and the wordmark, so a result that came
  through somebody else's endpoint still says whose window it is in.

### Changed

- **A shortcut is no longer assumed to mean "translate".** The hook passes the slot it
  fired on, the tray and the screen reader are reached from the same thread the hotkey
  already used, and the window a shortcut opens is raised whether it was hidden or
  minimised.
- **The selection path is unchanged on purpose.** A drag still costs nothing until the
  icon is clicked: the icon appears under the selection, clicking it translates, and
  clicking anywhere else makes it disappear. A shortcut, by contrast, translates at once —
  there is nothing to click after it, which is the whole difference between the two.

<a id="zh-cn"></a>

## 中文

这个版本让 Glossy 不再只能翻译鼠标够得着的东西。三个全局快捷键取代了原来的一个，卡片会显示自己是对什么做出的回答，还有两条把选区带不动的文字交给它的路：在屏幕上框一个矩形，以及整个文档。鼠标依然是默认方式——划词之后仍然要点一下图标——但翻译已经不必从鼠标开始。

### 新增

- **三个全局快捷键，各自录制、各自检查。** `hotkey` 仍然是翻译快捷键（`Ctrl+Alt+C`），`hotkey_settings` 把窗口切到最前（`Ctrl+Alt+G`），`hotkey_ocr` 开始一次屏幕取字（`Ctrl+Alt+Q`）。钩子上报的是 `hotkey::Slot`，所以回调自己决定这次按下是什么意思，而不是假定每个组合都是翻译；设置页把三个都列出来，各自有输入框、各自的冲突提示和各自的「清除」。留空的槽位完全不注册，被 Windows 拒绝的一个也只报告自己，不会带倒另外两个。
- **从屏幕上读文字。** 在看起来被定住的遮罩后面，先对整块桌面拍一张照，你拖动一个矩形框住要翻译的文字，框内的内容被送到 Glossy 的服务器，由服务器向百度 OCR 要结果并返回识别出的文字。这段文字随后走普通弹窗那条路翻译——同一张卡片、同一份额度、同一条历史。按 `Esc`、点右键，或者框出的区域没有选中任何东西，都会取消且不发出请求；遮罩是一个真正在最上层的窗口，所以截图本身不可能拍到遮罩。
- **文档翻译。** **功能**下的**文档翻译**页接收一个 `.txt`、`.md` 或 `.srt` 文件，只翻译其中属于正文的部分：Markdown 保留它的代码块、标题和列表，字幕文件保留它的序号和时间码，所以译文和原文是对得上的。文本按段落发送，每次请求最多 1500 个字符、整个文档最多 60 000 个字符；页面会报告进度，并可以在两次请求之间取消。**保存译文**把 `<原名>.<目标语言>.<扩展名>` 写进「文档」文件夹，原文件绝不会被动到。不是 UTF-8 的文件——带 BOM 的，或者系统代码页的——会被解码，而不是被当作无法读取，这正是 `platform::windows::encoding` 的用途。
- **卡片会显示翻译的是什么。** 两种卡片——单词卡和句子卡——现在都在语言栏下方以灰色显示原文，而不再只有句子卡这样做；这段灰色文字可以就地编辑：点它，改掉 Glossy 拿到的那段选区，然后按 `Ctrl+Enter` 或点开别处，就会翻译改过的文字。光标还在字段里时不会发出任何请求，文本为空或没有改动也不会有任何消耗，按 `Esc` 会把译文对应的那段原文放回去，并且不会关闭卡片。
- **Glossy 标识收住了点名渠道的那一行。** 显示由哪个服务回答的那一行，末尾是半透明的 logo 和字标，所以即便结果来自别人的端点，这张卡片也仍然说明自己是谁的窗口。

### 变更

- **快捷键不再被假定为「翻译」。** 钩子把触发的是哪个槽位一并传出，托盘和朗读走的是快捷键本来就在用的那同一个线程，快捷键打开的窗口无论是隐藏还是最小化都会被切到最前。
- **划词那条路有意保持不变。** 拖动一下依然不花任何额度，直到你点击图标：图标出现在选区下方，点击它才翻译，点别处则消失。相比之下，快捷键按下即翻译——它后面没有任何东西要点，这就是两者的全部区别。

<a id="es"></a>

## Español

Esta es la versión en la que Glossy deja de traducir solo lo que alcanza el ratón. Tres atajos globales en lugar de uno, una tarjeta que dice a qué fue una respuesta y dos formas de darle texto que una selección no puede llevar: un rectángulo dibujado sobre la pantalla y un documento completo. El ratón sigue siendo lo predeterminado —una selección todavía espera a que se pulse el icono—, pero ya nada de una traducción tiene que empezar con el ratón.

### Añadido

- **Tres atajos globales, cada uno grabado y cada uno comprobado.** `hotkey` sigue siendo el atajo de traducción (`Ctrl+Alt+C`), `hotkey_settings` trae la ventana al frente (`Ctrl+Alt+G`) y `hotkey_ocr` inicia una lectura de pantalla (`Ctrl+Alt+Q`). Lo que notifica el enganche es `hotkey::Slot`, así que la devolución de llamada decide qué significa una pulsación en lugar de suponer que toda combinación es una traducción, y la página de ajustes lista los tres con su propia caja, su propia línea de conflicto y su propio `Clear`. Una ranura vacía no se registra en absoluto, y una que Windows rechaza se informa sin llevarse por delante a las otras.
- **Leer texto de la pantalla.** Se toma una captura de todo el escritorio por detrás de una capa congelada, se arrastra un rectángulo sobre el texto y lo que queda dentro va al servidor de Glossy, que lo pide a Baidu OCR y responde con el texto reconocido. Ese texto se traduce después por la vía normal del emergente: la misma tarjeta, la misma cuota, el mismo historial. `Esc`, el botón derecho o un rectángulo que no selecciona nada cancelan sin petición; la capa es una ventana real por encima de todo, así que la captura no puede fotografiarla.
- **Traducir un documento.** La página **Document** dentro de *Features* toma un archivo `.txt`, `.md` o `.srt` y traduce la parte que es prosa: Markdown conserva sus bloques de código, sus títulos y sus listas, y un archivo de subtítulos conserva sus números de entrada y sus códigos de tiempo, así el resultado cuadra con el original. El texto se envía párrafo a párrafo, con un tope de 1500 caracteres por petición y 60 000 caracteres por documento; la página informa del avance y se puede cancelar entre peticiones. **Save the translation** escribe `<nombre>.<idioma de destino>.<ext>` en `Documentos` y nunca toca el archivo que se leyó. Un archivo que no es UTF-8 —con BOM o en la página de códigos del sistema— se decodifica en lugar de darse por ilegible, que es para lo que existe `platform::windows::encoding`.
- **La tarjeta muestra qué se tradujo.** Las dos formas de tarjeta —la de palabra y la de oración— abren ahora con el original en gris bajo la fila de idiomas, y no solo la de oración como antes; ese texto gris se puede editar en el sitio: haz clic en él, corrige la selección que recibió Glossy y `Ctrl+Enter` o hacer clic fuera traduce el texto corregido en su lugar. No se envía nada mientras el cursor sigue en el campo, un texto vacío o sin cambios no cuesta nada, y `Esc` devuelve el texto traducido sin cerrar la tarjeta.
- **La marca Glossy cierra la línea que nombra el motor.** La fila que dice qué servicio respondió termina con el logotipo translúcido y el nombre, así un resultado que llegó por el punto de otro sigue diciendo de qué ventana es.

### Cambiado

- **Un atajo ya no se supone que signifique "traducir".** El enganche pasa la ranura que se disparó, la bandeja y el lector de pantalla se alcanzan desde el mismo hilo que el atajo ya usaba, y la ventana que abre un atajo sube al frente tanto si estaba oculta como minimizada.
- **La vía de la selección no cambia a propósito.** Un arrastre sigue sin costar nada hasta que se pulsa el icono: el icono aparece bajo la selección, pulsarlo traduce y pulsar en otro sitio lo hace desaparecer. Un atajo, en cambio, traduce al instante —después no hay nada que pulsar, y esa es toda la diferencia entre los dos.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
