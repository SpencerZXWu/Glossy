/**
 * MD5 (RFC 1321).
 *
 * The Workers runtime only ships WebCrypto, which has no MD5, and Baidu signs
 * every request with `md5(appid + q + salt + key)`, so the algorithm has to
 * come along.
 */

const SHIFTS = [
  7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
  14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21,
  6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/** `floor(abs(sin(i + 1)) * 2^32)`, as the reference implementation derives it. */
const SINES = new Uint32Array(64);
for (let i = 0; i < 64; i += 1) {
  SINES[i] = Math.floor(Math.abs(Math.sin(i + 1)) * 4294967296);
}

const encoder = new TextEncoder();

export function md5(input) {
  const bytes = typeof input === "string" ? encoder.encode(input) : input;
  const length = bytes.length;
  const padded = new Uint8Array((((length + 8) >> 6) + 1) << 6);
  padded.set(bytes);
  padded[length] = 0x80;

  const view = new DataView(padded.buffer);
  view.setUint32(padded.length - 8, (length << 3) >>> 0, true);
  view.setUint32(padded.length - 4, Math.floor(length / 536870912), true);

  let a0 = 0x67452301;
  let b0 = 0xefcdab89;
  let c0 = 0x98badcfe;
  let d0 = 0x10325476;

  const word = new Uint32Array(16);
  for (let chunk = 0; chunk < padded.length; chunk += 64) {
    for (let i = 0; i < 16; i += 1) word[i] = view.getUint32(chunk + i * 4, true);

    let a = a0;
    let b = b0;
    let c = c0;
    let d = d0;

    for (let i = 0; i < 64; i += 1) {
      let mixed;
      let index;
      if (i < 16) {
        mixed = (b & c) | (~b & d);
        index = i;
      } else if (i < 32) {
        mixed = (d & b) | (~d & c);
        index = (5 * i + 1) % 16;
      } else if (i < 48) {
        mixed = b ^ c ^ d;
        index = (3 * i + 5) % 16;
      } else {
        mixed = c ^ (b | ~d);
        index = (7 * i) % 16;
      }

      const sum = (a + mixed + SINES[i] + word[index]) >>> 0;
      const rotated = (sum << SHIFTS[i]) | (sum >>> (32 - SHIFTS[i]));
      a = d;
      d = c;
      c = b;
      b = (b + rotated) >>> 0;
    }

    a0 = (a0 + a) >>> 0;
    b0 = (b0 + b) >>> 0;
    c0 = (c0 + c) >>> 0;
    d0 = (d0 + d) >>> 0;
  }

  let digest = "";
  const out = new Uint8Array(4);
  const outView = new DataView(out.buffer);
  for (const value of [a0, b0, c0, d0]) {
    outView.setUint32(0, value, true);
    for (const byte of out) digest += byte.toString(16).padStart(2, "0");
  }
  return digest;
}
