# 0172 — Pin T-3172's twenty paren and sign-run cases whole under decision 0168

- **Status:** accepted (2026-10-30, T-3189)
- **Context:** T-3172 (excel-graph 2026-10-30) probed Excel's caps on bare parentheses and unary
  sign runs: Excel accepts 256 nested parentheses around `1`, 1,024 minus signs before `1`, and
  1,023 plus signs before `"a"` or 1,023 alternating `-+` signs before a reference, stored and
  typed alike (docs/excel-semantics/syntax.md "Formula nesting depth cap"). RustSheet refuses all
  twenty committed cases (`probes/t3172-paren-256`, `-minus-255`/`-256`/`-1024`,
  `-plus-255`/`-256`/`-1023`, `-alt-255`/`-256`/`-1023`, stored and typed) as a whole with
  "Formula is nested too deeply", its directive-5 depth bound (`MAX_FORMULA_DEPTH`, Status:
  DEVIATION, decision 0014). They sat unowned on the M2 exit row. This is the same shape T-3165
  met for T-3097's paren cases and settled with decision 0168.
- **Decision:** Keep the bound and pin each of the twenty cases whole in
  `eval/suites/probes/deviations` with a decision-0168 line (`*` cell, decision 0014, the
  "Formula nesting depth cap" DEVIATION, `refused "Formula is nested too deeply"`). Each case
  holds one formula that meets the limit, as decision 0168 asks.
- **Consequences:** The twenty cases leave the probes known failures as passes with 40 deviation
  cells, and `make exit-owners MILESTONE=M2` no longer lists them. Lifting the bound is follow-up
  work: T-3193 for unary sign runs, T-3191 for parentheses (with t3172-paren-256). Once either
  lands, the matching lines read stale and leave the file in that PR.
