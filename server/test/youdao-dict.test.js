import assert from "node:assert/strict";
import test from "node:test";

import { dictEndpoint, isWordLookup, lookupWord, parseDict, splitEntry } from "../src/youdao-dict.js";

/** A trimmed answer from the public endpoint, kept as it came. */
const HELLO = {
  input: "hello",
  ec: {
    word: [
      {
        usphone: "həˈloʊ",
        ukphone: "həˈləʊ",
        "return-phrase": { l: { i: "hello" } },
        trs: [
          { tr: [{ l: { i: ["int. 喂，你好（用于问候或打招呼）；喂，你好（打电话时的招呼语）"] } }] },
          { tr: [{ l: { i: ["n. 招呼，问候"] } }] },
          { tr: [{ l: { i: ["v. 说（或大声说）“喂”；打招呼"] } }] },
        ],
      },
    ],
  },
  blng_sents_part: {
    "sentence-pair": [
      { sentence: "'Hello, Paul,' they chorused.", "sentence-translation": "“你好，保罗。”他们齐声问候道。" },
    ],
  },
  syno: { synos: [{ syno: { pos: "int.", ws: [{ w: "hallo" }, { w: "holloo" }], tran: "喂；哈罗" } }] },
};

/** The shell the endpoint answers with for a word it does not have. */
const UNKNOWN = { input: "qwertyzx", ec: { word: [{ "return-phrase": { l: { i: "qwertyzx" } }, trs: [] }] } };

test("splits a part of speech off an entry", () => {
  assert.deepEqual(splitEntry("int. 喂，你好；喂"), { part: "int.", senses: ["喂，你好", "喂"] });
  assert.deepEqual(splitEntry("vt. 起草"), { part: "vt.", senses: ["起草"] });
});

test("keeps an entry that has no part of speech of its own", () => {
  assert.deepEqual(splitEntry("喂，你好"), { part: "", senses: ["喂，你好"] });
  assert.deepEqual(splitEntry("It is. A sentence."), { part: "", senses: ["It is. A sentence."] });
  assert.deepEqual(splitEntry(""), { part: "", senses: [] });
});

test("caps the senses one part of speech may carry", () => {
  assert.equal(splitEntry("n. 一；二；三；四；五；六").senses.length, 4);
});

test("maps an answer onto the shape the card reads", () => {
  const entry = parseDict(HELLO);
  assert.equal(entry.word, "hello");
  assert.equal(entry.phonetic, "həˈloʊ");
  assert.deepEqual(entry.phonetics, { uk: "həˈləʊ", us: "həˈloʊ" });
  assert.deepEqual(
    entry.meanings.map((meaning) => meaning.partOfSpeech),
    ["int.", "n.", "v."],
  );
  assert.deepEqual(entry.meanings[0].definitions, ["喂，你好（用于问候或打招呼）", "喂，你好（打电话时的招呼语）"]);
  assert.equal(entry.example, "'Hello, Paul,' they chorused.");
  assert.deepEqual(entry.synonyms, ["hallo", "holloo"]);
  assert.equal(entry.found, true);
});

test("adds a repeated part of speech to the one already there", () => {
  const entry = parseDict({
    ec: { word: [{ trs: [{ tr: [{ l: { i: ["n. 一"] } }] }, { tr: [{ l: { i: ["n. 二"] } }] }] }] },
  });
  assert.equal(entry.meanings.length, 1);
  assert.deepEqual(entry.meanings[0].definitions, ["一", "二"]);
});

test("says nothing was found for a word the endpoint does not have", () => {
  const entry = parseDict(UNKNOWN);
  assert.equal(entry.found, false);
  assert.deepEqual(entry.meanings, []);
  assert.equal(entry.phonetic, null);
  assert.equal(entry.example, null);
});

test("leaves out the example the endpoint sends for a word it does not know", () => {
  // 你好 comes back with no entry of its own and a sentence that happens to
  // contain it, which is not an example of anything.
  const entry = parseDict({
    input: "你好",
    ec: { word: [{ "return-phrase": { l: { i: "你好" } }, trs: [] }] },
    blng_sents_part: { "sentence-pair": [{ sentence: "Hello, John, how are you?" }] },
    syno: { synos: [{ syno: { ws: [{ w: "hi" }] } }] },
  });
  assert.equal(entry.found, false);
  assert.equal(entry.example, null);
  assert.deepEqual(entry.synonyms, []);
});

test("survives an answer that is not shaped like one", () => {
  for (const data of [null, {}, [], { ec: { word: [] } }, { ec: { word: [{}] } }]) {
    const entry = parseDict(data);
    assert.equal(entry.found, false, JSON.stringify(data));
    assert.deepEqual(entry.meanings, []);
  }
});

test("strips the tags the example sentences come wrapped in", () => {
  const entry = parseDict({
    ec: { word: [{ usphone: "x", trs: [{ tr: [{ l: { i: ["n. 一"] } }] }] }] },
    blng_sents_part: { "sentence-pair": [{ sentence: "the <b>word</b> in a sentence" }] },
  });
  assert.equal(entry.example, "the word in a sentence");
});

test("accepts a word and refuses a selection", () => {
  for (const word of ["hello", "hello-world", "don't", "café", "你好", "bonjour le monde", "naïve"]) {
    assert.equal(isWordLookup(word), true, word);
  }
  for (const word of ["", "   ", "hello, world", "a\nb", "1st", "-hello", "x".repeat(65)]) {
    assert.equal(isWordLookup(word), false, word);
  }
});

test("has a working address unless the deployment named one", () => {
  assert.match(dictEndpoint({}), /^https:\/\//);
  assert.equal(dictEndpoint({ YOUDAO_DICT_ENDPOINT: "https://example.test/dict" }), "https://example.test/dict");
  assert.equal(dictEndpoint({ YOUDAO_DICT_ENDPOINT: "  " }), dictEndpoint({}));
});

test("asks the endpoint for the word and answers with the entry", async () => {
  const asked = [];
  const result = await lookupWord({
    fetchImpl: async (address) => {
      asked.push(address);
      return new Response(JSON.stringify(HELLO), { headers: { "Content-Type": "application/json" } });
    },
    word: " hello ",
  });
  assert.equal(result.ok, true);
  assert.equal(result.source, "youdao");
  assert.equal(result.word, "hello");
  assert.match(asked[0], /^https:\/\/dict\.youdao\.com\/jsonapi\?q=hello&dicts=/);
  assert.match(decodeURIComponent(asked[0]), /"dicts":\[\["ec"\]/);
});

test("answers an error rather than a word when the lookup cannot be made", async () => {
  const unreachable = await lookupWord({
    fetchImpl: async () => {
      throw new Error("getaddrinfo ENOTFOUND");
    },
    word: "hello",
  });
  assert.equal(unreachable.ok, false);
  assert.equal(unreachable.code, "upstream_unreachable");

  const refused = await lookupWord({
    fetchImpl: async () => new Response("nope", { status: 403 }),
    word: "hello",
  });
  assert.equal(refused.ok, false);
  assert.equal(refused.code, "upstream_error");

  const notJson = await lookupWord({
    fetchImpl: async () => new Response("<html></html>", { status: 200 }),
    word: "hello",
  });
  assert.equal(notJson.ok, false);
  assert.equal(notJson.code, "upstream_error");
});

test("refuses to look up something that is not a word", async () => {
  const result = await lookupWord({
    fetchImpl: async () => {
      throw new Error("this must not be called");
    },
    word: "hello, world",
  });
  assert.equal(result.ok, false);
  assert.equal(result.code, "invalid_request");
});
