---
id: T-3124
title: "Probe the M9 engineering argument kinds no Excel cell covers yet: empty text, an error or zero in 44 functions (BESSEL, BIT, base conversions, ERF, IM*)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M9 engineering functions below has at least one Excel-backed cell for each argument kind the report's argument coverage expects, so the engineering functions are verified on empty text, error and zero inputs no suite has tried.

## Context
Gap triage 2026-10-29 (`make eval` on 6793e919 with calipers, corpus-bench and poi fetched; report.json
`argument_coverage`, decision 0031 evidence gaps; decision 0094 asks for each argument mode to be covered). Every
visibly-off cell of that run has an open owner (`scripts/exit_owners.py --all-off`), and the evidence floor is met, so
the M9 argument kinds are the largest evidence gap left: 245 M9 functions have a value parameter with a kind no Excel
cell passes (M2 was closed by T-2861; M5 to M7 are T-2862, T-2863, T-2864, T-2865 and T-2871 to T-2878; M9 had none).
The kinds are those of `crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative,
fraction, numeric_text, text, empty_text, boolean, error, blank, range. A kind is seen only when an Excel-evidence cell
passes it as a literal or a single-cell reference: nested calls such as `NA()` are not classified (docs/progress.md
"Argument coverage"), so use literal errors (`#N/A`, `#DIV/0!`) and cells holding them. T-2866 taught the generator to
draw `""` and error literals, but at 3% per parameter the existing gen-m9 cases predate it and do not reach these pairs.
`array`-mode parameters are left out: their coverage is not classified by kind.
The engineering gaps are mostly empty text and an error in the second argument (BESSEL n, BIT shift and number2, base-conversion places) and zero as an IM* inumber; the 12 D-function `field` gaps are T-3129's.
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- CONVERT.number: an error.
- DEC2HEX.number: empty text, an error; DEC2HEX.places: empty text, an error, a blank cell.
- BESSELI.x: empty text; BESSELI.n: empty text, an error.
- BESSELJ.x: empty text; BESSELJ.n: empty text, an error.
- BESSELK.x: empty text; BESSELK.n: empty text, an error.
- BESSELY.x: empty text; BESSELY.n: empty text, an error.
- BIN2HEX.number: an error; BIN2HEX.places: empty text, an error.
- BIN2OCT.places: empty text, an error.
- BITAND.number1: empty text; BITAND.number2: empty text, an error.
- BITLSHIFT.number: empty text; BITLSHIFT.shift_amount: empty text, an error.
- BITOR.number1: empty text; BITOR.number2: empty text, an error.
- BITRSHIFT.number: empty text; BITRSHIFT.shift_amount: empty text, an error.
- BITXOR.number1: empty text; BITXOR.number2: empty text, an error.
- COMPLEX.real_num: empty text; COMPLEX.i_num: numeric text, empty text, an error; COMPLEX.suffix: an error.
- DEC2BIN.number: empty text, an error; DEC2BIN.places: empty text, a boolean, an error.
- DEC2OCT.number: empty text, an error; DEC2OCT.places: empty text, an error.
- DELTA.number1: empty text; DELTA.number2: empty text, an error.
- ERF.lower_limit: empty text; ERF.upper_limit: empty text, a boolean, an error.
- ERF.PRECISE.x: empty text.
- ERFC.x: empty text.
- ERFC.PRECISE.x: empty text.
- GESTEP.number: empty text; GESTEP.step: empty text, an error.
- HEX2BIN.number: zero, an error; HEX2BIN.places: empty text, an error.
- HEX2OCT.number: zero, an error; HEX2OCT.places: empty text, an error.
- IMAGINARY.inumber: zero, a blank cell.
- IMCOS.inumber: zero.
- IMCOSH.inumber: zero.
- IMCOT.inumber: zero.
- IMCSC.inumber: zero.
- IMCSCH.inumber: zero.
- IMDIV.inumber1: an error; IMDIV.inumber2: zero, an error.
- IMEXP.inumber: zero.
- IMLN.inumber: zero.
- IMLOG10.inumber: zero.
- IMPOWER.inumber: zero, an error; IMPOWER.number: empty text, an error.
- IMREAL.inumber: zero.
- IMSEC.inumber: zero.
- IMSECH.inumber: zero.
- IMSIN.inumber: zero.
- IMSINH.inumber: zero.
- IMSUB.inumber1: an error; IMSUB.inumber2: an error.
- OCT2BIN.number: an error; OCT2BIN.places: empty text, an error.
- OCT2DEC.number: an error.
- OCT2HEX.number: an error; OCT2HEX.places: empty text, an error.
Write the probe case probes/m9-eng-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-eng-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/engineering.md (or coercion.md), citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The D-functions (T-3129); M9 financial (T-3125, T-3126) and statistical (T-3127, T-3128, T-3129) functions.
- `array`-mode parameters (IMSUM, IMPRODUCT).
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
