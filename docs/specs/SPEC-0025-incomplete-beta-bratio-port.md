# SPEC-0025 — Port Excel's TOMS 708 `bratio` incomplete beta, one branch at a time

- **Status:** draft
- **Milestone / lane:** M9 / functions-stat
- **Tasks:** T-2504 (this spec); T-2505, T-2506, T-2508, T-2509, T-2510, T-2511 (the port); T-2507 (the min > 1
  density it needs); T-2633 (T-2119's chained BETA.INV cells)

## Problem
T-2464 found the kernel behind BETA.DIST, BETADIST, F.DIST, F.DIST.RT, FDIST, F.TEST, T.DIST*, TDIST, BINOM.DIST,
BINOM.DIST.RANGE and NEGBINOM.DIST. It is one call, K(a, b, x, y), into a port of Didonato and Morris's TOMS 708
`bratio`, the 1992 Fortran. Each branch is built from Excel's own EXP, LN, GAMMALN and log1p, with x87 stores in
fixed places (docs/excel-semantics/statistics.md "Incomplete beta is TOMS 708 `bratio`" and the T-2464 entries after
it). The engine has five different paths instead:
- `incomplete_beta.rs`: Cephes' fractions, a switch at the mean for every shape, and the complement as 1 - v;
- `beta.rs`: the BETA.DIST cdf on `beta_reg` since T-2119;
- `student_t.rs`: its own composite;
- `binom.rs` and `discrete.rs`: a pmf walk outside the closed forms;
- `inverse_beta.rs` and `fisher.rs`: statrs inverses, with F's x formed as d1X/(d1X + d2).

docs/research/t2464/r2-impact/RESULTS.md section 4 gives the size of the gap and the ladder that closes it. The
ladder removes about 1,800 of the 5,533 pre-T2464 visibly-off cells, and 22,413 of the 23,657 on the w2f probes.

A one-shot port is net negative: a plain TOMS port outside BETA.DIST is -103 visible on 10,602 cells (r2-impact
finding 2). Each branch only gains once its Excel-specific arithmetic (the density prefactor, bpser's exponent, bgrat at
b = 1/2) lands with it. So the port has to land branch by branch, behind a dispatcher that sends every branch not yet
ported to today's kernel.

## Goals / non-goals
Goals:
- One kernel module with one entry point, K(a, b, x, y) → (w, w1, branch), that every incomplete-beta caller uses.
- Each caller forms its own (a, b, x, y) as Excel does.
- A migration in which each later task switches named branches from the old path to the port and changes no other
  cell.
- Directive 5 throughout: bounded loops, no panics, no allocation that depends on the input.

Non-goals:
- The F density, the t density and the BINOM/NEGBINOM pmf. Those are separate kernels with no model yet (r2-impact
  section 4).
- F.TEST's error precedence (T-2093).
- Any engine code. This spec is documentation only.

## Excel behavior to match
Everything below is VERIFIED (excel-graph) in docs/excel-semantics/statistics.md unless marked otherwise.

- **Dispatch** ("Incomplete beta is TOMS 708 `bratio`"):
  - For min(a, b) <= 1, swap to (b, a, y, x) when x > 0.5.
  - For min(a, b) > 1, swap when λ = a - (a + b)x < 0. When a > b, λ is formed as (a + b)y - b.
  - The branch order is the 1992 Fortran's: closed forms, then fpser, apser, bpser, w1 = bpser(b0, a0, y0), bup +
    bgrat, bgrat (b0 > 15), L140 (b0 < 40: bup + bpser, bup + bgrat, bup + bup + bgrat), then bfrac or basym
    (b0 >= 40).
  - The w1-bpser cut is x0 >= 0.3, not R's 0.29.
  - `docs/research/t2464/r2-bratio/bratio_excel.py` `kernel` is the line-by-line model.
- **One evaluation**: bratio computes one tail, and the other is `(0.5 - w) + 0.5`, x87 stored. F.DIST and F.DIST.RT
  at one point are one evaluation on 1,578 of 1,578 pairs.
- **Closed forms on the oriented arguments**: "Incomplete beta closed forms act on the oriented arguments". These
  landed in T-2502 as `closed_form_tails`.
- **bpser**: "Incomplete beta bpser: a forward series times EXP(fl(a LN x) - lnB)". It is a forward x87 series from
  fl(1/a), I = fl(EXP-ext(z)·s), with z = fl64(fl(a·LN x) - L). L is r2-bpser-small's GL53/glalgdiv ln B.
- **L140 and bfrac**: "Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y". For
  min <= 1 that density is T-2503's `beta_density_small_shape`; for min > 1 it is T-2507's Loader density.
- **Student t**: "Student t tails are one kernel call, K(n/2, 1/2, y, z), with bgrat at b = 1/2 in extended precision".
  Excel's bgrat at b = 1/2 is `r2-tdist/xlk.py` `bgrat_half_ld`.
- **What is still open**: "Incomplete beta: the integrated model, what is open and what is rejected". The T bgrat
  family at (n/2, 1/2) with n/2 >= 8 is open, and so are bfrac's own noise and the Loader constant.

### Each caller's arguments
The callers form x and y themselves, never 1 - x inside the kernel (w5-triangulation 2.3; statistics.md "Caller
arguments").

| caller | K(a, b, x, y) | result |
|---|---|---|
| BETA.DIST(.,TRUE), BETADIST | (α, β, x', fl(1 - x')), with x' after the [A, B] rescale | w |
| F.DIST(.,TRUE) (d1, d2 truncated) | (d1/2, d2/2, 1/(1 + d2/(d1·X)), 1/(1 + (d1·X)/d2)) | w |
| F.DIST.RT, FDIST, F.TEST's tail | the same call | w1 |
| BINOM.DIST(k, n, p, TRUE) | (k + 1, n - k, p, fl(1 - p)) | w1. The k = 0 shortcut EXP(fl(n·l)) of T-2645 stays in front |
| NEGBINOM.DIST(f, r, p, TRUE) | (r, f + 1, p, fl(1 - p)) | w |
| T.DIST.RT(t, n) and T.DIST.2T | (n/2, 1/2, y, z): y = fl(n/fl(n + fl(t·t))); z = fl(fl(t·t)/fl(n + fl(t·t))), or fl(1 - y) at df 2 | L = w/2 (T.DIST.RT); w (T.DIST.2T, TDIST(.,.,2)); upper side fl(1 - L); y = 1 → L = 0.5 |
| T centre (t <= 1, df ≠ 2) | 0.5 - BETA.DIST(z, 1/2, n/2)/2, z formed from t | 19 of 23 cells, so the rule is UNVERIFIED and stays on `student_t.rs` |

On F, the forms 1/(1 + d2/(d1X)) and 1/(1 + (d2/d1)/X) are not separated: they differ on 2 of 566 closed cells. The
spec picks the first, as T-2502 did.

## Design

### Module layout
Today's `functions/statistical/incomplete_beta.rs` is 712 lines. It holds the old kernel plus helpers that other
modules import (`rlog1`, `stirling_correction`, `ln_near_one`, `incbcf`, `incbd`). The port gets a directory module
next to it:

```
functions/statistical/bratio/
  mod.rs        K(): the dispatch, the swap, `finish`, the `Branch` enum, delegation to the old kernel
  bpser.rs      bpser, fpser, apser (T-2505)
  bup.rs        bup and L140's combinations (T-2506)
  bfrac.rs      bfrac and basym (T-2508)
  bgrat.rs      bgrat, and bgrat at b = 1/2 in extended precision (T-2509, T-2510)
  ln_beta.rs    GL53 / glalgdiv ln B (T-2505), reusing gammaln.rs and incomplete_beta's algdiv pieces
```

`incomplete_beta.rs` keeps the old kernel as the delegate. Its shared helpers stay where they are; the port imports
them and copies none (directive 3). Once every branch has moved (after T-2511), a cleanup task deletes the old
kernel and `t2464_harness.rs`.

### Signature

```rust
/// Which bratio branch produced the value, for tests and the research harness.
pub(crate) enum Branch { Zero, One, Tiny, Closed, Fpser, Apser, WBpser, W1Bpser, BupBgrat, Bgrat,
                         BpserLarge, L140BupBpser, L140BupBgrat, L140BupBupBgrat, Bfrac, Basym }

pub(crate) struct Tails { pub w: f64, pub w1: f64, pub branch: Branch, pub swapped: bool }

/// Excel's K(a, b, x, y): I_x(a, b) and 1 - I_x(a, b), with x and y formed by the caller.
pub(crate) fn bratio(a: f64, b: f64, x: f64, y: f64) -> Option<Tails>;
```

- `None` is the domain error. Callers already turn it into `#NUM!`; their own argument checks stay in front of the
  call.
- `w` and `w1` are what the model's `_finish` returns: the computed tail, and the other as `(0.5 - w) + 0.5`. The
  computed tail goes through an x87 store, `x87_register` (`Register`); no new extended type is added.
- `branch` names the branch even when its arithmetic is still delegated. That keeps the dispatch testable from T-2505
  on.

### Migration rule
- `bratio()` always runs the 1992 dispatch, and each branch has a `ported` flag: a `const` match arm, not a runtime
  setting.
- A branch that is not ported calls today's code for the caller's tail and keeps today's value exactly. The
  delegate is `incomplete_beta::beta_reg_tail` for BETA.DIST/F, `student_t.rs` for T and the pmf walk for BINOM and
  NEGBINOM. So the caller passes its legacy evaluator as a closure, `bratio_or(a, b, x, y, legacy)`, and a cell
  whose branch has not moved is bit-identical to main.
- A task switches only the branches its piece completes, and only for the callers its evidence covers. Where a
  piece is net negative for one caller it stays delegated for that caller: T's bgrat family stays on `student_t.rs`
  in T-2510 (+60 visible if switched).
- Callers move onto `bratio_or` in T-2505, with every branch still delegated except those T-2505 ports. Each later
  task then flips flags and nothing else in the callers.

### Arithmetic and storage
- **Excel elementary functions**:
  - `x87_exp::excel_exp` and `excel_exp_extended`;
  - `x87_log10::excel_ln`;
  - `x87_log1p::excel_log1p` (T-2645);
  - `gammaln::excel_gammaln`;
  - ERFC.PRECISE's kernel, for bgrat at b = 1/2.

  No libm `exp`, `ln` or `tgamma` inside a ported branch.
- **Extended precision** reuses `arithmetic::{extended_sum, extended_product, extended_quotient, extended_sqrt}`,
  `arithmetic::register::{extended_of, extended_mul, extended_div}` and `x87_register::Register`, with no copies. A
  helper that is missing (for example a stored extended difference) goes into `arithmetic/register.rs` in the task
  that needs it.
- **Densities**: L140 and bfrac/basym take their prefactor from Excel's density times x·y. They call
  `beta_density::beta_density_small_shape` (min <= 1) and, once T-2507 lands, the Loader density in `loader.rs`
  (min > 1). They do not call a second copy.

### Bounded work and edges (directive 5)
- Every series and fraction has a constant cap. bpser and bup follow TOMS's own termination plus a hard cap (a named
  `const`, as `incomplete_beta::MAX_FRACTION_TERMS` is today). bfrac's fraction gets the same kind of cap. bgrat and
  basym use fixed-size coefficient arrays (TOMS's 30 and 20 terms).
- L140's n = int(b0) is below 40. bup's 20-step shifts are fixed.
- No recursion and no `Vec` in the kernel.
- `bratio_or` calls its legacy closure at most once per evaluation, and the delegates keep their existing caps
  (`MAX_FRACTION_TERMS`, the pmf walk's bound), so delegation adds no unbounded work.
- Non-finite or out-of-domain input returns `None` before dispatch: a, b <= 0, x or y outside [0, 1], or x + y not
  within TOMS's 3·eps of 1.
- The edges of the model, in order: x = 0 → (0, 1); y = 0 → (1, 0); max(a, b) < eps·0.001 → (b/(a+b), a/(a+b)).
- Excel's saturation edges stay where the callers have them today: F.DIST's large-d and huge-X cells, T's y = 1 → 0.5,
  and BINOM's k = 0 shortcut. Each task re-runs those callers' existing `#NUM!` and saturation probes.
- No `unwrap`/`expect`/`panic`. An integer conversion (`int(b0)`, `n`) goes through a checked cast that is bounded
  by the branch's own guard (b0 < 40).

### Gated regressions
The full port moves cells that pass today (r2-impact `branches.out`):

| cell | branch | moves in | effect |
|---|---|---|---|
| probes/stat-argument-modes F.DIST H215, F.DIST.RT H221, FDIST H226 | L140 bup+bpser | T-2506 | 2-4 ulp, bit-exact lost |
| gen-m9 F.DIST.RT 0a614cfe A19 | bpser(a,b>1) | T-2505 | 1 ulp |
| gen-m9 FDIST 05d44f6d A175 | w_bpser | T-2505 | 1 ulp |
| gen-m9 FDIST c18dbf21 A151 | bup+bgrat | T-2510 | 7 ulp |
| gen-m9 F.DIST.RT 60b0b6e6 A151 | (r2-impact) | by branch | 6 ulp at 6.6e-44, visibly off |
| gen-m9 FDIST 2801c1c1 A374 | (r2-impact) | by branch | 6 ulp, visibly off |
| ironcalc-calc BETADIST_BETAINV.xlsx, T-2119's nine chained BETA.INV cells | any kernel change | T-2505 on | 1-4 ulp (decision 0111, owned by T-2633) |

How each kind is handled:
- **Cells within 4 ulp with no cell going visibly off.** These use decision 0083's standing allowance: the task is
  `campaign: visible`, its Outcome lists the cells, and T-2788-style follow-up ulp tasks name them.
- **The two visibly-off cells.** These are outside 0083's condition 2. The task whose branch moves them carries a
  decision record (`make new-decision`) naming exactly those cells and a follow-up task that owns them, as decisions
  0111 did. Without that record the branch stays delegated for F.
- **The chained BETA.INV cells.** These stay under decision 0111 and T-2633 until T-2511 solves the inverse to the
  crossing.

### Test fixtures
- Each task adds a Rust unit test of its branch on rows taken from the research data. The rows are copied into a
  compact fixture under `crates/rustsheet-engine/src/test/fixtures/` (`bratio_<branch>.fixture`), read with
  `include_str!` and `test::kernel_rows::fixture_rows` the way `log1p_kernel.fixture` and `ln_kernel.fixture` are.
  The test does not read `docs/research/` at run time; decision 0101 covers the research tables the rows come
  from.
- A fixture is a hex-bits TSV: a, b, x, y, the expected w or w1, and the branch name. It takes at most a few hundred
  rows per branch, drawn from the cells that branch's RESULTS.md scores (for example r2-bratio's w_bpser cells for
  T-2505).
- A dispatch test checks that `bratio()` names the same branch as `bratio_excel.py` on a sample of points in every
  branch region. That includes the x0 = 0.3 cut, the swap at x = 0.5 and at λ = 0, and b0 = 15, 40 and 100.
- The eval suites stay the gate: probes-numeric (t2464-*), gen-m9, probes and ironcalc-calc, with no regression
  outside the gated table above.

### Task map

| task | ladder step | module | branches it ports (and callers) |
|---|---|---|---|
| T-2502 (done) | E1 | incomplete_beta.rs | closed forms on the oriented arguments; F's argument forms |
| T-2645 (done) | E0 | x87_log1p.rs, x87_exp.rs | the primitives; the BINOM k = 0 shortcut |
| T-2503 (done) | E2 | beta_density.rs | the density for min <= 1, which L140 and bfrac need |
| T-2505 | E3 | bratio/mod.rs, bpser.rs, ln_beta.rs | dispatch and `Tails`; fpser, apser, w_bpser, w1_bpser, bpser(a,b>1) for BETA.DIST, F, BINOM, NEGBINOM and T; callers moved onto `bratio_or` |
| T-2506 | E4 | bratio/bup.rs | L140 bup+bpser (b0' <= 1 prefactor from T-2503's density) |
| T-2507 | E5 | loader.rs | the min > 1 density (not a branch; it feeds T-2508) |
| T-2508 | E6 | bratio/bfrac.rs | bfrac and basym (min(a,b) >= 40) |
| T-2509 | r2-tdist | bratio/bgrat.rs | bgrat at b = 1/2 in extended precision; T.DIST* through K(n/2, 1/2, y, z) for the branches T-2505/T-2506 ported |
| T-2510 | E7 | bratio/bgrat.rs | bup+bgrat, bgrat, L140 bup+bgrat, L140 bup+bup+bgrat for BETA.DIST, F, BINOM and NEGBINOM. T stays delegated in this family |
| T-2511 | E8 | inverse_beta.rs, fisher.rs, student_t.rs | BETA.INV, F.INV, F.INV.RT and T.INV* solve the ported CDF to the crossing; settles T-2633 |

After T-2511, a cleanup task deletes the old kernel's unused paths and `t2464_harness.rs`. It is filed when T-2511
lands.

## Alternatives considered
- **Port everything in one PR.** Rejected: -103 visible outside BETA.DIST (r2-impact finding 2), an L-size diff, and
  the gated cells all at once.
- **Port R's `toms708.c`.** Rejected: Excel's dispatch is the 1992 Fortran's (x0 >= 0.3, 284 against 87 on the
  straddling cells), and R's arithmetic is not Excel's.
- **Keep per-caller kernels** (T's composite, BINOM's pmf walk). Rejected: F, BINOM, NEGBINOM and T are one
  evaluation of one kernel (1,578/1,578 F pairs; 348/348 BINOM/NEGBINOM computed tails), and a running pmf sum
  matches 501 of 1,371.
- **A runtime switch per branch.** Rejected: compile-time flags keep the delegated path bit-identical, and a
  hidden configuration surface would let two builds compute different values.

## Test plan
- Per task: the branch fixture test, the dispatch test, and `make check` with no regressions outside the gated table.
- Each task reports its eval delta against r2-impact's "expected visibly-off change", for example E3 -632 pre-T2464.
  A shortfall of more than a quarter of that is a reason to stop and re-measure, not to merge.
- Properties tested on every ported branch: w + w1 = 1 to the stored complement; the swap symmetry K(a, b, x, y) =
  swap(K(b, a, y, x)) away from the x = 0.5 and λ = 0 ties; monotonicity in x on each fixture's sorted rows.
- Performance stays as today, with fixed per-call work. Reviewer-performance is off (decision 0081), so there is no
  benchmark budget.

## Rollout
In order: T-2505, T-2506, T-2507 (it can run in parallel with T-2505 and T-2506), T-2508, T-2509, T-2510, T-2511, then
cleanup. Every branch not yet ported stays on today's kernel through `bratio_or` until its task lands. Each task is
size M at most, and a task may split by caller when its delta is mixed.
