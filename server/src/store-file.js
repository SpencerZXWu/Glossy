/**
 * The store for plain Node hosts (Tencent SCF Web 函数, a container, `npm run
 * dev:node`). It runs the shared policy over an in-memory state and mirrors
 * that state into one JSON file after every accepted request.
 *
 * Cloud functions freeze an idle instance instead of killing it, so the file is
 * what makes the counters survive a cold start. Only `/tmp` is writable on SCF.
 */

import { readFileSync, renameSync, writeFileSync } from "node:fs";

import { createState, peek as policyPeek, refund as policyRefund, reserve as policyReserve } from "./policy.js";

function hydrate(raw) {
  const state = createState();
  for (const [key, row] of Object.entries(raw.usage ?? {})) {
    state.usage.set(key, { chars: Number(row?.chars) || 0, requests: Number(row?.requests) || 0 });
  }
  for (const [key, count] of Object.entries(raw.minutes ?? {})) {
    state.minutes.set(key, Number(count) || 0);
  }
  return state;
}

export function createFileStore({ file = "" } = {}) {
  let day = "";
  let state = createState();
  let warned = false;

  if (file) {
    try {
      const raw = JSON.parse(readFileSync(file, "utf8"));
      if (typeof raw?.day === "string") {
        day = raw.day;
        state = hydrate(raw);
      }
    } catch {
      // No file yet, or it was truncated by a crash: start the day empty.
    }
  }

  function save() {
    if (!file) return;
    try {
      // Written through a temporary file so a crash mid-write cannot leave a
      // half-parsed counter file behind.
      const text = JSON.stringify({
        day,
        usage: Object.fromEntries(state.usage),
        minutes: Object.fromEntries(state.minutes),
      });
      writeFileSync(`${file}.tmp`, text, "utf8");
      renameSync(`${file}.tmp`, file);
    } catch (error) {
      if (!warned) {
        warned = true;
        console.error(`glossy-cloud: 无法写入计数文件 ${file}，额度只在内存里计数：${error}`);
      }
    }
  }

  /** Counters belong to one UTC day, the same as the Durable Object per day. */
  function sync(nextDay) {
    if (nextDay !== day) {
      day = nextDay;
      state = createState();
    }
  }

  return {
    reserve(nextDay, input) {
      sync(nextDay);
      const result = policyReserve(state, input);
      if (result.ok) save();
      return result;
    },
    refund(nextDay, input) {
      sync(nextDay);
      policyRefund(state, input);
      save();
    },
    peek(nextDay, input) {
      sync(nextDay);
      return policyPeek(state, input);
    },
  };
}
