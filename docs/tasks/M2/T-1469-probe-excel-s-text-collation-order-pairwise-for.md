---
id: T-1469
title: "Probe Excel's text collation order pairwise for comparison operators (evidence for T-0220)"
milestone: M2
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for Excel's text ordering under `<`/`>`/`=`, enough to fit the one collation key that T-0220
implements: ASCII punctuation, digits, letters in both cases, accented Latin letters, the hyphen and apostrophe,
and how text of different lengths compares.

## Context
T-0220 (engine, M2) must replace code-point text order with Excel's collation. It owns 5 gen-m2 known failures:
2cfe279fe77a000d A318 and A328, 96a0952412cefa6c A284, ef5c55a81783b7d5 A151, d04d501154afc771 A97. It also owns
probes/text-order-nbsp-space Calc!D17. Today the evidence is two gen-m2 cells and the CHAR(160) probe
(docs/excel-semantics/coercion.md "Text comparison uses Excel's collation, not code-point order"). They show that
"ü" sorts with "u", punctuation sorts before digits, and a hyphen is skipped (`"-1" > "0"` is TRUE). That is too
little to fit a key. Splitting the probe out lets T-0220 start from evidence.
Use the excel-probe skill (excel-graph). Suggested shape: one Data column with about 60 single characters and
short strings (`!` to `~` printable ASCII, `0`-`9`, `a`/`A`, `z`/`Z`, `à á â ä å æ ç é ñ ö ø ß ü ÿ`, `-`, `'`,
`"a-b"` vs `"ab"` vs `"a b"`, `"co-op"` vs `"coop"`, `"a"` vs `"aa"`). Then a Calc sheet that ranks each against
all the others with `=SUMPRODUCT(--(Data!$A$1:$A$60<A1))`, plus pairwise `<` and `=` cells for the
ties (case, hyphen-only differences). Rank cells keep the cell count small: one per string, plus the tie pairs.

## Acceptance criteria
- [ ] A new `probes` case (tagged `M2`, `collation`) is oracle-filled on excel-graph and committed with its `*.oracle.json`.
- [ ] docs/excel-semantics/coercion.md's collation entry gets the observed order (rank table or rule) and the case id as evidence.
- [ ] Engine mismatches stay known failures (baseline updated with `make eval-baseline`). They are T-0220's to fix: no new task unless a cell is outside T-0220's scope.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the collation (T-0220).
- Locales other than en-US, and SORT/criteria call sites (they share `compare_values`; T-0220 checks them).

## Outcome
Added probes/text-collation-order (146 strings, rank and pairwise cells), oracle-filled on excel-graph; documented the observed order in docs/excel-semantics/coercion.md. Engine mismatches are recorded as known failures for T-0220.
