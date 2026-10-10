/**
 * The shared store, against a Redis that is only as real as it needs to be.
 *
 * The tests here are about what the deployment depends on: that two instances of
 * the service see the same allowance, that the counters outlive the instance
 * that counted them, and that a Redis which cannot be reached costs accuracy
 * rather than translations. The protocol itself is spoken by the same client the
 * deployment uses, so a reply this fake gets wrong is a reply the real thing
 * would get wrong too.
 */

import assert from "node:assert/strict";
import { createServer } from "node:net";
import test from "node:test";

import { createRedisLink } from "../src/redis-link.js";
import { createRedisStore } from "../src/store-redis.js";

const DAY = "2025-09-01";
const NEXT_DAY = "2025-09-02";
const LIMITS = {
  charsPerClient: 100,
  charsPerIp: 150,
  charsTotal: 1000,
  maxCharsPerRequest: 50,
  requestsPerMinute: 3,
};
const NOW = Date.parse("2025-09-01T10:00:00Z");

/** Reads one `<count>\r\n$len\r\narg\r\n…` command, or null while incomplete. */
function readCommand(buffer) {
  if (!buffer.length) return null;
  if (buffer[0] !== 0x2a) throw new Error("a client must send an array");
  const head = buffer.indexOf("\r\n");
  if (head === -1) return null;
  const count = Number.parseInt(buffer.subarray(1, head).toString(), 10);
  const args = [];
  let cursor = head + 2;
  for (let index = 0; index < count; index += 1) {
    if (cursor >= buffer.length) return null;
    if (buffer[cursor] !== 0x24) throw new Error("a client must send bulk strings");
    const end = buffer.indexOf("\r\n", cursor);
    if (end === -1) return null;
    const length = Number.parseInt(buffer.subarray(cursor + 1, end).toString(), 10);
    const from = end + 2;
    if (buffer.length < from + length + 2) return null;
    args.push(buffer.subarray(from, from + length).toString());
    cursor = from + length + 2;
  }
  return { args, rest: buffer.subarray(cursor) };
}

function reply(value) {
  if (value === null || value === undefined) return "$-1\r\n";
  if (typeof value === "number") return `:${value}\r\n`;
  if (value === "OK") return "+OK\r\n";
  return `$${Buffer.byteLength(value)}\r\n${value}\r\n`;
}

/**
 * A Redis that keeps strings in a Map and understands the five commands the
 * store sends. Every command it was asked is kept in `seen`.
 */
async function fakeRedis(t) {
  const values = new Map();
  const seen = [];
  const server = createServer((socket) => {
    let buffer = Buffer.alloc(0);
    socket.on("data", (chunk) => {
      buffer = Buffer.concat([buffer, chunk]);
      for (;;) {
        let command;
        try {
          command = readCommand(buffer);
        } catch {
          socket.end("-ERR protocol error\r\n");
          return;
        }
        if (!command) return;
        buffer = command.rest;
        const [name, ...args] = command.args;
        seen.push(command.args);
        const verb = String(name).toUpperCase();
        if (verb === "AUTH") socket.write(reply("OK"));
        else if (verb === "GET") socket.write(reply(values.get(args[0]) ?? null));
        else if (verb === "SET") {
          const [key, value, ...flags] = args;
          if (flags.includes("NX") && values.has(key)) socket.write(reply(null));
          else {
            values.set(key, value);
            socket.write(reply("OK"));
          }
        } else if (verb === "DEL") socket.write(reply(values.delete(args[0]) ? 1 : 0));
        else if (verb === "EVAL") {
          // The only script the store sends: `EVAL <script> 1 <key> <token>`,
          // releasing the lock if it is still the one that was taken.
          const [key, token] = [args[2], args[3]];
          const mine = values.get(key) === token;
          if (mine) values.delete(key);
          socket.write(reply(mine ? 1 : 0));
        } else socket.write("-ERR unknown command\r\n");
      }
    });
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => {
    if (server.listening) server.close();
  });
  return { port: server.address().port, values, seen, server };
}

function linkTo(db, options = {}) {
  return createRedisLink({ host: "127.0.0.1", port: db.port, ...options });
}

function book(store, over = {}) {
  return store.reserve(DAY, {
    clientId: "client-a",
    ipHash: "ip-a",
    chars: 10,
    limits: LIMITS,
    now: NOW,
    minute: Math.floor(NOW / 60000),
    ...over,
  });
}

test("two instances of the service share one allowance", async (t) => {
  const db = await fakeRedis(t);
  const first = createRedisStore({ link: linkTo(db) });
  const second = createRedisStore({ link: linkTo(db) });

  assert.equal((await book(first, { chars: 30 })).ok, true);
  // The second instance has counted nothing itself, and still knows about it.
  assert.deepEqual(await second.peek(DAY, { clientId: "client-a", ipHash: "ip-a" }), {
    client: 30,
    ip: 30,
    total: 30,
  });
  assert.equal((await book(second, { chars: 71 })).ok, false);
  assert.equal((await book(second, { chars: 70 })).ok, true);
});

test("the counters outlive the instance that counted them", async (t) => {
  const db = await fakeRedis(t);

  // An instance that booked a few characters and was then recycled: everything
  // it held in memory and in `/tmp` is gone, and the next instance starts empty.
  await book(createRedisStore({ link: linkTo(db) }), { chars: 25 });

  const replacement = createRedisStore({ link: linkTo(db) });
  assert.equal((await replacement.peek(DAY, { clientId: "client-a" })).client, 25);
  assert.equal((await book(replacement, { chars: 76 })).ok, false);
});

test("the lock is taken around a change and let go again", async (t) => {
  const db = await fakeRedis(t);
  const store = createRedisStore({ link: linkTo(db) });

  await book(store, { chars: 10 });
  // A lock that was never released would keep the next request waiting until its
  // deadline, and every request after that as well.
  assert.equal([...db.values.keys()].some((key) => key.includes("lock")), false);
  const taking = db.seen.filter(([, , , flag]) => flag === "NX");
  assert.equal(taking.length, 1, "the change was made without the shared lock");
});

test("a refused request writes nothing", async (t) => {
  const db = await fakeRedis(t);
  const store = createRedisStore({ link: linkTo(db) });
  const before = db.seen.length;

  assert.equal((await book(store, { chars: 101 })).ok, false);
  // Only the lock is taken; the counters are not written, so a refused request
  // cannot spend anything.
  const writes = db.seen
    .slice(before)
    .filter(([name, key]) => name === "SET" && String(key).includes(DAY) && !String(key).includes("lock"));
  assert.deepEqual(writes, []);
});

test("a password is sent before anything else when one is configured", async (t) => {
  const db = await fakeRedis(t);
  const store = createRedisStore({ link: linkTo(db, { password: "hunter2" }) });

  await book(store, { chars: 10 });
  assert.deepEqual(db.seen[0], ["AUTH", "hunter2"]);
});

test("a named user authenticates with both halves", async (t) => {
  const db = await fakeRedis(t);
  const store = createRedisStore({ link: linkTo(db, { user: "glossy", password: "hunter2" }) });

  await book(store, { chars: 10 });
  assert.deepEqual(db.seen[0], ["AUTH", "glossy", "hunter2"]);
});

test("a Redis that cannot be reached costs accuracy, not translations", async (t) => {
  const db = await fakeRedis(t);
  const port = db.port;
  await new Promise((resolve) => db.server.close(resolve));

  const logged = [];
  const original = console.error;
  console.error = (line) => logged.push(line);
  t.after(() => {
    console.error = original;
  });

  const store = createRedisStore({ link: linkTo({ port }) });
  const booked = await book(store, { chars: 10 });
  assert.equal(booked.ok, true, "a translation was refused because Redis was down");
  assert.equal(booked.remaining, 90);
  // The counters are this instance's own again, and the log says so exactly once.
  assert.equal((await store.peek(DAY, { clientId: "client-a" })).client, 10);
  assert.equal(logged.length, 1);
  assert.match(logged[0], /共享计数不可用/);
});

test("a lock that is never released still answers", async (t) => {
  const db = await fakeRedis(t);
  const link = linkTo(db);
  db.values.set("glossy:quota:lock:2025-09-01", "someone-else");

  const store = createRedisStore({ link, waitMs: 120 });
  const before = Date.now();
  const booked = await book(store, { chars: 10 });
  assert.equal(booked.ok, true);
  // It waited for the lock rather than walking past it, then counted locally.
  assert.ok(Date.now() - before >= 100);
  assert.equal(db.values.get("glossy:quota:lock:2025-09-01"), "someone-else");
});

test("a day of its own is a key of its own", async (t) => {
  const db = await fakeRedis(t);
  const store = createRedisStore({ link: linkTo(db) });

  await book(store, { chars: 10 });
  const tomorrow = store.reserve(NEXT_DAY, {
    clientId: "client-a",
    ipHash: "ip-a",
    chars: 10,
    limits: LIMITS,
    now: NOW,
    minute: Math.floor(NOW / 60000),
  });
  assert.equal((await tomorrow).ok, true);
  // Yesterday's allowance cannot be spent today, and today's cannot be spent
  // twice because yesterday's key is empty.
  const keys = [...db.values.keys()].filter((key) => key.startsWith("glossy:quota:2"));
  assert.deepEqual(keys.sort(), ["glossy:quota:2025-09-01", "glossy:quota:2025-09-02"]);
  assert.equal((await store.peek(NEXT_DAY, { clientId: "client-a" })).client, 10);
});

test("the counters an old file store left behind are read as they are", async (t) => {
  const db = await fakeRedis(t);
  // The shape `store-file.js` writes, which a deployment that moved to Redis
  // still has on disk for the current day.
  db.values.set(
    "glossy:quota:2025-09-01",
    JSON.stringify({
      day: DAY,
      usage: { "client|client-a": { chars: 40, requests: 2 } },
      minutes: { "ip-a|100": 2 },
    }),
  );

  const store = createRedisStore({ link: linkTo(db) });
  assert.equal((await store.peek(DAY, { clientId: "client-a" })).client, 40);
  assert.equal((await book(store, { chars: 60 })).ok, true);
  assert.equal((await book(store, { chars: 1 })).ok, false);
});
