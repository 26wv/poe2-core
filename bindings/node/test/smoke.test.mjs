import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';
const poe2 = createRequire(import.meta.url)('../lib.js');
const LINE = '2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12';
test('discover returns the five ranked lists without reading the real home', () => {
  process.env.HOME = mkdtempSync(join(tmpdir(), 'poe2-node-'));
  for (const k of ['XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'WINEPREFIX', 'POE2_INSTALL_DIR', 'POE2_USER_DIR']) delete process.env[k];
  const d = poe2.discover();
  assert.deepEqual(Object.keys(d).sort(), ['build_planner', 'client_log', 'filters', 'installs', 'user_dir']);
  assert.ok(d.installs.every((c) => typeof c.value === 'string' && typeof c.exists === 'boolean'));
});
test('parseLine returns the CLI JSON shape', () => {
  assert.deepEqual(poe2.parseLine(LINE), {
    timestamp: '2026-01-02T03:04:05',
    prefix: { ms_counter: 123456789, hex_id: '1a2b3c4d', log_level: 'INFO', log_source: 'Client', pid: 4242 },
    event: { type: 'LevelUp', name: 'ExileOne', class: 'Smith of Kitava', level: 12 },
  });
  assert.equal(poe2.parseLine(''), null);
});
const area = (seed) => `2026/01/02 03:00:01 123457789 1a2b3c4e [DEBUG Client 4242] Generating level 1 area "G1_1" with seed ${seed}`;
test('u64 fields are numbers with JSON.parse semantics', () => {
  assert.equal(poe2.parseLine(area('4294967295')).event.seed, 4294967295);
  const big = poe2.parseLine(area('18446744073709551615')).event.seed;
  assert.equal(typeof big, 'number');
  assert.equal(big, JSON.parse('18446744073709551615'));
});
