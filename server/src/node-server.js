/**
 * Turns a plain Node `http` server into the Fetch-style `Request`/`Response`
 * the handler speaks, so the same rules run on SCF, on a container and locally.
 */

const DEFAULT_MAX_BODY = 64 * 1024;

/** Node's own headers must not be copied onto a `Request`; undici rejects some. */
const SKIP_HEADERS = new Set([
  "connection",
  "content-length",
  "expect",
  "host",
  "keep-alive",
  "proxy-connection",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
]);

export function createRequestListener({
  handler,
  clientIpHeaders = ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"],
  maxBody = DEFAULT_MAX_BODY,
  log = console,
}) {
  let ipReported = false;

  function readBody(request) {
    return new Promise((resolve, reject) => {
      const chunks = [];
      let size = 0;
      let tooLong = false;
      let settled = false;
      const done = (fn, value) => {
        if (settled) return;
        settled = true;
        fn(value);
      };
      const tooLongError = () => Object.assign(new Error("请求体太大了。"), { code: "too_long" });

      request.on("data", (chunk) => {
        size += chunk.length;
        if (size > maxBody) {
          // 继续读下去只是把数据丢掉，连接保持健康，客户端能收到干净的 413。
          // 只有明显在灌数据的时候才直接掐断连接。
          if (size > maxBody * 4) {
            done(reject, tooLongError());
            request.destroy();
            return;
          }
          if (!tooLong) {
            tooLong = true;
            chunks.length = 0;
          }
          return;
        }
        chunks.push(chunk);
      });
      request.on("end", () => done(tooLong ? reject : resolve, tooLong ? tooLongError() : Buffer.concat(chunks)));
      request.on("error", (error) => done(reject, error));
    });
  }

  return async function listener(request, response) {
    response.on("error", () => {});
    try {
      const url = new URL(request.url || "/", `http://${request.headers.host || "localhost"}`);
      const headers = new Headers();
      for (const [name, value] of Object.entries(request.headers)) {
        const lower = name.toLowerCase();
        if (SKIP_HEADERS.has(lower)) continue;
        if (Array.isArray(value)) value.forEach((entry) => headers.append(lower, entry));
        else if (value !== undefined) headers.set(lower, value);
      }

      const ip = clientIpFrom(request.headers, clientIpHeaders);
      if (ip) headers.set("cf-connecting-ip", ip);
      if (!ipReported) {
        ipReported = true;
        log.log(ip ? `glossy-cloud: 已从 ${ipHeaderUsed(request.headers, clientIpHeaders)} 取到调用方地址。` : `glossy-cloud: 没有取到调用方地址（试过 ${clientIpHeaders.join(", ")}），按 IP 的额度会合成一个桶。`);
      }

      const hasBody = request.method !== "GET" && request.method !== "HEAD";
      let body;
      if (hasBody) body = await readBody(request);

      const result = await handler(
        new Request(url, {
          method: request.method,
          headers,
          body,
        }),
      );

      response.writeHead(result.status, Object.fromEntries(result.headers));
      response.end(Buffer.from(await result.arrayBuffer()));
    } catch (error) {
      const tooLong = error?.code === "too_long";
      response.writeHead(tooLong ? 413 : 500, { "Content-Type": "application/json; charset=utf-8" });
      response.end(
        JSON.stringify(
          tooLong
            ? { ok: false, code: "too_long", message: "请求体太大了。" }
            : { ok: false, code: "internal_error", message: "服务端出错了。" },
        ),
      );
      if (!tooLong) log.error("glossy-cloud:", error);
    }
  };
}

function ipHeaderUsed(headers, names) {
  return names.find((name) => headers[name.toLowerCase()]) ?? names[0];
}

/**
 * 函数 URL 网关在 X-Forwarded-For 末尾追加固定两段：<边缘节点> 和 <内网跳转>。
 * 这两段每个请求都不一样，按它们分桶等于完全不分桶，所以要跳过；往前数第一个
 * 才是网关看到的调用方地址。客户端自己塞进来的前缀因此也不会被当成真实地址。
 */
const GATEWAY_APPENDED_HOPS = 2;

export function clientIpFrom(headers, names = ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"]) {
  for (const name of names) {
    const raw = headers[name.toLowerCase()];
    if (!raw) continue;
    const parts = (Array.isArray(raw) ? raw.join(",") : String(raw))
      .split(",")
      .map((part) => part.trim())
      .filter(Boolean);
    if (parts.length) return parts[Math.max(0, parts.length - 1 - GATEWAY_APPENDED_HOPS)];
  }
  return "";
}
