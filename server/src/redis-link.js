/**
 * A very small Redis client, just enough for the quota counters.
 *
 * The service is deployed as one bundled file pasted into the SCF console, so a
 * client library would have to be bundled with it and would multiply the size of
 * that paste. Four commands are all that is needed here — `AUTH`, `GET`, `SET`,
 * `EVAL` — and RESP is a simple enough protocol to speak directly.
 *
 * One socket is kept and reused across requests, and every command sent while a
 * reply is still on its way is queued behind it, so the counters do not pay for
 * a connection each.
 *
 * A failed command rejects, and the socket is dropped so the next one connects
 * again: the caller decides what a failure means, which for the quota store is
 * "count locally rather than refuse the translation".
 */

import { createConnection } from "node:net";
import { randomBytes } from "node:crypto";

/** How long one command may take before the link gives up on it. */
const COMMAND_TIMEOUT = 2000;

const CRLF = "\r\n";

/** Encodes one command the way the protocol asks: an array of bulk strings. */
function encode(args) {
  const parts = [`*${args.length}${CRLF}`];
  for (const arg of args) {
    const text = String(arg);
    parts.push(`$${Buffer.byteLength(text)}${CRLF}${text}${CRLF}`);
  }
  return parts.join("");
}

/**
 * Reads one reply off the front of `buffer`.
 *
 * Returns `null` while the reply is still incomplete, otherwise the value and
 * what is left of the buffer. An error reply throws, so a caller only ever sees
 * a value or an exception.
 */
function decode(buffer) {
  if (buffer.length === 0) return null;
  const type = String.fromCharCode(buffer[0]);
  const end = buffer.indexOf(CRLF, 1);
  if (end === -1) return null;
  const head = buffer.subarray(1, end).toString("utf8");
  const rest = buffer.subarray(end + 2);

  if (type === "+") return { value: head, rest };
  if (type === "-") {
    const error = new Error(head);
    error.redis = true;
    throw error;
  }
  if (type === ":") return { value: Number.parseInt(head, 10), rest };
  if (type === "$") {
    const length = Number.parseInt(head, 10);
    if (length === -1) return { value: null, rest };
    // The trailing CRLF is part of the reply, so it has to be here too.
    if (rest.length < length + 2) return null;
    return { value: rest.subarray(0, length).toString("utf8"), rest: rest.subarray(length + 2) };
  }
  if (type === "*") {
    const count = Number.parseInt(head, 10);
    if (count === -1) return { value: null, rest };
    const items = [];
    let cursor = rest;
    for (let index = 0; index < count; index += 1) {
      const item = decode(cursor);
      if (!item) return null;
      items.push(item.value);
      cursor = item.rest;
    }
    return { value: items, rest: cursor };
  }
  const error = new Error(`无法识别的 Redis 回复：${type}`);
  error.redis = true;
  throw error;
}

/**
 * Opens a link to one Redis instance.
 *
 * `password` is optional: the managed instances name a user and a password, the
 * plain ones are reachable with no credentials at all.
 */
export function createRedisLink({ host, port = 6379, user = "", password = "", connectTimeout = 5000 } = {}) {
  let socket = null;
  let opening = null;
  let buffer = Buffer.alloc(0);
  let queue = [];

  function settle(error) {
    const waiting = queue;
    queue = [];
    for (const entry of waiting) {
      clearTimeout(entry.timer);
      entry.reject(error);
    }
  }

  /** Drops the connection; the next command opens a fresh one. */
  function drop(error) {
    const dead = socket;
    socket = null;
    opening = null;
    buffer = Buffer.alloc(0);
    if (dead) dead.destroy();
    settle(error || new Error("Redis 连接已断开"));
  }

  function onData(chunk) {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      let reply;
      try {
        reply = decode(buffer);
      } catch (error) {
        drop(error);
        return;
      }
      if (!reply) return;
      buffer = reply.rest;
      const next = queue.shift();
      if (!next) continue;
      clearTimeout(next.timer);
      next.resolve(reply.value);
    }
  }

  function connect() {
    if (socket) return Promise.resolve(socket);
    if (opening) return opening;
    opening = new Promise((resolve, reject) => {
      const connecting = createConnection({ host, port });
      connecting.setNoDelay(true);
      const timer = setTimeout(() => {
        connecting.destroy();
        reject(new Error(`连接 Redis ${host}:${port} 超时`));
      }, connectTimeout);
      connecting.once("connect", async () => {
        clearTimeout(timer);
        socket = connecting;
        buffer = Buffer.alloc(0);
        connecting.on("data", onData);
        connecting.on("error", (error) => drop(error));
        connecting.on("close", () => drop());
        try {
          if (password) await send("AUTH", ...(user ? [user, password] : [password]));
          resolve(connecting);
        } catch (error) {
          drop(error);
          reject(error);
        }
      });
      connecting.once("error", (error) => {
        clearTimeout(timer);
        opening = null;
        reject(error);
      });
    }).catch((error) => {
      opening = null;
      throw error;
    });
    return opening;
  }

  /** Sends one command and resolves with its reply. */
  function send(...args) {
    return connect().then(
      (ready) =>
        new Promise((resolve, reject) => {
          const timer = setTimeout(() => {
            drop(new Error(`Redis 命令超时：${args[0]}`));
          }, COMMAND_TIMEOUT);
          queue.push({ resolve, reject, timer });
          ready.write(encode(args));
        }),
    );
  }

  return {
    send,
    close() {
      drop(new Error("Redis 连接已关闭"));
    },
  };
}

/** A token no other caller can pick, so only its owner releases a lock. */
export function lockToken() {
  return randomBytes(12).toString("hex");
}
