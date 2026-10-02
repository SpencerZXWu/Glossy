/**
 * The OCR upstream: Baidu's general text recognition.
 *
 * The App uploads a screenshot of the area the user framed and gets the text on
 * it back; the text is then translated like any other selection, so the picture
 * only ever passes through this server on its way to Baidu and is never stored.
 *
 * Baidu hands out a short-lived access token per key pair, so it is kept for as
 * long as it is good for. On a single-process host that means one extra request
 * every few weeks; on a Worker it means one per isolate.
 */

export const DEFAULT_OCR_ENDPOINT = "https://aip.baidubce.com/rest/2.0/ocr/v1/general_basic";
export const DEFAULT_OCR_TOKEN_ENDPOINT = "https://aip.baidubce.com/oauth/2.0/token";

/**
 * The languages `general_basic` can read. Anything else is asked for as the
 * mixed Chinese and English model, which is what a screenshot of a screen
 * almost always holds.
 */
const LANGUAGE_TYPES = new Set([
  "CHN_ENG",
  "ENG",
  "POR",
  "FRE",
  "GER",
  "ITA",
  "SPA",
  "RUS",
  "JAP",
  "KOR",
  "VIE",
]);

/** Maps the tag the App sends onto the model Baidu should read the picture with. */
export function ocrLanguage(language) {
  const lower = String(language || "").trim().toLowerCase();
  const base = lower.split(/[-_]/)[0];
  if (!base || base === "auto") return "CHN_ENG";

  const byBase = { zh: "CHN_ENG", en: "ENG", ja: "JAP", ko: "KOR", fr: "FRE", de: "GER", es: "SPA", it: "ITA", pt: "POR", ru: "RUS", vi: "VIE" };
  const mapped = byBase[base];
  return LANGUAGE_TYPES.has(mapped) ? mapped : "CHN_ENG";
}

/** Baidu answers failures with HTTP 200 and one of these codes. */
const OCR_ERRORS = {
  6: { code: "upstream_credentials", message: "百度智能云账号还没有开通「通用文字识别」。" },
  17: { code: "upstream_limit", message: "服务端识图额度今天用完了，请明天再试。" },
  18: { code: "upstream_limit", message: "识图请求太频繁了，请稍后再试。" },
  19: { code: "upstream_limit", message: "服务端识图总额度已用尽。" },
  100: { code: "upstream_error", message: "识图服务拒绝了这次请求。" },
  110: { code: "upstream_credentials", message: "服务端识图账号未通过认证。" },
  111: { code: "upstream_credentials", message: "服务端识图凭证已过期。" },
  216100: { code: "upstream_error", message: "识图服务不理解这次请求的参数。" },
  216101: { code: "upstream_error", message: "识图服务没有收到图片。" },
  216200: { code: "upstream_error", message: "图片是空的。" },
  216201: { code: "upstream_error", message: "图片格式不支持。" },
  216202: { code: "upstream_error", message: "图片太大了。" },
  216203: { code: "upstream_error", message: "图片打不开。" },
  216630: { code: "upstream_error", message: "识图服务没能认出这张图里的文字。" },
  216631: { code: "upstream_error", message: "识图服务没能认出这张图里的文字。" },
  282810: { code: "upstream_error", message: "图片太模糊了，请重新截图。" },
};

/** Codes worth one more try: the token may simply have gone stale early. */
const RETRY_WITH_FRESH_TOKEN = new Set([110, 111]);

/** Tokens live for 30 days; they are renewed a little before they run out. */
const TOKEN_MARGIN_MS = 60 * 60 * 1000;

const tokens = new Map();

async function accessToken({ fetchImpl, apiKey, secretKey, tokenEndpoint, now, force }) {
  const cached = tokens.get(apiKey);
  if (!force && cached && cached.expiresAt > now()) return cached.token;

  const url = new URL(tokenEndpoint || DEFAULT_OCR_TOKEN_ENDPOINT);
  url.searchParams.set("grant_type", "client_credentials");
  url.searchParams.set("client_id", apiKey);
  url.searchParams.set("client_secret", secretKey);

  let response;
  try {
    response = await fetchImpl(url.toString(), { method: "POST" });
  } catch (error) {
    return { error: `无法连接识图服务：${error}` };
  }

  let data;
  try {
    data = JSON.parse(await response.text());
  } catch {
    return { error: `识图服务返回了无法解析的内容（HTTP ${response.status}）。` };
  }

  if (typeof data?.access_token !== "string" || !data.access_token) {
    // The token endpoint reports its own failures in `error` / `error_description`.
    return {
      error:
        data?.error === "invalid_client"
          ? "服务端识图账号未通过认证，请检查 API Key 与 Secret Key。"
          : `识图服务没有发放凭证：${data?.error_description || data?.error || `HTTP ${response.status}`}`,
    };
  }

  const seconds = Number.parseInt(data.expires_in ?? "", 10);
  const lifetime = Number.isFinite(seconds) && seconds > 0 ? seconds * 1000 : 30 * 24 * 3600 * 1000;
  tokens.set(apiKey, { token: data.access_token, expiresAt: now() + lifetime - TOKEN_MARGIN_MS });
  return { token: data.access_token };
}

/** Forgets the cached token of one key pair; the tests and the retry path use it. */
export function forgetToken(apiKey) {
  tokens.delete(apiKey);
}

/**
 * Reads the text on one screenshot.
 *
 * `image` is base64 without the `data:` prefix, which is what the App uploads.
 * `endpoint` and `tokenEndpoint` only exist so a local mock can stand in while
 * developing; production leaves them unset.
 */
export async function recognizeWithBaidu({
  fetchImpl,
  apiKey,
  secretKey,
  image,
  language,
  endpoint,
  tokenEndpoint,
  now = () => Date.now(),
}) {
  if (!apiKey || !secretKey) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置识图密钥。" };
  }

  for (let attempt = 0; ; attempt += 1) {
    const token = await accessToken({ fetchImpl, apiKey, secretKey, tokenEndpoint, now, force: attempt > 0 });
    if (token.error) return { ok: false, code: "upstream_unreachable", message: token.error };

    const body = new URLSearchParams({
      image,
      language_type: ocrLanguage(language),
      detect_direction: "true",
      detect_language: "false",
      paragraph: "false",
      probability: "false",
    });

    const url = new URL(endpoint || DEFAULT_OCR_ENDPOINT);
    url.searchParams.set("access_token", token.token);

    let response;
    try {
      response = await fetchImpl(url.toString(), {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: body.toString(),
      });
    } catch (error) {
      return { ok: false, code: "upstream_unreachable", message: `无法连接识图服务：${error}` };
    }

    let data;
    try {
      data = JSON.parse(await response.text());
    } catch {
      return { ok: false, code: "upstream_error", message: `识图服务返回了无法解析的内容（HTTP ${response.status}）。` };
    }

    if (data.error_code) {
      // A stale token is the one failure the caller can still fix by itself.
      if (RETRY_WITH_FRESH_TOKEN.has(Number(data.error_code)) && attempt === 0) {
        forgetToken(apiKey);
        continue;
      }
      const known = OCR_ERRORS[Number(data.error_code)];
      return {
        ok: false,
        code: known ? known.code : "upstream_error",
        message: known ? known.message : `识图服务返回错误 ${data.error_code}。`,
        upstream: String(data.error_code),
      };
    }

    const words = Array.isArray(data.words_result) ? data.words_result : [];
    const text = words
      .map((entry) => (entry && typeof entry.words === "string" ? entry.words.trim() : ""))
      .filter(Boolean)
      .join("\n");

    if (!text) {
      return { ok: false, code: "no_text", message: "这张图里没有认出文字，换一块区域再试试。" };
    }

    return { ok: true, text, language: ocrLanguage(language) };
  }
}
