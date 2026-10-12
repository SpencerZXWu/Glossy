[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

### Added

- **`GET /v1/dict`: the word card's meanings, looked up by the deployment.** A
  single word's phonetics and definitions used to come from a public endpoint in
  the reader's own network, which answers in anything from half a second to
  twenty — measured here as one request in three taking over twenty seconds and
  coming back `522` — so a word card could sit empty while the card beside it
  was already translated. The relay now looks the word up from its own network
  and answers in a fraction of a second, and the public endpoint is only asked
  for a word the relay has no entry for. The route takes no credentials, books
  no characters (only the per-minute limit), and refuses anything that is not a
  single word.

### Changed

- **The word card is built on the relay's answer, not on a timeout.** The public
  endpoint's budget drops from eight seconds to three, and it is not called at
  all for a word the relay knew in full: what is left for it is the word the
  deployment does not have, which is the case its budget was sized for. A word
  the relay answered in 400 ms used to wait up to eight seconds for a second
  opinion it did not need.

<a id="zh-cn"></a>

## 中文

这一版把单词卡片的释义查询从"读者自己的网络去问一个常常要二十秒的公用接口"改成"由部署方自己的
网络去查"，卡片不再为了音标和释义空转。

### 新增

- **`GET /v1/dict`：单词卡片的释义改由部署方去查。** 单个单词的音标和释义以前来自读者自己网络里的
  `api.dictionaryapi.dev`，它的答复时间从半秒到二十秒都有——本机实测三次里有一次超过 20 秒并返回
  `522`——所以一张单词卡片可能在旁边的译文已经出来之后还空着。现在由中转服务从自己的网络去查，
  零点几秒就回来；只有中转里没有的词，才会去问那个公共接口。这个路由不需要任何密钥，不花字符额度
  （只计入每分钟请求数），并且拒绝不是单个单词的请求。

### 变更

- **单词卡片建立在中转的答复上，而不是建立在一个超时上。** 公共接口的预算从八秒降到三秒，而且中转
  已经查全的词根本不会去问它：留给它的是"部署那边也没有"的词，那个预算正是为这种情况定的。中转
  400 毫秒就答出来的词，以前还要再等最多八秒去听一个并不需要的第二个意见。

部署那边的 `/v1/dict` 不需要申请密钥、也不需要填环境变量：它用的词典接口本身是公开的，没配翻译
密钥的部署同样能用。`YOUDAO_DICT_ENDPOINT` 只在你想换一个词典后端时才需要。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
