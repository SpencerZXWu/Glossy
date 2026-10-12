[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

### Added

- **The deployment's own report, in a window and on a command line.** A relay
  built from `server/` keeps the day's counters to hold the free allowance
  honest; `GET /v1/stats` is those same numbers read the other way round — how
  much of today is gone, whose it went to, how many screenshots the month has
  read, and what is left in total. It is answered from the counters themselves
  rather than from a second book, so it can never disagree with what an App is
  told, and it books no characters and moves nothing. It is off unless the
  deployment sets `ADMIN_TOKEN`, because the report names installations and what
  each of them spent; a long tail is rolled into a line of its own rather than
  dropped, so its rows always add up to the total beside them. Two things read
  it: `scripts/stats.ps1`, which keeps one row per day in a CSV — its token is
  stored with DPAPI, so neither the script nor the scheduled task holds it — and
  a page of the window's own behind developer mode, where the token is typed in
  and kept in memory for as long as the window is open.
- **A third engine behind the other two.** A deployment that has opened Tencent
  Cloud's machine translation can put it last in the relay's chain with
  `TENCENT_MT=1`, reusing the key pair it already reads screenshots with. It is
  not a service an App can pick: the account is the operator's, and the engine is
  there so that an answer keeps coming when the engines a reader asked for have
  nothing left. It stays out of the chain unless that switch is set, because
  reading pictures and translating text are separate products on that account.

### Changed

- **The allowance line says when it is one instance's.** A deployment of several
  instances with no shared store counts in each of them and answers with
  whichever copy took the call, so the same question twice can come back with two
  different numbers. The relay now says which of the two it is (`shared` on
  `/v1/quota`, `counters.shared` on `/v1/health`), and the window adds a word to
  the line instead of letting the number move under the reader.
- **The allowance is not asked for on every page change.** An answer stands for
  twenty seconds before another one is asked for, so walking through the window
  no longer means watching the same number redraw itself.

### Fixed

- **The shared-counter tests left their sockets open.** A link keeps a socket for
  as long as it exists, and a socket that is still open keeps the test process
  alive: that file never printed its summary and the whole suite read as a hang.
  The links are closed when the file is over.

<a id="zh-cn"></a>

## 中文

<!-- TODO: translate the English section above, then delete this comment. -->

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
