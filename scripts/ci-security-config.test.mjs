// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Regression tests for the security-sensitive CI configuration.

import assert from "node:assert/strict";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

async function readYaml(path) {
  return Bun.YAML.parse(await Bun.file(join(root, path)).text());
}

const [dependabot, codeql, scorecard] = await Promise.all([
  readYaml(".github/dependabot.yml"),
  readYaml(".github/workflows/codeql.yml"),
  readYaml(".github/workflows/scorecard.yml"),
]);

function dependabotUpdate(ecosystem) {
  const matches = dependabot.updates.filter(
    (update) => update["package-ecosystem"] === ecosystem && update.directory === "/",
  );
  assert.equal(matches.length, 1, `${ecosystem} must have exactly one root update entry`);
  return matches[0];
}

function assertCommitPinned(reference) {
  assert.match(
    reference,
    /^[^\s@]+\/[^\s@]+@[0-9a-f]{40}$/,
    `${reference} must use a full commit SHA`,
  );
}

test("Dependabot applies the intended pull-request limits", () => {
  const expectedLimits = new Map([
    ["github-actions", 2],
    ["mix", 3],
    ["npm", 3],
    ["pip", 3],
  ]);

  for (const [ecosystem, expectedLimit] of expectedLimits) {
    const limit = dependabotUpdate(ecosystem)["open-pull-requests-limit"];
    assert.equal(limit, expectedLimit, `${ecosystem} should use its configured limit`);
    assert.ok(Number.isInteger(limit) && limit > 0, `${ecosystem} limit must be positive`);
  }
});

test("Dependabot preserves the Cargo security-only update policy", () => {
  assert.equal(dependabotUpdate("cargo")["open-pull-requests-limit"], 0);
});

test("CodeQL actions are immutable and checkout credentials are not persisted", () => {
  const steps = codeql.jobs.analyze.steps;
  const checkout = steps.find((step) => step.name === "Checkout");
  const initialize = steps.find((step) => step.name === "Initialize CodeQL");
  const analyze = steps.find((step) => step.name === "Perform CodeQL Analysis");

  assert.ok(checkout, "CodeQL workflow must check out the repository");
  assert.ok(initialize, "CodeQL workflow must initialize analysis");
  assert.ok(analyze, "CodeQL workflow must perform analysis");

  assert.equal(
    checkout.uses,
    "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
  );
  assert.equal(checkout.with?.["persist-credentials"], false);
  assert.equal(
    initialize.uses,
    "github/codeql-action/init@cdf488f595d80d6e07e03d4674febd5ab45fa938",
  );
  assert.equal(
    analyze.uses,
    "github/codeql-action/analyze@cdf488f595d80d6e07e03d4674febd5ab45fa938",
  );

  for (const step of [checkout, initialize, analyze]) assertCommitPinned(step.uses);
});

test("Scorecard runs on its scheduled, push, and manual entry points", () => {
  assert.equal(scorecard.name, "Scorecard");
  assert.deepEqual(scorecard.on, {
    schedule: [{ cron: "0 0 * * 0" }],
    push: { branches: ["main", "master"] },
    workflow_dispatch: null,
  });
  assert.deepEqual(scorecard.concurrency, {
    group: "${{ github.workflow }}-${{ github.ref }}",
    "cancel-in-progress": true,
  });
});

test("Scorecard grants only the permissions required by its reusable workflow", () => {
  assert.deepEqual(scorecard.permissions, {
    actions: "read",
    contents: "read",
    "security-events": "write",
    "id-token": "write",
  });
});

test("Scorecard delegates to an immutable reusable workflow", () => {
  assert.deepEqual(scorecard.jobs.scorecard, {
    uses: "hyperpolymath/standards/.github/workflows/scorecard-reusable.yml@8750b94ac1bbe8c51ad13fe106669b13478f0b62",
  });
  assertCommitPinned(scorecard.jobs.scorecard.uses);
});
