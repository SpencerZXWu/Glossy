/**
 * 有道智云的文本翻译（`openapi.youdao.com/api`，v3 签名）。
 *
 * 和百度那条路一样：密钥只留在服务端，App 什么也不用填。有道要的是一次
 * SHA-256 签名，语言代码也和 App 送来的 BCP 47 标签不同（`zh-CN` 要写成
 * `zh-CHS`），所以映射放在这里，客户端不用管。
 */

export const YOUDAO_ENDPOINT = "https://openapi.youdao.com/api";

/** 有道报回来的语言代码换回 App 用的标签。 */
const APP_TAGS = { "zh-CHS": "zh-CN", "zh-CHT": "zh-TW" };

/** 有道把中文拆成简繁两支；另外几个标签的拼法和常见写法不同。 */
const YOUDAO_CODES = {
  zh: "zh-CHS",
  nb: "no",
  nn: "no",
  iw: "he",
  in: "id",
};

/** 把一个语言标签映射成有道要的代码。`auto` 表示让它自己判断。 */
export function youdaoCode(language) {
  const lower = String(language || "").trim().toLowerCase();
  if (!lower || lower === "auto") return "auto";

  const base = lower.split(/[-_]/)[0];
  if (base === "zh") return /tw|hk|hant/.test(lower) ? "zh-CHT" : "zh-CHS";
  return YOUDAO_CODES[base] || base;
}

/**
 * 有道只把「前 10 个字符 + 全文长度 + 后 10 个字符」放进签名，超过 20 个字符
 * 的文本按这个规则截断，否则签名对不上。
 */
export function youdaoInput(text) {
  const chars = [...text];
  if (chars.length <= 20) return text;
  return `${chars.slice(0, 10).join("")}${chars.length}${chars.slice(-10).join("")}`;
}

/** 有道的失败也用 HTTP 200 返回，错误码在这张表里。 */
const YOUDAO_ERRORS = {
  101: { code: "upstream_error", message: "翻译服务说请求缺少参数。" },
  102: { code: "unsupported_language", message: "这个语言方向不支持翻译。" },
  103: { code: "too_long", message: "这次要翻译的文本太长了。" },
  108: { code: "upstream_credentials", message: "服务端翻译账号未通过认证。" },
  110: { code: "upstream_credentials", message: "服务端翻译账号没有开通这个服务。" },
  111: { code: "upstream_credentials", message: "服务端翻译账号无效。" },
  112: { code: "upstream_error", message: "服务端请求的翻译服务无效。" },
  113: { code: "upstream_error", message: "翻译服务没有收到要翻译的文本。" },
  114: { code: "too_long", message: "这次要翻译的文本太长了。" },
  202: { code: "upstream_credentials", message: "服务端签名错误。" },
  203: { code: "upstream_error", message: "翻译服务不接受本服务器的地址。" },
  401: { code: "upstream_limit", message: "服务端翻译账号已欠费。" },
  411: { code: "upstream_limit", message: "翻译服务调用过于频繁，请稍后再试。" },
  412: { code: "upstream_limit", message: "翻译服务调用过于频繁，请稍后再试。" },
  501: { code: "upstream_error", message: "翻译服务出了点问题，请稍后再试。" },
};

/** 值得重试的：限频和有道自己的抖动。 */
const TRANSIENT = new Set(["411", "412", "501"]);
const RETRY_DELAYS_MS = [600, 1400];

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function salt() {
  return `${Date.now()}${Math.floor(Math.random() * 1e6)}`;
}

async function signature(appKey, text, saltValue, curtime, secret) {
  const bytes = new TextEncoder().encode(appKey + youdaoInput(text) + saltValue + curtime + secret);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/**
 * 调用有道一次，返回译文或者一个 App 能翻成自己措辞的错误码。
 *
 * `endpoint` 只是为了本地开发时能用假接口顶替有道，线上不传。
 */
export async function translateWithYoudao({
  fetchImpl,
  appKey,
  secret,
  text,
  from,
  to,
  endpoint,
  sleep = wait,
  now = () => Date.now(),
}) {
  if (!appKey || !secret) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置有道翻译的密钥。" };
  }

  for (let attempt = 0; ; attempt += 1) {
    const result = await callYoudao({ fetchImpl, appKey, secret, text, from, to, endpoint, now });
    if (result.ok || attempt >= RETRY_DELAYS_MS.length || !TRANSIENT.has(result.upstream)) return result;
    await sleep(RETRY_DELAYS_MS[attempt]);
  }
}

async function callYoudao({ fetchImpl, appKey, secret, text, from, to, endpoint, now }) {
  const body = new URLSearchParams({
    q: text,
    from: youdaoCode(from),
    to: youdaoCode(to),
    appKey,
    salt: salt(),
    signType: "v3",
    curtime: String(Math.floor(now() / 1000)),
  });
  body.set("sign", await signature(appKey, text, body.get("salt"), body.get("curtime"), secret));

  let response;
  try {
    response = await fetchImpl(endpoint || YOUDAO_ENDPOINT, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: body.toString(),
    });
  } catch (error) {
    return { ok: false, code: "upstream_unreachable", message: `无法连接翻译服务：${error}` };
  }

  const raw = await response.text();
  let data;
  try {
    data = JSON.parse(raw);
  } catch {
    return { ok: false, code: "upstream_error", message: `翻译服务返回了无法解析的内容（HTTP ${response.status}）。` };
  }

  const errorCode = data.errorCode === undefined ? "0" : String(data.errorCode);
  if (errorCode !== "0") {
    const known = YOUDAO_ERRORS[Number(errorCode)];
    return {
      ok: false,
      code: known ? known.code : "upstream_error",
      message: known ? known.message : `翻译服务返回错误 ${errorCode}。`,
      upstream: errorCode,
    };
  }

  const parts = Array.isArray(data.translation) ? data.translation : [];
  const translation = parts
    .map((entry) => (typeof entry === "string" ? entry : ""))
    .filter((entry) => entry)
    .join("\n");

  if (!translation) {
    return { ok: false, code: "upstream_error", message: "翻译服务没有返回译文。" };
  }

  return {
    ok: true,
    from: typeof data.l === "string" && data.l ? APP_TAGS[data.l] || data.l : youdaoCode(from),
    to: youdaoCode(to),
    translation,
  };
}
