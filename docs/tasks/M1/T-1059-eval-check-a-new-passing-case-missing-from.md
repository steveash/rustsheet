---
id: T-1059
title: "eval-check: a new passing case missing from the baseline must mark it dirty"
milestone: M1
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`make eval-check` fails (dirty baseline) when a case with all cells passing is not yet in the baseline, as quality-gates.md:28-29 requires.

## Context
Found in the T-1056 review: `probes/error-arg-vs-range-check` was absent from eval/baseline/probes.txt yet `make eval-check` exited 0. In crates/rustsheet-eval/src/baseline.rs `diff` (~142-169) a case in `new.checked` but not in `old.checked` with no failures never sets `is_dirty()`.

## Acceptance criteria
- [ ] A unit test shows a new all-passing case makes the diff dirty.
- [ ] CI's baseline-up-to-date check fails on such a PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
