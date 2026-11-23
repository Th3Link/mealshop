import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { lstatSync, mkdtempSync, readdirSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const app = realpathSync(fileURLToPath(new URL('..', import.meta.url)))
const compilers = {
  native: join(app, 'node_modules/typescript/bin/tsc'),
  vue: join(app, 'scripts/check-vue.mjs'),
}

function run(engine, args) {
  const result = spawnSync(process.execPath, [compilers[engine], ...args], {
    cwd: app,
    encoding: 'utf8',
    timeout: 30000,
  })
  assert.ifError(result.error)
  assert.equal(result.signal, null)
  return result
}

function fixture(t, extension) {
  const directory = mkdtempSync(join(app, '.typecheck-test-'))
  const created = lstatSync(directory)
  t.after(() => {
    const target = realpathSync(directory)
    const current = lstatSync(target)
    assert.equal(dirname(target), app)
    assert.equal(current.dev, created.dev)
    assert.equal(current.ino, created.ino)
    rmSync(target, { recursive: true, force: true })
  })
  const project = join(directory, 'tsconfig.json')
  writeFileSync(
    project,
    JSON.stringify({
      extends: join(app, 'tsconfig.json'),
      include: [`input.${extension}`],
    }),
  )
  return { project, source: join(directory, `input.${extension}`) }
}

test('the native and Vue checks use their explicit official compiler generations', () => {
  for (const [engine, version] of [
    ['native', /Version 7\./],
    ['vue', /Version 6\./],
  ]) {
    const result = run(engine, ['--version'])
    assert.equal(result.status, 0, result.stdout + result.stderr)
    assert.match(result.stdout, version)
  }
})

test('the real project keeps full Vue coverage and native checking of ordinary modules', () => {
  const native = run('native', ['--noEmit', '--project', 'tsconfig.native.json', '--listFilesOnly'])
  const vue = run('vue', ['--noEmit', '--listFilesOnly'])
  assert.equal(native.status, 0, native.stdout + native.stderr)
  assert.equal(vue.status, 0, vue.stdout + vue.stderr)
  const listedNative = native.stdout.replaceAll('\\', '/')
  const listedVue = vue.stdout.replaceAll('\\', '/')
  for (const file of readdirSync(join(app, 'src'), { recursive: true })) {
    if (!/\.(ts|vue)$/.test(file)) continue
    const path = join(app, 'src', file).replaceAll('\\', '/')
    assert.ok(listedVue.includes(path), `Vue must check ${file}`)
    if (file.endsWith('.ts') && file !== 'main.ts') {
      assert.ok(listedNative.includes(path), `TypeScript 7 must check ${file}`)
    }
  }
})

for (const engine of ['native', 'vue']) {
  test(`${engine} accepts valid scripts and rejects incorrect exact-quantity types`, (t) => {
    const { project, source } = fixture(t, 'ts')
    writeFileSync(source, "export const grams: string = '50'\n")
    const valid = run(engine, ['--noEmit', '--project', project])
    assert.equal(valid.status, 0, valid.stdout + valid.stderr)
    writeFileSync(source, 'export const grams: string = 50\n')
    const invalid = run(engine, ['--noEmit', '--project', project])
    assert.notEqual(invalid.status, 0)
    assert.match(invalid.stdout, /TS2322/)
  })
}

test('Vue still checks both SFC scripts and template bindings', (t) => {
  const { project, source } = fixture(t, 'vue')
  for (const [content, expected] of [
    [
      '<script setup lang="ts">const amount = 1</script><template>{{ amount.toFixed(1) }}</template>',
      null,
    ],
    [
      '<script setup lang="ts">const amount = 1</script><template>{{ amount.missingMethod() }}</template>',
      /TS2339.*missingMethod/,
    ],
    [
      '<script setup lang="ts">const amount: number = "wrong"</script><template>{{ amount }}</template>',
      /TS2322/,
    ],
  ]) {
    writeFileSync(source, content)
    const result = run('vue', ['--noEmit', '--project', project])
    if (expected) {
      assert.notEqual(result.status, 0)
      assert.match(result.stdout, expected)
    } else {
      assert.equal(result.status, 0, result.stdout + result.stderr)
    }
  }
})
