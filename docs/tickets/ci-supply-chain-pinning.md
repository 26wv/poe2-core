# Pin CI actions to SHAs and build from the lockfiles

**Context.** Found in the v0.1.0 code review (including checkout@v4's Node 20 warning). Actions are pinned to tags (ci.yml:19-20, node.yml:24-25, :28, python.yml:24-25, :28) and `dtolnay/rust-toolchain@master` to a branch. No `cargo`, napi or maturin step passes `--locked` (ci.yml:25-26, node.yml:32, python.yml:34-35). `pip install maturin==1.15.0` has no hash. `actions/checkout` keeps `persist-credentials: true`.

**Problem.** A moved tag or a pushed commit runs code in CI with the repo token, and CI can silently test different crate versions than the committed Cargo.lock.

**Acceptance criteria.**
- Every action is pinned to a full commit SHA with a `# vX.Y.Z` comment; checkout moves to v5.
- Dependabot (`github-actions`) keeps the SHAs fresh.
- `cargo clippy`/`cargo test` use `--locked`; napi gets `-- --locked`; maturin builds locked (`[tool.maturin] locked = true` or a `cargo fetch --locked` step).
- maturin comes from a hashed requirements file (`pip install --require-hashes -r`).
- checkout sets `persist-credentials: false`; `npm ci --ignore-scripts` is used if the build allows it.

**Rank.** should
