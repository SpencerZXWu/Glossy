[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The release where the shared server stops relying on everyone's good manners, and where
the rectangle that reads the screen can be drawn more than once. The OCR service behind
that rectangle is metered by the month, so one installation can no longer spend the whole
free allowance on its own, and the Screenshot translation page says how much of the month
is left before a shortcut is pressed rather than after.

### Added

- **A monthly ceiling on screen readings, per device.** The OCR service behind the shared
  server hands out a small free allowance every month, so `/v1/ocr` now counts how many
  readings one install has used since the 1st and refuses the 101st with
  `ocr_month_quota_exceeded` (`OCR_PER_CLIENT_MONTH`, `0` turns it off). It is a second,
  monthly ledger next to the daily character one, kept in its own bucket per install, and
  a reading that is refused or that the upstream fails is refunded - `GET /v1/quota` and
  the `/v1/ocr` answer both report `ocrMonth` and `remainingOcrMonth`. Ordinary text
  translation is untouched and still runs on the daily characters. The popup explains the
  refusal in the reader's own words instead of showing the server's Chinese.
- **The Screenshot translation page says what is left of the month.** Below the description
  it now shows how many screen readings this device still has left this month, turning red
  and naming the reset day once they are gone, and staying out of the way on a server that
  sets no monthly limit.

### Fixed

- **The screen reading rectangle can be drawn again after the first one.** The `ocr`
  window was missing from the capability file, and since app commands are not gated by it
  but `core:event:listen` is, the page's `glossy://ocr` listener was refused: the flag that
  says "this session already sent a rectangle" was never cleared, so every overlay after
  the first ignored the pointer and the whole desktop had to be escaped with Task Manager.
  The window is now allowed to listen, the listener reports a refusal instead of failing
  silently, and a page that comes back to the front resets itself as a second line of
  defence. A regression test now checks that every page which listens for an event is
  covered by a capability.
- **Drawing the rectangle no longer costs a translation.** The overlay was not part of what
  counts as "the Glossy window", so the drag that marks a region was classified as a
  selection drag and spent an allowance the user never asked for.

<a id="zh-cn"></a>

## 中文

这一版里，共用的服务器不再指望所有人都守规矩，而读取屏幕的那个矩形也能画第二次了。
矩形背后的 OCR 服务是按月计量的，所以一个安装再也不能独自用光整份额度；而"截图翻译"
页会在你按下快捷键之前就告诉你这个月还剩多少，而不是按下之后。

### 新增

- **按设备计算的每月截图次数上限。** 共用服务器背后的 OCR 服务每月只给一份不大的免费
  额度，所以 `/v1/ocr` 现在会统计一个安装在 1 号之后读了多少次，第 101 次会被拒绝并返
  回 `ocr_month_quota_exceeded`（由 `OCR_PER_CLIENT_MONTH` 控制，设为 `0` 即关闭）。它是
  每日字符额度之外的第二本账，按安装各自记在一个桶里；被拒绝的、或者上游失败的读取都
  会退还次数，`GET /v1/quota` 和 `/v1/ocr` 的返回都会带上 `ocrMonth` 与
  `remainingOcrMonth`。常规文本翻译不受影响，照旧走每天的字符额度。弹窗会用人话解释这
  次拒绝，而不是把服务器的中文原样丢出来。
- **"截图翻译"页会显示本月还剩多少。** 现在在说明文字下面会写上这台设备本月还能读多少
  次屏，用完时变成红色并写明恢复日期；服务器没有设这个上限时，这一行不会出现。

### 修复

- **第一次之后还能再画框读屏了。** capability 文件里漏了 `ocr` 窗口，而应用自己的命令
  不受它管、`core:event:listen` 却受它管，于是页面那句 `glossy://ocr` 监听被拒，"本次
  会话已经发过矩形"的标志再也没被清掉——第一次之后的每一次覆盖层都无视鼠标，整个桌面
  只能用任务管理器逃出来。现在该窗口被允许监听，监听失败时也会报出来而不是默默失败，
  页面重新回到前台时还会自我重置，作为第二道保险。回归测试现在会检查每个监听事件的页
  面都有 capability 覆盖。
- **画框不再白白花掉一次翻译。** 覆盖层原来不算"Glossy 的窗口"，于是标记区域的拖动被
  判成划词拖动，扣掉了一次用户根本没要的额度。

<a id="es"></a>

## Español

La versión en la que el servidor compartido deja de fiarse de la buena educación de todo el
mundo, y en la que el rectángulo que lee la pantalla se puede dibujar más de una vez. El
servicio de OCR que hay detrás de ese rectángulo se mide por meses, así que una sola
instalación ya no puede gastar la cuota gratuita entera por su cuenta, y la página
Screenshot translation dice cuánto queda del mes antes de pulsar un atajo, no después.

### Añadido

- **Un techo mensual de lecturas de pantalla, por dispositivo.** El servicio de OCR que hay
  detrás del servidor compartido reparte cada mes una cuota gratuita pequeña, así que
  `/v1/ocr` ahora cuenta cuántas lecturas ha hecho una instalación desde el día 1 y rechaza
  la 101 con `ocr_month_quota_exceeded` (`OCR_PER_CLIENT_MONTH`; `0` lo desactiva). Es una
  segunda cuenta, mensual, junto a la diaria de caracteres, guardada en su propio espacio
  por instalación, y una lectura rechazada o que falla en el proveedor se devuelve: tanto
  `GET /v1/quota` como la respuesta de `/v1/ocr` informan de `ocrMonth` y
  `remainingOcrMonth`. La traducción de texto normal no se toca y sigue dependiendo de los
  caracteres diarios. El popup explica el rechazo con palabras del lector en vez de mostrar
  el chino del servidor.
- **La página Screenshot translation dice cuánto queda del mes.** Debajo de la descripción
  ahora indica cuántas lecturas de pantalla le quedan a este dispositivo este mes, se pone
  en rojo y nombra el día de reinicio cuando se acaban, y se aparta en un servidor que no
  ponga ese límite.

### Corregido

- **El rectángulo de lectura de pantalla se puede volver a dibujar tras el primero.**
  Faltaba la ventana `ocr` en el archivo de capacidades y, como los comandos de la
  aplicación no dependen de él pero `core:event:listen` sí, se rechazaba el listener
  `glossy://ocr` de la página: la marca que dice «esta sesión ya envió un rectángulo» nunca
  se borraba, así que cada capa posterior a la primera ignoraba el puntero y había que
  escapar del escritorio entero con el Administrador de tareas. Ahora esa ventana puede
  escuchar, el listener avisa del rechazo en lugar de fallar en silencio, y una página que
  vuelve al frente se reinicia como segunda defensa. Una prueba de regresión comprueba que
  toda página que escucha un evento está cubierta por una capacidad.
- **Dibujar el rectángulo ya no cuesta una traducción.** La capa no formaba parte de lo que
  cuenta como «la ventana de Glossy», así que el arrastre que marca una región se clasificaba
  como arrastre de selección y gastaba una cuota que el usuario no había pedido.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
