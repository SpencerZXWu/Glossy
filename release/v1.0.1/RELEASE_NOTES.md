[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Added

- **Only translate these source languages**, a new list in the Trigger panel of
  the settings window. Add a language and Glossy stops translating selections
  written in the others, which is how a reader who works in one language keeps a
  second one out of the way. The language of a selection is guessed from its
  script and its most frequent short words, and a selection that cannot be told
  apart (a name, a number, a line of code) is translated anyway, so a wrong guess
  never swallows text the user wanted. The list is stored as `sourceLangs`, and an
  empty list means every language, which is what settings files written before
  this release get.

### Fixed

- A double click that selects nothing no longer opens a popup. Clicks on the
  desktop, the taskbar, the start menu and the other surfaces of the shell are
  ignored, because the Ctrl+C Glossy sends afterwards still reaches the program in
  front, and a program that answers it by copying a stale clipboard entry used to
  bring the popup up out of nowhere.
- Selections that carry no language are skipped: a run of digits or punctuation,
  and - when the click happens over a program that copies it - the path of a file.
- A copy of a file in the file explorer no longer counts as a selection. Explorer
  publishes the files it copies as `CF_HDROP` while offering the path as text as
  well; the text is what used to be translated.
- The clipboard is really put back. Some programs fill it from a worker thread,
  so their write landed just after the restore and undid it; the restore now
  watches the clipboard for a moment longer and repairs it.
- Glossy no longer reads its own clipboard writes as a copy. Putting the previous
  content back happens after a selection has been read, and the write that does it
  moves the clipboard along just like a real copy; a second trigger arriving in
  that window used to see the restored text as a fresh selection and translate it.
  Every write Glossy makes is now remembered by its sequence number and ignored
  while waiting for an answer.

<a id="zh-cn"></a>

## 中文

### 新增

- **仅翻译这些源语言**，设置窗口「触发」面板里的新列表。加入一种语言之后，Glossy 就不再翻译用其他语言写成的选区——习惯用一种语言阅读的人，可以用它把另一种语言放到一边。选区的语言由文字系统和它最常出现的短词推断得出；无法判断的文本（人名、数字、一行代码）照常翻译，所以猜错也不会吞掉你想要的文字。该列表保存为 `sourceLangs`，留空表示不限语言——本次版本之前写下的设置文件就是这个状态。

### 修复

- 没有选中任何内容的双击不再弹出窗口。桌面、任务栏、开始菜单以及其他 shell 表面上的点击都被忽略：因为 Glossy 随后发出的 `Ctrl+C` 仍会送到最前面的那个程序，而用它答复、把剪贴板里的旧内容复制一遍的程序，过去会莫名其妙地把弹窗叫出来。
- 不含语言的选区会被跳过：一串数字或标点，以及在会复制它的程序上点击时得到的文件路径。
- 在资源管理器里复制文件不再算作选中内容。资源管理器把复制的文件以 `CF_HDROP` 发布，同时也会把路径作为文本提供；过去被翻译的正是那段文本。
- 剪贴板真的会被还原。有些程序从工作线程填充剪贴板，它们的写入紧跟在还原之后落地、把还原结果覆盖掉；现在还原会再多守望剪贴板一会儿并修好它。
- Glossy 不再把自己写入剪贴板的内容当成一次复制。还原发生在读取选区之后，而这一次写入会让剪贴板的序列号像真正的复制一样前进；在这个窗口期里到来的第二次触发，过去会把还原后的文字当作新的选区拿去翻译。现在 Glossy 会记下自己每一次写入的序列号，在等待答复期间跳过它们。

<a id="es"></a>

## Español

### Añadido

- **Traducir solo estos idiomas de origen**, una lista nueva en el panel
  Disparador de la ventana de ajustes. Al añadir un idioma, Glossy deja de
  traducir las selecciones escritas en los demás, que es como quien trabaja en un
  idioma mantiene el otro apartado. El idioma de una selección se deduce de su
  escritura y de sus palabras cortas más frecuentes, y una selección que no se
  puede distinguir (un nombre, un número, una línea de código) se traduce igual,
  así que una deducción equivocada nunca se traga el texto que querías. La lista
  se guarda como `sourceLangs`, y una lista vacía significa todos los idiomas, que
  es lo que reciben los archivos de ajustes escritos antes de esta versión.

### Corregido

- Un doble clic que no selecciona nada ya no abre ninguna ventana. Se ignoran los
  clics en el escritorio, la barra de tareas, el menú Inicio y las demás
  superficies del shell, porque el Ctrl+C que Glossy envía después sigue llegando
  al programa que está delante, y un programa que lo responde copiando una entrada
  antigua del portapapeles hacía aparecer la tarjeta de la nada.
- Se omiten las selecciones sin idioma: una ristra de dígitos o de puntuación y,
  cuando el clic cae sobre un programa que la copia, la ruta de un archivo.
- Copiar un archivo en el Explorador ya no cuenta como selección. El Explorador
  publica los archivos que copia como `CF_HDROP` y además ofrece la ruta como
  texto; ese texto era lo que se traducía.
- El portapapeles se devuelve de verdad a su sitio. Algunos programas lo llenan
  desde un hilo de trabajo, así que su escritura llegaba justo después de la
  restauración y la deshacía; ahora la restauración vigila el portapapeles un poco
  más y lo repara.
- Glossy ya no lee sus propias escrituras en el portapapeles como una copia.
  Devolver el contenido anterior ocurre después de leer una selección, y esa
  escritura mueve el portapapeles igual que una copia real; un segundo disparo que
  llegara en ese hueco veía el texto restaurado como una selección nueva y lo
  traducía. Ahora Glossy recuerda el número de secuencia de cada escritura suya y
  las omite mientras espera una respuesta.
