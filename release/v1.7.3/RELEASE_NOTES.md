[English](#en) · [中文](#zh-cn)

<a id="en"></a>

## English

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

<a id="zh-cn"></a>

## 中文

卡片自己兜底时也说明原因的一版。它写的就是中转兜底早就写的那一行，只是 App 之前无话可写。

### 修复

- **App 自己兜底时,那一行没有写原因。** 所选服务失败时,App 会接着试下一个,卡片会写出「回答的那一家」和「没答的那一家」——但只有**中转**兜底时才带着原因,因为中转会把拒绝码一起报回来。App 自己兜底的那种情况发出的原因码是空的,于是卡片上只有一句「有道翻译 未能应答」,而 `glossy.log` 里却写着完整的原因("Could not reach the Glossy translation server")。现在不应答的服务会和中转用同一套小小的原因码说明理由——服务商自己的拒绝(`upstream_limit` 之类),或者根本没有应答的是中转本身时用 `relay_unreachable`——卡片上就会写「有道翻译 未能应答——连不上 Glossy 的中转服务器」。中转自身的拒绝(它自己的当日额度、限流)故意不带原因码:把中转撞上的限制写成一页脚里那家引擎的错,那就是撒谎,细节留在日志里。

---

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by 
[SignPath Foundation](https://signpath.org) — see the 
[code signing policy](https://github.com/SpencerZXWu/Glossy#code-signing-policy). 
Privacy: [PRIVACY.md](https://github.com/SpencerZXWu/Glossy/blob/main/PRIVACY.md).
