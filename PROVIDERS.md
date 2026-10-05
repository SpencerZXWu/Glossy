# Glossy translation services

What each service behind the **Translation service** setting is, what it costs, the
terms it is used under, what the app does when it cannot answer, and where the text
goes. It is the companion to the research in the [README](./README.md), written for
the reader who wants to know what they are sending and to whom.

Every fact here is taken from this repository: the README, [PRIVACY.md](./PRIVACY.md),
[PRODUCT.md](./PRODUCT.md), the relay's own [server/README.md](./server/README.md),
[THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md), and the upstream pages those files
link to. Where the repository records no link to an upstream page, none is invented
here.

Last checked against this repository on 2026-10-05. The repository does not record the
date its provider research was done. Free quotas and terms change without notice, and
the numbers below are the ones the repository records rather than a live reading —
check the upstream page before relying on any of them.

## The services

The build offers six entries, stored in the settings file's single `service` field under
exactly these ids. A new install starts on `cloud-baidu`, and the four that need nothing
filled in are always in the dropdown. The other two answer with an account of the user's
own: they are in the dropdown only while the credentials on the **General** page are
filled in, and a settings file that names one of them without those credentials reads as
`cloud-baidu` instead — an entry that could only answer with an error is worse than an
entry that is not offered. Those two keep their credentials on this machine, DPAPI
protected for the Windows login that typed them, and send the text straight to the
service they belong to; Glossy's relay is not in the path.

| `service` | Who answers | What it costs the user | In the fallback order | Where the text goes |
| --- | --- | --- | --- | --- |
| `cloud-baidu` | Glossy's own relay, translating with Baidu (`vendor: "baidu"` on the wire) | nothing to set up | yes | the relay, then Baidu |
| `cloud-youdao` | the same relay, translating with Youdao (`vendor: "youdao"`) | nothing to set up | yes | the relay, then Youdao |
| `google` | Google's public `translate.googleapis.com` endpoint | free, no key | yes | Google |
| `api-baidu` | Baidu's 通用文本翻译 API, with the user's own APP ID and key | the user's own Baidu quota | yes, once its credentials are filled in | Baidu, directly |
| `api-openai` | whatever endpoint the user named, speaking the OpenAI chat completions protocol | the user's own account there | yes, once its credentials are filled in | that endpoint, directly |
| `offline` | OPUS-MT models on this machine | one 244 MB download | no, deliberately | nowhere |

## `cloud-baidu`

**Who answers.** A relay deployed from [`server/`](./server/README.md) holds the
project's vendor account and passes the text on to Baidu. The client never sees a key:
it sends the text plus an install id, and the relay decides what the vendor is. This
is the entry a fresh install starts on, because it needs nothing filled in and works
from mainland China.

**What it costs.** Nothing to set up: no account, no key, no card. The calls are made
on an account the project holds, and the relay meters a daily character allowance per
device (install id), per address and in total. What is left of it is written on the
**Language**, **Translate text** and **OCR** pages — the three the settings window can
start a translation from — next to a `Check again` button.

**The terms that apply.** Baidu's 翻译开放平台 服务协议. Its terms are silent on
commercial use, but 服务协议 forbids a *client program* from caching Baidu translation
data and forbids resale (转售). Glossy caches nothing, so this does not bite; a fork
that added a cache would need to revisit it. The quota tiers the repository records —
50k characters a month unverified, 1M personal-verified, 2M business-verified, and
QPS-limited to 1 / 10 / 100 — are what a user gets from their own Baidu account, and
are recorded in full under [Commercial use](#commercial-use). What makes the relay's
use belong to the account holder is that the credential stays on the server and the
allowance is never handed to anyone: the app sends the text and an install id, and
nothing else.

**When it cannot answer.** The relay prefers the vendor it was asked for and walks its
own list of configured backends before it reports a failure, so an answer can come
back from Youdao with the card naming both the engine that answered and the one that
stepped aside, and the reason the relay gave. If the relay as a whole cannot answer,
the app moves on: `cloud-baidu` is in the fallback order, and a fresh install's order
is `cloud-baidu` first, then `cloud-youdao`, then `google`.

**Where the text goes.** From the app to the relay, a deployment made from `server/`
that runs both as a Tencent Cloud SCF Web function and as a Cloudflare Worker. The
address is part of the build rather than a field, and it is the same deployment for
every copy of the app. The relay passes the text on to Baidu and stores nothing: the
text is not logged and not kept, and only the character count and the install id
remain. The install id is derived from the machine and the Windows account rather than
drawn at random, so reinstalling lands on the same id and the allowance does not start
over; the server sees only the 32 hex characters.

## `cloud-youdao`

**Who answers.** The same relay, asked to translate with 有道智云 (`vendor: "youdao"`
on the wire). Who answers the request, and everything about how it is reached, is the
same as `cloud-baidu`; only the engine inside the relay changes.

**What it costs.** Nothing to set up, exactly as `cloud-baidu`: no account, no key, and
the same daily allowance, counted per device, per address and in total, and shown on
the **Language**, **Translate text** and **OCR** pages.

**The terms that apply.** Youdao's own terms for 有道智云, under the same constraint
the README records for every free tier: what a vendor forbids is handing the raw quota
— a key, an interface — to other people or reselling it. The repository does not quote
a Youdao clause individually, and none is invented here. As with `cloud-baidu`, the
credential stays on the server and the app holds nothing, which is what makes the
account holder the one using the quota.

**When it cannot answer.** The relay walks its own backend list first, naming the
engine that answered and the one that did not. If the relay cannot answer at all,
`cloud-youdao` is in the fallback order; it is the second entry of a fresh install's
order, after `cloud-baidu`.

**Where the text goes.** The same path as `cloud-baidu`: the text and the install id to
the relay, which forwards it to Youdao and stores neither it nor anything about the
selection. Nothing but the character count and the install id remain on the server.

## `google`

**Who answers.** Google's public `translate.googleapis.com` endpoint, the
`translate_a/single` route that the Chrome dictionary uses. It needs no key and no
account.

**What it costs.** Free, with no key. Nothing is metered by Glossy, and there is no
allowance to show.

**The terms that apply.** This is a public endpoint run by Google rather than a
documented, keyed translation API, and the ROADMAP records its terms of use as
unclear. It can start rate-limiting or change its protocol at any time; the app already
retries across two client ids to soften that, and the fallback order is the real
answer. It is always queried with the `dict-chrome-ex` client id, and the throttled
`gtx` id is only kept as a fallback. It is blocked on some networks, including much of
mainland China.

**When it cannot answer.** A refusal or a rate limit is treated like any other failure:
`google` is in the fallback order, so the next entry in the configured order is tried —
on a fresh install, `cloud-youdao`. Turning the fallback switch off makes a failure be
shown as a failure instead.

**Where the text goes.** Straight to `translate.googleapis.com`, and no further: this
is the one network entry in the list that does not pass through Glossy's own relay.
Only the text being translated is sent. Google receives it under its own privacy
policy.

## `api-baidu`

**Who answers.** Baidu's own 通用文本翻译 API
(`fanyi-api.baidu.com/api/trans/vip/translate`), called with an APP ID and a key the user
obtained from the 百度翻译开放平台 console. The request is signed on this machine —
`md5(appid + text + salt + key)` — and sent from here, so Glossy's relay is in the path
neither for the text nor for the credential.

**What it costs.** Nothing to the project, and whatever the user's own Baidu account costs:
50k characters a month unverified, 1M personal-verified, 2M business-verified, QPS limited
to 1 / 10 / 100. A standard account is offered the same 23 languages as `cloud-baidu`; the
rest of Baidu's list answers with error `58001` unless the account is an enterprise
尊享版, so they are not offered at all.

**The terms that apply.** The same 翻译开放平台 服务协议 the relay entry is written against,
but here the account holder is the user: the quota is theirs, and so is what the terms
permit. As above, the terms forbid a *client program* from caching Baidu translation data
and forbid resale; Glossy caches nothing, and offers no way to hand the credential or the
quota to anybody else.

**When it cannot answer.** Baidu answers HTTP 200 with an `error_code` for a missing field
(`54000`), a bad signature (`54001`), a wrong APP ID (`52003`), a rate limit (`54003`) and
an exhausted monthly quota (`54004`); each becomes a line that names what to check, and the
fallback order moves on when it is on.

**Where the text goes.** Straight to Baidu from this machine. Only the text being
translated is sent, with the signature that authenticates the account.

## `api-openai`

**Who answers.** Whatever the user pointed the entry at: OpenAI, DeepSeek, Zhipu, or a model
server on their own machine. One entry covers all of them because they share one protocol —
the chat completions request — so the address, the key and the model are settings rather
than a menu of vendors. The model is asked for a JSON object holding the translation and,
for a single word, the phonetics, the meanings and an example sentence, so a word card is
filled from one request. Every language Glossy offers is offered here, because a model is
not limited by a vendor's table.

**What it costs.** Whatever the user's account with that service costs. Glossy counts
nothing for it and shows no allowance line.

**The terms that apply.** The terms of the service the user pointed the entry at, which this
project neither knows nor records.

**When it cannot answer.** The status code and, after it, the service's own reason — an
unknown model, a key without credit — are shown. `response_format` is deliberately not sent,
because it is an OpenAI extension rather than part of the protocol every compatible server
implements; a model that answers with prose instead of the JSON object it was asked for is
reported as such rather than guessed at.

**Where the text goes.** Straight to the address the user typed, from this machine, with the
key in the `Authorization` header.

## `offline`

**Who answers.** This machine. The entry runs OPUS-MT's `opus-mt-en-zh` and
`opus-mt-zh-en` as int8 ONNX through the same ONNX Runtime the screen recogniser
already downloads, and it translates Chinese and English both ways and nothing else —
which is what the two models were trained for. No request leaves the computer, so there
is no account, no key, no allowance and no rate limit.

**What it costs.** One 244 MB download from the **Resources** page. The two directions
are downloaded one at a time and each is about 114 MB, because a direction is a whole
translator on its own and one way round never waits for the other. Until a direction is
downloaded, choosing `offline` answers with an error that says so.

**The terms that apply.** `opus-mt-en-zh` (Helsinki-NLP) is **Apache License 2.0** and
`opus-mt-zh-en` (Helsinki-NLP) is **Creative Commons Attribution 4.0 International**
(CC-BY-4.0); both permit commercial use, and the full texts and the upstream pages are
in [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md). The models are the ONNX exports
published by Xenova. `NLLB-200` is **CC-BY-NC-4.0** and must **not** be shipped, which
is why this pack is OPUS-MT rather than a single model covering more languages.

**When it cannot answer.** `offline` is deliberately **not** in the fallback order. It
is a choice the reader makes, not a substitution the app makes for them: the window
never offers it among the fallback entries, and a failure of the offline pack is shown
as a failure rather than quietly answered by a network service.

**Where the text goes.** Nowhere. The text never leaves the computer, which is the
whole point of the entry, and there is nothing to count or to display.

## The fallback order

The service chosen above is always tried first and cannot be moved; the list under it
holds the other network engines once each, and the arrows put them in the order they
are tried. On a fresh install the order is the chosen service, then `cloud-youdao`,
then `google`. `offline` is never one of them — it is a choice, not a fallback. Turning
**Ask another service when the chosen one fails** off means a failure is shown as a
failure. When another engine had to answer, the card's footer names it and the line
under it names the engine that did not, with the relay's reason when the relay was the
one that moved on (`its allowance is used up`, `the server's account with it was
refused`, and so on); the same line is written to `glossy.log`.

## The other network calls

Besides translation, the app reaches the network for four things. Only the first two
are asked directly by the app; the third goes through the same relay as a selection,
and the fourth is off until it is turned on.

### Dictionary (`api.dictionaryapi.dev`)

Phonetics and definitions for a single English word come from
`api.dictionaryapi.dev`, a public service that receives only the word. It is free, it
needs no key, it is reachable from mainland China, and it is used strictly as a
best-effort extra: the lookup runs after the card is already showing, a failure is
ignored, and the endpoint's rate limiting is why the word's extras can be missing while
the translation is present. The Google endpoint is asked for details only when the
selected provider did not return them.

### Currency rates

When a translation still writes a currency amount in a unit the target language does
not use, the card looks up a rate. It asks Glossy's own relay first (`GET /v1/rates`),
which reads `open.er-api.com` (exchangerate-api.com) and falls back to the European
Central Bank through `api.frankfurter.app`; the app asks those two directly only when
the relay cannot answer. Both are keyless, and a rate does not count against the
translation allowance. Only a base currency code is carried, and nothing about the
user. A rate is cached for six hours, a stale table stays usable for seven days and is
labelled as such, and a failed round is followed by a two-minute quiet period. Every
other unit is built into the app.

### Screenshots, documents and subtitles

A screenshot translation sends only the rectangle that was dragged to the same relay
as a selection, which passes the image on to a cloud OCR service; the rest of the
screen never leaves the machine, and neither the image nor the text it produces is
stored by the app or by the relay. Document translation sends only the prose
paragraphs, and subtitle translation only the text read from the one rectangle that was
picked, both through the same relay and under the same daily allowance as a selection.

### Update checks

Off by default. Turned on in the settings, the app asks GitHub for `releases/latest` of
this repository and nothing else.

## Commercial use

The free tiers differ in what they permit. What they forbid is handing the raw quota —
a key, an interface — to other people or reselling it; keeping the credential on a
server of our own and letting the app call that server is a use of the quota by the
account holder, which is why the two server-backed entries relay it that way.

- **`cloud-baidu` / `cloud-youdao`** — they translate through an LLM account the project
  pays for, and fall back to Baidu credentials when no such account is configured, so
  what they make are paid calls and the terms above do not apply to them. No user ever
  sees or holds a vendor key: the app sends the text plus an install id, and the server
  meters a daily allowance per device and address.
- **`api-baidu` / `api-openai`** — the account is the user's own and so is the quota, and
  the credential is used only by the copy of the app it was entered into: it is DPAPI
  protected for that Windows login, the request is signed and sent from that machine, and
  nothing in Glossy can hand the key or the quota to anybody else. The client caches no
  translation, which is what the 服务协议 asks of a client program.

### Services Glossy does not ship

These are the services the repository researched and did **not** ship, with the reason.
They are recorded here because they are exactly the facts a reader wants to check, and
because the two server-backed entries above relay through Glossy's own deployment
instead of any of them.

- **Zhipu** — 用户协议 §非付费功能 licenses the free models for *非商业的、个人研究学习*
  use only (non-commercial personal study and research). Fine for personal use; not fine
  for a published or paid product.
- **ModelScope API-Inference** — explicitly 非商业化, 非盈利 (non-commercial,
  non-profit).
- **Aliyun 机器翻译** — the monthly free quota is explicitly 仅适用客户试用场景 (for
  customer trials only).
- **Tencent 腾讯云 TMT** — the free 5M characters/month quota is still advertised, but
  the service no longer offers text translation: as of the 2026-03 and 2026-07 releases
  the `TextTranslate`, `TextTranslateBatch`, `ImageTranslate`, `LanguageDetect` and
  `SpeechTranslate` actions were removed and the API 概览 lists only
  `ImageTranslateLLM`. The 计费概述 page is stale, so do not plan on it.
- **Baidu 翻译开放平台** — 50k characters/month unverified, 1M personal-verified, 2M
  business-verified. Its terms are silent on commercial use, but 服务协议 forbids a
  *client program* from caching Baidu translation data and forbids resale. Glossy does
  not cache anything, so this does not bite; a fork that adds a cache would need to
  revisit it. Quotas are QPS-limited to 1 / 10 / 100.
- **Volcengine 火山引擎** — 2M characters/month, but onboarding requires a sales
  contract.
- **NiuTrans 小牛翻译** — 200k characters/day after registering; commercial terms
  unpublished.
- **SiliconFlow** — `tencent/Hunyuan-MT-7B` is free, but the platform terms are silent
  about free models and are restricted to internal business purposes.
- **Fully offline** — `Opus-MT` weights are Apache-2.0 (`opus-mt-en-zh`) and CC-BY-4.0
  (`opus-mt-zh-en`), both commercial OK, ~113 MB int8 per direction and ~300 MB RAM.
  This is what the `offline` entry ships: both directions as int8 ONNX, run through the
  ONNX Runtime the recogniser already downloads. `NLLB-200` is CC-BY-NC-4.0 and must
  **not** be shipped.

## Reading the primary sources

- [`README.md`](./README.md) — the **Providers** and **Free quotas and commercial use**
  sections this page is drawn from, and every figure quoted above.
- [`PRIVACY.md`](./PRIVACY.md) — what is sent where, and what is stored locally.
- [`server/README.md`](./server/README.md) — how the relay is deployed, how its
  allowance is counted, and the vendor accounts it can hold.
- [`THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md) — the licences and the upstream
  pages for the files the app downloads, including the OPUS-MT models at
  [Helsinki-NLP](https://huggingface.co/Helsinki-NLP/opus-mt-en-zh).
- [`PRODUCT.md`](./PRODUCT.md) and [`ROADMAP.md`](./ROADMAP.md) — the promises the
  project has made about these services and the risks it has recorded against them.

Where the repository does not record a link to a service's own terms or pricing page,
this page names the document instead of linking it, so that a reader can look it up
rather than take a summary on trust.

---

# Glossy 翻译服务 · 中文

**翻译服务**设置背后每一项服务是什么、要花什么代价、适用什么条款、服务答不上来时应用会怎么做，
以及文字会发到哪里。它是 [README](./README.md) 里那份调研的配套文档，写给只想翻译、但想知道
自己发了什么、发给了谁的读者。

本文的每一条事实都取自本仓库：README、[PRIVACY.md](./PRIVACY.md)、[PRODUCT.md](./PRODUCT.md)、
中转服务自己的 [server/README.md](./server/README.md)、
[THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)，以及这些文件链接到的上游页面。仓库没有记录
上游链接的地方，本文也不会凭空补一个。

最后核对于 2026-10-05，依据本仓库。本仓库并未记录其调研日期。免费额度与条款随时可能变化，下文
的数字是仓库记录下来的，而不是实时读数——在依赖其中任何一个之前，请先查上游页面。

## 服务

本构建提供六个条目，存在设置文件唯一的 `service` 字段里，用的就是下面这些 id。全新安装从
`cloud-baidu` 开始；四个不需要填写任何东西的条目始终在下拉框里。另外两个用用户自己的账号作答：
只有在**常规**页把凭据填好之后它们才会出现在下拉框里，而一份点名了它们、却没有这些凭据的设置文件
会被读成 `cloud-baidu`——一个只能报错的条目，比一个根本不提供的条目更糟。这两个条目的凭据保存在
本机、用 `DPAPI` 按输入它的那个 Windows 登录加密，文字从本机直接发往它所属的那个服务，Glossy 的
中转服务完全不在链路上。

| `service` | 由谁作答 | 用户要付出什么 | 是否在回退顺序里 | 文字发往何处 |
| --- | --- | --- | --- | --- |
| `cloud-baidu` | Glossy 自己的中转服务，用百度来译（协议上是 `vendor: "baidu"`） | 无需填写 | 是 | 中转服务，再到百度 |
| `cloud-youdao` | 同一台中转服务，用有道来译（`vendor: "youdao"`） | 无需填写 | 是 | 中转服务，再到有道 |
| `google` | Google 公开的 `translate.googleapis.com` 接口 | 免费，无需密钥 | 是 | Google |
| `api-baidu` | 百度自家的通用文本翻译接口，用用户自己的 APP ID 与密钥 | 用户自己百度账号的额度 | 是，凭据填好之后 | 直接发往百度 |
| `api-openai` | 用户指定的任何兼容 OpenAI chat completions 协议的接口 | 用户在那个服务上的账号 | 是，凭据填好之后 | 直接发往那个接口 |
| `offline` | 本机的 OPUS-MT 模型 | 一次 244 MB 的下载 | 否，刻意如此 | 不发往任何地方 |

## `cloud-baidu`

**由谁作答。** 一台由 [`server/`](./server/README.md) 部署出来的中转服务持有本项目的厂商账号，
把文字转给百度。客户端永远看不到密钥：它发送文字加一个安装 id，由中转服务决定上游是谁。全新安装
默认就用它，因为它什么都不用填，而且在中国大陆可以直连。

**要花什么代价。** 无需填写：没有账号、没有密钥、不用绑卡。调用走的是本项目持有的账号，中转服务按
**每台设备（安装 id）、每个地址、以及总量**计算每日字符额度。剩余额度显示在**语言**、**文本翻译**
和 **OCR** 三个页面上——也就是设置窗口能发起翻译的三个页面——旁边有一个 `Check again` 按钮。

**适用什么条款。** 百度 翻译开放平台 服务协议。其条款对商业使用只字未提，但服务协议禁止*客户端
程序*缓存百度翻译数据，也禁止转售。Glossy 不缓存任何内容，所以这不会造成影响；某个加了缓存的分支
则需要重新审视这一点。仓库记录的额度档位——未认证 50k 字符/月、个人认证 1M、企业认证 2M，且受
QPS 限制为 1 / 10 / 100——是用户自建百度账号时才会拿到的，完整记录见下方[商业使用](#商业使用)。
中转服务之所以是账号持有者自己在使用额度，是因为凭据留在服务端、额度从不转手：应用只发送文字和
一个安装 id，别的什么都不发。

**答不上来时。** 中转服务会优先用它被点名的那个上游，在报告失败之前先走一遍自己配置好的上游列表，
所以答案有可能由有道给出，卡片会同时写明是谁作答、是谁让位，以及中转服务给出的原因。如果整台中转
服务都答不上来，应用会继续往下走：`cloud-baidu` 在回退顺序里，全新安装的顺序是 `cloud-baidu`
最前，然后 `cloud-youdao`，最后 `google`。

**文字发往何处。** 从应用发到中转服务——它由 `server/` 部署而成，既可跑成腾讯云 SCF Web 函数，
也可跑成 Cloudflare Worker。地址属于构建的一部分，而不是一个字段，对所有副本都是同一台部署。中转
服务把文字转给百度，什么都不存：文字不写日志、不落盘，留下的只有字符计数和安装 id。安装 id 是
从本机与 Windows 账号推导出来的，而不是随机生成的，所以重装后仍是同一个 id，额度不会重新开始；
服务端看到的只有那 32 个十六进制字符。

## `cloud-youdao`

**由谁作答。** 同一台中转服务，只是指定用有道智云来译（协议上是 `vendor: "youdao"`）。谁作答、
以及访问它的全部方式，都与 `cloud-baidu` 相同；变的只是中转服务内部的上游。

**要花什么代价。** 和 `cloud-baidu` 一样无需填写：没有账号、没有密钥，同一份每日额度，按每台设备、
每个地址和总量计算，显示在**语言**、**文本翻译**和 **OCR** 页面上。

**适用什么条款。** 有道智云自己的条款，并受 README 为所有免费额度记录的同一条约束：厂商禁止的是把
额度本身——密钥、接口——转手给他人或转售。仓库没有单独引用有道的某一条款，本文也不凭空补一条。
和 `cloud-baidu` 一样，凭据留在服务端、应用什么都不持有，这就让账号持有者成为额度真正的使用者。

**答不上来时。** 中转服务先走自己的上游列表，并写明是谁作答、是谁让位。如果整台中转服务都答不上来，
`cloud-youdao` 在回退顺序里；在全新安装的顺序中它排在 `cloud-baidu` 之后，是第二项。

**文字发往何处。** 与 `cloud-baidu` 同一条路：文字和安装 id 发给中转服务，由它转给有道，两边都不
保存这段文字，也不保存任何与选区有关的东西。服务端留下的只有字符计数和安装 id。

## `google`

**由谁作答。** Google 公开的 `translate.googleapis.com` 接口，也就是 Chrome 词典所用的
`translate_a/single` 路径。不需要密钥，也不需要账号。

**要花什么代价。** 免费，无需密钥。Glossy 不做任何计量，也没有额度可显示。

**适用什么条款。** 这是 Google 运营的公开端点，而不是一个有正式文档、需要密钥的翻译 API，
ROADMAP 记录的其使用条款并不明确。它随时可能开始限流或改变协议；应用已经靠两个客户端 id 重试来
缓和这一点，而真正的办法是回退顺序。它始终以 `dict-chrome-ex` 客户端 id 查询，被限流的 `gtx` id
只作为后备保留。它在部分网络中被屏蔽，包括中国大陆的大部分地区。

**答不上来时。** 被拒绝或被限流都按普通失败处理：`google` 在回退顺序里，因此会试用配置顺序中的
下一项——全新安装时是 `cloud-youdao`。把回退开关关掉，失败就会被当作失败显示出来。

**文字发往何处。** 直接发到 `translate.googleapis.com`，不再转发：这是列表里唯一不经过 Glossy
自己中转服务的联网条目。只发送被翻译的文字。Google 按其自己的隐私政策接收它。

## `api-baidu`

**由谁作答。** 百度自家的通用文本翻译接口（`fanyi-api.baidu.com/api/trans/vip/translate`），
用用户在百度翻译开放平台申请来的 APP ID 与密钥调用。请求在本机签名——`md5(appid + 原文 + salt + 密钥)`
——也从本机发出，所以无论原文还是凭据，都不经过 Glossy 的中转服务。

**要花什么代价。** 对项目而言不花什么，代价就是用户自己百度账号的那份额度：未认证每月 50k 字符、
个人认证 1M、企业认证 2M，QPS 限制 1 / 10 / 100。普通账号与 `cloud-baidu` 一样提供那 23 种语言；
百度列表里其余的语言在非企业尊享版账号上会返回 `58001`，所以压根不提供。

**适用什么条款。** 和走中转服务的那个条目依据同一份翻译开放平台服务协议，但这里的账号持有者是用户
本人：额度是他的，条款允许什么也由他决定。同上，该协议禁止*客户端程序*缓存百度翻译数据，也禁止转售；
Glossy 不缓存任何内容，也没有任何把凭据或额度转手给别人的途径。

**答不上来时。** 字段缺失（`54000`）、签名不对（`54001`）、APP ID 不对（`52003`）、被限流
（`54003`）以及当月额度用尽（`54004`）时，百度都以 HTTP 200 加 `error_code` 返回；每一种都会被
翻译成一行说明该检查什么，回退开关打开时会继续试下一项。

**文字发往何处。** 从本机直接发往百度。发送的只有被翻译的原文，以及用于认证账号的签名。

## `api-openai`

**由谁作答。** 用户把这个条目指向谁就是谁：OpenAI、DeepSeek、智谱，或他本机的模型服务。一个条目能
覆盖它们全部，是因为它们共用同一套协议——chat completions 请求——所以地址、密钥和模型是设置项，
而不是一份厂商菜单。模型被要求只回一个 JSON 对象，其中包含译文；如果选中的是单个单词，还要给出音标、
释义和例句，因此一张单词卡一次请求就能凑齐。这里提供 Glossy 支持的全部语言，因为模型不受某家厂商
的语言表限制。

**要花什么代价。** 就是用户在那个服务上的账号代价。Glossy 不做任何计量，也不显示额度行。

**适用什么条款。** 用户指向的那个服务自己的条款，本项目既不知道也不记录。

**答不上来时。** 状态码之后会带上该服务自己的原因——模型名不存在、密钥没有余额。`response_format`
是刻意不发的，因为它是 OpenAI 的扩展而不是所有兼容服务都实现的协议内容；如果模型回了一段散文而不是
被要求的 JSON 对象，应用会如实报告，而不是去猜。

**文字发往何处。** 从本机直接发往用户填写的地址，密钥放在 `Authorization` 头里。

## `offline`

**由谁作答。** 本机。该条目把 OPUS-MT 的 `opus-mt-en-zh` 与 `opus-mt-zh-en` 以 int8 ONNX 跑在
识别引擎已经在用的同一个 ONNX Runtime 上，只在中文和英文之间互译，别的都不行——这正是这两个模型
训练的方向。没有任何请求离开这台电脑，因此没有账号、没有密钥、没有额度，也没有限流。

**要花什么代价。** 在**资源**页下载一次，约 244 MB。两个方向各下各的，每个约 114 MB，因为一个
方向本身就是一台完整的翻译器，选一个方向不用等另一个下完。某个方向还没下载就选它，会直接报错说明
原因。

**适用什么条款。** `opus-mt-en-zh`（Helsinki-NLP）是 **Apache License 2.0**，
`opus-mt-zh-en`（Helsinki-NLP）是 **Creative Commons Attribution 4.0 International**
（CC-BY-4.0）；两者都可商用，完整文本与上游页面在
[THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md) 里。这些模型是 Xenova 发布的 ONNX 导出。
`NLLB-200` 是 **CC-BY-NC-4.0**，**不能**发布，这也是这个包用 OPUS-MT、而不是用单个覆盖更多语言的
模型的原因。

**答不上来时。** `offline` 刻意**不**在回退顺序里。它是读者主动做的选择，而不是应用替他做的替换：
窗口从不在回退条目里提供它，离线包一旦失败就会把失败显示出来，而不会悄悄交给某个联网服务去作答。

**文字发往何处。** 不发往任何地方。文字永远不离开这台电脑，这正是该条目的全部意义，也没有任何东西
需要计数或显示。

## 回退顺序

上面选中的服务总是最先尝试，且不能被移动；它下面的列表把其余每个联网引擎各列一次，箭头决定它们被
尝试的先后。全新安装的顺序是：所选服务、然后 `cloud-youdao`、然后 `google`。`offline` 永远不在
其中——它是一项选择，不是回退。把**所选服务失败时改问其它服务**关掉，失败就会被当作失败显示。
当是别的引擎作答时，卡片页脚会写明它，下面那行写明没有作答的那个引擎，并在是中转服务自己往下走时
给出它给的原因（`its allowance is used up`、`the server's account with it was refused` 等）；
同一行也会写进 `glossy.log`。

## 其余的联网请求

除翻译之外，应用还会为四件事联网。前两件由应用直接请求；第三件与划词一样走同一台中转服务；第四件
在打开之前一直是关着的。

### 词典（`api.dictionaryapi.dev`）

单个英文单词的音标和释义来自 `api.dictionaryapi.dev`，这个公共服务只会收到那个单词。它免费、无需
密钥、中国大陆可直接访问，而且严格只是尽力而为的补充：查询在卡片已经显示之后才进行，失败会被忽略，
也正是该端点的限流导致单词的附加信息可能缺失，而译文依然存在。只有在所选服务没有返回某些细节时，
才会去请求 Google 接口。

### 汇率

当译文仍用目标语言不使用的单位书写一笔货币金额时，卡片会查询汇率。它先问 Glossy 自己的中转服务
（`GET /v1/rates`），后者读 `open.er-api.com`（exchangerate-api.com），取不到就改用欧洲央行的
`api.frankfurter.app`；只有中转服务答不上来时，应用才直连这两家。两家都无需密钥，而且汇率不占翻译
额度。请求只带一个基准货币代码，不含任何与用户有关的信息。汇率缓存六小时，过期的表在七天内仍可用
并明确标注，一轮失败之后会有两分钟静默期。其余所有单位都内置于应用中。

### 截图、文档与字幕

截图翻译只把你框选的那块矩形发给与划词同一台中转服务，由它再把图片转给云端 OCR；屏幕的其余部分
从不离开本机，应用与中转服务也都不保存这张图片或它识别出的文字。文档翻译只发送正文段落，字幕翻译
只发送从你所选那一块矩形里读到的文字，两者都走同一台中转服务，并与划词共用同一份每日额度。

### 更新检查

默认关闭。在设置里打开后，应用只向 GitHub 查询本仓库的 `releases/latest`，不查别的。

## 商业使用

各免费额度在许可范围上并不相同。它们禁止的是把额度本身——密钥、接口——转手给他人或转售；把凭据
留在我们自己的服务器上、让应用去调用这台服务器，属于账号持有者自己使用额度，因此走服务器的那两个
渠道正是以这种方式中转：

- **`cloud-baidu` / `cloud-youdao`** —— 它们走的是本项目付费的 LLM 账号；没有配置该账号时则退回
  百度凭据，因此发出的是付费调用，上面的条款对它不适用。任何用户都看不到也拿不到厂商密钥：应用只
  发送文本和一个安装 id，服务器按设备和地址计算每日额度。
- **`api-baidu` / `api-openai`** —— 账号是用户自己的，额度也是他自己的，而且这份凭据只被填过它的
  那一份客户端使用：它按当前 Windows 登录用 `DPAPI` 加密，请求在那台机器上签名并发出，Glossy 里
  没有任何东西可以把密钥或额度转手给别人。客户端不缓存任何译文，这也正是服务协议对客户端程序的要求。

### Glossy 并未内置的服务

以下是仓库调研过、但**没有**内置的服务，以及原因。之所以记在这里，是因为它们正是读者想要核对的
事实，也因为上面那两个走服务器的条目正是改用 Glossy 自己的部署来中转，而不是其中任何一家。

- **智谱** —— 用户协议 §非付费功能 只把免费模型授权给*非商业的、个人研究学习*用途。个人使用没问题；
  用于已发布或收费的产品则不行。
- **ModelScope API-Inference** —— 明确为非商业化、非盈利。
- **阿里云机器翻译** —— 每月免费额度明确仅适用客户试用场景。
- **腾讯云 TMT** —— 每月 5M 字符的免费额度仍在宣传，但该服务已不再提供文本翻译：自 2026-03 和
  2026-07 的版本起，`TextTranslate`、`TextTranslateBatch`、`ImageTranslate`、`LanguageDetect`
  和 `SpeechTranslate` 动作已被移除，API 概览中只列出 `ImageTranslateLLM`。计费概述页面已经过时，
  所以不要指望它。
- **百度翻译开放平台** —— 未认证 50k 字符/月，个人认证 1M，企业认证 2M。其条款对商业使用只字未提，
  但服务协议禁止*客户端程序*缓存百度翻译数据，也禁止转售。Glossy 不缓存任何内容，所以这不会造成
  影响；某个加了缓存的分支则需要重新审视这一点。额度受 QPS 限制为 1 / 10 / 100。这套凭据在构建里
  仍然没有内置，但用户可以自己填——见上面的 `api-baidu`，那里的账号持有者就是用户本人。
- **火山引擎** —— 每月 2M 字符，但开通需要签订销售合同。
- **小牛翻译** —— 注册后每天 200k 字符；商业条款未公开。
- **SiliconFlow** —— `tencent/Hunyuan-MT-7B` 免费，但平台条款对免费模型只字未提，且只限于企业内部
  业务用途。
- **完全离线** —— `Opus-MT` 权重是 Apache-2.0（`opus-mt-en-zh`）与 CC-BY-4.0（`opus-mt-zh-en`），
  两者都可商用，int8 每个方向约 113 MB，内存约 300 MB。这正是 `offline` 条目所发布的：两个方向
  都以 int8 ONNX 提供，跑在识别引擎已经在下载的 ONNX Runtime 上。`NLLB-200` 是 CC-BY-NC-4.0，
  **不能**发布。

## 阅读一手来源

- [`README.md`](./README.md) —— 本文所依据的 **Providers** 与 **免费额度与商业使用** 两节，
  以及上文引用的每一个数字。
- [`PRIVACY.md`](./PRIVACY.md) —— 什么内容发往何处，以及本地存了什么。
- [`server/README.md`](./server/README.md) —— 中转服务如何部署、额度如何计算，以及它能持有
  哪些厂商账号。
- [`THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md) —— 应用所下载文件的许可证与上游页面，
  包括 [Helsinki-NLP](https://huggingface.co/Helsinki-NLP/opus-mt-en-zh) 的 OPUS-MT 模型。
- [`PRODUCT.md`](./PRODUCT.md) 与 [`ROADMAP.md`](./ROADMAP.md) —— 项目就这些服务做出的承诺，
  以及它为此记录下的风险。

仓库没有记录某项服务自身条款或价格页面的链接时，本文只写出该文档的名称而不给链接，好让读者自己去
查，而不是信任一份摘要。
