# 0184 — A call at the head of a group costs less parser depth

- **Status:** accepted (2026-10-30, T-3197)
- **Context:** Excel loads 64 and 65 levels of `(ABS(` (`=(ABS((ABS(...1...))))`), stored and
  typed (probes/t3097-nesting-paren-abs-64, -65; docs/excel-semantics/syntax.md "Formula nesting
  depth cap"). RustSheet refused them as "Formula is nested too deeply": a group and the call in it
  each reserved an operand's two levels of `MAX_FORMULA_DEPTH`, four per level, so 63 fit. Decision
  0168 pinned the four cases whole in `eval/suites/probes/deviations`; decision 0173 left them to
  T-3197.
- **Decision:** `Parser::parse_primary` bounds its own recursion in half levels (`MAX_NESTING`, twice
  `MAX_FORMULA_DEPTH`). The first operand of a group's content reserves three half levels, not four,
  unless it is a group itself, and there a plain function call adds one level to the tree depth, not
  two. A level of `(ABS(` costs seven half levels, so 72 fit and 73 are too deep; every other shape
  keeps its bound. Measured: 72 levels of `SUM((` parse, evaluate, print and survive a row insert
  and a sheet rename in about 1.9 MiB of stack in an unoptimized build, as 127 levels of `ABS(` do.
- **Consequences:** The four t3097-nesting-paren-abs lines leave `eval/suites/probes/deviations`;
  those cases match Excel. The at-limit tests on a 1 MiB optimized stack and a 2 MiB test thread
  cover 65, 72 and 73 levels of `(ABS(`, `SUM((` and their spaced form.
