# Lint the bindings in CI and widen the Python matrix

**Context.** Found in the v0.1.0 code review (only Python 3.12 in CI; `push` and `pull_request` both run every workflow). The binding crates are separate workspaces (ADR 0017), so ci.yml:24-25's `cargo fmt --all` and `clippy --workspace` never see them, and node.yml:31-33 and python.yml:34-36 only build and test. pyproject says Python >=3.9.

**Problem.** The FFI code is never formatted or linted in CI, although the core crates must pass `cargo fmt` and `clippy -D warnings`. The oldest supported Python is never tested. Same-repo PRs run every workflow twice.

**Acceptance criteria.**
- node.yml and python.yml run `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` in the binding folder.
- python.yml tests 3.9 and the newest supported Python.
- `push` is limited to `branches: [main]` in all three workflows, or the ticket records why the double run stays.

**Rank.** should
