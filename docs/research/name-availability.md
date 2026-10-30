# Name availability: `poe2-core` / `poe2-cli`

Checked 2026-10-23 with `curl`, User-Agent `poe2-core-name-check (local dev)`.
404 means no package by that name exists.

| Registry  | URL                                          | HTTP | Result |
|-----------|----------------------------------------------|------|--------|
| crates.io | https://crates.io/api/v1/crates/poe2-core     | 404  | free   |
| crates.io | https://crates.io/api/v1/crates/poe2_core     | 404  | free   |
| crates.io | https://crates.io/api/v1/crates/poe2-cli      | 404  | free   |
| crates.io | https://crates.io/api/v1/crates/poe2_cli      | 404  | free   |
| npm       | https://registry.npmjs.org/poe2-core          | 404  | free   |
| npm       | https://registry.npmjs.org/poe2core           | 404  | free   |
| npm       | https://registry.npmjs.org/poe2_core          | 404  | free   |
| npm       | https://registry.npmjs.org/poe2.core          | 404  | free   |
| PyPI      | https://pypi.org/pypi/poe2-core/json          | 404  | free   |
| PyPI      | https://pypi.org/pypi/poe2core/json           | 404  | free   |

Why the variants: crates.io treats `-` and `_` as the same name. npm rejects a new name
that matches an existing one once punctuation is removed, so the common punctuation variants
were checked. PyPI normalises `-`, `_` and `.`, so `poe2-core` also covers `poe2_core` and
`poe2.core`. PyPI also rejects names that match an existing project after dropping separators
and folding look-alike characters (o→0, i/l→1). Only the separator-free form `poe2core` was
checked.

Response bodies: crates.io returned ``{"errors":[{"detail":"crate `poe2-core` does not exist"}]}``;
npm returned `{"error":"Not found"}`.

Positive controls, to show that 404 is a real "not found" and not a blocked request:
`crates.io/api/v1/crates/serde`, `registry.npmjs.org/react` and `pypi.org/pypi/requests/json`
all returned 200.

## Decision

Keep the names. Library crate, npm package and PyPI project: `poe2-core`. CLI crate:
`poe2-cli`, which installs the binary `poe2`.

Caveat: a name is only held once something is published under it. Re-run these checks
before the first publish of each package.
