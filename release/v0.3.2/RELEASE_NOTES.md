[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Fixed

- The installer now ships `WebView2Loader.dll`, so a freshly installed Glossy
  starts instead of failing at launch with the Windows error "WebView2Loader.dll
  was not found" (`由于找不到 WebView2Loader.dll，无法继续执行代码`). The GNU
  toolchain Glossy is built with links that DLL dynamically, and `tauri-build`
  copies it next to `glossy.exe` for local runs only, so the previous installers
  carried nothing but the executable. `build.rs` now stages the DLL from the
  `webview2-com-sys` build output into `src-tauri/resources/` and
  `bundle.resources` ships it, which installs it flat into the app folder beside
  the executable. `scripts/release.ps1` refuses to stage a release when the
  staged DLL is missing or the generated installer does not include it, so this
  cannot ship silently again.

### Changed

- The README is written in the same three languages as the release notes, in the
  same single-file layout: an English block, then 中文, then Español, each behind
  an anchor the link line at the top jumps to. The English block stays first, so
  existing links to `README.md` and its headings keep working.
- Release notes are written in three languages — English, Chinese and Spanish — in a
  single `RELEASE_NOTES.md`, with a link line at the top that jumps to an anchor placed
  directly above each language's block. `scripts/release.ps1` fills the English block
  from this file and leaves the other two as placeholders it warns about until they are
  translated, so a release cannot go out half-translated by accident. `CHANGELOG.md`
  itself stays English, as Keep a Changelog expects.

<a id="zh-cn"></a>

## 中文

### 修复

- 安装包现在会一并装上 `WebView2Loader.dll`，所以新装好的 Glossy 可以正常启动，不会再在
  启动时弹出「由于找不到 WebView2Loader.dll，无法继续执行代码」的系统错误。Glossy 所用的
  GNU 工具链是动态链接这个 DLL 的，而 `tauri-build` 只在本地运行时把它复制到 `glossy.exe`
  旁边，所以此前的安装包里只有可执行文件本身。现在 `build.rs` 会把 `webview2-com-sys`
  构建产物中的该 DLL 暂存到 `src-tauri/resources/`，再由 `bundle.resources` 打进安装包，
  安装时直接落在程序目录里可执行文件的旁边。`scripts/release.ps1` 在暂存的 DLL 缺失、或者
  生成的安装脚本里没有包含它时会拒绝发版，所以这个问题不会再悄悄地溜出去。

### 变更

- README 和发行说明一样写成了三种语言，用同样的单文件排版：先是英文，然后是中文，最后是
  西班牙文，每一段都放在顶部链接行所跳转的锚点之后。英文仍排在最前，因此已有的指向
  `README.md` 及其各级标题的链接照旧可用。
- 发行说明改为三种语言——英文、中文、西班牙文——合并在同一个 `RELEASE_NOTES.md` 里，顶部
  的链接行可跳转到每种语言块正上方的锚点。`scripts/release.ps1` 会从 `CHANGELOG.md` 生成
  英文块，另外两块先留占位符并在发布前提醒，以免半个译文被误发出去。`CHANGELOG.md` 本身按
  Keep a Changelog 的惯例保持英文。

<a id="es"></a>

## Español

### Corregido

- El instalador ahora incluye `WebView2Loader.dll`, así que un Glossy recién instalado
  arranca en lugar de fallar al iniciarse con el error de Windows «no se encuentra
  WebView2Loader.dll» (`由于找不到 WebView2Loader.dll，无法继续执行代码`). La cadena de
  herramientas GNU con la que se compila Glossy enlaza ese DLL de forma dinámica, y
  `tauri-build` solo lo copia junto a `glossy.exe` para las ejecuciones locales, así que
  los instaladores anteriores no llevaban más que el ejecutable. Ahora `build.rs` prepara
  el DLL desde la salida de compilación de `webview2-com-sys` en `src-tauri/resources/` y
  `bundle.resources` lo empaqueta, de modo que se instala directamente en la carpeta de la
  aplicación, junto al ejecutable. `scripts/release.ps1` se niega a preparar una versión
  cuando falta el DLL o el instalador generado no lo incluye, así que esto no puede volver
  a pasar desapercibido.

### Cambios

- El README está escrito en los mismos tres idiomas que las notas de la versión, con el
  mismo formato de archivo único: un bloque en inglés, luego 中文 y después Español, cada
  uno detrás de un ancla a la que salta la línea de enlaces de arriba. El bloque en inglés
  sigue siendo el primero, así que los enlaces existentes a `README.md` y a sus encabezados
  siguen funcionando.
- Las notas de la versión se escriben en tres idiomas —inglés, chino y español— en un
  único `RELEASE_NOTES.md`, con una línea de enlaces arriba que salta al ancla situada justo
  encima del bloque de cada idioma. `scripts/release.ps1` rellena el bloque en inglés desde
  este archivo y deja los otros dos como marcadores de posición hasta que se traduzcan,
  avisando de ello, de modo que una versión no puede publicarse a medio traducir por
  accidente. `CHANGELOG.md` se mantiene en inglés, como espera Keep a Changelog.
