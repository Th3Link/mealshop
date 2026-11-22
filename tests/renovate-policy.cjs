const assert = require('node:assert/strict');
const { createRequire } = require('node:module');
const { join } = require('node:path');
const config = require('../renovate.json');
const renovate = createRequire(join(process.cwd(), 'package.json'));
const cargo = renovate('./dist/modules/versioning/cargo').api;

const rules = (config.packageRules ?? []).filter((rule) =>
  rule.matchManagers?.includes('cargo') && rule.matchPackageNames?.includes('age'));
assert.equal(rules.length, 1);
const [rule] = rules;
assert.deepEqual(rule.matchManagers, ['cargo']);
assert.deepEqual(rule.matchPackageNames, ['age']);
for (const [version, allowed] of [
  ['0.11.5', true], ['0.11.6', true],
  ['0.12.0', false], ['0.12.1', false], ['0.13.0', false],
  ['0.12.0-beta.1', false],
]) {
  assert.equal(cargo.matches(version, rule.allowedVersions), allowed, version);
}
console.log('PASS: actual Renovate Cargo matching keeps safe patches and rejects the HPKE/P-256 series');
