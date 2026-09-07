// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// groove-client — shared, pure Groove client logic (no browser or node APIs).
// Used by the browser extension (vendored copy in
// clients/browser-extension/background/groove-client.vendor.js — keep in
// sync, CI checks) and available to harness/groove-harness.js.
//
// Wire dialect: SPEC §2.1.2 (application/groove+json, ADR 0002).

(function (root, factory) {
  if (typeof module !== "undefined" && module.exports) {
    module.exports = factory();
  } else {
    root.GrooveClient = factory();
  }
})(typeof self !== "undefined" ? self : this, function () {
  "use strict";

  /** Accept header per ADR 0002: JSON required, A2ML advertised at lower q. */
  function buildAcceptHeader() {
    return "application/groove+json, application/groove+a2ml;q=0.5, application/json;q=0.2";
  }

  /**
   * Parse and validate a JSON Groove manifest (SPEC §2.1.2).
   * @param {string} text - raw response body
   * @returns {{ok: true, manifest: Object} | {ok: false, errors: string[]}}
   */
  function parseManifestJson(text) {
    var errors = [];
    var manifest;
    try {
      manifest = JSON.parse(text);
    } catch (e) {
      return { ok: false, errors: ["invalid JSON: " + e.message] };
    }
    if (manifest === null || typeof manifest !== "object" || Array.isArray(manifest)) {
      return { ok: false, errors: ["manifest must be a JSON object"] };
    }
    if (manifest.groove_version !== "1") {
      errors.push('groove_version must be the string "1"');
    }
    if (typeof manifest.service_id !== "string" || !/^[a-z][a-z0-9_-]*$/.test(manifest.service_id)) {
      errors.push("service_id missing or not matching ^[a-z][a-z0-9_-]*$");
    }
    if (
      manifest.capabilities === null ||
      typeof manifest.capabilities !== "object" ||
      Array.isArray(manifest.capabilities)
    ) {
      errors.push("capabilities must be a JSON object (map), not an array");
    }
    if (manifest.consumes !== undefined && (!Array.isArray(manifest.consumes) ||
        manifest.consumes.some(function (c) { return typeof c !== "string"; }))) {
      errors.push("consumes must be an array of strings when present");
    }
    return errors.length ? { ok: false, errors: errors } : { ok: true, manifest: manifest };
  }

  /** Capability types offered by a parsed manifest. */
  function offeredTypes(manifest) {
    var caps = manifest.capabilities || {};
    var types = [];
    for (var key in caps) {
      if (Object.prototype.hasOwnProperty.call(caps, key) && caps[key] && caps[key].type) {
        types.push(caps[key].type);
      }
    }
    return types;
  }

  /**
   * Structural capability match: every consumed type must be offered.
   * @param {string[]} offers - capability types the provider offers
   * @param {string[]} consumes - capability types the consumer needs
   * @returns {{compatible: boolean, reasons: string[]}}
   */
  function capabilityMatch(offers, consumes) {
    var reasons = [];
    (consumes || []).forEach(function (c) {
      if (offers.indexOf(c) === -1) {
        reasons.push("consumer consumes '" + c + "' but provider does not offer it");
      }
    });
    return { compatible: reasons.length === 0, reasons: reasons };
  }

  /**
   * Does an offered semver satisfy a required constraint?
   * Supports "1.2.3" (exact-or-newer within major) and "1.0+" (at least).
   * Conservative: unknown shapes return false.
   */
  function versionSatisfies(offered, required) {
    if (required === undefined || required === null || required === "") return true;
    if (typeof required !== "string" || typeof offered !== "string") return false;
    var shape = /^(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*)){0,2}$/;
    var plus = /\+$/.test(required);
    var constraint = required.replace(/\+$/, "");
    if (!shape.test(offered) || !shape.test(constraint)) return false;
    var req = constraint.split(".").map(Number);
    var off = offered.split(".").map(Number);
    if (!req.concat(off).every(Number.isSafeInteger)) return false;
    while (req.length < 3) req.push(0);
    while (off.length < 3) off.push(0);
    if (off[0] !== req[0]) return plus ? off[0] > req[0] : false;
    if (off[1] !== req[1]) return off[1] > req[1];
    return off[2] >= req[2];
  }

  return {
    buildAcceptHeader: buildAcceptHeader,
    parseManifestJson: parseManifestJson,
    offeredTypes: offeredTypes,
    capabilityMatch: capabilityMatch,
    versionSatisfies: versionSatisfies,
  };
});
