---
id: T-2672
title: "NPV to 15 digits on cancelling flows: Excel's discount and summation order (IRR.xlsx residues at IRR's root, 37 visibly-off cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2671]
---

## Goal
NPV is within half a unit of the 15th significant digit of Excel's value on every numeric cell below. Most of them
are sums that cancel, so this means using Excel's per-period discount and Excel's summation order, not a more
accurate sum. Needs an oracle sweep: the cached cells do not tell T-1528's candidates apart, so the fit waits for
T-2671's decomposition probe.

## Context
Gap triage 2026-10-25 (eval on origin/main). NPV has 113 visibly-off cells in the report, and 37 of them are
numeric with NPV as the outer call. T-1528 owns NPV's last digits, but it is blocked (time-boxed, decision 0056) and
deferred to phase 1b, and its acceptance criteria list none of the IRR.xlsx cells. T-2131 assumes those cells "follow
the IRR value". For the 12 cells in the first group, that is not so: the rate cell is bit-exact in ours and NPV
still differs. docs/excel-semantics/financial.md "NPV's residue at IRR's root is not the double sum" has the
quirk.

Visibly-off numeric NPV cells (Excel / ours):
- IRR.xlsx, rate bit-exact in ours (12; the residue isolates NPV's own arithmetic):
  - General!M2 `NPV(L2,A2:F2)` over {-20,5,5,5,5,5}: 6.4212018552224666E-12 / 6.418865439172805E-12
    (exact at that rate 6.421923509333131E-12);
  - M3 over {-1000,230,200,125,200,10}: -2.8358465972398184E-11 / -2.8180124900245573E-11 (exact
    -2.815865414018216E-11);
  - M8 over {-70,12,15}: -1.579429159894141E-10 / -1.5792522845003987E-10 (exact -1.5794321715940695E-10);
  - M5, M9, M10, M11, M13;
  - Three Parameters!E2 `NPV(D2,A2:C2)` over {1,-2,1}: 1.187938506697367E-14 / 1.2101430968414206E-14;
  - FullRow!Q2 (= E2's flows), FullColumn!B21 and B22 (= M2 and M3 over whole rows).
- IRR.xlsx, rate also off in ours (12; re-measure after T-2131): General!M4, M6, M12, M16, M17, M18, M19, Three
  Parameters!E3, E4, E5, FullRow!R2, FullColumn!B23.
- probes/fin-argument-modes H186, H367, H368, H372 `NPV(0.1,-10000,2750,4250,3250,2750)` in four argument forms:
  302.233453999042 / 302.23345399904133 (exact at the double rate 302.23345399904366).
- probes-numeric/m9-fin-nper-npv A148, A154, A156, A159, A164, A167, e.g. A148
  `NPV(0.1,-1E6,250000,250000,250000,250000,250000)`: -47548.46149808013 / -47548.4614980802.
- gen-m9 (T-2257 sweep cells T-1528 lists as visibly off): 1cd3096e3e87c965 Calc!A345 (0.7710730536356959 /
  0.771073053635682), 3aa168259335be54 Calc!A214 (2.671477233044049 / 2.671477233044019), abb7d565af89b849
  Calc!A387 (0.00020006599755898017 / 0.00020006599755891932).

Excel is not the correctly rounded sum: at M2, M8 and E2 it is nearer the exact value than ours, at M3 further
from it. A triage scan of plain doubles (sum of v/(1+r)^i, reverse order, Horner from the last value, running factor
by * or /, v*(1+r)^-i, v/exp(i*log1p r), fsum) matched none of the 12 rate-exact residues, except Horner on M8.
T-1528's Blocked section lists the formulations its three attempts rejected on m9-fin-nper-npv (best 27/88).
Engine: `compute_npv` in crates/rustsheet-engine/src/functions/financial/financial_util.rs. The lab is
tools/src/rustsheet_tools/fitlab/kernels/annuity.py (`annuity-npv`).

The NPV cells that are #DIV/0! in Excel (rate -1 or 1E+300) are T-2493's and T-2618's. Cells where an inner
function differs (FORECAST.ETS, VAR.S) belong to that function's task.

T-2671 (2026-10-26) found the formulation: NPV = (v1/d0 + v2/d1 + ... + vn/d(n-1)) / (1+r) with d0 = 1 and
d(i) = d(i-1)*(1+r), all in doubles, summed left to right, with one division by (1+r) at the end. It is bit-exact on
all 382 probes/npv-term-order cells and the 88 m9-fin-nper-npv cells (lab `annuity-npv`, 470/470), and on M2, M3, M8,
Three Parameters!E2, H367 and A148 above (financial.md "NPV's residue at IRR's root is not the double sum"). The
fit criterion below is met by that entry; what remains is the engine. Keep T-2493's `#DIV/0!` rules at huge rates
and flow counts (financial.md) when the power source changes from Excel's `^` to the running product: check
probes/npv-power-edges and npv-power-edges-2 stay green. The 244 probes/npv-term-order known failures are this task's.

## Acceptance criteria
- [x] An NPV formulation fitted on T-2671's probe, held out on the cells above, is recorded in financial.md under
      "NPV's residue at IRR's root is not the double sum" with its hit counts (bit-exact, within 4 ulp, within 15
      digits) on: the probe, the 12 rate-exact IRR.xlsx cells, m9-fin-nper-npv's 88 NPV cells, and the gen-m9 NPV
      cells. The rejected candidates are recorded too.
- [x] Visible parity: the 12 rate-exact IRR.xlsx cells, the four fin-argument-modes cells, the six m9-fin-nper-npv
      cells and the three gen-m9 cells listed above are within half a unit of the 15th significant digit of Excel's
      value. Their known-failure lines disappear from eval/baseline/ironcalc-calc.txt, probes.txt, probes-numeric.txt
      and gen-m9.txt (`make eval-baseline`) wherever they become bit-exact, and no baseline gains a line.
- [x] The 12 IRR.xlsx cells whose rate is also off are re-measured, and the Outcome says where they land. They stay
      T-2131's when the rate is the cause.
- [x] NPV's callers that share `compute_npv` (IRR's and MIRR's NPV, if they do) are checked on IRR.xlsx, MIRR.xlsx
      and probes/solver-iteration-traces, and none of their cells becomes newly failing or visibly off.
- [x] If two attempts find no formulation that raises the within-15-digits count, the task is set `blocked` with a
      "Blocked because: time-boxed (decision 0056)" section listing the candidates and counts, and T-1528's Blocked
      section gets the new evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- NPV's error order and its #DIV/0! cases (T-2493, T-2618, T-2103).
- IRR's iterate (T-2131); PMT, IPMT, PPMT (T-1528, T-2243).
- Bit-exactness beyond the visible-parity target: what is left goes to a follow-up with `defer: phase-1b`, or back
  to T-1528.

## Visible-parity stage
Decision 0083: the cells listed under Acceptance criteria are visibly off (a number more than half a unit of the
15th significant digit from Excel's). The stage target is that each one is within that half unit. Bit-level
criteria still unmet then move to a follow-up task filed without the key and with `defer: phase-1b`.

## Outcome
`excel_npv` now computes Excel's rule, which T-2671 fitted: (v1/d0 + ... + vn/d(n-1))/(1+r), with d a running product
in doubles and one division at the end. T-2493's `#DIV/0!` edges are unchanged (npv-power-edges, npv-power-edges-2 and
the unit test stay green). Eval: 344 known failures fixed, no regressions (probes 2984 -> 2703, including all 244
npv-term-order lines; ironcalc-calc 617 -> 590; gen-m9 3666 -> 3630). Visible parity 3165 (lock 3254).
Hold-out counts: the 12 rate-exact IRR.xlsx cells 12/12 bit-exact; fin-argument-modes H186, H367, H368, H372 4/4;
m9-fin-nper-npv 88/88; gen-m9 NPV cells 252/255. The listed gen-m9 cells: 1cd3096e3e87c965 A345 and 3aa168259335be54
A214 are now bit-exact. abb7d565af89b849 A387 is IRR's: Excel's NPV needs a rate 1673 ulp below our IRR (T-2131).
The rule misses only 9d9ceba257f8001b A85 (4 ulp, not visibly off). That goes to follow-up T-2953 (`defer: phase-1b`).
The 12 rate-off IRR.xlsx cells stay failing, but the rule at Excel's cached rate gives each of them to the bit, so they
are T-2131's. IRR and MIRR do not call `excel_npv` (IRR has its own `irr_npv`), and IRR.xlsx, MIRR.xlsx and
solver-iteration-traces gained no failure. financial.md records all of this in both NPV entries.
