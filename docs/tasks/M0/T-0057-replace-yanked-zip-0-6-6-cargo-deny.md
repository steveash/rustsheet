---
id: T-0057
title: "Replace yanked zip 0.6.6 (cargo deny advisories fails on main)"
milestone: M0
lane: quality
status: dropped
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`make deny` (cargo deny advisories) passes again: no yanked crate in Cargo.lock.

## Context
crates.io yanked zip 0.6.6 on 2026-10-16; `cargo update -p zip` cannot select a version because crates/rustsheet-xlsx, rustsheet-eval and rustsheet all require `zip = "0.6.6"` (default-features = false, features = ["deflate"]). The `hygiene` CI job fails on every PR (seen on PR #891). Pick a non-yanked zip release (a newer major needs API changes in the xlsx reader/writer), update the three Cargo.toml files and Cargo.lock with cargo, and fix call sites.

## Acceptance criteria
- [ ] `make check` (incl. `make deny`) green; no xlsx round-trip eval regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The zip upgrade itself (done in T-1668).

## Outcome
Dropped as a duplicate of T-1668, which upgraded zip to 9.0.3 on `main` (de25e70c) the same day.
`make deny` passes on `main` again.
