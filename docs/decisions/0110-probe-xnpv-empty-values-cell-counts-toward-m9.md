# 0110 — Probe xnpv-empty-values-cell counts toward M9, not M2

- **Status:** accepted (2026-10-25, steward run; no task id, steward PRs are exempt). Amends the
  list of `eval/milestones/probe-milestones.tsv` lines (decisions 0051, 0058); the rule is unchanged.
- **Context:**
  - The 2026-10-25 `make eval` (origin/main 2c523462) reports one M0–M2 probe known failure:
    `probes/xnpv-empty-values-cell` Sheet1!C8, `XNPV(1,A2:A3,B11:B12)` (ours `#NUM!`, Excel
    `#VALUE!`), owned by T-2603 (M9, functions-financial). It is the only thing keeping M2's exit
    criterion "M0–M2 probes at 100% bit-exact" unmet.
  - T-2599 (#1690) added the case tagged `M2`. Its subject is XNPV: every formula in it is
    `XNPV(..)`, with `ABS` only building the rate argument and `ISERROR` only observing the result.
    XNPV's catalog milestone is M9 (`eval/catalog/scope.json`), and the other XNPV probes
    (`xnpv-argument-count`, `xirr-xnpv-whole-column-nogaps`) already count toward M9.
  - Decision 0058's Consequences foresaw this: "A probe that 0057 lets through with a wrong later
    tag (`M2` on an M9 subject, as `tbilleq-zero-price`) still needs a reviewer to catch it and a
    line here." No case file is edited (oracle input hashes; decision 0051).
- **Decision:** add `probes/xnpv-empty-values-cell.case.json	M9	XNPV` to
  `eval/milestones/probe-milestones.tsv`. 0051's exception does not apply: the subject includes no
  M2 function and no M1 subject.
- **Consequences:** the C8 failure moves from the M0–M2 row to the M9 row of "Probe milestones
  (decision 0051)"; it stays a known failure in `eval/baseline/probes.txt` and keeps its owner
  T-2603. No baseline, gate or scored cell changes. With it, M0–M2 probes have 0 known failures, so
  M2's exit is met and M5 becomes the current milestone again (docs/milestones.md phase order).
