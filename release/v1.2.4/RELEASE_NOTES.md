[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

The same application as v1.2.3, built from committed source this time. Nothing in Glossy
changed between the two releases: the v1.2.3 installers were built from a working copy that
was never committed, so its tag pointed at a commit that declared 1.1.0. This release
commits that work and builds the installers from the tag that names them.

### Changed

- **A tag now builds the release it names.** Pushing a `vX.Y.Z` tag builds the installer, the
  MSI and the portable package on a Windows runner (`.github/workflows/release.yml`) through
  the same `scripts/release.ps1` a local release uses, and the run stops before the build
  when the tag does not match the version its commit declares. A release can no longer go out
  whose name and files disagree with the source they came from. The rest of the per-version
  process is unchanged: the notes below each release are still the committed
  `release/vX.Y.Z/RELEASE_NOTES.md`, and a build signed on a workstation is still what ships
  once there is a certificate.
- **The changelog has its 1.2.1 section back.** The 1.2.2 entry had swallowed the language
  bar entry, and the `1.2.1` heading with it, so the exchange-rate change and the language
  bar change were listed under one version. They are under 1.2.1 and 1.2.2 again.

### Unchanged

- Everything v1.2.2 and v1.2.3 changed is exactly as it was: a selection shows an icon
  instead of translating on its own, the card's close button is a settings button, the
  language bars offer only the languages the chosen engine translates, the engine can be
  switched while an answer is on its way, and exchange rates come through the server. Those
  entries are in [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md)
  under their own versions.

<a id="zh-cn"></a>

## 中文

和 v1.2.3 完全相同的应用，只是这次从已提交的源码构建。两个版本之间 Glossy 本身没有任何变化：v1.2.3 的安装包是用一份从未提交的工作区副本构建的，于是它的标签指向一个声明 1.1.0 的提交。本版本把这些改动正式提交，并从与版本号相符的标签构建安装包。

### 变更

- **标签构建的就是它自己声明的那个版本。** 推送 `vX.Y.Z` 标签后，Windows runner 会用本地发布同一套 `scripts/release.ps1` 构建安装包、MSI 与便携 zip（`.github/workflows/release.yml`）；当标签与提交里声明的版本不一致时，运行会在构建之前停下。发布不可能再出现名字、文件与来源代码三者对不上的情况。除此之外每个版本的流程没变：release 的说明仍然是已提交的 `release/vX.Y.Z/RELEASE_NOTES.md`，将来有证书时，对外发布的仍是本机签名的构建。
- **CHANGELOG 找回了 1.2.1 一节。** 1.2.2 那节把语言栏条目连同 `1.2.1` 标题一起吞掉了，于是汇率改动和语言栏改动被列在同一个版本下。现在它们各自回到 1.2.1 和 1.2.2。

### 未变

- v1.2.2 与 v1.2.3 的改动原样保留：选区只显示图标而不自行翻译、卡片关闭按钮改成了设置按钮、语言栏只提供所选引擎能翻的语言、翻译途中也能切换引擎、汇率改由服务器转发。这些条目在 [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) 里各自对应的版本下。

<a id="es"></a>

## Español

La misma aplicación que la v1.2.3, pero esta vez compilada desde código registrado en el repositorio. Nada cambió en Glossy entre ambas versiones: los instaladores de la v1.2.3 se compilaron desde una copia de trabajo que nunca se registró, así que su etiqueta apuntaba a una confirmación que declaraba la 1.1.0. Esta versión registra ese trabajo y compila los instaladores desde la etiqueta que los nombra.

### Cambiado

- **Una etiqueta ahora compila la versión que nombra.** Al enviar una etiqueta `vX.Y.Z`, un runner de Windows compila el instalador, el MSI y el zip portátil (`.github/workflows/release.yml`) con el mismo `scripts/release.ps1` que usa una versión local, y la ejecución se detiene antes de compilar si la etiqueta no coincide con la versión que declara su confirmación. Ya no puede publicarse una versión cuyo nombre y archivos no concuerden con el código del que salieron. El resto del proceso por versión no cambia: las notas siguen siendo el `release/vX.Y.Z/RELEASE_NOTES.md` registrado, y cuando haya certificado se seguirá publicando la compilación firmada en un equipo local.
- **El registro de cambios recupera su sección 1.2.1.** La entrada de la 1.2.2 se había tragado la entrada de la barra de idiomas y, con ella, el título `1.2.1`, así que el cambio de tipos de cambio y el de la barra de idiomas aparecían bajo una sola versión. Vuelven a figurar en la 1.2.1 y la 1.2.2.

### Sin cambios

- Todo lo que cambiaron la v1.2.2 y la v1.2.3 sigue igual: una selección muestra un icono en lugar de traducir por su cuenta, el botón de cerrar de la tarjeta es un botón de ajustes, las barras de idiomas solo ofrecen los idiomas que traduce el motor elegido, el motor puede cambiarse mientras una respuesta está en camino y los tipos de cambio pasan por el servidor. Esas entradas están en [CHANGELOG.md](https://github.com/SpencerZXWu/Glossy/blob/main/CHANGELOG.md) bajo su propia versión.

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
