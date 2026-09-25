[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The release where the numbers are measurements. Nothing changes on screen; what changes is
that the wait between letting go of a selection and the card appearing is timed by the
application itself, the cost of doing nothing is written down with the machine it was
measured on, and the three things that could be held for a day of use are counted instead
of assumed.

Measured on an Intel Core i9-14900HX, 16 GB of RAM, Windows 11 build 26200, 2560x1600 at
150 %, release build: 0.00 % of CPU over 20 s and 39.4 MB of working set while idle, and
42 ms at p50 between the mouse-up and the painted card, against a budget of 150 ms.

### Added

- **The selection-to-popup path is timed, and the timing is repeatable.** `src-tauri/src/timing.rs`
  stamps the mouse-up that ends a drag, the popup reports its first frame back through the
  new `popup_painted` command, and one line per sample goes to stderr and to
  `GLOSSY_TIMING_LOG`. Both sides stay silent unless `GLOSSY_TIMING` is set, and a mark
  older than five seconds is dropped rather than paired with the wrong paint.
  `scripts/latency.ps1` is the procedure: it samples the idle cost, injects twelve
  selections, discards a warm-up drag and prints the samples with p50, p95 and the maximum.
- **Counters for the three things a long run could hold.** `src-tauri/src/vitals.rs` keeps
  the balance of the mouse hook, the clipboard and the speech voice, and a test reads it:
  the hook now gives its handle back instead of leaving it to process exit, every clipboard
  open goes through one `close()`, and a voice is counted where it is created and released.
  An imbalance seen on two selections in a row is said once on stderr.

### Changed

- **The clipboard is given back after the card is on its way, not before it appears.** The
  restore waits up to 80 ms for the application that was copied from to stop writing, and
  that wait used to happen before the popup was revealed. It now happens afterwards, from
  the same owner, so the paths that show nothing still put the clipboard back.
- **A copy is no longer given a fixed 20 ms to appear.** The first look at the clipboard
  happens immediately — most applications are already done by then — and the 20 ms is a
  ceiling rather than a wait, retried every 4 ms.

<a id="zh-cn"></a>

## 中文

这一版里，数字是量出来的。屏幕上没有任何变化；变化的是：从松开划选到卡片出现的那段等待，现在由应用自己计时；什么都不做时的开销，连同测量所用的机器一起写下来；一天里可能被一直占着的三处，改成数出来的，而不是想当然的。

在 Intel Core i9-14900HX、16 GB 内存、Windows 11 build 26200、2560x1600 缩放 150 % 上测得，release 构建：空闲时 20 秒内 0.00 % CPU、工作集 39.4 MB；从松开鼠标到卡片画出的中位数是 42 ms，预算是 150 ms。

### 新增

- **选区到弹窗的路径有计时，而且可复现。** `src-tauri/src/timing.rs` 在结束划选的那次鼠标抬起处打点，弹窗通过新增的 `popup_painted` 命令回报第一帧，每个样本打印一行到 stderr 和 `GLOSSY_TIMING_LOG`。除非设置了 `GLOSSY_TIMING`，两边都不出声；超过五秒的标记会被丢弃，而不会被配到另一次绘制上。`scripts/latency.ps1` 就是那套流程：先采一次空闲开销，再注入十二次划选，丢掉一次热身拖拽，最后连同样本打印 p50、p95 和最大值。
- **给一天里可能被一直占着的三处配了计数。** `src-tauri/src/vitals.rs` 记录鼠标钩子、剪贴板和朗读引擎的收支，测试直接读这些计数：钩子现在会归还句柄，而不是留给进程退出；每次打开剪贴板都走同一个 `close()`；朗读引擎在创建和释放处各记一笔。连续两次取词都看到不平衡，才在 stderr 上说一次。

### 变更

- **剪贴板在卡片上路之后才还回去，而不是在它出现之前。** 还原最多要等 80 ms，等被复制的程序停止写入，这段等待原本发生在弹窗显示之前。现在它发生在之后，仍由同一个持有者负责，所以那些什么都不显示的路径也照样会把剪贴板放回去。
- **复制不再固定等 20 ms。** 第一次查看剪贴板立刻进行——多数程序那时已经写完了——20 ms 是上限而不是等待，每 4 ms 重试一次。

<a id="es"></a>

## Español

La versión en la que los números son mediciones. En pantalla no cambia nada; lo que cambia es que la espera entre soltar una selección y ver la tarjeta la cronometra la propia aplicación, el coste de no hacer nada queda escrito junto con la máquina en la que se midió, y las tres cosas que podrían quedar retenidas durante un día se cuentan en vez de darse por supuestas.

Medido en un Intel Core i9-14900HX, 16 GB de RAM, Windows 11 build 26200, 2560x1600 al 150 %, compilación release: 0,00 % de CPU en 20 s y 39,4 MB de conjunto de trabajo en reposo, y 42 ms de mediana entre soltar el ratón y la tarjeta pintada, frente a un presupuesto de 150 ms.

### Añadido

- **El camino de la selección al globo está cronometrado, y la medición se repite.** `src-tauri/src/timing.rs` marca el soltar del ratón que cierra el arrastre, el globo informa de su primer fotograma mediante el nuevo comando `popup_painted`, y cada muestra imprime una línea en stderr y en `GLOSSY_TIMING_LOG`. Ninguno de los dos lados hace nada salvo que se defina `GLOSSY_TIMING`, y una marca de más de cinco segundos se descarta en lugar de emparejarse con el pintado equivocado. `scripts/latency.ps1` es el procedimiento: toma una muestra del coste en reposo, inyecta doce selecciones, descarta un arrastre de calentamiento e imprime las muestras con p50, p95 y el máximo.
- **Contadores para las tres cosas que una ejecución larga podría retener.** `src-tauri/src/vitals.rs` lleva el equilibrio del enganche del ratón, el portapapeles y la voz, y una prueba lo lee: el enganche ahora devuelve su manejador en lugar de dejarlo al final del proceso, cada apertura del portapapeles pasa por un único `close()`, y la voz se cuenta donde se crea y se libera. Un desequilibrio visto en dos selecciones seguidas se avisa una vez por stderr.

### Cambiado

- **El portapapeles se devuelve cuando la tarjeta ya va de camino, no antes de que aparezca.** La restauración espera hasta 80 ms a que la aplicación de la que se copió deje de escribir, y esa espera ocurría antes de mostrar el globo. Ahora ocurre después, desde el mismo dueño, así que los caminos que no muestran nada también devuelven el portapapeles.
- **Una copia ya no dispone de 20 ms fijos para aparecer.** La primera consulta del portapapeles es inmediata —la mayoría de las aplicaciones ya han terminado— y los 20 ms son un techo y no una espera, con reintentos cada 4 ms.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
