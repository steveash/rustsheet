# 0035 — Nightly fuzzing workflow

- **Status:** accepted (2026-10-06, T-0021)
- **Context:** Formulas and workbooks are untrusted (CLAUDE.md directive 5), but panics and
  stack overflows were found by hand (T-0931). Building the fuzz targets found another one in
  minutes: `column_to_number` overflowed `i32` on letter runs such as `Data!ISERROR(…)` (a panic
  with debug assertions; in release the value wrapped, so `NKJDCLBT1` lexed as column 1584).
  Fuzzing needs a nightly toolchain and libFuzzer, which the stable, pinned workspace must not
  depend on, and a CI workflow, which is a protected path.
- **Options considered:**
  - *Fuzz targets as a workspace member.* Pulls nightly-only flags and libFuzzer into
    `make check`; rejected.
  - *Fuzz on every PR.* Minutes of CPU per PR for little marginal coverage; rejected for now.
  - *A separate `fuzz/` crate (its own workspace, own nightly `rust-toolchain.toml`) run by a
    nightly workflow.*
- **Decision:** `fuzz/` holds cargo-fuzz targets (`formula_parse`, `formula_eval`,
  `xlsx_import`) that use only public engine/facade APIs. `.github/workflows/fuzz.yml` runs each
  for 20 minutes nightly (and on `workflow_dispatch`) with `permissions: contents: read`, no
  `pull_request_target`, `persist-credentials: false`, and no secrets; a crash, timeout (30 s per
  input) or out-of-memory (2 GiB) fails the job and uploads only `fuzz/artifacts/<target>/`.
  The grown corpus is kept in the Actions cache (written only by runs on `main`). The steward
  files one task per distinct crash (steward skill; docs/process/agent-workflow.md).
- **Consequences:** `make check` is unchanged and does not build `fuzz/`; an API change that
  breaks a target turns the nightly fuzz job red and is fixed like any other crash task. Each
  fix of a fuzz finding adds the reproducer as a unit test. Running locally:
  `fuzz/seed_corpus.py`, then
  `cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" <target> corpus/<target> -- -dict=dict/formula.dict`.
- **Revision (2026-10-07, T-1067):** the first nightly run failed before fuzzing: the prebuilt
  cargo-fuzz from `taiki-e/install-action` is a musl binary and defaults `--target` to
  `x86_64-unknown-linux-musl`, which the nightly toolchain does not have (E0463; ASan does not
  support musl either). The workflow and the local command now pass the host triple from
  `rustc -vV` explicitly. Permissions, triggers and secrets are unchanged.
- **Revision (2026-10-30, T-3163):** capped formula seeds, a `cargo fuzz cmin` before the cache save and a `v2`
  cache key; see decision 0171.
