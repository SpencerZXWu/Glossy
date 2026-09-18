# Release artifacts

Staging area for the files that get uploaded to
[GitHub Releases](https://github.com/SpencerZXWu/Glossy/releases). Nothing here is
produced by hand — build it with `scripts/release.ps1`.

## Layout

```
release/
  v0.1.0/
    Glossy_0.1.0_x64-setup.exe   the NSIS installer — the release asset
    RELEASE_NOTES.md             paste into the release description
    SHA256SUMS.txt               checksum of the installer
```

Installers are **git-ignored** (`release/**/*.exe` and friends), so one 1.8 MB binary
per version never accumulates in the repository history — the uploaded asset is the
published one. The notes and checksums stay tracked so the description of every
release is versioned alongside the code.

## Producing a release

```powershell
powershell -ExecutionPolicy Bypass -File scripts/release.ps1              # build + stage
powershell -ExecutionPolicy Bypass -File scripts/release.ps1 -SkipBuild   # re-stage the existing build
```

`-ExecutionPolicy Bypass` is required because PowerShell scripts are blocked by the
default execution policy on this machine — the same reason the README uses `npm.cmd`.

The script takes the version from `scripts/version.ps1 -Get` and stops a running Glossy
(the linker cannot replace `glossy.exe` while it is loaded), builds the NSIS bundle and
stages it under `release/v<version>/`. `RELEASE_NOTES.md` is extracted from the
matching `CHANGELOG.md` section unless the file is already there, so notes written by
hand survive; `-ForceNotes` regenerates them.

## Uploading

1. Add the version's section to `CHANGELOG.md` and set the version with
   `powershell -ExecutionPolicy Bypass -File scripts/version.ps1 -Set X.Y.Z` — that
   writes all six places at once. `scripts/version.ps1 -Check` lists any that drifted
   apart, and CI fails on a file that does not agree.
2. `git tag -a vX.Y.Z -m "Glossy vX.Y.Z"` and `git push origin vX.Y.Z`.
3. Create the release on GitHub for that tag — paste `RELEASE_NOTES.md` into the description.
4. Attach the installer and `SHA256SUMS.txt`.

Versions and their acceptance criteria are in [../ROADMAP.md](../ROADMAP.md).
