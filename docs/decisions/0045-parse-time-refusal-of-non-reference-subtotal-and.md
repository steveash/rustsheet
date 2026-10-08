# 0045 — Parse-time refusal of non-reference SUBTOTAL and AREAS arguments

- **Status:** accepted (2026-10-08, T-1163)
- **Context:** Excel for the web refuses a workbook (createSession 501 `unsupportedWorkbook`,
  `FileCorruptTryRepair`) holding SUBTOTAL with an argument after the function number that is
  not a reference operand (`{1,2}`, `A1:A2*1`, `1`, `SUM(..)`, an empty argument, a nested
  SUBTOTAL), or `AREAS(1)`. These are per-formula scratch probes (T-1063, T-1163). The forms the
  intersection accepts as operands all load (probes/subtotal-reference-operands). The engine
  already refuses Excel-refused shapes at parse time on the same kind of web evidence: `(1)(2)`
  (T-0149), non-reference union and intersection operands (T-0161, T-0193), and the `:` operands
  of T-1117.
- **Decision:** `Parser::function_call` (parser/reference_operators.rs) is where the parser
  checks reference-only arguments. It turns a SUBTOTAL argument after the first, or an AREAS
  argument, that fails `is_reference_operand` (or is empty) into a parse error. The cell shows
  `#ERROR!`, as `(1)(2)` does. Another function joins only with oracle evidence that Excel refuses
  its non-reference arguments. A reference operand that evaluates to a value is `#VALUE!` at run
  time, and an array result spills `#VALUE!` per element. Both are probed.
- **Consequences:** an imported workbook holding a refused form now gets `#ERROR!` where it used to
  get a computed value. A nested SUBTOTAL argument used to be skipped. Excel for the web will not
  open such a file at all, and none of the upstream test workbooks hold one (all their SUBTOTAL
  and AREAS arguments are references). Desktop Excel's behavior is UNVERIFIED: typed entry and
  desktop loading of these formulas are M14 questions (decision 0039), filed as a follow-up task.
  If desktop evidence shows Excel loads and evaluates a form, the check narrows for that form.
