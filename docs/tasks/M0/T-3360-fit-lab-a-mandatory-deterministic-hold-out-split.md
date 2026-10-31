---
id: T-3360
title: "Fit lab: a mandatory deterministic hold-out split, with train and test counts side by side (decision 0188 R4.1)"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every fit-lab comparison splits its cells deterministically (about 30% held out, by case or cell hash), scores every
candidate on train and test separately, and accepts a candidate only if its test exact rate is within a stated margin
of its train rate and it beats the correctly rounded candidate on test.

## Context
- Decision 0188 R4.1 (owner's 2026-10-31 assessment): the fit lab has no hold-out, fits are scored on the cells they are
  tuned on (gated gen-m9 included), and functions fitted that way generalize badly to fresh points (T.INV 30% against
  69% on the public cases, T.INV.2T 14% against 51%, F.INV 63% against 78%).
- Code: tools/src/rustsheet_tools/fitlab, docs/architecture/fit-lab.md.

## Acceptance criteria
- [x] The split is deterministic and documented (hash, seed, ratio); the same cell is always on the same side.
- [x] The report shows train and test counts (bit-exact, within 4 ulp, within 15 digits) side by side for every
      candidate, the correctly rounded one included, and the accept verdict with the margin used.
- [x] Tests cover the split's stability and the accept rule (a candidate that wins on train but not on test is refused).
- [x] docs/architecture/fit-lab.md describes the split and the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-running existing fits (T-3365 and the generalization tasks do that).

## Outcome
- `fitlab/holdout.py`: a mandatory split. A cell is held out when sha256(`rustsheet-fitlab-holdout/v1` NUL case NUL
  `Sheet!A1`) has its first 8 bytes big-endian below 300 mod 1000, so about 30% of cells; the side depends on nothing
  else. The accept rule: test exact rate at most `Fit.margin` (default 5 points, compared as exact fractions) below
  train, and more test bit-exact cells than `correctly rounded`, with the test within-4-ulp count breaking a tie. A fit
  without that candidate, or without test cells, accepts nothing.
- `fit.run` scores every candidate on train and test separately; the whole-sweep score is their merge
  (`score.merged`), so no cell is evaluated twice. `score.Score` gains a within-15-digits count
  (`within_fifteen_digits`, a port of the harness's `closeness.rs` rule).
- The report has a new hold-out table: train and test exact, within 4 ulp and within 15 digits per candidate (the
  engine and the correctly rounded one always shown), with the verdict and the margin. The JSON gains `holdout`, plus
  per-candidate `train`, `test` and `verdict`. Example, `gamma-int`: 57 of 171 cells held out; correctly rounded
  test 14/57 exact; "double: product downward" train 63/114 (55.3%), test 29/57 (50.9%), accept.
- `tests/test_fitlab_holdout.py` covers:
  - split stability: order, other cells and values do not move a cell, and 20 cells' sides are pinned;
  - the ratio;
  - the accept rule: a train winner that loses on test is refused, plus the margin boundary, the tie-break, and no
    reference or no test cells;
  - merge equivalence, the 15-digit rule, and the report.
- docs/architecture/fit-lab.md has a new section, "Hold-out split and accept rule".
- Eval delta: none (tools only).
