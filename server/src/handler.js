/**
 * The HTTP surface. Kept free of Worker globals so `node --test` can drive it
 * with an in-memory store and a stubbed upstream.
 *
 * Nothing here knows which services the deployment holds keys for: the upstream
 * built by `createUpstream` reports that through its `configured` flag.
 */

import { isBaseCode } from "./rates.js";
import { noticeFor } from "./version.js";

const CLIENT_ID = /^[A-Za-z0-9_-]{8,64}$/;
const MAX_BODY = 64 * 1024;
// A screenshot goes through this route, so it is the only one that may be big.
const MAX_OCR_BODY = 4 * 1024 * 1024;
const BASE64 = /^[A-Za-z0-9+/\r\n]+={0,2}$/;

/** Decoded size of a base64 payload, counting the padding off. */
function base64Bytes(value) {
  const text = value.replace(/[\r\n]/g, "");
  const padding = text.endsWith("==") ? 2 : text.endsWith("=") ? 1 : 0;
  return Math.floor((text.length * 3) / 4) - padding;
}

function json(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json; charset=utf-8", "Cache-Control": "no-store" },
  });
}

function fail(status, code, message, extra = {}) {
  return json({ ok: false, code, message, ...extra }, status);
}

/** Hashes the caller's IP so the counters never hold an address in the clear. */
async function hashIp(ip, salt) {
  if (!ip) return "unknown";
  const bytes = new TextEncoder().encode(`${salt}:${ip}`);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .slice(0, 16)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function limitsFrom(env) {
  const number = (name, fallback) => {
    const value = Number.parseInt(env[name] ?? "", 10);
    return Number.isFinite(value) && value >= 0 ? value : fallback;
  };
  return {
    charsPerClient: number("DAILY_CHARS_PER_CLIENT", 20000),
    charsPerIp: number("DAILY_CHARS_PER_IP", 30000),
    // 百度翻译认证版每月 100 万字符，按天摊约 3.3 万；留些余量，别把月额度提前烧完。
    // 换成大模型上游时这个上限更要紧，按量计费的额度不像免费额度那样能重来。
    charsTotal: number("DAILY_CHARS_TOTAL", 30000),
    maxCharsPerRequest: number("MAX_CHARS_PER_REQUEST", 2000),
    requestsPerMinute: number("MAX_REQUESTS_PER_MINUTE", 30),
    // Reading a picture costs the operator one call whatever is on it, so it is
    // booked as a fixed number of characters: that keeps OCR inside the same
    // daily allowance as everything else, and keeps it from being a free way to
    // spend the upstream account's quota.
    ocrCharsPerRequest: number("OCR_CHARS_PER_REQUEST", 100),
    // 截图识别的上游是按次计费的月额度（腾讯云通用印刷体识别每月 1000 次），
    // 字符额度管不住它：一张图不管内容多少都算一次调用，所以要按次、按用户、
    // 按整月来限制，免得一个用户一个下午就把整个账号的月度额度用光。
    ocrPerClientMonth: number("OCR_PER_CLIENT_MONTH", 100),
    maxImageBytes: number("MAX_IMAGE_BYTES", 3 * 1024 * 1024),
  };
}

function dayKey(now) {
  return new Date(now).toISOString().slice(0, 10);
}

function monthKey(now) {
  return new Date(now).toISOString().slice(0, 7);
}

/**
 * 腾讯云函数 URL 会把整个路径透传过来（可能带着 /release 之类的发布前缀），
 * 所以只比对最后两段，前缀里有 V1 之类的字眼也不会误判。
 */
export function routeOf(pathname) {
  const parts = pathname.split("/").filter(Boolean);
  return parts.length >= 2 ? `/${parts.slice(-2).join("/")}` : pathname;
}

export function createHandler({ store, ocrStore, upstream, config, now = () => Date.now() }) {
  const limits = limitsFrom(config);

  return async function handle(request) {
    const url = new URL(request.url);
    const route = routeOf(url.pathname);
    const at = now();
    const day = dayKey(at);
    const month = monthKey(at);
    const minute = Math.floor(at / 60000);
    const ipHash = await hashIp(request.headers.get("CF-Connecting-IP"), config.IP_SALT || "");

    if (route === "/v1/health") {
      if (request.method !== "GET") return fail(405, "method_not_allowed", "只支持 GET。");
      return json({
        ok: true,
        service: "glossy-cloud",
        day,
        configured: Boolean(upstream.configured),
        ocr: Boolean(upstream.ocrConfigured),
        ocrVendor: upstream.ocrVendor || null,
        vendors: upstream.vendors || [],
      });
    }

    if (route === "/v1/quota") {
      if (request.method !== "GET") return fail(405, "method_not_allowed", "只支持 GET。");
      const clientId = url.searchParams.get("client") || "";
      if (!CLIENT_ID.test(clientId)) return fail(400, "invalid_request", "缺少或非法的客户端标识。");
      const usage = await store.peek(day, { clientId, ipHash });
      const readings = await ocrStore.peekOcr(month, { clientId });
      return json({
        ok: true,
        day,
        month,
        limits: {
          charsPerClient: limits.charsPerClient,
          charsPerIp: limits.charsPerIp,
          charsTotal: limits.charsTotal,
          maxCharsPerRequest: limits.maxCharsPerRequest,
          requestsPerMinute: limits.requestsPerMinute,
          ocrCharsPerRequest: limits.ocrCharsPerRequest,
          ocrPerClientMonth: limits.ocrPerClientMonth,
          maxImageBytes: limits.maxImageBytes,
        },
        usage: { ...usage, ocrMonth: readings.requests },
        remaining: Math.max(0, limits.charsPerClient - usage.client),
        remainingOcrMonth:
          limits.ocrPerClientMonth > 0
            ? Math.max(0, limits.ocrPerClientMonth - readings.requests)
            : null,
      });
    }

    if (route === "/v1/rates") {
      if (request.method !== "GET") return fail(405, "method_not_allowed", "只支持 GET。");
      const clientId = url.searchParams.get("client") || "";
      const base = url.searchParams.get("base") || "";
      if (!CLIENT_ID.test(clientId)) return fail(400, "invalid_request", "缺少或非法的客户端标识。");
      if (!isBaseCode(base)) return fail(400, "invalid_request", "需要三个字母的货币代码，例如 USD。");

      // Rates cost no characters, so the daily allowances are untouched: the
      // call is only booked against the per-minute limit, which keeps a card
      // that shows a conversion free for the user.
      const reserved = await store.reserve(day, { clientId, ipHash, chars: 0, limits, now: at, minute });
      if (!reserved.ok) {
        return fail(429, reserved.code, reserved.message, {
          retryAfter: reserved.retryAfter ?? null,
          limit: reserved.limit ?? null,
          remaining: reserved.remaining ?? 0,
        });
      }

      let result;
      try {
        result = await upstream.rates({ base });
      } catch (error) {
        return fail(502, "upstream_error", `汇率服务调用失败：${error}`);
      }
      if (!result.ok) {
        const status = result.code === "invalid_request" ? 400 : 502;
        return fail(status, result.code, result.message);
      }

      return json({
        ok: true,
        base: result.base,
        source: result.source,
        date: result.date ?? null,
        rates: result.rates,
      });
    }

    if (route === "/v1/ocr") {
      if (request.method !== "POST") return fail(405, "method_not_allowed", "只支持 POST。");

      const length = Number.parseInt(request.headers.get("Content-Length") ?? "", 10);
      if (Number.isFinite(length) && length > MAX_OCR_BODY) return fail(413, "too_long", "这张图太大了。");

      let payload;
      try {
        payload = await request.json();
      } catch {
        return fail(400, "invalid_request", "请求体不是合法的 JSON。");
      }
      if (!payload || typeof payload !== "object") return fail(400, "invalid_request", "请求体不是合法的 JSON。");

      const clientId = typeof payload.clientId === "string" ? payload.clientId : "";
      // A `data:` prefix is what a browser makes of a canvas; taking it off here
      // means the App never has to care which form it is holding.
      const image = String(payload.image || "").replace(/^data:image\/[a-z0-9.+-]+;base64,/i, "").replace(/[\r\n\s]/g, "");
      const language = typeof payload.language === "string" ? payload.language : "";

      if (!CLIENT_ID.test(clientId)) return fail(400, "invalid_request", "缺少或非法的客户端标识。");
      if (!image) return fail(400, "invalid_request", "没有收到要识别的图片。");
      if (!BASE64.test(image)) return fail(400, "invalid_request", "图片不是合法的 base64。");
      const bytes = base64Bytes(image);
      if (bytes > limits.maxImageBytes) {
        return fail(413, "too_long", "这张图太大了，请截小一点。", {
          limit: limits.maxImageBytes,
          bytes,
        });
      }

      if (!upstream.ocrConfigured) return fail(503, "not_configured", "服务端还没有配置识图密钥。");

      // 先记月度次数再记账：读一张图在上游就是一次调用，字符额度拦不住它。
      // 上游调用失败会把这一次还回来，所以只有真的识别出结果才消耗次数。
      const readings = await ocrStore.reserveOcr(month, {
        clientId,
        limit: limits.ocrPerClientMonth,
      });
      if (!readings.ok) {
        return fail(429, readings.code, readings.message, {
          limit: readings.limit ?? null,
          remaining: 0,
        });
      }

      // Same allowance as a translation, booked before the call so a screenshot
      // cannot be used to spend the upstream account outside the daily limits.
      const chars = limits.ocrCharsPerRequest;
      const reserved = await store.reserve(day, { clientId, ipHash, chars, limits, now: at, minute });
      if (!reserved.ok) {
        await ocrStore.refundOcr(month, { clientId });
        return fail(429, reserved.code, reserved.message, {
          retryAfter: reserved.retryAfter ?? null,
          limit: reserved.limit ?? null,
          remaining: reserved.remaining ?? 0,
        });
      }

      let result;
      try {
        result = await upstream.ocr({ image, language });
      } catch (error) {
        await store.refund(day, { clientId, ipHash, chars });
        await ocrStore.refundOcr(month, { clientId });
        return fail(502, "upstream_error", `识图服务调用失败：${error}`);
      }

      if (!result.ok) {
        await store.refund(day, { clientId, ipHash, chars });
        await ocrStore.refundOcr(month, { clientId });
        const status =
          result.code === "not_configured" ? 503 : result.code === "no_text" ? 422 : result.code === "upstream_limit" ? 429 : 502;
        // The vendor's own error code rides along: a refusal that maps onto a
        // generic relay code is otherwise impossible to tell apart from the
        // hundred others behind it, and OCR is the one route whose failures are
        // usually an account setup problem rather than a broken request.
        return fail(status, result.code, result.message, { upstream: result.upstream ?? null });
      }

      return json({
        ok: true,
        text: result.text,
        language: result.language || null,
        chars,
        usage: {
          client: reserved.used,
          remaining: reserved.remaining,
          ocrMonth: readings.used,
          remainingOcrMonth: readings.remaining,
        },
      });
    }

    if (route !== "/v1/translate") {
      return fail(404, "not_found", "没有这个接口。");
    }
    if (request.method !== "POST") {
      return fail(405, "method_not_allowed", "只支持 POST。");
    }

    const length = Number.parseInt(request.headers.get("Content-Length") ?? "", 10);
    if (Number.isFinite(length) && length > MAX_BODY) return fail(413, "too_long", "请求体太大了。");

    let payload;
    try {
      payload = await request.json();
    } catch {
      return fail(400, "invalid_request", "请求体不是合法的 JSON。");
    }
    if (!payload || typeof payload !== "object") return fail(400, "invalid_request", "请求体不是合法的 JSON。");

    const text = typeof payload.text === "string" ? payload.text : "";
    const clientId = typeof payload.clientId === "string" ? payload.clientId : "";
    const from = typeof payload.from === "string" && payload.from ? payload.from : "auto";
    const to = typeof payload.to === "string" ? payload.to : "";
    // Which vendor the user picked in the App. An unknown name is not an error:
    // the upstream falls back to the order the deployment was configured with.
    const vendor = typeof payload.vendor === "string" ? payload.vendor.trim().toLowerCase() : "";
    // The build the App is from, from 2.1.1 onwards. Older builds send nothing,
    // and nothing means "leave it alone": they cannot read a notice anyway.
    const appVersion = typeof payload.appVersion === "string" ? payload.appVersion : "";

    if (!text.trim()) return fail(400, "invalid_request", "没有要翻译的内容。");
    if (!CLIENT_ID.test(clientId)) return fail(400, "invalid_request", "缺少或非法的客户端标识。");
    if (!upstream.isLanguageTag(to) || !upstream.isLanguageTag(from)) {
      return fail(400, "unsupported_language", "这个语言方向不支持翻译。");
    }

    const chars = [...text].length;
    if (chars > limits.maxCharsPerRequest) {
      return fail(
        413,
        "too_long",
        `单次最多翻译 ${limits.maxCharsPerRequest} 个字符，这次是 ${chars} 个。`,
        { limit: limits.maxCharsPerRequest, chars },
      );
    }

    const reserved = await store.reserve(day, { clientId, ipHash, chars, limits, now: at, minute });
    if (!reserved.ok) {
      return fail(429, reserved.code, reserved.message, {
        retryAfter: reserved.retryAfter ?? null,
        limit: reserved.limit ?? null,
        remaining: reserved.remaining ?? 0,
      });
    }

    let result;
    try {
      result = await upstream.translate({ text, from, to, vendor });
    } catch (error) {
      await store.refund(day, { clientId, ipHash, chars });
      return fail(502, "upstream_error", `翻译服务调用失败：${error}`);
    }

    if (!result.ok) {
      await store.refund(day, { clientId, ipHash, chars });
      const status = result.code === "not_configured" ? 503 : result.code === "unsupported_language" ? 400 : 502;
      return fail(status, result.code, result.message);
    }

    // Something to tell this App about itself — an update it should take, or one
    // line for everyone. Off unless the deployment sets MIN_VERSION or
    // ANNOUNCEMENT, and never a reason to refuse: the translation is returned
    // either way, and a build too old to know the field simply ignores it.
    const notice = noticeFor({
      appVersion,
      minVersion: config.MIN_VERSION,
      announcement: config.ANNOUNCEMENT,
    });

    return json({
      ok: true,
      from: result.from,
      to: result.to,
      translation: result.translation,
      vendor: result.vendor || null,
      // Which vendors were walked past on the way here, so the App can say why
      // the engine it asked for did not answer instead of quietly showing
      // another one's work as its own.
      attempts: result.attempts || [],
      chars,
      usage: { client: reserved.used, remaining: reserved.remaining },
      ...(notice ? { notice } : {}),
    });
  };
}
