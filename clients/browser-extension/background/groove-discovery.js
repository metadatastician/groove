// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Groove Discovery — background script for the Groove browser extension.
//
// Probes the registry-derived localhost targets for groove-aware services
// (dual-stack: [::1] first, then 127.0.0.1, per TRANSPORT §7.6), maintains a
// live registry, and speaks the SPEC §4 lifecycle: connect / heartbeat /
// disconnect with linear handles.
//
// Load order (manifest.json): groove-targets.gen.js (GROOVE_TARGETS, from
// registry/groove-registry.json — never hand-edit), groove-client.vendor.js
// (GrooveClient: parse/match logic shared with clients/js/), then this file.
//
// Classic scripts (MV2, no modules). Node tests import the pure core via the
// module.exports guard at the bottom.

/* global GROOVE_TARGETS, GrooveClient */

// Probe interval (60 seconds) and per-request timeout (2 seconds).
const PROBE_INTERVAL_MS = 60_000;
const PROBE_TIMEOUT_MS = 2_000;

// Heartbeat per SPEC §4.3: 5s interval, drop after 3 misses.
const HEARTBEAT_INTERVAL_MS = 5_000;
const HEARTBEAT_MAX_MISSES = 3;

// Current groove registry — populated by discovery, persisted to storage.
let grooveRegistry = new Map();

// Live connections: serviceName -> { handle, baseUrl, misses, timer }.
const connections = new Map();
const connecting = new Map();

// ============================================================================
// Probing (pure-ish core; fetchImpl injectable for tests)
// ============================================================================

/**
 * Bound both headers and body by the same deadline and a 64 KiB body limit.
 */
async function fetchWithTimeout(fetchImpl, url, options, timeoutMs) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetchImpl(url, { ...options, signal: controller.signal });
    const chunks = [];
    let size = 0;
    if (response.body) {
      const reader = response.body.getReader();
      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          size += value.byteLength;
          if (size > 65536) throw new Error("Groove response exceeds 64 KiB");
          chunks.push(value);
        }
      } finally {
        reader.releaseLock();
      }
    }
    return new Response(size ? new Blob(chunks) : null, {
      status: response.status, statusText: response.statusText, headers: response.headers,
    });
  } finally {
    clearTimeout(timeoutId);
    controller.abort();
  }
}

/**
 * Probe one base URL for a Groove manifest.
 * @returns {Promise<Object|null>} parsed+validated manifest or null
 */
async function probeOne(baseUrl, fetchImpl) {
  try {
    const response = await fetchWithTimeout(
      fetchImpl,
      `${baseUrl}/.well-known/groove`,
      { method: "GET", headers: { Accept: GrooveClient.buildAcceptHeader() } },
      PROBE_TIMEOUT_MS
    );
    if (!response.ok) return null;
    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("a2ml")) return null; // serve-only encoding; we require JSON (ADR 0002)
    const parsed = GrooveClient.parseManifestJson(await response.text());
    if (!parsed.ok) {
      console.warn(`Groove: invalid manifest from ${baseUrl}:`, parsed.errors);
      return null;
    }
    return parsed.manifest;
  } catch {
    return null;
  }
}

/**
 * Probe a target on [::1] first, then 127.0.0.1 (TRANSPORT §7.6).
 * @returns {Promise<{manifest: Object, baseUrl: string}|null>}
 */
async function probeTarget(target, fetchImpl) {
  for (const host of [`http://[::1]:${target.port}`, `http://127.0.0.1:${target.port}`]) {
    const manifest = await probeOne(host, fetchImpl);
    if (manifest) return { manifest, baseUrl: host };
  }
  return null;
}

/**
 * Discover all registry targets; update and persist the registry.
 * @returns {Promise<number>} number of discovered services
 */
async function discoverAll(fetchImpl = fetch) {
  const results = await Promise.allSettled(
    GROOVE_TARGETS.map(async (target) => ({ target, found: await probeTarget(target, fetchImpl) }))
  );

  let discovered = 0;
  const registry = {};

  for (const result of results) {
    if (result.status !== "fulfilled") continue;
    const { target, found } = result.value;

    if (found) {
      const { manifest, baseUrl } = found;
      registry[target.id] = {
        name: target.id,
        port: target.port,
        baseUrl,
        status: connections.has(target.id) ? "connected" : "discovered",
        serviceId: manifest.service_id,
        version: manifest.service_version || "unknown",
        capabilities: GrooveClient.offeredTypes(manifest),
        consumes: manifest.consumes || [],
        endpoints: manifest.endpoints || {},
        lastProbe: new Date().toISOString(),
      };
      discovered++;
    } else {
      registry[target.id] = {
        name: target.id,
        port: target.port,
        status: "not_found",
        capabilities: [],
        consumes: [],
        lastProbe: new Date().toISOString(),
      };
    }
  }

  grooveRegistry = new Map(Object.entries(registry));
  if (typeof browser !== "undefined") {
    await browser.storage.local.set({ grooveRegistry: registry });
  }

  console.log(`Groove: discovered ${discovered}/${GROOVE_TARGETS.length} services`);
  return discovered;
}

// ============================================================================
// Lifecycle: connect / heartbeat / disconnect (SPEC §4)
// ============================================================================

/** The consumer manifest this extension presents on connect. */
function consumerManifest() {
  const version =
    typeof browser !== "undefined" ? browser.runtime.getManifest().version : "0.0.0";
  return {
    groove_version: "1",
    service_id: "groove-browser-extension",
    service_version: version,
    mode: "passive",
    capabilities: {},
    consumes: [],
    lease: { mode: "hard", ttl_ms: 15000 },
  };
}

/**
 * Connect to a discovered service (POST connect, start heartbeat loop).
 * @returns {Promise<{ok: boolean, handle?: string, error?: string, reasons?: string[]}>}
 */
async function connectService(serviceName, fetchImpl = fetch) {
  if (connecting.has(serviceName)) return connecting.get(serviceName);
  const pending = connectServiceOnce(serviceName, fetchImpl)
    .finally(() => connecting.delete(serviceName));
  connecting.set(serviceName, pending);
  return pending;
}

async function connectServiceOnce(serviceName, fetchImpl) {
  const entry = grooveRegistry.get(serviceName);
  if (!entry || entry.status === "not_found") {
    return { ok: false, error: `${serviceName} not discovered` };
  }
  if (connections.has(serviceName)) {
    return { ok: true, handle: connections.get(serviceName).handle };
  }

  try {
    const response = await fetchWithTimeout(
      fetchImpl,
      `${entry.baseUrl}/.well-known/groove/connect`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(consumerManifest()),
      },
      PROBE_TIMEOUT_MS
    );
    const body = await response.json().catch(() => ({}));
    if (response.status === 409) {
      return { ok: false, error: "incompatible", reasons: body.reasons || [] };
    }
    if (!response.ok || typeof body.handle !== "string" || !body.handle) {
      return { ok: false, error: `connect failed (${response.status})` };
    }
    if (body.lease?.mode !== "hard" || body.lease?.ttl_ms !== 15000) {
      // Do not claim a renewable session if the provider omitted the lease.
      await fetchWithTimeout(fetchImpl, `${entry.baseUrl}/.well-known/groove/disconnect`, {
        method: "POST", headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ handle: body.handle }),
      }, PROBE_TIMEOUT_MS);
      return { ok: false, error: "provider did not accept the requested hard lease" };
    }

    const conn = { handle: body.handle, baseUrl: entry.baseUrl, misses: 0, timer: null };
    conn.timer = setInterval(() => heartbeat(serviceName, fetchImpl), HEARTBEAT_INTERVAL_MS);
    connections.set(serviceName, conn);
    entry.status = "connected";
    console.log(`Groove: connected to ${serviceName}`);
    return { ok: true, handle: body.handle };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

/** One heartbeat; degrade on miss, drop after HEARTBEAT_MAX_MISSES. */
async function heartbeat(serviceName, fetchImpl = fetch) {
  const conn = connections.get(serviceName);
  const entry = grooveRegistry.get(serviceName);
  if (!conn) return;

  let alive = false;
  try {
    const response = await fetchWithTimeout(
      fetchImpl,
      `${conn.baseUrl}/.well-known/groove/heartbeat?handle=${encodeURIComponent(conn.handle)}`,
      { method: "GET" },
      PROBE_TIMEOUT_MS
    );
    alive = response.status === 204;
  } catch {
    alive = false;
  }

  if (connections.get(serviceName) !== conn) return;
  if (alive) {
    conn.misses = 0;
    if (entry) entry.status = "connected";
    return;
  }

  conn.misses += 1;
  if (entry) entry.status = "degraded";
  if (conn.misses >= HEARTBEAT_MAX_MISSES) {
    // Graceful degradation (SPEC §4.4): losing a partner is not an error.
    clearInterval(conn.timer);
    connections.delete(serviceName);
    if (entry) entry.status = "not_found";
    console.log(`Groove: lost ${serviceName} after ${conn.misses} missed heartbeats`);
  }
}

/**
 * Disconnect (POST disconnect; the handle is linearly consumed server-side).
 */
async function disconnectService(serviceName, fetchImpl = fetch) {
  if (connecting.has(serviceName)) await connecting.get(serviceName);
  const conn = connections.get(serviceName);
  if (!conn) return { ok: false, error: `${serviceName} not connected` };

  clearInterval(conn.timer);
  connections.delete(serviceName);
  const entry = grooveRegistry.get(serviceName);
  if (entry && entry.status === "connected") entry.status = "discovered";

  try {
    const response = await fetchWithTimeout(
      fetchImpl,
      `${conn.baseUrl}/.well-known/groove/disconnect`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ handle: conn.handle }),
      },
      PROBE_TIMEOUT_MS
    );
    return { ok: response.ok };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

/** Find which discovered service provides a capability type. */
function findCapability(capabilityName) {
  for (const [, entry] of grooveRegistry) {
    if (entry.status !== "not_found" && entry.capabilities.includes(capabilityName)) {
      return entry;
    }
  }
  return null;
}

// ============================================================================
// Extension message handler + lifecycle hooks (browser only)
// ============================================================================

if (typeof browser !== "undefined") {
  browser.runtime.onMessage.addListener((message, _sender) => {
    switch (message.type) {
      case "groove:discover":
        return discoverAll().then((count) => ({ discovered: count }));

      case "groove:status":
        return Promise.resolve(Object.fromEntries(grooveRegistry));

      case "groove:connect":
        return connectService(message.service);

      case "groove:disconnect":
        return disconnectService(message.service);

      case "groove:find-capability":
        return Promise.resolve(findCapability(message.capability));

      case "groove:summary":
        return browser.storage.local
          .get("grooveRegistry")
          .then((data) => data.grooveRegistry || {});

      default:
        return Promise.resolve({ error: "unknown message type" });
    }
  });

  // Discover on startup, then periodically.
  discoverAll();
  setInterval(() => discoverAll(), PROBE_INTERVAL_MS);
  browser.runtime.onStartup?.addListener(() => discoverAll());
  browser.runtime.onInstalled?.addListener(() => discoverAll());

  console.log("Groove extension: background script loaded");
}

// Node test hook (classic script in the browser; module in tests).
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    probeOne,
    probeTarget,
    discoverAll,
    connectService,
    heartbeat,
    disconnectService,
    findCapability,
    _registry: () => grooveRegistry,
    _connections: () => connections,
  };
}
