# 0042 — An excel-graph 0 scores a kept subnormal as a pass

- **Status:** accepted (2026-10-08, T-1027)
- **Context:**
  - Excel keeps a subnormal result from `+`, `-`, SUM, SUBTOTAL(9,..), EXP, MIN, ABS and SIN.
    For example, `(3E-308-2.5E-308)=0` is FALSE (docs/excel-semantics/numbers.md "Subnormal
    results become 0").
  - The Microsoft Graph API reports such a cell's value as 0.0. So the excel-graph expectation for
    probes/subnormal-function-results A4, A6, A60-A62, A65, A69 and A73-A75 is 0 whether Excel
    holds a true 0 or a kept subnormal. The `C` companions (`(x)=0`) are what tell the two apart.
  - Once the engine keeps the subnormal as Excel does (T-1027), those numeric cells would fail
    against an expectation that cannot see the difference. Editing the oracle sidecars or the
    baseline by hand is forbidden.
- **Options considered:**
  - *(a) Keep them as known failures.* That records a correct engine as failing forever, and M1
    asks for 100% bit-exact probes. Rejected.
  - *(b) Drop the cells from scoring*, by adding a `check` list or editing the case. This loses
    the true-0 checks the same cells make for other inputs, and it edits a case to meet an exit
    number. Rejected.
  - *(c) In the harness, score an excel-graph expectation of exactly 0 against a subnormal
    engine value as a pass.* Graph cannot distinguish these two values, so the rule discards no
    information Graph provides. Chosen.
- **Decision:**
  - `runner.rs` `graph_hides_subnormal`: the cell scores as a pass only when all of these hold:
    - its expectation came from an `excel-graph` oracle sidecar;
    - the expectation is exactly 0;
    - the engine's value is a non-zero number below the smallest normal double.
  - Cached values and `excel-desktop` expectations are untouched, and so is every other value.
  - Whether a result is flushed or kept stays evidenced by `(x)=0` companion cells. Every probe
    of a subnormal result must keep one.
- **Consequences:**
  - A wrongly kept subnormal, where Excel flushes, is no longer caught by the numeric cell, only
    by its companion. The probes that pin the flush (subnormal-function-results,
    subnormal-flush-edges, subnormal-sumsq-sumproduct-edges) all have companions.
  - If an oracle backend that reports subnormals ever lands, this rule is not applied to it.
