---
id: T-3139
title: "rustsheet-gen --policy numeric-fp: an oracle-filled gen-m2 batch of near-zero cancellations through M2 functions at the 8-ULP snap boundary"
milestone: M2
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
gen-m2 gets seeded, oracle-filled cells in which M2 functions supply, wrap or consume a cancelling `+`/`-` whose
result lies near the near-zero snap boundary. Then the M2 exit gate (gen-m2 with no visibly-off cell) covers the
rule that every open M2 probe failure is about. Today gen-m2's random policy almost never builds such a
cancellation, so it is 100% while the probes keep finding snap failures.

## Context
- All 24 M0-M2 probe failures in the eval on 1c955d1cb are near-zero snap cells:
  - probes/t3094-snap-lambda-call-forms, 15 cells (T-3105);
  - probes/t3048-snap-conditional-sums, 8 cells (T-3066, T-3100);
  - probes/t3073-lookup-near-equal-approximate Sheet1!E8, 1 cell (T-3100).
  Each was found by a hand-built probe. gen-m2 (16,280 cells, 0 failures) has no cancellation pool. The k-ULP sweep
  `snap-addsub` (tools/src/rustsheet_tools/gen/numeric_rules.py, suite probes-numeric) uses bare
  operators and SUM only, and probes-numeric is not an M2 exit suite.
- docs/architecture/synthetic-generator.md §5.5 designs a `numeric_fp` policy ("arithmetic chains, cancellation,
  ROUND/TRUNC/MOD/INT, 15-digit boundaries"), but it is not implemented. "Implemented so far" lists random,
  numeric-rules, operators-refs, names-3d, tables and number-formats.
- The rule to aim at is docs/excel-semantics/numbers.md "Near-zero snapping":
  - snap iff |result| < 8 ULP of the smaller addend (the `+` side keeps k = 8, the `-` side snaps only to k = 3);
  - the snap follows the tree root: pickers, LET, unary signs and groups do not snap;
  - LAMBDA and defined-name bodies snap at their own root;
  - SUM, AVERAGE and AVERAGEA snap their final addition wherever they sit;
  - SUMIF, SUMIFS, AVERAGEIF(S) and SUMPRODUCT never snap;
  - array operands snap per element;
  - the operand-order question is still open (T-3100).
- Why opus: the operands must be exact doubles at chosen ULP distances (powers-of-two bases on an `Inputs` sheet,
  as numeric_rules.py does, since Excel cuts typed literals to 15 digits). k must straddle both sides' boundaries.
  The batch must place the cancellation in each structural position the rule distinguishes:
  - at the root, with the operands read through VLOOKUP, HLOOKUP, INDEX, MATCH-fed INDEX, MIN, MAX, ABS, ROUND
    and IF picks;
  - under an M2 wrapper: `IF(TRUE,x-b)`, `IFERROR(x-b,0)`, `ROUND(x-b,20)`, `ABS(x-b)`, `(x-b)&""`, `(x-b)=0`,
    `AND(x-b=0)`, `COUNTIF(r,x-b)`;
  - inside the summing functions: `SUM`, `AVERAGE`, `SUMIF`, `SUMIFS` over a range holding x and -b, with and
    without a trailing 0 addend;
  - in both operand orders.
  Each cell needs a `hypotheses` tag (`numeric-fp/<position>/<form>/base=../k=..`) so a failure points at its rule.
  The failures then have to be triaged to owners.

## Acceptance criteria
- [ ] `rustsheet-gen --policy numeric-fp --seed <n>` writes deterministic gen-m2 cases (tag M2, `hypotheses` per
      cell), reusing numeric_rules.py's base and ULP helpers (`BASES`, `_fmt_k`) instead of copying them. It
      covers the positions above for k = 0..9 on at least four power-of-two bases and 1.5, with both signs and
      both operand orders, in at least 1,500 formula cells. Only M2 functions and operators are called.
- [ ] tools tests: the policy is deterministic for a seed, every formula parses with spec2xlsx, and only catalog
      M0-M2 functions appear (tools/tests/test_generator.py or a new test file). `uv run` ruff and mypy strict pass.
- [ ] The batch is oracle-filled on excel-graph (`rustsheet-oracle fill -b excel-graph --suite gen-m2`) and
      committed with its sidecars. `make eval-baseline` records it as new coverage with no regressions in other
      cases.
- [ ] Every failing cell has an open M2 owner. Cells that fit an open task's rule are named in that task (T-3100
      for operand order, T-3066 for AVERAGE). Any other cell gets a new engine or functions-* M2 task.
      `make exit-owners MILESTONE=M2` reports 0.
- [ ] synthetic-generator.md §5.5 lists `numeric-fp` under "Implemented so far", and its scope is described.
      `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the failures (the owning tasks).
- The other rows of §5.5 (`coercion`, `error_prop`, `lookup`, ...) and M5+ functions (ROUNDUP of MOD, TRUNC, INT
  chains): file follow-ups for their own milestones.
- LAMBDA and defined-name bodies (T-3105, T-3093 and T-3111 cover them).
