// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Regression tests for the repository badges rendered from README.adoc.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { test } from "node:test";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const readme = readFileSync(join(root, "README.adoc"), "utf8");

function bestPracticesProjectIds(source) {
  const references = source.matchAll(
    /https:\/\/(?:www\.)?bestpractices\.dev\/projects\/([0-9]+)(?=[/?#"'\]\s]|$)/g,
  );
  return [...new Set([...references].map((match) => match[1]))];
}

test("README does not claim an OpenSSF Best Practices project", () => {
  assert.deepEqual(bestPracticesProjectIds(readme), []);
  assert.doesNotMatch(readme, /OpenSSF Best Practices/i);
});

test("regression guard detects the exact badge removed from the README", () => {
  const removedBadge =
    'nimage:https://www.bestpractices.dev/projects/8509/badge[OpenSSF Best Practices,link="https://www.bestpractices.dev/projects/8509"]';

  assert.deepEqual(bestPracticesProjectIds(removedBadge), ["8509"]);
  assert.match(removedBadge, /OpenSSF Best Practices/i);
});

test("project detection covers badge and link variants without false positives", () => {
  const fixture = [
    "image:https://bestpractices.dev/projects/1/badge[]",
    "link=https://www.bestpractices.dev/projects/999?source=readme",
    "https://www.bestpractices.dev/en/about",
    "https://www.bestpractices.dev/projects/not-a-number/badge",
  ].join("\n");

  assert.deepEqual(bestPracticesProjectIds(fixture), ["1", "999"]);
});

test("README retains this repository's distinct OpenSSF Scorecard badge", () => {
  assert.match(
    readme,
    /^image:https:\/\/api\.scorecard\.dev\/projects\/github\.com\/metadatastician\/groove\/badge\[OpenSSF Scorecard,link="https:\/\/scorecard\.dev\/viewer\/\?uri=github\.com\/metadatastician\/groove"\]$/m,
  );
  assert.deepEqual(bestPracticesProjectIds(readme), []);
});
