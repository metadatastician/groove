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
const alignment = JSON.parse(
  readFileSync(join(root, "registry", "bebop-voice-signal-alignment.json"), "utf8"),
);
const fixtures = readFileSync(
  join(root, "spec", "conformance", "bebop", "voice_signal_frames.hex"),
  "utf8",
);

// Field walkers: consume a field from buf at offset, return the new offset.
// Throws on any bounds violation.
const walkers = {
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
  u16(buf, at, ctx) {
    if (at + 2 > buf.length) throw new Error(`${ctx}: truncated u16`);
    return at + 2;
  },
  f32(buf, at, ctx) {
    if (at + 4 > buf.length) throw new Error(`${ctx}: truncated f32`);
    return at + 4;
  },
  vec3(buf, at, ctx) {
    for (let i = 0; i < 3; i++) at = walkers.f32(buf, at, ctx);
    return at;
  },
  sdp_payload(buf, at, ctx) {
    at = walkers.string(buf, at, ctx);
    return walkers.string(buf, at, ctx);
  },
  ice_candidate_payload(buf, at, ctx) {
    at = walkers.string(buf, at, ctx);
    at = walkers.u16(buf, at, ctx);
    at = walkers.string(buf, at, ctx);
    return walkers.string(buf, at, ctx);
  },
};

let failures = 0;
const seen = new Set();

for (const line of fixtures.split("\n")) {
  if (!line || line.startsWith("#")) continue;
  const [name, hex] = line.split(" ");
  const buf = Buffer.from(hex, "hex");
  const ctx = `frame '${name}'`;

  try {
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
      at = walk(buf, at, `${ctx} field '${kind}'`);
    }
    if (at !== buf.length) {
      throw new Error(`${ctx}: layout consumed ${at} of ${buf.length} bytes`);
    }

    seen.add(variant.name);
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

if (failures > 0) {
  console.error(`bebop-alignment: ${failures} failure(s)`);
  process.exit(1);
}
console.log(`bebop-alignment: all ${seen.size} variants conform`);
