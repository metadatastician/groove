// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Unit + integration tests for the shared groove client and the extension's
// discovery core. Runs with `bun test`; no browser required — the
// integration test spins up a stub Groove provider on an ephemeral port.

import { test } from "node:test";
import assert from "node:assert/strict";
import http from "node:http";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const GrooveClient = require("../../js/groove-client.js");

// Make the classic-script globals available before loading the discovery core.
globalThis.GrooveClient = GrooveClient;
globalThis.GROOVE_TARGETS = require("../background/groove-targets.gen.js").GROOVE_TARGETS;
const discovery = require("../background/groove-discovery.js");

// ---------------------------------------------------------------- unit: parse

test("parseManifestJson accepts a valid manifest", () => {
  const result = GrooveClient.parseManifestJson(
    JSON.stringify({
      groove_version: "1",
      service_id: "groove-ref",
      service_version: "0.2.0",
      mode: "active",
      capabilities: { attestation: { type: "attestation" } },
      consumes: [],
    })
  );
  assert.equal(result.ok, true);
  assert.equal(result.manifest.service_id, "groove-ref");
});

test("parseManifestJson rejects bad groove_version, id, capabilities shape", () => {
  const bad = GrooveClient.parseManifestJson(
    JSON.stringify({ groove_version: "2", service_id: "Bad_ID!", capabilities: [] })
  );
  assert.equal(bad.ok, false);
  assert.equal(bad.errors.length, 3);
});

test("parseManifestJson rejects non-JSON", () => {
  assert.equal(GrooveClient.parseManifestJson("{ nope").ok, false);
});

// ---------------------------------------------------------------- unit: match

test("capabilityMatch truth table", () => {
  assert.equal(GrooveClient.capabilityMatch(["voice", "text"], ["voice"]).compatible, true);
  assert.equal(GrooveClient.capabilityMatch(["voice"], []).compatible, true);
  const miss = GrooveClient.capabilityMatch(["voice"], ["integrity", "scanning"]);
  assert.equal(miss.compatible, false);
  assert.equal(miss.reasons.length, 2);
});

test("versionSatisfies handles exact, plus-constraints, and garbage", () => {
  assert.equal(GrooveClient.versionSatisfies("2.1.0", "2.0.0"), true);
  assert.equal(GrooveClient.versionSatisfies("2.1.0", "2.2.0"), false);
  assert.equal(GrooveClient.versionSatisfies("3.0.0", "1.0+"), true);
  assert.equal(GrooveClient.versionSatisfies("1.0.0", "2.0+"), false);
  assert.equal(GrooveClient.versionSatisfies("weird", "1.0.0"), false);
  assert.equal(GrooveClient.versionSatisfies("1.2.3", undefined), true);
});

test("offeredTypes extracts capability types from the map", () => {
  assert.deepEqual(
    GrooveClient.offeredTypes({
      capabilities: { a: { type: "voice" }, b: { type: "text" }, c: {} },
    }),
    ["voice", "text"]
  );
});

test("malformed numeric versions never satisfy a constraint", () => {
  for (const invalid of ["1..2", "1.2.3.4", " 1", "01.2", "1e0.2", "-1", "9007199254740992", 1, {}]) {
    assert.equal(GrooveClient.versionSatisfies(invalid, "1.0+"), false);
    assert.equal(GrooveClient.versionSatisfies("1.2.3", invalid), false);
  }
});

test("consumes cannot smuggle non-string capability identifiers", () => {
  const base = { groove_version: "1", service_id: "test", capabilities: {} };
  assert.equal(GrooveClient.parseManifestJson(JSON.stringify(base)).ok, true);
  for (const consumes of [null, {}, [null], [1], [{}]]) {
    assert.equal(GrooveClient.parseManifestJson(JSON.stringify({ ...base, consumes })).ok, false);
  }
});

test("probe deadline includes a stalled response body", async () => {
  const server = http.createServer((_req, res) => {
    res.writeHead(200, { "Content-Type": "application/groove+json" });
    res.write('{"groove_version":');
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  try {
    const result = await Promise.race([
      discovery.probeOne(`http://127.0.0.1:${server.address().port}`, fetch),
      new Promise((_, reject) => {
        const guard = setTimeout(() => reject(new Error("probe exceeded deadline")), 3500);
        guard.unref();
      }),
    ]);
    assert.equal(result, null);
  } finally {
    server.closeAllConnections();
    server.close();
  }
});

test("probe rejects oversized bodies", async () => {
  const fetchImpl = async () => new Response("x".repeat(65537));
  assert.equal(await discovery.probeOne("http://127.0.0.1:1", fetchImpl), null);
});

// -------------------------------------------------------- integration: probe

/** Minimal stub Groove provider (JSON dialect) on an ephemeral port. */
function stubProvider() {
  const manifest = {
    groove_version: "1",
    service_id: "stub-provider",
    service_version: "0.0.1",
    mode: "active",
    capabilities: { attestation: { type: "attestation" } },
    consumes: [],
  };
  let handleCounter = 0;
  const handles = new Set();

  const server = http.createServer((req, res) => {
    if (req.method === "GET" && req.url === "/.well-known/groove") {
      res.writeHead(200, { "Content-Type": "application/groove+json" });
      res.end(JSON.stringify(manifest));
    } else if (req.method === "GET" && req.url === "/.well-known/groove/heartbeat") {
      res.writeHead(204);
      res.end();
    } else if (req.method === "POST" && req.url === "/.well-known/groove/connect") {
      const handle = `stub-${++handleCounter}`;
      handles.add(handle);
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ handle, provider: "stub-provider" }));
    } else if (req.method === "POST" && req.url === "/.well-known/groove/disconnect") {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        const { handle } = JSON.parse(body || "{}");
        const known = handles.delete(handle);
        res.writeHead(known ? 200 : 410, { "Content-Type": "application/json" });
        res.end(JSON.stringify(known ? { disconnected: true } : { error: "gone" }));
      });
    } else {
      res.writeHead(404);
      res.end();
    }
  });

  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () =>
      resolve({ server, port: server.address().port, handles })
    );
  });
}

test("probeOne discovers the stub provider and rejects a dead port", async () => {
  const { server, port } = await stubProvider();
  try {
    const manifest = await discovery.probeOne(`http://127.0.0.1:${port}`, fetch);
    assert.equal(manifest.service_id, "stub-provider");

    const dead = await discovery.probeOne("http://127.0.0.1:1", fetch);
    assert.equal(dead, null);
  } finally {
    server.close();
  }
});

test("probeTarget falls back from [::1] to 127.0.0.1", async () => {
  const { server, port } = await stubProvider(); // v4-only stub
  try {
    const found = await discovery.probeTarget({ id: "stub", port }, fetch);
    assert.ok(found, "v4 fallback finds the stub");
    assert.equal(found.manifest.service_id, "stub-provider");
    assert.ok(found.baseUrl.includes("127.0.0.1"));
  } finally {
    server.close();
  }
});
