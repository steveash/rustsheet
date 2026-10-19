# 0080 — LAMBDA calls nest within a stack budget

- **Status:** accepted (2026-10-19, T-0439)
- **Context:** A LAMBDA that calls itself through a parameter
  (`LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,20000))`) overflowed the stack and aborted the process
  on `main` on a default-sized thread stack (the eval harness runs on 256 MB), and T-0439's closures add a second way in (a chain of 20,000 closures made by REDUCE).
  Cells and defined names already stop at the evaluation's stack budget; LAMBDA calls did not.
  Excel evaluates both forms 300 deep (probes/lambda-call-depth, excel-graph); deeper workbooks made
  Excel for the web's createSession fail, so its own limit is unknown.
- **Decision:** Nested LAMBDA calls may use four times the evaluation's stack budget
  (`set_max_evaluation_stack`), measured from the cell's base; past it a call is `#NUM!`. This is a
  directive-5 DEVIATION (decision 0014): docs/excel-semantics/arrays.md "How deep LAMBDA calls nest".
  The 300-level probe cells it changes (probes/lambda-call-depth H3, H6, and I3, I6 that read them)
  are listed in `eval/suites/probes/deviations` (decision 0036).
- **Consequences:** With the defaults about 200 levels fit in a release build and 50 in a debug build.
  The default needs about 2 MB of host stack: an embedder with a larger stack may raise the limit with
  `set_max_evaluation_stack`, and one with a smaller stack (musl threads, small workers) must lower it. Raising the default
  so that 300 levels fit turns the listed cells into passes; the PR doing so removes their lines.
