/**
 * The other OCR upstream: Tencent Cloud's general print recognition.
 *
 * It answers the same question as `ocr.js` — the text on a screenshot the user
 * framed — and the same shape back, so the handler never knows which vendor a
 * deployment reads pictures with. Tencent is asked first when it is configured
 * because its general print model gives every account a monthly free allowance
 * and reads Chinese and English off a screen a little more steadily.
 *
 * Tencent signs with TC3-HMAC-SHA256 and has no token to cache, so unlike the
 * Baidu path every call carries its own signature. The signing itself lives in
 * `tencent-sign.js`, because the translator signs the same way.
 */

import { tc3Headers } from "./tencent-sign.js";

export const DEFAULT_TENCENT_OCR_ENDPOINT = "https://ocr.tencentcloudapi.com";
export const DEFAULT_TENCENT_OCR_REGION = "ap-guangzhou";
export const TENCENT_OCR_ACTION = "GeneralBasicOCR";
export const TENCENT_OCR_VERSION = "2018-11-19";

const SERVICE = "ocr";

/**
 * The models `general_basic` knows. `auto` reads mixed Chinese and English,
 * which is what a screenshot of a screen almost always holds, so anything this
 * server is not sure about is asked for as `auto` rather than guessed at.
 */
const LANGUAGE_TYPES = new Set(["auto", "zh", "en", "jap", "kor"]);

/** Maps the tag the App sends onto the language Tencent should read the picture with. */
export function tencentLanguage(language) {
  const lower = String(language || "").trim().toLowerCase();
  const base = lower.split(/[-_]/)[0];
  if (!base || base === "auto") return "auto";

  const mapped = { zh: "zh", en: "en", ja: "jap", ko: "kor" }[base];
  return mapped && LANGUAGE_TYPES.has(mapped) ? mapped : "auto";
}

/** Tencent answers a refusal with HTTP 200 and one of these codes. */
const TENCENT_ERRORS = {
  "AuthFailure.SignatureExpire": { code: "upstream_credentials", message: "腾讯云识图凭证已过期。" },
  "AuthFailure.SecretIdNotFound": { code: "upstream_credentials", message: "腾讯云识图 SecretId 不对。" },
  "AuthFailure.SignatureFailure": { code: "upstream_credentials", message: "腾讯云识图签名没通过。" },
  "AuthFailure.UnauthorizedOperation": { code: "upstream_credentials", message: "这个腾讯云账号没有识图权限。" },
  "FailedOperation.ImageDecodeFailed": { code: "upstream_error", message: "这张图没能解码。" },
  "FailedOperation.OcrFailed": { code: "upstream_error", message: "识图服务没能认出这张图里的文字。" },
  "FailedOperation.UnKnowError": { code: "upstream_error", message: "识图服务出了点问题，请稍后再试。" },
  "FailedOperation.UnOpenError": { code: "upstream_limit", message: "腾讯云的「通用印刷体识别」还没有开通，请先在控制台开通。" },
  "LimitExceeded": { code: "upstream_limit", message: "识图请求太频繁了，请稍后再试。" },
  "RequestLimitExceeded": { code: "upstream_limit", message: "识图请求太频繁了，请稍后再试。" },
  "ResourceUnavailable.InArrears": { code: "upstream_limit", message: "腾讯云账号欠费，识图服务已停用。" },
  "ResourceUnavailable.NotExist": { code: "upstream_limit", message: "腾讯云识图的免费额度用完了。" },
  "ResourcesSoldOut": { code: "upstream_limit", message: "识图服务暂时没有可用的额度。" },
  "InternalError": { code: "upstream_error", message: "识图服务内部出错了，请稍后再试。" },
};

/** Catches the codes Tencent adds suffixes to, so nothing falls through to a bare number. */
const PREFIX_ERRORS = {
  AuthFailure: { code: "upstream_credentials", message: "腾讯云识图没有通过认证。" },
  LimitExceeded: { code: "upstream_limit", message: "识图请求太频繁了，请稍后再试。" },
  ResourceUnavailable: { code: "upstream_limit", message: "识图服务的额度不够，或者这个接口还没有开通。" },
  ResourcesSoldOut: { code: "upstream_limit", message: "识图服务暂时没有可用的额度。" },
  InternalError: { code: "upstream_error", message: "识图服务内部出错了，请稍后再试。" },
  FailedOperation: { code: "upstream_error", message: "识图服务没能处理这张图。" },
  InvalidParameter: { code: "upstream_error", message: "识图服务不理解这次请求的参数。" },
};

function tencentProblem(raw) {
  const known = TENCENT_ERRORS[raw] || PREFIX_ERRORS[String(raw).split(".")[0]];
  if (known) return { ...known, upstream: raw };
  return { code: "upstream_error", message: `识图服务返回错误 ${raw}。`, upstream: raw };
}

/**
 * Reads the text on one screenshot.
 *
 * `image` is base64 without the `data:` prefix, which is what the App uploads.
 * `endpoint` and `region` only exist so a local mock can stand in while
 * developing; production leaves them unset.
 */
export async function recognizeWithTencent({
  fetchImpl,
  secretId,
  secretKey,
  image,
  language,
  endpoint,
  region,
  now = () => Date.now(),
}) {
  if (!secretId || !secretKey) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置识图密钥。" };
  }

  const url = endpoint || DEFAULT_TENCENT_OCR_ENDPOINT;
  const payload = JSON.stringify({ ImageBase64: image, LanguageType: tencentLanguage(language) });
  const headers = await tc3Headers({
    service: SERVICE,
    host: new URL(url).host,
    action: TENCENT_OCR_ACTION,
    version: TENCENT_OCR_VERSION,
    region: region || DEFAULT_TENCENT_OCR_REGION,
    payload,
    secretId,
    secretKey,
    timestamp: now(),
  });

  let response;
  try {
    response = await fetchImpl(url, { method: "POST", headers, body: payload });
  } catch (error) {
    return { ok: false, code: "upstream_unreachable", message: `无法连接识图服务：${error}` };
  }

  let data;
  try {
    data = JSON.parse(await response.text());
  } catch {
    return { ok: false, code: "upstream_error", message: `识图服务返回了无法解析的内容（HTTP ${response.status}）。` };
  }

  const refusal = data?.Response?.Error;
  if (refusal) return { ok: false, ...tencentProblem(String(refusal.Code || "")) };

  const detections = Array.isArray(data?.Response?.TextDetections) ? data.Response.TextDetections : [];
  const text = detections
    .map((entry) => (entry && typeof entry.DetectedText === "string" ? entry.DetectedText.trim() : ""))
    .filter(Boolean)
    .join("\n");

  if (!text) {
    return { ok: false, code: "no_text", message: "这张图里没有认出文字，换一块区域再试试。" };
  }

  return { ok: true, text, language: tencentLanguage(language) };
}
