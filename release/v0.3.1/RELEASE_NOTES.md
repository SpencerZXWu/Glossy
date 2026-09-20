[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Changed

- Unit and currency conversion reads the numbers and units out of the
  **translation** instead of the selection. The translator is what decides
  whether a symbol is a unit at all — it settles `5 in the morning` against
  `5 in`, and it writes `12 ft` as `12英尺` for a Chinese reader — so the card
  converts what the reader is actually being shown. A source language whose unit
  words the tables never knew now works as soon as the translation writes the
  measurement in one they do, for example Spanish `mide 12 pies de ancho` →
  `房间宽12英尺。` → `12英尺 ≈ 3.66 m`. When the translation holds nothing
  convertible — the translator dropped the measurement, or spelled the number
  out in words — the original is read instead, so nothing that used to be
  annotated lost its annotation. Where both have something, the translation is
  the one that is shown.

<a id="zh-cn"></a>

## 中文

### 变更

- 单位与货币换算改为从**译文**中读取数字和单位，不再读选中的原文。一个符号到底是不是
  单位，本来就该由译者决定：它分得清 `5 in the morning` 和 `5 in`，也会为中文读者把
  `12 ft` 写成 `12英尺`，所以卡片换算的正是读者眼前看到的那句话。源语言里的单位词，
  即使内置单位表从来没见过，只要译文用了它认识的写法就能换算，例如西班牙语
  `mide 12 pies de ancho` → `房间宽12英尺。` → `12英尺 ≈ 3.66 m`。译文里没有任何可换算的
  内容时——译者省掉了计量单位，或者把数字写成了汉字——就改为读原文，这样以前能被标注的
  内容不会丢掉。两者都有内容时，以译文为准。

<a id="es"></a>

## Español

### Cambios

- La conversión de unidades y divisas lee ahora los números y las unidades de la
  **traducción**, no de la selección. El traductor es quien decide si un símbolo es
  una unidad: distingue `5 in the morning` de `5 in` y escribe `12 ft` como `12英尺`
  para un lector chino, así que la tarjeta convierte lo que el lector está viendo de
  verdad. Un idioma de origen cuyas palabras de unidad las tablas nunca conocieron ya
  funciona en cuanto la traducción escribe la medida en una que sí conocen; por
  ejemplo, el español `mide 12 pies de ancho` → `房间宽12英尺。` → `12英尺 ≈ 3.66 m`.
  Cuando la traducción no tiene nada convertible —el traductor omitió la medida o
  escribió el número con palabras— se lee el original en su lugar, de modo que nada de
  lo que antes se anotaba se queda sin anotar. Si ambos tienen algo, se muestra la
  traducción.
