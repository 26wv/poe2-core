# ADR 0023: Node numbers follow JSON.parse

Status: accepted, 2026-10-24

Amends ADR 0017's "native objects" wording: Node no longer uses napi's `serde-json` feature, and `nextLine` no longer returns an `Option<serde_json::Value>`.

**Context.** ADR 0017 hands Node the core's JSON shape as native objects. With default features (no `napi6`), napi's `serde-json` conversion turns a `u64` up to `i64::MAX` into a JS number, exact only up to 2^53 and rounded above it, and a larger `u64` into a string. That means `seed` and `ms_counter` could arrive as either a number or a string depending on their value. A JS consumer of `poe2 parse --json` gets plain numbers from `JSON.parse`.

**Decision.**
- The Node binding serialises each value with `serde_json::to_string` and builds the JS value by calling the global `JSON.parse` on that text.
- Every number is a plain JS `number`, never a string or a BigInt. `napi6`/BigInt stays off, and napi's `serde-json` feature is dropped.
- Integers above 2^53 lose precision exactly as `JSON.parse` loses it. Only pathological `seed` or `ms_counter` values get that large.
- The Python binding keeps exact ints, because Python's `json.loads` has arbitrary-precision integers.

**Considered options.**
- napi's `serde-json` conversion: mixed number and string types.
- Turning on BigInt (`napi6`): a second numeric type for consumers to handle, and it differs from `poe2 parse --json` read by `JSON.parse`.
- Converting each `u64` to `f64` before napi's conversion: the same result as calling `JSON.parse`, but it needs a tree walk.

**Consequences.** Node objects equal `JSON.parse` of the CLI's output. The generated `index.d.ts` now types the results as `unknown`. Each value is parsed twice, once by serde_json and once by `JSON.parse`, which is negligible at log rates.
