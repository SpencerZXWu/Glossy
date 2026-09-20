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
