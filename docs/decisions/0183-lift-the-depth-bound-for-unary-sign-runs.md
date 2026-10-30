# 0183 — Lift the depth bound for unary sign runs; cap open signs at 1,023

- **Status:** accepted (2026-10-30, T-3193)
- **Context:** Excel loads 1,024 unary minus signs before `1`, 1,023 plus signs before `"a"` and
  1,023 alternating `-+` signs before `C1`, and refuses one more of each, stored and typed
  (probes/t3172-minus-*, -plus-*, -alt-*; docs/excel-semantics/syntax.md "Formula nesting depth
  cap"). RustSheet refused every run of about 250 signs or more as "Formula is nested too deeply",
  because each sign node cost a level of `MAX_FORMULA_DEPTH`. Decision 0168 pinned the eighteen
  cases whole (T-3189) until a parser change (T-3193) landed.
- **Decision:** A unary sign costs no depth level. Instead, at most `MAX_SIGNS` (1,023) signs may be
  open on one parse path, and a minus written directly before a number literal is not counted
  (`Parser::parse_signs`). That reproduces all three probed caps. Signs in nested runs
  (`=-(-(...))`) count together, which is the conservative reading because Excel was probed with
  unbroken runs only (T-3266). The evaluator already walked a sign run in a loop. The printer
  now prints signs, written parentheses and whitespace in a loop too (`stringify/wrapped.rs`), so
  the remaining per-sign recursion (Clone, Drop, analysis, edits) is bounded by `MAX_SIGNS`.
- **Consequences:** The eighteen t3172 sign-run lines leave `eval/suites/probes/deviations`; those
  cases now match Excel. At-limit tests on a 1 MiB release stack and a 2 MiB debug stack cover a
  1,023-sign run (bare, spaced, line-fed) under the deepest nesting, and the refusal of one more.
  The measured headroom is about 2x (the derived `Clone` overflows at 512 KiB). T-3253 owns
  widening it, and any new recursive walker must be measured against these tests.
