#!/usr/bin/env node
// SPDX-License-Identifier: MPL-2.0
//
// Self-check for scripts/check-bebop-alignment.mjs (spline ADR-0005
// criterion (c)).
//
// This proves the LeaveReason conformance check is FALSIFIABLE: it feeds
// the validator a frame whose LeaveReason byte is not in the registry's
// cross_plane.leave_reason.values map and asserts the checker rejects it.
// A gate that cannot be made to fail is not a gate (gitar finding on PR
// #30: the enum walker only bounds-checked LeaveReason, never validated
// its decoded value against the registry).
//
// Run: node scripts/check-bebop-alignment.test.mjs

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { validateFrame } from "./check-bebop-alignment.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const alignment = JSON.parse(
  readFileSync(join(root, "registry", "bebop-voice-signal-alignment.json"), "utf8"),
);
const fixturesText = readFileSync(
  join(root, "spec", "conformance", "bebop", "voice_signal_frames.hex"),
  "utf8",
);

let failures = 0;

function check(description, fn) {
  try {
    fn();
    console.log(`PASS ${description}`);
  } catch (e) {
    console.error(`FAIL ${description}: ${e.message}`);
    failures += 1;
  }
}

function realFrame(name) {
  for (const line of fixturesText.split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const [n, hex] = line.split(" ");
    if (n === name) return Buffer.from(hex, "hex");
  }
  throw new Error(`fixture '${name}' not found`);
}

// --- Control: the real vendored 'leave' frame must still validate. ---
check("real 'leave' fixture (LeaveReason=1 'kicked') validates", () => {
  validateFrame(alignment, "leave", realFrame("leave"));
});

// --- The falsifiability proof: tamper with the LeaveReason byte so it no
// longer names a value the registry declares, and assert the checker
// REJECTS it. This is what makes the gate demonstrably able to fail. ---
check("tampered 'leave' frame with out-of-registry LeaveReason byte is REJECTED", () => {
  const buf = Buffer.from(realFrame("leave"));
  // Last byte is the LeaveReason enum payload. The registry only declares
  // 0-4 (voluntary/kicked/banned/timeout/server_shutdown); 99 is not one
  // of them — this is exactly the "burble renumbered the enum" scenario
  // the registry's A3 ruling exists to catch.
  buf[buf.length - 1] = 99;

  let threw = false;
  let message = "";
  try {
    validateFrame(alignment, "leave", buf);
  } catch (e) {
    threw = true;
    message = e.message;
  }
  if (!threw) {
    throw new Error("validateFrame accepted an out-of-registry LeaveReason value — gate is fake");
  }
  if (!message.includes("LeaveReason value 99")) {
    throw new Error(`rejected for the wrong reason: ${message}`);
  }
});

// --- Sanity: every value the registry itself declares as valid must be
// individually accepted (no off-by-one on the boundary of the map). ---
check("every registry-declared LeaveReason value individually validates", () => {
  const base = Buffer.from(realFrame("leave"));
  for (const key of Object.keys(alignment.cross_plane.leave_reason.values)) {
    const buf = Buffer.from(base);
    buf[buf.length - 1] = Number(key);
    validateFrame(alignment, "leave", buf);
  }
});

if (failures > 0) {
  console.error(`check-bebop-alignment.test: ${failures} failure(s)`);
  process.exit(1);
}
console.log("check-bebop-alignment.test: all checks passed");
