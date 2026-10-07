---
id: T-1067
title: "Nightly fuzz cannot build: cargo-fuzz defaults to the musl target"
milestone: M1
lane: harness
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
The nightly fuzz workflow (`.github/workflows/fuzz.yml`) builds and runs all three targets
(formula_parse, formula_eval, xlsx_import) for the full 20 minutes, so a red run means a real crash.

## Context
The first scheduled run (Actions run 37568817512, 2026-10-07, main dc7b693) failed on every target
about 2 s into "fuzz for 20 minutes", before any input ran, and uploaded no crash artifacts. The log
shows `cargo build ... --target x86_64-unknown-linux-musl` then `E0463 can't find crate for core`
(`could not compile cfg-if`): the cargo-fuzz binary from `taiki-e/install-action` is a musl build,
and cargo-fuzz defaults `--target` to the triple it was built for, which the nightly toolchain
installed by `dtolnay/rust-toolchain` does not have (and ASan does not support musl anyway).
Likely fix: pass the host triple explicitly (`cargo fuzz run --target x86_64-unknown-linux-gnu ...`,
or derive it from `rustc -vV`), and document the same in decision 0035's local command if needed.
This is a CI change (protected path): it needs a decision-record note and explicit approval from
reviewer-architecture and reviewer-security. Until fixed, directive-5 fuzzing gives no signal.

## Acceptance criteria
- [x] A `workflow_dispatch` run of the fuzz workflow on the PR head (or the first nightly after
      merge) gets past the build and fuzzes each target; any crash it finds is filed per
      docs/process/agent-workflow.md.
- [x] The local reproduction command in decision 0035 / fuzz docs works on a glibc host.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing crashes the fuzzer finds once it runs (one P0 task each).

## Outcome
`fuzz.yml` passes `--target "$(rustc -vV | sed -n 's/^host: //p')"`. The cause was reproduced locally:
`--target x86_64-unknown-linux-musl` gives E0463. With the host triple, all three targets build and fuzz.
The `workflow_dispatch` run on the PR head (Actions run 37589979810) got past the build and fuzzed every
target. formula_eval found a real timeout after about 17 minutes (`LOOKUP(20,A:A,B:B%%…%)`), filed as
P0 T-1078. Decision 0035 is amended, and the steward reproduction command in agent-workflow.md points
at the note. Eval delta: none (CI-only).
