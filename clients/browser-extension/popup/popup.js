// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Groove Discovery popup — renders the service discovery + connection status.

const servicesEl = document.getElementById("services");
const badgeEl = document.getElementById("count-badge");
const probeBtn = document.getElementById("btn-probe");

const STATUS_LABEL = {
  connected: "connected",
  discovered: "discovered",
  degraded: "degraded",
  not_found: "",
};

/**
 * Render the service list from the groove registry.
 *
 * @param {Object} registry - Map of service name → groove entry
 */
function renderServices(registry) {
  const entries = Object.values(registry);
  const live = entries.filter((e) => e.status !== "not_found").length;
  const connected = entries.filter((e) => e.status === "connected").length;

  // Update badge.
  badgeEl.textContent = connected > 0 ? `${connected} connected` : `${live} discovered`;
  badgeEl.className = live > 0 ? "badge" : "badge zero";

  if (entries.length === 0) {
    showMessage("No groove targets in the registry");
    return;
  }

  // Sort: connected, then discovered/degraded, then by name.
  const rank = (e) =>
    e.status === "connected" ? 0 : e.status === "degraded" ? 1 : e.status === "discovered" ? 2 : 3;
  entries.sort((a, b) => rank(a) - rank(b) || a.name.localeCompare(b.name));

  servicesEl.replaceChildren(...entries.map((entry) => {
    const row = element("div", "service");
    const status = Object.hasOwn(STATUS_LABEL, entry.status) ? entry.status : "not_found";
    const info = element("div", "service-info");
    const name = element("div", "service-name", entry.name);
    if (STATUS_LABEL[status]) name.append(element("small", "", ` (${STATUS_LABEL[status]})`));
    info.append(name, element("div", "service-caps", entry.capabilities?.join(", ") || "none"));
    row.append(element("div", `dot ${status}`), info, element("div", "service-port", `:${entry.port}`));
    return row;
  }));
}

/**
 * All remote strings are text nodes, never HTML or class names.
 */
function element(tag, className, text) {
  const node = document.createElement(tag);
  node.className = className;
  if (text !== undefined) node.textContent = String(text);
  return node;
}

function showMessage(message) {
  servicesEl.replaceChildren(element("div", "empty", message));
}

/**
 * Load the current groove status from the background script.
 */
async function loadStatus() {
  try {
    const registry = await browser.runtime.sendMessage({ type: "groove:status" });
    renderServices(registry);
  } catch (err) {
    showMessage(`Error: ${err.message}`);
  }
}

/**
 * Trigger a full re-probe of all groove targets.
 */
async function probeAll() {
  probeBtn.textContent = "Probing...";
  probeBtn.disabled = true;

  try {
    await browser.runtime.sendMessage({ type: "groove:discover" });
    // Wait briefly for results to propagate.
    await new Promise((r) => setTimeout(r, 500));
    await loadStatus();
  } catch (err) {
    showMessage(`Probe failed: ${err.message}`);
  }

  probeBtn.textContent = "Probe All";
  probeBtn.disabled = false;
}

// Wire up the probe button.
probeBtn.addEventListener("click", probeAll);

// Load status on popup open.
loadStatus();
