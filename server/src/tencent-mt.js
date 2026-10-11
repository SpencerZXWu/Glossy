/**
 * Tencent Cloud machine translation (TMT).
 *
 * The relay asks it when the engines it normally uses have nothing left to give:
 * the same account that reads screenshots holds it, so a deployment that already
 * has a reader needs one switch rather than a second account. It is not offered
 * to the App as a service of its own — nobody picks it, it sits behind whichever
 * service was asked for — because it is the operator's account paying for it and
 * its allowance belongs to the whole deployment at once.
 *
 * The endpoint, the action and the version are the ones the vendor's own SDK
 * uses (`tmt.tencentcloudapi.com`, `TextTranslate`, `2018-03-21`), and the
 * signing is the same code the reader signs with.
 */

import { tc3Headers } from "./tencent-sign.js";

export const DEFAULT_TENCENT_MT_ENDPOINT = "https://tmt.tencentcloudapi.com";
export const DEFAULT_TENCENT_MT_REGION = "ap-guangzhou";
export const TENCENT_MT_ACTION = "TextTranslate";
export const TENCENT_MT_VERSION = "2018-03-21";

const SERVICE = "tmt";

/**
 * The codes TMT takes, by the tag an App asks with.
 *
 * TMT names eighteen languages, in the shape its own clients use: `zh`, `en`,
 * `ja`, `ko` and so on. Two entries are worth spelling out. Both halves of
 * Chinese are here, with the Hong Kong tag reading as `zh-TW` because TMT has
 * one Traditional entry for the text endpoint. Pairs this vendor does not have
 * are refused before a request is made, so the relay walks on to the next engine
 * instead of spending a round trip on an answer that can only be a refusal.
 */
const MT_CODES = {
  zh: "zh",
  "zh-cn": "zh",
  "zh-hans": "zh",
  "zh-tw": "zh-TW",
  "zh-hk": "zh-TW",
  "zh-hant": "zh-TW",
  en: "en",
  ja: "ja",
  ko: "ko",
  fr: "fr",
  es: "es",
  it: "it",
  de: "de",
  ru: "ru",
  pt: "pt",
  vi: "vi",
  id: "id",
  th: "th",
  ms: "ms",
  tr: "tr",
  ar: "ar",
  hi: "hi",
};

/** The TMT code for one tag, or an empty string when it has no such language. */
export function mtLanguage(tag) {
  const lower = String(tag || "").trim().toLowerCase();
  if (!lower || lower === "auto") return "auto";
  return MT_CODES[lower] || MT_CODES[lower.split(/[-_]/)[0]] || "";
}

/** Whether this vendor is offered the pair at all. */
export function mtSupports(from, to) {
  return Boolean(mtLanguage(from) && mtLanguage(to));
}

const MT_ERRORS = {
  "AuthFailure.SignatureExpire": { code: "upstream_credentials", message: "腾讯云机器翻译凭证已过期。" },
  "AuthFailure.SecretIdNotFound": { code: "upstream_credentials", message: "腾讯云机器翻译 SecretId 不对。" },
  "AuthFailure.SignatureFailure": { code: "upstream_credentials", message: "腾讯云机器翻译签名没通过。" },
  "AuthFailure.UnauthorizedOperation": { code: "upstream_credentials", message: "这个腾讯云账号没有机器翻译权限。" },
  "FailedOperation.NoFreeAmount": { code: "upstream_limit", message: "腾讯云机器翻译的免费额度用完了。" },
  "FailedOperation.ServiceIsolate": { code: "upstream_limit", message: "腾讯云账号欠费，机器翻译已停用。" },
  "FailedOperation.UserNotRegistered": {
    code: "upstream_credentials",
    message: "这个腾讯云账号还没有开通机器翻译。",
  },
  FailedOperation: { code: "upstream_error", message: "机器翻译没能处理这次请求。" },
  InternalError: { code: "upstream_error", message: "机器翻译内部出错了，请稍后再试。" },
  InvalidParameter: { code: "upstream_error", message: "机器翻译不理解这次请求的参数。" },
  InvalidParameterValue: { code: "upstream_error", message: "机器翻译不理解这次请求的参数。" },
  LimitExceeded: { code: "upstream_limit", message: "翻译请求太频繁了，请稍后再试。" },
  RequestLimitExceeded: { code: "upstream_limit", message: "翻译请求太频繁了，请稍后再试。" },
  UnsupportedOperation: { code: "unsupported_language", message: "这个语言方向腾讯云机器翻译不支持。" },
};

const PREFIX_ERRORS = {
  AuthFailure: { code: "upstream_credentials", message: "腾讯云机器翻译没有通过认证。" },
  FailedOperation: { code: "upstream_error", message: "机器翻译没能处理这次请求。" },
  InternalError: { code: "upstream_error", message: "机器翻译内部出错了，请稍后再试。" },
  InvalidParameter: { code: "upstream_error", message: "机器翻译不理解这次请求的参数。" },
  LimitExceeded: { code: "upstream_limit", message: "翻译请求太频繁了，请稍后再试。" },
  RequestLimitExceeded: { code: "upstream_limit", message: "翻译请求太频繁了，请稍后再试。" },
  UnsupportedOperation: { code: "unsupported_language", message: "这个语言方向腾讯云机器翻译不支持。" },
};

function mtProblem(raw) {
  const known = MT_ERRORS[raw] || PREFIX_ERRORS[String(raw).split(".")[0]];
  if (known) return { ...known, upstream: raw };
  return { code: "upstream_error", message: `机器翻译返回错误 ${raw}。`, upstream: raw };
}

/**
 * Translates one text with TMT.
 *
 * `endpoint` and `region` only exist so a local mock can stand in while
 * developing; production leaves them unset.
 */
export async function translateWithTencentMt({
  fetchImpl,
  secretId,
  secretKey,
  text,
  from,
  to,
  endpoint,
  region,
  projectId = 0,
  now = () => Date.now(),
}) {
  if (!secretId || !secretKey) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置机器翻译的密钥。" };
  }
  const source = mtLanguage(from);
  const target = mtLanguage(to);
  if (!source || !target) {
    return { ok: false, code: "unsupported_language", message: "这个语言方向腾讯云机器翻译没有。" };
  }

  const url = endpoint || DEFAULT_TENCENT_MT_ENDPOINT;
  const payload = JSON.stringify({
    SourceText: text,
    Source: source,
    Target: target,
    ProjectId: projectId,
  });
  const headers = await tc3Headers({
    service: SERVICE,
    host: new URL(url).host,
    action: TENCENT_MT_ACTION,
    version: TENCENT_MT_VERSION,
    region: region || DEFAULT_TENCENT_MT_REGION,
    payload,
    secretId,
    secretKey,
    timestamp: now(),
  });

  let response;
  try {
    response = await fetchImpl(url, { method: "POST", headers, body: payload });
  } catch (error) {
    return { ok: false, code: "upstream_unreachable", message: `无法连接机器翻译服务：${error}` };
  }

  let data;
  try {
    data = JSON.parse(await response.text());
  } catch {
    return {
      ok: false,
      code: "upstream_error",
      message: `机器翻译返回了无法解析的内容（HTTP ${response.status}）。`,
    };
  }

  const refusal = data?.Response?.Error;
  if (refusal) return { ok: false, ...mtProblem(String(refusal.Code || "")) };

  const translation = typeof data?.Response?.TargetText === "string" ? data.Response.TargetText.trim() : "";
  if (!translation) {
    return { ok: false, code: "upstream_error", message: "机器翻译没有返回译文。" };
  }

  // The tag the caller asked with, rather than the vendor's own word for what it
  // found: this is a chain behind another service, and a card that suddenly
  // names a language by a code its own list does not hold would be worse than
  // one that says what was asked for.
  return {
    ok: true,
    from: String(from || "").trim() || "auto",
    to: String(to || "").trim(),
    translation,
  };
}
