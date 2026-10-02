# Glossy compatibility statement

This file is the promise about what a new version may do to an
installation that already exists. It is written for the reader who has to decide
whether updating is safe, and for whoever changes the code next: it says what is
part of the contract and what a release is allowed to break.

The version makes the difference. `1.x` releases are additive — settings are only
added, never renamed or removed, and the IPC commands the windows call keep their
names and their arguments. `2.0.0` is the release that may break that contract, and
it is the only one so far that did.

## What `2.0.0` reads

Any settings file written from `1.3.0` onwards, and every earlier one as well: a
file that names no `formatVersion` is a file written before the number froze, and
it is treated as version `0` and migrated rather than set aside. There is no
setting a previous release could have written that this one refuses to load.

| Written by | `formatVersion` | What `2.0.0` does with it |
| --- | --- | --- |
| `1.3.0` and later | `1` | Migrated. The four fields that named the translation service are folded into one; everything else is read as it stands |
| Before `1.3.0` | absent | Read as version `0` and migrated the same way |
| `2.0.0` and later | `2` | Read as it stands |
| A newer release | `3` or higher | Set aside, not read. The file is copied to `settings.backup-<unix seconds>.json` beside it and the app starts from the defaults, so a newer build's choices are never silently rewritten into older ones |

A file that is not a JSON object at all — a hand edit that broke it, or half a file
after a crash — takes the same path: copied aside, then defaults. Nothing is
overwritten before it has been understood.

### The one deliberate break

Version `1.x` spelled the choice of translation service across four keys,
because the app used to offer modes it no longer has: a provider of your own with
your own key, and a relay at an address you typed in. Neither exists any more, and
`2.0.0` folds what remains into the single `service` key it writes.

| A `1.x` file said | `2.0.0` reads it as |
| --- | --- |
| `channel: "cloud"` with `cloudVendor: "baidu"` (or a value this build never offered) | `service: "cloud-baidu"` |
| `channel: "cloud"` with `cloudVendor: "youdao"` | `service: "cloud-youdao"` |
| `channel: "api"`, or no service fields at all | `service: "google"` |
| `channel: "offline"` | `service: "offline"` |

When such a file is written back, the keys nothing reads any more are dropped:
`channel`, `cloudProvider`, `cloudVendor`, `provider`, `cloudEndpoint`,
`credentials`, `apiKey` and `appId`. The last four are the break of substance:
**`2.0.0` cannot hold an API key of yours, because no service it offers asks for
one.** A key that is in the file is not disclosed, not migrated and not used — it
is simply gone the next time the settings are saved. Nothing else about a `1.x`
file changes: every other value is read as it stands, and a key this build does not
know is left alone rather than reset.

The export file follows the same shape. An export written by `1.x` imports into
`2.0.0` — it goes through the same migration — but what `2.0.0` writes is the
version 2 shape, which an older build will read as the defaults for the fields it
knows about. An export is not a format either side keeps compatible; the settings
file is.

## What will not change inside a major version

For the whole of `2.x`, and for the whole of `1.x` before it:

- **Settings are only added.** A key that exists keeps its name and its meaning, and
  the file keeps loading. Removing or renaming one is a major release, as above.
- **IPC command names and their arguments stay as they are.** A window from any
  `2.x` build and a backend from any other `2.x` build are not a supported pairing —
  the windows ship inside the binary — so what this protects is anything built
  against the names, and the ability to read an older build's source and find the
  same calls.
- **`formatVersion` only moves when `parse` cannot absorb the change on its own.**
  A new key does not move it; a renamed or removed one does.
- **`contract/contract.json` is the published list** of the format version, of every
  key the settings file holds and of every command the app answers to. A test
  compares it to the code and fails when the two disagree, so the list cannot drift
  away from what the app does without the build failing first.

## What a future major release may break

Only this, and it has to be a major release to do it:

- A settings key may be renamed, removed, or given a different meaning.
- An IPC command may be renamed, or lose an argument.
- The settings file may move to a format the previous one cannot read.

And whatever it breaks, it still has to do three things, because these are not part
of the contract — they are the reason a break is survivable:

1. **Migrate rather than drop.** Anything from `1.3.0` onwards is carried forward.
   `2.0.0` is the worked example: it removed four keys and lost nobody their
   language, their shortcuts, their history settings or their ignored programs.
2. **Set aside what it cannot read.** A file from a newer build, or a broken one, is
   copied next to the original before the defaults are used.
3. **Say so in the release notes.** A break that is not in the notes is a bug, however
   well the migration works.

## What no version will do

- Send anything anywhere that [`PRIVACY.md`](./PRIVACY.md) does not list.
- Weaken what is stored on this machine without saying so in the release notes.
- Read a settings file it does not understand and then write over it.
- Publish an installer that was not built from a tag in this repository, by
  [`.github/workflows/release.yml`](./.github/workflows/release.yml) or by
  [`scripts/release.ps1`](./scripts/release.ps1) on a machine holding the same
  sources.

## How to check any of this

- [`contract/contract.json`](./contract/contract.json) — the published list itself.
- `cargo test --lib` — includes the test that keeps the contract and the code in
  step, and the tests that read a `1.x` file and assert what it becomes.
- [`scripts/version.ps1`](./scripts/version.ps1) `-Check` — lists any place the
  version number has drifted apart.
- [`CHANGELOG.md`](./CHANGELOG.md) — what each release said it did.

---

# Glossy 兼容性声明 · 中文

本文件是有关新版本可以对你已有的安装做什么的承诺。它的读者有两类：需要判断"升级是否安全"的用户，
以及接下来修改代码的人——它说明什么属于契约、什么允许被某个版本打破。

版本号决定了差别。`1.x` 的版本是**只增不减**的：设置只新增，绝不重命名或删除，各个窗口调用的 IPC
命令保持名称与参数不变。`2.0.0` 是允许打破该契约的版本，也是到目前为止唯一打破过的版本。

## `2.0.0` 能读什么

从 `1.3.0` 起写出的任何设置文件，以及更早的每一个文件：一个没有写 `formatVersion` 的文件，是版本号
固定下来之前写出的文件，会被当作版本 `0` 进行迁移，而不是被搁置。以前任何版本写下的设置，本版本都
不会拒绝加载。

| 写出者 | `formatVersion` | `2.0.0` 如何处理 |
| --- | --- | --- |
| `1.3.0` 及以后 | `1` | 迁移。命名翻译服务的四个字段被合并为一个；其余内容按原样读取 |
| `1.3.0` 之前 | 缺失 | 按版本 `0` 读取，迁移方式相同 |
| `2.0.0` 及以后 | `2` | 按原样读取 |
| 更新的版本 | `3` 及以上 | 搁置，不读取。文件被复制到同目录下的 `settings.backup-<unix 秒>.json`，应用从默认值启动，因此新版本的选项绝不会被悄悄改写成旧版本的含义 |

完全不是 JSON 对象的文件——被手工编辑改坏的，或者崩溃后只剩一半的——走同一条路：先复制备份，再用
默认值。任何文件在被读懂之前都不会被覆盖。

### 唯一一处刻意打破的地方

`1.x` 把"用哪种翻译服务"分散在四个键里，因为当时的应用还提供它现在已经没有的模式：使用你自己密钥的
自备渠道，以及由你填入地址的中转服务。两者都不复存在，于是 `2.0.0` 把剩下的内容合并进它写出的唯一
一个 `service` 键。

| `1.x` 文件里写的 | `2.0.0` 读取为 |
| --- | --- |
| `channel: "cloud"` 加 `cloudVendor: "baidu"`（或本版本从未提供过的值） | `service: "cloud-baidu"` |
| `channel: "cloud"` 加 `cloudVendor: "youdao"` | `service: "cloud-youdao"` |
| `channel: "api"`，或者完全没有服务字段 | `service: "google"` |
| `channel: "offline"` | `service: "offline"` |

这样的文件被重新写出时，已经无人读取的键会被丢弃：`channel`、`cloudProvider`、`cloudVendor`、
`provider`、`cloudEndpoint`、`credentials`、`apiKey` 与 `appId`。后四个才是实质性的打破：**`2.0.0`
无法保存你的 API 密钥，因为它提供的服务都不需要密钥。** 文件中已有的密钥不会被泄露、不会被迁移、
也不会被使用——只是在下次保存设置时消失。`1.x` 文件的其余部分没有任何变化：其他每个值都按原样读取，
本版本不认识的键会被保留而不是被重置。

导出文件是同一套结构。`1.x` 写出的导出文件可以导入 `2.0.0`（它同样经过迁移），但 `2.0.0` 写出的是
版本 2 的结构，旧版本读它时，自己认识的字段会退回默认值。导出文件不是双方共同维护兼容的格式，设置
文件才是。

## 同一大版本内不会改变的东西

在整个 `2.x` 期间，以及此前的整个 `1.x` 期间：

- **设置只增不减。** 已存在的键保持名称与含义，文件继续可加载。删除或重命名一个键属于大版本才能做的
  事，如上所述。
- **IPC 命令的名称与参数保持不变。** 任意 `2.x` 的窗口搭配任意另一个 `2.x` 的后端并不是受支持的组合
  ——窗口随二进制一起发布——这里保护的是：任何按这些名称编写的东西，以及任何人去读旧版本源码时能找到
  同样的调用。
- **`formatVersion` 只在 `parse` 自身无法吸收该变化时才变动。** 新增一个键不会让它变动；重命名或删除
  一个键则会。
- **`contract/contract.json` 是公开发布的清单**：格式版本、设置文件中的每一个键、应用响应的每一条命令。
  有一个测试把它与代码对照，两者不一致时构建就会失败，因此这份清单无法在构建通过的前提下与应用的
  实际行为脱节。

## 未来的大版本可以打破什么

仅限以下内容，而且必须是某个大版本才能做：

- 设置键可以被重命名、删除，或被赋予不同的含义。
- IPC 命令可以被重命名，或减少一个参数。
- 设置文件可以迁移到一个上一个版本读不懂的结构。

而无论打破了什么，它仍然必须做三件事——这些不属于契约，而是"打破仍然可以承受"的原因：

1. **迁移而不是丢弃。** 从 `1.3.0` 起的所有内容都会被带过来。`2.0.0` 就是现成的范例：它删掉了四个键，
   却没有让任何人失去语言、快捷键、历史记录设置或忽略程序列表。
2. **读不懂的就先搁置。** 来自更新版本的文件，或者损坏的文件，会在使用默认值之前被复制到原文件旁边。
3. **在发布说明中写明。** 没有写进发布说明的打破就是 bug，无论迁移做得多好。

## 任何版本都不会做的事

- 向 [`PRIVACY.md`](./PRIVACY.md) 未列出的任何地方发送数据。
- 在不于发布说明中说明的情况下，削弱本机存储的保护程度。
- 读不懂一个设置文件却仍然覆盖它。
- 发布并非由本仓库中的某个 tag 构建出的安装包，无论是通过
  [`.github/workflows/release.yml`](./.github/workflows/release.yml)，还是在持有同样源码的机器上通过
  [`scripts/release.ps1`](./scripts/release.ps1) 构建的。

## 如何自行核实以上任何一条

- [`contract/contract.json`](./contract/contract.json) —— 公开发布的清单本身。
- `cargo test --lib` —— 包含让契约与代码保持一致的测试，以及读取 `1.x` 文件并断言其结果的测试。
- [`scripts/version.ps1`](./scripts/version.ps1) `-Check` —— 列出所有版本号已经漂移的地方。
- [`CHANGELOG.md`](./CHANGELOG.md) —— 每个版本自称做了什么。
