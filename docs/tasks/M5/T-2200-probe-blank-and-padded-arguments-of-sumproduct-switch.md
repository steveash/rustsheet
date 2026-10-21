---
id: T-2200
title: "Probe blank and padded arguments of SUMPRODUCT, SWITCH and ARABIC (T-2150, T-2166, T-2153)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
excel-graph values for the blank-cell and whitespace questions that T-2150 (SUMPRODUCT), T-2166 (SWITCH) and T-2153
(ARABIC) left open. Each task can then generalise its rule from evidence rather than from the few gen-m5 cells.

## Context
- The M5 exit needs gen-m5 at 99.9% (decision 0085 item 5): at most 22 known failures; 190 are left today. T-2150 owns 9,
  T-2153 owns 12 and T-2166 owns 1. Each says "Probe first" or "Probe before generalising". The functions lanes are full,
  so the probe moves here.
- Rules and open questions:
  - docs/excel-semantics/coercion.md "SUMPRODUCT: a blank one-cell argument beside other arguments is `#VALUE!` (T-2150)".
    Unprobed: a blank one-cell argument beside a multi-cell range, and a one-cell text or boolean argument.
  - docs/excel-semantics/logic.md "SWITCH: a blank cell case value does not match FALSE (T-2166)". Unprobed: a blank
    case cell against 0, "" and TRUE, and a blank expression cell. The omitted-argument form `SWITCH(0,,"a","b")` = "a"
    is already evidenced. T-2189 probes omitted *value* arguments of IFS and SWITCH; this task does not repeat it.
  - docs/excel-semantics/numbers.md "ARABIC of a blank cell, and padded numerals (T-2153)". Unprobed: `ARABIC(0)`,
    inner spaces, and other whitespace (CHAR(10), CHAR(13), CHAR(160), a tab inside or around a numeral).
- Tag the case `M5` (decision 0051).

## Acceptance criteria
- [ ] New case `eval/suites/probes/cases/t2200-blank-and-padded-m5-args.case.json` (`rustsheet-case/v1`, one `M5` tag),
      with one stored blank cell, one-cell text, boolean and number cells, and a 2x2 number range. It has:
      - SUMPRODUCT: the blank one-cell reference beside a 2x2 range, beside a 1x1 range of a number, alone, first and last
        in a three-argument list; a one-cell text and a one-cell boolean reference beside a number and beside a 2x2 range;
        and the blank one-cell reference beside an error argument and a shape mismatch, in both orders;
      - SWITCH: a blank case cell against expressions FALSE, 0, "", TRUE and a blank expression cell; a blank expression
        cell against cases FALSE, 0, "" and a blank case cell; each with a distinct result per case and no default;
      - ARABIC: `ARABIC(0)`, `ARABIC(FALSE)`, `ARABIC("X X")`, `ARABIC("XI ")` with each of CHAR(9), CHAR(10), CHAR(13)
        and CHAR(160) as padding on either side, and `ARABIC(" ")`.
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`), with no
      pending cell.
- [ ] coercion.md, logic.md and numbers.md entries above drop their "Unprobed" lines, state the probed results as rules,
      and cite the case.
- [ ] The case's failing cells are new-coverage F lines in eval/baseline/probes.txt (`make eval-baseline`), with no
      regression in any suite. T-2150's, T-2153's and T-2166's Contexts name the case as their evidence and as cells they
      must pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-2150, T-2153, T-2166).
- SUMX2MY2, SUMX2PY2 and SUMXMY2 (T-0961, done).
- IFS and SWITCH with an omitted value argument (T-2189).

## Outcome
Probed with excel-graph (70 cells). Findings recorded in coercion.md, logic.md and numbers.md; notable: a lone blank
one-cell SUMPRODUCT argument is `#VALUE!`, one-cell text/boolean beside a number is 0, `ARABIC(0)` is `#VALUE!`,
CHAR(160) is not trimmed by ARABIC. New failing cells baselined in eval/baseline/probes.txt as new coverage.
The baseline also gained the four `weeknum-stale-order-column-3..6` entries (and their oracle sidecars, filled in the same
`fill` run): those cases were on main without a baseline entry, so they ride along here.
