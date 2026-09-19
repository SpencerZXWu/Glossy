### Added

- Unit and currency conversion in the card. A measurement or an amount written
  the way the original's language writes it, but not the way the target language
  does, is expressed a second time in the unit a reader of the target language
  expects, with the rate underneath: `12 ft ≈ 3.66 m` / `1 ft = 0.3048 m`,
  `212 °F ≈ 100 °C` / `°C = (°F − 32) × 5/9`, `200 US dollars ≈ ¥1,342.82` /
  `1 USD = 6.71409 CNY`. Length, mass, volume, speed, area and temperature
  convert inside their own category; the target unit is picked so a person would
  read it (5 mi → `8.05 km`, 100 lb → `45.4 kg`, never `45,359,237 mg`). Metric
  is used for every target language except English, and the target currency
  follows the target language (`zh-CN` → CNY, `en` → USD, `ja` → JPY, and so on).
- Live currency rates. The rate comes from `open.er-api.com` with the ECB
  (`frankfurter.app`) as a fallback, is cached in `%APPDATA%\com.glossy.translator\rates.json`
  for six hours, and is usable for seven days: an old table is still shown, marked
  `Stale rate`, rather than nothing. Every other unit is built into the app, so
  the feature works offline. A card whose original holds money waits at most
  2.5 s for the rate (2164 ms cold, 390 ms once the table is cached).
- A `Convert units and currency` switch in the settings. Off means the card shows
  nothing but the translation, and no rate is ever requested. On by default.
- One visual language for the three windows. A single token sheet
  (`src/styles/tokens.css`) holds every colour, radius, shadow, font size and
  duration — the palette is the Windows 11 / WinUI one, and the surface sheets
  only alias the tokens onto the names their rules already used — so the popup,
  the settings window and the "Glossy is running" card can no longer drift apart.
  `theme.js` resolves `system`/`light`/`dark` in one place instead of three
  copies of the same branch.
- The Mica material behind the settings window. On Windows 11 the window is
  transparent and Windows draws the desktop-tinted backdrop; a title bar colour
  that follows the theme is set at the same time. A build too old for Mica keeps
  an opaque background of its own — the stylesheet only turns transparent once
  the backend confirms the backdrop (`SurfaceInfo.ready`), which also removes the
  grey flash the window used to open with.

### Changed

- The card says `Looking up the dictionary…` while the phonetic symbols, parts of
  speech and definitions are still on their way, instead of staying a bare
  translation and changing later.
- Word lookups share one 1.5 s budget between their two sources, and a source that
  just timed out is not asked again for two minutes. The first lookup of a word
  went from 6025 ms to 1269 ms, and every lookup after it to 289 ms. The
  placeholder is on screen after 300–600 ms.
