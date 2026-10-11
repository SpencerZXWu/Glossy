/**
 * The TC3 signature every Tencent Cloud API call is made with.
 *
 * It is here rather than beside either caller because two products now use it —
 * the reader (`tencent-ocr.js`) and the translator (`tencent-mt.js`) — and a
 * second copy of a signing routine is a second place for the two to drift
 * apart. Nothing below knows what a request means: it is handed the service, the
 * action and the payload, and it answers the headers that carry them.
 */

export const CONTENT_TYPE = "application/json; charset=utf-8";

const encoder = new TextEncoder();
const HEX = Array.from({ length: 256 }, (_, index) => index.toString(16).padStart(2, "0"));

function hex(bytes) {
  let out = "";
  for (const byte of bytes) out += HEX[byte];
  return out;
}

async function sha256Hex(text) {
  const digest = await crypto.subtle.digest("SHA-256", encoder.encode(text));
  return hex(new Uint8Array(digest));
}

async function hmacSha256(key, text) {
  const imported = await crypto.subtle.importKey("raw", key, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  const mac = await crypto.subtle.sign("HMAC", imported, encoder.encode(text));
  return new Uint8Array(mac);
}

/**
 * The headers one Tencent API call needs, signature and all.
 *
 * Timestamps are the caller's, so a test can pin them and compare the signature
 * against a known one.
 */
export async function tc3Headers({ service, host, action, version, region, payload, secretId, secretKey, timestamp }) {
  const seconds = Math.floor(timestamp / 1000);
  const date = new Date(seconds * 1000).toISOString().slice(0, 10);
  const signedHeaders = "content-type;host;x-tc-action";
  const canonical = [
    "POST",
    "/",
    "",
    `content-type:${CONTENT_TYPE}\nhost:${host}\nx-tc-action:${action.toLowerCase()}\n`,
    signedHeaders,
    await sha256Hex(payload),
  ].join("\n");

  const scope = `${date}/${service}/tc3_request`;
  const toSign = ["TC3-HMAC-SHA256", seconds, scope, await sha256Hex(canonical)].join("\n");

  const secretDate = await hmacSha256(encoder.encode(`TC3${secretKey}`), date);
  const secretService = await hmacSha256(secretDate, service);
  const secretSigning = await hmacSha256(secretService, "tc3_request");
  const signature = hex(await hmacSha256(secretSigning, toSign));

  return {
    Authorization: `TC3-HMAC-SHA256 Credential=${secretId}/${scope}, SignedHeaders=${signedHeaders}, Signature=${signature}`,
    "Content-Type": CONTENT_TYPE,
    Host: host,
    "X-TC-Action": action,
    "X-TC-Timestamp": String(seconds),
    "X-TC-Version": version,
    ...(region ? { "X-TC-Region": region } : {}),
  };
}
