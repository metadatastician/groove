// SPDX-License-Identifier: MPL-2.0
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
test('repository settings use groove identity and delegate branch protection', () => {
  const settings = Bun.YAML.parse(read('.github/settings.yml'));
  assert.equal(settings.repository.name, 'groove');
  assert.equal(settings.repository.homepage, '');
  assert.match(settings.repository.description, /^Rust protocol for zero-configuration service discovery/);
  assert.equal(settings.repository.default_branch, 'main');
  assert.equal(Object.hasOwn(settings, 'branches'), false);
});
test('GitHub community configuration contains no inherited repository identity', () => {
  for (const path of readdirSync(new URL('../.github/', import.meta.url), { recursive: true, withFileTypes: true })) {
    if (!path.isFile()) continue;
    const text = readFileSync(`${path.parentPath}/${path.name}`, 'utf8');
    assert.ok(!text.includes(['paint', 'type'].join('-')), path.name);
  }
});
test('scanner keeps three-event coverage, immutable validated pin and no secrets', () => {
  const workflow = Bun.YAML.parse(read('.github/workflows/secret-scanner.yml'));
  assert.deepEqual(Object.keys(workflow.on).sort(), ['merge_group', 'pull_request', 'push']);
  assert.deepEqual(workflow.jobs.scan.permissions, { contents: 'read' });
  assert.equal(Object.hasOwn(workflow.jobs.scan, 'secrets'), false);
  assert.equal(workflow.jobs.scan.uses, 'metadatastician/standards/.github/workflows/secret-scanner-reusable.yml@b8f60511e76c9015d86990317162dce653c92fe1');
});
