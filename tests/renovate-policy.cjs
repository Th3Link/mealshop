const assert = require('node:assert/strict');
const { createRequire } = require('node:module');
const { join } = require('node:path');
const { readFileSync } = require('node:fs');
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

const nodeVersion = renovate('./dist/modules/versioning/node').api;
const { getRegexPredicate } = renovate('./dist/util/string-match');
const { parse } = renovate('yaml');
const manifest = require('../clients/nextcloud/package.json');
const lock = require('../clients/nextcloud/package-lock.json');
const nodeRules = (config.packageRules ?? []).filter((candidate) =>
  candidate.matchPackageNames?.includes('node'));
assert.equal(nodeRules.length, 1, 'Node updates need an explicit Nextcloud compatibility boundary');
const [nodeRule] = nodeRules;
assert.deepEqual(nodeRule.matchPackageNames, ['node']);
assert.deepEqual(nodeRule.matchDatasources, ['node-version', 'docker']);
const nodeAllowed = getRegexPredicate(nodeRule.allowedVersions);
assert.equal(typeof nodeAllowed, 'function');
for (const [version, allowed] of [
  ['24', true], ['24.15.0', true], ['24.23.1', true], ['v24.23.1', true],
  ['24-alpine', true], ['24.23.1-alpine3.24', true],
  ['25.0.0', false], ['26', false], ['26.13.1', false],
  ['26-alpine', false], ['240-alpine', false], ['v26.13.1', false],
]) {
  assert.equal(nodeAllowed(version), allowed, version);
}
const minimum = /^\^(\d+\.\d+\.\d+)$/.exec(manifest.engines.node)?.[1];
assert.ok(minimum, 'Declare the supported Node minimum in engines.node');
assert.ok(nodeAllowed(minimum));
assert.equal(lock.packages[''].engines.node, manifest.engines.node);
assert.equal(manifest.volta?.node, undefined, 'engines.node is the canonical CI version source');
assert.equal(manifest.devEngines?.runtime, undefined, 'engines.node is the canonical CI version source');
for (const [path, dependency] of Object.entries(lock.packages)) {
  if (path.includes('node_modules/@nextcloud/') && dependency.engines?.node) {
    assert.ok(nodeVersion.matches(minimum, dependency.engines.node), `${path} must support ${minimum}`);
  }
}
const dockerfile = readFileSync(join(__dirname, '../Dockerfile'), 'utf8');
const tag = /^FROM node:([^@\s]+)@sha256:[a-f0-9]{64} AS web$/m.exec(dockerfile)?.[1];
assert.ok(tag, 'The web builder must remain digest-pinned');
assert.ok(nodeAllowed(tag), tag);
assert.equal(tag.split('-')[0], minimum.split('.')[0]);
const workflow = parse(readFileSync(join(__dirname, '../.github/workflows/ci.yml'), 'utf8'));
let nodeSteps = 0;
for (const job of Object.values(workflow.jobs)) {
  for (const step of job.steps ?? []) {
    if (!step.uses?.startsWith('actions/setup-node@')) continue;
    nodeSteps++;
    assert.equal(step.with['node-version-file'], 'clients/nextcloud/package.json');
    assert.equal(step.with['node-version'], undefined, 'A literal must not override the canonical version');
  }
}
assert.equal(nodeSteps, 2);
console.log('PASS: Node/Docker updates, both CI consumers and locked Nextcloud engines share a supported toolchain');
