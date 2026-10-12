/**
 * The word lookup behind `/v1/dict`.
 *
 * The App's word card used to be filled from a public endpoint in the reader's
 * own network that answers in anything from half a second to twenty, and is
 * sometimes unreachable there. This is the same kind of data asked for from the
 * relay instead: the deployment's network reaches it in a fraction of a second,
 * and the reader never waits on a lookup that was never going to arrive.
 *
 * It needs no credentials. Unlike the translation vendors and the screenshot
 * readers, the dictionary endpoint is public, so a deployment offers this
 * whether or not it holds keys for anything else — and the word itself is all
 * that goes out, no account, no installation id.
 *
 * `endpoint` exists so a test, or a deployment behind its own proxy, can stand
 * in for the public address.
 */

const ENDPOINT = "https://dict.youdao.com/jsonapi";

/**
 * Which dictionaries to ask for: the English–Chinese entries, the example
 * sentences, and the synonyms. The endpoint answers with everything it has
 * unless it is told which parts to send.
 */
const DICTS = JSON.stringify({
  count: 99,
  dicts: [["ec"], ["blng_sents_part"], ["syno"]],
});

/** The endpoint answers in about a tenth of a second; this is only a guard. */
const TIMEOUT_MS = 5000;

/** The same caps the card draws with, so nothing is sent that cannot be shown. */
const MAX_MEANINGS = 4;
const MAX_DEFINITIONS = 4;
const MAX_SYNONYMS = 6;
const MAX_EXAMPLE = 240;

/** The longest word worth asking about: a compound, not a sentence. */
export const MAX_WORD_CHARS = 64;

/**
 * What counts as a word here.
 *
 * Letters from any script, plus the marks and joiners inside them, and the
 * hyphens, apostrophes and spaces an English or French headword brings with it.
 * Anything longer than a compound is a selection, not a word.
 */
const WORD = /^[\p{L}\p{M}][\p{L}\p{M}'’\- ]{0,63}$/u;

export function isWordLookup(value) {
  const word = String(value ?? "").trim();
  return word.length > 0 && word.length <= MAX_WORD_CHARS && WORD.test(word);
}

/** The address the lookup uses, unless the deployment named its own. */
export function dictEndpoint(config = {}) {
  return String(config.YOUDAO_DICT_ENDPOINT || "").trim() || ENDPOINT;
}

/** One `int. 喂，你好…` line, split into its part of speech and its senses. */
export function splitEntry(line) {
  const text = String(line ?? "").trim();
  // `n.`, `int.`, `vt.`, `adj.` and the like, always at the front and always
  // followed by a space: a definition that begins with an abbreviation the
  // dictionary does not use keeps its own words instead of losing them.
  const match = /^([A-Za-z]{1,6}(?:\.[A-Za-z]{1,6}){0,3}\.)\s+(.+)$/.exec(text);
  const part = match ? match[1].toLowerCase() : "";
  const rest = match ? match[2] : text;
  const senses = rest
    .split(/[；;]/)
    .map((sense) => sense.trim())
    .filter((sense) => sense.length > 0)
    .slice(0, MAX_DEFINITIONS);
  return { part, senses };
}

/** Strips the `<b>` the example sentences come wrapped in, and any other tag. */
function plainText(value) {
  return String(value ?? "")
    .replace(/<[^>]*>/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

/**
 * Turns one answer from the endpoint into the shape the card reads.
 *
 * Pure, so the mapping can be tested against a stored answer without a network
 * call: everything this returns is either present in `data` or left empty.
 */
export function parseDict(data) {
  const entry = data?.ec?.word?.[0] ?? null;
  const phonetics = {
    uk: plainText(entry?.ukphone),
    us: plainText(entry?.usphone),
  };

  // The parts of speech are kept in the order they were given, with the senses
  // of a repeated one added to the part already there: the endpoint sometimes
  // lists `n.` twice rather than once with two lines.
  const meanings = [];
  for (const line of entry?.trs ?? []) {
    const text = line?.tr?.[0]?.l?.i?.[0];
    const { part, senses } = splitEntry(text);
    if (!senses.length) continue;
    const existing = meanings.find((meaning) => meaning.partOfSpeech === part);
    if (existing) {
      for (const sense of senses) {
        if (existing.definitions.length < MAX_DEFINITIONS) existing.definitions.push(sense);
      }
      continue;
    }
    if (meanings.length >= MAX_MEANINGS) break;
    meanings.push({ partOfSpeech: part, definitions: senses });
  }

  const synonyms = [];
  for (const row of data?.syno?.synos ?? []) {
    for (const item of row?.syno?.ws ?? []) {
      const word = plainText(item?.w);
      if (!word || synonyms.length >= MAX_SYNONYMS) continue;
      if (!synonyms.some((kept) => kept.toLowerCase() === word.toLowerCase())) synonyms.push(word);
    }
  }

  const pair = data?.blng_sents_part?.["sentence-pair"]?.[0] ?? null;
  const example = plainText(pair?.sentence).slice(0, MAX_EXAMPLE);

  // The endpoint answers with an empty shell for a word it does not know, and
  // with an example sentence anyway — one that happens to contain the word. A
  // card the dictionary has nothing for is a card with nothing on it, not a
  // card with an example from the end of the entry that was not there.
  const found = Boolean(phonetics.uk || phonetics.us || meanings.length);

  return {
    word: plainText(entry?.["return-phrase"]?.l?.i ?? data?.input),
    phonetic: phonetics.us || phonetics.uk || null,
    phonetics,
    meanings,
    example: found ? example || null : null,
    synonyms: found ? synonyms : [],
    // "found" means there is something to show rather than that it replied.
    found,
  };
}

/**
 * Looks one word up.
 *
 * An answer the endpoint does not have, a network that does not reach it and a
 * reply that is not JSON all come back the same way: `found: false` with the
 * word echoed, because a card with one line missing is still a card. A
 * deployment-side failure (`ok: false`) is kept apart from that, so the route
 * can answer an error when the lookup itself could not be made.
 */
export async function lookupWord({ fetchImpl, word, endpoint, timeoutMs = TIMEOUT_MS } = {}) {
  const query = String(word ?? "").trim();
  if (!isWordLookup(query)) {
    return { ok: false, code: "invalid_request", message: "这里只查一个单词。" };
  }

  const address = `${endpoint || ENDPOINT}?q=${encodeURIComponent(query)}&dicts=${encodeURIComponent(DICTS)}`;
  let response;
  try {
    response = await fetchImpl(address, {
      headers: { Accept: "application/json" },
      signal: typeof AbortSignal.timeout === "function" ? AbortSignal.timeout(timeoutMs) : undefined,
    });
  } catch (error) {
    return { ok: false, code: "upstream_unreachable", message: `词典服务连不上：${error}` };
  }

  if (!response.ok) {
    return { ok: false, code: "upstream_error", message: `词典服务返回 HTTP ${response.status}。` };
  }

  let data;
  try {
    data = await response.json();
  } catch {
    return { ok: false, code: "upstream_error", message: "词典服务返回的不是 JSON。" };
  }

  return { ok: true, source: "youdao", ...parseDict(data) };
}
