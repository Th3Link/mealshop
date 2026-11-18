import assert from "node:assert/strict";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  lstatSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import test from "node:test";

const script = new URL("../scripts/npm_toolchain.mjs", import.meta.url);

for (const [description, value, accepted] of [
  ["current exact npm pin", "npm@12.2.3", true],
  ["another exact npm pin", "npm@10.9.2", true],
  ["missing pin", undefined, false],
  ["different package manager", "pnpm@12.2.3", false],
  ["version range", "npm@^12.2.3", false],
  ["shell syntax", "npm@12.2.3; echo unexpected", false],
  ["non-string metadata", ["npm@12.2.3"], false],
]) {
  test(description, (t) => {
    const temporaryRoot = realpathSync(tmpdir());
    const directory = mkdtempSync(join(temporaryRoot, "mealshop-npm-pin-"));
    const created = lstatSync(directory);
    t.after(() => {
      const target = realpathSync(directory);
      const current = lstatSync(target);
      assert.equal(dirname(target), temporaryRoot);
      assert.equal(current.dev, created.dev);
      assert.equal(current.ino, created.ino);
      rmSync(target, { recursive: true, force: true });
    });
    mkdirSync(join(directory, "scripts"));
    mkdirSync(join(directory, "clients/nextcloud"), { recursive: true });
    const target = join(directory, "scripts/npm_toolchain.mjs");
    copyFileSync(script, target);
    writeFileSync(
      join(directory, "clients/nextcloud/package.json"),
      JSON.stringify({ packageManager: value }),
    );
    const result = spawnSync(process.execPath, [target], { encoding: "utf8" });
    if (accepted) {
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.stdout, value);
    } else {
      assert.notEqual(result.status, 0);
      assert.equal(result.stdout, "");
      assert.match(result.stderr, /packageManager must pin npm exactly/);
    }
  });
}

test("the repository's declared pin is emitted independently of the working directory", () => {
  const declared = JSON.parse(
    readFileSync(new URL("../clients/nextcloud/package.json", import.meta.url)),
  ).packageManager;
  const result = spawnSync(process.execPath, [fileURLToPath(script)], {
    cwd: tmpdir(),
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, declared);
});
