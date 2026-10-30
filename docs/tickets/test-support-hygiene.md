# Test support: FakeSystem edges, temp dirs, known-limit comments

**Context.** Found in the v0.1.0 code review. test_support.rs:58-66 asserts nothing about `b`. `FakeSystem::add` silently stores nodes beneath an existing link or file; untested edges are `modified` on a dir, `list_dir` on a file, and a read through a link to a file. FakeSystem (`split`, fake.rs:123-135) treats a UNC `\\server\share` root as a drive with no `Known limit:` comment, and the two "copied from" test helpers (poe2-cli/tests/common/mod.rs:10, parse_memory.rs:2) don't say why they are copies. Test temp dirs use predictable `/tmp/poe2-*-test-<pid>-<n>` names with `remove_dir_all` then `create_dir_all` (test_support.rs:14, poe2-cli/tests/common/mod.rs:17).

**Problem.** The fake can model impossible trees, and on a shared host another user could pre-create a test dir and swap fixtures.

**Acceptance criteria.**
- The test_support test asserts `b` is empty and removed on drop.
- `FakeSystem::add` asserts each ancestor is a dir; the edges above have tests.
- Temp dirs come from `CARGO_TARGET_TMPDIR` or `create_dir` (fail if it exists).
- `Known limit:` comments mark the UNC-as-drive limit and the copied helpers.

**Rank.** nice
