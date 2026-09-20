/**
 * Language codes and the Baidu call itself.
 *
 * Baidu spells most languages with three letters and a few with codes of its
 * own, so the codes the App sends (BCP 47 or plain two letter tags) are mapped
 * here instead of in every client.
 */

import { md5 } from "./md5.js";

const BAIDU_ENDPOINT = "https://fanyi-api.baidu.com/api/trans/vip/translate";

const BAIDU_CODES = {
  zh: "zh",
  ja: "jp",
  ko: "kor",
  fr: "fra",
  es: "spa",
  ar: "ara",
  uk: "ukr",
  vi: "vie",
  ms: "may",
  da: "dan",
  fi: "fin",
  he: "heb",
  no: "nor",
  ro: "rom",
  sv: "swe",
};

/** Maps one tag onto the code Baidu expects. Empty means "detect it". */
export function baiduCode(language) {
  const lower = String(language || "").trim().toLowerCase();
  if (!lower || lower === "auto") return "auto";

  const base = lower.split(/[-_]/)[0];
  if (base === "zh") return /tw|hk|hant/.test(lower) ? "cht" : "zh";
  return BAIDU_CODES[base] || base;
}

/** True for something that could be a language tag, so junk never reaches Baidu. */
export function isLanguageTag(language) {
  const lower = String(language || "").trim().toLowerCase();
  if (!lower || lower === "auto") return true;
  return /^[a-z]{2,3}([-_][a-z0-9]{2,8})*$/.test(lower) && lower.length <= 16 && baiduCode(lower) !== "";
}

/** Baidu answers failures with HTTP 200 and one of these codes. */
const BAIDU_ERRORS = {
  52001: { code: "upstream_timeout", message: "翻译服务响应超时，请稍后再试。" },
  52002: { code: "upstream_error", message: "翻译服务出了点问题，请稍后再试。" },
  52003: { code: "upstream_credentials", message: "服务端翻译账号未通过认证。" },
  54000: { code: "upstream_error", message: "翻译服务拒绝了这次请求。" },
  54001: { code: "upstream_credentials", message: "服务端签名错误。" },
  54003: { code: "upstream_limit", message: "翻译服务调用过于频繁，请稍后再试。" },
  54004: { code: "upstream_limit", message: "服务端翻译额度已用尽。" },
  54005: { code: "upstream_limit", message: "短时间内长文本请求太多，请稍后再试。" },
  58000: { code: "upstream_error", message: "翻译服务拒绝了本服务器的地址。" },
  58001: { code: "unsupported_language", message: "这个语言方向不支持翻译。" },
  58002: { code: "upstream_error", message: "翻译服务当前已关闭。" },
  90107: { code: "upstream_credentials", message: "服务端翻译账号未通过认证。" },
};

function salt() {
  return `${Date.now()}${Math.floor(Math.random() * 1e6)}`;
}

/** Answers worth retrying: per-second throttling and Baidu's own hiccups. */
const TRANSIENT = new Set(["52001", "52002", "54003"]);
const RETRY_DELAYS_MS = [600, 1400];

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Calls Baidu once and answers with either the translation or a code the App
 * can turn into its own wording.
 *
 * Baidu's standard plan allows one request per second, and clicking translate
 * twice in a row is faster than that, so the transient answers below get one
 * more try before the App sees an error.
 *
 * `endpoint` only exists so a local mock can stand in for Baidu while
 * developing; production leaves it unset.
 */
export async function translateUpstream({ fetchImpl, appId, key, text, from, to, endpoint, sleep = wait }) {
  if (!appId || !key) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置百度翻译的密钥。" };
  }

  for (let attempt = 0; ; attempt += 1) {
    const result = await callBaidu({ fetchImpl, appId, key, text, from, to, endpoint });
    if (result.ok || attempt >= RETRY_DELAYS_MS.length || !TRANSIENT.has(result.upstream)) return result;
    await sleep(RETRY_DELAYS_MS[attempt]);
  }
}

async function callBaidu({ fetchImpl, appId, key, text, from, to, endpoint }) {
  const body = new URLSearchParams({
    q: text,
    from: baiduCode(from),
    to: baiduCode(to),
    appid: appId,
    salt: salt(),
    sign: "",
  });
  body.set("sign", md5(`${appId}${text}${body.get("salt")}${key}`));

  let response;
  try {
    response = await fetchImpl(endpoint || BAIDU_ENDPOINT, {
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

  if (data.error_code) {
    const known = BAIDU_ERRORS[String(data.error_code)];
    return {
      ok: false,
      code: known ? known.code : "upstream_error",
      message: known ? known.message : `翻译服务返回错误 ${data.error_code}。`,
      upstream: String(data.error_code),
    };
  }

  const parts = Array.isArray(data.trans_result) ? data.trans_result : [];
  const translation = parts
    .map((entry) => (entry && typeof entry.dst === "string" ? entry.dst : ""))
    .filter((entry) => entry)
    .join("\n");

  if (!translation) {
    return { ok: false, code: "upstream_error", message: "翻译服务没有返回译文。" };
  }

  return {
    ok: true,
    from: typeof data.from === "string" ? data.from : baiduCode(from),
    to: typeof data.to === "string" ? data.to : baiduCode(to),
    translation,
  };
}
