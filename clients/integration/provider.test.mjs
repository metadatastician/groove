// SPDX-License-Identifier: MPL-2.0
// Native provider + actual browser discovery core, not a mock wire lifecycle.
// Build first: cargo build --locked --workspace
import { test } from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
globalThis.GrooveClient = require("../js/groove-client.js");
globalThis.GROOVE_TARGETS = [];
const core = require("../browser-extension/background/groove-discovery.js");

async function startProvider() {
  const binary = process.env.GROOVE_PROVIDER_BIN || fileURLToPath(new URL("../../target/debug/groove-provider", import.meta.url));
  const child = spawn(binary, ["--port", "0"], { stdio: ["ignore", "pipe", "pipe"] });
  let output = "";
  let errors = "";
  child.stderr.on("data", (data) => { errors += data; });
  try {
    const port = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("provider startup deadline")), 5000);
      const fail = (error) => { clearTimeout(timeout); reject(error); };
      child.once("error", fail);
      child.once("exit", (code) => fail(new Error(`provider exited ${code}: ${errors}`)));
      child.stdout.on("data", (data) => {
        output += data;
        const match = output.match(/127\.0\.0\.1:(\d+)/);
        if (match) { clearTimeout(timeout); resolve(Number(match[1])); }
      });
    });
    return { child, baseUrl: `http://127.0.0.1:${port}` };
  } catch (error) {
    child.kill("SIGKILL");
    throw error;
  }
}

test("browser core negotiates one renewable native session and consumes it once", async () => {
  const { child, baseUrl } = await startProvider();
  const service = "native-integration";
  try {
    const manifest = await core.probeOne(baseUrl, fetch);
    assert.equal(manifest.service_id, "groove-ref");
    core._registry().set(service, { baseUrl, status: "discovered" });
    const [first, duplicate] = await Promise.all([
      core.connectService(service), core.connectService(service),
    ]);
    assert.equal(first.ok, true);
    assert.equal(first.handle, duplicate.handle);
    assert.equal(core._connections().size, 1);
    await core.heartbeat(service);
    assert.equal(core._connections().get(service).misses, 0);
    assert.equal((await core.disconnectService(service)).ok, true);
    assert.equal(core._connections().size, 0);
    const replay = await fetch(`${baseUrl}/.well-known/groove/disconnect`, {
      method: "POST", headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ handle: first.handle }),
    });
    assert.equal(replay.status, 410);
    const stale = await fetch(`${baseUrl}/.well-known/groove/heartbeat?handle=${first.handle}`);
    assert.equal(stale.status, 404);
  } finally {
    await core.disconnectService(service);
    core._registry().delete(service);
    if (child.exitCode === null && child.signalCode === null) {
      const exited = new Promise((resolve) => child.once("exit", resolve));
      const deadline = setTimeout(() => child.kill("SIGKILL"), 5000);
      child.kill("SIGINT");
      await exited;
      clearTimeout(deadline);
      assert.equal(child.exitCode, 0, "explicit provider shutdown completes cleanly");
    }
  }
});
