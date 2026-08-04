#!/usr/bin/env node
// SPDX-License-Identifier: MPL-2.0
//
// Bebop VoiceSignal alignment conformance (spline ADR-0005 criterion (c)).
//
// Validates RECORDED frames — bytes produced by burble's actual generator,
// vendored in spec/conformance/bebop/voice_signal_frames.hex — against the
// alignment record registry/bebop-voice-signal-alignment.json:
//
//   1. every frame's discriminator tag maps to the variant name the fixture
//      claims;
//   2. walking the variant's declared field layout consumes the frame
//      EXACTLY (no bytes over, none missing) with every string length
//      prefix in bounds;
//   3. every variant in the alignment is exercised by at least one frame.
//
// This checks the ALIGNMENT (the mapping), not the schema — burble owns the
// schema (spline ADR-0003). If burble's wire output drifts from the recorded
// mapping, this fails.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// Field walkers: consume a field from buf at offset, return the new offset.
// Throws on any bounds violation. `alignment` is passed through so kinds
// that cross-check against the registry (e.g. leave_reason) can reach it.
export const walkers = {
  string(buf, at, ctx) {
    if (at + 4 > buf.length) throw new Error(`${ctx}: truncated string length prefix`);
    const len = buf.readUInt32LE(at);
    if (at + 4 + len > buf.length) {
      throw new Error(`${ctx}: string length ${len} exceeds frame`);
    }
    return at + 4 + len;
  },
  bool(buf, at, ctx) {
    if (at + 1 > buf.length) throw new Error(`${ctx}: truncated bool`);
    const b = buf[at];
    if (b !== 0 && b !== 1) throw new Error(`${ctx}: invalid bool byte ${b}`);
    return at + 1;
  },
  enum(buf, at, ctx) {
    if (at + 1 > buf.length) throw new Error(`${ctx}: truncated enum`);
    return at + 1;
  },
  // Unlike the generic `enum` walker, this validates the decoded byte
  // against the registry's cross_plane.leave_reason.values name<->value
  // map (burble owner ruling A3: voice_signal.LeaveReason MUST stay
  // value-aligned with room_event.LeaveReason). Bounds-checking alone
  // cannot catch a renumbered enum — this is the check that can actually
  // fail when burble's LeaveReason drifts from the recorded mapping.
  leave_reason(buf, at, ctx, alignment) {
    if (at + 1 > buf.length) throw new Error(`${ctx}: truncated leave_reason enum`);
    const b = buf[at];
    const values = alignment?.cross_plane?.leave_reason?.values;
    if (!values) {
      throw new Error(`${ctx}: registry has no cross_plane.leave_reason.values to check against`);
    }
    if (!Object.prototype.hasOwnProperty.call(values, String(b))) {
      const valid = Object.entries(values)
        .map(([k, v]) => `${k}=${v}`)
        .join(", ");
      throw new Error(
        `${ctx}: LeaveReason value ${b} is not in registry cross_plane.leave_reason.values (valid: ${valid})`,
      );
    }
    return at + 1;
  },
  u16(buf, at, ctx) {
    if (at + 2 > buf.length) throw new Error(`${ctx}: truncated u16`);
    return at + 2;
  },
  f32(buf, at, ctx) {
    if (at + 4 > buf.length) throw new Error(`${ctx}: truncated f32`);
    return at + 4;
  },
  vec3(buf, at, ctx, alignment) {
    for (let i = 0; i < 3; i++) at = walkers.f32(buf, at, ctx, alignment);
    return at;
  },
  sdp_payload(buf, at, ctx, alignment) {
    at = walkers.string(buf, at, ctx, alignment);
    return walkers.string(buf, at, ctx, alignment);
  },
  ice_candidate_payload(buf, at, ctx, alignment) {
    at = walkers.string(buf, at, ctx, alignment);
    at = walkers.u16(buf, at, ctx, alignment);
    at = walkers.string(buf, at, ctx, alignment);
    return walkers.string(buf, at, ctx, alignment);
  },
};

// Validates a single frame's bytes against its alignment-declared variant.
// Pure function (no I/O, no process.exit) so it can be exercised directly
// from a test with synthetic buffers. Throws on any conformance failure.
export function validateFrame(alignment, name, buf) {
  const ctx = `frame '${name}'`;
  if (buf.length < 1) throw new Error(`${ctx}: empty frame`);
  const tag = buf[0];
  const variant = alignment.variants[String(tag)];
  if (!variant) throw new Error(`${ctx}: tag ${tag} not in alignment`);
  if (variant.name !== name) {
    throw new Error(`${ctx}: tag ${tag} maps to '${variant.name}' in the alignment`);
  }

  let at = 1;
  for (const kind of variant.fields) {
    const walk = walkers[kind];
    if (!walk) throw new Error(`${ctx}: alignment declares unknown field kind '${kind}'`);
    at = walk(buf, at, `${ctx} field '${kind}'`, alignment);
  }
  if (at !== buf.length) {
    throw new Error(`${ctx}: layout consumed ${at} of ${buf.length} bytes`);
  }
  return { tag, variantName: variant.name };
}

function runAll(alignment, fixturesText) {
  let failures = 0;
  const seen = new Set();

  for (const line of fixturesText.split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const [name, hex] = line.split(" ");
    const buf = Buffer.from(hex, "hex");

    try {
      const { tag, variantName } = validateFrame(alignment, name, buf);
      seen.add(variantName);
      console.log(`PASS ${name} (tag ${tag}, ${buf.length} bytes)`);
    } catch (e) {
      console.error(`FAIL ${e.message}`);
      failures += 1;
    }
  }

  for (const { name } of Object.values(alignment.variants)) {
    if (!seen.has(name)) {
      console.error(`FAIL variant '${name}' has no recorded frame — coverage hole`);
      failures += 1;
    }
  }

  return { failures, seenCount: seen.size };
}

// Only run as a CLI check when executed directly (not when imported by tests).
if (import.meta.url === `file://${process.argv[1]}`) {
  const alignment = JSON.parse(
    readFileSync(join(root, "registry", "bebop-voice-signal-alignment.json"), "utf8"),
  );
  const fixtures = readFileSync(
    join(root, "spec", "conformance", "bebop", "voice_signal_frames.hex"),
    "utf8",
  );

  const { failures, seenCount } = runAll(alignment, fixtures);

  if (failures > 0) {
    console.error(`bebop-alignment: ${failures} failure(s)`);
    process.exit(1);
  }
  console.log(`bebop-alignment: all ${seenCount} variants conform`);
}
