---
id: T-3125
title: "Probe the M9 securities argument kinds no Excel cell covers yet: empty text, an error or numeric text in 26 coupon, price, yield and T-bill functions"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M9 securities functions below (coupon dates, accrued interest, discount securities, PRICE/YIELD/DURATION, the odd-coupon functions and the T-bills) has at least one Excel-backed cell for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-29 (`make eval` on 6793e919 with calipers, corpus-bench and poi fetched; report.json
`argument_coverage`, decision 0031 evidence gaps; decision 0094 asks for each argument mode to be covered). Every
visibly-off cell of that run has an open owner (`scripts/exit_owners.py --all-off`), and the evidence floor is met, so
the M9 argument kinds are the largest evidence gap left: 245 M9 functions have a value parameter with a kind no Excel
cell passes (M2 was closed by T-2861; M5 to M7 are T-2862, T-2863, T-2864, T-2865 and T-2871 to T-2878; M9 had none).
The kinds are those of `crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative,
fraction, numeric_text, text, empty_text, boolean, error, blank, range. A kind is seen only when an Excel-evidence cell
passes it as a literal or a single-cell reference: nested calls such as `NA()` are not classified (docs/progress.md
"Argument coverage"), so use literal errors (`#N/A`, `#DIV/0!`) and cells holding them. T-2866 taught the generator to
draw `""` and error literals, but at 3% per parameter the existing gen-m9 cases predate it and do not reach these pairs.
`array`-mode parameters are left out: their coverage is not classified by kind.
These are the Analysis ToolPak securities functions, whose check order is already recorded (financial.md "Analysis ToolPak securities functions check frequency and dates first", "PRICE, YIELD, DURATION and MDURATION check order", "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order", "Discount securities check order"). With one bad argument and valid others, the order does not decide the result; keep it that way, so every cell tests one coercion. Use a valid schedule (e.g. settlement 2024-01-15, maturity 2030-06-30, frequency 2, basis 0; the odd-coupon functions need dates on the maturity's schedule, financial.md "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule").
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- ACCRINT.issue: empty text, an error; ACCRINT.first_interest: empty text, a boolean, an error; ACCRINT.settlement: numeric text, empty text, an error; ACCRINT.rate: empty text, an error; ACCRINT.par: numeric text, empty text, an error; ACCRINT.frequency: numeric text, empty text, a boolean, an error; ACCRINT.basis: text, empty text, an error, a blank cell; ACCRINT.calc_method: an error.
- ACCRINTM.issue: empty text, an error; ACCRINTM.settlement: empty text, an error; ACCRINTM.rate: empty text, an error; ACCRINTM.par: empty text, an error; ACCRINTM.basis: empty text, an error, a blank cell.
- COUPDAYBS.settlement: numeric text, empty text, a boolean, an error; COUPDAYBS.maturity: empty text, an error; COUPDAYBS.frequency: empty text, an error; COUPDAYBS.basis: empty text, an error.
- COUPDAYS.settlement: numeric text, empty text, an error; COUPDAYS.maturity: zero, empty text, an error; COUPDAYS.frequency: text, empty text, an error, a blank cell; COUPDAYS.basis: empty text, an error, a blank cell.
- COUPDAYSNC.settlement: empty text, an error; COUPDAYSNC.maturity: numeric text, empty text, an error; COUPDAYSNC.frequency: empty text, an error; COUPDAYSNC.basis: empty text, an error.
- COUPNCD.settlement: numeric text, empty text, an error; COUPNCD.maturity: numeric text, empty text, an error; COUPNCD.frequency: empty text, an error; COUPNCD.basis: text, empty text, an error.
- COUPNUM.settlement: numeric text, empty text, an error; COUPNUM.maturity: empty text, an error, a blank cell; COUPNUM.frequency: empty text, an error, a blank cell; COUPNUM.basis: empty text, an error.
- COUPPCD.settlement: numeric text, empty text, a boolean, an error, a blank cell; COUPPCD.maturity: numeric text, empty text, an error; COUPPCD.frequency: empty text, a boolean, an error; COUPPCD.basis: empty text, an error.
- DISC.settlement: empty text, an error; DISC.maturity: empty text, an error; DISC.pr: empty text, an error; DISC.redemption: empty text, an error; DISC.basis: empty text, an error.
- DURATION.settlement: empty text, a boolean, an error; DURATION.maturity: numeric text, empty text, an error; DURATION.coupon: empty text, an error; DURATION.yld: empty text, an error; DURATION.frequency: empty text, an error, a blank cell; DURATION.basis: empty text, an error.
- INTRATE.settlement: empty text, an error; INTRATE.maturity: empty text, an error; INTRATE.investment: empty text, an error; INTRATE.redemption: empty text, an error, a blank cell; INTRATE.basis: empty text, an error.
- MDURATION.settlement: numeric text, empty text, a boolean, an error, a blank cell; MDURATION.maturity: empty text, an error; MDURATION.coupon: empty text, an error; MDURATION.yld: empty text, an error; MDURATION.frequency: empty text, an error; MDURATION.basis: empty text, an error.
- ODDFPRICE.settlement: empty text, an error, a blank cell; ODDFPRICE.maturity: empty text, an error; ODDFPRICE.issue: numeric text, empty text, an error; ODDFPRICE.first_coupon: numeric text, empty text, an error; ODDFPRICE.rate: empty text; ODDFPRICE.yld: empty text, an error; ODDFPRICE.redemption: numeric text, empty text, an error; ODDFPRICE.frequency: empty text, an error; ODDFPRICE.basis: empty text, an error, a blank cell.
- ODDFYIELD.settlement: numeric text, empty text, an error; ODDFYIELD.maturity: numeric text, empty text, an error; ODDFYIELD.issue: empty text, an error; ODDFYIELD.first_coupon: numeric text, empty text, an error; ODDFYIELD.rate: empty text, an error; ODDFYIELD.pr: numeric text, empty text; ODDFYIELD.redemption: numeric text, empty text, an error; ODDFYIELD.frequency: empty text, an error; ODDFYIELD.basis: empty text, an error, a blank cell.
- ODDLPRICE.settlement: numeric text, empty text, an error; ODDLPRICE.maturity: empty text, an error; ODDLPRICE.last_interest: numeric text, empty text, an error; ODDLPRICE.rate: empty text, an error; ODDLPRICE.yld: empty text, an error; ODDLPRICE.redemption: a negative, numeric text, empty text, an error; ODDLPRICE.frequency: empty text, an error; ODDLPRICE.basis: empty text, an error, a blank cell.
- ODDLYIELD.settlement: numeric text, empty text, an error; ODDLYIELD.maturity: numeric text, empty text, an error; ODDLYIELD.last_interest: numeric text, empty text, an error; ODDLYIELD.rate: numeric text, empty text, an error; ODDLYIELD.pr: empty text; ODDLYIELD.redemption: numeric text, empty text, an error; ODDLYIELD.frequency: empty text, an error; ODDLYIELD.basis: empty text, an error.
- PRICE.settlement: empty text, a boolean, an error; PRICE.maturity: empty text, a boolean, an error, a blank cell; PRICE.rate: empty text, an error; PRICE.yld: empty text, an error; PRICE.redemption: numeric text, empty text, an error; PRICE.frequency: empty text, an error; PRICE.basis: empty text, an error.
- PRICEDISC.settlement: empty text, an error; PRICEDISC.maturity: empty text, an error, a blank cell; PRICEDISC.discount: empty text, an error; PRICEDISC.redemption: empty text, an error; PRICEDISC.basis: empty text, an error, a blank cell.
- PRICEMAT.settlement: empty text, an error; PRICEMAT.maturity: empty text, an error; PRICEMAT.issue: empty text, an error; PRICEMAT.rate: empty text, an error; PRICEMAT.yld: empty text, an error; PRICEMAT.basis: empty text, an error, a blank cell.
- RECEIVED.settlement: empty text, an error; RECEIVED.maturity: empty text, an error, a blank cell; RECEIVED.investment: empty text, an error; RECEIVED.discount: empty text, an error; RECEIVED.basis: empty text, an error.
- TBILLEQ.settlement: empty text; TBILLEQ.maturity: empty text, an error; TBILLEQ.discount: empty text.
- TBILLPRICE.settlement: empty text; TBILLPRICE.maturity: empty text, an error; TBILLPRICE.discount: empty text.
- TBILLYIELD.settlement: empty text; TBILLYIELD.maturity: empty text, an error; TBILLYIELD.pr: numeric text, empty text, a blank cell.
- YIELD.settlement: numeric text, empty text, an error; YIELD.maturity: numeric text, empty text, an error; YIELD.rate: empty text, an error; YIELD.pr: numeric text, empty text, an error; YIELD.redemption: empty text, an error; YIELD.frequency: empty text, a boolean, an error; YIELD.basis: empty text, an error.
- YIELDDISC.settlement: empty text, an error; YIELDDISC.maturity: empty text, an error; YIELDDISC.pr: empty text, an error; YIELDDISC.redemption: empty text, an error; YIELDDISC.basis: empty text, an error.
- YIELDMAT.settlement: empty text, an error; YIELDMAT.maturity: empty text, an error; YIELDMAT.issue: empty text, an error; YIELDMAT.rate: empty text, an error; YIELDMAT.pr: empty text, an error; YIELDMAT.basis: empty text, an error.
Write the probe case probes/m9-fin-securities-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-fin-securities-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/financial.md "Financial function argument modes", citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The annuity, depreciation and cash-flow functions (T-3126).
- Numeric last digits of the probed values: only the coercion is in scope; a value within 15 digits but not bit-exact goes to the function's existing last-digit task.
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
