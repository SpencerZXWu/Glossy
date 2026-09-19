# Glossy roadmap

Planned work, grouped into releases. Every release listed here maps to a GitHub
milestone of the same name; the acceptance criteria are what has to be true before
the milestone is closed and the tag is pushed.

## Where the project stands

| Area | State |
| --- | --- |
| Version | `0.3.0`. `src-tauri/tauri.conf.json` is authoritative; `scripts/version.ps1` keeps the five other locations in step and CI fails when one drifts |
| Size | ~12,400 lines: ~7,100 Rust, ~4,300 frontend (plain HTML/CSS/JS) and ~970 frontend test lines, comments included |
| Tests | 128 Rust tests and 81 frontend tests (`node --test`); `cargo fmt`, `cargo clippy`, `cargo test` and the frontend suite run in CI on `windows-latest` |
| Platform | Windows only — no `cfg(target_os)` gating, the `windows` crate is used unconditionally |
| Distribution | NSIS installer only; no code signing, a self-update skeleton that stays inert until a signing key pair exists, optional start with Windows |
| Repository | MIT licensed, changelog and roadmap in place, `v0.1.0` tagged and released with an installer served from GitHub Releases; `v0.2.1` staged in `release/v0.2.1/`, `v0.3.0` in `release/v0.3.0/` |

No defect is carried into the plan below. The last one — API keys sitting in
`%APPDATA%\com.glossy.translator\settings.json` as readable text — is fixed by the
first item of v0.2.0, which is already in the tree.

## Versioning policy

Semantic Versioning, with explicit meaning for the `0.x` range:

- `0.1.x` — patches. Bug fixes only: no new settings, no changes to the settings file format.
- `0.x.0` (x ≥ 2) — feature releases. May add settings; `Settings` is
  `#[serde(default)]`, so files written by older versions keep loading.
- `1.0.0` — the freeze. The settings JSON format and the IPC command names stop
  changing; later additions go through a migration function.
- Every `X.Y.0` gets a GitHub milestone, and every release is a `vX.Y.Z` tag with the
  NSIS installer attached.

## v0.1.1 — Open-source ready (patch)

Goal: a stranger can clone the repository, build it and trust the result.

| Work item | Acceptance criteria | State |
| --- | --- | --- |
| Add `LICENSE` | File present at the root, referenced from the README | done |
| Single-source the version | Only `tauri.conf.json` carries the number; `scripts/version.ps1` writes the other five locations, and CI fails when one of them disagrees with it | done |
| Add `CHANGELOG.md` | Keep a Changelog format, with a retrospective `0.1.0` section | done |
| CI workflow | `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` on a Windows runner for every push and pull request | done, in `.github/workflows/ci.yml` |
| Build documentation | README covers the required Rust (GNU toolchain) and Node versions, `npm run dev`, `npm run build` and where the installer lands | done |

All five items are in the tree; the release is waiting for its tag. The work needed
to get `clippy` and `rustfmt` to pass cleanly — a derived `Default` for the three
enums in `settings.rs`, one unreachable branch in `default_target_lang`, and a
`cargo fmt` pass over ten files — is behaviour-preserving.

Estimated effort: 0.5–1 day.

## v0.2.0 — Daily driver

Goal: from "it runs" to "I leave it running".

| Work item | Details | State |
| --- | --- | --- |
| Encrypt stored credentials | API keys are protected with DPAPI (`CryptProtectData`, current-user scope) before writing. A settings file written by an older version is migrated and rewritten on the first launch of this build, and a key that belongs to another Windows login is dropped instead of being sent on | done, in `src-tauri/src/secrets.rs` |
| Autostart | `tauri-plugin-autostart` backs a settings toggle. The single-instance guard already ships: a second launch says so in the notification area instead of installing a second mouse hook | done, in `src-tauri/src/autostart.rs` |
| Auto-update skeleton | `tauri-plugin-updater` with a signing key, fed from GitHub releases; can be switched off in the settings | done, in `src-tauri/src/updater.rs`; waiting for a key pair |
| Translation history | A list in the settings window (in memory, optional persistence, configurable cap) with search, reopen-in-card and copy | done, in `src-tauri/src/history.rs` |
| Pin the popup | A button in the header that keeps the card open through clicks outside and disables auto-close until it is closed | done |
| Import and export settings | JSON file out and in, validating through `sanitized()`. Now that credentials are protected, an export has to confirm before it includes them, and an import has to protect what it brings | done, in `export_settings` / `import_settings` |
| Frontend tests | `node --test` coverage for the logic behind `i18n.js`, `render.js` and the classification in `classify.rs`; at least 25 cases | done, 71 cases in `tests/`, wired into CI |
| Manual regression list | A pre-release checklist in the README: multiple monitors, 150% scaling, both colour schemes, every provider, the ignore list | done, 17 items in the README |

All eight items are in the tree. The update check works end to end except for the
signing key: `tauri.conf.json` carries a placeholder public key, so the check
reports that this build cannot update itself, and the uploader needs a real key
pair before a release can be published as an update.

Estimated effort: 3–5 days.

## v0.3.0 — One visual language, and the reading card

Goal: the three windows look like one product, and a translated passage carries
the numbers a reader of the target language needs.

| Work item | Details | State |
| --- | --- | --- |
| Design tokens | `src/styles/tokens.css` holds every colour, radius, shadow, font size and duration; the WinUI palette; `app.css`, `popup.css` and `notice.css` alias the tokens instead of repeating literals | done |
| Theme resolution | `js/theme.js` resolves `system`/`light`/`dark` in one place for all three windows | done |
| Window material | Mica behind the settings window (`window-vibrancy`), a theme-matched title bar colour, and a `SurfaceInfo.ready` handshake so the stylesheet only turns transparent once the backdrop is confirmed | done, in `src-tauri/src/surface.rs` |
| Unit and currency conversion | A measurement or amount that the target language does not use is shown converted, with its rate, under the card: length, mass, volume, speed, area, temperature, and money through a live rate table | done, in `src-tauri/src/units/` |
| Conversion switch | `Convert units and currency`, on by default; off means no conversion and no rate request | done |
| Word-card placeholder | The card says the dictionary is being looked up instead of changing under the reader, and the lookups share a short budget | done |

Remaining visual work — the navigation and settings-panel rework, the popup card
rebuild, icon and motion polish, and the accessibility pass — is listed in v0.4.0.
It is behaviour-preserving and ships as its own step.

Estimated effort: 3–4 days.

## v0.4.0 — Translation quality and the rest of the appearance work

Goal: make the reading use case actually good.

| Work item | Details | State |
| --- | --- | --- |
| Settings window rework | A left navigation rail in place of one long scroll, a search box that filters the options, and grouped panels with the WinUI control look | planned |
| Popup rebuild | The card rebuilt on the token layer: header actions, original/translation hierarchy, dictionary and conversion blocks, and a compact mode | partly, the conversion block is in the tree |
| Icons, motion and accessibility | One icon set, transitions on the popup's appearance and dismissal, full keyboard operability, focus rings, high-contrast colours | planned |
| Provider fallback | When the active provider fails or rate-limits (Google answering `429`), fall back through a configurable order and name the provider that answered in the card footer | planned |
| More providers | At least two more free tiers (Youdao, Tencent, Volcengine, or a self-hosted LibreTranslate), plus a custom base URL per provider | planned |
| Richer word cards | Inflections, synonyms and merged definitions from several sources; an optional sentence-by-sentence view that pairs the original with the translation | planned |
| Word to sentence | Optionally translate the sentence the selected word sits in, alongside the word itself | planned |
| Text to speech | A pronunciation button for the original and the translation, via Windows SAPI or edge-tts | planned |
| Selection robustness | A dedicated pass over rich text, browsers, Office and terminals, where `stripTags` is currently a simple cleanup; backed by a set of real-world fixtures | planned |

Estimated effort: 6–10 days.

## v0.5.0 — Cross-platform and distribution

Goal: leave Windows behind and stop being flagged by SmartScreen.

| Work item | Details |
| --- | --- |
| Platform abstraction | Move the direct Win32 calls in `input.rs`, `platform.rs`, `clipboard.rs` and `hotkey.rs` behind `platform/windows.rs` with `cfg(target_os)` dispatch. This is a prerequisite — there is no gating today |
| macOS | Selection capture through the Accessibility API with a permission onboarding flow; the popup becomes an `NSPanel` that does not take focus; signing and notarisation |
| Linux | X11 first, where the `PRIMARY` selection maps naturally onto select-to-translate. On Wayland the global hook is not available, so it is documented as an unsupported combination rather than silently failing |
| Code signing | Azure Trusted Signing or an EV certificate, so a fresh install no longer shows a SmartScreen warning |
| Packaging matrix | NSIS, MSI and a portable zip; x64 and ARM64 |
| Content Security Policy | `tauri.conf.json` sets `"csp": null` today; replace it with an explicit whitelist (the frontend loads no remote scripts, so this is cheap) |

Estimated effort: 8–15 days.

## v1.0.0 — Stable release

Goal: turn a personal tool into something that can be promised.

| Work item | Details |
| --- | --- |
| Format freeze | Settings JSON and IPC command names frozen, with a migration function and a fallback that backs up the original file and starts from defaults |
| Performance budget | Idle CPU below 0.5%, memory below 80 MB, selection to popup under 150 ms, and no handle or GDI leak over long runs — the hook and the clipboard are the places to watch |
| Stability | Crash recovery and optional anonymous error reporting, off by default and asked about on first run |
| Accessibility | Full keyboard operability, a high-contrast theme, correct focus order and aria labels |
| Documentation | Multi-language README, FAQ, and a note on provider quotas and terms of use |

Estimated effort: 5–10 days.

## Priority

When time is short, the order of return on effort is: **v0.1.1 → the remaining
v0.2.0 work → provider fallback in v0.4.0 → v0.5.0**.
Cross-platform support is the only item large enough that it may never be finished,
so it is deliberately scheduled last; decide on it after v0.4.0 rather than investing
in it early.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| The Google endpoint is unofficial | It can start rate-limiting or change protocol at any time, and its terms of use are unclear | A retry across two clients already lives in `google.rs`; provider fallback in v0.4.0 is the real fix |
| A currency rate service changes shape or goes down | An amount in the card loses its conversion | Two independent sources (exchangerate-api.com, and the ECB through `frankfurter.app`), a six-hour memory and disk cache, a stale table that stays usable for seven days and is labelled as such, and a two-minute quiet period after both fail |
| Antivirus flags the low-level mouse hook | Installs and runs get blocked | Code signing in v0.4.0, plus a README section explaining what the hook does and why |
| macOS and Linux permission models | The port costs more than expected | Kept as its own release, X11 first, Wayland explicitly unsupported |
| Credential leakage | A readable API key on disk | Fixed for v0.2.0: keys are encrypted with DPAPI and unreadable outside the Windows login that entered them |
| Hand-built releases | Installers cannot be reproduced | Version check, format, lints and tests run in CI since v0.1.1; the installer itself is still built locally by `scripts/release.ps1`, and a tagged release workflow is the next step |

## Release process

1. Set the version in its single source and update `CHANGELOG.md`:
   `powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set X.Y.Z` writes all
   six locations, and `-Check` lists any that drifted apart.
2. `cargo test`, the frontend tests and CI all pass.
3. `powershell -ExecutionPolicy Bypass -File scripts/release.ps1` builds the installer
   and stages it in `release/vX.Y.Z/` with `SHA256SUMS.txt` and the text for the
   release description.
4. Tag `vX.Y.Z` on `main` and push the tag.
5. Publish a GitHub release at that tag: paste `release/vX.Y.Z/RELEASE_NOTES.md` into
   the description and attach the installer together with the checksum.
6. From v0.2.0 on, the release also serves as the auto-update feed.

## What is explicitly out of scope

- Browser extensions and mobile apps. Glossy is a desktop utility, and the selection
  capture it relies on has no equivalent there.
- OCR and screenshot translation. It shares the popup, but not the input path, and it
  would double the surface area.
- Bundling translation engines or models locally. It conflicts with the "lightweight"
  goal; users who need it can point a provider at their own endpoint.
