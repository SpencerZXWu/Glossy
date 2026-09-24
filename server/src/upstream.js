/**
 * Language codes and the translation calls behind the cloud channel.
 *
 * Baidu spells most languages with three letters and a few with codes of its
 * own, so the codes the App sends (BCP 47 or plain two letter tags) are mapped
 * here instead of in every client. Youdao has its own mapping in `youdao.js`;
 * the LLM upstream in `llm.js` needs no mapping at all, it is given a name
 * instead, which it reads better than a tag.
 */

import { translateWithLlm } from "./llm.js";
import { md5 } from "./md5.js";
import { fetchRates } from "./rates.js";
import { translateWithYoudao } from "./youdao.js";

const BAIDU_ENDPOINT = "https://fanyi-api.baidu.com/api/trans/vip/translate";

/**
 * Vendors the App can name in a request. Anything else — an empty string, an
 * older build, a hand written call — starts at the top of the list.
 */
export const VENDORS = ["baidu", "youdao"];

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

/**
 * Whether this deployment has anything it can translate with.
 *
 * One place, so the health endpoint and the factory cannot disagree: a
 * deployment is ready as soon as it holds either an LLM key or one of the two
 * machine translation pairs.
 */
export function upstreamConfigured(config) {
  return Boolean(
    config.LLM_API_KEY ||
      (config.BAIDU_APP_ID && config.BAIDU_KEY) ||
      (config.YOUDAO_APP_KEY && config.YOUDAO_APP_SECRET),
  );
}

/**
 * Builds the upstream the handler talks to.
 *
 * Every configured backend is kept and tried in order, LLM first because it
 * translates better and the operator opted into it by setting a key. A backend
 * that is down, throttled or out of quota therefore hands the request to the
 * next one instead of failing the user.
 *
 * A request may name the vendor the user picked in the App (`vendor`), which
 * moves that backend to the front; the others stay behind it as a fallback, so
 * a broken vendor still answers with a translation.
 */
export function createUpstream(config) {
  const fetchImpl = (...args) => fetch(...args);
  const backends = [];

  if (config.LLM_API_KEY) {
    backends.push({
      name: "llm",
      call: (input) =>
        translateWithLlm({
          fetchImpl,
          key: config.LLM_API_KEY,
          endpoint: config.LLM_ENDPOINT,
          model: config.LLM_MODEL,
          ...input,
        }),
    });
  }

  if (config.BAIDU_APP_ID && config.BAIDU_KEY) {
    backends.push({
      name: "baidu",
      call: (input) =>
        translateUpstream({
          fetchImpl,
          appId: config.BAIDU_APP_ID,
          key: config.BAIDU_KEY,
          endpoint: config.BAIDU_ENDPOINT,
          ...input,
        }),
    });
  }

  if (config.YOUDAO_APP_KEY && config.YOUDAO_APP_SECRET) {
    backends.push({
      name: "youdao",
      call: (input) =>
        translateWithYoudao({
          fetchImpl,
          appKey: config.YOUDAO_APP_KEY,
          secret: config.YOUDAO_APP_SECRET,
          endpoint: config.YOUDAO_ENDPOINT,
          ...input,
        }),
    });
  }

  return {
    isLanguageTag,
    configured: upstreamConfigured(config),
    /** Names of the vendors this deployment can serve, in the order it tries them. */
    vendors: backends.map((backend) => backend.name),
    /**
     * Live exchange rates, fetched from here rather than from the user's own
     * network. No key and no vendor choice involved, so this is always offered
     * — a deployment without keys can still serve rates.
     */
    rates: (input = {}) =>
      fetchRates({
        fetchImpl,
        primary: config.RATES_ENDPOINT,
        fallback: config.RATES_FALLBACK_ENDPOINT,
        timeoutMs: Number.parseInt(config.RATES_TIMEOUT_MS ?? "", 10) || undefined,
        ...input,
      }),
    async translate(input = {}) {
      if (!backends.length) {
        return { ok: false, code: "not_configured", message: "服务端还没有配置翻译密钥。" };
      }

      const wanted = VENDORS.indexOf(String(input.vendor || "").trim().toLowerCase()) !== -1
        ? String(input.vendor).trim().toLowerCase()
        : "";
      const order = wanted
        ? [...backends.filter((backend) => backend.name === wanted), ...backends.filter((backend) => backend.name !== wanted)]
        : backends;

      let last;
      for (const backend of order) {
        last = { ...(await backend.call(input)), vendor: backend.name };
        if (last.ok) return last;
      }
      return last;
    },
  };
}
