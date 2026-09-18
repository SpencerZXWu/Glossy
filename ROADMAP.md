# Glossy roadmap

Planned work, grouped into releases. Every release listed here maps to a GitHub
milestone of the same name; the acceptance criteria are what has to be true before
the milestone is closed and the tag is pushed.

## Where the project stands

| Area | State |
| --- | --- |
| Version | `0.1.0`. `src-tauri/tauri.conf.json` is authoritative; `scripts/version.ps1` keeps the five other locations in step and CI fails when one drifts |
| Size | ~7,600 lines: ~4,600 Rust, ~3,000 frontend (plain HTML/CSS/JS), tests and comments included |
| Tests | 87 Rust tests, no frontend tests; `cargo fmt`, `cargo clippy` and `cargo test` run in CI on `windows-latest` |
| Platform | Windows only — no `cfg(target_os)` gating, the `windows` crate is used unconditionally |
| Distribution | NSIS installer only; no code signing, no auto-update, no autostart |
| Repository | MIT licensed, changelog and roadmap in place, `v0.1.0` tagged and released with an installer served from GitHub Releases |

One item is carried into the plan below because it blocks other work:

1. **Credentials are stored in plaintext.** API keys live in
   `%APPDATA%\com.glossy.translator\settings.json` as-is. This is the one security
   defect in the project and is the highest priority fix.

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

| Work item | Details |
| --- | --- |
| Encrypt stored credentials | Protect API keys with DPAPI (`CryptProtectData`, current-user scope) before writing. A plaintext settings file is migrated and rewritten on first launch, and exports omit secrets unless confirmed |
| Autostart | `tauri-plugin-autostart` backs a settings toggle. The single-instance guard already ships: a second launch says so in the notification area instead of installing a second mouse hook |
| Auto-update skeleton | `tauri-plugin-updater` with a signing key, fed from GitHub releases; can be switched off in the settings |
| Translation history | A list in the settings window (in memory, optional persistence, configurable cap) with search, reopen-in-card and copy |
| Pin the popup | A button in the header that keeps the card open through clicks outside and disables auto-close until it is closed |
| Import and export settings | JSON file out (with an explicit confirmation when credentials are included) and in, validating through `sanitized()` |
| Frontend tests | `node --test` coverage for the logic behind `i18n.js`, `render.js` and the classification in `classify.rs`; at least 25 cases |
| Manual regression list | A pre-release checklist in the README: multiple monitors, 150% scaling, both colour schemes, every provider, the ignore list |

Estimated effort: 3–5 days.

## v0.3.0 — Translation quality

Goal: make the reading use case actually good.

| Work item | Details |
| --- | --- |
| Provider fallback | When the active provider fails or rate-limits (Google answering `429`), fall back through a configurable order and name the provider that answered in the card footer |
| More providers | At least two more free tiers (Youdao, Tencent, Volcengine, or a self-hosted LibreTranslate), plus a custom base URL per provider |
| Richer word cards | Inflections, synonyms and merged definitions from several sources; an optional sentence-by-sentence view that pairs the original with the translation |
| Word to sentence | Optionally translate the sentence the selected word sits in, alongside the word itself |
| Text to speech | A pronunciation button for the original and the translation, via Windows SAPI or edge-tts |
| Selection robustness | A dedicated pass over rich text, browsers, Office and terminals, where `stripTags` is currently a simple cleanup; backed by a set of real-world fixtures |

Estimated effort: 5–8 days.

## v0.4.0 — Cross-platform and distribution

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

When time is short, the order of return on effort is: **v0.1.1 → the credential
encryption and history work in v0.2.0 → provider fallback in v0.3.0 → v0.4.0**.
Cross-platform support is the only item large enough that it may never be finished,
so it is deliberately scheduled last; decide on it after v0.3.0 rather than investing
in it early.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| The Google endpoint is unofficial | It can start rate-limiting or change protocol at any time, and its terms of use are unclear | A retry across two clients already lives in `google.rs`; provider fallback in v0.3.0 is the real fix |
| Antivirus flags the low-level mouse hook | Installs and runs get blocked | Code signing in v0.4.0, plus a README section explaining what the hook does and why |
| macOS and Linux permission models | The port costs more than expected | Kept as its own release, X11 first, Wayland explicitly unsupported |
| Credential leakage | A readable API key on disk | DPAPI encryption in v0.2.0 |
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
