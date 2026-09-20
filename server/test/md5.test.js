import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import { md5 } from "../src/md5.js";

const VECTORS = [
  ["", "d41d8cd98f00b204e9800998ecf8427e"],
  ["a", "0cc175b9c0f1b6a831c399e269772661"],
  ["abc", "900150983cd24fb0d6963f7d28e17f72"],
  ["message digest", "f96b697d7cb7938d525a2f31aaf161d0"],
  ["abcdefghijklmnopqrstuvwxyz", "c3fcd3d76192e4007dfb496cca67e13b"],
  ["12345678901234567890123456789012345678901234567890123456789012345678901234567890", "57edf4a22be3c955ac49da2e2107b67a"],
  ["The quick brown fox jumps over the lazy dog", "9e107d9d372bb6826bd81d3542a419d6"],
];

test("known RFC 1321 vectors", () => {
  for (const [input, expected] of VECTORS) {
    assert.equal(md5(input), expected, `md5(${JSON.stringify(input)})`);
  }
});

test("agrees with node's hash for assorted inputs", () => {
  const cases = [
    "x".repeat(55),
    "x".repeat(56),
    "x".repeat(57),
    "x".repeat(63),
    "x".repeat(64),
    "x".repeat(65),
    "x".repeat(1000),
    "中文测试 — 长度不等于字节数",
    "emoji 😀🎉 and ünïcödé",
    "mixed 中文 with ascii text 12345",
  ];
  for (const input of cases) {
    const expected = createHash("md5").update(input, "utf8").digest("hex");
    assert.equal(md5(input), expected, `md5(${JSON.stringify(input.slice(0, 20))}…)`);
  }
});

test("hashes bytes and strings identically", () => {
  assert.equal(md5(new TextEncoder().encode("abc")), md5("abc"));
});
