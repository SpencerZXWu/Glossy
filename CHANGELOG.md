# Changelog

All notable changes to Glossy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
See [ROADMAP.md](./ROADMAP.md) for what is planned next.

## [2.1.0] - 2026-10-05

The release that lets the window be something other than grey, that opens with a
guided tour of what it does, and that gives Ctrl+Enter back to the field it is typed in.

### Added

- **A guided tour on the first launch.** The window that only ever opens once now
  explains itself: four steps — selecting a text, typing or pasting one, reading a
  screenshot, and downloading what has to be on this machine — each with its own
  short animation and two lines saying what to do and what happens. The scenes are
  drawn from the same tokens as the window behind them rather than recorded, so
  they follow the palette, the accent and the interface language, and a reader who
  asked Windows for less motion gets the last frame of each scene instead of no
  scene at all. The rail, the arrow keys, Escape and a play/pause button work the
  way the four steps deserve, and **Watch the guide again** on the **General**
  page plays it from the start.
- **The state in the title bar is a switch.** *Listening* used to be a label that only said
  what the app was doing; it now takes a click and turns the selection capture off and on
  from wherever the window is, and a second click brings it back. It is the same setting the
  master switch on **General** drives — the label, the dot's colour and the switch itself
  never disagree, whichever of the two is used — and it is named as the switch it is, so a
  screen reader reads its state rather than a colour.

- **Six palettes, and an accent colour of your own.** The **Colours** page used to offer one
  choice — follow Windows, always light, always dark — and now offers three: the mode, the
  palette, and the accent. The palettes are **WinUI** (what the app has always used), **warm
  paper**, **Nord**, **Solarized**, **Dracula** and **true black**, and each states both
  halves, so the mode still decides light or dark and the palette decides the hues. Every
  card in the picker previews itself: its two halves carry that palette's own tokens, so what
  a card shows is what choosing it does, and the light and dark halves are both visible
  whichever mode the window is in. The row under it takes an accent — hand the choice back to
  the palette, or name one of eight colours, or pick any colour at all with the system's own
  picker — and the whole app follows it, selection colour and caret included.
- **The window frame follows the palette.** Windows draws the title bar and the frame, so a
  palette states its window colour to them through DWM (`set_window_surface`), and choosing
  one takes the window off the Mica backdrop: Mica is tinted by the desktop, which is exactly
  what a palette with a window colour of its own cannot have. The default palette keeps both,
  as it always has.

### Changed

- **The interface language moved to the Language page.** It was a select in the title bar,
  where it sat next to the state of the capture and read as a translation setting rather than
  one of the window's own; it now has a labelled row on the page that is named after it, in
  both languages.
- **The palette layer is one layer of the token system.** Every colour in a palette lives in
  `tokens.css` beside the two the app already had, and a palette states only the colours that
  carry meaning — the three opaque surfaces, the four text tiers, the strokes, the accent and
  the three states. The layering fills stay the white and black alphas they always were, so a
  palette changes the colours and never the depth system. `tests/tokens.test.js` re-measures
  the file: every text tier and the label on an accent fill has to clear 4.5:1 in all twelve
  palette-and-mode pairs, the WinUI palette has to stay identical to the `:root` defaults it
  is stated beside, and the picker has to offer exactly the palettes the stylesheet defines.
- **Text selection, the caret and the accent of a checkbox come from the palette too.** They
  were WebView2's defaults, which belong to no design system, and a themed window that keeps
  them reads as half-finished.

### Fixed

- **Dragging across a picture no longer copied it and threw the selection away.** A drag was
  taken as *the user has selected text* without asking whether anything had been selected, so
  dragging across images to pick several of them in WeChat — or in any program that lets a
  drag move or select something that is not text — had Glossy press Ctrl+C into the program,
  copying the picture and cancelling the gesture the user was in the middle of. A drag or a
  double click now asks the focused element through UI Automation whether it has a text
  selection at all, and only presses the shortcut when the answer is yes or when the element
  cannot say: a control with text and nothing selected is left alone entirely, while the
  selection shortcut is never affected, because the user pressed it on purpose.
- **A Windows contrast theme is no longer painted over.** The `prefers-contrast` block was
  stated on `:root` alone, which every `:root[data-theme="dark"]` rule outranked — so a
  contrast scheme with a dark Windows theme behind it had its palette replaced by the app's
  dark one, which is the opposite of following the setting.
- **A custom accent takes the text colour that reads better on it.** The choice was made at a
  luminance of 0.5, which leaves every mid-tone accent — amber, sky, coral — with white text
  at about 2:1 on it. The crossover is 0.179, and both halves are compared rather than guessed.
- **Ctrl+Enter in the card's grey original replaced the text behind it instead of translating
  the correction.** The card's write-back key is registered with Windows rather than with the
  page, and a registered shortcut is delivered to Glossy before the window that has the caret —
  so with the caret in the editable original, Ctrl+Enter never reached the field that promises to
  translate what is typed in it. What happened instead was the translation being pasted back over
  the original text in the program behind the card. The field now tells the card when it holds the
  caret, and the card lets go of the accelerator for as long as it does, which is the pair the
  field's own hint and the button's label already described: Ctrl+Enter translates the correction
  inside the field, and writes the translation back anywhere else on the card.

## [2.0.0] - 2026-10-02

The release that writes the project's promises down, and uses the contract break it is
allowed to take to put the settings file in one piece.

### Added

- **`FAQ.md`, `PROVIDERS.md`, `COMPATIBILITY.md`.** The questions the README answered only
  in passing, the terms of every translation service the app offers, and what a new version
  may do to an installation that already exists — three documents a reader can check instead
  of trusting, each linked from the README.

### Changed

- **The settings file is format version `2`.** The choice of translation service used to be
  spelled across four keys, because the app once offered modes it no longer has; it is now
  one `service` value (`cloud-baidu`, `cloud-youdao`, `google` or `offline`). Every settings
  file from `1.3.0` onwards is migrated on first launch, and nothing else in it moves: the
  language, the shortcuts, the ignored programs and the history setting all arrive as they
  were. This is the one break a major version is allowed to make, and
  [`COMPATIBILITY.md`](./COMPATIBILITY.md) is the promise about it, key by key.
- **A file written by a newer version is set aside rather than misread.** The rule already
  existed; the compatibility statement is where it is written down now.

### Removed

- **The mode that took an API key of the user's own**, and the storage that went with it: the
  `credentials` map, the `apiKey` and `appId` fields, the DPAPI machinery that encrypted them
  and the module that held it. No service this build offers asks for a key — the two
  server-backed entries hold the vendor account on the relay, Google takes none and the
  offline models run here — so the mode had nothing left to configure. A key left in an old
  settings file is dropped the next time the settings are saved, and is neither used nor sent
  anywhere. Nothing in either window changes: the credential panel was already gone in `0.8.0`,
  and the dropdown had four entries before this release and has four after it.
- **The relay address as a setting.** It was already part of the build rather than a field in
  the window; the `cloudEndpoint` key a `1.x` file may hold is dropped with the rest.

### Fixed

- **Choosing the offline channel in the settings window did not stick.** The list the window
  validates the stored service against held only the three services that go through a network,
  so `offline` was read as an unknown value and the dropdown fell back to Baidu the moment the
  save came back — the choice was written to the file and then shown as something else, and
  reopening the window showed Baidu while the card translated offline. The list holds all four
  entries now, and the list of fallbacks is a second, shorter one that leaves `offline` out:
  the models on this machine are a choice the reader makes, never a substitution the app makes
  for them, which is what the README has promised all along. The backend now drops `offline`
  from a hand-edited fallback order too, so the promise holds for a file as well as for a click.
- **`PRIVACY.md` and the README described screenshot translation as going through the relay.**
  It does not: PP-OCRv4 reads the rectangle on this machine and only the recognised text is
  sent, which is what the code has done since the reader became local. The privacy document
  said otherwise, and the Chinese settings table promised a monthly screenshot allowance that
  no build has counted since then. Both now say what actually happens.

## [1.8.3] - 2026-10-01

The release that translates without a connection, and that stops handing out a fresh
allowance to anybody who installs the app again.

### Added

- **Offline translation, the fourth channel.** `opus-mt-en-zh` and `opus-mt-zh-en` — OPUS-MT's
  English and Chinese models, Helsinki-NLP, Apache-2.0 and CC-BY-4.0 — run as int8 ONNX through the same
  runtime the recogniser uses. The text never leaves the machine, there is no allowance to
  count, and a sentence takes about a second. Chinese and English both ways and nothing else.
  The two directions are downloaded and deleted one at a time on the **Resources**
  page — each is about 114 MB and is a whole translator on its own, so one way
  round never waits for the other — and each file is checked against its digest
  before it is used. They come from ModelScope's mirror of the two `Xenova`
  repositories, which carries byte-for-byte the same artifacts and answers about
  ten times faster than the Hugging Face mirrors do from a mainland connection.
  The channel answers with a readable error while a direction is not there, which
  is also what the fallback order does with it.
- **The allowance is shown where it is spent.** Today's free characters used to appear only
  next to the service dropdown on the Language page; the same line is now written on the
  **Translate text** page and the **OCR** page too, because those are the pages a translation
  is started from.

### Fixed

- **A failed card offered no way to change the engine.** The bottom row of the popup — the
  one that names the engine and opens the list of the others — was drawn only on a card that
  had something to show, so a translation that failed left the reader with a retry button and
  nothing else: switching to another engine, the offline one included, meant opening the
  settings window. The row is on the failure card now, which is exactly where a switch is
  wanted, and the card that a failed screenshot draws carries it too.
- **The daily allowance started over after a reinstall.** It is counted per installation id,
  and that id was drawn at random when the settings file was first written — so installing
  Glossy again (or deleting the settings file) was a new device with a new allowance. The id
  is now derived from the machine it runs on together with the Windows account, so an install
  that comes back lands on the id it had. Nothing about the machine is sent anywhere: only the
  32 hex characters the id hashes to.

### Changed

- **Subtitle text size.** The Settings window's subtitle page now takes the size of the
  translated line directly, `13`–`40` CSS pixels, instead of the line being scaled to fit the
  box and nothing else. What is set is still stepped down when a long translation would not
  fit, and the box can still make it bigger than the setting.

## [1.8.2] - 2026-10-01

The release that makes a subtitle reading actually start.

### Fixed

- **A subtitle reading that started and then read nothing.** Starting a run ends the one
  before it, and the way a run was ended took the two boxes with it — so the run that
  followed was left with no area to read and no place to draw in: the loop returned before
  its first tick, the log said the reading had started, nothing was translated, and neither
  dashed box ever appeared. Ending a run and forgetting the boxes are two different things
  now, and a start stores both of its own rectangles after the run before it has been ended.

### Changed

- **The settings window stays where it is when a reading starts.** It used to hide itself,
  because it is in front of the video — but a window that vanished the moment the second
  rectangle was released read as the feature closing itself. It is yours to move, minimise
  or close, and closing it still only hides it, as it does everywhere else in Glossy.

## [1.8.1] - 2026-10-01

The release that makes the two rectangles of a subtitle reading visible, and lets them be moved
while the reading is running.

### Added

- **Both boxes are drawn, and both can be moved.** A region of the screen that is read every
  second is invisible by nature, so each box wears a faint dashed frame: one around the part
  the subtitles are read from, one around the window the translation goes in. Neither frame
  takes a click — they are guides, not targets. **Move the boxes** on the page — or
  *Adjust the subtitle boxes* on the tray menu, which is the way to them while a reading runs
  and the settings window is out of the way — opens both over the video, where they are
  dragged by their middles and resized by their eight handles. A box is never dragged off the
  screen or shrunk below what a reading needs, and the reading follows the boxes it is left
  with: the next line is read from the new rectangle, and the translation is drawn in the new
  place, without stopping and starting again. `Esc` leaves the boxes where they were.

## [1.8.0] - 2026-10-01

The release that reads subtitles off the screen and keeps them translated. It is a page that
does not exist until developer mode is turned on, because it is the first thing Glossy does
that keeps looking at the screen by itself.

### Added

- **Subtitle translation.** Pick the part of the screen the subtitles appear in, then the
  place the translation is to be drawn: Glossy reads that rectangle about once a second,
  translates it whenever what it read changes, and writes the translation into a window
  drawn exactly over the second rectangle. The window is transparent, always on top and
  never takes a click, so a subtitle line is never in the way of the video it belongs to.
  The two languages are chosen before the reading starts — what the subtitles are written
  in, and what they are read in — and only **one** recognition language is used, picked from
  the packs that are already on the disk: reading a small region with one recogniser costs a
  fraction of reading it with every checked language, and a subtitle is only ever written in
  one of them. The region is read on this machine, exactly like a screenshot, and only the
  text that comes back is sent to the translator, so one subtitle line costs one small
  request of the daily allowance and nothing of the picture leaves the machine. The settings
  window puts itself away when a reading starts, because it is in front of the video; the
  tray icon brings it back, and the **Stop** button on the page ends the reading — as does
  turning developer mode off.
- **Developer mode**, on the **Updates** page. It is a key field and nothing else: type the
  key and the pages that are still being built appear in the sidebar; **Turn off** puts them
  away again and closes any of them that is open. The key is compared in the process rather
  than in the window, so the page never holds it, and the switch is stored as
  `developerMode` in `settings.json`. With it off, Glossy behaves exactly as it did before
  this release: the entry is not in the sidebar at all, and a subtitle reading asked for by
  any other route is refused.

### Changed

- **The screenshot overlay says what the rectangle is for.** It is one window used for three
  things now — a screenshot, the area the subtitles are read from, and the place their
  translation is drawn — so the sentence on it names the one that is being asked for rather
  than always talking about recognising text.
- **A page whose sidebar entry is hidden is skipped by the keyboard.** Walking the sidebar
  with the arrows no longer stops on an entry that is not on show, and a page that is open
  when the mode that shows it goes away is closed behind it.

## [1.7.3] - 2026-09-30

The release where a card that fell back by itself says why. It is the same line the relay's
own fallback already wrote; the app simply had nothing to put in it.

### Fixed

- **A fallback the app made itself showed no reason.** When the chosen service fails, the app
  tries the next one and the card names the engine that answered and the one that did not —
  but only the *relay's* fallback carried a reason, because the refusal code it reports came
  with it. The app-side case went out with an empty code, so a card read "Youdao Translate did
  not answer" with nothing after it, while `glossy.log` held the whole story ("Could not reach
  the Glossy translation server"). A service that does not answer now reports why in the same
  small set of codes the relay uses — the vendor's own refusal (`upstream_limit` and friends)
  or `relay_unreachable` when it was the relay itself that never answered — and the card says
  "Youdao Translate did not answer — the Glossy relay could not be reached". The relay's own
  refusals (its daily allowance, its rate limit) deliberately carry no code: a line blaming the
  engine named in the footer for a limit the relay hit would be a lie, and the log has the
  detail.

## [1.7.2] - 2026-09-30

The release that puts the other recognition languages back, after checking each one on a
real screen — where more than one of them can be checked at once and the best reading wins,
and where reading a screenshot got three times quicker.

### Added

- **The notification area menu carries the two actions that need no window.** It had
  **Open Glossy** and **Quit** — but closing the settings window hides it, and that menu is
  then the only place the shortcut keys are written down at all, so **Translate a screenshot**
  and **Translate the clipboard** are on it now as well. Each entry names the combination
  doing the same thing, taken from the settings, and the menu is rebuilt whenever the settings
  are saved, so a shortcut the user records is in it before the next launch. The clipboard
  entry translates what is on the clipboard when it is clicked, rather than pressing `Ctrl+C`
  first the way the shortcut does: a menu click cannot promise the program the user meant is
  still in front. When there is nothing to translate, or translations are switched off, that
  is said in the card instead of the click doing nothing.
- **A screenshot can be read with several languages at once.** The rows on the **Resources**
  page are checkboxes now rather than a single choice: every language that is checked reads
  the same screenshot, and each line is kept from the language that read it best. A picture
  holding Japanese and English no longer needs the user to say which it is, and the choice
  costs almost nothing — measured on a real machine, a screenshot read with two languages
  takes about 0.7 s against 0.5 s for one, and the extra work grows with the languages
  checked rather than with the size of the picture. The order is the one the page shows, at
  least one language always stays checked, and a language the settings file names that this
  build does not offer is dropped rather than carried into a download of a model that does
  not exist.

### Changed

- **The detector stops stretching a wide, short screenshot.** The region is brought up to the
  detector's size on its shorter side, which is what makes small text readable — but on its own
  that turned a 900×140 region into 4736×736, fifteen times the pixels of the picture it came
  from. The longer side is held at 1440 now, so the detector's work stays near the size of the
  region whatever its shape: reading the same picture went from 1.3 s to 0.17 s, and a
  screenshot translation from about 1.4 s to 0.5 s. The recognisers also share one detector
  instead of loading a copy each, which is what made reading with two languages cost 2.3 s
  before it cost 0.7 s.
- **The recognition languages are six again, and each one says what it reads.** The
  **Resources** page offers Chinese and English, Japanese, Traditional Chinese, the Latin-script
  languages, the Cyrillic ones and Korean, as it did before 1.6.1 — each is downloaded, checked
  and removed on its own, and a language already on the disk shows up as installed rather than
  being fetched a second time. The hint above them says the thing that made this look broken:
  a pack reads its own script and nothing else, so the language has to be picked rather than
  guessed.

### Fixed

- **Japanese screenshots were read with the Chinese recogniser.** Since the language packs
  were taken off the **Resources** page in 1.6.1 only Chinese and English could be picked, and
  a settings file still naming another language was quietly read as Chinese and English rather
  than reported — the Chinese pack's dictionary holds no kana at all, so a Japanese screenshot
  came back with every kana missing and the kanji guessed, which is what "Japanese recognition
  is completely wrong" was. Each of the six languages was then read over a picture written in
  its own script — `ch`, `ja`, `cht`, `latin`, `cyrillic` and `ko` — and each one read its own
  script back: Japanese at 0.998, Traditional Chinese at 1.000, English and French at 1.000 and
  0.984, Russian at 1.000, Korean at 0.997. Nothing in the models, the dictionaries or the
  engine was wrong; only the page was, and all six are offered again.
- **The source language a card detected showed up as a code.** Youdao names the pair it
  translated rather than the source alone — `l: "en2zh-CHS"` — and the relay passed the whole
  of it on as `from`, so a card that had asked for the language to be detected read
  "Detect language · EN2ZH-CHS" where "Detect language · English" belongs. The relay now
  answers with the left half of the pair, and `normalize_lang_code` reads a pair that way too,
  so a deployment older than this build is fixed by the app rather than by a redeploy — and a
  pair can no longer reach the next request as a source the provider would reject.
- **The page renamed to *Files and logs* in 1.7.0 still had its old heading.** The sidebar
  said **Files and logs** while the heading above the page said *Settings file*, and the
  English list of pages in `README.md` said the same. Both say what the page is now.

## [1.7.1] - 2026-09-29

The release where a channel that stopped answering says so — and where the card can take the
picture itself. Nothing about which engine is asked first changed; what changed is that the
app no longer looks as if it had moved the choice by itself.

### Added

- **A camera button in the card.** Reading a rectangle off the screen no longer needs the
  keyboard: the button sits in the card's header, next to the pin and the wordbook star. The
  card itself is taken off the screen first — it is always on top and normally sits right next
  to what the user wants to read — and the answer, or the reason there is none, comes back to
  the same card, exactly as it does for `Ctrl+Alt+Q`.
- **The relay reports which backends it walked past.** `POST /v1/translate` now answers with
  `attempts`: one `{vendor, code}` per upstream that refused, in the order they were tried, so
  a deployment that holds both keys can say why the one that was asked for stepped aside
  instead of quietly answering with the other one's translation. The relay also writes it to
  its own log (`console.warn`), where the operator — the only one who can fix a vendor's
  credentials or quota — can see it. A deployment older than this build simply leaves the
  field out, which the app reads the same way.

### Changed

- **A button that has a shortcut says so in its tooltip.** The camera and the settings buttons
  in the card, and the screenshot button in the settings window, now name the combination that
  does the same thing. The three shortcuts are recorded by the user, so the tooltip is built
  from the stored value every time the window is drawn rather than written into the markup: it
  always names the key that works now, and a field the user cleared leaves the tooltip with the
  action alone.

### Fixed

- **The Glossy mark in the settings window's card was 128 pixels square.** Nothing in
  `app.css` gave the mark a size, so the card drawn on the Translate page used the size of the
  file itself while the floating popup drew the same mark at 13 pixels. It is 13 pixels in both
  windows now, faded in the same way, and a contrast theme lifts that fade in both.
- **The fallback list follows the service chosen above it.** The chosen service is always
  asked first, so a copy of it in the list below was a second attempt on the same backend —
  and it pushed one engine out of the list altogether: switching the first entry from Baidu
  to Youdao left Youdao listed twice and Baidu nowhere. The list is now rebuilt from the
  choice: every other engine once, in the order the user put them in, with an engine that
  was not in the list yet landing at the end.
- **A card whose answer came from another engine says which one — and why.** The relay in
  front of Baidu and Youdao walks its own list of backends when the one it was asked for
  fails, which is what made a card read "Youdao" while the settings still said Baidu, with
  the fallback list switched off and nothing to explain it. The card names the engine that
  answered in its footer, and the line under it names the engine that did *not*: "Baidu
  Translate did not answer — its allowance is used up", with the reason the relay gave when it
  gave one (a code this build does not know leaves the line at "Baidu Translate did not
  answer"). The name in that line is the engine (`Baidu Translate`) rather than the identifier
  of the entry of the channel list, which is what it used to print — and it used to say that
  the engine that *failed* was the one that answered. The same line goes into `glossy.log`.

## [1.7.0] - 2026-09-29

The release where Glossy writes down what went wrong. A failure in the mouse hook or in the
card no longer ends the process in silence, and the messages the app used to print to a
console nobody was watching now land in a file beside its settings.

### Added

- **An error log beside the settings.** `glossy.log` sits next to `settings.json` and takes
  every failure the app reports, stamped in UTC. It stops at 256 KB by becoming
  `glossy.log.1` and starting a new file, so a machine that runs for months leaves two
  bounded files instead of one that grows without end; nothing else is ever written into it.
  The new **Files and logs** section of the settings window — the page that was *Settings
  file* — shows its path and how much it holds, and offers `Open the folder`, which opens
  Explorer with the file selected, `Export…`, which writes it to `Documents\glossy-log-<unix
  seconds>.txt` the way an exported settings file is written, `Copy`, which puts it on the
  clipboard to paste into a message, and `Clear`. The log exists because a release is linked
  as a Windows GUI application and has no console at all: everything the app printed with
  `eprintln!` reached nobody when it was started from Explorer or the notification area,
  which is how it is normally started.
- **The Updates section says where a new build can be downloaded by hand.** The line that
  already says this build cannot update itself now ends with the releases address —
  `github.com/SpencerZXWu/Glossy/releases/latest` — and clicking it opens that page in the
  browser, which is the way to a new build until a signing key pair exists. The window itself
  never navigates anywhere: the address is part of the build, and the command that opens it
  takes no argument, so nothing else can be opened this way.

### Changed

- **The log is handed over by the user, not sent anywhere.** The planned anonymous reporting
  — a payload shown before it was sent, described in `PRIVACY.md` — is not built: `Export…`
  and `Copy` give whoever is asked for the log exactly the same thing, and nothing about a
  failure leaves the machine unless the user does it themselves. The settings page that was
  called *Settings file* is now **Files and logs**, because it holds both.

### Fixed

- **A failure in the mouse hook or in the card no longer takes the notification area icon
  with it.** The hook callback is called by Windows, and a panic that unwound out of it
  would have left through an `extern` boundary and aborted the whole process. The click
  handler, the accelerator handlers and the thread that reads a selection are now guarded:
  a failure is written to the log and the next click is answered as if nothing had happened.
  A panic anywhere else is recorded with its location before the process goes, which is the
  only way a crash nobody was watching can still be read afterwards.

## [1.6.1] - 2026-09-28

The release where the machine reads more than Chinese and English, and where the
settings say where every file it downloads comes from. The card for a screenshot is also up
before the picture is read rather than after it, and the engine the card switches to is no
longer at the mercy of the settings window.

### Added

- **The reading engine is a shared half plus a language.** The ONNX Runtime and the detector
  — about 21 MB, shared by every language — are now separate files from a language's
  recogniser and its dictionary, about 10 MB, which is what the app has always downloaded
  for Chinese and English. A language is a pack of its own on the **Resources** page, where
  it can be downloaded, picked and deleted on its own; the engine and the languages already
  on the disk are not fetched again. **Chinese and English is the language this build
  offers**: Japanese, Traditional Chinese, the Latin-script languages, the Cyrillic ones and
  Korean are built the same way and are already in the table the downloader and the notices
  file use, but they are not offered yet — their recognition has to be checked on a real
  screen first, and one word in `ocr/models.rs` offers them.
- **Every downloaded file says where it comes from.** Each row on the Resources page names
  its source and its licence, and [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) — put
  next to the app by the installer and into the portable archive — carries the full licence
  texts together with the address and SHA-256 of everything the app downloads. The list is
  checked against the table the downloader uses, so it cannot quietly go stale. All of it is
  Apache-2.0: PaddleOCR's PP-OCR models, distributed as ONNX by RapidOCR.

### Changed

- **The Resources page is a list of files, not two paragraphs.** Everything that can be
  downloaded is a row of its own with its size, its state and the button that acts on it,
  the languages it can read are one list with the one in use marked, and the sources are
  folded away at the bottom for whoever wants them.
- **The first-use prompt names the size of what it is about to fetch.** A second language no
  longer reads as a second whole engine.

### Fixed

- **The keyboard stays where it was.** Every list in the settings window is drawn
  again rather than updated in place — by a save, by a download's progress, or by the window
  taking the focus back — which used to throw the focus away with the element it was on, so
  picking a language with Space moved the reader to the top of the page. The control that had
  it is found again by what it says about itself, and the row it belonged to stands in when
  the control itself is what the change removed.
- **A language reports its own progress on its own row.** The bar and its numbers used to
  appear under the shared engine and count the runtime and the detector together with the
  language, so a 10 MB download read as a 33 MB one. Each row now counts the files it is
  fetching — the engine row the runtime and the detector, a language's row that language —
  and a failure is reported on the row it belongs to.
- **The card for a screenshot is up while the picture is being read.** Taking a screenshot
  showed nothing at all until the text had been recognised and translated, because the
  waiting card was only put up when the reading engine still had to be downloaded. It now
  appears the moment the rectangle is accepted — with the name of the engine that is about
  to be asked, or saying that the reading engine is being fetched — and the translation
  takes its place.
- **A choice made in the card is no longer undone by the settings window.** The settings
  window writes the whole settings file on every save, engine and target language included,
  taken from its own controls; a window that was not on screen while the card switched its
  engine could write the older choice back, which puts the app back on Baidu. Those two are
  only written back when the pick was made on that screen, and the window re-reads the
  stored settings whenever it comes back into view.

## [1.6.0] - 2026-09-28

The release where the two windows can be worked with the keyboard alone, and where a
contrast theme is something the app follows rather than something it paints over.

### Added

- **The keyboard reaches everything, in both windows.** The settings window walks its
  sidebar with the arrows, Home and End, and its sections with Tab in the order they are
  drawn; the confirm dialog that asks before something is thrown away now keeps the
  keyboard inside its two buttons while it is up, and gives it back to whatever had it
  before. In the card, the caret going into the grey original — or a Tab once the card
  holds the keyboard — is what hands the popup the front, and the engine menu at its foot
  opens with the arrows, moves with the arrows, Home and End, and closes with `Esc` or Tab
  back onto the button it belongs to.
- **The keyboard goes back when the card does.** A card that was given the front so a
  field in it could be typed into returns it to the program the selection came from when
  it closes, instead of leaving it on a window that is no longer there.
- **A high-contrast theme, taken from Windows.** With a contrast theme turned on, every
  colour in the app comes from the system palette: the surfaces turn solid, the strokes
  stop being subtle, the accent becomes the system highlight and the focus rings stay on
  top of it. It is not an entry in the theme menu, because the choice was already made in
  Windows — and a custom accent stops overriding the system highlight for as long as one
  is on.

### Changed

- **Nothing is read through something else while a contrast theme is on.** The card's
  opacity, the faint wordmark at its foot, the dimmed sections whose controls are off, and
  the pulsing placeholder all keep their own colour and their own weight in that mode.

### Fixed

- **The same measurement written twice is one row.** The annotation under the card
  compared the numbers as they were typed, so `12 ft` twice was one row but `12ft` next to
  `12 ft`, or `12 ft` next to `12 feet`, became two rows with the same answer. It compares
  the amount and the unit now, so a repeated measurement is one row however the
  translation spells it — and two different amounts are still two rows, which is what the
  reader asked about.
- **A shortcut field no longer holds the keyboard.** Recording starts when the box is
  focused, and it used to record every key it saw — including Tab, which is how the
  keyboard had reached the box. Tab and Shift+Tab now leave it unchanged, the hint under it
  says so, and the combination that was there is still there afterwards.

## [1.5.5] - 2026-09-27

The release where the screen is read on the machine it is on, and where the settings stop
being a list of pages that each held one switch.

### Added

- **The screenshot is read here, not by a service.** Recognition runs on this machine
  through PP-OCRv4 on ONNX Runtime: the picture never leaves the machine, it works with no
  connection at all, and a reading costs nothing. The engine is fetched the first time a
  screenshot is asked for — a prompt gives its size, about 33 MB, before anything is
  downloaded — and lands next to the app's own files, where *Delete the engine* takes it
  off the disk again.
- **A Resources page.** Everything Glossy keeps on the machine is downloaded, sized and
  removed in one place: the recognition engine today, and the offline translation pack when
  it lands.
- **The version this copy is.** The Updates page prints it above the check button, so a
  report about a broken build starts with the number that identifies it.

### Changed

- **The settings are fewer pages.** *General* carries the shortcuts and the trigger rules
  as well as the master switch — the three were read together anyway — and *Wordbook* moved
  up into the features, where the rest of what Glossy does lives.
- **Document translation is closed while it is rebuilt.** The entry is disabled and marked
  *In development*, the page says so, and the commands behind it refuse instead of starting
  a translation that cannot be finished yet. Settings already saved are untouched.

### Removed

- **The monthly ceiling on screen readings.** Recognition happens on the machine now, so
  there is no allowance to stay inside: the screenshot page no longer reports how many
  readings the month has left, and the app no longer reads a remaining-allowance figure out
  of the shared server's answer.

## [1.5.3] - 2026-09-26

The release where replacing a selection actually lands on it, in every program it was
asked to.

### Fixed

- **The replacement is written over the selection instead of next to it.** A paste goes to
  whichever window is in front, and the card was in front because it had just been clicked,
  so `Ctrl+Enter` could land in the popup or at the end of the line. The window the
  selection was read from — remembered while the selection is still live — is put back in
  front and given the keyboard before the paste, and keeps it until the paste has been
  made.
- **Clicking the badge no longer throws away the selection in WeChat.** WeChat clears the
  text it has selected the moment it stops being the window in front, so the badge — whose
  whole job is to be clicked — cost the very selection the translation was meant to
  replace. The popup is shown without taking the foreground, and the style that does that
  survives being shown, so the click lands on Glossy while the program behind it stays in
  front and keeps the selection selected. The card's own original can still be corrected in
  place: opening that field is what asks for the keyboard.

## [1.5.2] - 2026-09-26

The release where a translation can take the place of the text it came from, and where the
Document page asks for everything it needs to know before it starts instead of after.

### Added

- **A translation can replace the text it was made from.** With the card up for a
  selection, `Ctrl+Enter` — or the third button under the translation — writes the
  translation over that selection in the program in front: select, read, replace, which is
  what writing a reply or a paragraph in another language is actually made of. The key is
  registered for exactly as long as a card holds the translation of a selection that is
  still in place, so no other program loses `Ctrl+Enter` to Glossy, and a card brought back
  from the history — which has no selection behind it — never takes it at all. The text
  travels through the clipboard with a pasted `Ctrl+V`, so the clipboard is put back the way
  the *restore clipboard* setting asks.
- **The Document page asks for its languages and its options up front.** *Document* opens
  on a box a file can be dropped onto or clicked to browse through, a source/target pair of
  pickers above it, and an *Advanced settings* block: **Leave the pieces that hold no
  letters as they are** keeps numbers, symbols and page furniture out of the bill,
  **Save the translation as soon as it is finished** writes the file without a second
  click, and **Longest piece sent in one request** caps how much goes into a single call.
- **A translated document is saved where you say.** The page remembers a folder and shows
  it next to the buttons, `Change…` opens the folder picker, and until one is chosen the
  translation lands on the **Desktop** rather than in `Documents`.

### Fixed

- **Clicking the gear on a pinned card no longer throws the card away.** The settings
  button was the one header button that still dismissed the card it was pressed on, so a
  card the user had explicitly asked to stay disappeared exactly when it was needed.

## [1.5.1] - 2026-09-26

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

## [1.5.0] - 2026-09-25

The release where Glossy stops translating only what the mouse can reach. Three global
shortcuts instead of one, a card that says what it was an answer to, and two ways of
handing it text a selection cannot carry: a rectangle drawn over the screen, and a whole
document. The mouse is still the default — a selection still waits for the icon to be
clicked — but nothing about a translation has to start with the mouse any more.

### Added

- **Three global shortcuts, each one recorded and each one checked.** `hotkey` stays the
  translate shortcut (`Ctrl+Alt+C`), `hotkey_settings` brings the window to the front
  (`Ctrl+Alt+G`) and `hotkey_ocr` starts a screen reading (`Ctrl+Alt+Q`). `hotkey::Slot`
  is what the hook reports, so the callback decides what a press means instead of
  assuming every combination is a translation, and the settings page lists all three with
  their own field, their own clash line and their own `Clear`. A slot left empty is not
  registered at all, and one Windows refuses is reported without taking the others down.
- **Reading text off the screen.** A screen shot of the whole desktop is taken behind a
  frozen-looking overlay, a rectangle is dragged over the text, and what is inside it goes
  to Glossy's server, which asks an OCR service and answers with the recognised text. That text
  is then translated by the ordinary popup path — the same card, the same allowances, the
  same history. `Esc`, a right click or a rectangle that selects nothing cancels without a
  request; the overlay is a real window above everything, so the screenshot cannot capture
  the overlay itself.
- **Translating a document.** The **Document** page under *Features* takes a `.txt`, `.md`,
  `.srt`, `.pdf` or `.docx` file and translates the parts of it that are prose: Markdown
  keeps its code fences, its headings and its lists, and a subtitle file keeps its cue
  numbers and time codes, so the result lines up with the original. Text is sent paragraph
  by paragraph, capped at 1500 characters per request and 60 000 characters per document,
  and the page reports how far along it is and can be cancelled between requests. **Save the
  translation** writes `<name>.<target language>.<ext>` into `Documents` and never touches
  the file that was read. A file that is not UTF-8 — a BOM, or the system code page — is
  decoded rather than reported as unreadable, which is what `platform::windows::encoding`
  is for.
  A **PDF** is read into the words of its pages without a layout engine — wrapped lines are
  joined back into paragraphs, a scanned page is refused before any allowance is spent on it
  — and its translation is written as a `.txt`. A **Word** document is taken apart
  paragraph by paragraph and the translation is written back into a copy of the same
  `.docx`: the translated text takes the place of the first run of each paragraph, so the
  styles, the tables, the pictures, the headers and the footers are the ones the reader
  already knows. An older `.doc` has to be saved as `.docx` first. Both are read and written
  in process, with no service and no network involved.
- **The card shows what was translated.** Both shapes of card — the word card and the
  sentence card — now open with the grey original text under the language row, not just the
  sentence card as before, and that grey text can be edited in place: click it, correct the
  selection Glossy was handed, and `Ctrl+Enter` or clicking away translates the corrected
  text instead. Nothing is sent while the caret is still in the field, an empty or
  unchanged text costs nothing, and `Esc` puts the translated text back without closing the
  card.
- **The Glossy mark closes the line that names the engine.** The row that says which
  service answered ends with the translucent logo and the wordmark, so a result that came
  through somebody else's endpoint still says whose window it is in.

### Changed

- **A shortcut is no longer assumed to mean "translate".** The hook passes the slot it
  fired on, the tray and the screen reader are reached from the same thread the hotkey
  already used, and the window a shortcut opens is raised whether it was hidden or
  minimised.
- **The selection path is unchanged on purpose.** A drag still costs nothing until the
  icon is clicked: the icon appears under the selection, clicking it translates, and
  clicking anywhere else makes it disappear. A shortcut, by contrast, translates at once —
  there is nothing to click after it, which is the whole difference between the two.
- **Screen reading reads pictures with Tencent Cloud, and Baidu is the fallback.** Baidu's
  general text recognition now has to be paid for before it will answer at all, and a free
  key that is refused with `error_code 6` is not something a self-hosted deployment should
  hit on its first screenshot. `server/src/tencent-ocr.js` signs a `GeneralBasicOCR` call
  with TC3-HMAC-SHA256, so a deployment holding a `TENCENT_SECRET_ID`/`TENCENT_SECRET_KEY`
  pair gets the 1,000 calls that interface hands out free every month (reissued on the first
  of each month and valid only for that month, so it never accumulates and a month that has
  used it up is refused until the next one). Both pairs can be set at once:
  Tencent is asked first and the request is only handed to Baidu when Tencent is
  unreachable, out of quota or refusing the keys — a picture that simply holds no text is
  not asked twice. `GET /v1/health` now reports which one a deployment will use as
  `ocrVendor`, and a refusal answers with the vendor's own code in `upstream`, so an
  account that is not set up can be told apart from a request that is malformed.

### Fixed

- **The screenshot overlay can always be put away.** The overlay covers a whole monitor,
  sits above everything and is the window that receives the clicks and the keys, so a page
  inside it that did not come up left the desktop covered by something that could not be
  dismissed. `Esc` and the right button are now watched from the process as well, not only
  from the page, the overlay closes itself after 30 seconds, and pressing `Ctrl+Alt+Q`
  again while it is up puts it away instead of starting a second screenshot. The hint line
  says all three. The page keeps cancelling on `Esc`, on a right click and on a click that
  drags nothing, which is still the usual way out.

## [1.4.1] - 2026-09-25
The global hotkey stops being a text field. A combination has to be spelled the way the
backend reads it back, and typing it by hand was the one place a working setup could be
broken by a typo, so the field now records what is pressed instead of accepting what is
written.

### Added

- **The hotkey is recorded, not typed.** `src/js/keycombo.js` reads a keyboard event into
  the spelling `hotkey::parse` accepts: modifiers in a fixed order, letters and digits
  upper-cased, function keys and the named keys the backend knows. The field opens for
  recording when it is clicked, `Record` reopens it, `Esc` or clicking away puts the stored
  combination back, and `Clear` switches the hotkey off. A key Glossy cannot name is
  reported by name instead of being silently dropped; a stray key without a modifier says
  which modifier is missing.
- **Combinations Windows will not hand over are refused before they are saved.**
  `Ctrl+Alt+Delete`, `Win+L`, `Win+Tab`, `Alt+Tab`, `Alt+Esc`, `Ctrl+Esc` and
  `Ctrl+Shift+Esc` never reach `RegisterHotKey`, so recording one now says so and keeps the
  combination that was working. Ones Windows or Explorer usually owns — `Win+D`, `Win+E`,
  `Win+R`, `Alt+F4` and the like — are recorded with a warning, because only the
  registration can tell. A test reads the named keys out of `hotkey.rs` and fails if the two
  tables ever drift apart.

## [1.4.0] - 2026-09-25

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

## [1.3.0] - 2026-09-25

The settings file and the command names stop moving. Nothing changes on screen; what this
version adds is the promise that a future release keeps reading the file this one writes,
and a test that fails when the promise is broken by accident.

### Added

- **The settings file says which format it is in.** `%APPDATA%\com.glossy.translator\settings.json`
  now opens with `formatVersion`. Adding a setting does not move it; renaming one, removing
  one, or changing what one means does, and a file written before the change is carried
  forward by `Settings::migrate` before it is merged key by key, so nothing is lost on the
  way up (`src-tauri/src/settings.rs`).
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
  it with the command list in `lib.rs` and with every `invoke("…")` in `src/js/`, so adding,
  renaming or removing one of them fails CI until the contract is edited in the same
  commit.

## [1.2.4] - 2026-09-24

The release v1.2.3 was meant to be: the same application, built from committed source. The
v1.2.3 installers were built from a working copy that was never committed, so its tag
pointed at a commit that declared 1.1.0. This version commits that work and builds the
installers from the tag that names them.

### Changed

- **The installer is built from the tag that names it.** Pushing a `vX.Y.Z` tag now builds
  the installer, the MSI and the portable package on a Windows runner
  (`.github/workflows/release.yml`) through the same `scripts/release.ps1` a local release
  uses, and the run stops before the build when the tag does not match the version its
  commit declares, so a release cannot go out whose name and files disagree with the source
  they came from. A release signed on a workstation is still what ships once there is a
  certificate: `scripts/release.ps1 -Sign -Publish` pushes the same tag, and the workflow
  that starts from that push finds the release already published and leaves it alone.
- **The changelog has its 1.2.1 section back.** The 1.2.2 entry had swallowed the language
  bar entry, and the `1.2.1` heading with it, so the exchange-rate change and the language
  bar change were listed under one version. They are under 1.2.1 and 1.2.2 again.

## [1.2.3] - 2026-09-24

The engine can be changed while an answer is still on its way.

### Changed

- **The engine switcher is on the card before the answer is.** The name in the card's bottom
  row was drawn from the engine that answered, so a card that had not been filled yet had no
  bottom row at all: a translation that was slow — or one that went to the wrong engine — had
  to be waited out before the list of engines could be opened, and the word card behaved the
  same way while its dictionary lookups were still running. The row is now drawn on the
  loading card too, and the name in it is the engine the settings picked, which is the one
  that was asked rather than the one that answered (`src/js/render.js`, `src/js/popup.js`).
  Choosing from it during a translation restarts that translation on the engine that was
  picked: the answer on its way is dropped, and the characters are billed to the new engine.
- **The Worker's default daily allowance matches the code.** `DAILY_CHARS_TOTAL` in
  `server/wrangler.toml` said `300000` while the code default and the function that is
  deployed both use `30000`; the file now says `30000`, with the reasoning written next to
  it, so a Worker deployed from it allows the same day as the function already running.

## [1.2.2] - 2026-09-24

A selection no longer costs anything the moment it is made. Glossy answers it with a small
icon under the text, and the translation — and the day's allowance with it — waits for a
click on that icon. The card's close button is gone, and a settings button has taken its
place.

### Changed

- **A selection shows an icon instead of the card.** Releasing the mouse used to start the
  translation straight away, which spent one of the day's translations on every word the
  reader happened to swipe over. The selection now puts a 44px Glossy icon under the text
  and stops there: the translation is asked for when the icon is clicked, so a selection
  that was never meant to be translated costs nothing. Clicking anywhere else dismisses
  the icon, and the shortcut (`Ctrl+Alt+C`) behaves the same way.
- **The card's close button is a settings button.** Closing the card is what a click
  anywhere else already does, so the `×` was one button for something the window never
  needed help with; it now opens the settings window through the new `open_settings`
  command. `Esc` still closes the card.

The unit card's currency conversions now go through Glossy's own server instead of straight
to a third-party rate feed.

### Changed

- **The exchange rate is fetched through the server.** Every currency conversion used to
  reach out to `open.er-api.com` (with `api.frankfurter.app` as a fallback) from the client,
  which made one third-party service per conversion the one call in the app that bypassed
  the relay. The server gained `GET /v1/rates?client=<id>&base=<code>`
  (`server/src/rates.js`, wired into `server/src/upstream.js` and `server/src/handler.js`):
  it normalises both vendors into one table and costs **no character allowance** — it is
  booked as `chars: 0`, so a card that adds a converted amount never eats into the day's
  translations. `src-tauri/src/units/currency.rs` asks the relay first and only falls back
  to calling the two vendors itself when the server cannot answer, so an older deployment
  keeps working exactly as before.

## [1.2.1] - 2026-09-24

A language bar now tells the truth about the engine behind it. The eight languages a
standard Baidu account refuses are no longer offered, and the language a reader picks in
the card is the language the next selection starts from.

### Changed

- **A language bar offers only the languages the chosen engine translates.** The bars were
  built from one list for all three engines, and a standard Baidu account refuses eight of
  its 31 entries — `uk`, `tr`, `hi`, `id`, `ms`, `he`, `no` and `sk` — so 百度 could be
  asked for a language it answers with `58001`. `src-tauri/src/translate/languages.rs` is
  now the only table (`ALL`, plus `BAIDU`, `YOUDAO` and Google's, which is every language),
  published to both windows through the new `service_languages` command, and the two bars —
  the card's and the settings window's — rebuild themselves from it whenever the engine
  changes, in either direction. A code the provider detected that the shared menu never had
  stays selectable, so a bar is never blank, and a target the backend is handed anyway is
  clamped to a language the engine does take rather than sent and refused.
- **The language a reader picks in the card is the configured language from then on.** The
  target chosen in either window's bar was good for the card on screen and forgotten at the
  next selection, so translating a page into Japanese and then selecting the next word put
  it back into the configured language. The new `set_target_lang` stores the pick, and the
  source language keeps its old behaviour — it is reset to *detect it* for every selection.
  The swap button does not save either, so a reversal does not change the setting.

## [1.2.0] - 2026-09-22

Windows is no longer the operating system of the whole backend: every call into Win32 now
lives behind one dispatch layer, so a port has a single place to stand. The window content
is governed by an explicit Content Security Policy instead of none, and a release ships
three ways — the installer, an MSI package and a portable zip. macOS and Linux are still
ahead (`src/platform/mod.rs` refuses to compile anywhere but Windows), and this release
deliberately stops at the interface those ports will be built against.

### Added

- **The platform layer.** `src/platform/mod.rs` publishes what the rest of the backend is
  allowed to assume about an operating system — without a screen, `ScreenRect` and its
  padded hit test — and dispatches to `src/platform/windows/`, which holds the ten modules
  that talk to Win32: `clipboard`, `console`, `desktop`, `hotkey`, `input`, `input_hook`,
  `instance`, `secrets`, `speech` and `uia`. A target that is not Windows fails to compile
  with a message pointing at the layer instead of failing to link. No module in it leaks a
  `windows` type through its public surface: the low-level mouse hook reports a
  `Click { pressed, x, y }`, the window handle travels as an `isize`, and the accessibility
  text read for a word's sentence is asked for as a `Unit::Line` or `Unit::Paragraph`.
- **An MSI package and a portable zip, next to the installer.** `bundle.targets` builds
  NSIS and MSI, and `scripts/release.ps1` stages both, then packs
  `Glossy_<version>_x64_portable.zip` — the release binary and the `WebView2Loader.dll` the
  GNU toolchain links against, with nothing to install — and checksums all three. A build
  that names MSI in its targets but produces no MSI fails the release instead of staging
  two files out of three.

### Security

- **The content is behind a policy now.** `app.security.csp` was `null`, which let the
  window load anything a script asked for. It is an explicit whitelist: `default-src`,
  `script-src`, `style-src` and `font-src` are `'self'`, images may add `data:`, and
  `object-src`, `frame-src` and `form-action` are closed. The frontend loads no remote
  script, style, font or image, so nothing had to be widened for it — verified in the
  running app by reading each window's console, where a deliberate remote `fetch` is
  refused and nothing this app loads is.

### Changed

- **The backend's OS-bound modules moved under `src/platform/windows/`.** `clipboard.rs`,
  `console.rs`, `hotkey.rs`, `input.rs`, `instance.rs`, `secrets.rs`, `speech.rs` and
  `platform.rs` (now `desktop.rs`) were moved, not rewritten, so the behaviour of a
  selection is the same one as before. What they exported is reached through the layer
  now: `platform::clipboard`, `platform::hotkey`, `platform::speech`, `platform::instance`
  and `platform::desktop` from the window setup, the popup, the hint, the settings and the
  console borrow. The mouse hook itself was split: the thread, the message loop and the
  callback plumbing live in `platform::windows::input_hook`, and the click chain that
  decides what a gesture meant stays in `selection.rs`, where it can be tested without a
  screen.
- **`windows` and `window-vibrancy` are Windows-only dependencies.** `Cargo.toml` declares
  them under `[target.'cfg(windows)'.dependencies]`, so the tree a port starts from does
  not contain them at all.
- **The screenshot-shaped parts still inside the Windows modules are named as debt.** The
  hotkey's key parsing, the speech queue, `secrets::is_protected`, `context::sentence_in`
  and the click chain in `selection.rs` are portable but still sit in OS-bound files; the
  capability table in `src/platform/mod.rs` says so, and the roadmap records it.

### Fixed

- **The start hint goes away every time it appears.** A 6.5 second timer in the page took it
  off screen, and the page loads once: the timer ran for the first hint and never again, so
  the card a later start of Glossy put back stayed there for as long as the process lived.
  Starting Glossy a second time — double clicking the shortcut again, which is how most
  people reopen the settings window — is exactly the path that showed it. The countdown runs
  in `notice.rs` now, started by `notice::show` itself and guarded by a show counter, so
  every show gets a full one and a countdown left over from an earlier hint cannot take down
  the one that replaced it. `notice.js` no longer arms a timer.
- **The names that live only in an attribute are translated.** `i18n.apply` fills
  `data-i18n-aria-label` beside `data-i18n-title`, which is what the close button of the hint
  and the interface language selector were missing: both had an English tooltip and an
  English accessible name in every language, and the selector's `ui.lang` key — written for
  exactly this control — had no reader left. It has one again.
- **A counted sentence reads correctly for one item.** `"{0} program(s) ignored."` printed
  its own brackets at one program. `ignored.count` and `source.count` are stored as
  `key.one` and `key.other` and read through the new `i18n.plural`, so a single item says
  `1 program ignored.` The Chinese text is unchanged.
- **Choosing a language no longer takes the card away.** The list of a language selector is
  a window of its own, drawn by the browser process of the webview and taller than the card,
  so pressing an entry fell outside the card's edges: the mouse hook read it as a click on
  the program behind Glossy and dismissed the card mid-choice. A click now also belongs to
  the card when the window under it hangs off one of the card's windows or is drawn by the
  process that draws the card, so the entries are pressed normally and only a click that
  really lands elsewhere closes the card. `platform::desktop::owns_point` walks the owner
  chain from the top level window down, `child_process_id` finds the browser process behind
  the card, and `popup::owns_point` remembers both when the card is placed.

## [1.1.2] - 2026-09-22

The bottom line of the card and the order of the settings window: the card says which
engine translated it and can change engines in place, and the settings finally group each
switch with the thing it changes. The local model is gone, so nothing has to be installed
to translate.

### Added

- **The card names the engine that translated it, and can change it.** The line under
  the translation printed the literal word `cloud` for both 百度 and 有道, because it
  was reading the storage shape instead of the service. It names the service that
  answered now — Baidu, Youdao or Google — and the name is a button: pressing
  it opens the services in place inside the card with the current one ticked,
  and picking one saves the choice and translates the selection again. `cloud::answered_by`
  keeps the literal `cloud` out of what the card is told, `current_service` and
  `set_service` carry the choice back and forth, and Escape closes the list before it
  closes the card.

### Changed

- **The settings window shows the services as plain names.** The line under the
  dropdown that promised a service needed no setup and cost nothing is gone. The card's
  own line carries the same names, so the two places a service is chosen finally agree.
- **The name of the engine shares one line with the two read-aloud buttons.** It sat on
  the line above them and could wrap away from them on a narrow card. The footer is a
  single row now, and the name is given the room it needs and kept to one line
  rather than wrapped, so the buttons keep their place at the end of the line.
- **The settings window groups each switch with the thing it changes.** The panels now
  run from the master switch through triggering, languages, the fallback service and the
  card's options to reading, units, history, the settings file and updates, and the two
  switches that describe the card — showing the original sentence and the compact layout
  — sit with the rest of the card's options instead of in the master and reading panels.
  Starting Glossy with the system is its own group: it sat among the switches the master
  switch dims while translation is off, which left it unreachable from a paused app.

### Removed

- **The local model is no longer an option.** The entry that translated through Ollama,
  or any other OpenAI compatible server on the same machine, is gone from the dropdown,
  from the fallback list and from the card's own list, along with the address and model
  fields it needed and the two settings keys behind them. Nothing has to be installed to
  translate: the two engines that go through Glossy's server and the free Google
  endpoint are the whole list. A settings file that still names the local model reads as
  the built-in engine and is rewritten on the next save.

## [1.1.1] - 2026-09-21

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

## [1.1.0] - 2026-09-21

### Added

- **The word card shows how the word behaves.** A word entry now carries its
  inflections (plural, third person, participles, past, comparative, superlative),
  the words that mean roughly the same, and — when **Reading** is on — the sentence
  the selection was taken from next to a translation of that sentence. The Rust side
  does the work: `morphology.rs` derives the forms, `translate/dictionary.rs` and
  `translate/mod.rs` collect and de-duplicate the alternatives, and `context.rs`
  looks the sentence up.

- **Sentence by sentence.** A paragraph whose translation divides differently than
  the original is no longer a wall of text: with **Sentence by sentence** on, the
  card lists the original and the translation one aligned row at a time. A
  translation that comes back as a single row is left as the plain paragraph, since
  a one-row table would only repeat the card.

- **Read it out loud.** Every card carries a pronunciation button for the original
  and one for the translation. The text is spoken through Windows SAPI at the rate
  chosen in the settings window (**Speaking rate**: Slow / Normal / Fast / Very
  fast), and pressing the button again stops it, as does closing the card.

- **A fallback order you can set.** When the chosen service fails, Glossy now walks
  a list of the other services instead of giving up. The settings window shows that
  list under **Fallback**: the service chosen above is always tried first and cannot
  be moved, the rest can be reordered with the arrows, and the whole thing can be
  turned off. The card names the service that answered when it was not the one
  asked for.

- **A compact card.** **Compact popup** keeps the translation, the phonetic symbols
  and the meanings, and leaves out the example, the inflections, the synonyms, the
  context sentence, the sentence-by-sentence view and the unit conversions — for a
  popup that is meant to be read at a glance.

- **Reading, and how much of it.** The settings window has a **Reading** panel that
  decides what a word card is allowed to show: **Word in its sentence** (the context
  lookup), **Sentence by sentence** (the aligned view) and **Compact popup**, plus
  the **Speaking rate** of the pronunciation buttons.

- **The selection is cleaned up before it is sent.** `text.rs` reduces what
  Ctrl+C handed over to the sentences the author wrote: soft hyphens and zero-width
  spaces, terminal escape sequences, line breaks the window width inserted, runs of
  non-breaking spaces and stray control characters are removed, so the provider is
  not asked to translate the layout of the page.

### Changed

- **Exporting the settings file no longer asks about keys.** With no field left
  anywhere that takes an API key, there is nothing an export could carry: the
  checkbox, its warning and the confirmation dialog are gone, and
  `export_settings` has no `include_credentials` argument any more. The file holds
  the choices and nothing secret.

- **Four translation services, and no key field anywhere.** The dropdown in the
  settings window now holds exactly four entries — **Baidu Translate** and **Youdao
  Translate** (the project's server, each naming the upstream it should use, so
  there is nothing to fill in), **Local translation · Ollama** (any
  OpenAI-compatible endpoint on this machine) and **Google** (the free public
  endpoint) — and the **Extensions · my own API** panel is gone, along with the
  Zhipu, DeepL and OpenAI entries it served. A new install starts on Baidu
  Translate. Nothing about the stored shape changed: `channel` + `cloudProvider` +
  `cloudVendor` / `provider` are written exactly as before, so a settings file from
  an older build keeps working — a `provider` this build no longer offers reads as
  the built-in Baidu entry and is replaced the next time the file is written, and a
  `cloudVendor` that is not `youdao` reads as Baidu. README and ROADMAP were updated
  in all three languages to match.

- **The original line in the card is capped at four lines.** A long selection used
  to push the translation down and out of view; the source block now scrolls inside
  itself once it would grow past four lines, in the floating popup and in the card
  of the settings window alike. The translation is what the card is read for, so it
  keeps its place.

### Fixed

- The confirmation that appears at the bottom of the settings window was invisible
  in dark mode while the window sat on the Mica backdrop: the toast drew its
  background from `--fg` and its text from `--bg`, and Mica remaps `--bg` to a 6 %
  white, so dark-mode text landed on a dark surface. The toast has its own
  `--g-toast-surface` / `--g-toast-text` / `--g-toast-outline` tokens now, and
  dialog boxes switched to a solid surface with the dialog shadow, so both read the
  same in either theme and under any backdrop.

## [1.0.2] - 2026-09-20

### Added

- **Baidu and Youdao as channels of their own, still with nothing to set up.**
  The dropdown now lists **Baidu Translate** and **Youdao Translate** next to
  Glossy's own server. Both are that same server — no key, no field to fill in —
  and they only say which upstream it should translate with, so the vendor that
  answers is visible in the list without anyone handling credentials. The server
  answers with the named upstream first (the request gained a `vendor` field) and
  still falls back to the others when it cannot, and the result card names whichever
  one answered. The settings file stores the choice as `cloudVendor` (`baidu`,
  `youdao`, or empty for "let the server decide"), the server keeps its
  `/v1/health` list of the upstreams a deployment actually has, and
  `server/src/youdao.js` implements the Youdao Zhiyun upstream with its v3
  signature.

- **Translation service, one list.** The settings window no longer asks whether
  results come from a cloud or from your own account; it asks which service
  translates, and the same dropdown holds all of them: Glossy's own server, which
  holds the provider credentials so no key of yours is involved; a model running on
  this machine over any OpenAI-compatible endpoint (Ollama's
  `http://127.0.0.1:11434/v1` by default, model `qwen2.5:7b`), which keeps the text
  on the computer and costs nothing; the free public Google endpoint, which needs no
  key; and then the vendors that take the user's own key — Baidu, Zhipu, DeepL and
  OpenAI. A new settings file starts on Glossy's server, and a file written before
  this release keeps the provider it already had, so nothing has to be re-entered.
  The credentials of the last group moved out of that panel into **Extensions · my
  own API**, a panel of its own at the bottom of the settings window, where they sit
  dimmed until one of those services is chosen. Under the stored shape nothing
  changed — Glossy's server and the local model are still the cloud channel, the
  rest still use the API channel — so an older build reads the file as it always
  did. The server gained a second upstream to go with it: it now translates through
  an LLM account when one is configured and falls back to the Baidu credentials it
  had before, so a deployment works either way.

- **The translation server's address is no longer a field.** Glossy's own service
  is the one that needs nothing set up, and a text box for the address only gave
  people a way to break it — a wrong or stale address left the app unable to reach
  any server at all, which is what the settings file of an upgraded install could
  hold. The address is now part of the build: whichever one the app was compiled
  with wins, a leftover value in an older settings file is ignored, and the window
  keeps showing what is left of today's allowance with its `Check again` button.
  Running your own deployment is a one-line change of `DEFAULT_ENDPOINT` in
  `src-tauri/src/translate/cloud.rs`; a build made without one still honors the
  setting.

### Fixed

- A second launch of Glossy now shows the same starting hint in the corner of the
  screen that a start of its own would, instead of an unstyled Windows dialog. The
  running instance is told through a named event and answers by opening the settings
  window when it is already on screen, or by showing the hint when it is not; the
  dialog is left only for the case where nothing answers at all.

## [1.0.1] - 2026-09-20

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

## [1.0.0] - 2026-09-20

The first stable release: Glossy now ships with a server of its own, so it can be
installed and used without anyone having to obtain an API key first.

### Added

- A `cloud` translation provider, **Glossy Cloud**. It talks to a server
  deployed from the new `server/` directory instead of to a vendor, so there is
  nothing to fill in: the account behind the server pays for the translation and
  the keys never reach the app. The settings window asks that server how much of
  today's allowance is left and shows the answer under the provider row, with
  **Check again** next to it; a server that cannot be reached says so in the same
  line instead of blocking the rest of the window. The address of the server
  lives in the new `cloudEndpoint` setting, is validated for an `https://` URL,
  and is written to the settings file as plain text because it is not a secret; a
  build made from this source already carries the address of the deployment the
  author runs, so the provider works as soon as it is selected.
- `server/`, a translation proxy that keeps the provider credentials on the
  machine that already holds them and hands each client a daily character
  allowance instead. It runs unchanged on two hosts — a Cloudflare Worker with a
  Durable Object for exact counting, and a Node 18 runtime for Tencent Cloud SCF
  Web 函数 — and carries its own README with the deployment steps for both,
  `npm test` coverage for the quota rules, the signature the providers expect and
  the IP parsing, plus a script that builds the SCF zip. The allowance is checked
  per installation, per caller address and globally, requests are capped per
  minute and per request, and the caller address is read from the end of
  `X-Forwarded-For` minus the two hops a function URL gateway appends, so a
  client cannot spoof its way into a fresh bucket. Baidu answers a burst with its
  per-second throttling code; that answer, and Baidu's own hiccups, are retried
  twice before the app is told anything.
- An install id (`cloudId` in the settings file), generated once so the server
  counts one device's characters rather than one launch's, and rewritten when a
  settings file written before the cloud provider existed has none.

### Changed

- A fresh install starts on the `cloud` provider instead of `google`, so the first
  selection translates without anything being configured or obtained. Existing
  settings keep the provider they were saved with.
- The new Glossy mark is the icon of the application, of the installer and of the
  tray, and it replaces the drawn `G` at the top of the settings window and on the
  start card. `assets/icon.png` is the source; `npm.cmd run icon` rebuilds the
  whole icon set, and the windows show the copy in `src/images/logo.png`.

### Fixed

- The language selectors in the popup card are readable in dark mode. Windows
  paints a `select` **and its native option list** with the background of the
  control, so the translucent fill the card has used left the language names
  sitting on the desktop behind the popup; both the popup selectors and the
  settings window's now use an opaque surface, carry a visible border and grow to
  a size that reads as a combo box.

## [0.3.3] - 2026-09-20

### Added

- The translate box that the settings window already carried now sits at the top
  of the window, above the enable switch, so the app can be used without
  selecting text anywhere: it gained **Paste** (reads the Windows clipboard
  through a new `read_clipboard` command), **Clear**, Ctrl+Enter to translate the
  whole text, and a pasted text is translated as soon as it lands. A new
  **Units** switch on the same heading turns unit conversion on and off from
  there — it is bound to the existing setting below, so either switch moves both
  and the visible card is redrawn with or without its conversions.

## [0.3.2] - 2026-09-20

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

## [0.3.1] - 2026-09-20

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

## [0.3.0] - 2026-09-19

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

## [0.2.1] - 2026-09-19

### Fixed

- Word lookups no longer leave the card empty for half a minute. The translation
  is rendered as soon as the provider answers - 266-679 ms in place of the
  24-28 s a lookup used to take - and the dictionary details follow on their own
  through the new `word_details` command, merging into the card that is already
  on screen instead of holding it back.
- Phonetic symbols, parts of speech, definitions and the example sentence are
  looked up by two sources at once, and the free endpoint that carries the
  example gets a short budget of its own: one source timing out no longer
  discards what the other one found. The dictionary timeout went from 4 s to 8 s,
  which is what the endpoint's variable response time (0.75-20 s) needs.
- The history panel refreshes itself. A translation emits `glossy://history`, so
  the list in the settings window is current without restarting the app, and the
  details that arrive late are written back to the stored entry.
- The history search box follows the shared input rule - same height, radius,
  background and padding as every other field - instead of keeping the browser
  default look.

## [0.2.0] - 2026-09-18

### Added

- The popup can be pinned. The thumbtack in its header keeps the card open
  through clicks elsewhere on the desktop and switches the auto-close timer off
  until the pin is released or the card is closed.
- Historical translations. The settings window keeps the last translations (the
  cap is configurable; `Off` clears and disables the list) in `history.json`,
  with a search box that filters both the original and the translation, a copy
  button, a remove button, and a click that shows the stored card again without
  asking the provider a second time.
- Glossy can start with Windows. The new toggle writes a `--autostart` entry to
  the login items; a start that came from Windows is silent - no console window,
  no "Glossy is running" card - and leaves the tray icon in place.
- Settings can be exported to and imported from a JSON file. The export writes
  `Documents\glossy-settings.json` and only includes the API keys when the
  corresponding box is ticked and the confirmation is accepted; an import
  validates every field through `sanitized()` and protects the keys it brings
  with DPAPI before they reach the disk.
- The frontend has a test suite. `node --test` now covers `i18n.js` and
  `render.js` (71 cases, no browser needed) and runs in CI next to the Rust
  checks. The README also gained a 17-item manual regression checklist for the
  behaviour the tests cannot reach.
- An update check, through `tauri-plugin-updater`. The settings window gained an
  **Updates** section: a toggle that asks GitHub for a new release on every start
  (off by default), a **Check now** button, and a **Download and restart** button
  that installs what it finds. This build has no update signing key yet, so it
  says so instead of pretending, and the section stays inert until
  `tauri.conf.json` carries a real public key.
- The version number has a single source. `tauri.conf.json` carries the number,
  `scripts/version.ps1` writes it to the five other places that need it
  (`package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`) and
  `scripts/release.ps1` refuses to build when they disagree. `version.ps1 -Check`
  runs in CI, so a forgotten bump fails the build instead of shipping a mislabelled
  installer.
- Continuous integration: `.github/workflows/ci.yml` runs the version check,
  `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` on a Windows
  runner for every push and pull request.
- Glossy now lives in the notification area. It keeps watching for selections
  after its window is closed, and the tray icon brings the window back. Its menu
  has **Open Glossy** and **Quit**; use **Quit** to stop Glossy completely.
- A silent start now announces itself with a small card in the bottom right
  corner: *Glossy is running in the background*. It goes away after a few
  seconds, and clicking it opens the settings window.

### Changed

- The README documents building from source: the required Node and Rust (GNU
  toolchain) versions, the `rustup` command that installs them, where the installer
  and the standalone binary land, and the checks to run before pushing.
- The settings window opens on the very first launch only. Every later launch
  starts silently in the notification area, so the only way back to the settings
  is the tray icon. Delete `settings.json` to get the window back on the next
  start.
- Starting Glossy no longer opens a console window. The executable is linked as
  a Windows GUI application in every build, and diagnostics are written to the
  console of the terminal that started it, if there is one.

### Fixed

- The phonetic row of a word card is no longer drawn for a whitespace-only
  phonetic, which used to leave an empty line above the definition.
- Language codes are matched regardless of case and region spelling, so `zh-cn`,
  `ZH` and `zh` are named instead of being printed as raw codes.
- Only one Glossy can run at a time. A second launch used to install a second
  mouse hook and a second popup; it now reports that Glossy is already running
  in the notification area and exits.
- Reading a selection no longer destroys what the clipboard held: the copy that
  lifts the selection out of the foreground application replaces the whole
  clipboard, and an image, a file list or rich text used to be lost for good.
  Every published format is now captured first and written back afterwards.
- Dragging the popup by its header keeps its click area in step with the window.
  The stale rectangle made clicks on the moved popup close it, and the release
  that followed triggered a second translation.
- Triple and quadruple clicks translate once. Every click of a chain used to
  report a double click, so selecting a paragraph sent two requests; the chain is
  now collapsed into a single request for the text the last click selected.
- Short selections that end like a sentence are now translated as prose instead
  of being sent to the dictionary. "Nice to meet you." and "今天天气很好。" used to
  be judged as words because the search for punctuation skipped the last
  character.
- Google definitions keep the text that follows a less-than sign. Stripping the
  markup of a definition ended at the first `<` it could not match, so `if x < 5`
  was shown as `if x`.
- Translations show the language in words instead of the provider's own code.
  Baidu and DeepL report their own codes — `jp`, `cht`, `auto` — and the popup
  printed those raw; every provider code is now mapped onto the shared set before
  the result is displayed, and a target language that arrives in a provider's
  spelling is normalised when the settings are saved.
- A settings change is no longer lost when the window is closed right after it.
  Closing the window used to cancel the save that was still waiting for its
  moment, so the last edit was gone; the pending change is now written out before
  the window goes away.
- The shortcut row shows the state of the shortcut that was just saved. The
  window used to read the status before the backend had re-registered the key, so
  a shortcut that worked was reported as unavailable until the next refresh.
- The popup card is measured against the monitor it lands on. The height limit
  came from the monitor the window was still on, so a selection made on a screen
  with a different scale factor was cut short; the backend now reports the usable
  height of the monitor under the cursor and the card is re-measured after the
  window moves there.
- Pressing the global shortcut translates the text that is selected right now.
  It used to read the clipboard after copying, which restored the previous
  content, so an older copy was translated instead of the live selection.
- The global shortcut ignores selections made inside Glossy's own windows and in
  the applications on the ignore list, exactly like the mouse triggers do.
- Writing to the clipboard no longer leaks the memory block it allocates when a
  step of the copy fails.
- One unreadable setting no longer resets all the others. A single field that
  could not be read made the whole file fall back to the defaults; the settings
  that can be read are now kept, and the field that could not be is reported in
  the console.
- The floating popup keeps its distance on a scaled display. The gap below the
  cursor, the margin to the screen edge and the position of a dragged popup were
  measured in physical pixels, so on a display set to 125 % or 150 % the popup
  drifted away from the cursor and could leave the screen. The start hint now
  keeps its distance from the screen corner there as well.

### Security

- Saved API keys are encrypted with Windows DPAPI (`CryptProtectData`) and tied
  to the Windows login that entered them, so `settings.json` no longer carries
  them in clear text. A file written by an older version is protected the first
  time Glossy starts, and a key that belongs to another login is dropped instead
  of being sent to the provider.

## [0.1.0] - 2026-09-17

First public release.

### Added

- Selection capture based on a low-level mouse hook: the translation is triggered
  when the mouse button is released after dragging across text, or on double click.
  Selections shorter than the configured minimum length are ignored, and a drag that
  starts or ends on the popup itself never triggers a translation.
- Floating popup window below the cursor: non-activating, always on top, draggable
  by its header, clamped to the work area of the monitor under the cursor and flipped
  above the cursor when there is no room below.
- Word card with phonetic symbols, part of speech, definitions and one example;
  sentence mode renders a fluent translation instead.
- Language bar in the popup: pick either side of the pair to translate again, or press
  `⇄` to translate the result back. The pair resets for every new selection and the
  overrides are never persisted.
- Five translation providers: Google (free, no key), Baidu, Zhipu GLM, DeepL and OpenAI.
- Per-provider credentials, so switching back to an already configured provider just works.
- Settings window with a master toggle, trigger options, minimum selection length,
  global hotkey, per-program ignore list, theme, popup width, text size, opacity and
  auto-close, plus an interface language switch (`Follow Windows`, `简体中文`, `English`).
- Translate-in-place card in the settings window: type or paste text, select part of it
  (or press *Translate*) and the result renders below in the same card the popup uses.
- Global hotkey (default `Ctrl+Alt+C`) that translates the clipboard content, and a
  setting to restore the previous clipboard content after reading a selection.

[Unreleased]: https://github.com/SpencerZXWu/Glossy/compare/v1.2.1...HEAD
[1.2.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.2...v1.2.0
[1.1.2]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.2...v1.1.0
[1.0.2]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/SpencerZXWu/Glossy/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.3...v1.0.0
[0.3.3]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/SpencerZXWu/Glossy/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.3.0
[0.2.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.2.0
[0.1.0]: https://github.com/SpencerZXWu/Glossy/releases/tag/v0.1.0
