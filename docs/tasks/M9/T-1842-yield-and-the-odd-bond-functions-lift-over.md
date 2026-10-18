---
id: T-1842
title: "YIELD and the ODD* bond functions lift over arrays, with their solver and coupon work weighted (SPEC-0011)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1822]
---

## Goal
YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD lift over arrays in every argument and refuse a multi-cell range there, with each element's solver and coupon work charged to the lifted call.

## Context
Split from T-1822, which entered PRICE, DURATION, MDURATION, ACCRINT and the COUP* family as `LiftArrays` weighted per coupon
schedule walk (functions/lift/kinds.rs `COUPON_WALK_WEIGHT`, financial.md "Coupon schedule work cap"). YIELD's solver
(`compute_yield_from_price`) runs up to 100 Newton steps of three PRICE evaluations and 200 bisection steps of two, each
PRICE a walk and a sum over up to about 32,400 coupons, so one element can cost seconds at the calendar's longest
schedule; ODDFYIELD and ODDLYIELD solve over ODDFPRICE/ODDLPRICE alike (`solve_yield`), and ODDFPRICE walks its schedule
(`coup_number`). Decide each kernel's weight from a measured worst case (or a tighter per-call bound first), then enter
them. Every argument is probed over an array (fin-argument-modes H190-H230, H344-H351) and over a range (H191-H223, H345;
fin-remaining-later-argument-ranges H105-H164, H173-H184, all `#VALUE!`). Their settlements read through
`get_settlement_and_maturity` / `get_odd_period_dates`, which T-1822 left node-based for YIELD and the ODD* functions.

## Acceptance criteria
- [x] The YIELD and ODD* lift cells of probes/fin-argument-modes pass (baseline lines removed), no new failures.
- [x] A test shows a lifted YIELD over many elements is refused by the work bound before its first element.
- [x] `make check` green; reviewers approve.

## Out of scope
- The solver stopping points (T-1374, T-1525).

## Outcome
- Probed first: probes/fin-yield-odd-lift (51 checked cells, excel-graph, 2026-10-18): lifted element values against the
  scalar call, element errors in position, boolean elements (`#VALUE!`, as a typed boolean), an error scalar beside a
  refused range (the argument order decides), a range beside an array, a row against a column, the first argument over
  a range on its own rows with the `@` control, and arguments of +-1E300 read from a cell.
- YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD are `LiftArrays` (functions/lift/kinds.rs,
  `// --- Financial ATP (T-1842) ---`). The kernels borrow their values and keep the scalar check order: YIELD reads
  through `bond_args` (now told which function it reads, `BondFunction`, for YIELD's price > 0 check), the ODD* through
  one `odd_args::<N>` with the date-order check as a closure; the node-based `get_settlement_and_maturity`,
  `get_financial_date` and `get_odd_period_dates` are deleted, and their static-analysis arms (10 lines).
- A tighter per-call bound first: `CouponSchedule` and `OddFirstSchedule` walk the schedule once per element, outside
  the solvers, so a Newton or bisection step is a sum over the coupons, not four or more walks and a sum. Same values
  (the operations of each term are unchanged; every timing input below gave bit-identical results before and after),
  and a scalar YIELD at the longest schedule went from 569 ms to 54 ms.
- Weights from release timings (throwaway timing test, calendar's longest schedule: settlement 1 to 9999-12-31,
  quarterly, basis 1): schedule walk 4.4 ms (YIELD) / 6.3 ms (ODDF long first period), price sum over 32,400 coupons
  0.55 ms, solver at most 397 prices, so about 0.23 s an element at the bound. YIELD and ODDFYIELD are `Weighted(2^24)`:
  8 elements, about 1.8 s at the bound; the slowest inputs found (price 1E300) take 54 ms an element, 16 elements
  0.94 s. ODDFPRICE is 8 walks (up to 6.3 ms an element): 256 elements, 1.68 s measured. ODDLPRICE/ODDLYIELD are 2 walks
  (up to 2.2 ms): 1,024 elements, 2.18 s (ODDLPRICE) and 1.71 s (ODDLYIELD) measured. DEVIATION: financial.md "Yield
  and odd-coupon work cap" (T-1869 narrows it to a schedule-length budget after T-1846).
- Tests: test/test_fn_yield_odd_lift.rs (lifts, element values and errors, ranges, far-out arguments without a panic,
  the work caps) and lift/tests `a_lifted_yield_over_many_elements_is_refused_before_its_first_element` (9 elements and
  2^20 elements: no kernel call).
- Scalar changes: none in values; the touched kernels' error messages are empty.
- Eval: probes 77 known failures fixed (fin-argument-modes 41, fin-remaining-later-argument-ranges 36), 0 regressions;
  probes and probes-numeric compared cell by cell against main: no other cell changed. The new case adds 7 known
  failures: T-1374's base-value ulps (A1, A3, A5, I5, I7, I9) and a negative basis (L5).
- Follow-ups: T-1869 (schedule-length budget for these kernels), T-1870 (a negative basis is `#NUM!` in Excel, basis 0
  here: `YIELD(..,-1E300)`).
