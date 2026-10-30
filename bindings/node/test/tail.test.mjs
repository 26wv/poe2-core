import { test, after } from 'node:test';
import assert from 'node:assert/strict';
import { appendFileSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const poe2 = createRequire(import.meta.url)('../lib.js');
const dir = mkdtempSync(join(tmpdir(), 'poe2-node-tail-'));
after(() => rmSync(dir, { recursive: true, force: true }));
// Every call below returns at once (ADR 0014), so the timeout only guards against a regression.
const opts = { timeout: 10_000 };
test('for...of yields the lines available now, then ends', opts, () => {
  const p = join(dir, 'Client.txt');
  writeFileSync(p, 'a\nb\n');
  const t = new poe2.LogTail(p, true);
  assert.deepEqual([...t].map((l) => l.event.raw), ['a', 'b']);
  appendFileSync(p, 'c\n');
  assert.deepEqual([...t].map((l) => l.event.raw), ['c']);
  assert.equal(t.nextLine(), null);
});
test('without fromStart it follows from the end', opts, () => {
  const p = join(dir, 'end.txt');
  writeFileSync(p, 'old\n');
  const t = new poe2.LogTail(p);
  assert.equal(t.nextLine(), null);
  appendFileSync(p, 'new\n');
  assert.equal(t.nextLine().event.raw, 'new');
  assert.equal(t.nextLine(), null); // EOF closes the handle (tail.rs), so after() can remove dir on Windows
});
test('a missing path throws', opts, () => {
  assert.throws(() => new poe2.LogTail(join(dir, 'missing.txt')));
});
test('u64 fields arrive as numbers (ADR 0023)', opts, () => {
  const p = join(dir, 'area.txt');
  writeFileSync(p, '2026/01/02 03:00:01 123457789 1a2b3c4e [DEBUG Client 4242] Generating level 1 area "G1_1" with seed 18446744073709551615\n');
  const t = new poe2.LogTail(p, true);
  const { prefix, event } = t.nextLine();
  assert.equal(typeof event.seed, 'number');
  assert.equal(event.seed, JSON.parse('18446744073709551615'));
  assert.equal(prefix.ms_counter, 123457789);
  assert.equal(t.nextLine(), null);
});
