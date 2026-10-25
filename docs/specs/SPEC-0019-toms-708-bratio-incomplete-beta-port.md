# SPEC-0019 — Excel's incomplete beta as a TOMS 708 `bratio` port, one branch at a time

- **Status:** draft
- **Milestone / lane:** M9 / functions-stat
- **Tasks:** T-2504 (this spec). Port: T-2505, T-2506, T-2508, T-2509, T-2510, T-2511. Densities the port uses as
  prefactors: T-2503, T-2507, and T-2585, which shares their pieces. Done before it: T-2502 (closed forms), T-2645
  (log1p, EXP of an extended argument). Research that feeds it: T-2512, T-2586. Touched: T-2633.

## Problem
Five function families compute the regularized incomplete beta I_x(a, b): BETA.DIST, F (F.DIST, F.DIST.RT, FDIST,
F.TEST), T (T.DIST, T.DIST.RT, T.DIST.2T, TDIST, T.TEST), BINOM.DIST and NEGBINOM.DIST, plus the inverses built on
them. Excel uses one kernel for all of them, a port of TOMS 708 `bratio` (Didonato and Morris, 1992 Fortran) with its
own elementary functions and x87 stored operations. docs/excel-semantics/statistics.md "Incomplete beta is TOMS 708
`bratio`" (T-2464) has the rule.

The engine instead has five separate approximations:
- `incomplete_beta.rs`: Cephes continued fractions, reflected at the mean, complement 1 - v, with Excel's closed forms
  in front (T-2502);
- `student_t.rs`: its own composite (a double-double series, a Cephes deep tail, `incbcf`/`incbd` from z);
- `binom.rs` / `discrete.rs`: a double-double pmf walk;
- `inverse_beta.rs`: statrs' AS 64/109 Newton, with crossing refinements;
- `fisher.rs`: the unoriented F argument forms d1X/(d1X + d2).

r2-impact (docs/research/t2464/r2-impact/RESULTS.md) counts 5,533 visibly-off cells in this family before the
research, 1,236 of them gated. The port pieces fix about 1,800 of those cells. On the w2f probes they fix 22,413 of
23,657.

Replacing two kernels across five families is an architectural change (CLAUDE.md directive 4), and the order matters:
- **A plain TOMS port is net negative outside BETA.DIST.** r2-impact finding 2: on the 10,602 F, T, BINOM and
  NEGBINOM cells it is -103 visible (R_LIKE) or -51 (TOMS with Excel's EXP, LN and GAMMALN). The Excel pieces are
  what wins: BEST is +543.
- **Even the full model loses cells.** BEST+log1p loses 893 bit-exact cells, 6 of them gated, and breaks 213.
- So the port has to move one piece at a time, each piece with its Excel arithmetic. Each step must also keep every
  cell it does not move bit-identical.

## Goals / non-goals
Goals:
- One kernel, K(a, b, x, y) → (w, w1, branch), behind every incomplete-beta caller, with bratio's dispatch and Excel's
  arithmetic in each branch. statistics.md's T-2464 and T-2513 entries are the rules.
- A migration in small steps (S/M tasks). Each step moves one branch for the callers where that branch wins, and leaves
  every other (caller, branch) pair on that caller's current code bit for bit.
- Every loop bounded, no panics, no allocation in the kernel (directive 5).
- One definition per concept: the x87 helpers, the Excel elementary functions, the densities and the Loader pieces are
  shared, not copied (directive 3).

Non-goals:
- The pieces Excel's arithmetic leaves open: bgrat for b ≠ 1/2, algdiv for b0 >= 8, bfrac's fraction noise, the
  b < 1 bpser ceiling (T-2512); Loader's constant residual and bd0's series (T-2586). The port ships the best measured
  form and the visible-parity floor. The research tasks amend the owning port task's criteria when they solve a piece.
- The F density (T-2586, T-2095), the BINOM/NEGBINOM interior pmf (T-2146) and F.TEST's error precedence (T-2093).
- BINOM.DIST.RANGE, BINOM.INV and CRITBINOM. They keep the pmf walk. No evidence ties them to the kernel yet: r2-impact
  hypothesis 3 is unmeasured.
- Public API: none. Everything here is `pub(crate)` inside `functions/statistical`.

## Excel behavior to match
All of this is in docs/excel-semantics/statistics.md (excel-graph oracle). The entries named below are the
evidence for each piece.
- **Dispatch, the computed tail and the complement** ("Incomplete beta is TOMS 708 `bratio`", VERIFIED):
  - min(a, b) <= 1 swaps to (b, a, y, x) when x > 0.5;
  - min(a, b) > 1 swaps when λ = a - (a + b)·x < 0 (statistics.md). TOMS, and the research model, form λ as
    (a + b)·y - b when a > b. The two differ only by rounding at the mean, and that case is unprobed; T-2508 owns it.
    bfrac recomputes its own λ = fl(a - fl((a + b)·x)) (statistics.md "L140 and bfrac"; a λ from y is rejected);
  - then the branches below, with x0 >= 0.3 for w1 (TOMS 1992, not R's 0.29: 284 against 87 cells of
    t2464-r2-bratio-x29);
  - the other tail is (0.5 - w) + 0.5;
  - one evaluation gives both tails (1,578 of 1,578 F pairs).
- **Closed forms** ("Incomplete beta closed forms act on the oriented arguments", VERIFIED, 1,286 of 1,286; T-2502).
- **bpser** ("Incomplete beta bpser", PARTIAL):
  - the forward x87 series from fl(1/a);
  - I = fl(EXP(z)·s), with z = fl64(fl(a·LN x) - L) an extended argument to EXP;
  - L in the GL form for b0 < 8, or GL + algdiv for b0 >= 8;
  - no fpser and no apser.
- **bup and bfrac prefactors** ("Incomplete beta L140 and bfrac", VERIFIED for L140 to 15 digits): brcmp1 = brcomp =
  Excel's own density times x·y.
- **bgrat at b = 1/2** ("Student t tails are one kernel call", VERIFIED): TOMS bgrat in x87 extended with three stored
  doubles, lnx, z = fl(-ν·lnx) and ERFC.PRECISE(fl(sqrt z)). lnx comes from Excel's log1p when z <= 0.01.
- **Densities used as prefactors**:
  - "Beta density for min(a,b) <= 1" (model M2; PARTIAL, VERIFIED for max <= 8);
  - "Beta density for min(a,b) > 1 is Loader's saddle point" (PARTIAL);
  - Excel's log1p ("Excel's log1p is a 14-term atanh series", VERIFIED).
- **Rejected forms** are listed in "Incomplete beta: the integrated model, what is open and what is rejected". A port
  task does not retry them without new evidence.
- **Formula literals.** Excel cuts formula number literals to 15 significant digits; it does not round them
  (numbers.md "Formula number literals are read at 15 significant digits"). Two research results were refuted because
  they rounded instead: r2-bratio's "(7.3, 0.6) lnB +24 units" and its "(30.7, 1.7) sd 47". So port rule 3 below.
- **Unknowns, each owned:**
  - Excel's `pow` inside the dispatch tests ((x0·b0)^a0 <= 0.7 and x0^a0 <= 0.9). The model uses libm, and nothing
    is probed at those boundaries. T-2505 probes 2 × 64 cells straddling each.
  - The tiny-shape branch, max(a, b) < 1E-18 → (b/(a + b), a/(a + b)), is TOMS's and unprobed. T-2505 probes it.
  - Whether Excel's bfrac can fail to converge, and what Excel shows then: T-2508 probes the largest shapes.
  - bfrac's iteration count at huge shapes: T-2508 measures it.
  - F's x = 1/(1 + d2/(d1·X)) and 1/(1 + (d2/d1)/X) are not separated, nor whether those divisions round twice:
    T-2506 tests this (r2-impact hypothesis 1).
  - λ's form for a > b, at the swap and in bfrac: T-2508.
  - Whether F.TEST, T.TEST and the inverses' crossings call this kernel at all: their rows below are UNVERIFIED.
    T-2509 probes T.TEST; T-2511 probes F.TEST and the crossings.
- **Relation to decision 0114** (adopted on main while this spec was in review). It tags these port tasks
  `campaign: accuracy`, whose rule is an accurate kernel scored against a high-precision reference and not fitted to
  Excel's last bits. This spec instead ports Excel's measured pieces, because on this family they beat the correctly
  rounded value on visible cells: BEST is visibly off on 3,070 of 40,014 cdf cells against CR's 3,396 (r2-bratio), and
  on T, BINOM and NEGBINOM, decision 0114 itself puts Excel among the inaccurate ones. This is open for the owner. The
  port rules hold either way: the migration rule lets a caller take an accurate piece in place of Excel's, where it
  scores better on visible cells. Its Outcome then names the choice and both scores.

## Design

### Module layout
T-2505 turns `functions/statistical/incomplete_beta.rs` into a directory module. Its callers keep the import path
`incomplete_beta::…`.

| file | holds | written by |
|---|---|---|
| `incomplete_beta/mod.rs` | K: `dispatch`, `evaluate`, `bratio`, `tails_for`; `Branch`, `Caller`, the port table; the (0.5 - w) + 0.5 complement; module doc with the branch map | T-2505 |
| `incomplete_beta/closed.rs` | `closed_form_tails` (T-2502's, moved verbatim) | T-2505 (move) |
| `incomplete_beta/bpser.rs` | Excel's bpser | T-2505 |
| `incomplete_beta/ln_beta.rs` | the single ln B definition (below): `ln_beta_gl(p, q)`, Excel's GL form fl(GL(p) + fl(GL(q) - GL(fl(p + q)))), with the 8 cut chosen by the caller (bpser < 8, density <= 8); `algdiv`, TOMS in doubles with Excel's LN; `ln_beta_sum(a, b)`, the plain `ln_gamma` sum two callers compute inline today | the first of T-2505 and T-2503 to land creates it; the second builds on that file and adds nothing parallel |
| `incomplete_beta/bup.rs` | bup with brcmp1 = fl(P·fl(y0·x0))/b0' and w4's term arithmetic, for every bup call (L140, bup+bgrat) | T-2506 |
| `incomplete_beta/bgrat.rs` | bgrat with its incomplete-gamma ratio: b = 1/2 in extended precision (T-2509); other b with Excel's pieces (T-2510) | T-2509, T-2510 |
| `incomplete_beta/bfrac.rs` | bfrac (λ = fl(a - fl((a + b)·x)), brcomp = density·x·y) and basym | T-2508 |
| `incomplete_beta/density.rs` | `beta_density(x, a, b)`: BETA.DIST(.,FALSE) and the prefactor P. It dispatches M2 for min <= 1 (T-2503) and Loader for min > 1 (T-2507). Today's powf × tgamma form stays here, named, for F.DIST(.,FALSE) (`fisher.rs` `f_pdf`) and for any region whose Excel form has not landed | T-2505 (move); T-2503, T-2507 (bodies) |
| `incomplete_beta/toms.rs` | TOMS 708's plain-double helpers shared outside the port: `rlog1` and `stirling_correction` (`incomplete_gamma.rs` imports them) and `ln_near_one` (`inverse_beta.rs`'s gamma limit and `legacy.rs` use it), all moved | T-2505 (move) |
| `incomplete_beta/legacy.rs` | today's kernel, moved verbatim: Cephes `incbcf`/`incbd`, the mean reflection, the MAXGAM and `large_prefactor` prefactors, `unoriented_shape_one`, `A1_CLOSED_FORM_MIN_X`. It is the delegate (Migration rule) and is deleted with the last one | T-2505 (move) |

- Today's test-only `beta_reg(x, a, b)` becomes a `#[cfg(test)]` helper in `mod.rs` over `bratio`.

**One ln B definition.** Today ln B is formed in four places (`rg -n "ln_gamma\(.*ln_gamma|excel_gammaln\(0.5\)"`):
- `student_t.rs:342`, `log_gamma_ratio`: exp(GL(1/2) + (GL(a) - GL(a + 1/2))), Excel's GL form for B(a, 1/2).
  T-2505 replaces it with `ln_beta_gl(0.5, a)`, the same three GAMMALNs in the same order. If `report-diff` shows
  any changed cell, it stays until T-2509 deletes the composite.
- `t_dist.rs:274` (T.INV's tail start) and `fisher.rs:323` (`f_pdf`), `ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)`
  inline. T-2505 replaces both with `ln_beta_sum`, which is value-identical.
- `incomplete_beta.rs:329`, `large_prefactor`'s `ln_gamma(small) - gamma_ln_shift(small, large)`. This is the one
  copy that stays: it moves into `legacy.rs` with `gamma_ln_shift`, frozen as delegate internals, and is deleted with
  `legacy.rs`. New code never calls it.

Outside the directory:
- `loader.rs` gains Excel's x87 bd0 as `excel_bd0` (T-2507), beside its accurate `bd0`, `bd0_dev` and `stirlerr`;
  the distinct name keeps the two apart. The module doc then covers both kinds. T-2585's T density and T-2146's pmf
  reuse `excel_bd0` and the accurate `stirlerr`.
- New extended-precision elementary functions go next to their double versions: in `x87_exp.rs`, `x87_log10.rs` or
  `x87_log1p.rs`. That covers bgrat's ln and exp in T-2509, for instance, as `Register` or double-double values. They
  never go in a branch file.
- T-2503 and T-2507 were claimed before this spec. If either lands before T-2505, its body sits in today's
  `incomplete_beta::beta_density`, and T-2505 moves it unchanged.

### The kernel's signature
```rust
/// bratio's branch, after its swap. One variant per computing path of the dispatch; the names
/// follow r2-bratio's `bratio_excel.py` labels.
pub(crate) enum Branch {
    Edge,         // x = 0 or y = 0: (0, 1) or (1, 0)
    Tiny,         // max(a, b) < 1E-18: (b/(a+b), a/(a+b))
    Closed,       // b0 = 1, or a0 = 1 with y0 <= 0.9 (T-2502)
    Bpser,        // w = bpser(a0, b0, x0), min <= 1 ("w_bpser"); covers TOMS's fpser region
    W1Bpser,      // w1 = bpser(b0, a0, y0), min <= 1, x0 >= 0.3 ("w1_bpser")
    BpserLarge,   // a, b > 1, b0 < 40, b0·x0 <= 0.7 ("bpser(a,b>1)")
    Bgrat,        // min <= 1, b0 > 15: w1 = bgrat(b0, a0, y0, x0, 0)
    BupBgrat,     // min <= 1: bup(b0, a0, y0, x0, 20), then bgrat(b0 + 20, ...)
    L140Bpser,    // a, b > 1, b0 < 40, b0·x0 > 0.7, x0 <= 0.7: bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0)
    L140Bgrat,    // as L140, x0 > 0.7, a0 > 15: bup + bgrat
    L140BupBgrat, // as L140, x0 > 0.7, a0 <= 15: bup + bup(a0, b0', x0, y0, 20) + bgrat
    Bfrac,        // b0 >= 40, TOMS's bfrac test
    Basym,        // b0 >= 40 otherwise
}

/// The dispatch's result: the branch, whether bratio swapped, the oriented (a0, b0, x0, y0)
/// and λ. No arithmetic beyond the dispatch tests.
pub(crate) struct Dispatch { /* branch, swap, a0, b0, x0, y0, lambda */ }

/// K's value: I_x(a, b) = w and 1 - I_x(a, b) = w1, on the caller's orientation, with the branch.
pub(crate) struct Tails { pub(crate) w: f64, pub(crate) w1: f64, pub(crate) branch: Branch, pub(crate) swap: bool }

pub(crate) fn dispatch(a: f64, b: f64, x: f64, y: f64) -> Dispatch;
/// The branch's Excel pieces where they are implemented, else `legacy::tails(a, b, x, y)`.
pub(crate) fn evaluate(d: &Dispatch) -> Tails;
/// K(a, b, x, y) = evaluate(dispatch(a, b, x, y)).
pub(crate) fn bratio(a: f64, b: f64, x: f64, y: f64) -> Tails;
/// K for one caller: `None` when that caller does not take the branch yet, and the caller then
/// runs its delegate. Dispatch only, so a delegated call costs no kernel evaluation.
pub(crate) fn tails_for(caller: Caller, a: f64, b: f64, x: f64, y: f64) -> Option<Tails>;
```
- `dispatch` is r2-bratio's `kernel` before its first evaluation, with Excel's changes:
  - x0 >= 0.3;
  - the closed forms first in the min <= 1 path;
  - no fpser test and no apser test: those inputs continue down the dispatch, as statistics.md "bpser" measured.
- Its constants are TOMS's: eps = 1E-15, 15·eps for bgrat, 100·eps for basym, n = 20 for bup+bgrat, 40, 15, 0.7.
- The caller forms both x and y (statistics.md); K never forms 1 - x itself. The one exception is `closed.rs`,
  where Excel recomputes 1 - x0.
- The x87 helpers are reused, never copied:
  - every stored op is `arithmetic::extended_sum`, `extended_product` or `extended_quotient` (fl53(fl64(.)));
  - an unstored value is an `x87_register::Register`;
  - Excel's elementary functions are `x87_exp::excel_exp` and `excel_exp_extended`, `x87_log10::excel_ln`,
    `x87_log1p::excel_log1p` and `excel_ln_one_minus`, `gammaln::excel_gammaln`, and `erf::erfc` (ERFC.PRECISE);
  - double-double arithmetic is `x87_exp::{two_sum, dd_add, dd_mul, dd_div}`.
- Nothing in K reads a runtime configuration. The research model's `Cfg` switches become the one measured choice per
  piece. Alternatives that tied are named in the piece's doc comment and in its task Outcome.

### Callers and their argument forms
From statistics.md "Incomplete beta is TOMS 708 `bratio`" and "Student t tails are one kernel call". Each row lists the
`Caller` variant, K's arguments, the tail it reads, and the delegate it keeps until it takes a branch.

| caller (`Caller`) | functions | K's arguments | reads | delegate (today's code, bit for bit) |
|---|---|---|---|---|
| `BetaDist` | BETA.DIST/BETADIST cdf; BETA.INV/BETAINV's `refine_increasing` targets | t = (x - A)/(B - A), K(α, β, t, fl(1 - t)) | w (BETA.INV above p = 0.5: w1 against 1 - p) | `legacy::tails(α, β, t, 1 - t)` |
| `F` | F.DIST(TRUE), F.DIST.RT, FDIST (`fisher.rs` `f_tail`) | d truncated; u = 1/(1 + d2/(d1·X)), v = 1/(1 + (d1·X)/d2); K(d1/2, d2/2, u, v) | w; w1 for .RT and FDIST | `legacy::tails` at d1X/(d1X + d2), d2/(d1X + d2), today's `f_tail_pre_t2502` |
| `FQuantile` (UNVERIFIED as a K call; T-2511 probes it) | F.INV, F.INV.RT, FINV crossings (`f_quantile`) | as `F` | as `F` | as `F` |
| `BetaQuantile` (UNVERIFIED as a K call; T-2511 probes it) | `inverse_beta::beta_lower_crossing` (BETA.INV at a huge shape, F's huge-df quantile) | K(α, β, x, 1 - x) | w | `legacy::tails` (today's unoriented call); the Q(b, -ν ln x) limit stays where it applies |
| `FTest` (UNVERIFIED as a K call; T-2511 probes it) | F.TEST/FTEST's fallback (shape 1, a + b > 200, non-half-integer shapes) in `f_test_upper_tail` | statistics.md "F.TEST / FTEST": x = df_small/(df_big·f + df_small), y = df_big·f/(df_big·f + df_small), each formed directly; x never exceeds the mean, so no reflection. K(df_small/2, df_big/2, x, y) | w | `legacy::tails` (unoriented). T-2361's double-double form stays the main path |
| `Binom` | BINOM.DIST/BINOMDIST(TRUE), after the k = 0 shortcut (T-2645) | K(k + 1, n - k, p, fl(1 - p)) | w1 | the pmf walk (`discrete.rs` `binomial_cdf`), then statrs |
| `NegBinom` | NEGBINOM.DIST/NEGBINOMDIST(TRUE) | K(r, f + 1, p, fl(1 - p)) | w | the pmf walk, then statrs |
| `StudentT` | T.DIST(TRUE), T.DIST.RT, T.DIST.2T, TDIST; T.TEST's tail (UNVERIFIED as a K call; T-2509 probes it) | y = fl(n/fl(n + fl(t·t))), z = fl(fl(t·t)/fl(n + fl(t·t))) (df 2: z = fl(1 - y)); K(n/2, 1/2, y, z) | I = w; L = I/2; 2T = I; the upper side fl(1 - L); y = 1 gives L = 0.5 | `student_t.rs`, including its df-2 closed form, which calls `closed::closed_form_tails` until T-2509 |

- `Caller::takes(self, Branch) -> bool` is the port table, one `const fn` match. Each port task adds the (caller,
  branch) pairs it moves, and nothing else. The table is the single place that says what is ported.
- `Edge` is taken where `Closed` is: it is today's x <= 0 and y <= 0 handling in every delegate. `Tiny` is TOMS's
  (unknown above) and is taken only once T-2505's probe agrees with it.
- After T-2505, `Closed` is taken by `BetaDist`, `F`, `Binom` and `NegBinom`. Today they apply the closed forms, so
  there is no change. It is not taken by `FQuantile`, `BetaQuantile` or `FTest`: T-2502 kept them unoriented because
  the oriented forms moved gated cells (`F_DIST.xlsx` J3/L3, gen-m9/68d8ffb69ff739d8 A143). T-2511 moves them.
- T is not a `tails_for` caller before T-2509. It then takes every branch it reaches (min = 1/2: Closed at df 2, Bpser,
  W1Bpser, Bgrat, BupBgrat) in one step, because r2-tdist's rule is one call for all of them.

### The migration rule
**A branch moves only together with its Excel piece. Everything else delegates to today's code.**
- **Implemented.** A branch's Excel arithmetic lands in its file, and `evaluate` uses it. Until then `evaluate`
  returns `legacy::tails` for that branch. So `bratio` is total at every step.
- **Taken.** A caller takes a branch (a new `Caller::takes` arm) only in the task that implements the branch, or in a
  later one, and only where port rule 2 below holds for that caller. One branch can be taken by BETA.DIST and not
  yet by BINOM. For example, r2-impact finds BINOM's bpser(a,b>1) breaks 8 cells and leaves 48 visibly off.
- **The delegate is today's code, unchanged.** A (caller, branch) pair that is not taken gives exactly today's
  double. `tails_for` returns `None`, and the caller runs the delegate in the table above. A step can therefore
  change only cells whose (caller, branch) it takes. Port rule 2 checks this on every cell.
- **No plain-TOMS stage.** No branch moves to unmodified TOMS 708 arithmetic, with libm or with Excel's elementary
  functions, as an intermediate step (finding 2).
- **Moving a branch with a different piece.** That is a change to this spec, made first, with its scores. T-2509's
  T bgrat is the precedent: r2-impact's E7 had kept T on `student_t.rs`.

### Deleting the T-2502 seams
T-2502 left unoriented entry points so that callers whose Excel cells it had not measured kept their values. Under
this design they become delegates, and each is deleted by the task that removes its last caller.
- **T-2505 deletes the entry points:**
  - `beta_reg_complement_unoriented`, `beta_reg_upper_unoriented`, `beta_reg_complement`, `beta_reg_upper` and
    `beta_reg_tail` with its `closed_forms` flag;
  - every caller goes through `tails_for` (or `bratio`) and, on `None`, its delegate.
  - The only unoriented kernel left is `legacy::tails(a, b, x, y) -> (f64, f64)`. Per tail it is exactly today's
    `beta_reg_tail(…, closed_forms = false, …)`: lower from (x, a, b) and upper from (y, b, a), each through
    `unoriented_shape_one` and then `beta_tails`.
  - `student_t.rs`'s imports of `incbcf`, `incbd` and `beta_reg_upper` become `legacy::{incbcf, incbd, tails}`.
- **The delegates' internals stay in `legacy.rs`:**
  - `unoriented_shape_one` and the `A1_CLOSED_FORM_MIN_X` branch serve a0 = 1 with y0 > 0.9. That input reaches Bpser
    (x0 < 0.1 with b0·x0 <= 0.7), Bgrat and BupBgrat (b0·x0 > 0.7). They serve any caller that has not taken the branch
    the input reaches: after T-2505 that is Bpser for a caller that did not take it, and Bgrat and BupBgrat for all.
    They are deleted by T-2510 if no caller then delegates any of those three branches; otherwise by the task that
    ports the last one.
  - F's pre-T-2502 argument forms (today's `f_tail_pre_t2502`, renamed `f_tail_delegate`) are deleted when `F` and
    `FQuantile` take every branch they reach: T-2510 for `F`, T-2511 for `FQuantile`, unless a branch stays delegated.
  - `legacy.rs` as a whole goes when no `Caller` delegates to it. T-2511 states which (caller, branch) pairs, if any,
    still delegate, and names their owner (T-2512 or a follow-up).
- **The research harness** (`t2464_harness.rs`) is deleted by T-2505, as its task requires, only if T-2512 and T-2586
  are done. Otherwise it stays, and the last of those two deletes it, with docs/research/t2464/README.md updated.
  `excelmath.py` uses it.

### Bounded work, no panics (directive 5)
- **Every loop has a named `const` cap** with a doc comment that gives the convergence bound and the largest count
  measured on the task's data. A unit test at the worst point found shows the loop ends below the cap. Reaching the
  cap returns the partial result, never a panic and never a NaN for valid input:
  - bpser's "until s stops changing". The cap is derived for any shape, the largest included: BINOM passes shapes up to
    about 2^31 (k + 1, n - k with n < 2^31 - 1), and BETA.DIST has no bound. Every bpser call has x0 <= 0.7, or
    b0·x0 <= 0.7, or y0 <= 0.7 (W1Bpser). So from the first term on, the term ratio |(1 - b/n)·x| is at most
    max(0.7, b0·x0/n) <= 0.7, whatever the shape. A term then falls below 2^-64 of s, which is at least the first
    term, within 125 steps, and the factor a/(a + n) only shortens that. T-2505 sets the cap with that margin (for
    example 256) and tests the largest shape class;
  - bup: n <= 40 terms (L140) or 20 (bup+bgrat), by construction;
  - bgrat: 30 terms. Its coefficient tables are fixed-size arrays (`[f64; 30]`), not `Vec`;
  - basym: TOMS's fixed 20;
  - bfrac: a cap that T-2508 sets from measured counts up to the largest shapes the callers accept (F's df go
    to 1E10). If the cap can bind on a valid input, T-2508 records a `DEVIATION (decision 0014)`
    entry in statistics.md, stating Excel's value and ours past it;
  - `legacy.rs` keeps its caps (300 fraction steps, 20 `rlog1` terms);
  - the inverse searches (T-2511): one named cap on the cdf evaluations of each search. `special::refine_increasing`
    has one today (at most 129 cdfs), and so do `nearest_crossing` and `beta_quantile_crossing` (at most 67). The
    lifting weight charges the cap times K's per-call bound.
- **Checked conversions.** Integer counts taken from shapes use checked conversions, for example `u32::try_from` on
  `trunc()` with a fallback branch: bup's n = int(b0) (b0 < 40) and any loop count. An `as` cast on an unbounded f64
  is not used.
- **K is total and allocation-free.**
  - Inputs:
    - NaN in any argument gives NaN in both tails;
    - x <= 0 gives (0, 1), and y <= 0 gives (1, 0);
    - x > 1 or y > 1 gives NaN in both tails (no caller passes them);
    - a non-positive or infinite shape gives NaN;
    - x + y = 1 is the callers' contract, to within their own roundings (T's y and z, F's u and v, BINOM's p and
      fl(1 - p)). K does not test it, unlike TOMS's ierr = 4: each branch reads only the arguments it is defined on,
      as Excel's formulas do. For any (x, y) in [0, 1]^2 K still returns finite values in [0, 1] without a panic. A
      property test feeds random pairs with |x + y - 1| up to 1E-3.
  - The callers' existing guards decide the error: `number_or_error` maps a non-finite tail to `#NUM!`. K itself never
    makes an error value.
  - No `unwrap`, `expect` or unchecked indexing, and no recursion: the swap is applied once.
- **Excel's `#NUM!` edges stay in the callers**, with their entries:
  - BETA.DIST α, β <= 0, x outside [A, B], A = B;
  - F's truncated df outside [1, 1E10] (statistics.md "F-distribution df limit (T-1530)");
  - BINOM.DIST from 2^31 - 1 trials;
  - NEGBINOM.DIST's argument rules.

  No kernel-level `#NUM!` is known. A probe that finds one (bfrac at huge shapes, say) is recorded and implemented as
  a caller rule.
- **Saturation.**
  - A tail that underflows takes Excel's EXP model's result: `excel_exp` and its subnormal handling. The (0.5 - w) +
    0.5 complement then rounds to 1.
  - T keeps `student_t.rs`'s subnormal flush until T-2509 measures it on K.
  - The excel-graph oracle shows a subnormal as 0 (decision 0042), so a subnormal against an oracle 0 is not evidence
    against a branch.
- **Lifting cost.** The lifting table charges incomplete-beta callers by a per-call bound:
  - `functions/lift/kinds/statistical.rs` `QUANTILE_WEIGHT` and the BINOM.INV weight;
  - `functions/lift/tests/kernel_work.rs` `ENGINE_BETA_CDF`.

  Each port task restates that bound for the new branch's caps, re-measures the slowest point in release, and raises a
  weight if the bound grows.

### Gated regressions and how they are handled
The gated `eval-check` sees the vendored gated suites. r2-impact's `branches.out` names every gated cell the full model
moves. Each task handles its own cells, measured on its own head. A decision record cannot be approved before its cells
are measured, so there is no umbrella record.

| cell | function, branch | model vs Excel | task | handling |
|---|---|---|---|---|
| gen-m9/0a614cfe974b59c3 A19 | F.DIST.RT, BpserLarge | 1 ulp, bit-exact lost | T-2505 | decision 0083's allowance |
| gen-m9/05d44f6dd6cfc567 A175 | FDIST, Bpser | 1 ulp, bit-exact lost | T-2505 | decision 0083's allowance |
| gen-m9/60b0b6e69ad35a45 A151 | F.DIST.RT, BpserLarge | 6 ulp at 6.6E-44: already failing, becomes visibly off | T-2505 | not covered by 0083 (condition 2): a decision record, or `F` keeps the cell's region delegated |
| `F_DIST.xlsx` J3/L3, gen-m9/68d8ffb69ff739d8 A143 | F.INV, F.TEST, Closed | 1-2 ulp under the oriented closed forms (T-2502) | T-2511 | `FQuantile` and `FTest` take `Closed` there; 0083's allowance if still <= 4 ulp |
| probes/stat-argument-modes H215, H221, H226 | F.DIST, F.DIST.RT, FDIST at (2.5, 2.5, 0.6000000000000001), L140Bpser | 2-4 ulp, bit-exact lost | T-2506 | test r2-impact hypothesis 1 first (F's x form, bup's W rounding at b0' = 0.5); then 0083's allowance |
| gen-m9/2801c1c11c21e134 A374 | FDIST, L140Bpser | 6 ulp: already failing, becomes visibly off | T-2506 | a decision record, or `F` keeps the region delegated |
| gen-m9/c18dbf21f11093f3 A151 | FDIST, BupBgrat | 7 ulp, bit-exact lost | T-2510 | beyond 0083's 4 ulp: a decision record, or `F` keeps BupBgrat delegated (T-2510 moves a branch only where it gains per function) |
| `BETADIST_BETAINV.xlsx` H2, H23, H42-H49, H67 | BETA.INV chained on BETA.DIST | already known failures (T-2119, decision 0111) | every task that moves `BetaDist` | the Outcome states each cell's old and new ulp; a cell that becomes visibly off needs a record. T-2633 now depends on T-2505 (Rollout) |
| ironcalc-calc incomplete-beta workbooks | args unresolved in r2-impact | not scored | every task | each task's Outcome lists them from its eval diff |

- Decision 0083's allowance holds only while all four of its conditions do: numeric, <= 4 ulp, no cell newly visibly
  off, net visible gain. The Outcome lists every cell, and a follow-up ulp task names them.
- Anything else needs a decision record in the same PR, made with `scripts/new_decision.py`, in the form of
  decisions 0106 and 0111. It names each cell and its ulp distance. A cell that becomes visibly off in a gated suite
  needs the owner's ruling, as in decision 0111.
- **The narrower alternative is always open.** A caller can keep the branch, or the measured region of it that holds
  the cell, on its delegate. The Outcome then names that region.

### Port rules (every port task: T-2503, T-2505 to T-2511, T-2585)
1. **The migration rule** above. The `Caller::takes` diff is the list of what the PR moves, and the Outcome repeats
   it.
2. **probes-numeric, cell by cell: net gain, listed losses** (the owner's ruling of 2026-10-25, with its thresholds:
   decision 0115).
   - The gated `eval-check` cannot see the ungated probes-numeric suite (`gate = false`), which holds most of this
     family's cells.
   - Every port task runs `rustsheet-eval run` over probes-numeric on origin/main (the merge base) and on HEAD, and
     compares the two with `rustsheet-eval report-diff` (T-2679).
   - Per function (the specialised function the report blames, which is per `Caller`), a port is allowed only if:
     - its fixed cells are at least 2 × its lost cells. A lost cell is one that was bit-exact and is not now, or one
       already off that moved further from Excel's value;
     - its visibly-off count goes down, net;
     - a function with no fixed cell has no lost cell.
   - No cell may become visibly off (beyond half a unit in the 15th significant digit), and no cell may change type or
     error, unless Excel agrees with the new value. "Excel agrees" means another oracle reading of the same formula
     gives the new value (the build-dependent bits of decision 0112, a duplicate cell in another case). Or it means the
     cell's input is a misparsed xlsx number, checked at x ± 1 ulp (numbers.md "Stored numbers in an xlsx are not
     always parsed to the nearest double"). Each such cell is named with its evidence.
   - The Outcome lists every lost cell with its ulp distance, or cites a committed file that holds the list when it
     is long. It also gives `report-diff`'s per-function counts and ratio.
   - A task that misses a threshold for a function narrows what it moves for that caller, by branch or by a measured
     region of one. Otherwise it needs a decision record naming the cells.
   - Gated suites are unchanged: 0 regressions, or decision 0083's allowance, or a decision record (port rule 4).
   - `report-diff` is its own task, T-2679. It is built on `closeness.rs` (`Visible`, `within_fifteen_digits`,
     `blamed`) and `visible_parity.rs`, and it reuses `baseline::diff` for the fixed and lost sets. One definition, so
     each task produces the lists mechanically. Until it lands, T-2503 and T-2507, already in progress, may compare
     the two reports by hand, with the same buckets.
   - The lost cells are bit-level debt. They go to the task's bit-level follow-up (decision 0083), or to T-2512 or
     T-2586. r2-impact's ladder estimates their size: 362 lost bit-exact cells by step S3, 514 by S7, against about
     1,450 visibly-off cells fixed.
3. **Formula literals are cut to 15 significant digits.**
   - Expected values and fixture rows built from formula text use the arguments as Excel reads them: cut to 15
     significant digits, never rounded and never the Python `float` of the literal text. Taking the parsed formula
     from the engine, or `rustsheet-eval snapshot`, does this.
   - New probes keep literals at 15 significant digits or fewer, or pass exact arguments through cells.
4. **Gated regressions:** the table above; 0083's allowance or a decision record, nothing silent.
5. **Evidence.** Every piece cites its statistics.md entry in its doc comment. A quirk found while porting goes into
   that entry in the same PR (directive 6). The entry gains an "Implemented in" line, and its "Ours" line is updated.
6. **Tests do not read docs/** (decision 0066). Fixture rows go under `crates/rustsheet-engine/src/test/fixtures/`, in
   the T-2645 format (bits in hex, a header naming the source data, the generating script and its configuration).
   Counts over research data (cells.tsv, beta_exact.tsv) come from the eval, or from an `#[ignore]` test that reads a
   path from an environment variable (T-2645's `RUSTSHEET_T2464_LOG1P`), never from a docs literal.

## Alternatives considered
- **A plain TOMS 708 port first, then the Excel pieces** (the order a textbook port would take). Net -103 visible, or
  -51 with Excel's elementary functions, on the non-BETA callers, and 1,740 / 1,666 bit-exact lost (r2-impact
  finding 2). Rejected.
- **The whole BEST model in one PR.** Every branch at once: 893 bit-exact cells lost (6 gated), 213 broken, and an L
  size against directive 4. Rejected.
- **Branch-level switching without callers.** One switch per branch for every caller. BINOM's bpser(a,b>1) and T's
  bgrat would then move while they lose: plain TOMS bgrat fixes 89 T cells and breaks 145. Rejected for the
  (caller, branch) table.
- **The research model's runtime `Cfg`.** It would ship dead configurations and untested combinations. Rejected: each
  piece is the one measured choice, and the ties are named.
- **Keeping T on its own kernel** (r2-impact E7). r2-tdist's one-call rule, with bgrat at b = 1/2 in extended
  precision, takes the T tails from 1,286 visibly-off cells to 7. Rejected.
- **Accurate (correctly rounded) pieces instead of Excel's, as the default.** On this family's cdf cells the
  correctly rounded value is visibly off on 3,396 of 40,014, against BEST's 3,070 (r2-bratio). TOMS's own pieces lose
  to Excel's in every one-piece ablation (bpser +961 against +1,395). Not adopted as the default. A caller may still
  take an accurate piece where it scores better on visible cells (see "Relation to decision 0114").
- **Densities in their own top-level module.** They are the bup and bfrac prefactors, so they sit with the kernel
  (`incomplete_beta/density.rs`). The Loader pieces stay in `loader.rs`, which three density tasks share. Rejected.

## Test plan
Two kinds of evidence for each branch:
- **Port correctness (positive control).** A fixture generated from the configuration that ships, not from r2-bratio's
  BEST: BEST keeps TOMS's fpser and apser and `w3_lg="toms"`. The generator is r2-bpser-small's
  `lib.py` `V(nofpser=True, noapser=True, lg="GL53/glalgdiv")` on r2-bratio's kernel, with the engine's GAMMALN model
  (`R2_PROBED_GL=0`), so that model and port share every elementary function. Later branches extend the same
  configuration with their pieces. The port equals the model bit for bit on
  every fixture row of the branches it implements. The rows include both swap states and points at each boundary:
  x = 0.5, λ = 0, x0 = 0.3, x0 = 0.1, b0·x0 = 0.7, x0 = 0.7, b0 = 15, b0 = 40, a0 = 15, integer and non-integer b0 in
  L140, and the y0 = 0.9 cut. A separate fixture checks `dispatch` against the model's branch labels on 100% of its
  rows.
- **Excel agreement.** The floors in each task's acceptance criteria, demonstrated by the eval on the named
  probes-numeric cases, plus port rule 2's comparison. The numbers live only in the task files; where this spec
  and a task file differ, the task file wins.

| branch | task | Excel-agreement evidence (floors: the task file) | extra checks |
|---|---|---|---|
| Edge, Tiny | T-2505 | the tiny-shape probe agrees before Tiny is taken | NaN, ±0, subnormal x, 1 - 2^-53; x, y outside [0, 1]; x + y ≠ 1 |
| dispatch | T-2505 | t2464-r2-bratio-x29 | the `pow` boundary probes (Excel behavior, unknowns) |
| Closed | T-2502 (done) | w5's closed-form set, kept | T-2502's cut tests move with the code |
| Bpser, W1Bpser, BpserLarge | T-2505 | t2464-r2-bpser-small-shapes and the old sets, scored with the shipped configuration (GL53, no fpser or apser; verify-r2-bpser-small `old_sets.out`), both L forms reported | no fpser or apser: t2464-r2-bpser-small-tiny, -tiny2 |
| L140Bpser | T-2506 | r2-bratio's L140 sets; ix L140 | H215/H221/H226 (hypothesis 1) |
| Bfrac, Basym | T-2508 | r2-bratio's bfrac cells; the (100, 120) run | the cap at the largest shapes; the non-convergence probe; λ at a > b |
| Bgrat, BupBgrat for T | T-2509 | T bgrat cells; t2464-r2-tdist-p1 part C; the T family on cells.tsv | wrapper identities bit for bit (2T = 2·RT, ...); T.TEST as a K call |
| Bgrat, BupBgrat, L140Bgrat, L140BupBgrat, other callers | T-2510 | net visible gain per function; non-T b = 1/2 cells | c18dbf21 A151 |
| density min <= 1 (prefactor P) | T-2503 | its criteria | no F.DIST(.,FALSE) change |
| density min > 1 | T-2507 | its criteria | the `excel_bd0` fixture shared with T-2585 |
| inverses | T-2511 | t1745-beta-inv-roundtrip; the 1,207 numeric inverse cells | bounded on T-1871's cells; the search caps; F.TEST and the crossings as K calls |

- **Metamorphic properties**, as unit tests over the fixtures' points:
  - w + w1 = 1 to within the complement's rounding, and exactly (0.5 - w) + 0.5 where Excel's form says so;
  - K(a, b, x, y) and K(b, a, y, x) give swapped tails bit for bit;
  - BINOM.DIST(r - 1, f + r, p, TRUE) and NEGBINOM.DIST(f, r, p, TRUE) are exact complements wherever both take the
    branch (statistics.md: 441 of 460 pairs);
  - F.DIST + F.DIST.RT = 1 at the mean (t1049-fdistrt A27).
- **Performance.** The kernel benchmark (or a release-mode timing in the Outcome) at the slowest point of each new
  branch; the lifting bound is restated (directive 5 section). No budget beyond decision 0081's "no more than 50%
  slower" on the recalc benchmark.

## Rollout
Ordered by `depends_on`. Steps with the same number can run in parallel.

**T-2505's split point.** T-2505 is M at the upper end. If it outgrows M, it splits at the value-neutral refactor:
- part 1: the directory module, `legacy.rs`, `toms.rs`, `closed.rs`, `dispatch`, `Branch`, `Caller` and the port
  table with only `Closed` and `Edge` taken; the entry points deleted; the ln B copies replaced. `report-diff` shows
  0 changed cells in every suite;
- part 2: `bpser.rs` and `ln_beta.rs`'s `algdiv`, and the bpser branches taken.

The worker files part 2 as a new task with T-2505's bpser criteria and moves them there, as task-format.md's "one
task, one PR" requires.

| step | task | module / branch | callers that take it | depends_on |
|---|---|---|---|---|
| done | T-2645 | `x87_log1p.rs`, `excel_exp_extended`; BINOM k = 0 / k = n | BINOM | — |
| done | T-2502 | `closed_form_tails` | BetaDist, F, Binom, NegBinom; T df 2 | — |
| 1 | **T-2504** (this spec) | — | — | — |
| 2 | T-2503 (in progress) | `density.rs` M2 (min <= 1); `ln_beta.rs` if first | BETA.DIST(.,FALSE) | T-2645 |
| 2 | T-2507 (in progress) | `density.rs` Loader (min > 1); `excel_bd0` in `loader.rs` | BETA.DIST(.,FALSE) | T-2645 |
| 2 | T-2679 | `rustsheet-eval report-diff` (harness) | — | T-2504 |
| 3 | T-2505 | directory module, `dispatch`, `legacy.rs`, `bpser.rs`, `ln_beta.rs`; Edge, Bpser, W1Bpser, BpserLarge; the entry points deleted | BetaDist, F, Binom, NegBinom, where rule 2 holds | T-2645, T-2502, T-2504, T-2679 |
| 3 | T-2585 | T density (`student_t.rs`) on `loader.rs`'s `excel_bd0` and `stirlerr` | T.DIST(.,FALSE) | T-2645, T-2507 |
| 4 | T-2506 | `bup.rs`; L140Bpser | BetaDist, F, Binom, NegBinom | T-2503, T-2505 |
| 4 | T-2508 | `bfrac.rs`; Bfrac, Basym | BetaDist, F, Binom, NegBinom | T-2505, T-2507 |
| 5 | T-2509 | `bgrat.rs` at b = 1/2; T becomes a K caller | StudentT (all its branches) | T-2645, T-2505, T-2506 |
| 6 | T-2510 | `bgrat.rs` other b; Bgrat, BupBgrat, L140Bgrat, L140BupBgrat; deletes `unoriented_shape_one`, `A1_CLOSED_FORM_MIN_X` and, for `F`, `f_tail_delegate` where nothing delegates to them | BetaDist, F, Binom, NegBinom, where it gains | T-2505, T-2506, T-2509 |
| 7 | T-2511 | inverses on K; `FQuantile`, `BetaQuantile`, `FTest` take their branches; `legacy.rs` deleted or its remaining pairs named | all inverse callers | T-2505, T-2506, T-2508, T-2509, T-2510 |
| any time | T-2512, T-2586 (research) | amend T-2505, T-2506, T-2508, T-2510 (T-2512) and T-2507, T-2585, T-2146 (T-2586) when a piece is solved | — | — |

`depends_on` check (T-2504):
- Every listed edge matches what the code needs:
  - bpser needs `excel_exp_extended` (T-2645);
  - bup's prefactor is the min <= 1 density (T-2503);
  - bfrac's prefactor is the min > 1 density (T-2507);
  - T's bup+bgrat needs bup (T-2506);
  - T-2510's b = 1/2 bgrat is T-2509's;
  - T's density needs T-2507's bd0.
- Every port task already depended on T-2505, and through it on this spec, except T-2503, T-2507 and T-2585. Those
  three only add bodies to the density module and to `loader.rs`. T-2503 and T-2507 were claimed before this spec,
  so they get no new edge. The layout above accommodates either landing order. Their task files get only a `spec:`
  line, so that their branches do not conflict, but the port rules bind them too. T-2503's "Bit-exact
  probes-numeric cells lost are listed in the Outcome" already matches port rule 2 (decision 0115).
- **New: T-2679** (`rustsheet-eval report-diff`, harness lane), split out of T-2505. T-2505 now depends on it, and
  so, through T-2505, does every later port.
- **Changed: T-2633** (`depends_on: []` → `[T-2505]`). T-2633's goal fits BETA.DIST's last bits and the chained
  BETA.INV cells on today's Cephes kernel, which T-2505 replaces for those cells:
  - t0969-beta-prefactor D85/D86 sit in the bpser branches;
  - gen-m9/35e8168aae2b00f8 A159, `BETADIST(1E-10,0.5,10000000000.5)`, is Bgrat (T-2510).
  So a fit made first would be moved again. After T-2505, T-2633 fits the ported kernel or names the cells T-2510
  owns.

What stays on the old path until when:
- each (caller, branch) pair stays on its delegate until a task takes it, per the table above;
- T stays on `student_t.rs` until T-2509;
- the inverses stay unoriented until T-2511;
- F.DIST(.,FALSE) stays on today's density until the F density is solved (T-2586, T-2095);
- BINOM.DIST.RANGE, BINOM.INV and CRITBINOM stay on the walk until a probe ties them to the kernel. That probe is a
  follow-up for T-2505 to file.
