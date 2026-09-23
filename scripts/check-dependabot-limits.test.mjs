// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Regression tests for the per-ecosystem Dependabot pull-request caps.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const config = Bun.YAML.parse(
  readFileSync(join(root, ".github", "dependabot.yml"), "utf8"),
);

const expectedLimits = new Map([
  ["github-actions", 2],
  ["mix", 3],
  ["npm", 3],
  ["pip", 3],
]);

function updatesByEcosystem(updates) {
  assert.ok(Array.isArray(updates), "Dependabot config must contain an updates array");

  const indexed = new Map();
  for (const update of updates) {
    const ecosystem = update?.["package-ecosystem"];
    assert.equal(typeof ecosystem, "string", "each update must name a package ecosystem");
    assert.ok(!indexed.has(ecosystem), `duplicate Dependabot update for ${ecosystem}`);
    indexed.set(ecosystem, update);
  }
  return indexed;
}

function assertExpectedLimits(updates) {
  const indexed = updatesByEcosystem(updates);

  for (const [ecosystem, expected] of expectedLimits) {
    assert.ok(indexed.has(ecosystem), `missing Dependabot update for ${ecosystem}`);
    const actual = indexed.get(ecosystem)["open-pull-requests-limit"];
    assert.ok(
      Number.isInteger(actual) && actual >= 0,
      `${ecosystem} limit must be a non-negative integer`,
    );
    assert.equal(actual, expected, `${ecosystem} has the wrong pull-request limit`);
  }
}

test("configured ecosystems use the estate pull-request caps", () => {
  assert.equal(config.version, 2);
  assertExpectedLimits(config.updates);
});

test("zero remains a valid boundary for the intentionally disabled Cargo queue", () => {
  const cargo = updatesByEcosystem(config.updates).get("cargo");

  assert.ok(cargo, "missing Dependabot update for cargo");
  assert.equal(cargo["open-pull-requests-limit"], 0);
});

test("an ecosystem outside the cap policy may remain uncapped", () => {
  const nix = updatesByEcosystem(config.updates).get("nix");

  assert.ok(nix, "missing Dependabot update for nix");
  assert.equal(Object.hasOwn(nix, "open-pull-requests-limit"), false);
});

test("cap validation rejects missing and malformed limits", () => {
  const invalidValues = [undefined, "3", -1, 2.5];

  for (const invalid of invalidValues) {
    const updates = structuredClone(config.updates);
    const npm = updates.find((update) => update["package-ecosystem"] === "npm");

    if (invalid === undefined) {
      delete npm["open-pull-requests-limit"];
    } else {
      npm["open-pull-requests-limit"] = invalid;
    }

    assert.throws(
      () => assertExpectedLimits(updates),
      /npm limit must be a non-negative integer/,
    );
  }
});

test("cap validation rejects duplicate ecosystem blocks", () => {
  const updates = structuredClone(config.updates);
  updates.push(structuredClone(updates.find((update) => update["package-ecosystem"] === "pip")));

  assert.throws(() => assertExpectedLimits(updates), /duplicate Dependabot update for pip/);
});
