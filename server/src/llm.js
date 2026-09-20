/**
 * The LLM upstream: one OpenAI compatible chat completion.
 *
 * Unlike Baidu this is not a translation API but a general model, so the prompt
 * carries the whole contract: translate, keep the formatting, answer with the
 * detected language and the translation as JSON. Any service that speaks
 * `/chat/completions` works - Zhipu, DeepSeek, Moonshot, Qwen, OpenAI itself -
 * which is what makes it possible to run this deployment on an account Glossy
 * pays for, or on a model host in the same country.
 */

export const DEFAULT_LLM_ENDPOINT = "https://open.bigmodel.cn/api/paas/v4/chat/completions";
export const DEFAULT_LLM_MODEL = "glm-4-flash";

/** Tags the App sends, as names a model reads more reliably. */
const LANGUAGE_NAMES = {
  zh: "Chinese",
  cht: "Traditional Chinese",
  en: "English",
  ja: "Japanese",
  ko: "Korean",
  fr: "French",
  de: "German",
  es: "Spanish",
  pt: "Portuguese",
  it: "Italian",
  ru: "Russian",
  ar: "Arabic",
  th: "Thai",
  vi: "Vietnamese",
  id: "Indonesian",
  ms: "Malay",
  hi: "Hindi",
  nl: "Dutch",
  pl: "Polish",
  tr: "Turkish",
  uk: "Ukrainian",
};

function languageName(tag) {
  const lower = String(tag || "").trim().toLowerCase();
  if (!lower || lower === "auto") return "the language you detect";
  const base = lower.split(/[-_]/)[0];
  if (base === "zh") return /tw|hk|hant/.test(lower) ? "Traditional Chinese" : "Chinese";
  return LANGUAGE_NAMES[base] || lower;
}

function buildPrompt(text, from, to) {
  return [
    `Translate the text below from ${languageName(from)} into ${languageName(to)}.`,
    "Answer with JSON only, no code fence and no explanation:",
    '{"sourceLang":"<the language the text is in, as a BCP 47 tag>","translation":"<the translation>"}',
    "Keep the line breaks and the punctuation of the original. Translate names and",
    "technical terms the way a native speaker would read them, and do not add notes.",
    "",
    "Text:",
    text,
  ].join("\n");
}

/** Models like to wrap JSON in a fence even when told not to. */
export function extractJson(raw) {
  const text = String(raw || "").trim();
  const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i);
  return fenced ? fenced[1].trim() : text;
}

const ERROR_MESSAGES = {
  401: "服务端的翻译账号未通过认证。",
  403: "服务端的翻译账号没有权限。",
  404: "服务端配置的翻译模型不存在。",
  429: "翻译服务调用过于频繁或额度已用尽，请稍后再试。",
};

/**
 * Asks the model for one translation and answers with either the translation or
 * a code the App can turn into its own wording.
 *
 * `endpoint` and `model` only exist so a local mock can stand in while
 * developing; production leaves them unset.
 */
export async function translateWithLlm({
  fetchImpl,
  key,
  endpoint,
  model,
  text,
  from,
  to,
  temperature = 0.2,
  timeoutMs = 20000,
}) {
  if (!key) {
    return { ok: false, code: "not_configured", message: "服务端还没有配置大模型翻译的密钥。" };
  }

  const body = JSON.stringify({
    model: model || DEFAULT_LLM_MODEL,
    temperature,
    messages: [
      { role: "system", content: "You are a careful professional translator." },
      { role: "user", content: buildPrompt(text, from, to) },
    ],
  });

  let response;
  try {
    response = await fetchImpl(endpoint || DEFAULT_LLM_ENDPOINT, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${key}`,
      },
      body,
      signal: AbortSignal.timeout(timeoutMs),
    });
  } catch (error) {
    return { ok: false, code: "upstream_unreachable", message: `无法连接翻译服务：${error}` };
  }

  const raw = await response.text();
  if (!response.ok) {
    return {
      ok: false,
      code: response.status === 401 || response.status === 403 ? "upstream_credentials" : "upstream_error",
      message: ERROR_MESSAGES[response.status] || `翻译服务返回 HTTP ${response.status}。`,
      upstream: String(response.status),
    };
  }

  let data;
  try {
    data = JSON.parse(raw);
  } catch {
    return { ok: false, code: "upstream_error", message: "翻译服务返回了无法解析的内容。" };
  }

  const content = data?.choices?.[0]?.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    return { ok: false, code: "upstream_error", message: "翻译服务没有返回译文。" };
  }

  let parsed;
  try {
    parsed = JSON.parse(extractJson(content));
  } catch {
    return { ok: false, code: "upstream_error", message: "翻译服务没有按要求返回 JSON。" };
  }

  const translation = typeof parsed?.translation === "string" ? parsed.translation.trim() : "";
  if (!translation) {
    return { ok: false, code: "upstream_error", message: "翻译服务没有返回译文。" };
  }

  return {
    ok: true,
    from: typeof parsed.sourceLang === "string" && parsed.sourceLang ? parsed.sourceLang : from || "auto",
    to,
    translation,
  };
}
