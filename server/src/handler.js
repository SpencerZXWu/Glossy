/**
 * The HTTP surface. Kept free of Worker globals so `node --test` can drive it
 * with an in-memory store and a stubbed upstream.
 *
 * Nothing here knows which services the deployment holds keys for: the upstream
 * built by `createUpstream` reports that through its `configured` flag.
 */

const CLIENT_ID = /^[A-Za-z0-9_-]{8,64}$/;
const MAX_BODY = 64 * 1024;

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
  };
}

function dayKey(now) {
  return new Date(now).toISOString().slice(0, 10);
}

/**
 * 腾讯云函数 URL 会把整个路径透传过来（可能带着 /release 之类的发布前缀），
 * 所以只比对最后两段，前缀里有 V1 之类的字眼也不会误判。
 */
export function routeOf(pathname) {
  const parts = pathname.split("/").filter(Boolean);
  return parts.length >= 2 ? `/${parts.slice(-2).join("/")}` : pathname;
}

export function createHandler({ store, upstream, config, now = () => Date.now() }) {
  const limits = limitsFrom(config);

  return async function handle(request) {
    const url = new URL(request.url);
    const route = routeOf(url.pathname);
    const at = now();
    const day = dayKey(at);
    const minute = Math.floor(at / 60000);
    const ipHash = await hashIp(request.headers.get("CF-Connecting-IP"), config.IP_SALT || "");

    if (route === "/v1/health") {
      if (request.method !== "GET") return fail(405, "method_not_allowed", "只支持 GET。");
      return json({
        ok: true,
        service: "glossy-cloud",
        day,
        configured: Boolean(upstream.configured),
        vendors: upstream.vendors || [],
      });
    }

    if (route === "/v1/quota") {
      if (request.method !== "GET") return fail(405, "method_not_allowed", "只支持 GET。");
      const clientId = url.searchParams.get("client") || "";
      if (!CLIENT_ID.test(clientId)) return fail(400, "invalid_request", "缺少或非法的客户端标识。");
      const usage = await store.peek(day, { clientId, ipHash });
      return json({
        ok: true,
        day,
        limits: {
          charsPerClient: limits.charsPerClient,
          charsPerIp: limits.charsPerIp,
          charsTotal: limits.charsTotal,
          maxCharsPerRequest: limits.maxCharsPerRequest,
          requestsPerMinute: limits.requestsPerMinute,
        },
        usage,
        remaining: Math.max(0, limits.charsPerClient - usage.client),
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

    return json({
      ok: true,
      from: result.from,
      to: result.to,
      translation: result.translation,
      vendor: result.vendor || null,
      chars,
      usage: { client: reserved.used, remaining: reserved.remaining },
    });
  };
}
