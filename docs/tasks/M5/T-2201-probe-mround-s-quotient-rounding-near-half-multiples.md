---
id: T-2201
title: "Probe MROUND's quotient rounding near half-multiples and the product's last digit (T-2167)"
milestone: M5
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
State, from excel-graph evidence, how Excel's MROUND rounds `number/multiple` and forms `n*multiple`. The rule must
predict every probed cell, in a form T-2167 can implement directly. Today one cell (`MROUND(0.3,0.2)` = 0.4) fits "a
15-significant-digit cut of the quotient, then half away from zero", and nothing tells that rule from its rivals.

## Context
- The M5 exit needs gen-m5 at 99.9% (decision 0085 item 5); T-2167 (functions-math, opus) owns gen-m5/043ee562f32eb901
  Calc!A199 and begins with "Probe first". The functions lanes are full, so the probe and the rule move here, and T-2167
  only implements.
- Observation: docs/excel-semantics/numbers.md "MROUND rounds the quotient as a 15-digit decimal (T-2167)". In doubles,
  0.3/0.2 is 1.4999999999999998, which rounds to 1 (ours: 0.2). Excel gives 0.4.
- Rival rules that one cell does not separate: (1) a 15-significant-digit decimal cut of the quotient (ROUND's stage 1,
  numbers.md ROUND entries); (2) a quotient tolerance such as `q + k*eps`; (3) an x87 80-bit quotient; (4) a decimal
  `number - multiple*floor(...)` remainder test. The product may be formed in binary: ours gives `MROUND(0.5,0.2)` =
  0.6000000000000001 (unprobed in Excel). The ROUND, ROUNDUP and MOD entries in numbers.md show the 15-digit techniques
  Excel uses elsewhere; docs/research/excel-kernel-playbook.md has the probe designs for such questions.
- This needs opus: it is numeric rule fitting. The cells must be chosen to separate rules (1)-(4) at the 15th and 16th
  digit, and the chosen rule checked against all of them.
- Tag the case `M5` (decision 0051). Code that T-2167 will change: `fn_mround` in
  crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] New case `eval/suites/probes/cases/t2201-mround-quotient-rounding.case.json` (`rustsheet-case/v1`, one `M5` tag), with
      stored inputs and at least 150 cells, including:
      - the near-half quotients from T-2167 (0.3/0.2, 0.7/0.2, 2.5/1, 1.15/0.1, 1.05/0.1) and their negatives;
      - at least 60 seeded pairs whose binary quotient lies within 4 ulp of k+0.5, on both sides, with the seed and script
        recorded in the case description;
      - quotients that agree to 15 digits with k+0.5 but differ at the 16th, which separate rule (1) from rule (2);
      - products whose last bit shows binary vs decimal forming (`MROUND(0.5,0.2)`, `MROUND(0.7,0.1)`, `MROUND(1E+15+0.3,0.2)`),
        each beside `(x)=<decimal>` companion cells;
      - the multiples 1E-15 and 1E+15, as controls.
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`), with no
      pending cell.
- [ ] numbers.md "MROUND rounds the quotient as a 15-digit decimal" becomes a VERIFIED rule. It gives the quotient rounding
      and the product forming as an algorithm T-2167 can code, cites the case, and lists every probed cell the rule does
      not predict (target: none).
- [ ] The case's failing cells are new-coverage F lines in eval/baseline/probes.txt (`make eval-baseline`), with no regression
      in any suite. T-2167's Context names the case and the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `fn_mround` (T-2167).
- MROUND argument modes and lifting (arrays.md "CEILING/FLOOR family and MROUND lift over arrays", done).
- INT's 15-digit cut (T-2171).
