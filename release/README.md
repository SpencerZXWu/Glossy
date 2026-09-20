# Release artifacts

Staging area for the files that get uploaded to
[GitHub Releases](https://github.com/SpencerZXWu/Glossy/releases). It is built by
`scripts/release.ps1`; nothing here is edited by hand except the two translations in
`RELEASE_NOTES.md`.

## Layout

```
release/
  v0.1.0/
    Glossy_0.1.0_x64-setup.exe   the NSIS installer — the release asset
    RELEASE_NOTES.md             paste into the release description (EN · 中文 · ES)
    SHA256SUMS.txt               checksum of the installer
```

Installers are **git-ignored** (`release/**/*.exe` and friends), so one 1.8 MB binary
per version never accumulates in the repository history — the uploaded asset is the
published one. The notes and checksums stay tracked so the description of every
release is versioned alongside the code; the two translations in `RELEASE_NOTES.md`
are the one thing here that is written by hand rather than produced by the script.

## Producing a release

```powershell
powershell -ExecutionPolicy Bypass -File scripts/release.ps1              # build + stage
powershell -ExecutionPolicy Bypass -File scripts/release.ps1 -SkipBuild   # re-stage the existing build
```

`-ExecutionPolicy Bypass` is required because PowerShell scripts are blocked by the
default execution policy on this machine — the same reason the README uses `npm.cmd`.

The script takes the version from `scripts/version.ps1 -Get` and stops a running Glossy
(the linker cannot replace `glossy.exe` while it is loaded), builds the NSIS bundle and
stages it under `release/v<version>/`. `RELEASE_NOTES.md` is written in three languages
unless the file is already there, so notes written by hand survive; `-ForceNotes`
regenerates them and resets the translations.

## Release notes in three languages

One `RELEASE_NOTES.md` carries English, Chinese and Spanish. A link line at the top
jumps to the anchor that sits directly above each language's block, so a reader opens
the release and switches language without leaving the page:

```markdown
[English](#en) · [中文](#zh-cn) · [Español](#es)

<a id="en"></a>

## English

### Changed

- ...from the matching CHANGELOG.md section...

<a id="zh-cn"></a>

## 中文

### 变更

- ...

<a id="es"></a>

## Español

### Cambios

- ...
```

`CHANGELOG.md` stays English-only, as Keep a Changelog expects. The script fills the
English block from it and leaves the other two as `TODO` placeholders; it warns while
any of those placeholders is still in the file, so an untranslated release is noticed
before it is published. Google Translate is a fine starting point for the two
translations, but read the result before shipping it. `RELEASE_NOTES.md` is UTF-8
without a BOM — GitHub renders the anchors and the `·` separators straight from it.

`scripts/release.ps1` itself must stay saved as UTF-8 **with** a BOM: Windows
PowerShell 5.1 decodes a BOM-less script as ANSI, which would replace the Chinese and
Spanish labels with mojibake in the generated notes, so the script refuses to run
without one.

## Uploading

1. Add the version's section to `CHANGELOG.md` and set the version with
   `powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set X.Y.Z` — that
   writes all six places at once. `scripts/version.ps1 -Check` lists any that drifted
   apart, and CI fails on a file that does not agree.
2. `git tag -a vX.Y.Z -m "Glossy vX.Y.Z"` and `git push origin vX.Y.Z`.
3. Create the release on GitHub for that tag — paste `RELEASE_NOTES.md` into the
   description, all three languages and the anchor links included.
4. Attach the installer and `SHA256SUMS.txt`. The installer has to be the one **this
   version staged**, whose file name carries the version, and `SHA256SUMS.txt` is
   generated from that same file — a mismatched pair means the wrong build is going
   out. v0.3.1 was published with v0.3.0's installer attached, which is how a release
   shipped without the fixes it described.

The GNU build links `WebView2Loader.dll` dynamically, so an installer that does not
carry it produces an app that dies on launch with "WebView2Loader.dll was not found".
`scripts/release.ps1` stops before staging when the DLL is not in `src-tauri/resources/`
or the generated `installer.nsi` does not install it.

Versions and their acceptance criteria are in [../ROADMAP.md](../ROADMAP.md).
