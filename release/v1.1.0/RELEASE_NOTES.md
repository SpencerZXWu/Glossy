[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Added

- **The word card shows how the word behaves.** A word entry now carries its
  inflections (plural, third person, participles, past, comparative, superlative),
  the words that mean roughly the same, and — when **Reading** is on — the sentence
  the selection was taken from next to a translation of that sentence. The Rust side
  does the work: `morphology.rs` derives the forms, `translate/dictionary.rs` and
  `translate/mod.rs` collect and de-duplicate the alternatives, and `context.rs`
  looks the sentence up.

- **Sentence by sentence.** A paragraph whose translation divides differently than
  the original is no longer a wall of text: with **Sentence by sentence** on, the
  card lists the original and the translation one aligned row at a time. A
  translation that comes back as a single row is left as the plain paragraph, since
  a one-row table would only repeat the card.

- **Read it out loud.** Every card carries a pronunciation button for the original
  and one for the translation. The text is spoken through Windows SAPI at the rate
  chosen in the settings window (**Speaking rate**: Slow / Normal / Fast / Very
  fast), and pressing the button again stops it, as does closing the card.

- **A fallback order you can set.** When the chosen service fails, Glossy now walks
  a list of the other services instead of giving up. The settings window shows that
  list under **Fallback**: the service chosen above is always tried first and cannot
  be moved, the rest can be reordered with the arrows, and the whole thing can be
  turned off. The card names the service that answered when it was not the one
  asked for.

- **A compact card.** **Compact popup** keeps the translation, the phonetic symbols
  and the meanings, and leaves out the example, the inflections, the synonyms, the
  context sentence, the sentence-by-sentence view and the unit conversions — for a
  popup that is meant to be read at a glance.

- **Reading, and how much of it.** The settings window has a **Reading** panel that
  decides what a word card is allowed to show: **Word in its sentence** (the context
  lookup), **Sentence by sentence** (the aligned view) and **Compact popup**, plus
  the **Speaking rate** of the pronunciation buttons.

- **The selection is cleaned up before it is sent.** `text.rs` reduces what
  Ctrl+C handed over to the sentences the author wrote: soft hyphens and zero-width
  spaces, terminal escape sequences, line breaks the window width inserted, runs of
  non-breaking spaces and stray control characters are removed, so the provider is
  not asked to translate the layout of the page.

### Changed

- **Exporting the settings file no longer asks about keys.** With no field left
  anywhere that takes an API key, there is nothing an export could carry: the
  checkbox, its warning and the confirmation dialog are gone, and
  `export_settings` has no `include_credentials` argument any more. The file holds
  the choices and nothing secret.

- **Four translation services, and no key field anywhere.** The dropdown in the
  settings window now holds exactly four entries — **Baidu Translate** and **Youdao
  Translate** (the project's server, each naming the upstream it should use, so
  there is nothing to fill in), **Local translation · Ollama** (any
  OpenAI-compatible endpoint on this machine) and **Google** (the free public
  endpoint) — and the **Extensions · my own API** panel is gone, along with the
  Zhipu, DeepL and OpenAI entries it served. A new install starts on Baidu
  Translate. Nothing about the stored shape changed: `channel` + `cloudProvider` +
  `cloudVendor` / `provider` are written exactly as before, so a settings file from
  an older build keeps working — a `provider` this build no longer offers reads as
  the built-in Baidu entry and is replaced the next time the file is written, and a
  `cloudVendor` that is not `youdao` reads as Baidu. README and ROADMAP were updated
  in all three languages to match.

- **The original line in the card is capped at four lines.** A long selection used
  to push the translation down and out of view; the source block now scrolls inside
  itself once it would grow past four lines, in the floating popup and in the card
  of the settings window alike. The translation is what the card is read for, so it
  keeps its place.

### Fixed

- The confirmation that appears at the bottom of the settings window was invisible
  in dark mode while the window sat on the Mica backdrop: the toast drew its
  background from `--fg` and its text from `--bg`, and Mica remaps `--bg` to a 6 %
  white, so dark-mode text landed on a dark surface. The toast has its own
  `--g-toast-surface` / `--g-toast-text` / `--g-toast-outline` tokens now, and
  dialog boxes switched to a solid surface with the dialog shadow, so both read the
  same in either theme and under any backdrop.

<a id="zh-cn"></a>

## 中文

### 新增

- **单词卡片会说明这个词怎么变。** 词条现在带着它的词形变化（复数、第三人称、分词、过去式、比较级、最高级）、
  意思相近的词，以及——打开 **阅读** 时——它被选中时所在的句子和该句的译文。
  这些都由 Rust 一侧完成：`morphology.rs` 推出词形，`translate/dictionary.rs` 和 `translate/mod.rs`
  收集并去重这些替代项，`context.rs` 负责查找句子。

- **逐句对照。** 译文切分方式与原文不同的段落不再是一整面文字墙：打开 **原文与译文逐句对照** 后，
  卡片会把原文和译文按对齐的一行一句列出。如果译文只回来一行，就仍然按普通段落显示，
  因为只有一行的表格只是把卡片重复了一遍。

- **朗读出来。** 每张卡片都有两个朗读按钮，一个朗读原文，一个朗读译文。文本通过 Windows SAPI
  按设置窗口中选择的语速（**朗读语速**：慢 / 正常 / 快 / 很快）读出，再次按下按钮即可停止，
  关闭卡片也一样。

- **可以自己排的备用顺序。** 所选服务失败时，Glossy 不再直接放弃，而是依次尝试其他服务。
  设置窗口在 **备用服务** 下显示这个列表：上面选中的服务永远最先尝试且不能移动，
  其余的可以用箭头调整顺序，整个功能也可以关闭。当回答问题的不是首选服务时，卡片会写明是谁回答的。

- **精简卡片。** **精简卡片，不显示附加信息** 会保留译文、音标和释义，省略例句、词形变化、近义词、
  所在句子、逐句对照和单位换算——留给只求一眼看完的弹窗。

- **阅读，以及读多少。** 设置窗口新增 **阅读** 面板，决定单词卡片可以显示什么：
  **显示所查单词所在的句子**（上下句查找）、**原文与译文逐句对照**（对齐视图）、
  **精简卡片，不显示附加信息**，以及朗读按钮的 **朗读语速**。

- **选区在送出之前先清理一遍。** `text.rs` 把 Ctrl+C 交出来的内容还原成作者写下的句子：
  软连字符和零宽空格、终端转义序列、窗口宽度插入的换行、成串的不换行空格和多余的控制字符都会被去掉，
  这样翻译渠道就不会被要求翻译网页的排版。

### 变更

- **导出设置文件不再询问密钥。** 既然已经没有任何字段可以填 API 密钥，导出也就没有什么可带的：
  复选框、它的提示和确认对话框都已移除，`export_settings` 也不再接受 `include_credentials` 参数。
  文件里只有各项设置，没有任何机密。

- **四个翻译渠道，并且到处都没有密钥输入框。** 设置窗口的下拉框现在恰好四项——
  **百度翻译** 和 **有道翻译**（本项目的服务器，各自指定要用哪个上游，因此没有要填的东西）、
  **本地模型**（这台机器上任何兼容 OpenAI 的接口）和 **Google**（免费的公开接口）——
  而 **扩展 · 我的 API** 面板连同它服务的智谱、DeepL 和 OpenAI 条目一起消失了。
  全新安装默认使用百度翻译。存储结构完全没有变化：`channel` + `cloudProvider` +
  `cloudVendor` / `provider` 的写法与以前一模一样，所以旧版本写下的设置文件仍然可用——
  本版本不再提供的 `provider` 会读成内置的百度条目，并在下次写文件时被替换掉；
  不是 `youdao` 的 `cloudVendor` 会读成百度。README 和 ROADMAP 已同步更新三种语言。

- **卡片里的原文最多四行。** 过去很长的选区会把译文挤下去、挤出视野；现在原文区块一旦要超过四行，
  就在自身内部滚动，浮动弹窗和设置窗口里的卡片都是如此。卡片是用来看译文的，所以译文始终留得住位置。

### 修复

- 设置窗口底部出现的确认提示在深色模式下、窗口压在 Mica 背景上时看不见：
  该提示的背景取自 `--fg`、文字取自 `--bg`，而 Mica 会把 `--bg` 重映射为 6 % 的白色，
  于是深色模式的文字落在了深色表面上。现在该提示有了自己的 `--g-toast-surface` /
  `--g-toast-text` / `--g-toast-outline` 令牌，对话框也改用实色表面加上对话框阴影，
  因此在两种主题下、在任何背景之上读起来都是一样的。

<a id="es"></a>

## Español

### Añadido

- **La tarjeta de una palabra explica cómo se comporta la palabra.** Una entrada
  ahora incluye sus formas (plural, tercera persona, participios, pasado,
  comparativo, superlativo), las palabras que significan más o menos lo mismo y
  —con **Reading** activado— la frase de la que se tomó la selección junto a una
  traducción de esa frase. El trabajo lo hace el lado Rust: `morphology.rs` deriva
  las formas, `translate/dictionary.rs` y `translate/mod.rs` reúnen y depuran las
  alternativas, y `context.rs` busca la frase.

- **Frase por frase.** Un párrafo cuya traducción se divide de manera distinta que
  el original ya no es un muro de texto: con **Pair the original and the translation
  sentence by sentence** activado, la tarjeta lista el original y la traducción como
  filas alineadas. Una traducción que vuelve en una sola fila se deja como párrafo
  normal, porque una tabla de una fila solo repetiría la tarjeta.

- **Se puede leer en voz alta.** Cada tarjeta lleva un botón de pronunciación para el
  original y otro para la traducción. El texto se lee con SAPI de Windows a la
  velocidad elegida en la ventana de ajustes (**Speaking rate**: Slow / Normal /
  Fast / Very fast), y volver a pulsar el botón lo detiene, igual que cerrar la
  tarjeta.

- **Un orden de reserva que puedes fijar.** Cuando el servicio elegido falla, Glossy
  recorre ahora una lista con los demás servicios en lugar de rendirse. La ventana de
  ajustes muestra esa lista bajo **Ask another service when the chosen one fails**: el
  servicio elegido arriba se intenta siempre primero y no se puede mover, los demás
  pueden reordenarse con las flechas y el conjunto se puede apagar. Cuando responde
  otro servicio, la tarjeta dice cuál fue.

- **Una tarjeta compacta.** **Draw a compact card, without the extras** conserva la
  traducción, los símbolos fonéticos y los significados, y deja fuera el ejemplo, las
  formas, los sinónimos, la frase de contexto, la vista frase por frase y las
  conversiones de unidades, para un emergente pensado para leerse de un vistazo.

- **Reading, y cuánto de ella.** La ventana de ajustes tiene un panel **Reading** que
  decide qué puede mostrar la tarjeta de una palabra: **Show the sentence a word was
  selected from** (la búsqueda de contexto), **Pair the original and the translation
  sentence by sentence** (la vista alineada) y **Draw a compact card, without the
  extras**, además de la **Speaking rate** de los botones de pronunciación.

- **La selección se limpia antes de enviarse.** `text.rs` reduce lo que entregó
  Ctrl+C a las frases que escribió el autor: se eliminan los guiones suaves y los
  espacios de ancho cero, las secuencias de escape de la terminal, los saltos de
  línea que insertó el ancho de la ventana, las series de espacios duros y los
  caracteres de control sueltos, de modo que no se pide al proveedor que traduzca la
  maquetación de la página.

### Cambios

- **Exportar el archivo de ajustes ya no pregunta por las claves.** Sin ningún campo
  que acepte una clave de API, no hay nada que una exportación pueda llevarse: la
  casilla, su aviso y el diálogo de confirmación se han eliminado, y `export_settings`
  ya no tiene el argumento `include_credentials`. El archivo contiene las opciones y
  nada secreto.

- **Cuatro servicios de traducción y ningún campo de clave en ninguna parte.** El
  desplegable de la ventana de ajustes tiene ahora exactamente cuatro entradas —
  **Baidu Translate** y **Youdao Translate** (el servidor del proyecto, cada una
  diciendo qué upstream debe usar, así que no hay nada que rellenar), **Local model**
  (cualquier punto de conexión compatible con OpenAI en esta máquina) y **Google** (el
  punto de conexión público gratuito) — y el panel **Extensions · my own API**
  desapareció junto con las entradas de Zhipu, DeepL y OpenAI que servía. Una
  instalación nueva empieza en Baidu Translate. La forma de guardar no cambia en
  nada: `channel` + `cloudProvider` + `cloudVendor` / `provider` se escriben
  exactamente igual que antes, así que un archivo de ajustes de una compilación
  anterior sigue funcionando — un `provider` que esta compilación ya no ofrece se lee
  como la entrada integrada de Baidu y se reemplaza la próxima vez que se escriba el
  archivo, y un `cloudVendor` que no sea `youdao` se lee como Baidu. El README y el
  ROADMAP se actualizaron en los tres idiomas.

- **La línea del original en la tarjeta se limita a cuatro líneas.** Una selección
  larga empujaba la traducción hacia abajo y fuera de la vista; ahora el bloque de
  origen se desplaza dentro de sí mismo en cuanto pasaría de cuatro líneas, tanto en
  el emergente flotante como en la tarjeta de la ventana de ajustes. La tarjeta se lee
  por la traducción, así que esta conserva su sitio.

### Correcciones

- La confirmación que aparece en la parte inferior de la ventana de ajustes era
  invisible en modo oscuro cuando la ventana estaba sobre el fondo Mica: el aviso
  tomaba su fondo de `--fg` y su texto de `--bg`, y Mica reasigna `--bg` a un 6 % de
  blanco, así que el texto del modo oscuro acababa sobre una superficie oscura. Ahora
  el aviso tiene sus propios tokens `--g-toast-surface` / `--g-toast-text` /
  `--g-toast-outline`, y los cuadros de diálogo pasaron a una superficie sólida con la
  sombra de diálogo, de modo que ambos se leen igual en cualquiera de los dos temas y
  sobre cualquier fondo.
