# 0185 — A call at the head of a group shares its depth level; unpin the paren-abs cases

- **Status:** accepted (2026-10-30, T-3197)
- **Context:** Excel loads 64 and 65 levels of `(ABS(` (`=(ABS((ABS(...))))`), stored and typed
  (probes/t3097-nesting-paren-abs-64 and -65; docs/excel-semantics/syntax.md "Formula nesting depth
  cap"). RustSheet refused them as "Formula is nested too deeply": the group and the call each cost
  an operand's two levels of `MAX_FORMULA_DEPTH`, four per level, 260 for 65 levels. Decision 0168
  pinned the four cases whole (T-3165, T-3189) until a parser change (T-3197) landed.
- **Decision:** A name, reference, boolean or call written directly after a group's `(` is parsed
  ahead of the group's content and handed to it as the pending operand (T-3191's mechanism,
  `Parser::parse_group_head`). It costs one level, the group's spare one (a group costs two levels
  for one tree node), so the bound still covers the tree's real depth. Parsing it ahead also keeps
  each level to one pass through the precedence chain, which is what makes the cheaper cost fit
  the stack: with the cost cut alone, about 78 levels overflowed a 2 MiB debug stack. A sign, `@`,
  whitespace or another group first in a group still costs two.
- **Consequences:** The four t3097-nesting-paren-abs lines leave `eval/suites/probes/deviations`;
  those cases now match Excel. The parser accepts up to 84 levels of `(ABS(` where nothing counts
  calls; at-limit tests cover that on a 1 MiB release stack and a 2 MiB debug stack. The pending
  operand records where its text starts, so an intersection read back as a `#NAME?` name (U+3000
  separator) keeps its whole text.
