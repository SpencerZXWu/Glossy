[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The settings file and the names of the commands stop moving. Nothing changes on screen:
this release is the promise that a later version keeps reading what an earlier one wrote,
and a test that fails when that promise is broken by accident.

### Added

- **The settings file says which format it is in.**
  `%APPDATA%\com.glossy.translator\settings.json` now opens with `formatVersion`. Adding
  a setting does not move it; renaming one, removing one or changing what one means does,
  and a file written before the change is carried forward by `Settings::migrate` before it
  is merged key by key, so nothing is lost on the way up.
- **A file this build cannot read is kept instead of overwritten.** A settings file that is
  not a JSON object, or one that declares a `formatVersion` newer than this build
  understands, is copied to `settings.backup-<unix seconds>.json` beside it, one line is
  printed, and the app starts from the defaults. Losing the settings is recoverable;
  overwriting them is not.

### Changed

- **The shape of the file and the names of the commands are frozen, and the freeze is
  checked.** `contract/contract.json` is the published contract: the format version, every
  key the settings file holds and every IPC command the app answers to. The Rust suite
  compares it with the shape `Settings` actually serializes, and the frontend suite compares
  it with the command list in `lib.rs` and with every `invoke("…")` in `src/js/`, so
  adding, renaming or removing one of them fails the build until the contract is edited in
  the same commit.

### Unchanged

- Everything v1.2.2 to v1.2.4 changed is exactly as it was: a selection shows an icon
  instead of translating on its own, the card's close button is a settings button, the
  language bars offer only the languages the chosen engine translates, the engine can be
  switched while an answer is on its way, exchange rates come through the server and the
  installer is built from the tag that names it. Those entries are in
  [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) under their
  own versions.

<a id="zh-cn"></a>

## 中文

设置文件与命令名从此不再变动。界面上没有任何变化：这个版本给出的是一个承诺——以后的版本始终读得懂以前的版本写下的文件——外加一个在该承诺被无意打破时会失败的测试。

### 新增

- **设置文件会说明自己用的是哪一版格式。** `%APPDATA%\com.glossy.translator\settings.json` 现在以 `formatVersion` 开头。新增一项设置不会推动它；改名、删除或改变某一项的含义才会，而在此之前写出的文件会先由 `Settings::migrate` 往前带一步，再逐键合并，因此向上迁移的过程中不丢任何东西。
- **本构建读不了的文件会被保留，而不是被覆盖。** 不是 JSON 对象的设置文件，或声明的 `formatVersion` 比本构建更新的文件，会被复制为旁边的 `settings.backup-<Unix 秒数>.json`，打印一行提示，然后从默认值开始。设置丢了还能重来，被覆盖就没有了。

### 变更

- **文件形状与命令名被冻结，而且这层冻结是有检查的。** `contract/contract.json` 就是公布出去的契约：格式版本、设置文件里的每一个键，以及应用能应答的每一条命令。Rust 测试把它与 `Settings` 实际序列化出的形状比对，前端测试把它与 `lib.rs` 中的命令表和 `src/js/` 里每一次 `invoke("…")` 比对，因此新增、改名或删除其中之一都必须连同契约一起改，否则构建直接失败。

### 未变

- v1.2.2 到 v1.2.4 的所有改动原样保留：选区只显示图标而不自行翻译、卡片关闭按钮改成了设置按钮、语言栏只提供所选引擎能翻的语言、翻译途中也能切换引擎、汇率经由服务器、安装包由与版本号相符的标签构建。这些条目在 [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) 里各自对应的版本下。

<a id="es"></a>

## Español

El archivo de ajustes y los nombres de los comandos dejan de moverse. En pantalla no cambia
nada: esta versión es la promesa de que una versión posterior seguirá leyendo lo que
escribió una anterior, y una prueba que falla cuando esa promesa se rompe por accidente.

### Añadido

- **El archivo de ajustes dice en qué formato está escrito.**
  `%APPDATA%\com.glossy.translator\settings.json` ahora empieza por `formatVersion`.
  Añadir un ajuste no lo mueve; renombrar uno, eliminar uno o cambiar lo que significa uno
  sí lo hace, y un archivo escrito antes del cambio pasa por `Settings::migrate` antes de
  fusionarse clave por clave, de modo que no se pierde nada al subir.
- **Un archivo que esta compilación no puede leer se conserva en lugar de sobrescribirse.**
  Un archivo de ajustes que no sea un objeto JSON, o que declare un `formatVersion` más nuevo
  de lo que entiende esta compilación, se copia a `settings.backup-<segundos unix>.json` a su
  lado, se imprime una línea y la aplicación arranca con los valores por defecto. Perder los
  ajustes tiene arreglo; sobrescribirlos, no.

### Cambiado

- **La forma del archivo y los nombres de los comandos quedan congelados, y la congelación se
  comprueba.** `contract/contract.json` es el contrato publicado: la versión de formato, todas
  las claves del archivo de ajustes y todos los comandos IPC a los que responde la aplicación.
  La suite de Rust lo compara con la forma que `Settings` serializa realmente, y la del frontend
  con la lista de comandos de `lib.rs` y con cada `invoke("…")` de `src/js/`, así que añadir,
  renombrar o eliminar uno de ellos hace fallar la compilación hasta que el contrato se edite
  en la misma confirmación.

### Sin cambios

- Todo lo que cambiaron la v1.2.2 a la v1.2.4 sigue igual: una selección muestra un icono en
  lugar de traducir por su cuenta, el botón de cerrar de la tarjeta es un botón de ajustes, las
  barras de idiomas solo ofrecen los idiomas que traduce el motor elegido, el motor puede
  cambiarse mientras una respuesta está en camino, los tipos de cambio pasan por el servidor y
  el instalador se compila desde la etiqueta que lo nombra. Esas entradas están en
  [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) bajo su propia
  versión.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
