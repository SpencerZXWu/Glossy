[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The appearance pass v1.1.0 left open: the popup was measured in a browser, and what
came back was contrast, type size, motion and two states that said the same thing
twice. Almost nothing here changes what Glossy does; the read-aloud buttons are the
one exception, and that is because their logic was wrong as well as their look.

### Changed

- **The muted text is readable now.** The phonetic symbols, the example, the
  footnotes, the block titles and the line that stands in while a word card is still
  being completed all drew from `--g-text-tertiary`, which measured about 3.3:1
  against the card. The token is `rgba(0, 0, 0, 0.558)` in light and
  `rgba(255, 255, 255, 0.5646)` in dark, which puts every one of them at 4.86:1 or
  better in either theme.

- **The opacity setting dims the plate, not the words.** The card used to fade as a
  whole, so a setting below about 70 % took the text down with it. The surface moved
  into `.card::before` — border, background and shadow, with `isolation: isolate` on
  the card to keep the layer under the text — and that layer is the only thing
  `--popup-opacity` touches. The text stays fully opaque at the 50 % minimum.

- **The smallest text and the buttons both grew.** The close, copy and pin buttons go
  from 24px to `calc(26px * var(--popup-font))` with a 15px icon, the language swap
  button is 24px, the footnotes from 10.5px to 11.5px and the block titles from 10px
  to 10.5px.

- **The read-aloud buttons are a pair of icons in the corner.** They were two wide
  pills under the translation; they are now 26px buttons built like the pin, copy and
  close buttons, side by side and right-aligned at the bottom corner of the card,
  each one labelled for its own side. A speaker at rest becomes a stop square while
  that side is being read, so which of the two is playing is never something the
  reader has to remember.

- **Nothing flashes while a word card is completed.** `refine()` used to draw a card
  with the extras stripped out and fill them in afterwards; it draws the one card with
  a pending marker instead, so the provider's content is never wiped and redrawn.

- **Motion and colour come from the tokens.** Every literal `120ms ease` became
  `var(--g-dur-fast) var(--g-ease-standard)`, the entrance animation translates the
  card without animating its opacity, `.card::before` fades in on its own keyframes,
  the focus ring is one `:focus-visible` rule on `--accent`, and the scrollbar thumb is
  `--g-stroke-strong`, lifting to `--muted` under the pointer.

### Fixed

- **Pressing a pronunciation button a second time did nothing.** `say` answers as
  soon as the voice has accepted the text — the reading itself runs on a SAPI thread
  of its own — so a second press on a button that was still lit was treated as a
  first press: the highlight lasted one frame and a reading could only be stopped by
  closing the card. `speech.rs` now keeps a speaking flag under a generation counter,
  the `speaking` command reports it, and the card follows it on a 250ms timer so the
  button goes back to rest when the voice falls silent.

- **A reading outlived the card that started it.** Closing the card, replacing it
  with a new selection, or switching to the other side all used to leave the old
  reading talking over the new one. Each of them stops the voice first: `popup::hide`
  on the way out, `dismiss()` before the window goes, and the card itself before
  arming the next button.

- **A reading that could not start said nothing.** A missing voice, a busy one or an
  outright refusal now says so on the button that asked — tinted, relabelled, and
  back to rest a couple of seconds later.

- **Reduced motion was not honoured.** The `prefers-reduced-motion` block sat above
  the rules it was meant to switch off, and at equal specificity the later declaration
  wins, so the loading shimmer and the plate fade kept playing. The block is last in
  `popup.css` now and covers `.card::before`, not just `.card`.

- **An error message repeated itself.** The card printed the localized headline and
  then the raw message, and the two were often the same sentence. The headline is
  always the localized one, and the provider's own text follows as a muted note only
  when it is different and not empty.

- **An empty translation looked like an answer.** The fallback message draws as
  `translation empty` — muted and italic — so it reads as a notice. The line that
  stands in while a lookup is running stays a plain muted line: a pulsing dot would
  claim more liveness than a single lookup deserves.

<a id="zh-cn"></a>

## 中文

v1.1.0 留下的外观工作：我们在浏览器里实测了弹窗，拿回来的问题是对比度、字号、动效，以及两处把同一句话说了两遍的状态。这一版几乎不改变 Glossy 的行为，唯一的例外是朗读按钮——它的逻辑和外观一样都是错的。

### 变更

- **弱化文字现在读得清了。** 音标、例句、脚注、区块标题，以及词语卡片还在补全时顶位的那一行，都取自 `--g-text-tertiary`，而它在卡片上的实测对比度约为 3.3:1。该 token 现在是浅色 `rgba(0, 0, 0, 0.558)`、深色 `rgba(255, 255, 255, 0.5646)`，两种主题下这些文字都在 4.86:1 以上。

- **不透明度设置只压底板，不压文字。** 以前整张卡片一起变淡，设置低于约 70% 时文字会跟着一起淡掉。现在底板移到 `.card::before`（边框、底色、阴影都在这一层，卡片上加 `isolation: isolate` 让该层留在文字之下），`--popup-opacity` 只作用于它；即使调到下限 50%，文字也保持完全不透明。

- **最小的字和按钮都变大了。** 关闭、复制、固定按钮从 24px 改为 `calc(26px * var(--popup-font))`、图标 15px，语言交换按钮 24px，脚注 10.5px → 11.5px，区块标题 10px → 10.5px。

- **朗读按钮变成了角落里的一对图标。** 它们原本是译文下方两颗很宽的胶囊按钮；现在是与固定、复制、关闭同一套做法的 26px 按钮，并排右对齐在卡片右下角，各自带有标明读哪一侧的提示文字。静止时是喇叭，朗读时换成停止方块，正在读哪一侧不需要靠记忆。

- **词语卡片补全时不再闪。** `refine()` 以前先画一张抽掉附加内容的卡片、再填回去；现在只画这一张卡并带上"补齐中"标记，所以服务商返回的内容不会被擦掉重画。

- **动效与颜色统一走 token。** 所有写死的 `120ms ease` 改为 `var(--g-dur-fast) var(--g-ease-standard)`，入场动画只做位移、不再对卡片做透明度动画，`.card::before` 用独立的 keyframes 淡入，焦点环合并为 `--accent` 上的一条 `:focus-visible` 规则，滚动条拇指改用 `--g-stroke-strong`，指针悬停时提亮为 `--muted`。

### 修复

- **再按一次朗读按钮没有任何反应。** `say` 只要语音接受了文本就会立刻返回——真正的朗读跑在 SAPI 自己的线程上——所以对着一个还亮着的按钮再按一次会被当成第一次按下：高亮只维持一帧，想停下朗读只能关掉卡片。现在 `speech.rs` 用世代计数器维护一个朗读标志，`speaking` 命令上报它，卡片每 250ms 跟随它，声音一停按钮就回到静止态。

- **朗读活得比唤起它的卡片还久。** 关闭卡片、用新的选区替换卡片、切到另一侧，过去都会让旧的朗读继续念、盖过新的那张卡。现在这三者都会先停掉语音：退出时由 `popup::hide` 停，窗口消失前由 `dismiss()` 停，卡片自己在武装下一个按钮前也会停。

- **朗读失败时什么都不说。** 系统里没有语音、语音被占用或直接拒绝，现在都会在发起它的按钮上说出来——染色、换标签，几秒后回到静止态。

- **"减少动效"没有被遵守。** `prefers-reduced-motion` 那个块写在了它要关闭的规则之前，同优先级下后写的声明获胜，于是加载微光与底板淡入照旧播放。现在该块位于 `popup.css` 最后，并且同时覆盖 `.card::before`，不再只是 `.card`。

- **错误信息重复了自己。** 卡片先打印本地化标题、再打印原始信息，而两者常常就是同一句话。现在标题始终是本地化的那一句，服务商自己的文本只在确有不同且非空时作为弱化附注跟在后面。

- **空译文看起来像答案。** 兜底提示改为 `translation empty` —— 弱化且斜体 —— 读起来是一条提示而不是一条译文。查询进行中顶位的那一行保持普通弱化文本：脉冲圆点会显得比一次查询更"活跃"。

<a id="es"></a>

## Español

El trabajo de apariencia que v1.1.0 dejó pendiente: medimos la ventana emergente en un navegador y lo que salió fue contraste, tamaño de letra, movimiento, y dos estados que decían lo mismo dos veces. Casi nada de esto cambia lo que hace Glossy; los botones de lectura son la única excepción, y lo son porque su lógica estaba tan mal como su aspecto.

### Cambios

- **El texto atenuado ya se lee.** Los símbolos fonéticos, el ejemplo, las notas al pie, los títulos de bloque y la línea que ocupa el lugar mientras se completa una tarjeta de palabra salían de `--g-text-tertiary`, que medía unas 3,3:1 sobre la tarjeta. El token es `rgba(0, 0, 0, 0.558)` en claro y `rgba(255, 255, 255, 0.5646)` en oscuro, lo que deja a todos ellos en 4,86:1 o mejor en cualquier tema.

- **La opacidad atenúa la placa, no las palabras.** Antes la tarjeta se desvanecía entera, así que un ajuste por debajo del 70 % se llevaba el texto con ella. La superficie pasó a `.card::before` —borde, fondo y sombra, con `isolation: isolate` en la tarjeta para mantener esa capa bajo el texto—, y es lo único que toca `--popup-opacity`. El texto queda totalmente opaco en el mínimo del 50 %.

- **El texto más pequeño y los botones crecieron.** Los botones de cerrar, copiar y fijar pasan de 24px a `calc(26px * var(--popup-font))` con un icono de 15px, el botón de cambiar idioma mide 24px, las notas al pie pasan de 10,5px a 11,5px y los títulos de bloque de 10px a 10,5px.

- **Los botones de lectura son un par de iconos en el rincón.** Antes eran dos pastillas anchas bajo la traducción; ahora son botones de 26px hechos como los de fijar, copiar y cerrar, uno junto al otro y alineados a la derecha en la esquina inferior de la tarjeta, cada uno etiquetado para su propio lado. Un altavoz en reposo se convierte en un cuadrado de detener mientras se lee ese lado, así que cuál de los dos está sonando nunca es algo que el lector tenga que recordar.

- **Nada parpadea mientras se completa una tarjeta de palabra.** `refine()` dibujaba una tarjeta sin los extras y los rellenaba después; ahora dibuja una sola tarjeta con la marca de pendiente, así que el contenido del proveedor nunca se borra ni se vuelve a dibujar.

- **El movimiento y el color salen de los tokens.** Cada `120ms ease` literal pasó a `var(--g-dur-fast) var(--g-ease-standard)`, la animación de entrada desplaza la tarjeta sin animar su opacidad, `.card::before` se desvanece con sus propios keyframes, el anillo de foco es una sola regla `:focus-visible` sobre `--accent`, y el pulgar de la barra de desplazamiento es `--g-stroke-strong`, que sube a `--muted` bajo el puntero.

### Correcciones

- **Volver a pulsar un botón de lectura no hacía nada.** `say` responde en cuanto la voz acepta el texto —la lectura en sí corre en un hilo propio de SAPI—, así que una segunda pulsación sobre un botón que seguía encendido se trataba como la primera: el resaltado duraba un fotograma y sólo se podía detener una lectura cerrando la tarjeta. Ahora `speech.rs` mantiene una bandera de lectura bajo un contador de generación, el comando `speaking` la informa, y la tarjeta la sigue con un temporizador de 250ms para que el botón vuelva al reposo cuando la voz se calla.

- **Una lectura sobrevivía a la tarjeta que la inició.** Cerrar la tarjeta, sustituirla por una selección nueva o cambiar al otro lado dejaban la lectura antigua hablando por encima de la nueva. Ahora las tres detienen la voz primero: `popup::hide` a la salida, `dismiss()` antes de que la ventana desaparezca, y la propia tarjeta antes de armar el botón siguiente.

- **Una lectura que no podía empezar no decía nada.** Una voz ausente, ocupada o que rechaza el texto ahora lo dice en el botón que la pidió: teñido, reetiquetado y de vuelta al reposo un par de segundos después.

- **No se respetaba el movimiento reducido.** El bloque `prefers-reduced-motion` estaba antes de las reglas que debía desactivar y, a igual especificidad, gana la declaración posterior, así que el brillo de carga y el fundido de la placa seguían reproduciéndose. Ahora el bloque está al final de `popup.css` y cubre también `.card::before`, no sólo `.card`.

- **El mensaje de error se repetía.** La tarjeta imprimía el titular localizado y después el mensaje original, y muchas veces eran la misma frase. El titular es siempre el localizado, y el texto del proveedor lo sigue como nota atenuada sólo cuando es distinto y no está vacío.

- **Una traducción vacía parecía una respuesta.** El mensaje de reserva se dibuja como `translation empty` —atenuado y en cursiva—, así que se lee como un aviso. La línea que ocupa el lugar mientras se busca sigue siendo texto atenuado normal: un punto pulsante reclamaría más vida de la que merece una sola búsqueda.
