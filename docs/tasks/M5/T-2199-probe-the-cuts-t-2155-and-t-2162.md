---
id: T-2199
title: "Probe the cuts T-2155 and T-2162 need: REPT, CODE and UNICODE near 255 characters, COMBIN and COMBINA near 2^31"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
excel-graph values that pin two cuts the gen-m5 triage could only bracket: the text length at which REPT, CODE and
UNICODE become `#VALUE!` (T-2155), and the `number` at which COMBIN and COMBINA become `#NUM!` (T-2162). The two
functions tasks can then implement the cuts without probing first.

## Context
- The M5 exit needs gen-m5 at 99.9% (decision 0085 item 5): at most 22 known failures; 190 are left today. T-2155 owns 6
  and T-2162 owns 5, and both begin with "Probe first (`excel-probe`)". The functions lanes are full, so the probe moves here.
- Text cut: docs/excel-semantics/text.md "REPT, CODE and UNICODE refuse a text argument over 255 characters (T-2155)":
  600 characters fail and 255 work. Still open: the exact cut, whether it counts UTF-16 units or code points, and whether
  other one-text-argument functions share it. LENB, PROPER, UPPER, EXACT, FIND and SEARCH do not.
- Number cut: docs/excel-semantics/numbers.md "COMBINA is COMBIN(n+k-1, k)", paragraph "Bound (T-2162)": the cut lies
  between 1.23E+8 and 6.2E+9 (2^31 is a guess). Still open: whether it applies to `n` or to `n+k-1`.
- Earlier COMBIN evidence: probes/combin-algorithm-sweep. Tag the new case `M5` (decision 0051).

## Acceptance criteria
- [ ] New case `eval/suites/probes/cases/t2199-text-length-and-combin-cuts.case.json` (`rustsheet-case/v1`, one `M5` tag), with:
      - `REPT(REPT("x",n),2)`, `CODE(REPT("x",n))` and `UNICODE(REPT("x",n))` for n = 254, 255, 256, 257, 300 and
        32767, plus `LEN` of each REPT cell;
      - the same three over a text of 254 "x" followed by one astral character (`UNICHAR(128512)`, 2 UTF-16 units), so
        256 units but 255 code points, and over 255 "x" followed by it;
      - CHAR-free controls with a 256-character text for CLEAN, TRIM, LOWER, T, VALUE (a 256-digit numeral) and
        LEFT(..,1), each beside `LEN` of the result;
      - `COMBIN(n,k)` and `COMBINA(n,k)` for n = 2^31-2, 2^31-1, 2^31, 2^31+1, 2^32 and k = 0, 1, 2;
      - `COMBINA(n,k)` with n < 2^31 <= n+k-1 (n = 2^31-1, k = 2; n = 2^31-10, k = 20), and `COMBIN(n,k)` with
        k > 2^31 (n = k = 2^31+1).
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`), with no
      pending cell.
- [ ] text.md (T-2155 entry) and numbers.md (COMBINA "Bound" paragraph) state each cut as a rule, cite the case and drop
      their "Unprobed" lines. Any other function found to share the text cut gets its own sentence there.
- [ ] The case's failing cells are new-coverage F lines in eval/baseline/probes.txt (`make eval-baseline`), with no
      regression in any suite. T-2155's and T-2162's Contexts name the case as their evidence and as cells they must pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-2155, T-2162).
- REPT's 32,767-unit result limit, which is already implemented.
