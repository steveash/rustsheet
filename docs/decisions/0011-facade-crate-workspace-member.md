# 0011 — Add the `rustsheet` facade crate to the workspace

- **Status:** accepted (2026-10-03)
- **Context:** SPEC-0001 (public API v0) puts the typed facade in `crates/rustsheet`. The root
  `Cargo.toml` is a protected path (it carries the workspace lint policy), and adding a member
  changes it. T-0113 creates the crate.
- **Decision:** The only change to `Cargo.toml` is the new `crates/rustsheet` entry in
  `[workspace] members`. The crate opts into the workspace lints (`[lints] workspace = true`);
  the lint policy itself, `[workspace.lints]`, is untouched, with no new `allow`s.
- **Consequences:** Later API tasks (T-0102, T-0103, T-0104) add code to this crate without
  further `Cargo.toml` changes. Reviewers (architecture, security) confirm the diff there is
  limited to the member line.
