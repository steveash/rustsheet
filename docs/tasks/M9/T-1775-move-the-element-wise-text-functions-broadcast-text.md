---
id: T-1775
title: "Move the element-wise text functions (broadcast_text, TextArg) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1771, T-1773, T-1785, T-1774, T-1932]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The element-wise text functions lift through the SPEC-0011 driver, with no result change. `TextArg` and `broadcast_text` are deleted, and the driver's size bound and text budget cover them. This is the last migration task, and it proves that no ad-hoc lift remains.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Today's lifting code" and Test plan "No ad-hoc lift remains".
- `broadcast_text` has 17 call sites: LEFT, RIGHT, MID, FIND/SEARCH, LEN, REPT, SUBSTITUTE, REPLACE, EXACT, TEXT, CODE/UNICODE/CHAR/UNICHAR, FILTERXML, BAHTTEXT, CONCATENATE, DATEVALUE/TIMEVALUE text.
- They must keep these rules:
  - the left-to-right first-failure rule (T-1022, arrays.md "Errors broadcast through `&`, comparisons, POWER and the element-wise functions"; probes/error-precedence-text);
  - REPLACE, MID and FIND coerce every argument before any range check (T-1056);
  - CONCATENATE's scalar-error short cut (arrays.md "CONCATENATE with a scalar error argument returns that error");
  - T-1771's ordering: the array-size check comes first (`#ERROR!`), and the 64 MiB text budget's error stays `#VALUE!`. The budget moves into the driver.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite. probes/error-precedence-text, error-arg-vs-range-check and concatenate-limit-spill keep passing, as do T-1771's size and budget tests.
- [ ] crates/rustsheet/benches/text.rs runs on main and on the branch in release and is within 5%; the numbers are in the Outcome.
- [ ] SPEC-0011's final grep check:
  - `rg -n "fn lift_numbers|fn lift_number_operands|fn lift_with_defaults|macro_rules! single_number_fn|fn lifted_arg|fn parity_predicate|fn broadcast_text|enum TextArg|fn lift_with_cost|fn is_lifted|enum CriteriaArg" crates/rustsheet-engine/src` finds nothing.
  - `rg -n "bcast_get|bcast_idx" crates/rustsheet-engine/src` finds only functions/lift/, arithmetic.rs and the spec's non-goal rows (IF, INDEX, GROUPBY/PIVOTBY).

  Both outputs are in the Outcome.
- [ ] `scripts/check_duplication.py` does not rise, and any drop is recorded with `--update`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New text lifts or changes in text semantics. TEXTSPLIT, TEXTBEFORE and TEXTAFTER delimiters (T-1271).

## Blocked because
The bench criterion (`crates/rustsheet/benches/text.rs` within 5% of main) is not met, and only the owner can relax it (decision 0077 did so for T-1785 alone). Everything else is done on branch `agent/quality/T-1775-text-lifts-on-driver`: `make check` green, eval 0 fixes and 0 regressions in every suite, reviewers' blocking findings fixed (excel-fidelity's filterxml-array-forms A9 regression; quality's duplication `--update`).

Measured on a noisy 4-core box, main 2b52f8cd (the branch point) vs the branch head, release, `text` bench, min of 11 recalcs, three alternating runs each:

| Case | main (ms) | branch (ms) | callgrind, case function inclusive |
|---|---|---|---|
| `A1&"x"` (control, untouched) | 77.0 / 84.4 / 81.0 | 76.2 / 79.9 / 75.8 | — |
| `MID(A1,2,3)` per row | 73.8 / 70.5 / (113.6 outlier) | 72.0 / 71.1 / 69.7 | 1.685G → 1.959G (+16%, about 211 instructions a call) |
| `LEFT(A1,2)` per row | 105.2 / 100.5 / 99.3 | 116.7 / 103.5 / 104.6 | 1.309G → 1.580G (+21%, about 208 a call) |
| CONCATENATE 255 x 130 units | 65.9 / 63.4 / 63.5 | 67.6 / 70.3 / 68.1 | 11.00G → 11.06G (+0.5%) |

Before the branch's three-argument fast path in `lift_call_with` (call.rs), MID was +56% in callgrind (+724 instructions a call, most of it building and dropping an 8-slot `ArgBuf`). What remains is mostly the driver's fixed per-call cost that T-1932 owns (about 165 instructions a call for DAY), plus about 40 of the text adapter's own (`lambda_argument_error`, `text_result`, a second `dyn` call). Resume when T-1932 lands (rerun the table), or when the owner records a decision like 0077 for this task.

### Status of the other acceptance criteria (for the resuming run)
- `make eval`: 0 fixes, 0 regressions in every suite (probes 1995 → 1995).
- First SPEC-0011 grep: no output.
- Second grep: `functions/lift/`, `arithmetic.rs` (and its `arithmetic/bcast_tests.rs`), the non-goals `logical/mod.rs` (IF) and `lookup_and_reference/index.rs` (INDEX), plus two loops outside this task: CHOOSE's `choose_array` (lookup_and_reference/mod.rs, T-1810) and IMAGE's `image_grid` (information/offline_errors.rs, T-1962).
- `scripts/check_duplication.py`: 3831 → 3742, recorded with `--update`.
- Result changes on unprobed edges, all in text.md "Element-wise text functions on the lift driver" (UNVERIFIED, T-1963): 1x1 array lifts, a LAMBDA beside an array keeps the array's shape, VALUETOTEXT's scalar error format over an array, legacy `RIGHT(range)` gets the `@`.

