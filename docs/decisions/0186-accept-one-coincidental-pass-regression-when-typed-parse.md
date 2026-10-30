# 0186 — Accept one coincidental-pass regression when typed parse errors are refused

- **Status:** accepted (2026-10-30, T-3213)
- **Context:** T-3213 refuses a typed formula that does not parse, as Excel does (Graph PATCH 400;
  probes/t3113-entered-refused, probes/t3213-typed-parse-errors). One baselined pass depended on the
  old behavior: probes/t3003-unqualified-structured-d2 S!F2, `=FORMULATEXT(D2)` where D2 is
  `=[@Col]*2` typed just right of table `T`. Excel grows the table and stores the formula. RustSheet
  cannot parse `[@Col]` outside a table, so before T-3213 it stored a parse error. That cell is
  `#ERROR!`, which was wrong (S!D2 is a known failure), but it kept the text, so FORMULATEXT matched
  by coincidence. Now the entry is refused, D2 is blank and F2 is `#N/A`. T-3274 owns the fix: the
  grown cell should parse inside the table it joins, which passes D2 and F2 together.
- **Decision:** `make eval-baseline` runs once with `--allow-regressions` for this one cell (S!F2),
  in T-3213's PR. No other regression is accepted.
- **Consequences:** probes/t3003-unqualified-structured-d2 S!F2 is a known failure until T-3274
  lands. T-3274's case list gains S!F2. The mirror case, S!K2 (`=TYPE(D2)`, 1), now passes by
  coincidence because a blank D2 also has TYPE 1; T-3274 notes it.
