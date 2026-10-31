---
id: T-2895
title: "Fuzz target bitcode_workbook: decoded bitcode through Model::from_bytes never panics"
milestone: M12
lane: harness
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-2913]
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
A cargo-fuzz target `bitcode_workbook` decodes arbitrary bytes as a `Workbook` through `Model::from_bytes` and, when that
loads, runs `evaluate()` and a few reads and edits, with no panic.

## Context
SPEC-0026 decision 5. Directive 5: decoded bitcode is untrusted. No fuzz target decodes bitcode today; the existing
targets are fuzz/fuzz_targets/{formula_eval,formula_parse,xlsx_import}.rs. Follow xlsx_import.rs's layout and
registration (fuzz/Cargo.toml). Seed the corpus with `Model::to_bytes()` of a few small workbooks (a formula, a style, a
shared formula, two sheets), since random bytes rarely decode. Depends on T-2913, which turns the known panics into `Err`.
The target also covers what the validator cannot: `bitcode::decode` allocates from lengths in the buffer before any
check runs, so a run that finds a decode-time allocation blow-up files it as its own task (bitcode's own limits, or a
size cap on the input).

## Acceptance criteria
- [x] `fuzz/fuzz_targets/bitcode_workbook.rs` exists and is registered, with a seed corpus of engine-written workbooks.
- [x] A local run of at least 10 minutes finds no crash, or each crash found is filed as a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- CI scheduling of long fuzz runs (M12 release work).

## Outcome
**What the target does.** `fuzz/fuzz_targets/bitcode_workbook.rs` decodes arbitrary bytes through
`Model::from_bytes`. When they load, it:
- evaluates;
- reads three cells on each of up to four sheets;
- types a formula and a dynamic array;
- inserts a row and deletes a column;
- evaluates again;
- saves with `to_bytes` and reloads.

**Seeds and nightly runs.** Seeds are `Model::to_bytes()` of five small engine-written workbooks: numbers and a
formula; text, a boolean, an error and a percentage; a shared formula; a dynamic array with a `#` reference; two
sheets. The fuzz crate's `bitcode_seeds` binary writes them, `fuzz/seed_corpus.py` runs it, and the target joins the
nightly matrix in `.github/workflows/fuzz.yml` with `-max_len=65536`.

**Local runs** (nightly toolchain, ASan, as the workflow runs them):
- **First run.** It crashed after 207k runs on a `debug_assert!` in `model.rs` ("Larger-than-1x1 array reached
  scalar-context cell"): a decoded plain formula cell whose formula spills.
- **Fork-mode run.** 638 s with `-fork=2 -ignore_crashes=1`, about 2.3M runs, coverage 18,161. It found one more
  input with the same cause and no other crash, timeout or OOM.
- **Filed as T-3467 (P1)**, with the minimized 427-byte input. Release builds give `#VALUE!` there, no panic.

**Left out.** `fuzz/Cargo.lock` is stale: it pins `zip` 0.6.6 against the workspace's 9.0.3, so every fuzz build
re-resolves it. Refreshing it would pull in `zip` 9.2.0, newer than the workspace's, so it is left out of this task's
change.
