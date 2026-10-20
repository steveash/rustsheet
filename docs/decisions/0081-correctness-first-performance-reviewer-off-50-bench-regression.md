# 0081 — Correctness first: performance reviewer off, 50% bench regression budget

- **Status:** accepted (2026-10-20, owner instruction, T-2107). It supersedes the "within 5%" bench
  rule in `.claude/agents/reviewer-performance.md` and in task and spec acceptance criteria for as
  long as this decision stands; decision 0077 (T-1785) was a one-off case of the same trade.
- **Context:** Several PRs were held by the performance reviewer or by a "within 5%" bench criterion
  although their results were correct and every other reviewer approved: T-1785 (decision 0077),
  T-1775 (#1164, LEFT +3-10% and CONCATENATE +6-10% wall-clock on `benches/text.rs`), T-1829 (#1153)
  and the SPEC-0013 spec T-1898 (#1167), which hit the 3-round review limit on performance alone.
  The project is still reaching Excel parity; the owner's direction (2026-10-20) is to get the
  semantics right first and optimize afterwards.
- **Decision:**
  1. The `review` skill does not launch `reviewer-performance`. A PR merges when the security,
     architecture, quality and (where it applies) Excel-fidelity reviewers approve and CI is green.
  2. A benchmark criterion in a task or spec ("within 5%", "no slower"; e.g. the tables in SPEC-0009 and SPEC-0011) is read as **no more than
     50% slower** than main on the named bench. Beyond 50% needs a decision record. Measuring is
     optional; a task that measures records the numbers in its Outcome.
  3. Directive 5 is unchanged: no unbounded recursion, allocation or work on untrusted input. That is
     a safety rule, and the security reviewer keeps enforcing it (resource caps, `LiftCost`,
     cancellation), as do the fuzzers. A quadratic or unbounded regression on untrusted input is
     not a bench-percentage question: it still blocks under the security review.
  4. PRs held only by a performance review or a bench criterion when this lands are re-judged under
     it: T-1898 (#1167), T-1775 (#1164), T-1829 (#1153).
  5. Optimization tasks (lane `perf`, e.g. T-1932) stay filed and are picked after correctness work.
- **Consequences:** fewer review rounds and no held correctness work. Performance debt can accrue
  up to the 50% budget per change, so the optimization pass that ends this phase starts with a
  bench sweep against the numbers recorded in Outcomes. Re-enabling the performance reviewer (no
  later than the M3 benchmark regression gate in `docs/process/quality-gates.md`) takes a decision
  record that supersedes this one.
