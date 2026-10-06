# 0034 — Allow six gen-m2 lookup regressions unmasked by T-0211

- **Status:** accepted (2026-10-06, T-0211)
- **Context:** T-0211 makes VLOOKUP/HLOOKUP search before validating the index, as Excel does
  (probes/lookup-index-order, excel-graph; docs/excel-semantics/lookup.md). That fixes 263 gen-m2
  cells. Six other cells passed before only by accident: each has an out-of-range index, Excel's
  approximate search *finds* the value there and then fails the index with `#REF!`, and our search
  (wrong on unsorted, mixed-type data; T-0212) finds nothing. The old code returned `#REF!` before
  searching, which happened to equal Excel's answer. Now the search runs first and returns `#N/A`.
  Hand trace of T-0212's emulated algorithm: A70 looks up -0.0001 in {blank,"?b",blank,-0.0001,…}
  and hits C4; A204 looks up 0.5 in {FALSE,2,0.2,100,blank,0.5} and hits the last row.
  The cells:
  `gen-m2/1b6ceed7489e0800` Calc!A193, `gen-m2/d0c29dd5c530e28c` Calc!A204,
  `gen-m2/dd99be1f2454159f` Calc!A369, `gen-m2/f7e6d351c8538ef5` Calc!A70,
  `gen-m2/f9d4df4efdfbcbd5` Calc!A45 and Calc!A283.
- **Options considered:**
  - *Keep validating first for these shapes.* That contradicts the oracle (an out-of-range
    index with a value that is not found is `#N/A`: probes/lookup-index-order H19–H21, H27, H41)
    and loses the 263 fixes.
  - *Do T-0212 in the same PR.* T-0212 is a separate size-M task that depends on this one.
  - *Record the six as known failures and give them to T-0212.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0211 PR,
  for exactly these six cells. They are added to T-0212's list of cells to fix.
- **Consequences:** `eval/baseline/gen-m2.txt` loses 263 lines and gains these six. Reviewers
  confirm the baseline diff adds no other line. T-0212 must make all six pass.
