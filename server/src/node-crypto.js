/**
 * Node 18 (what SCF offers) has no global `crypto`; it only landed in Node 19.
 * The handler uses bare `crypto.subtle`, so bridge it here — importing this
 * module before the handler is enough.
 */

import { webcrypto } from "node:crypto";

if (!globalThis.crypto) globalThis.crypto = webcrypto;
